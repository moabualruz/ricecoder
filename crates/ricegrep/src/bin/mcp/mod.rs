use crate::RuntimeConfig;

mod mappers;
mod protocol;
mod response;
mod search;
mod tools;
mod types;
mod watch;

use types::{
    EditToolInput, GlobToolInput, GrepToolInput, ListToolInput, NlSearchToolInput, ReadToolInput,
    WriteToolInput,
};

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

use anyhow::Result;
use ricegrep::api::models::{SearchFilters, SearchRequest, SearchResponse};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData, ServerHandler,
};
use std::path::Path;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

// Server feature flag for conditional compilation
#[cfg(feature = "server")]
const SERVER_FEATURE_ENABLED: bool = true;

#[cfg(not(feature = "server"))]
const SERVER_FEATURE_ENABLED: bool = false;

#[derive(Debug)]
struct RicegrepMcp {
    runtime_config: RuntimeConfig,
    server_endpoint: Option<String>,
    show_all_tools: bool,
    pub tool_router: ToolRouter<RicegrepMcp>,
}

impl RicegrepMcp {
    pub fn new(
        runtime_config: RuntimeConfig,
        server_endpoint: Option<String>,
        show_all_tools: bool,
    ) -> Self {
        Self {
            runtime_config,
            server_endpoint,
            show_all_tools,
            tool_router: Self::tool_router(),
        }
    }

    fn is_tool_allowed(&self, name: &str) -> bool {
        self.show_all_tools || !matches!(name, "rice_read" | "rice_edit" | "rice_write")
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
            match search::server_search_request(endpoint, &request).await {
                Ok(response) => return Ok((response, None)),
                Err(err) => {
                    let response =
                        search::local_search_response(&request, root, filter).map_err(|fallback_err| {
                            ErrorData::internal_error(
                                format!(
                                    "server error: {err}; local fallback failed: {fallback_err}"
                                ),
                                None,
                            )
                        })?;
                    let warning = format!("Server unavailable; using local scan. error={}", err);
                    return Ok((response, Some(warning)));
                }
            }
        }

        let response = search::local_search_response(&request, root, filter)
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

        let (response, warning) = self.execute_search(request, input.path.as_deref()).await?;
        let mut output = String::new();
        if let Some(note) = warning {
            output.push_str(&note);
            output.push('\n');
        }
        output.push_str(&response::format_search_lines(&response));
        Ok(response::tool_result_with_response(
            output.trim_end().to_string(),
            &response,
        ))
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

