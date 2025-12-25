//! MCP Protocol Handlers
//!
//! Handles JSON-RPC protocol messages for the MCP server.

use anyhow::Result;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};

use super::types::{
    EditToolInput, GlobToolInput, GrepToolInput, ListToolInput, NlSearchToolInput, ReadToolInput,
    WriteToolInput,
};
use super::RicegrepMcp;

/// Helper function to get tool title for display
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

/// Helper function to get tool annotations (safety metadata)
fn tool_annotations(name: &str) -> serde_json::Value {
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

/// Helper function to get tool output schema
fn tool_output_schema(name: &str) -> serde_json::Value {
    match name {
        "rice_grep" | "rice_nl_search" => search_output_schema(),
        _ => text_only_output_schema(),
    }
}

/// Schema for text-only output
fn text_only_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "content": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "type": { "type": "string", "enum": ["text"] },
                        "text": { "type": "string" }
                    }
                }
            }
        }
    })
}

/// Schema for search result output
fn search_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "content": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "type": { "type": "string", "enum": ["text"] },
                        "text": { "type": "string" }
                    }
                }
            },
            "isError": { "type": "boolean" }
        }
    })
}

/// Handles MCP JSON-RPC protocol requests
///
/// Supports:
/// - `initialize`: Server initialization and capability negotiation
/// - `tools/list`: List available tools with metadata
/// - `tools/call`: Execute a specific tool
///
/// Returns JSON-RPC 2.0 compliant responses
pub async fn handle_mcp_request(
    mcp: &RicegrepMcp,
    tool_router: &ToolRouter<RicegrepMcp>,
    request: &serde_json::Value,
) -> Result<serde_json::Value> {
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

/// Calls a specific tool by name with the provided arguments
///
/// Deserializes arguments into the appropriate input type and
/// executes the corresponding method on the RicegrepMcp struct.
///
/// Returns the tool result as a JSON value
pub async fn call_tool(
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
        "rice_write" => {
            let input: WriteToolInput = serde_json::from_value(arguments)?;
            mcp.write(Parameters(input)).await?
        }
        _ => return Ok(serde_json::json!([])),
    };
    Ok(serde_json::to_value(&result)?)
}
