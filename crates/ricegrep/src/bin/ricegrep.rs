//! Minimal RiceGrep CLI entrypoint.
//!
//! This CLI is a thin client that forwards search requests to a running
//! RiceGrep CLI. Commands are implemented for incremental expansion.

use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode, Stdio};
use std::time::{Duration, Instant};

use dirs;

use anyhow::{Context, Result};
use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser, Subcommand};
use ignore::WalkBuilder;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use ricegrep::admin::{AdminAction, AdminCommandRequest, AdminToolset};
use ricegrep::api::models::{
    HealthStatus, RankingConfig, SearchFilters, SearchRequest, SearchResponse, SearchResult,
};
use ricegrep::chunking::{ChunkMetadata, LanguageDetector, LanguageKind};
use ricegrep::lexical::{
    Bm25IndexHandle, LexicalHit, LexicalSearcher, SearchFilters as LexicalFilters,
};
use ricegrep::metadata::{ChunkMetadataView, MetadataStore};
use std::io::Write;
use tempfile::tempdir;
use uuid::Uuid;
use walkdir::WalkDir;

const DEFAULT_ENDPOINT: &str = "http://localhost:3000";
const DEFAULT_WATCH_DEBOUNCE_SECONDS: u64 = 2;
const DEFAULT_LOCAL_LIMIT: usize = 50;
const EXIT_UNIMPLEMENTED: u8 = 2;
const STATE_DIR_NAME: &str = ".ricecoder";
const STORE_DIR_NAME: &str = ".ricegrep";
const INDEX_DIR_NAME: &str = "index";
const METADATA_FILE_NAME: &str = "metadata.bin";

mod installer;
mod mcp;

use installer::{InstallArgs, UninstallArgs};
use mcp::McpArgs;

#[derive(Parser, Debug)]
#[command(name = "ricegrep")]
#[command(about = "AI-enhanced code search tool with ripgrep compatibility")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Base URL for the RiceGrep server API (requires --server)
    #[arg(long, default_value = DEFAULT_ENDPOINT, global = true)]
    endpoint: String,

    /// Enable server mode for search and indexing
    #[arg(long, global = true)]
    server: bool,

    /// Output results as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Suppress progress output
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    /// Configuration root directory
    #[arg(long, global = true)]
    config_root: Option<String>,

    /// Legacy pattern without subcommand
    #[arg(value_name = "PATTERN")]
    pattern: Option<String>,

    /// Path arguments
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<String>,

    #[command(flatten)]
    legacy_flags: SearchFlags,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Search for patterns using local index or server mode
    Search(SearchArgs),

    /// Ripgrep-compatible search alias
    #[command(alias = "rg")]
    Ripgrep(SearchArgs),

    /// Watch mode for continuous indexing
    Watch(WatchArgs),

    /// Manage search indexes
    Index(IndexArgs),

    /// Replace symbols in a file
    Replace(ReplaceArgs),

    /// Search file and folder names
    Files(FilesArgs),

    /// Plugin management (stub)
    Plugin,

    /// Start MCP stdio server
    Mcp(McpArgs),

    /// Export skill definitions
    ExportSkills(ExportSkillsArgs),

    /// Install assistant integration
    Install(InstallArgs),

    /// Uninstall assistant integration
    Uninstall(UninstallArgs),

    /// Check RiceGrep server health
    Health,

    /// Run local end-to-end system checks
    E2e(E2eArgs),

    /// Process orchestration (stub)
    Process,
}

#[derive(Parser, Debug)]
struct WatchArgs {
    /// Paths to watch
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<String>,

    /// Watch timeout in seconds
    #[arg(long)]
    timeout: Option<u64>,

    /// Debounce window for file changes in seconds
    #[arg(long, default_value_t = DEFAULT_WATCH_DEBOUNCE_SECONDS)]
    debounce_secs: u64,

    /// Clear screen between updates
    #[arg(long)]
    clear_screen: bool,
}

#[derive(Parser, Debug)]
struct IndexArgs {
    #[command(subcommand)]
    command: IndexCommand,
}

#[derive(Subcommand, Debug)]
enum IndexCommand {
    /// Build a search index
    Build(IndexPathArgs),

    /// Update an existing search index
    Update(IndexPathArgs),

    /// Clear cached index data
    Clear,

    /// Show index status
    Status,
}

#[derive(Parser, Debug)]
struct IndexPathArgs {
    /// Paths to index
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<String>,

    /// Don't respect .gitignore, .ignore, and other ignore files
    #[arg(long)]
    no_ignore: bool,
}

#[derive(Parser, Debug)]
struct ReplaceArgs {
    /// Symbol or text to replace
    old_symbol: String,

    /// Replacement value
    new_symbol: String,

    /// File containing the symbol
    file_path: PathBuf,

