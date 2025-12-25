//! MCP Response Helpers
//!
//! Functions for building tool metadata, schemas, and formatting responses.

use ricegrep::api::models::SearchResponse;
use rmcp::model::{CallToolResult, Content};
use std::collections::HashMap;

/// Get human-readable title for a tool
pub fn tool_title(name: &str) -> &'static str {
    match name {
        "grep" => "File Content Search",
        "rice_grep" => "File Content Search",
        "rice_nl_search" => "Natural Language Search",
        "rice_glob" => "File Glob Finder",
        "rice_list" => "Directory Lister",
        "rice_read" => "File Reader",
        "rice_edit" => "File Editor",
        "rice_write" => "File Writer",
        _ => "Ricegrep Tool",
    }
}

/// Get tool metadata annotations (safety, idempotency, destructiveness)
pub fn tool_annotations(name: &str) -> serde_json::Value {
    let (safe, idempotent, destructive) = match name {
        "rice_edit" | "rice_write" => (false, false, true),
        _ => (true, true, false),
    };
    serde_json::json!({
        "audience": ["user", "assistant"],
        "priority": 0.85,
        "safe": safe,
        "idempotent": idempotent,
        "destructive": destructive
    })
}

/// Get tool output schema (varies by tool type)
pub fn tool_output_schema(name: &str) -> serde_json::Value {
    match name {
        "grep" | "rice_grep" | "rice_nl_search" => search_output_schema(),
        _ => text_only_output_schema(),
    }
}

/// Schema for text-only tool responses
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

/// Schema for search tool responses with structured results
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

/// Create a simple text-based tool result
pub fn tool_text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(text)],
        is_error: None,
        meta: None,
        structured_content: None,
    }
}

/// Create a tool result with both text and structured search response
pub fn tool_result_with_response(text: String, response: &SearchResponse) -> CallToolResult {
    let structured = serde_json::to_value(response).ok();
    CallToolResult {
        content: vec![Content::text(text)],
        is_error: None,
        meta: None,
        structured_content: structured,
    }
}

/// Format search results as human-readable lines (OpenCode-compatible)
pub fn format_search_lines(response: &SearchResponse) -> String {
    format_search_lines_opencode_style(response, false)
}

/// Format search results in OpenCode grep.ts format
/// 
/// Output format:
/// ```
/// Found N matches
/// 
/// <path>:
///   Line <n>: <text>
///   Line <m>: <text>
/// 
/// <path2>:
///   Line <x>: <text>
/// ```
pub fn format_search_lines_opencode_style(
    response: &SearchResponse,
    truncated: bool,
) -> String {
    const MAX_LINE_LENGTH: usize = 2000;
    
    if response.results.is_empty() {
        return "No files found".to_string();
    }

    let mut output = vec![format!("Found {} matches", response.total_found)];
    output.push(String::new()); // Blank line

    let mut current_file = String::new();
    for result in &response.results {
        let file_path = result.metadata.file_path.display().to_string();
        
        // Add file header if changed
        if current_file != file_path {
            if !current_file.is_empty() {
                output.push(String::new()); // Blank line between files
            }
            current_file = file_path.clone();
            output.push(format!("{}:", file_path));
        }

        // Truncate line if too long
        let line_text = if result.content.len() > MAX_LINE_LENGTH {
            format!("{}...", &result.content[..MAX_LINE_LENGTH])
        } else {
            result.content.clone()
        };

        output.push(format!("  Line {}: {}", result.metadata.start_line, line_text));
    }

    if truncated {
        output.push(String::new());
        output.push("(Results are truncated. Consider using a more specific path or pattern.)".to_string());
    }

    output.join("\n")
}

/// Sort search results by file modification time (newest first)
/// GAP-3 implementation: Match OpenCode grep.ts mtime sorting behavior
pub async fn sort_results_by_mtime(response: &mut SearchResponse) {
    use tokio::fs;
    use std::time::SystemTime;

    // Collect unique file paths and their mtimes
    let mut file_mtimes: HashMap<std::path::PathBuf, SystemTime> = HashMap::new();
    
    for result in &response.results {
        let path = &result.metadata.file_path;
        if !file_mtimes.contains_key(path) {
            // Stat the file to get mtime
            if let Ok(metadata) = fs::metadata(path).await {
                if let Ok(mtime) = metadata.modified() {
                    file_mtimes.insert(path.clone(), mtime);
                }
            }
        }
    }

    // Sort results by file mtime (newest first), then by line number within each file
    response.results.sort_by(|a, b| {
        let a_mtime = file_mtimes.get(&a.metadata.file_path);
        let b_mtime = file_mtimes.get(&b.metadata.file_path);
        
        match (a_mtime, b_mtime) {
            (Some(a_time), Some(b_time)) => {
                // Sort by mtime descending (newest first)
                b_time.cmp(a_time).then_with(|| {
                    // Within same file, sort by line number ascending
                    if a.metadata.file_path == b.metadata.file_path {
                        a.metadata.start_line.cmp(&b.metadata.start_line)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.metadata.start_line.cmp(&b.metadata.start_line),
        }
    });
}
