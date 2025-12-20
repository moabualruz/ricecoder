//! Simple MCP server implementation for RiceGrep
//! Similar to mgrep's MCP implementation but lightweight

use crate::error::RiceGrepError;
use crate::search::{RegexSearchEngine, SearchEngine, SearchQuery, ProgressVerbosity};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::io::{self, Write, BufRead};

/// JSON-RPC message structure
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcMessage {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(flatten)]
    method: JsonRpcMethod,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
enum JsonRpcMethod {
    #[serde(rename = "initialize")]
    Initialize { params: InitializeParams },
    #[serde(rename = "tools/list")]
    ToolsList,
    #[serde(rename = "tools/call")]
    ToolsCall { params: ToolsCallParams },
}

#[derive(Debug, Serialize, Deserialize)]
struct InitializeParams {
    protocol_version: String,
    capabilities: serde_json::Value,
    client_info: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolsCallParams {
    name: String,
    arguments: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    result: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    error: JsonRpcErrorData,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcErrorData {
    code: i32,
    message: String,
}

/// RiceGrep MCP Server implementation
pub struct RiceGrepMcpServer {
    /// Search engine instance
    search_engine: Arc<Mutex<RegexSearchEngine>>,
}

impl RiceGrepMcpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        Self {
            search_engine: Arc::new(Mutex::new(RegexSearchEngine::new())),
        }
    }

    /// Start the MCP server in stdio mode (similar to mgrep)
    pub async fn start_stdio_server(self) -> Result<(), RiceGrepError> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = io::BufReader::new(stdin);
        let mut writer = io::BufWriter::new(stdout);

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    match self.handle_message(&line.trim()).await {
                        Ok(Some(response)) => {
                            if let Err(e) = writeln!(writer, "{}", response) {
                                eprintln!("Failed to write response: {}", e);
                                break;
                            }
                            if let Err(e) = writer.flush() {
                                eprintln!("Failed to flush response: {}", e);
                                break;
                            }
                        }
                        Ok(None) => {} // No response needed
                        Err(e) => {
                            eprintln!("Error handling message: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming MCP messages
    async fn handle_message(&self, message: &str) -> Result<Option<String>, RiceGrepError> {
        let request: JsonRpcMessage = serde_json::from_str(message)
            .map_err(|e| RiceGrepError::Mcp {
                message: format!("Failed to parse message: {}", e),
            })?;

        let response = match request.method {
            JsonRpcMethod::Initialize { params } => {
                self.handle_initialize(params, request.id).await?
            }
            JsonRpcMethod::ToolsList => {
                self.handle_tools_list(request.id).await?
            }
            JsonRpcMethod::ToolsCall { params } => {
                self.handle_tools_call(params, request.id).await?
            }
        };

        Ok(Some(serde_json::to_string(&response)
            .map_err(|e| RiceGrepError::Mcp {
                message: format!("Failed to serialize response: {}", e),
            })?))
    }

    /// Handle initialize request
    async fn handle_initialize(&self, _params: InitializeParams, id: Option<serde_json::Value>) -> Result<JsonRpcResponse, RiceGrepError> {
        let result = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "ricegrep",
                "version": "0.1.0"
            }
        });

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        })
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, id: Option<serde_json::Value>) -> Result<JsonRpcResponse, RiceGrepError> {
        let tools = vec![
            serde_json::json!({
                "name": "search",
                "description": "Search for patterns in codebases with AI enhancement",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Search pattern (regex or literal)"
                        },
                        "path": {
                            "type": "string",
                            "description": "Search path",
                            "default": "."
                        },
                        "ai_enhanced": {
                            "type": "boolean",
                            "description": "Enable AI-enhanced search",
                            "default": false
                        }
                    },
                    "required": ["pattern"]
                }
            })
        ];

        let result = serde_json::json!({ "tools": tools });

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        })
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, params: ToolsCallParams, id: Option<serde_json::Value>) -> Result<JsonRpcResponse, RiceGrepError> {
        let result = match params.name.as_str() {
            "search" => {
                self.handle_search_tool(params.arguments).await?
            }
            _ => {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: serde_json::json!({
                        "content": [{ "type": "text", "text": "Tool not implemented" }],
                        "isError": true
                    }),
                });
            }
        };

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        })
    }

    /// Handle search tool call
    async fn handle_search_tool(&self, arguments: Option<serde_json::Value>) -> Result<serde_json::Value, RiceGrepError> {
        let empty_map = serde_json::Map::new();
        let args = arguments
            .as_ref()
            .and_then(|a| a.as_object())
            .unwrap_or(&empty_map);

        let pattern = args.get("pattern")
            .and_then(|p| p.as_str())
            .ok_or_else(|| RiceGrepError::Mcp {
                message: "Missing required 'pattern' argument".to_string(),
            })?;

        let path = args.get("path")
            .and_then(|p| p.as_str())
            .unwrap_or(".");

        let ai_enhanced = args.get("ai_enhanced")
            .and_then(|a| a.as_bool())
            .unwrap_or(false);

        // Create search query
        let query = SearchQuery {
            pattern: pattern.to_string(),
            paths: vec![std::path::PathBuf::from(path)],
            case_insensitive: false,
            case_sensitive: false,
            word_regexp: false,
            fixed_strings: false,
            follow: false,
            hidden: false,
            no_ignore: false,
            ignore_file: None,
            quiet: true,
            dry_run: false,
            max_file_size: None,
            progress_verbosity: ProgressVerbosity::Quiet,
            max_files: None,
            max_matches: Some(100),
            max_lines: None,
            invert_match: false,
            ai_enhanced,
            no_rerank: false,
            fuzzy: None,
            max_count: Some(100),
            spelling_correction: None,
        };

        // Execute search
        let mut search_engine = self.search_engine.lock().await;
        let results = search_engine.search(query).await?;

        // Format results for MCP
        let content = format!(
            "Found {} matches in {} files\n{}",
            results.total_matches,
            results.files_searched,
            results.matches.iter()
                .take(10) // Limit to first 10 results
                .map(|m| format!("{}:{}: {}", m.file.display(), m.line_number, m.line_content.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(serde_json::json!({
            "content": [{ "type": "text", "text": content }]
        }))
    }
}

impl Default for RiceGrepMcpServer {
    fn default() -> Self {
        Self::new()
    }
}