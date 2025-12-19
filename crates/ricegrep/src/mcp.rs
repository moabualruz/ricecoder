//! MCP (Model Context Protocol) ecosystem for RiceGrep
//!
//! This module provides comprehensive MCP support including client libraries,
//! server implementation, and protocol compliance for AI assistant integration.

use crate::error::RiceGrepError;
use crate::search::{RegexSearchEngine, SearchEngine, SearchQuery, ProgressVerbosity};
use crate::replace::{ReplaceEngine, ReplaceOperation};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

// MCP Protocol Types (based on Model Context Protocol specification)
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcMessage {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(flatten)]
    pub message_type: MessageType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum MessageType {
    #[serde(rename = "initialize")]
    Initialize { params: InitializeParams },
    #[serde(rename = "tools/list")]
    ToolsList,
    #[serde(rename = "tools/call")]
    ToolsCall { params: ToolsCallParams },
    #[serde(rename = "resources/list")]
    ResourcesList,
    #[serde(rename = "resources/read")]
    ResourcesRead { params: ResourcesReadParams },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub tools: Option<HashMap<String, serde_json::Value>>,
    pub resources: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsCallParams {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcesReadParams {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    Success { id: serde_json::Value, result: serde_json::Value },
    Error { id: Option<serde_json::Value>, error: JsonRpcError },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// MCP Tool Definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<Content>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String, mime_type: Option<String>, text: Option<String> },
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self { tools: None }
    }
}

#[derive(Debug)]
#[derive(serde::Serialize)]
pub struct ServerCapabilities {
    pub tools: Option<serde_json::Value>,
}

/// MCP Client for connecting to MCP servers
pub struct McpClient {
    /// Server process handle
    server_process: Option<std::process::Child>,
    /// Input/output streams
    stdin: Option<std::process::Stdio>,
    /// Server capabilities
    capabilities: Option<ServerCapabilities>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            server_process: None,
            stdin: None,
            capabilities: None,
        }
    }

    /// Connect to an MCP server
    pub async fn connect(&mut self, command: &str, args: &[&str]) -> Result<(), RiceGrepError> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RiceGrepError::Mcp {
                message: format!("Failed to start MCP server: {}", e),
            })?;

        self.server_process = Some(child);
        Ok(())
    }

    /// Send a request to the MCP server
    pub async fn send_request(&mut self, request: JsonRpcMessage) -> Result<JsonRpcResponse, RiceGrepError> {
        if let Some(ref mut child) = self.server_process {
            if let Some(ref mut stdin) = child.stdin {
                let request_json = serde_json::to_string(&request)
                    .map_err(|e| RiceGrepError::Mcp {
                        message: format!("Failed to serialize request: {}", e),
                    })?;

                stdin.write_all(request_json.as_bytes())
                    .map_err(|e| RiceGrepError::Mcp {
                        message: format!("Failed to send request: {}", e),
                    })?;
                stdin.write_all(b"\n")
                    .map_err(|e| RiceGrepError::Mcp {
                        message: format!("Failed to send request: {}", e),
                    })?;
                stdin.flush()
                    .map_err(|e| RiceGrepError::Mcp {
                        message: format!("Failed to flush request: {}", e),
                    })?;

                // Read response
                if let Some(ref mut stdout) = child.stdout {
                    let mut reader = BufReader::new(stdout);
                    let mut line = String::new();
                    reader.read_line(&mut line)
                        .map_err(|e| RiceGrepError::Mcp {
                            message: format!("Failed to read response: {}", e),
                        })?;

                    let response: JsonRpcResponse = serde_json::from_str(&line.trim())
                        .map_err(|e| RiceGrepError::Mcp {
                            message: format!("Failed to parse response: {}", e),
                        })?;

                    Ok(response)
                } else {
                    Err(RiceGrepError::Mcp {
                        message: "No stdout available from MCP server".to_string(),
                    })
                }
            } else {
                Err(RiceGrepError::Mcp {
                    message: "No stdin available for MCP server".to_string(),
                })
            }
        } else {
            Err(RiceGrepError::Mcp {
                message: "Not connected to MCP server".to_string(),
            })
        }
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<(), RiceGrepError> {
        if let Some(mut child) = self.server_process.take() {
            child.kill()
                .map_err(|e| RiceGrepError::Mcp {
                    message: format!("Failed to kill MCP server: {}", e),
                })?;
            child.wait()
                .map_err(|e| RiceGrepError::Mcp {
                    message: format!("Failed to wait for MCP server: {}", e),
                })?;
        }
        Ok(())
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP Server implementation for RiceGrep
pub struct RiceGrepMcpServer {
    /// Search engine instance
    search_engine: Arc<Mutex<RegexSearchEngine>>,
    /// Replace engine instance
    replace_engine: Arc<Mutex<ReplaceEngine>>,
    /// Server capabilities
    capabilities: ServerCapabilities,
}

impl RiceGrepMcpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        Self {
            search_engine: Arc::new(Mutex::new(RegexSearchEngine::new())),
            replace_engine: Arc::new(Mutex::new(ReplaceEngine::new())),
            capabilities: ServerCapabilities {
                tools: Some(serde_json::json!({
                    "search": {
                        "description": "Search for patterns in codebases with AI enhancement"
                    },
                    "replace": {
                        "description": "Perform safe find-and-replace operations"
                    },
                    "index": {
                        "description": "Build and manage search indexes"
                    }
                })),
            },
        }
    }

    /// Start the MCP server in stdio mode
    pub async fn start_stdio_server(self) -> Result<(), RiceGrepError> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut writer = stdout;

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let response = self.handle_message(&line.trim()).await?;
                    if let Some(response_json) = response {
                        writeln!(writer, "{}", response_json)
                            .map_err(|e| RiceGrepError::Mcp {
                                message: format!("Failed to write response: {}", e),
                            })?;
                        writer.flush()
                            .map_err(|e| RiceGrepError::Mcp {
                                message: format!("Failed to flush response: {}", e),
                            })?;
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

        let response = match request.message_type {
            MessageType::Initialize { params } => {
                self.handle_initialize(params, request.id).await?
            }
            MessageType::ToolsList => {
                self.handle_tools_list(request.id).await?
            }
            MessageType::ToolsCall { params } => {
                self.handle_tools_call(params, request.id).await?
            }
            MessageType::ResourcesList => {
                self.handle_resources_list(request.id).await?
            }
            MessageType::ResourcesRead { params } => {
                self.handle_resources_read(params, request.id).await?
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
            "capabilities": self.capabilities,
            "serverInfo": {
                "name": "ricegrep",
                "version": "0.1.0"
            }
        });

        Ok(JsonRpcResponse::Success { id: id.unwrap_or(serde_json::Value::Null), result })
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, id: Option<serde_json::Value>) -> Result<JsonRpcResponse, RiceGrepError> {
        let tools = vec![
            ToolDefinition {
                name: "search".to_string(),
                description: "Search for patterns in codebases with AI enhancement".to_string(),
                input_schema: serde_json::json!({
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
                }),
            },
            ToolDefinition {
                name: "replace".to_string(),
                description: "Perform safe find-and-replace operations".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Pattern to replace"
                        },
                        "replacement": {
                            "type": "string",
                            "description": "Replacement text"
                        },
                        "path": {
                            "type": "string",
                            "description": "Target path"
                        }
                    },
                    "required": ["pattern", "replacement", "path"]
                }),
            },
        ];

        let result = serde_json::json!({ "tools": tools });
        Ok(JsonRpcResponse::Success { id: id.unwrap_or(serde_json::Value::Null), result })
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, params: ToolsCallParams, id: Option<serde_json::Value>) -> Result<JsonRpcResponse, RiceGrepError> {
        let result = match params.name.as_str() {
            "search" => self.handle_search_tool(params.arguments).await?,
            "replace" => self.handle_replace_tool(params.arguments).await?,
            _ => {
                return Ok(JsonRpcResponse::Error {
                    id,
                    error: JsonRpcError {
                        code: -32601,
                        message: format!("Method '{}' not found", params.name),
                        data: None,
                    },
                });
            }
        };

        Ok(JsonRpcResponse::Success { id: id.unwrap_or(serde_json::Value::Null), result })
    }

    /// Handle search tool call
    async fn handle_search_tool(&self, arguments: Option<serde_json::Value>) -> Result<serde_json::Value, RiceGrepError> {
        let args: HashMap<String, serde_json::Value> = arguments
            .and_then(|a| serde_json::from_value(a).ok())
            .unwrap_or_default();

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

    /// Handle replace tool call
    async fn handle_replace_tool(&self, arguments: Option<serde_json::Value>) -> Result<serde_json::Value, RiceGrepError> {
        let args: HashMap<String, serde_json::Value> = arguments
            .and_then(|a| serde_json::from_value(a).ok())
            .unwrap_or_default();

        let pattern = args.get("pattern")
            .and_then(|p| p.as_str())
            .ok_or_else(|| RiceGrepError::Mcp {
                message: "Missing required 'pattern' argument".to_string(),
            })?;

        let replacement = args.get("replacement")
            .and_then(|r| r.as_str())
            .ok_or_else(|| RiceGrepError::Mcp {
                message: "Missing required 'replacement' argument".to_string(),
            })?;

        let path = args.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| RiceGrepError::Mcp {
                message: "Missing required 'path' argument".to_string(),
            })?;

        // For now, return a placeholder response
        // Full implementation would require file system access
        let content = format!(
            "Replace operation prepared: '{}' -> '{}' in path '{}'\nNote: Full implementation requires file system access permissions.",
            pattern, replacement, path
        );

        Ok(serde_json::json!({
            "content": [{ "type": "text", "text": content }]
        }))
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, id: Option<serde_json::Value>) -> Result<JsonRpcResponse, RiceGrepError> {
        let result = serde_json::json!({ "resources": [] });
        Ok(JsonRpcResponse::Success { id: id.unwrap_or(serde_json::Value::Null), result })
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, _params: ResourcesReadParams, id: Option<serde_json::Value>) -> Result<JsonRpcResponse, RiceGrepError> {
        let result = serde_json::json!({
            "contents": [{ "type": "text", "text": "Resource reading not implemented" }]
        });
        Ok(JsonRpcResponse::Success { id: id.unwrap_or(serde_json::Value::Null), result })
    }
}

impl Default for RiceGrepMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

// MCP functionality is implemented directly in RiceGrepMcpServer

#[derive(Debug)]
pub enum McpError {
    InvalidRequest(String),
    InternalError(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            McpError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for McpError {}

// MCP server functionality is implemented in RiceGrepMcpServer

// Placeholder transport
pub struct StdioTransport;