    /// Show preview without applying changes
    #[arg(long)]
    preview: bool,

    /// Apply changes without prompting
    #[arg(long)]
    force: bool,

    /// Show what would change without writing
    #[arg(long)]
    dry_run: bool,
}

#[derive(Parser, Debug)]
struct SearchArgs {
    /// Query string
    query: String,

    /// Optional search paths
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<String>,

    /// Maximum number of results
    #[arg(long = "max-count", short = 'm', alias = "limit")]
    max_count: Option<usize>,

    /// Filter by repository id
    #[arg(long)]
    repository_id: Option<u32>,

    /// Filter by language
    #[arg(long = "language", short = 't', alias = "type")]
    language: Option<String>,

    /// Filter by file path pattern
    #[arg(long = "file-path-pattern", short = 'g', alias = "glob")]
    file_path_pattern: Option<String>,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    #[command(flatten)]
    flags: SearchFlags,
}

#[derive(Parser, Debug, Default, Clone)]
struct SearchFlags {
    /// Case-insensitive search
    #[arg(long, short = 'i')]
    ignore_case: bool,

    /// Force case-sensitive search
    #[arg(long)]
    case_sensitive: bool,

    /// Match whole words only
    #[arg(long, short = 'w')]
    word_regexp: bool,

    /// Treat the pattern as a literal string
    #[arg(long, short = 'F')]
    fixed_strings: bool,

    /// Print line numbers
    #[arg(long, short = 'n')]
    line_number: bool,

    /// Do not print line numbers
    #[arg(long)]
    no_line_number: bool,

    /// Count matches per file
    #[arg(long, short = 'c')]
    count: bool,

    /// Show matching file paths only
    #[arg(long, short = 'l')]
    files_with_matches: bool,

    /// Show non-matching file paths only (not supported)
    #[arg(long, short = 'L')]
    files_without_match: bool,

    /// List searched files (not supported)
    #[arg(long)]
    files: bool,

    /// Context lines before match
    #[arg(long, short = 'B')]
    before_context: Option<usize>,

    /// Context lines after match
    #[arg(long, short = 'A')]
    after_context: Option<usize>,

    /// Context lines before and after match
    #[arg(long, short = 'C')]
    context: Option<usize>,

    /// Answer generation (not supported by server mode)
    #[arg(long)]
    answer: bool,

    /// Natural language query parsing (not supported by server mode)
    #[arg(long)]
    natural_language: bool,

    /// AI-enhanced search (not supported by server mode)
    #[arg(long)]
    ai_enhanced: bool,

    /// Show full content (not supported by server mode)
    #[arg(long)]
    content: bool,

    /// Disable semantic reranking
    #[arg(long)]
    no_rerank: bool,

    /// Replace matches with new value (use replace command)
    #[arg(long)]
    replace: Option<String>,

    /// Preview replacements (use replace command)
    #[arg(long)]
    preview: bool,
}

#[derive(Parser, Debug)]
struct ExportSkillsArgs {
    /// Output format (json or yaml)
    #[arg(long, default_value = "json")]
    format: String,
}

#[derive(Parser, Debug)]
struct FilesArgs {
    /// Glob pattern to match file or folder names
    pattern: String,

    /// Paths to search
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<String>,

    /// Match on full path instead of file name
    #[arg(long)]
    full_path: bool,

    /// Include directories in results
    #[arg(long)]
    include_dirs: bool,

    /// Case-insensitive matching
    #[arg(long, short = 'i')]
    ignore_case: bool,
}

#[derive(Parser, Debug)]
struct E2eArgs {
    /// Source workspace to copy for testing
    #[arg(long, value_name = "PATH", default_value = "projects/ricecoder")]
    source: PathBuf,

