//! Tool invoker implementations for ricecoder-tools
//!
//! This module provides implementations of the ToolInvoker trait for each tool
//! provided by ricecoder-tools (webfetch, patch, todowrite, todoread, websearch).
//!
//! These invokers wire the agent system to the actual tool implementations
//! in the ricecoder-tools crate.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::tool_registry::{ToolInvoker, ToolMetadata};

// Import actual tool implementations
use ricecoder_tools::{
    webfetch::{WebfetchTool, WebfetchInput, OutputFormat},
    search::{SearchTool, SearchInput, SearchType},
    todo::{Todo, TodoStatus, TodoPriority, TodoTools, TodowriteInput, TodoreadInput},
    patch::{PatchTool, PatchInput},
    read::{FileReadTool, FileReadInput},
    write::{WriteTool, WriteInput},
    edit::{FileEditTool, FileEditInput},
    list::{ListTool, ListInput},
    glob::{GlobTool, GlobInput},
    grep::{GrepTool, GrepInput},
    context::ToolContext,
};

/// Webfetch tool invoker
///
/// Invokes the webfetch tool to fetch web content from URLs.
pub struct WebfetchToolInvoker;

#[async_trait::async_trait]
impl ToolInvoker for WebfetchToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking webfetch tool");

        // Extract URL from input
        let url = input
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' field in input".to_string())?;

        // Extract optional format
        let format = input
            .get("format")
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "markdown" => OutputFormat::Markdown,
                "html" => OutputFormat::Html,
                _ => OutputFormat::Text,
            })
            .unwrap_or(OutputFormat::Text);

        // Extract optional timeout
        let timeout = input
            .get("timeout")
            .and_then(|v| v.as_u64());

        // Extract optional max_size
        let max_size = input
            .get("max_size")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        info!(url = %url, ?format, "Fetching web content");

        // Create webfetch tool and execute
        let tool = WebfetchTool::new()
            .map_err(|e| format!("Failed to create webfetch tool: {}", e.message))?;

        let mut webfetch_input = WebfetchInput::new(url).with_format(format);
        if let Some(t) = timeout {
            webfetch_input = webfetch_input.with_timeout(t);
        }
        if let Some(size) = max_size {
            webfetch_input = webfetch_input.with_max_size(size);
        }

        let result = tool.fetch(webfetch_input).await;

        if result.success {
            if let Some(data) = result.data {
                Ok(json!({
                    "success": true,
                    "content": data.content,
                    "url": url,
                    "metadata": {
                        "provider": result.metadata.provider,
                        "truncated": data.truncated,
                        "size": data.returned_size,
                        "original_size": data.original_size,
                        "from_cache": data.from_cache,
                        "duration_ms": result.metadata.duration_ms
                    }
                }))
            } else {
                Err("Webfetch succeeded but returned no data".to_string())
            }
        } else if let Some(err) = result.error {
            Err(format!("{}: {}", err.code, err.message))
        } else {
            Err("Webfetch failed with unknown error".to_string())
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "webfetch".to_string(),
            name: "Webfetch".to_string(),
            description: "Fetch and process web content from URLs".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to fetch"
                    },
                    "max_size": {
                        "type": "integer",
                        "description": "Maximum content size in bytes (optional)"
                    }
                },
                "required": ["url"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "content": { "type": "string" },
                    "url": { "type": "string" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" },
                            "truncated": { "type": "boolean" },
                            "size": { "type": "integer" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Patch tool invoker
///
/// Invokes the patch tool to apply unified diff patches to files.
pub struct PatchToolInvoker;

#[async_trait::async_trait]
impl ToolInvoker for PatchToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking patch tool");

        // Extract file path and patch content from input
        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'file_path' field in input".to_string())?;

        let patch_content = input
            .get("patch_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'patch_content' field in input".to_string())?;

        info!(file_path = %file_path, "Applying patch");

        // Create patch input and apply
        let patch_input = PatchInput {
            file_path: file_path.to_string(),
            patch_content: patch_content.to_string(),
        };

        // Use async version with timeout
        match PatchTool::apply_patch_with_timeout(&patch_input).await {
            Ok(output) => {
                Ok(json!({
                    "success": output.success,
                    "applied_hunks": output.applied_hunks,
                    "failed_hunks": output.failed_hunks,
                    "file_path": file_path,
                    "failed_hunk_details": output.failed_hunk_details,
                    "metadata": {
                        "provider": "builtin"
                    }
                }))
            }
            Err(e) => {
                Err(format!("{}: {}", e.code, e.message))
            }
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "patch".to_string(),
            name: "Patch".to_string(),
            description: "Apply unified diff patches to files safely".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to patch"
                    },
                    "patch_content": {
                        "type": "string",
                        "description": "Unified diff patch content"
                    }
                },
                "required": ["file_path", "patch_content"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "applied_hunks": { "type": "integer" },
                    "failed_hunks": { "type": "integer" },
                    "file_path": { "type": "string" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Todowrite tool invoker
///
/// Invokes the todowrite tool to create or update todos.
pub struct TodowriteToolInvoker;

#[async_trait::async_trait]
impl ToolInvoker for TodowriteToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking todowrite tool");

        // Parse todos from input
        let todos_value = input
            .get("todos")
            .ok_or_else(|| "Missing 'todos' field in input".to_string())?;

        // Parse todos array
        let todos: Vec<Todo> = serde_json::from_value(todos_value.clone())
            .map_err(|e| format!("Failed to parse todos: {}", e))?;

        info!(todo_count = todos.len(), "Writing todos");

        // Create TodoTools and write
        let todo_tools = TodoTools::new()
            .map_err(|e| format!("Failed to create todo tools: {}", e.message))?;

        let write_input = TodowriteInput { todos };

        // Use async version with timeout
        match todo_tools.write_todos_with_timeout(write_input, None).await {
            Ok(output) => {
                Ok(json!({
                    "success": true,
                    "title": output.title,
                    "output": output.output,
                    "created": output.metadata.created,
                    "updated": output.metadata.updated,
                    "todos": output.metadata.todos,
                    "metadata": {
                        "provider": "builtin"
                    }
                }))
            }
            Err(e) => {
                Err(format!("{}: {}", e.code, e.message))
            }
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "todowrite".to_string(),
            name: "Todowrite".to_string(),
            description: "Create or update todos in the task list".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "todos": {
                        "type": "array",
                        "description": "List of todos to create or update",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "title": { "type": "string" },
                                "description": { "type": "string" },
                                "status": { "type": "string" },
                                "priority": { "type": "string" }
                            }
                        }
                    }
                },
                "required": ["todos"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "created": { "type": "integer" },
                    "updated": { "type": "integer" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Todoread tool invoker
///
/// Invokes the todoread tool to read todos from the task list.
pub struct TodoreadToolInvoker;

#[async_trait::async_trait]
impl ToolInvoker for TodoreadToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking todoread tool");

        // Extract optional filters from input
        let status_filter = input
            .get("status")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<TodoStatus>().ok());

        let priority_filter = input
            .get("priority")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<TodoPriority>().ok());

        info!(
            status_filter = ?status_filter,
            priority_filter = ?priority_filter,
            "Reading todos"
        );

        // Create TodoTools and read
        let todo_tools = TodoTools::new()
            .map_err(|e| format!("Failed to create todo tools: {}", e.message))?;

        let read_input = TodoreadInput {
            status_filter,
            priority_filter,
        };

        // Use async version with timeout
        match todo_tools.read_todos_with_timeout(read_input, None).await {
            Ok(output) => {
                Ok(json!({
                    "success": true,
                    "title": output.title,
                    "output": output.output,
                    "todos": output.metadata.todos,
                    "metadata": {
                        "provider": "builtin"
                    }
                }))
            }
            Err(e) => {
                Err(format!("{}: {}", e.code, e.message))
            }
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "todoread".to_string(),
            name: "Todoread".to_string(),
            description: "Read todos from the task list with optional filtering".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "description": "Filter by status (optional)"
                    },
                    "priority": {
                        "type": "string",
                        "description": "Filter by priority (optional)"
                    }
                }
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "todos": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "title": { "type": "string" },
                                "description": { "type": "string" },
                                "status": { "type": "string" },
                                "priority": { "type": "string" }
                            }
                        }
                    },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Websearch tool invoker
