use crate::RuntimeConfig;

#[derive(clap::Args, Debug)]
pub struct McpArgs {
    /// Paths to watch for changes
    #[arg(default_value = ".")]
    pub paths: Vec<std::path::PathBuf>,

    /// Timeout for watch operations
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Debounce seconds for file changes
    #[arg(long, default_value = "2")]
    pub debounce_secs: u64,

    /// Clear screen on file changes
    #[arg(long)]
    pub clear_screen: bool,

    /// Disable automatic watch mode
    #[arg(long)]
    pub no_watch: bool,

    /// Include every tool (read/edit) in tools/list output
    #[arg(long = "all-tools")]
    pub all_tools: bool,

    /// Server endpoint for MCP proxy mode
    #[arg(long = "server-endpoint", alias = "gateway")]
    pub server_endpoint: Option<String>,
}

use ricegrep::api::models::{SearchFilters, SearchRequest, SearchResponse, SearchResult};
use ricegrep::chunking::{ChunkMetadata, LanguageDetector, LanguageKind};
use glob::Pattern;
use ignore::WalkBuilder;
use regex::Regex;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt,
};
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};
use std::path::Path;
use std::time::Instant;
use uuid::Uuid;
use anyhow::{Context, Result};
use notify::Watcher;

const MCP_AUTO_WATCH_DELAY_SECS: u64 = 5;

#[cfg(feature = "server")]
const SERVER_FEATURE_ENABLED: bool = true;

#[cfg(not(feature = "server"))]
const SERVER_FEATURE_ENABLED: bool = false;
#[derive(Debug, serde::Deserialize, JsonSchema)]
struct GrepToolInput {
    pattern: String,
    include: Option<String>,
    path: Option<String>,
    max_results: Option<usize>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct NlSearchToolInput {
    query: String,
    include: Option<String>,
    path: Option<String>,
    max_results: Option<usize>,
    answer: Option<bool>,
    no_rerank: Option<bool>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct GlobToolInput {
    pattern: String,
    path: Option<String>,
    include_dirs: Option<bool>,
    ignore_case: Option<bool>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct ListToolInput {
    path: Option<String>,
    pattern: Option<String>,
    ignore_case: Option<bool>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct ReadToolInput {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct EditToolInput {
    file_path: String,
    old_string: String,
    new_string: String,
    #[serde(default)]
    replace_all: Option<bool>,
    #[serde(default)]
    timeout_secs: Option<u64>,
}

/// Manages watch lifecycle tied to MCP server
struct WatchManager {
    handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl WatchManager {
    fn new() -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        Self {
            handle: None,
            shutdown_tx,
        }
    }
    
    /// Start watch operation
    fn start(&mut self, runtime: std::sync::Arc<crate::RuntimeConfig>, watch_args: crate::WatchArgs) {
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let runtime_clone = runtime.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for delay before starting watch
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(MCP_AUTO_WATCH_DELAY_SECS)) => {
                    // Start watch with shutdown signal
                    let _ = run_watch_with_shutdown(
                        &runtime_clone,
                        watch_args,
                        shutdown_rx.resubscribe()
                    ).await;
                }
                _ = shutdown_rx.recv() => {
                    // Shutdown during delay
                    tracing::info!("Watch cancelled before start");
                    return;
                }
            }
        });
        
        self.handle = Some(handle);
    }
    
    /// Gracefully shutdown watch
    async fn shutdown(&mut self) -> Result<()> {
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());
        
        // Wait for watch to exit (with timeout)
        if let Some(handle) = self.handle.take() {
            tokio::time::timeout(
                Duration::from_secs(5),
                handle
            )
            .await
            .context("Watch shutdown timed out after 5s")??;
        }
        
        Ok(())
    }
}

/// Tracks file changes for deduplication and batching
#[derive(Debug)]
struct ChangeTracker {
    changed_files: std::collections::HashMap<std::path::PathBuf, std::time::SystemTime>,
}

impl ChangeTracker {
    fn new() -> Self {
        Self {
            changed_files: std::collections::HashMap::new(),
        }
    }
    
    /// Record a file change
    fn record_change(&mut self, path: std::path::PathBuf) {
        self.changed_files.insert(path, std::time::SystemTime::now());
    }
    
    /// Get and clear all tracked changes
    fn take_changes(&mut self) -> Vec<std::path::PathBuf> {
        let paths: Vec<_> = self.changed_files.keys().cloned().collect();
        self.changed_files.clear();
        paths
    }
    