    /// Write JSON report to this path
    #[arg(long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Override config root for install/uninstall tests
    #[arg(long, value_name = "PATH")]
    config_root: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
struct SkillDefinition {
    name: String,
    description: String,
    command: String,
}

#[derive(Debug, serde::Deserialize)]
struct CliConfig {
    endpoint: String,
    server_enabled: bool,
    json: bool,
    quiet: bool,
    ai_enabled: Option<bool>,
    color: Option<String>,
    config_root: std::path::PathBuf,
}

#[derive(Debug, Clone)]
struct RuntimeConfig {
    endpoint: String,
    server_enabled: bool,
    json: bool,
    quiet: bool,
    config_root: std::path::PathBuf,
}

impl RuntimeConfig {
    fn resolve(matches: &ArgMatches, cli: &Cli, config: CliConfig) -> Self {
        let endpoint = match matches.value_source("endpoint") {
            Some(ValueSource::CommandLine) => cli.endpoint.clone(),
            _ => config.endpoint,
        };
        let server_enabled = match matches.value_source("server") {
            Some(ValueSource::CommandLine) => cli.server,
            _ => config.server_enabled,
        };
        let json = match matches.value_source("json") {
            Some(ValueSource::CommandLine) => cli.json,
            _ => config.json,
        };
        let quiet = match matches.value_source("quiet") {
            Some(ValueSource::CommandLine) => cli.quiet,
            _ => config.quiet,
        };
        let config_root = cli
            .config_root
            .as_ref()
            .map(|s| std::path::PathBuf::from(s))
            .unwrap_or(config.config_root);

        Self {
            endpoint,
            server_enabled,
            json,
            quiet,
            config_root,
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = run().await {
        eprintln!("{err}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<()> {
    let config = load_config()?;
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;
    let runtime = RuntimeConfig::resolve(&matches, &cli, config);

    match cli.command {
        Some(Command::Search(args)) => run_search(&runtime, args).await,
        Some(Command::Ripgrep(args)) => run_search(&runtime, args).await,
        Some(Command::Watch(args)) => run_watch(&runtime, args).await,
        Some(Command::Index(args)) => run_index(&runtime, args).await,
        Some(Command::Replace(args)) => run_replace(&runtime, args),
        Some(Command::Files(args)) => run_files(args),
        Some(Command::Plugin) => unimplemented_command("plugin"),
        Some(Command::Mcp(args)) => mcp::run_mcp(&runtime, args).await,
        Some(Command::ExportSkills(args)) => run_export_skills(args),
        Some(Command::Install(args)) => installer::run_install(args).await,
        Some(Command::Uninstall(args)) => installer::run_uninstall(args).await,
        Some(Command::Health) => run_health(&runtime).await,
        Some(Command::E2e(args)) => run_e2e(args).await,
        Some(Command::Process) => unimplemented_command("process"),
        None => match cli.pattern {
            Some(pattern) => {
                let args = SearchArgs {
                    query: pattern,
                    paths: cli.paths.clone(),
                    max_count: None,
                    repository_id: None,
                    language: None,
                    file_path_pattern: None,
                    json: runtime.json,
                    flags: cli.legacy_flags.clone(),
                };
                run_search(&runtime, args).await
            }
            None => {
                Cli::command().print_help()?;
                Ok(())
            }
        },
    }
}

fn load_config() -> Result<CliConfig> {
    // For now, return defaults
    Ok(CliConfig {
        endpoint: DEFAULT_ENDPOINT.to_string(),
        server_enabled: false,
        json: false,
        quiet: false,
        ai_enabled: None,
        color: None,
        config_root: dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join(".ricegrep"),
    })
}

fn print_json(value: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(feature = "server")]
async fn server_search_request(
    runtime: &RuntimeConfig,
    request: SearchRequest,
) -> Result<SearchResponse> {
    let url = format!("{}/search", runtime.endpoint.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("failed to send server search request")?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "server search request failed: {}",
            response.status()
        ));
    }
    let search_response: SearchResponse = response
        .json()
        .await
        .context("failed to parse server search response")?;
    Ok(search_response)
}

#[cfg(not(feature = "server"))]
async fn server_search_request(
    _runtime: &RuntimeConfig,
    _request: SearchRequest,
) -> Result<SearchResponse> {
    Err(anyhow::anyhow!(
        "Server mode is disabled. Rebuild with --features server to enable it."
    ))
}

fn collect_glob_matches(
    root: &str,
    pattern: &str,
    include_dirs: bool,
    ignore_case: bool,
) -> Result<(Vec<String>, bool)> {
    use glob::Pattern;
    use std::time::SystemTime;
    
    // Gap #1 FIX: Use real glob pattern matching, not substring matching
    let glob_pattern = if ignore_case {
        Pattern::new(&pattern.to_lowercase())?
    } else {
        Pattern::new(pattern)?
    };
    
    // Gap #2 FIX: Hard limit at 100 files (matches OpenCode behavior)
    const LIMIT: usize = 100;
    let mut file_results = Vec::new();
    let mut truncated = false;
    
    let walker = WalkBuilder::new(root).build();
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        
        // Get the full path string for glob matching
        let path_str = path.to_string_lossy();
        
        // Match against the full path using glob semantics
        let matches_pattern = if ignore_case {
            glob_pattern.matches(&path_str.to_lowercase())
        } else {
            glob_pattern.matches(&path_str)
        };
        
        if matches_pattern && (include_dirs || path.is_file()) {
            // Gap #3 FIX: Get mtime for sorting
            let mtime = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            
            let full_path = std::path::Path::new(root)
                .join(path)
                .canonicalize()
                .unwrap_or_else(|_| path.to_path_buf());
            
            file_results.push((full_path.display().to_string(), mtime));
            
            // Gap #2 FIX: Stop at limit and mark as truncated
            if file_results.len() >= LIMIT {
                truncated = true;
                break;
            }
        }
    }
    
    // Gap #3 FIX: Sort by mtime descending (newest first)
    file_results.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Extract paths only, return with truncation flag
    Ok((file_results.into_iter().map(|(path, _)| path).collect(), truncated))
}

fn list_directory_entries(
    root: &str,
    pattern: Option<&str>,
    ignore_case: bool,
) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let matches = if let Some(pat) = pattern {
            if ignore_case {
                name.to_lowercase().contains(&pat.to_lowercase())
            } else {
                name.contains(pat)
            }
        } else {
            true
        };
        if matches {
            entries.push(format!(
                "{} {}",
                if path.is_dir() { "d" } else { "f" },
                path.display()
            ));
        }
    }
    Ok(entries)
}

