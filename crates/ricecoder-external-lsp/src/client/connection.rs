//! LSP client connection management

use super::protocol::{JsonRpcHandler, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId};
use crate::error::{ExternalLspError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

/// A pending request awaiting a response
pub struct PendingRequest {
    /// Request ID
    pub id: RequestId,
    /// Request method name
    pub method: String,
    /// Time when request was sent
    pub sent_at: Instant,
    /// Request timeout
    pub timeout: Duration,
    /// Response channel sender
    pub response_tx: tokio::sync::oneshot::Sender<Result<Value>>,
}

/// Notification handler callback
pub type NotificationHandler = Box<dyn Fn(&str, Option<Value>) + Send + Sync>;

/// Manages connection to an external LSP server
pub struct LspConnection {
    /// JSON-RPC protocol handler
    handler: JsonRpcHandler,
    /// Pending requests awaiting responses
    pending_requests: Arc<RwLock<HashMap<RequestId, PendingRequest>>>,
    /// Notification broadcast channel
    notification_tx: broadcast::Sender<(String, Option<Value>)>,
}

impl LspConnection {
    /// Create a new LSP connection
    pub fn new() -> Self {
        let (notification_tx, _) = broadcast::channel(100);
        Self {
            handler: JsonRpcHandler::new(),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            notification_tx,
        }
    }

    /// Get the JSON-RPC handler
    pub fn handler(&self) -> &JsonRpcHandler {
        &self.handler
    }

    /// Create a new request and track it
    pub async fn create_tracked_request(
        &self,
        method: impl Into<String>,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<(JsonRpcRequest, tokio::sync::oneshot::Receiver<Result<Value>>)> {
        let request = self.handler.create_request(method.into(), params);
        let request_id = request.id.ok_or_else(|| {
            ExternalLspError::ProtocolError("Request ID not set".to_string())
        })?;

        let (tx, rx) = tokio::sync::oneshot::channel();

        let pending = PendingRequest {
            id: request_id,
            method: request.method.clone(),
            sent_at: Instant::now(),
            timeout,
            response_tx: tx,
        };

        self.pending_requests.write().await.insert(request_id, pending);

        Ok((request, rx))
    }

    /// Handle a response and correlate it to a pending request
    pub async fn handle_response(&self, response: JsonRpcResponse) -> Result<()> {
        let mut pending = self.pending_requests.write().await;

        if let Some(pending_req) = pending.remove(&response.id) {
            // Check if request timed out
            if pending_req.sent_at.elapsed() > pending_req.timeout {
                return Err(ExternalLspError::Timeout {
                    timeout_ms: pending_req.timeout.as_millis() as u64,
                });
            }

            // Send response to waiting task
            let result = if let Some(error) = response.error {
                Err(ExternalLspError::ProtocolError(format!(
                    "{}: {}",
                    error.code, error.message
                )))
            } else {
                Ok(response.result.unwrap_or(Value::Null))
            };

            // Ignore send error if receiver was dropped
            let _ = pending_req.response_tx.send(result);

            Ok(())
        } else {
            Err(ExternalLspError::ProtocolError(format!(
                "Received response for unknown request ID: {}",
                response.id
            )))
        }
    }

    /// Get pending request count
    pub async fn pending_request_count(&self) -> usize {
        self.pending_requests.read().await.len()
    }

    /// Check for timed out requests and clean them up
    pub async fn cleanup_timed_out_requests(&self) -> Vec<RequestId> {
        let mut pending = self.pending_requests.write().await;
        let mut timed_out = Vec::new();
        let mut to_remove = Vec::new();

        for (id, req) in pending.iter() {
            if req.sent_at.elapsed() > req.timeout {
                timed_out.push(*id);
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            if let Some(pending_req) = pending.remove(&id) {
                // Send timeout error to waiting task
                let _ = pending_req.response_tx.send(Err(ExternalLspError::Timeout {
                    timeout_ms: pending_req.timeout.as_millis() as u64,
                }));
            }
        }

        timed_out
    }

    /// Clear all pending requests
    pub async fn clear_pending_requests(&self) {
        self.pending_requests.write().await.clear();
    }

    /// Get list of pending request IDs
    pub async fn get_pending_request_ids(&self) -> Vec<RequestId> {
        self.pending_requests.read().await.keys().copied().collect()
    }

    /// Handle a notification from the server
    pub async fn handle_notification(&self, notification: JsonRpcNotification) -> Result<()> {
        // Broadcast notification to all subscribers
        let _ = self.notification_tx.send((notification.method, notification.params));
        Ok(())
    }

    /// Subscribe to notifications
    pub fn subscribe_notifications(&self) -> broadcast::Receiver<(String, Option<Value>)> {
        self.notification_tx.subscribe()
    }

    /// Handle textDocument/publishDiagnostics notification
    pub async fn handle_publish_diagnostics(
        &self,
        params: Option<Value>,
    ) -> Result<()> {
        self.handle_notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        })
        .await
    }

    /// Handle window/logMessage notification
    pub async fn handle_log_message(&self, params: Option<Value>) -> Result<()> {
        self.handle_notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "window/logMessage".to_string(),
            params,
        })
        .await
    }

    /// Handle window/showMessage notification
    pub async fn handle_show_message(&self, params: Option<Value>) -> Result<()> {
        self.handle_notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "window/showMessage".to_string(),
            params,
        })
        .await
    }

    /// Send textDocument/didOpen notification
    pub async fn send_did_open(
        &self,
        uri: String,
        language_id: String,
        version: i32,
        text: String,
    ) -> Result<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri,
                "languageId": language_id,
                "version": version,
                "text": text
            }
        });

        self.handle_notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/didOpen".to_string(),
            params: Some(params),
        })
        .await
    }

    /// Send textDocument/didChange notification
    pub async fn send_did_change(
        &self,
        uri: String,
        version: i32,
        content_changes: Vec<Value>,
    ) -> Result<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri,
                "version": version
            },
            "contentChanges": content_changes
        });

        self.handle_notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/didChange".to_string(),
            params: Some(params),
        })
        .await
    }

    /// Send textDocument/didClose notification
    pub async fn send_did_close(&self, uri: String) -> Result<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri
            }
        });

        self.handle_notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/didClose".to_string(),
            params: Some(params),
        })
        .await
    }

    /// Send textDocument/didSave notification
    pub async fn send_did_save(&self, uri: String, text: Option<String>) -> Result<()> {
        let mut params = serde_json::json!({
            "textDocument": {
                "uri": uri
            }
        });

        if let Some(text) = text {
            params["text"] = serde_json::json!(text);
        }

        self.handle_notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/didSave".to_string(),
            params: Some(params),
        })
        .await
    }
}