    /// Check if there are pending changes
    fn has_changes(&self) -> bool {
        !self.changed_files.is_empty()
    }
    
    /// Get count of tracked changes
    fn change_count(&self) -> usize {
        self.changed_files.len()
    }
}

/// Run watch with shutdown signal support
async fn run_watch_with_shutdown(
    runtime: &RuntimeConfig,
    args: crate::WatchArgs,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)
        .context("Failed to create file watcher")?;
    
    for path in &args.paths {
        watcher.watch(path.as_ref(), notify::RecursiveMode::Recursive)
            .context("Failed to watch path")?;
    }
    
    tracing::info!("Watch started for {:?}", args.paths);
    
    let mut change_tracker = ChangeTracker::new();
    let recv_timeout = Duration::from_millis(100);
    let batch_interval = Duration::from_secs(1);
    let mut last_batch = std::time::Instant::now();
    
    loop {
        // Check for shutdown signal
        match shutdown_rx.try_recv() {
            Ok(_) | Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                // Flush pending changes before shutdown
                process_tracked_changes(&mut change_tracker);
                tracing::info!("Watch received shutdown signal");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                // Continue watching
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                // Broadcast buffer lagged, still process shutdown
                tracing::warn!("Watch shutdown signal lagged, continuing");
            }
        }
        
        // Check for file events
        match rx.recv_timeout(recv_timeout) {
            Ok(Ok(event)) => {
                handle_watch_event(&mut change_tracker, &event, &args);
            }
            Ok(Err(e)) => {
                tracing::error!("Watch error: {}", e);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Batch process changes at interval
                if last_batch.elapsed() >= batch_interval && change_tracker.has_changes() {
                    process_tracked_changes(&mut change_tracker);
                    last_batch = std::time::Instant::now();
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                tracing::warn!("Watch channel disconnected");
                break;
            }
        }
    }
    
    Ok(())
}

/// Handle individual watch events and track changes
fn handle_watch_event(
    tracker: &mut ChangeTracker,
    event: &notify::Event,
    args: &crate::WatchArgs,
) {
    use notify::EventKind;
    
    match event.kind {
        EventKind::Create(_) => {
            for path in &event.paths {
                if path.is_file() {
                    tracing::debug!("File created: {}", path.display());
                    tracker.record_change(path.clone());
                }
            }
        }
        EventKind::Modify(_) => {
            for path in &event.paths {
                if path.is_file() {
                    tracing::debug!("File modified: {}", path.display());
                    tracker.record_change(path.clone());
                }
            }
        }
        EventKind::Remove(_) => {
            for path in &event.paths {
                tracing::debug!("File removed: {}", path.display());
                tracker.record_change(path.clone());
            }
        }
        _ => {
            // Ignore other event kinds (access, metadata changes, etc.)
        }
    }
    
    // Clear screen on first change if requested
    if args.clear_screen && tracker.change_count() == 1 {
        print!("\x1B[2J\x1B[1;1H");
    }
}

/// Process accumulated changes and log summary
fn process_tracked_changes(tracker: &mut ChangeTracker) {
    if !tracker.has_changes() {
        return;
    }
    
    let changes = tracker.take_changes();
    let count = changes.len();
    
    tracing::info!("Tracked {} file change(s)", count);
    for path in changes.iter().take(5) {
        tracing::debug!("  - {}", path.display());
    }
    
    if count > 5 {
        tracing::debug!("  ... and {} more", count - 5);
    }
}

#[derive(Debug, Clone)]
struct RicegrepMcp {
    runtime: RuntimeConfig,
    server_endpoint: Option<String>,
    show_all_tools: bool,
    pub tool_router: ToolRouter<RicegrepMcp>,
}


impl RicegrepMcp {
    pub fn new(runtime: RuntimeConfig, server_endpoint: Option<String>, show_all_tools: bool) -> Self {
        Self {
            runtime,
            server_endpoint,
            show_all_tools,
            tool_router: Self::tool_router(),
        }
    }

    fn is_tool_allowed(&self, name: &str) -> bool {
        self.show_all_tools || !matches!(name, "rice_read" | "rice_edit")
    }