fn format_file_content(path: &str, content: &str, offset: usize, limit: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = offset;
    let end = (start + limit).min(total_lines);

    let mut output = String::new();
    output.push_str("<file>\n");

    for (idx, line) in lines.iter().enumerate().skip(start).take(end - start) {
        let line_num = idx + 1;
        // Truncate lines longer than 2000 characters
        let formatted_line = if line.len() > 2000 {
            format!("{}...(line truncated)", &line[..2000])
        } else {
            line.to_string()
        };
        output.push_str(&format!("{:05}| {}\n", line_num, formatted_line));
    }

    if end < total_lines {
        output.push_str(&format!(
            "(File has more lines - total {} lines)\n",
            total_lines
        ));
    } else {
        output.push_str(&format!("(End of file - total {} lines)\n", total_lines));
    }

    output.push_str("</file>");
    output
}

fn read_file_numbered(path: &str, offset: Option<usize>, limit: Option<usize>) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let offset_val = offset.unwrap_or(0);
    let limit_val = limit.unwrap_or(2000); // Default limit of 2000 lines

    Ok(format_file_content(path, &content, offset_val, limit_val))
}

fn unimplemented_command(name: &str) -> Result<()> {
    eprintln!("Command '{name}' is not yet implemented in the minimal CLI.");
    std::process::exit(i32::from(EXIT_UNIMPLEMENTED));
}

async fn run_health(runtime: &RuntimeConfig) -> Result<()> {
    if !runtime.server_enabled {
        return Err(anyhow::anyhow!(
            "Server mode is disabled. Use --server to enable remote health checks."
        ));
    }
    let status = server_health_request(runtime).await?;
    if runtime.json {
        print_json(&status)?;
    } else {
        println!("Server health: {}", status);
    }
    Ok(())
}

#[cfg(feature = "server")]
async fn server_health_request(runtime: &RuntimeConfig) -> Result<serde_json::Value> {
    let endpoint = &runtime.endpoint;
    let url = format!("{}/health", endpoint.trim_end_matches('/'));

    match reqwest::Client::new().get(&url).send().await {
        Ok(response) if response.status().is_success() => response
            .json()
            .await
            .context("failed to parse health response"),
        Ok(response) => Ok(serde_json::json!({
            "healthy": false,
            "status": response.status().as_u16()
        })),
        Err(err) => Ok(serde_json::json!({
            "healthy": false,
            "error": err.to_string()
        })),
    }
}

#[cfg(not(feature = "server"))]
async fn server_health_request(_runtime: &RuntimeConfig) -> Result<serde_json::Value> {
    Err(anyhow::anyhow!(
        "Server mode is disabled. Rebuild with --features server to enable it."
    ))
}

async fn run_search(runtime: &RuntimeConfig, args: SearchArgs) -> Result<()> {
    let response = if runtime.server_enabled {
        let filters = args.file_path_pattern.clone().map(|pattern| SearchFilters {
            repository_id: args.repository_id,
            language: args.language.clone(),
            file_path_pattern: Some(pattern),
        });
        let request = SearchRequest {
            query: args.query.clone(),
            limit: args.max_count,
            filters,
            ranking: None,
            timeout_ms: None,
        };
        server_search_request(runtime, request).await?
    } else {
        run_local_search(&args)?
    };

    if runtime.json {
        print_json(&serde_json::to_value(&response)?)?;
    } else {
        for result in &response.results {
            println!(
                "{}:{}-{} score={:.3}",
                result.metadata.file_path.display(),
                result.metadata.start_line,
                result.metadata.end_line,
                result.score
            );
        }
    }
    Ok(())
}