///
/// Invokes the websearch tool to search the web.
pub struct WebsearchToolInvoker;

#[async_trait::async_trait]
impl ToolInvoker for WebsearchToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking websearch tool");

        // Extract query from input
        let query = input
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'query' field in input".to_string())?;

        let limit = input.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
        let offset = input.get("offset").and_then(|v| v.as_u64()).map(|v| v as usize);

        // Extract optional search type
        let search_type = input
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "fast" => SearchType::Fast,
                "deep" => SearchType::Deep,
                _ => SearchType::Auto,
            })
            .unwrap_or(SearchType::Auto);

        info!(query = %query, ?limit, ?offset, ?search_type, "Searching web");

        // Create search tool and execute
        let tool = SearchTool::new();

        let mut search_input = SearchInput::new(query).with_search_type(search_type);
        if let Some(l) = limit {
            search_input = search_input.with_limit(l);
        }
        if let Some(o) = offset {
            search_input = search_input.with_offset(o);
        }

        let result = tool.search(search_input).await;

        if result.success {
            if let Some(data) = result.data {
                Ok(json!({
                    "success": true,
                    "results": data.results,
                    "total_count": data.total_count,
                    "query": query,
                    "metadata": {
                        "provider": result.metadata.provider,
                        "limit": limit.unwrap_or(8),
                        "offset": offset.unwrap_or(0),
                        "duration_ms": result.metadata.duration_ms
                    }
                }))
            } else {
                Err("Search succeeded but returned no data".to_string())
            }
        } else if let Some(err) = result.error {
            Err(format!("{}: {}", err.code, err.message))
        } else {
            Err("Search failed with unknown error".to_string())
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "websearch".to_string(),
            name: "Websearch".to_string(),
            description: "Search the web and return ranked results".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (optional, default: 10)"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Result offset for pagination (optional, default: 0)"
                    }
                },
                "required": ["query"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "results": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string" },
                                "url": { "type": "string" },
                                "snippet": { "type": "string" },
                                "rank": { "type": "integer" }
                            }
                        }
                    },
                    "total_count": { "type": "integer" },
                    "query": { "type": "string" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" },
                            "limit": { "type": "integer" },
                            "offset": { "type": "integer" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Read tool invoker
///
/// Invokes the read tool to read file contents.
pub struct ReadToolInvoker;

#[async_trait::async_trait]
impl ToolInvoker for ReadToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking read tool");

        // Extract file path from input (supports both file_path and filePath)
        let file_path = input
            .get("file_path")
            .or_else(|| input.get("filePath"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'file_path' or 'filePath' field in input".to_string())?;

        // Extract optional parameters
        let offset = input.get("offset").and_then(|v| v.as_u64()).map(|v| v as usize);
        let limit = input.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
        let line_numbers = input.get("line_numbers").and_then(|v| v.as_bool());

        info!(file_path = %file_path, ?offset, ?limit, "Reading file");

        // Create read input
        let read_input = FileReadInput {
            file_path: file_path.to_string(),
            start_line: None,
            end_line: None,
            offset,
            limit,
            max_size_bytes: None,
            detect_binary: Some(true),
            content_filter: None,
            line_numbers,
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(true),
            return_attachments: Some(true),
        };

        // Execute synchronous read
        match FileReadTool::read_file(&read_input) {
            Ok(output) => {
                if output.success {
                    Ok(json!({
                        "success": true,
                        "content": output.content,
                        "file_path": file_path,
                        "lines_read": output.lines_read,
                        "total_lines": output.total_lines,
                        "file_size": output.file_size,
                        "is_binary": output.is_binary,
                        "mime_type": output.mime_type,
                        "metadata": {
                            "provider": "builtin"
                        }
                    }))
                } else {
                    Err(output.error.unwrap_or_else(|| "Unknown read error".to_string()))
                }
            }
            Err(e) => Err(format!("{}: {}", e.code, e.message)),
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "read".to_string(),
            name: "Read".to_string(),
            description: "Read file contents with line numbers and pagination".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Line offset to start reading from (0-based)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of lines to read (default: 2000)"
                    }
                },
                "required": ["file_path"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "content": { "type": "string" },
                    "file_path": { "type": "string" },
                    "lines_read": { "type": "integer" },
                    "total_lines": { "type": "integer" },
                    "file_size": { "type": "integer" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Write tool invoker
///
/// Invokes the write tool to create or overwrite files.
pub struct WriteToolInvoker {
    workspace_root: std::path::PathBuf,
}

impl WriteToolInvoker {
    /// Create a new write tool invoker with the given workspace root
    pub fn new(workspace_root: std::path::PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait::async_trait]
impl ToolInvoker for WriteToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking write tool");

        // Extract file path from input (supports both file_path and filePath)
        let file_path = input
            .get("file_path")
            .or_else(|| input.get("filePath"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'file_path' or 'filePath' field in input".to_string())?;

        // Extract content
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'content' field in input".to_string())?;

        info!(file_path = %file_path, content_len = content.len(), "Writing file");

        // Create write tool and execute
        let mut write_tool = WriteTool::new(self.workspace_root.clone());
        
        let write_input = WriteInput {
            file_path: file_path.to_string(),
            content: content.to_string(),
        };

        match write_tool.execute(write_input).await {
            Ok(output) => {
                Ok(json!({
                    "success": true,
                    "title": output.title,
                    "file_path": output.metadata.filepath,
                    "existed": output.metadata.exists,
                    "diagnostics": output.output,
                    "metadata": {
                        "provider": "builtin"
                    }
                }))
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "write".to_string(),
            name: "Write".to_string(),
            description: "Write content to a file (create or overwrite)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["file_path", "content"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "title": { "type": "string" },
                    "file_path": { "type": "string" },
                    "existed": { "type": "boolean" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Edit tool invoker
///
/// Invokes the edit tool to perform string replacements in files.
pub struct EditToolInvoker;

#[async_trait::async_trait]
impl ToolInvoker for EditToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking edit tool");

        // Extract file path from input (supports both file_path and filePath)
        let file_path = input
            .get("file_path")
            .or_else(|| input.get("filePath"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'file_path' or 'filePath' field in input".to_string())?;

        // Extract old_string and new_string (supports camelCase aliases)
        let old_string = input
            .get("old_string")
            .or_else(|| input.get("oldString"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'old_string' or 'oldString' field in input".to_string())?;

        let new_string = input
            .get("new_string")
            .or_else(|| input.get("newString"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'new_string' or 'newString' field in input".to_string())?;

        // Extract optional replace_all flag
        let replace_all = input
            .get("replace_all")
            .or_else(|| input.get("replaceAll"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!(file_path = %file_path, replace_all = replace_all, "Editing file");

        // Create edit input
        let edit_input = FileEditInput {
            file_path: file_path.to_string(),
            old_string: old_string.to_string(),
            new_string: new_string.to_string(),
            start_line: None,
            end_line: None,
            replace_all,
        };

        // Execute synchronous edit
        match FileEditTool::edit_file(&edit_input) {
            Ok(output) => {
                if output.success {
                    Ok(json!({
                        "success": true,
                        "file_path": file_path,
                        "strategy_used": output.strategy_used,
                        "strategies_attempted": output.strategies_attempted,
                        "diff": output.diff,
                        "metadata": {
                            "provider": "builtin"
                        }
                    }))
                } else {
                    // Return structured error with closest match info if available
                    let mut error_response = json!({
                        "success": false,
                        "error": output.error,
                        "strategies_attempted": output.strategies_attempted
                    });
                    
                    if let Some(closest) = output.closest_match {
                        error_response["closest_match"] = json!({
                            "strategy": closest.strategy,
                            "similarity": closest.similarity,
                            "line_number": closest.line_number,
                            "matched_text": closest.matched_text
                        });
                    }
                    
                    Ok(error_response)
                }
            }
            Err(e) => Err(format!("{}: {}", e.code, e.message)),
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "edit".to_string(),
            name: "Edit".to_string(),
            description: "Edit files by replacing text with multiple matching strategies".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "Text to find and replace"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "Text to replace with"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "Replace all occurrences (default: false)"
                    }
                },
                "required": ["file_path", "old_string", "new_string"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "file_path": { "type": "string" },
                    "strategy_used": { "type": "string" },
                    "diff": { "type": "string" },
                    "error": { "type": "string" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

// =============================================================================
// List Tool Invoker
// =============================================================================

/// List directory tool invoker
///
/// Lists directory contents with filtering and ignore pattern support.
pub struct ListToolInvoker {
    workspace_root: PathBuf,
}

impl ListToolInvoker {
    /// Create a new list tool invoker
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait::async_trait]
impl ToolInvoker for ListToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking list tool");

        // Parse input
        let list_input: ListInput = serde_json::from_value(input)
            .map_err(|e| format!("Invalid list input: {}", e))?;

        // Create tool and execute
        let tool = ListTool::new(self.workspace_root.clone());
        let ctx = ToolContext::new("session".to_string(), "message".to_string(), "list".to_string());

        let result = tool.list_directory(&list_input, &ctx).await;

        match result {
            Ok(output) => Ok(json!({
                "success": true,
                "output": output.output,
                "count": output.count,
                "truncated": output.truncated,
                "path": output.path
            })),
            Err(e) => Err(format!("List failed: {}", e.message)),
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "list".to_string(),
            name: "List".to_string(),
            description: "Lists files and directories in a given path with tree output".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The absolute path to list (defaults to workspace root)"
                    },
                    "ignore": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of glob patterns to ignore"
                    }
                }
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "output": { "type": "string" },
                    "count": { "type": "integer" },
                    "truncated": { "type": "boolean" },
                    "path": { "type": "string" }
                }
            }),
            available: true,
        }
    }
}

// =============================================================================
// Glob Tool Invoker
// =============================================================================

/// Glob pattern matching tool invoker
///
/// Fast file pattern matching with safety limits.
pub struct GlobToolInvoker {
    workspace_root: PathBuf,
}

impl GlobToolInvoker {
    /// Create a new glob tool invoker
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait::async_trait]
impl ToolInvoker for GlobToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking glob tool");

        // Parse input
        let glob_input: GlobInput = serde_json::from_value(input)
            .map_err(|e| format!("Invalid glob input: {}", e))?;

        // Create tool and execute
        let tool = GlobTool::new(self.workspace_root.clone());
        let ctx = ToolContext::new("session".to_string(), "message".to_string(), "glob".to_string());

        let result = tool.find_files(&glob_input, &ctx).await;

        match result {
            Ok(output) => Ok(json!({
                "success": true,
                "files": output.files,
                "count": output.count,
                "truncated": output.truncated
            })),
            Err(e) => Err(format!("Glob failed: {}", e.message)),
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "glob".to_string(),
            name: "Glob".to_string(),
            description: "Fast file pattern matching with safety limits (100 file limit)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern like '**/*.js' or 'src/**/*.ts'"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (defaults to workspace root)"
                    }
                },
                "required": ["pattern"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "files": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "count": { "type": "integer" },
                    "truncated": { "type": "boolean" }
                }
            }),
            available: true,
        }
    }
}

// =============================================================================
// Grep Tool Invoker
// =============================================================================

/// Grep content search tool invoker
///
/// Fast content search with regex support and safety limits.
pub struct GrepToolInvoker {
    workspace_root: PathBuf,
}

impl GrepToolInvoker {
    /// Create a new grep tool invoker
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait::async_trait]
impl ToolInvoker for GrepToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking grep tool");

        // Parse input
        let grep_input: GrepInput = serde_json::from_value(input)
            .map_err(|e| format!("Invalid grep input: {}", e))?;

        // Create tool and execute
        let tool = GrepTool::new(self.workspace_root.clone());
        let ctx = ToolContext::new("session".to_string(), "message".to_string(), "grep".to_string());

        let result = tool.search(&grep_input, &ctx).await;

        match result {
            Ok(output) => {
                // Format matches for output
                let matches: Vec<serde_json::Value> = output.matches.iter().map(|m| {
                    json!({
                        "file": m.file,
                        "line": m.line,
                        "content": m.content
                    })
                }).collect();

                Ok(json!({
                    "success": true,
                    "matches": matches,
                    "count": output.count,
                    "truncated": output.truncated
                }))
            },
            Err(e) => Err(format!("Grep failed: {}", e.message)),
        }
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "grep".to_string(),
            name: "Grep".to_string(),
            description: "Fast content search with regex support (100 match limit)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (defaults to workspace root)"
                    },
                    "include": {
                        "type": "string",
                        "description": "File pattern to include (e.g., '*.rs', '*.{ts,tsx}')"
                    }
                },
                "required": ["pattern"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "matches": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "file": { "type": "string" },
                                "line": { "type": "integer" },
                                "content": { "type": "string" }
                            }
                        }
                    },
                    "count": { "type": "integer" },
                    "truncated": { "type": "boolean" }
                }
            }),
            available: true,
        }
    }
}