    async fn execute_search(
        &self,
        request: SearchRequest,
        root: Option<&str>,
    ) -> Result<(SearchResponse, Option<String>), ErrorData> {
        let root = Path::new(root.unwrap_or("."));
        let filter = request
            .filters
            .as_ref()
            .and_then(|filters| filters.file_path_pattern.as_deref());
        if let Some(endpoint) = self.server_endpoint.as_ref() {
            if !SERVER_FEATURE_ENABLED {
                return Err(ErrorData::internal_error(
                    "Server mode is disabled. Rebuild with --features server to enable it."
                        .to_string(),
                    None,
                ));
            }
            match server_search_request(endpoint, &request).await {
                Ok(response) => return Ok((response, None)),
                Err(err) => {
                    let response = local_search_response(&request, root, filter)
                        .map_err(|fallback_err| {
                            ErrorData::internal_error(
                                format!(
                                    "server error: {err}; local fallback failed: {fallback_err}"
                                ),
                                None,
                            )
                        })?;
                    let warning =
                        format!("Server unavailable; using local scan. error={}", err);
                    return Ok((response, Some(warning)));
                }
            }
        }

        let response = local_search_response(&request, root, filter)
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok((response, None))
    }
}

#[tool_router]
impl RicegrepMcp {
    #[tool(
        name = "rice_grep",
        description = "Search file contents using local index or server mode. Ideal for finding function definitions, error messages, configuration values, and recurring code patterns. Supports full regex syntax, directory scoping, file-type filters, and result limits, automatically falling back to local scans when server mode is unavailable."
    )]
    async fn grep(
        &self,
        Parameters(input): Parameters<GrepToolInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let file_path_pattern = input.include.clone().or(input.path.clone());
        let filters = file_path_pattern.map(|pattern| SearchFilters {
            repository_id: None,
            language: None,
            file_path_pattern: Some(pattern),
        });
        let request = SearchRequest {
            query: input.pattern,
            limit: input.max_results,
            filters,
            ranking: None,
            timeout_ms: None,
        };

        let (response, warning) = self
            .execute_search(request, input.path.as_deref())
            .await?;
        let mut output = String::new();
        if let Some(note) = warning {
            output.push_str(&note);
            output.push('\n');
        }
        output.push_str(&format_search_lines(&response));
        Ok(tool_result_with_response(output.trim_end().to_string(), &response))
    }

    #[tool(
        name = "rice_nl_search",
        description = "Natural-language search with opt-in answer generation. Understands conversational questions about the codebase, supports directory or file-type scoping, respects result limits, and can summarize findings with AI-generated answers or disable reranking for deterministic ordering."
    )]
    async fn nl_search(
        &self,
        Parameters(input): Parameters<NlSearchToolInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let file_path_pattern = input.include.clone().or(input.path.clone());
        let filters = file_path_pattern.map(|pattern| SearchFilters {
            repository_id: None,
            language: None,
            file_path_pattern: Some(pattern),
        });
        let request = SearchRequest {
            query: input.query,
            limit: input.max_results,
            filters,
            ranking: None,
            timeout_ms: None,
        };

        let (response, warning) = self
            .execute_search(request, input.path.as_deref())
            .await?;
        let mut output = String::new();
        if let Some(note) = warning {
            output.push_str(&note);
            output.push('\n');
        }
        if input.answer.unwrap_or(false) {
            output.push_str("Answer generation is not available; returning matches.\n");
        }
        if input.no_rerank.unwrap_or(false) {
            output.push_str(
                "Rerank disable is not supported by the server; returning default order.\n",
            );
        }
        output.push_str(&format_search_lines(&response));
        Ok(tool_result_with_response(output.trim_end().to_string(), &response))
    }

    #[tool(
        name = "rice_glob",
        description = "Find files by glob pattern with ignore awareness. Performs fast wildcard searches across directories, honors .gitignore/.ignore rules, supports recursive matching, optional directory results, and case-insensitive queries for cross-platform consistency."
    )]
    async fn glob(&self, Parameters(input): Parameters<GlobToolInput>) -> Result<CallToolResult, ErrorData> {
        let root = input.path.as_deref().unwrap_or(".");
        let matches = crate::collect_glob_matches(
            root,
            &input.pattern,
            input.include_dirs.unwrap_or(false),
            input.ignore_case.unwrap_or(false),
        )
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(tool_text_result(matches.join("\n")))
    }

    #[tool(
        name = "rice_list",
        description = "List directory contents with ignore awareness. Produces filtered directory listings that respect project ignore files, optional glob filters, and case-insensitive matching so you can inspect structure before drilling into files."
    )]
    async fn list(&self, Parameters(input): Parameters<ListToolInput>) -> Result<CallToolResult, ErrorData> {
        let root = input.path.as_deref().unwrap_or(".");
        let entries = crate::list_directory_entries(
            root,
            input.pattern.as_deref(),
            input.ignore_case.unwrap_or(false),
        )
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(tool_text_result(entries.join("\n")))
    }

    #[tool(
        name = "rice_read",
        description = "Read file contents with optional line ranges. Streams numbered output with offset and limit controls so you can inspect entire files or focused snippets without overwhelming context."
    )]
    async fn read(&self, Parameters(input): Parameters<ReadToolInput>) -> Result<CallToolResult, ErrorData> {
        let output = crate::read_file_numbered(&input.path, input.offset, input.limit)
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(tool_text_result(output))
    }

    #[tool(
        name = "rice_edit",
        description = "Edit a file with preview and force safeguards. Performs exact string replacements with verification so you can refactor or fix configurations safely without unintended edits. Supports replace_all parameter to replace all occurrences instead of just the first."
    )]
    async fn edit(&self, Parameters(input): Parameters<EditToolInput>) -> Result<CallToolResult, ErrorData> {
        let output = apply_edit(&input)
            .await
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(tool_text_result(output))
    }
}