fn run_local_search(args: &SearchArgs) -> Result<SearchResponse> {
    let root = resolve_repo_root(&args.paths)?;
    let index_dir = local_index_dir(&root);
    let metadata_path = local_metadata_path(&root);
    if !index_dir.exists() {
        return Err(anyhow::anyhow!(
            "Local index not found at {}. Run 'ricegrep index build <path>' first.",
            index_dir.display()
        ));
    }
    let handle = Bm25IndexHandle::open(&index_dir).context("failed to open local index")?;
    let searcher = LexicalSearcher::new(handle);
    let mut filters = LexicalFilters::default();
    filters.language = args.language.clone();
    filters.repository_id = args.repository_id;
    filters.file_path_prefix = args.file_path_pattern.clone();
    let limit = args.max_count.unwrap_or(DEFAULT_LOCAL_LIMIT);
    let start = Instant::now();
    let hits = searcher
        .search_with_filters(&args.query, &filters, limit)
        .context("local lexical search failed")?;
    let store = MetadataStore::load(&metadata_path).ok();
    let mut results: Vec<SearchResult> = hits
        .into_iter()
        .map(|hit| build_search_result(hit, store.as_ref()))
        .collect();
    if results.is_empty() {
        results = scan_local_matches(&root, &args.query, limit)?;
    }
    Ok(SearchResponse {
        total_found: results.len(),
        results,
        query_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        request_id: Uuid::new_v4().to_string(),
    })
}

fn build_search_result(hit: LexicalHit, store: Option<&MetadataStore>) -> SearchResult {
    let metadata = store
        .and_then(|store| store.get(hit.chunk_id).ok())
        .map(metadata_from_view)
        .unwrap_or_else(|| fallback_metadata(&hit));
    SearchResult {
        chunk_id: hit.chunk_id,
        score: hit.score,
        content: String::new(),
        metadata,
        highlights: Vec::new(),
    }
}

fn metadata_from_view(view: ChunkMetadataView) -> ChunkMetadata {
    ChunkMetadata {
        chunk_id: view.chunk_id,
        repository_id: view.repository_id,
        file_path: PathBuf::from(view.file_path),
        language: parse_language(&view.language),
        start_line: view.start_line,
        end_line: view.end_line,
        token_count: view.token_count,
        checksum: view.checksum.to_string(),
    }
}

fn fallback_metadata(hit: &LexicalHit) -> ChunkMetadata {
    ChunkMetadata {
        chunk_id: hit.chunk_id,
        repository_id: hit.repository_id,
        file_path: PathBuf::from(&hit.file_path),
        language: parse_language(&hit.language),
        start_line: 0,
        end_line: 0,
        token_count: 0,
        checksum: String::new(),
    }
}

fn parse_language(value: &str) -> LanguageKind {
    match value.to_lowercase().as_str() {
        "rust" => LanguageKind::Rust,
        "python" => LanguageKind::Python,
        "javascript" => LanguageKind::JavaScript,
        "typescript" => LanguageKind::TypeScript,
        "tsx" => LanguageKind::Tsx,
        "java" => LanguageKind::Java,
        "go" => LanguageKind::Go,
        "c" => LanguageKind::C,
        "cpp" => LanguageKind::Cpp,
        _ => LanguageKind::PlainText,
    }
}

fn resolve_repo_root(paths: &[String]) -> Result<PathBuf> {
    let mut root = if let Some(first) = paths.first() {
        PathBuf::from(first)
    } else {
        std::env::current_dir().context("failed to determine current directory")?
    };
    if root.is_file() {
        root = root
            .parent()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| anyhow::anyhow!("invalid path for search root"))?;
    }
    if !root.exists() {
        return Err(anyhow::anyhow!(
            "search path does not exist: {}",
            root.display()
        ));
    }
    Ok(root)
}

pub(crate) async fn ensure_local_index_ready(paths: &[String]) -> Result<()> {
    let root = resolve_repo_root(paths)?;
    ensure_local_index_for_root(&root).await
}

async fn ensure_local_index_for_root(root: &Path) -> Result<()> {
    let index_dir = local_index_dir(root);
    let metadata_path = local_metadata_path(root);
    let index_healthy =
        index_dir.exists() && metadata_path.exists() && Bm25IndexHandle::open(&index_dir).is_ok();
    if index_healthy {
        return Ok(());
    }
    println!(
        "Local index missing or incomplete at {}. Rebuilding...",
        index_dir.display()
    );
    let toolset = AdminToolset::new(index_dir.clone(), None);
    toolset
        .reindex_repository_with_metadata(root, &metadata_path)
        .await
        .context("failed to rebuild local index")?;
    println!("Local index rebuilt at {}", index_dir.display());
    Ok(())
}

fn local_state_dir(root: &Path) -> PathBuf {
    root.join(STATE_DIR_NAME).join(STORE_DIR_NAME)
}

fn local_index_dir(root: &Path) -> PathBuf {
    local_state_dir(root).join(INDEX_DIR_NAME)
}

fn local_metadata_path(root: &Path) -> PathBuf {
    local_index_dir(root).join(METADATA_FILE_NAME)
}