impl Default for LspConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_tracked_request() {
        let conn = LspConnection::new();
        let (request, _rx) = conn
            .create_tracked_request("test", None, Duration::from_secs(5))
            .await
            .unwrap();

        assert_eq!(request.method, "test");
        assert!(request.id.is_some());
        assert_eq!(conn.pending_request_count().await, 1);
    }

    #[tokio::test]
    async fn test_handle_response() {
        let conn = LspConnection::new();
        let (request, rx) = conn
            .create_tracked_request("test", None, Duration::from_secs(5))
            .await
            .unwrap();

        let request_id = request.id.unwrap();

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("success".to_string())),
            error: None,
            id: request_id,
        };

        conn.handle_response(response).await.unwrap();

        let result = rx.await.unwrap().unwrap();
        assert_eq!(result, Value::String("success".to_string()));
        assert_eq!(conn.pending_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_handle_error_response() {
        let conn = LspConnection::new();
        let (request, rx) = conn
            .create_tracked_request("test", None, Duration::from_secs(5))
            .await
            .unwrap();

        let request_id = request.id.unwrap();

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(crate::client::protocol::JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            }),
            id: request_id,
        };

        conn.handle_response(response).await.unwrap();

        let result = rx.await.unwrap();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_timed_out_requests() {
        let conn = LspConnection::new();
        let (request, _rx) = conn
            .create_tracked_request("test", None, Duration::from_millis(1))
            .await
            .unwrap();

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(10)).await;

        let timed_out = conn.cleanup_timed_out_requests().await;
        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out[0], request.id.unwrap());
        assert_eq!(conn.pending_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_unknown_response_id() {
        let conn = LspConnection::new();

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("success".to_string())),
            error: None,
            id: 999,
        };

        let result = conn.handle_response(response).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_notification() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test/notification".to_string(),
            params: Some(Value::String("test".to_string())),
        };

        conn.handle_notification(notification).await.unwrap();

        let (method, params) = rx.recv().await.unwrap();
        assert_eq!(method, "test/notification");
        assert_eq!(params, Some(Value::String("test".to_string())));
    }

    #[tokio::test]
    async fn test_handle_publish_diagnostics() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        let params = Some(serde_json::json!({
            "uri": "file:///test.rs",
            "diagnostics": []
        }));

        conn.handle_publish_diagnostics(params.clone())
            .await
            .unwrap();

        let (method, received_params) = rx.recv().await.unwrap();
        assert_eq!(method, "textDocument/publishDiagnostics");
        assert_eq!(received_params, params);
    }

    #[tokio::test]
    async fn test_handle_log_message() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        let params = Some(serde_json::json!({
            "type": 1,
            "message": "Test log message"
        }));

        conn.handle_log_message(params.clone()).await.unwrap();

        let (method, received_params) = rx.recv().await.unwrap();
        assert_eq!(method, "window/logMessage");
        assert_eq!(received_params, params);
    }

    #[tokio::test]
    async fn test_handle_show_message() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        let params = Some(serde_json::json!({
            "type": 1,
            "message": "Test show message"
        }));

        conn.handle_show_message(params.clone()).await.unwrap();

        let (method, received_params) = rx.recv().await.unwrap();
        assert_eq!(method, "window/showMessage");
        assert_eq!(received_params, params);
    }

    #[tokio::test]
    async fn test_multiple_notification_subscribers() {
        let conn = LspConnection::new();
        let mut rx1 = conn.subscribe_notifications();
        let mut rx2 = conn.subscribe_notifications();

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: None,
        };

        conn.handle_notification(notification).await.unwrap();

        let (method1, _) = rx1.recv().await.unwrap();
        let (method2, _) = rx2.recv().await.unwrap();

        assert_eq!(method1, "test");
        assert_eq!(method2, "test");
    }

    #[tokio::test]
    async fn test_send_did_open() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications(); // mut needed for recv()

        conn.send_did_open(
            "file:///test.rs".to_string(),
            "rust".to_string(),
            1,
            "fn main() {}".to_string(),
        )
        .await
        .unwrap();

        let (method, params) = rx.recv().await.unwrap();
        assert_eq!(method, "textDocument/didOpen");
        assert!(params.is_some());

        let params = params.unwrap();
        assert_eq!(params["textDocument"]["uri"], "file:///test.rs");
        assert_eq!(params["textDocument"]["languageId"], "rust");
        assert_eq!(params["textDocument"]["version"], 1);
        assert_eq!(params["textDocument"]["text"], "fn main() {}");
    }

    #[tokio::test]
    async fn test_send_did_change() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        let changes = vec![serde_json::json!({
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 0}
            },
            "text": "// comment\n"
        })];

        conn.send_did_change("file:///test.rs".to_string(), 2, changes.clone())
            .await
            .unwrap();

        let (method, params) = rx.recv().await.unwrap();
        assert_eq!(method, "textDocument/didChange");
        assert!(params.is_some());

        let params = params.unwrap();
        assert_eq!(params["textDocument"]["uri"], "file:///test.rs");
        assert_eq!(params["textDocument"]["version"], 2);
        assert_eq!(params["contentChanges"], serde_json::json!(changes));
    }

    #[tokio::test]
    async fn test_send_did_close() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        conn.send_did_close("file:///test.rs".to_string())
            .await
            .unwrap();

        let (method, params) = rx.recv().await.unwrap();
        assert_eq!(method, "textDocument/didClose");
        assert!(params.is_some());

        let params = params.unwrap();
        assert_eq!(params["textDocument"]["uri"], "file:///test.rs");
    }

    #[tokio::test]
    async fn test_send_did_save() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        conn.send_did_save("file:///test.rs".to_string(), Some("fn main() {}".to_string()))
            .await
            .unwrap();

        let (method, params) = rx.recv().await.unwrap();
        assert_eq!(method, "textDocument/didSave");
        assert!(params.is_some());

        let params = params.unwrap();
        assert_eq!(params["textDocument"]["uri"], "file:///test.rs");
        assert_eq!(params["text"], "fn main() {}");
    }

    #[tokio::test]
    async fn test_send_did_save_without_text() {
        let conn = LspConnection::new();
        let mut rx = conn.subscribe_notifications();

        conn.send_did_save("file:///test.rs".to_string(), None)
            .await
            .unwrap();

        let (method, params) = rx.recv().await.unwrap();
        assert_eq!(method, "textDocument/didSave");
        assert!(params.is_some());

        let params = params.unwrap();
        assert_eq!(params["textDocument"]["uri"], "file:///test.rs");
        assert!(params.get("text").is_none());
    }
}