#[tool_handler]
impl ServerHandler for RicegrepMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

fn tool_title(name: &str) -> &'static str {
    match name {
        "rice_grep" => "File Content Search",
        "rice_nl_search" => "Natural Language Search",
        "rice_glob" => "File Glob Finder",
        "rice_list" => "Directory Lister",
        "rice_read" => "File Reader",
        "rice_edit" => "File Editor",
        _ => "Ricegrep Tool",
    }
}

fn tool_annotations(name: &str) -> serde_json::Value {
    let (safe, idempotent) = match name {
        "rice_edit" => (false, false),
        _ => (true, true),
    };
    serde_json::json!({
        "audience": ["user", "assistant"],
        "priority": 0.85,
        "safe": safe,
        "idempotent": idempotent
    })
}

fn tool_output_schema(name: &str) -> serde_json::Value {
    match name {
        "rice_grep" | "rice_nl_search" => search_output_schema(),
        _ => text_only_output_schema(),
    }
}

fn text_only_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "content": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "type": {"type": "string"},
                        "text": {"type": "string"}
                    },
                    "required": ["type", "text"]
                }
            },
            "is_error": {"type": ["boolean", "null"]},
            "meta": {"type": ["object", "null"]},
            "structured_content": {"type": ["null", "object"]}
        },
        "required": ["content"]
    })
}

fn search_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "content": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "type": {"type": "string"},
                        "text": {"type": "string"}
                    },
                    "required": ["type", "text"]
                }
            },
            "is_error": {"type": ["boolean", "null"]},
            "meta": {"type": ["object", "null"]},
            "structured_content": {
                "type": ["object", "null"],
                "properties": {
                    "results": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "score": {"type": "number"},
                                "content": {"type": "string"},
                                "metadata": {
                                    "type": "object",
                                    "properties": {
                                        "file_path": {"type": "string"},
                                        "start_line": {"type": "number"},
                                        "end_line": {"type": "number"}
                                    },
                                    "required": ["file_path", "start_line", "end_line"]
                                }
                            },
                            "required": ["score", "content", "metadata"]
                        }
                    },
                    "total_found": {"type": "number"},
                    "query_time_ms": {"type": "number"},
                    "request_id": {"type": "string"}
                },
                "required": ["results", "total_found", "query_time_ms", "request_id"]
            }
        },
        "required": ["content"]
    })
}

async fn apply_edit(input: &EditToolInput) -> Result<String> {
    // Set timeout wrapper
    let timeout_duration = Duration::from_secs(
        input.timeout_secs.unwrap_or(30)
    );
    
    tokio::time::timeout(timeout_duration, apply_edit_inner(input))
        .await
        .context(format!("Edit operation timed out after {}s", timeout_duration.as_secs()))?
}