fn scan_local_matches(root: &Path, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let detector = LanguageDetector::default();
    let mut results = Vec::new();
    let mut next_id: u64 = 1;
    for entry in WalkDir::new(root) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let contents = match std::fs::read_to_string(entry.path()) {
            Ok(contents) => contents,
            Err(_) => continue,
        };
        let language = detector
            .detect(entry.path(), &contents)
            .unwrap_or(LanguageKind::PlainText);
        for (index, line) in contents.lines().enumerate() {
            if !line.contains(query) {
                continue;
            }
            let line_number = (index + 1) as u32;
            let metadata = ChunkMetadata {
                chunk_id: next_id,
                repository_id: None,
                file_path: entry.path().to_path_buf(),
                language,
                start_line: line_number,
                end_line: line_number,
                token_count: 0,
                checksum: String::new(),
            };
            results.push(SearchResult {
                chunk_id: next_id,
                score: 1.0,
                content: line.to_string(),
                metadata,
                highlights: vec![query.to_string()],
            });
            next_id = next_id.saturating_add(1);
            if results.len() >= limit {
                return Ok(results);
            }
        }
    }
    Ok(results)
}

async fn run_watch(runtime: &RuntimeConfig, args: WatchArgs) -> Result<()> {
    ensure_local_index_ready(&args.paths).await?;
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    for path in &args.paths {
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    }
    println!("Watching for changes...");
    let timeout = args.timeout.map(Duration::from_secs);
    let started = Instant::now();
    loop {
        if let Some(limit) = timeout {
            if started.elapsed() >= limit {
                break;
            }
        }
        let recv_timeout = timeout
            .map(|limit| limit.saturating_sub(started.elapsed()))
            .unwrap_or(Duration::from_millis(500));
        match rx.recv_timeout(recv_timeout) {
            Ok(Ok(event)) => {
                if let Event {
                    kind: notify::EventKind::Modify(_),
                    ..
                } = event
                {
                    if args.clear_screen {
                        print!("\x1B[2J\x1B[1;1H");
                    }
                    println!("File changed: {:?}", event.paths);
                }
            }
            Ok(Err(err)) => eprintln!("Watch event error: {}", err),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

async fn run_index(runtime: &RuntimeConfig, args: IndexArgs) -> Result<()> {
    match args.command {
        IndexCommand::Build(paths) => {
            let root = resolve_repo_root(&paths.paths)?;
            let index_dir = local_index_dir(&root);
            let metadata_path = local_metadata_path(&root);
            let toolset = AdminToolset::new(index_dir.clone(), None)
                .with_no_ignore(paths.no_ignore);
            let stats = toolset
                .reindex_repository_with_metadata(&root, &metadata_path)
                .await
                .context("failed to build local index")?;
            if runtime.json {
                print_json(&serde_json::to_value(&stats)?)?;
            } else {
                println!(
                    "Indexed {} files ({} chunks) in {:.2}s",
                    stats.files_indexed,
                    stats.chunks_indexed,
                    stats.duration.as_secs_f64()
                );
                println!("Index stored at {}", index_dir.display());
            }
            Ok(())
        }
        IndexCommand::Update(paths) => {
            let root = resolve_repo_root(&paths.paths)?;
            let index_dir = local_index_dir(&root);
            let metadata_path = local_metadata_path(&root);
            let toolset = AdminToolset::new(index_dir.clone(), None)
                .with_no_ignore(paths.no_ignore);
            let stats = toolset
                .reindex_repository_with_metadata(&root, &metadata_path)
                .await
                .context("failed to update local index")?;
            if runtime.json {
                print_json(&serde_json::to_value(&stats)?)?;
            } else {
                println!(
                    "Updated index with {} files ({} chunks) in {:.2}s",
                    stats.files_indexed,
                    stats.chunks_indexed,
                    stats.duration.as_secs_f64()
                );
            }
            Ok(())
        }
        IndexCommand::Clear => {
            let root = resolve_repo_root(&[])?;
            let index_dir = local_index_dir(&root);
            if index_dir.exists() {
                std::fs::remove_dir_all(&index_dir)?;
            }
            if runtime.json {
                print_json(&serde_json::json!({
                    "cleared": true,
                    "path": index_dir.display().to_string()
                }))?;
            } else {
                println!("Cleared local index at {}", index_dir.display());
            }
            Ok(())
        }
        IndexCommand::Status => {
            let root = resolve_repo_root(&[])?;
            let index_dir = local_index_dir(&root);
            let metadata_path = local_metadata_path(&root);
            if !index_dir.exists() {
                if runtime.json {
                    print_json(&serde_json::json!({"indexed": false}))?;
                } else {
                    println!("Index status: not built");
                }
                return Ok(());
            }
            let handle = Bm25IndexHandle::open(&index_dir).context("failed to open local index")?;
            let metadata = MetadataStore::load(&metadata_path).ok();
            let status = serde_json::json!({
                "indexed": true,
                "index_path": index_dir.display().to_string(),
                "documents": handle.document_count(),
                "tokens": handle.token_count(),
                "metadata_loaded": metadata.is_some(),
            });
            if runtime.json {
                print_json(&status)?;
            } else {
                println!("Index status: ready");
                println!("Documents: {}", handle.document_count());
                println!("Tokens: {}", handle.token_count());
                println!(
                    "Metadata store: {}",
                    if metadata.is_some() {
                        "ready"
                    } else {
                        "missing"
                    }
                );
            }
            Ok(())
        }
    }
}

fn run_replace(_runtime: &RuntimeConfig, args: ReplaceArgs) -> Result<()> {
    let content = std::fs::read_to_string(&args.file_path)?;
    if args.dry_run || args.preview {
        let new_content = content.replace(&args.old_symbol, &args.new_symbol);
        println!("Preview:");
        println!("{}", new_content);
        if !args.force {
            println!("Use --force to apply changes");
        }
        return Ok(());
    }
    if !args.force {
        println!("This will modify the file. Use --force to proceed.");
        return Ok(());
    }
    let new_content = content.replace(&args.old_symbol, &args.new_symbol);
    std::fs::write(&args.file_path, new_content)?;
    println!("Replaced in {}", args.file_path.display());
    Ok(())
}

fn run_files(args: FilesArgs) -> Result<()> {
    let root = args.paths.first().map(|s| s.as_str()).unwrap_or(".");
    let (matches, _truncated) = collect_glob_matches(root, &args.pattern, args.include_dirs, args.ignore_case)?;
    for m in matches {
        println!("{}", m);
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct E2eStepResult {
    name: String,
    success: bool,
    expected: String,
    actual: String,
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(serde::Serialize)]
struct E2eReport {
    success: bool,
    workspace: String,
    steps: Vec<E2eStepResult>,
}

async fn run_e2e(args: E2eArgs) -> Result<()> {
    if !args.source.exists() {
        return Err(anyhow::anyhow!(
            "E2E source workspace not found: {}",
            args.source.display()
        ));
    }
    let temp = tempdir().context("failed to create temp workspace")?;
    let workspace_root = temp.path().join("ricecoder-e2e");
    copy_dir_recursive(&args.source, &workspace_root)?;

    let config_root = args
        .config_root
        .unwrap_or_else(|| workspace_root.join(".e2e-config"));
    std::fs::create_dir_all(&config_root)?;

    let fixture_root = workspace_root.join("e2e-fixtures");
    std::fs::create_dir_all(&fixture_root)?;
    let fixture_path = fixture_root.join("fixture.rs");
    let fixture_text = "ricegrepe2e";
    std::fs::write(
        &fixture_path,
        format!("pub fn e2e() {{ println!(\"{}\"); }}\n", fixture_text),
    )?;

    let exe = std::env::current_exe().context("failed to locate ricegrep binary")?;
    let mut steps = Vec::new();

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "index build",
        &["index", "build", fixture_root.to_string_lossy().as_ref()],
        "exit=0",
        |output| output.status.success(),
    )?);

    steps.push(run_cli_step(
        &exe,
        &fixture_root,
        "index status",
        &["index", "status"],
        "indexed=true",
        |output| output.stdout.contains("ready") || output.stdout.contains("indexed"),
    )?);

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "search local",
        &[
            "search",
            fixture_text,
            fixture_root.to_string_lossy().as_ref(),
        ],
        "fixture path in output",
        |output| output.stdout.contains("fixture.rs"),
    )?);

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "files",
        &["files", ".rs", fixture_root.to_string_lossy().as_ref()],
        "non-empty output",
        |output| !output.stdout.trim().is_empty(),
    )?);

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "replace",
        &[
            "replace",
            fixture_text,
            "ricegrep-e2e-updated",
            fixture_path.to_string_lossy().as_ref(),
            "--force",
        ],
        "file content updated",
        |output| output.status.success(),
    )?);

    let updated = std::fs::read_to_string(&fixture_path)?;
    steps.push(E2eStepResult {
        name: "replace verification".to_string(),
        success: updated.contains("ricegrep-e2e-updated"),
        expected: "file contains updated content".to_string(),
        actual: updated.trim().to_string(),
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
    });

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "index update",
        &["index", "update", fixture_root.to_string_lossy().as_ref()],
        "exit=0",
        |output| output.status.success(),
    )?);

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "watch timeout",
        &[
            "watch",
            "--timeout",
            "1",
            fixture_root.to_string_lossy().as_ref(),
        ],
        "exit=0",
        |output| output.status.success(),
    )?);

    steps.push(run_cli_step_with_stdin(
        &exe,
        &workspace_root,
        "mcp tools list",
        &["mcp", "--no-watch"],
        "tools list response",
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n",
        |output| output.stdout.contains("tools"),
    )?);

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "plugin install",
        &[
            "install",
            "opencode",
            "--force",
            "--config-root",
            config_root.to_string_lossy().as_ref(),
        ],
        "exit=0",
        |output| output.status.success(),
    )?);

    steps.push(run_cli_step(
        &exe,
        &workspace_root,
        "plugin uninstall",
        &[
            "uninstall",
            "opencode",
            "--force",
            "--config-root",
            config_root.to_string_lossy().as_ref(),
        ],
        "exit=0",
        |output| output.status.success(),
    )?);

    steps.push(run_cli_step(
        &exe,
        &fixture_root,
        "index clear",
        &["index", "clear"],
        "exit=0",
        |output| output.status.success(),
    )?);

    let success = steps.iter().all(|step| step.success);
    let report = E2eReport {
        success,
        workspace: workspace_root.display().to_string(),
        steps,
    };
    let json = serde_json::to_string_pretty(&report)?;
    println!("{}", json);
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &json)?;
    }
    if !success {
        return Err(anyhow::anyhow!("E2E run failed; see JSON report"));
    }
    Ok(())
}

