//! MCP Response Helpers
//!
//! Functions for building tool metadata, schemas, and formatting responses.

use ricegrep::api::models::SearchResponse;
use rmcp::model::{CallToolResult, Content};

/// Get human-readable title for a tool
pub fn tool_title(name: &str) -> &'static str {
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
        "rice_grep" | "rice_nl_search" => search_output_schema(),
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

/// Format search results as human-readable lines
pub fn format_search_lines(response: &SearchResponse) -> String {
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