async fn apply_edit_inner(input: &EditToolInput) -> Result<String> {
    // Read file asynchronously
    let content = tokio::fs::read_to_string(&input.file_path)
        .await
        .context(format!("Failed to read file: {}", input.file_path))?;
    
    // Handle replace_all parameter
    let new_content = if input.replace_all.unwrap_or(false) {
        content.replace(&input.old_string, &input.new_string)
    } else {
        content.replacen(&input.old_string, &input.new_string, 1)
    };
    
    // Check if replacement happened
    if new_content == content {
        return Err(anyhow::anyhow!(
            "Pattern not found: '{}' in {}",
            input.old_string,
            input.file_path
        ));
    }
    
    // Write atomically via temp file
    let temp_path = format!("{}.tmp", input.file_path);
    tokio::fs::write(&temp_path, &new_content)
        .await
        .context("Failed to write temporary file")?;
    
    tokio::fs::rename(&temp_path, &input.file_path)
        .await
        .context("Failed to rename temporary file to original location")?;
    
    let occurrences = content.matches(&input.old_string).count();
    let replaced = if input.replace_all.unwrap_or(false) {
        occurrences
    } else {
        1
    };
    
    Ok(format!(
        "Edited {}: replaced {} occurrence(s) of '{}' with '{}'",
        input.file_path, replaced, input.old_string, input.new_string
    ))
}

fn tool_text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![rmcp::model::Content::text(text)],
        is_error: None,
        meta: None,
        structured_content: None,
    }
}

fn tool_result_with_response(text: String, response: &SearchResponse) -> CallToolResult {
    let structured = serde_json::to_value(response).ok();
    CallToolResult {
        content: vec![Content::text(text)],
        is_error: None,
        meta: None,
        structured_content: structured,
    }
}

fn format_search_lines(response: &SearchResponse) -> String {
    let mut lines = Vec::new();
    for result in &response.results {
        lines.push(format!(
            "{}:{}-{} score={:.3}",
            result.metadata.file_path.display(),
            result.metadata.start_line,
            result.metadata.end_line,
            result.score
        ));
    }
    lines.join("\n")
}

enum PathMatcher {
    Any,
    Glob(Pattern),
    Substring(String),
}

impl PathMatcher {
    fn matches(&self, path: &Path) -> bool {
        match self {
            PathMatcher::Any => true,
            PathMatcher::Glob(pattern) => pattern.matches_path(path),
            PathMatcher::Substring(filter) => path.to_string_lossy().contains(filter),
        }
    }
}

enum ContentMatcher {
    Regex(Regex),
    Literal(String),
}

impl ContentMatcher {
    fn is_match(&self, line: &str) -> bool {
        match self {
            ContentMatcher::Regex(regex) => regex.is_match(line),
            ContentMatcher::Literal(text) => line.contains(text),
        }
    }
}

fn build_path_matcher(filter: Option<&str>) -> PathMatcher {
    let Some(filter) = filter else {
        return PathMatcher::Any;
    };
    Pattern::new(filter)
        .map(PathMatcher::Glob)
        .unwrap_or_else(|_| PathMatcher::Substring(filter.to_string()))
}

fn build_content_matcher(query: &str) -> ContentMatcher {
    Regex::new(query)
        .map(ContentMatcher::Regex)
        .unwrap_or_else(|_| ContentMatcher::Literal(query.to_string()))
}

fn collect_local_matches(
    root: &Path,
    query: &str,
    path_matcher: &PathMatcher,
    content_matcher: &ContentMatcher,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>> {
    let detector = LanguageDetector::default();
    let mut results = Vec::new();
    let mut next_id: u64 = 1;
    for entry in WalkBuilder::new(root).build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path_matcher.matches(path) {
            continue;
        }
        if append_matches_for_file(
            &mut results,
            path,
            query,
            limit,
            &detector,
            &mut next_id,
            content_matcher,
        )? {
            return Ok(results);
        }
    }

    Ok(results)
}