// =============================================================================
// Extensible Tool Invoker
// =============================================================================

/// Extensible tool invoker
///
/// A generic tool invoker that can be configured with different backends
/// including MCP servers, external APIs, and custom implementations.
pub struct ExtensibleToolInvoker {
    backend: Arc<RwLock<Option<Box<dyn ToolBackend + Send + Sync>>>>,
}

impl ExtensibleToolInvoker {
    /// Create a new extensible tool invoker
    pub fn new() -> Self {
        Self {
            backend: Arc::new(RwLock::new(None)),
        }
    }

    /// Configure the tool backend
    pub async fn configure_backend<T: ToolBackend + Send + Sync + 'static>(
        &self,
        backend: T,
    ) -> Result<(), String> {
        let mut backend_slot = self.backend.write().await;
        *backend_slot = Some(Box::new(backend));
        Ok(())
    }

    /// Check if a backend is configured
    pub async fn has_backend(&self) -> bool {
        let backend = self.backend.read().await;
        backend.is_some()
    }
}

#[async_trait::async_trait]
impl ToolInvoker for ExtensibleToolInvoker {
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!("Invoking extensible tool");

        let backend = self.backend.read().await;
        let backend = backend
            .as_ref()
            .ok_or_else(|| "No tool backend configured".to_string())?;

        backend.invoke_tool(input).await
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "extensible".to_string(),
            name: "Extensible Tool Invoker".to_string(),
            description: "Execute tools through configurable backends (MCP, APIs, etc.)"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "description": "Name of the tool to execute"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Parameters to pass to the tool"
                    },
                    "backend_config": {
                        "type": "object",
                        "description": "Backend-specific configuration"
                    }
                },
                "required": ["tool_name"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "result": { "type": "object" },
                    "error": { "type": "string" },
                    "execution_time_ms": { "type": "integer" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "provider": { "type": "string" },
                            "backend": { "type": "string" }
                        }
                    }
                }
            }),
            available: true,
        }
    }
}

/// Tool backend trait for extensible tool execution
#[async_trait::async_trait]
pub trait ToolBackend {
    /// Invoke a tool with the given input
    async fn invoke_tool(&self, input: serde_json::Value) -> Result<serde_json::Value, String>;

    /// Get backend-specific metadata
    fn backend_metadata(&self) -> serde_json::Value;
}
