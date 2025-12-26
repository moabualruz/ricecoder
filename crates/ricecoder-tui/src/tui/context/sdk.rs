//! SDK Client Context Provider (DEPRECATED)
//!
//! **Status**: This SDK layer is deprecated. Real backend integration is now handled by `AppContext`.
//!
//! ## Migration Guide
//! - Use `AppContext::send_message()` for AI chat (blocking)
//! - Use `AppContext::send_message_streaming()` for streaming responses
//! - Use `AppContext::load_sessions()` and `AppState.sessions` for session data
//! - Use `AppContext::load_providers()` and `AppState.providers` for provider info
//!
//! ## Why Deprecated?
//! The SDK layer was originally designed as an intermediary to a backend server.
//! However, the TUI now integrates directly with backend crates:
//! - `ricecoder-providers` for AI provider management
//! - `ricecoder-sessions` for session persistence
//! - `ricecoder-mcp` for MCP server status
//!
//! This module remains for backwards compatibility and event subscription patterns,
//! but all placeholder methods now point to AppContext for real implementations.

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};

/// SDK events from the backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SdkEvent {
    /// Session created
    SessionCreated {
        session_id: String,
    },
    /// Message received
    MessageReceived {
        session_id: String,
        message_id: String,
        content: String,
    },
    /// Provider status changed
    ProviderStatusChanged {
        provider_id: String,
        status: String,
    },
    /// MCP status changed
    McpStatusChanged {
        name: String,
        status: String,
    },
    /// Model list updated
    ModelListUpdated {
        provider_id: String,
    },
    /// Generic event for extensibility
    Generic {
        event_type: String,
        data: serde_json::Value,
    },
}

/// SDK client interface (mock for now, will integrate with actual API)
#[derive(Debug, Clone)]
pub struct SdkClient {
    base_url: String,
}

impl SdkClient {
    /// Create new SDK client
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Send chat message (DEPRECATED - use AppContext::send_message)
    /// 
    /// This method is a placeholder. Real message sending is now handled by:
    /// - `AppContext::send_message()` for blocking requests
    /// - `AppContext::send_message_streaming()` for streaming responses
    /// 
    /// Those methods integrate with ricecoder-providers for actual AI calls.
    pub async fn send_message(
        &self,
        session_id: &str,
        content: String,
    ) -> Result<String, String> {
        // SDK layer is deprecated - real implementation in AppContext
        Ok(format!(
            "DEPRECATED: Use AppContext::send_message() instead. Session: {}, Message: {}",
            session_id, content
        ))
    }

    /// List sessions (DEPRECATED - use AppContext::load_sessions)
    /// 
    /// This method returns empty data. Real session listing is now handled by:
    /// - `AppContext::load_sessions()` during initialization
    /// - Session data is stored in `AppState.sessions`
    /// 
    /// The AppContext integrates with ricecoder-sessions for session persistence.
    pub async fn list_sessions(&self) -> Result<Vec<String>, String> {
        // SDK layer is deprecated - real session data comes from AppContext state
        Ok(vec![])
    }

    /// Get provider info (DEPRECATED - use AppContext::load_providers)
    /// 
    /// This method returns minimal placeholder data. Real provider info is now available via:
    /// - `AppContext::load_providers()` during initialization
    /// - Provider data is stored in `AppState.providers`
    /// 
    /// The AppContext integrates with ricecoder-providers::ProviderManager for:
    /// - Provider registry and configuration
    /// - Model discovery and selection
    /// - AI chat requests via `ProviderManager::chat()`
    pub async fn get_provider(&self, provider_id: &str) -> Result<serde_json::Value, String> {
        // SDK layer is deprecated - real provider data comes from AppContext state
        Ok(serde_json::json!({
            "id": provider_id,
            "name": provider_id,
            "status": "see AppContext for real provider data"
        }))
    }
}

/// SDK provider with client and event stream
#[derive(Clone)]
pub struct SdkProvider {
    client: Arc<SdkClient>,
    event_tx: broadcast::Sender<SdkEvent>,
    url: Arc<RwLock<String>>,
}

impl SdkProvider {
    /// Create new SDK provider
    pub fn new(base_url: String) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            client: Arc::new(SdkClient::new(base_url.clone())),
            event_tx,
            url: Arc::new(RwLock::new(base_url)),
        }
    }

    /// Get SDK client
    pub fn client(&self) -> Arc<SdkClient> {
        self.client.clone()
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<SdkEvent> {
        self.event_tx.subscribe()
    }

    /// Emit event (for testing or internal use)
    pub fn emit(&self, event: SdkEvent) -> Result<(), String> {
        self.event_tx
            .send(event)
            .map(|_| ())
            .map_err(|e| format!("Failed to emit event: {}", e))
    }

    /// Get base URL
    pub async fn url(&self) -> String {
        self.url.read().await.clone()
    }

    /// Update base URL (reconnect required)
    pub async fn set_url(&self, url: String) {
        let mut write = self.url.write().await;
        *write = url;
        // TODO: Trigger reconnection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sdk_client_creation() {
        let client = SdkClient::new("http://localhost:8080".to_string());
        assert_eq!(client.base_url(), "http://localhost:8080");
    }

    #[tokio::test]
    async fn test_sdk_client_send_message() {
        let client = SdkClient::new("http://localhost:8080".to_string());
        let result = client
            .send_message("test-session", "Hello".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sdk_client_list_sessions() {
        let client = SdkClient::new("http://localhost:8080".to_string());
        let result = client.list_sessions().await;
        assert!(result.is_ok());
        // SDK is deprecated - returns empty vec, use AppContext::load_sessions instead
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_sdk_provider_creation() {
        let provider = SdkProvider::new("http://localhost:8080".to_string());
        assert_eq!(provider.url().await, "http://localhost:8080");
    }

    #[tokio::test]
    async fn test_sdk_provider_event_subscription() {
        let provider = SdkProvider::new("http://localhost:8080".to_string());
        let mut rx = provider.subscribe();

        let event = SdkEvent::SessionCreated {
            session_id: "test-123".to_string(),
        };

        provider.emit(event.clone()).unwrap();

        let received = rx.recv().await.unwrap();
        match received {
            SdkEvent::SessionCreated { session_id } => {
                assert_eq!(session_id, "test-123");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_sdk_provider_multiple_subscribers() {
        let provider = SdkProvider::new("http://localhost:8080".to_string());
        let mut rx1 = provider.subscribe();
        let mut rx2 = provider.subscribe();

        let event = SdkEvent::MessageReceived {
            session_id: "s1".to_string(),
            message_id: "m1".to_string(),
            content: "test".to_string(),
        };

        provider.emit(event).unwrap();

        let r1 = rx1.recv().await.unwrap();
        let r2 = rx2.recv().await.unwrap();

        match (r1, r2) {
            (
                SdkEvent::MessageReceived { session_id: s1, .. },
                SdkEvent::MessageReceived { session_id: s2, .. },
            ) => {
                assert_eq!(s1, "s1");
                assert_eq!(s2, "s1");
            }
            _ => panic!("Wrong event types"),
        }
    }

    #[tokio::test]
    async fn test_sdk_provider_url_update() {
        let provider = SdkProvider::new("http://localhost:8080".to_string());
        assert_eq!(provider.url().await, "http://localhost:8080");

        provider
            .set_url("http://newhost:9000".to_string())
            .await;
        assert_eq!(provider.url().await, "http://newhost:9000");
    }
}