struct CliOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

fn run_cli_step(
    exe: &Path,
    cwd: &Path,
    name: &str,
    args: &[&str],
    expected: &str,
    predicate: impl Fn(&CliOutput) -> bool,
) -> Result<E2eStepResult> {
    let output = ProcessCommand::new(exe)
        .args(args)
        .current_dir(cwd)
        .output()
        .context("failed to run CLI command")?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let cli_output = CliOutput {
        status: output.status,
        stdout: stdout.clone(),
        stderr: stderr.clone(),
    };
    let success = predicate(&cli_output);
    Ok(E2eStepResult {
        name: name.to_string(),
        success,
        expected: expected.to_string(),
        actual: if success {
            "ok".to_string()
        } else {
            format!("stdout={} stderr={}", stdout.trim(), stderr.trim())
        },
        exit_code: cli_output.status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}

fn run_cli_step_with_stdin(
    exe: &Path,
    cwd: &Path,
    name: &str,
    args: &[&str],
    expected: &str,
    stdin_payload: &str,
    predicate: impl Fn(&CliOutput) -> bool,
) -> Result<E2eStepResult> {
    let mut child = ProcessCommand::new(exe)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn CLI command")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(stdin_payload.as_bytes())
            .context("failed to write stdin payload")?;
    }
    let output = child
        .wait_with_output()
        .context("failed to read CLI output")?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let cli_output = CliOutput {
        status: output.status,
        stdout: stdout.clone(),
        stderr: stderr.clone(),
    };
    let success = predicate(&cli_output);
    Ok(E2eStepResult {
        name: name.to_string(),
        success,
        expected: expected.to_string(),
        actual: if success {
            "ok".to_string()
        } else {
            format!("stdout={} stderr={}", stdout.trim(), stderr.trim())
        },
        exit_code: cli_output.status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    for entry in WalkDir::new(source) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(source)?;
        if relative.components().any(|component| {
            let name = component.as_os_str().to_string_lossy();
            name == ".git" || name == "target"
        }) {
            continue;
        }
        let destination = target.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, &destination)?;
        }
    }
    Ok(())
}