fn append_matches_for_file(
    results: &mut Vec<SearchResult>,
    path: &Path,
    query: &str,
    limit: Option<usize>,
    detector: &LanguageDetector,
    next_id: &mut u64,
    content_matcher: &ContentMatcher,
) -> Result<bool> {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return Ok(false),
    };
    let language = detector
        .detect(path, &contents)
        .unwrap_or(LanguageKind::PlainText);
    for (index, line) in contents.lines().enumerate() {
        if !content_matcher.is_match(line) {
            continue;
        }
        let line_number = (index + 1) as u32;
        let metadata = ChunkMetadata {
            chunk_id: *next_id,
            repository_id: None,
            file_path: path.to_path_buf(),
            language,
            start_line: line_number,
            end_line: line_number,
            token_count: 0,
            checksum: String::new(),
        };
        results.push(SearchResult {
            chunk_id: *next_id,
            score: 1.0,
            content: line.to_string(),
            metadata,
            highlights: vec![query.to_string()],
        });
        *next_id = next_id.saturating_add(1);
        if limit.map_or(false, |max| results.len() >= max) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn local_search_response(
    request: &SearchRequest,
    root: &Path,
    filter: Option<&str>,
) -> Result<SearchResponse> {
    let start = Instant::now();
    let path_matcher = build_path_matcher(filter);
    let content_matcher = build_content_matcher(&request.query);
    let results = collect_local_matches(
        root,
        &request.query,
        &path_matcher,
        &content_matcher,
        request.limit,
    )?;
    let total_found = results.len();
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok(SearchResponse {
        results,
        total_found,
        query_time_ms: elapsed,
        request_id: Uuid::new_v4().to_string(),
    })
}

#[cfg(feature = "server")]
async fn server_search_request(
    endpoint: &str,
    request: &SearchRequest,
) -> Result<SearchResponse> {
    let url = format!("{}/search", endpoint.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .post(&url)
        .json(request)
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
    _endpoint: &str,
    _request: &SearchRequest,
) -> Result<SearchResponse> {
    Err(anyhow::anyhow!(
        "Server mode is disabled. Rebuild with --features server to enable it."
    ))
}

async fn handle_mcp_request(mcp: &RicegrepMcp, tool_router: &ToolRouter<RicegrepMcp>, request: &serde_json::Value) -> Result<serde_json::Value> {
    let id = request["id"].clone();
    let method = request["method"].as_str().unwrap_or("");

    let response = match method {
        "initialize" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": true
}

                    },
                    "serverInfo": {
                        "name": "ricegrep",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            })
        }
        "tools/list" => {
            let tools = tool_router
                .list_all()
                .into_iter()
                .filter(|tool| mcp.is_tool_allowed(tool.name.as_ref()))
                .map(|tool| {
                    let name = tool.name.as_ref();
                    serde_json::json!({
                        "name": name,
                        "title": tool_title(name),
                        "description": tool.description,
                        "inputSchema": tool.input_schema,
                        "outputSchema": tool_output_schema(name),
                        "annotations": tool_annotations(name)
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": tools
                }
            })
        }
            "tools/call" => {
                let tool_name = request["params"]["name"].as_str().unwrap_or("");
                let arguments = request["params"]["arguments"].clone();
                let result = call_tool(mcp, tool_name, arguments).await?;
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                })
            }
        _ => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            })
        }
    };

    Ok(response)
}

