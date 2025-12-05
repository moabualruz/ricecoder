//! JSON-RPC 2.0 protocol handling

use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// JSON-RPC 2.0 request ID
pub type RequestId = u64;

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request method name
    pub method: String,
    /// Request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Request ID (required for requests expecting responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Response result (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Response error (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Response ID (matches request ID)
    pub id: RequestId,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification (request without ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Notification method name
    pub method: String,
    /// Notification parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 message (can be request, response, or notification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// Request message
    Request(JsonRpcRequest),
    /// Response message
    Response(JsonRpcResponse),
    /// Notification message
    Notification(JsonRpcNotification),
}

/// Handles JSON-RPC 2.0 protocol communication
pub struct JsonRpcHandler {
    /// Next request ID to use
    next_id: Arc<AtomicU64>,
}

impl JsonRpcHandler {
    /// Create a new JSON-RPC handler
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Generate the next request ID
    pub fn next_request_id(&self) -> RequestId {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Create a JSON-RPC request
    pub fn create_request(
        &self,
        method: impl Into<String>,
        params: Option<Value>,
    ) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: Some(self.next_request_id()),
        }
    }

    /// Create a JSON-RPC notification (no response expected)
    pub fn create_notification(
        &self,
        method: impl Into<String>,
        params: Option<Value>,
    ) -> JsonRpcNotification {
        JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }

    /// Serialize a request to JSON
    pub fn serialize_request(&self, request: &JsonRpcRequest) -> crate::error::Result<String> {
        serde_json::to_string(request)
            .map_err(|e| crate::error::ExternalLspError::ProtocolError(e.to_string()))
    }

    /// Serialize a notification to JSON
    pub fn serialize_notification(&self, notification: &JsonRpcNotification) -> crate::error::Result<String> {
        serde_json::to_string(notification)
            .map_err(|e| crate::error::ExternalLspError::ProtocolError(e.to_string()))
    }

    /// Parse a JSON-RPC response
    pub fn parse_response(&self, json: &str) -> crate::error::Result<JsonRpcResponse> {
        serde_json::from_str(json)
            .map_err(|e| crate::error::ExternalLspError::ProtocolError(format!("Failed to parse response: {}", e)))
    }

    /// Parse a JSON-RPC notification
    pub fn parse_notification(&self, json: &str) -> crate::error::Result<JsonRpcNotification> {
        serde_json::from_str(json)
            .map_err(|e| crate::error::ExternalLspError::ProtocolError(format!("Failed to parse notification: {}", e)))
    }

    /// Parse a JSON-RPC message (can be response or notification)
    pub fn parse_message(&self, json: &str) -> crate::error::Result<JsonRpcMessage> {
        serde_json::from_str(json)
            .map_err(|e| crate::error::ExternalLspError::ProtocolError(format!("Failed to parse message: {}", e)))
    }

    /// Check if a response indicates an error
    pub fn is_error_response(response: &JsonRpcResponse) -> bool {
        response.error.is_some()
    }

    /// Extract error message from response
    pub fn extract_error_message(response: &JsonRpcResponse) -> Option<String> {
        response.error.as_ref().map(|e| e.message.clone())
    }

    /// Create a JSON-RPC error response
    pub fn create_error_response(
        id: RequestId,
        code: i32,
        message: impl Into<String>,
    ) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

impl Default for JsonRpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_request() {
        let handler = JsonRpcHandler::new();
        let request = handler.create_request("initialize", Some(json!({"processId": 1234})));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "initialize");
        assert!(request.id.is_some());
        assert!(request.params.is_some());
    }

    #[test]
    fn test_create_notification() {
        let handler = JsonRpcHandler::new();
        let notification = handler.create_notification("initialized", None);

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "initialized");
        assert!(notification.params.is_none());
    }

    #[test]
    fn test_serialize_request() {
        let handler = JsonRpcHandler::new();
        let request = handler.create_request("test", None);
        let json = handler.serialize_request(&request).unwrap();

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test\""));
    }

    #[test]
    fn test_parse_response() {
        let handler = JsonRpcHandler::new();
        let json = r#"{"jsonrpc":"2.0","result":{"key":"value"},"id":1}"#;
        let response = handler.parse_response(json).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_parse_error_response() {
        let handler = JsonRpcHandler::new();
        let json = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
        let response = handler.parse_response(json).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);
    }

    #[test]
    fn test_request_id_increments() {
        let handler = JsonRpcHandler::new();
        let id1 = handler.next_request_id();
        let id2 = handler.next_request_id();
        let id3 = handler.next_request_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_is_error_response() {
        let error_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            }),
            id: 1,
        };

        assert!(JsonRpcHandler::is_error_response(&error_response));

        let success_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"key": "value"})),
            error: None,
            id: 1,
        };

        assert!(!JsonRpcHandler::is_error_response(&success_response));
    }
}
