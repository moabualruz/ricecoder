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
        description = "Edit a file with preview and force safeguards. Performs exact string replacements with verification so you can refactor or fix configurations safely without unintended edits."
    )]
    async fn edit(&self, Parameters(input): Parameters<EditToolInput>) -> Result<CallToolResult, ErrorData> {
        let output = apply_edit(&input).map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
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

fn apply_edit(input: &EditToolInput) -> Result<String> {
    let content = std::fs::read_to_string(&input.file_path)?;
    if !content.contains(&input.old_string) {
        return Err(anyhow::anyhow!("old_string not found in file"));
    }
    let new_content = content.replace(&input.old_string, &input.new_string);
    std::fs::write(&input.file_path, &new_content)?;
    Ok(format!(
        "Edited {}: replaced '{}' with '{}'",
        input.file_path, input.old_string, input.new_string
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
    let mcp = RicegrepMcp::new(runtime.clone(), args.server_endpoint.clone(), args.all_tools);

    let tool_router = &mcp.tool_router;

    if !args.no_watch {
        let runtime_clone = runtime.clone();
        let watch_args = crate::WatchArgs {
            paths: args
                .paths
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect(),
            timeout: args.timeout,
            debounce_secs: args.debounce_secs,
            clear_screen: args.clear_screen,
        };
        tokio::spawn(async move {
            sleep(Duration::from_secs(MCP_AUTO_WATCH_DELAY_SECS)).await;
            let _ = crate::run_watch(&runtime_clone, watch_args).await;
        });
    }

    let mut stdin = io::BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        if stdin.read_line(&mut buffer).await? == 0 {
            break;
        }
        let request: serde_json::Value = serde_json::from_str(&buffer.trim())?;
        let response = handle_mcp_request(&mcp, tool_router, &request).await?;
        let response_str = serde_json::to_string(&response)? + "\n";
        stdout.write_all(response_str.as_bytes()).await?;
        stdout.flush().await?;
    }

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
}