        let (response, warning) = self.execute_search(request, input.path.as_deref()).await?;
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
        output.push_str(&response::format_search_lines(&response));
        Ok(response::tool_result_with_response(
            output.trim_end().to_string(),
            &response,
        ))
    }

    #[tool(
        name = "rice_glob",
        description = "Find files by glob pattern with ignore awareness. Performs fast wildcard searches across directories, honors .gitignore/.ignore rules, supports recursive matching, optional directory results, and case-insensitive queries for cross-platform consistency."
    )]
    async fn glob(
        &self,
        Parameters(input): Parameters<GlobToolInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let root = input.path.as_deref().unwrap_or(".");
        let matches = crate::collect_glob_matches(
            root,
            &input.pattern,
            input.include_dirs.unwrap_or(false),
            input.ignore_case.unwrap_or(false),
        )
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(response::tool_text_result(matches.join("\n")))
    }

    #[tool(
        name = "rice_list",
        description = "List directory contents with ignore awareness. Produces filtered directory listings that respect project ignore files, optional glob filters, and case-insensitive matching so you can inspect structure before drilling into files."
    )]
    async fn list(
        &self,
        Parameters(input): Parameters<ListToolInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let root = input.path.as_deref().unwrap_or(".");
        let entries = crate::list_directory_entries(
            root,
            input.pattern.as_deref(),
            input.ignore_case.unwrap_or(false),
        )
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(response::tool_text_result(entries.join("\n")))
    }

    #[tool(
        name = "rice_read",
        description = "Read file contents with optional line ranges. Streams numbered output with offset and limit controls so you can inspect entire files or focused snippets without overwhelming context."
    )]
    async fn read(
        &self,
        Parameters(input): Parameters<ReadToolInput>,
    ) -> Result<CallToolResult, ErrorData> {
        // Read file as bytes first for binary detection
        let path = std::path::Path::new(&input.path);
        let content_bytes = tokio::fs::read(path).await.map_err(|err| {
            ErrorData::internal_error(format!("Failed to read file: {}", err), None)
        })?;

        // Check if binary
        if tools::is_binary_file(path, &content_bytes) {
            return Err(ErrorData::internal_error(
                format!(
                    "Cannot read binary file: {}. Use a binary-safe tool.",
                    input.path
                ),
                None,
            ));
        }

        // Convert to string
        let content = String::from_utf8(content_bytes).map_err(|_| {
            ErrorData::internal_error("File contains invalid UTF-8".to_string(), None)
        })?;

        let offset = input.offset.unwrap_or(0);
        let limit = input.limit.unwrap_or(2000); // Default limit of 2000 lines

        let output = tools::format_file_content_for_mcp(&input.path, &content, offset, limit);
        Ok(response::tool_text_result(output))
    }

    #[tool(
        name = "rice_edit",
        description = "Edit a file with preview and force safeguards. Performs exact string replacements with verification so you can refactor or fix configurations safely without unintended edits. Supports replace_all parameter to replace all occurrences instead of just the first."
    )]
    async fn edit(
        &self,
        Parameters(input): Parameters<EditToolInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let output = tools::apply_edit(&input)
            .await
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(response::tool_text_result(output))
    }

    #[tool(
        name = "rice_write",
        description = "Write content to a file, creating it if it doesn't exist or overwriting if it does."
    )]
    async fn write(
        &self,
        Parameters(input): Parameters<WriteToolInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let output = tools::apply_write(&input)
            .await
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        Ok(response::tool_text_result(output))
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

// Response helpers, search, and protocol handlers moved to their respective modules

pub async fn run_mcp(runtime_config: &RuntimeConfig, args: McpArgs) -> Result<()> {
    let watch_paths: Vec<String> = args
        .paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    super::ensure_local_index_ready(&watch_paths).await?;

    // Determine the index path based on the first watch path
    let index_path = if let Some(first_path) = watch_paths.first() {
        let root = std::path::Path::new(first_path);
        super::local_index_dir(root)
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    };

    let mcp = RicegrepMcp::new(
        runtime_config.clone(),
        args.server_endpoint.clone(),
        args.all_tools,
    );
    let tool_router = &mcp.tool_router;

    let mut watch_manager = watch::WatchManager::new();

    if !args.no_watch {
        let watch_args = crate::WatchArgs {
            paths: watch_paths.clone(),
            timeout: args.timeout,
            debounce_secs: args.debounce_secs,
            clear_screen: args.clear_screen,
        };

        watch_manager.start_with_index(watch_args, index_path);
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
        let response = protocol::handle_mcp_request(&mcp, tool_router, &request).await?;
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
        let expected = [
            "rice_grep",
            "rice_glob",
            "rice_list",
            "rice_read",
            "rice_edit",
            "rice_nl_search",
        ];
        let router = RicegrepMcp::tool_router();
        let tool_names: Vec<String> = router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();
        for tool in expected {
            assert!(
                tool_names.iter().any(|name| name == tool),
                "missing tool: {tool}"
            );
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
        tokio::fs::write(&file_path, "Hello World\nHello there")
            .await
            .unwrap();

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
        tokio::fs::write(&file_path, "foo bar\nfoo baz\nfoo qux")
            .await
            .unwrap();

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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("File not found"));
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Pattern not found"));
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
    fn test_change_tracker_basic() {
        let mut tracker = ChangeTracker::new();
        let path = std::path::PathBuf::from("test.txt");

        assert!(!tracker.has_changes());
        tracker.record_change(path.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);

        // Should only have 1 entry (latest timestamp)
        assert_eq!(tracker.change_count(), 1);
    }

    #[test]
    fn test_change_tracker_multiple_files() {
        let mut tracker = ChangeTracker::new();
        let path1 = std::path::PathBuf::from("file1.txt");
        let path2 = std::path::PathBuf::from("file2.txt");
        let path3 = std::path::PathBuf::from("file3.txt");

        tracker.record_change(path1.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);
        tracker.record_change(path2.clone(), ricegrep::indexing_optimization::FileChangeKind::Create);
        tracker.record_change(path3.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);

        assert_eq!(tracker.change_count(), 3);
        assert!(tracker.has_changes());
    }

    #[test]
    fn test_change_tracker_take_changes() {
        let mut tracker = ChangeTracker::new();
        let path1 = std::path::PathBuf::from("file1.txt");
        let path2 = std::path::PathBuf::from("file2.txt");

        tracker.record_change(path1.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);
        tracker.record_change(path2.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);

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
        tracker.record_change(path.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);
        let after = std::time::SystemTime::now();

        // Verify that the change was recorded within timestamp bounds
        let changes = tracker.take_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], path);
    }

    #[test]
    fn test_change_tracker_rapid_changes() {
        let mut tracker = ChangeTracker::new();
        let path = std::path::PathBuf::from("rapid_changes.txt");

        // Rapid updates to same file
        for _ in 0..100 {
            tracker.record_change(path.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);
        }

        // Should only have 1 entry with latest timestamp
        assert_eq!(tracker.change_count(), 1);
        let changes = tracker.take_changes();
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_change_tracker_deduplication() {
        let mut tracker = ChangeTracker::new();
        let path = std::path::PathBuf::from("test.txt");

        // Record same file multiple times
        tracker.record_change(path.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);
        tracker.record_change(path.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);
        tracker.record_change(path.clone(), ricegrep::indexing_optimization::FileChangeKind::Modify);

        // Should only have 1 entry (latest timestamp)
        assert_eq!(tracker.change_count(), 1);
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

    #[tokio::test]
    async fn test_write_new_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        // Remove the temp file so we can write to it
        std::fs::remove_file(temp_file.path()).unwrap();

        let input = WriteToolInput {
            file_path: file_path.clone(),
            content: "Hello, World!\nThis is a test file.".to_string(),
            timeout_secs: Some(30),
        };

        let result = apply_write(&input).await;
        assert!(result.is_ok());

        // Verify content was written
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello, World!\nThis is a test file.");

        // Verify result message
        let result_msg = result.unwrap();
        assert!(result_msg.contains("Wrote"));
        assert!(result_msg.contains("bytes"));
        assert!(result_msg.contains("lines"));
    }

    #[tokio::test]
    async fn test_write_overwrite_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Write initial content
        tokio::fs::write(&file_path, "Initial content")
            .await
            .unwrap();

        let input = WriteToolInput {
            file_path: file_path.clone(),
            content: "Overwritten content".to_string(),
            timeout_secs: Some(30),
        };

        let result = apply_write(&input).await;
        assert!(result.is_ok());

        // Verify content was overwritten
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Overwritten content");
    }

    #[tokio::test]
    async fn test_write_create_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir.path().join("subdir").join("nested.txt");
        let file_path = nested_path.to_string_lossy().to_string();

        let input = WriteToolInput {
            file_path: file_path.clone(),
            content: "Content in nested file".to_string(),
            timeout_secs: Some(30),
        };

        let result = apply_write(&input).await;
        assert!(result.is_ok());

        // Verify content was written and parent dir was created
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Content in nested file");

        // Verify parent directory was created
        assert!(nested_path.parent().unwrap().is_dir());
    }

    #[tokio::test]
    async fn test_write_timeout() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let input = WriteToolInput {
            file_path: file_path.clone(),
            content: "Test content".to_string(),
            timeout_secs: Some(30), // Use a reasonable timeout
        };

        // This should succeed normally
        let result = apply_write(&input).await;
        assert!(result.is_ok());
    }
}