fn run_export_skills(args: ExportSkillsArgs) -> Result<()> {
    let skills = vec![
        SkillDefinition {
            name: "grep".to_string(),
            description: "Search file contents using local index or server mode".to_string(),
            command: "ricegrep search <query>".to_string(),
        },
        SkillDefinition {
            name: "nl_search".to_string(),
            description: "Natural-language search with opt-in answer flags".to_string(),
            command: "ricegrep search <query> --natural-language".to_string(),
        },
        SkillDefinition {
            name: "glob".to_string(),
            description: "Find files by glob pattern with ignore awareness".to_string(),
            command: "ricegrep files <pattern>".to_string(),
        },
        SkillDefinition {
            name: "list".to_string(),
            description: "List directory contents with ignore awareness".to_string(),
            command: "ricegrep list <path>".to_string(),
        },
        SkillDefinition {
            name: "read".to_string(),
            description: "Read file contents with optional line ranges".to_string(),
            command: "ricegrep read <file>".to_string(),
        },
        SkillDefinition {
            name: "edit".to_string(),
            description: "Edit a file with preview and force safeguards".to_string(),
            command: "ricegrep replace <old> <new> <file> --force".to_string(),
        },
    ];
    if args.format == "yaml" {
        println!("{}", serde_yaml::to_string(&skills)?);
    } else {
        print_json(&serde_json::to_value(&skills)?)?;
    }
    Ok(())
}
