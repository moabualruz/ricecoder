//! JSON-RPC message transport over stdio
//!
//! This module handles the low-level communication protocol for LSP,
//! including message framing with Content-Length headers and JSON-RPC
//! message parsing and serialization.

use crate::types::{LspError, LspResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

/// JSON-RPC request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID
    pub id: serde_json::Value,
    /// Method name
    pub method: String,
    /// Request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: serde_json::Value, method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
        }
    }
}

/// JSON-RPC response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID
    pub id: serde_json::Value,
    /// Response result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Response error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(id: serde_json::Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    /// Parse error (-32700)
    pub fn parse_error(message: String) -> Self {
        Self::new(-32700, message)
    }

    /// Invalid request (-32600)
    pub fn invalid_request(message: String) -> Self {
        Self::new(-32600, message)
    }

    /// Method not found (-32601)
    pub fn method_not_found(method: String) -> Self {
        Self::new(-32601, format!("Method not found: {}", method))
    }

    /// Invalid params (-32602)
    pub fn invalid_params(message: String) -> Self {
        Self::new(-32602, message)
    }

    /// Internal error (-32603)
    pub fn internal_error(message: String) -> Self {
        Self::new(-32603, message)
    }

    /// Server error (-32000 to -32099)
    pub fn server_error(code: i32, message: String) -> Self {
        Self::new(code, message)
    }
}

/// JSON-RPC notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Notification parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification
    pub fn new(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
        }
    }
}

/// LSP message (can be request, response, or notification)
#[derive(Debug, Clone)]
pub enum LspMessage {
    /// Request message
    Request(JsonRpcRequest),
    /// Response message
    Response(JsonRpcResponse),
    /// Notification message
    Notification(JsonRpcNotification),
}

impl LspMessage {
    /// Parse a message from JSON
    pub fn from_json(json: &str) -> LspResult<Self> {
        let value: Value = serde_json::from_str(json)
            .map_err(|e| LspError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        // Check if it's a response (has result or error)
        if value.get("result").is_some() || value.get("error").is_some() {
            let response: JsonRpcResponse = serde_json::from_value(value)
                .map_err(|e| LspError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(LspMessage::Response(response))
        }
        // Check if it's a request (has id and method)
        else if value.get("id").is_some() && value.get("method").is_some() {
            let request: JsonRpcRequest = serde_json::from_value(value)
                .map_err(|e| LspError::ParseError(format!("Failed to parse request: {}", e)))?;
            Ok(LspMessage::Request(request))
        }
        // Otherwise it's a notification (has method but no id)
        else if value.get("method").is_some() {
            let notification: JsonRpcNotification = serde_json::from_value(value)
                .map_err(|e| LspError::ParseError(format!("Failed to parse notification: {}", e)))?;
            Ok(LspMessage::Notification(notification))
        } else {
            Err(LspError::InvalidRequest(
                "Message must be a request, response, or notification".to_string(),
            ))
        }
    }

    /// Serialize message to JSON
    pub fn to_json(&self) -> LspResult<String> {
        match self {
            LspMessage::Request(req) => serde_json::to_string(req)
                .map_err(|e| LspError::SerializationError(format!("Failed to serialize request: {}", e))),
            LspMessage::Response(resp) => serde_json::to_string(resp)
                .map_err(|e| LspError::SerializationError(format!("Failed to serialize response: {}", e))),
            LspMessage::Notification(notif) => serde_json::to_string(notif)
                .map_err(|e| LspError::SerializationError(format!("Failed to serialize notification: {}", e))),
        }
    }
}

/// Stdio transport for LSP messages
pub struct StdioTransport {
    reader: BufReader<io::Stdin>,
    writer: io::Stdout,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(io::stdin()),
            writer: io::stdout(),
        }
    }