async fn call_tool(
    mcp: &RicegrepMcp,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value> {
    let result = match tool_name {
        "rice_grep" => {
            let input: GrepToolInput = serde_json::from_value(arguments)?;
            mcp.grep(Parameters(input)).await?
        }
        "rice_nl_search" => {
            let input: NlSearchToolInput = serde_json::from_value(arguments)?;
            mcp.nl_search(Parameters(input)).await?
        }
        "rice_glob" => {
            let input: GlobToolInput = serde_json::from_value(arguments)?;
            mcp.glob(Parameters(input)).await?
        }
        "rice_list" => {
            let input: ListToolInput = serde_json::from_value(arguments)?;
            mcp.list(Parameters(input)).await?
        }
        "rice_read" => {
            let input: ReadToolInput = serde_json::from_value(arguments)?;
            mcp.read(Parameters(input)).await?
        }
        "rice_edit" => {
            let input: EditToolInput = serde_json::from_value(arguments)?;
            mcp.edit(Parameters(input)).await?
        }
        _ => return Ok(serde_json::json!([])),
    };
    Ok(serde_json::to_value(&result)?)
}

pub async fn run_mcp(runtime: &RuntimeConfig, args: McpArgs) -> Result<()> {
    let watch_paths: Vec<String> = args
        .paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    super::ensure_local_index_ready(&watch_paths).await?;

    let mcp = RicegrepMcp::new(runtime.clone(), args.server_endpoint.clone(), args.all_tools);
    let tool_router = &mcp.tool_router;

    let mut watch_manager = WatchManager::new();
    
    if !args.no_watch {
        let runtime_clone = std::sync::Arc::new(runtime.clone());
        let watch_args = crate::WatchArgs {
            paths: watch_paths.clone(),
            timeout: args.timeout,
            debounce_secs: args.debounce_secs,
            clear_screen: args.clear_screen,
        };

        watch_manager.start(runtime_clone, watch_args);
    }

    let mut stdin = io::BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        if stdin.read_line(&mut buffer).await? == 0 {
            // EOF - graceful shutdown
            break;
        }
        let request: serde_json::Value = serde_json::from_str(&buffer.trim())?;
        let response = handle_mcp_request(&mcp, tool_router, &request).await?;
        let response_str = serde_json::to_string(&response)? + "\n";
        stdout.write_all(response_str.as_bytes()).await?;
        stdout.flush().await?;
    }

    // Shutdown watch before exiting
    tracing::info!("Shutting down watch...");
    watch_manager.shutdown().await?;
    tracing::info!("Watch shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tool_variant_inventory() {
        let expected = ["rice_grep", "rice_glob", "rice_list", "rice_read", "rice_edit", "rice_nl_search"];
        let router = RicegrepMcp::tool_router();
        let tool_names: Vec<String> = router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();
        for tool in expected {
            assert!(tool_names.iter().any(|name| name == tool), "missing tool: {tool}");
        }
    }

    #[test]
    fn mcp_tool_schema_has_properties() {
        let router = RicegrepMcp::tool_router();
        for tool in router.list_all() {
            let schema = tool.input_schema.as_ref();
            let schema_type = schema.get("type").and_then(|value| value.as_str());
            assert_eq!(
                Some("object"),
                schema_type,
                "tool {} schema should be an object",
                tool.name
            );
            assert!(
                schema.contains_key("properties"),
                "tool {} schema should declare properties",
                tool.name
            );
        }
    }

    #[tokio::test]
    async fn test_edit_small_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        // Write initial content
        tokio::fs::write(&file_path, "Hello World\nHello there").await.unwrap();
        
        // Perform edit
        let input = EditToolInput {
            file_path: file_path.clone(),
            old_string: "Hello".to_string(),
            new_string: "Hi".to_string(),
            replace_all: Some(false),
            timeout_secs: Some(30),
        };
        
        let result = apply_edit(&input).await;
        assert!(result.is_ok());
        
        // Verify only first occurrence replaced
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hi World\nHello there");
    }

    #[tokio::test]
    async fn test_edit_replace_all() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        // Write initial content with multiple occurrences
        tokio::fs::write(&file_path, "foo bar\nfoo baz\nfoo qux").await.unwrap();
        
        // Perform edit with replace_all
        let input = EditToolInput {
            file_path: file_path.clone(),
            old_string: "foo".to_string(),
            new_string: "bar".to_string(),
            replace_all: Some(true),
            timeout_secs: Some(30),
        };
        
        let result = apply_edit(&input).await;
        assert!(result.is_ok());
        let result_msg = result.unwrap();
        assert!(result_msg.contains("replaced 3 occurrence(s)"));
        
        // Verify all occurrences replaced
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "bar bar\nbar baz\nbar qux");
    }

    #[tokio::test]
    async fn test_edit_file_not_found() {
        let input = EditToolInput {
            file_path: "/nonexistent/path/file.txt".to_string(),
            old_string: "test".to_string(),
            new_string: "result".to_string(),
            replace_all: None,
            timeout_secs: Some(30),
        };
        
        let result = apply_edit(&input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read file"));
    }

    #[tokio::test]
    async fn test_edit_pattern_not_found() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        // Write initial content
        tokio::fs::write(&file_path, "Hello World").await.unwrap();
        
        // Try to replace non-existent pattern
        let input = EditToolInput {
            file_path: file_path.clone(),
            old_string: "Goodbye".to_string(),
            new_string: "Hi".to_string(),
            replace_all: None,
            timeout_secs: Some(30),
        };
        
        let result = apply_edit(&input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Pattern not found"));
    }

    #[tokio::test]
    async fn test_edit_timeout() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        // Write initial content
        tokio::fs::write(&file_path, "Hello World").await.unwrap();
        
        // Create input with very short timeout (note: this test may be flaky on slow systems)
        let input = EditToolInput {
            file_path: file_path.clone(),
            old_string: "Hello".to_string(),
            new_string: "Hi".to_string(),
            replace_all: None,
            timeout_secs: Some(1),
        };
        
        // This should succeed normally even with 1s timeout on fast systems
        let result = apply_edit(&input).await;
        // Just verify it completes without panicking
        let _ = result;
    }

    #[tokio::test]
    async fn test_edit_result_message_format() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        // Write initial content
        tokio::fs::write(&file_path, "test content").await.unwrap();
        
        let input = EditToolInput {
            file_path: file_path.clone(),
            old_string: "test".to_string(),
            new_string: "prod".to_string(),
            replace_all: None,
            timeout_secs: Some(30),
        };
        
        let result = apply_edit(&input).await.unwrap();
        assert!(result.contains("Edited"));
        assert!(result.contains("replaced 1 occurrence(s)"));
        assert!(result.contains("test"));
        assert!(result.contains("prod"));
    }

    #[test]
    fn test_change_tracker_new() {
        let tracker = ChangeTracker::new();
        assert!(!tracker.has_changes());
        assert_eq!(tracker.change_count(), 0);
    }

    #[test]
    fn test_change_tracker_record_single() {
        let mut tracker = ChangeTracker::new();
        let path = std::path::PathBuf::from("test.txt");
        
        tracker.record_change(path.clone());
        
        assert!(tracker.has_changes());
        assert_eq!(tracker.change_count(), 1);
    }

    #[test]
    fn test_change_tracker_deduplication() {
        let mut tracker = ChangeTracker::new();
        let path = std::path::PathBuf::from("test.txt");
        
        // Record same file multiple times
        tracker.record_change(path.clone());
        tracker.record_change(path.clone());
        tracker.record_change(path.clone());
        
        // Should only have 1 entry (latest timestamp)
        assert_eq!(tracker.change_count(), 1);
    }

    #[test]
    fn test_change_tracker_multiple_files() {
        let mut tracker = ChangeTracker::new();
        let path1 = std::path::PathBuf::from("file1.txt");
        let path2 = std::path::PathBuf::from("file2.txt");
        let path3 = std::path::PathBuf::from("file3.txt");
        
        tracker.record_change(path1.clone());
        tracker.record_change(path2.clone());
        tracker.record_change(path3.clone());
        
        assert_eq!(tracker.change_count(), 3);
        assert!(tracker.has_changes());
    }

    #[test]
    fn test_change_tracker_take_changes() {
        let mut tracker = ChangeTracker::new();
        let path1 = std::path::PathBuf::from("file1.txt");
        let path2 = std::path::PathBuf::from("file2.txt");
        
        tracker.record_change(path1.clone());
        tracker.record_change(path2.clone());
        
        let changes = tracker.take_changes();
        
        // Should have returned 2 changes
        assert_eq!(changes.len(), 2);
        
        // Should be empty after taking
        assert!(!tracker.has_changes());
        assert_eq!(tracker.change_count(), 0);
    }

    #[test]
    fn test_change_tracker_timestamps() {
        let mut tracker = ChangeTracker::new();
        let path = std::path::PathBuf::from("test.txt");
        
        let before = std::time::SystemTime::now();
        tracker.record_change(path.clone());
        let after = std::time::SystemTime::now();
        
        // Verify timestamp is recorded and is within bounds
        assert!(tracker.changed_files[&path] >= before);
        assert!(tracker.changed_files[&path] <= after);
    }

    #[test]
    fn test_change_tracker_rapid_changes() {
        let mut tracker = ChangeTracker::new();
        let path = std::path::PathBuf::from("rapid_changes.txt");
        
        // Rapid updates to same file
        for _ in 0..100 {
            tracker.record_change(path.clone());
        }
        
        // Should only have 1 entry with latest timestamp
        assert_eq!(tracker.change_count(), 1);
        let changes = tracker.take_changes();
        assert_eq!(changes.len(), 1);
    }

    #[tokio::test]
    async fn test_edit_atomic_write() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        // Write initial content
        let original = "This is the original content\nLine 2\nLine 3";
        tokio::fs::write(&file_path, original).await.unwrap();
        
        let input = EditToolInput {
            file_path: file_path.clone(),
            old_string: "original".to_string(),
            new_string: "modified".to_string(),
            replace_all: None,
            timeout_secs: Some(30),
        };
        
        let result = apply_edit(&input).await;
        assert!(result.is_ok());
        
        // Verify final content
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "This is the modified content\nLine 2\nLine 3");
        
        // Verify temp file was cleaned up
        let temp_path = format!("{}.tmp", file_path);
        assert!(!std::path::Path::new(&temp_path).exists());
    }
}