    /// Read a message from stdin
    pub fn read_message(&mut self) -> LspResult<LspMessage> {
        let mut headers = HashMap::new();

        // Read headers
        loop {
            let mut line = String::new();
            self.reader
                .read_line(&mut line)
                .map_err(|e| LspError::IoError(format!("Failed to read header: {}", e)))?;

            if line == "\r\n" || line == "\n" {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // Get content length
        let content_length: usize = headers
            .get("Content-Length")
            .ok_or_else(|| LspError::InvalidRequest("Missing Content-Length header".to_string()))?
            .parse()
            .map_err(|e| LspError::InvalidRequest(format!("Invalid Content-Length: {}", e)))?;

        // Read content
        let mut content = vec![0u8; content_length];
        use std::io::Read;
        self.reader
            .read_exact(&mut content)
            .map_err(|e| LspError::IoError(format!("Failed to read content: {}", e)))?;

        let json = String::from_utf8(content)
            .map_err(|e| LspError::ParseError(format!("Invalid UTF-8: {}", e)))?;

        LspMessage::from_json(&json)
    }

    /// Write a message to stdout
    pub fn write_message(&mut self, message: &LspMessage) -> LspResult<()> {
        let json = message.to_json()?;
        let content_length = json.len();

        write!(
            self.writer,
            "Content-Length: {}\r\n\r\n{}",
            content_length, json
        )
        .map_err(|e| LspError::IoError(format!("Failed to write message: {}", e)))?;

        self.writer
            .flush()
            .map_err(|e| LspError::IoError(format!("Failed to flush output: {}", e)))?;

        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// Async stdio transport for LSP messages
pub struct AsyncStdioTransport {
    reader: tokio::io::BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
}

impl AsyncStdioTransport {
    /// Create a new async stdio transport
    pub fn new() -> Self {
        Self {
            reader: tokio::io::BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
        }
    }

    /// Read a message from stdin asynchronously
    pub async fn read_message(&mut self) -> LspResult<LspMessage> {
        let mut headers = HashMap::new();

        // Read headers
        loop {
            let mut line = String::new();
            self.reader
                .read_line(&mut line)
                .await
                .map_err(|e| LspError::IoError(format!("Failed to read header: {}", e)))?;

            if line == "\r\n" || line == "\n" {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // Get content length
        let content_length: usize = headers
            .get("Content-Length")
            .ok_or_else(|| LspError::InvalidRequest("Missing Content-Length header".to_string()))?
            .parse()
            .map_err(|e| LspError::InvalidRequest(format!("Invalid Content-Length: {}", e)))?;

        // Read content
        let mut content = vec![0u8; content_length];
        use tokio::io::AsyncReadExt;
        self.reader
            .read_exact(&mut content)
            .await
            .map_err(|e| LspError::IoError(format!("Failed to read content: {}", e)))?;

        let json = String::from_utf8(content)
            .map_err(|e| LspError::ParseError(format!("Invalid UTF-8: {}", e)))?;

        LspMessage::from_json(&json)
    }

    /// Write a message to stdout asynchronously
    pub async fn write_message(&mut self, message: &LspMessage) -> LspResult<()> {
        let json = message.to_json()?;
        let content_length = json.len();

        self.writer
            .write_all(format!("Content-Length: {}\r\n\r\n{}", content_length, json).as_bytes())
            .await
            .map_err(|e| LspError::IoError(format!("Failed to write message: {}", e)))?;

        self.writer
            .flush()
            .await
            .map_err(|e| LspError::IoError(format!("Failed to flush output: {}", e)))?;

        Ok(())
    }
}

impl Default for AsyncStdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_creation() {
        let req = JsonRpcRequest::new(
            json!(1),
            "initialize".to_string(),
            Some(json!({"processId": 1234})),
        );
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "initialize");
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let resp = JsonRpcResponse::success(json!(1), json!({"capabilities": {}}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let error = JsonRpcError::parse_error("Invalid JSON".to_string());
        let resp = JsonRpcResponse::error(json!(1), error);
        assert!(resp.error.is_some());
        assert!(resp.result.is_none());
    }

    #[test]
    fn test_jsonrpc_notification_creation() {
        let notif = JsonRpcNotification::new("initialized".to_string(), None);
        assert_eq!(notif.jsonrpc, "2.0");
        assert_eq!(notif.method, "initialized");
    }

    #[test]
    fn test_lsp_message_from_request_json() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":1234}}"#;
        let msg = LspMessage::from_json(json_str).unwrap();
        match msg {
            LspMessage::Request(req) => {
                assert_eq!(req.method, "initialize");
            }
            _ => panic!("Expected request"),
        }
    }

    #[test]
    fn test_lsp_message_from_notification_json() {
        let json_str = r#"{"jsonrpc":"2.0","method":"initialized"}"#;
        let msg = LspMessage::from_json(json_str).unwrap();
        match msg {
            LspMessage::Notification(notif) => {
                assert_eq!(notif.method, "initialized");
            }
            _ => panic!("Expected notification"),
        }
    }

    #[test]
    fn test_lsp_message_to_json() {
        let req = JsonRpcRequest::new(json!(1), "initialize".to_string(), None);
        let msg = LspMessage::Request(req);
        let json_str = msg.to_json().unwrap();
        assert!(json_str.contains("initialize"));
    }

    #[test]
    fn test_jsonrpc_error_codes() {
        let parse_err = JsonRpcError::parse_error("test".to_string());
        assert_eq!(parse_err.code, -32700);

        let invalid_req = JsonRpcError::invalid_request("test".to_string());
        assert_eq!(invalid_req.code, -32600);

        let method_not_found = JsonRpcError::method_not_found("test".to_string());
        assert_eq!(method_not_found.code, -32601);

        let invalid_params = JsonRpcError::invalid_params("test".to_string());
        assert_eq!(invalid_params.code, -32602);

        let internal_err = JsonRpcError::internal_error("test".to_string());
        assert_eq!(internal_err.code, -32603);
    }
}
