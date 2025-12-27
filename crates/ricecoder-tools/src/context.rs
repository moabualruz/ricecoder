//! Tool execution context (OpenCode-compatible)
//!
//! Provides complete execution context matching OpenCode's Tool.Context interface.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use futures::FutureExt;

/// Tool execution context (OpenCode-compatible)
///
/// Matches OpenCode `Tool.Context<M>` interface with all fields:
/// - sessionID, messageID, agent, abort (required)
/// - callID, extra (optional)
/// - metadata() callback for side-channel reporting
#[derive(Clone)]
pub struct ToolContext {
    /// Session ID for tracking
    pub session_id: String,
    
    /// Message ID within the session
    pub message_id: String,
    
    /// Agent name executing the tool
    pub agent: String,
    
    /// Abort signal for cancellation
    pub abort: Arc<tokio::sync::Notify>,
    
    /// Abort state flag (for non-blocking checks)
    aborted: Arc<AtomicBool>,
    
    /// Optional call ID for specific tool invocation
    pub call_id: Option<String>,
    
    /// Optional extra metadata
    pub extra: Option<HashMap<String, serde_json::Value>>,
    
    /// Metadata callback for incremental reporting
    metadata_callback: Arc<RwLock<Option<MetadataCallback>>>,
}

/// Metadata update for incremental reporting (OpenCode-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataUpdate {
    /// Optional updated title
    pub title: Option<String>,
    
    /// Optional updated metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Callback type for metadata updates
pub type MetadataCallback = Arc<dyn Fn(MetadataUpdate) + Send + Sync>;

impl ToolContext {
    /// Create a new tool execution context
    pub fn new(session_id: String, message_id: String, agent: String) -> Self {
        Self {
            session_id,
            message_id,
            agent,
            abort: Arc::new(tokio::sync::Notify::new()),
            aborted: Arc::new(AtomicBool::new(false)),
            call_id: None,
            extra: None,
            metadata_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the call ID
    pub fn with_call_id(mut self, call_id: impl Into<String>) -> Self {
        self.call_id = Some(call_id.into());
        self
    }

    /// Set extra metadata
    pub fn with_extra(mut self, extra: HashMap<String, serde_json::Value>) -> Self {
        self.extra = Some(extra);
        self
    }

    /// Set a metadata callback for incremental reporting (OpenCode-compatible)
    ///
    /// Matches OpenCode `metadata({ title?, metadata? })` signature
    pub async fn set_metadata_callback<F>(&self, callback: F)
    where
        F: Fn(MetadataUpdate) + Send + Sync + 'static,
    {
        *self.metadata_callback.write().await = Some(Arc::new(callback));
    }

    /// Report metadata update (OpenCode-compatible)
    ///
    /// Matches OpenCode `ctx.metadata({ title, metadata })` call
    pub async fn report_metadata(&self, update: MetadataUpdate) {
        if let Some(callback) = self.metadata_callback.read().await.as_ref() {
            callback(update);
        }
    }

    /// Report metadata with title only (convenience method)
    pub async fn report_title(&self, title: impl Into<String>) {
        self.report_metadata(MetadataUpdate {
            title: Some(title.into()),
            metadata: None,
        })
        .await;
    }

    /// Report metadata with custom fields (convenience method)
    pub async fn report_custom_metadata(&self, metadata: HashMap<String, serde_json::Value>) {
        self.report_metadata(MetadataUpdate {
            title: None,
            metadata: Some(metadata),
        })
        .await;
    }

    /// Check if abort has been signaled
    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::SeqCst)
    }

    /// Wait for abort signal
    pub async fn wait_for_abort(&self) {
        self.abort.notified().await;
    }

    /// Signal abort
    pub fn signal_abort(&self) {
        self.aborted.store(true, Ordering::SeqCst);
        self.abort.notify_waiters();
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(),
            "default".to_string(),
        )
    }
}

impl fmt::Debug for ToolContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolContext")
            .field("session_id", &self.session_id)
            .field("message_id", &self.message_id)
            .field("agent", &self.agent)
            .field("call_id", &self.call_id)
            .field("extra", &self.extra)
            .field("has_metadata_callback", &self.metadata_callback.try_read().map(|cb| cb.is_some()).unwrap_or(false))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_context() {
        let ctx = ToolContext::new(
            "session-123".to_string(),
            "message-456".to_string(),
            "build".to_string(),
        );

        assert_eq!(ctx.session_id, "session-123");
        assert_eq!(ctx.message_id, "message-456");
        assert_eq!(ctx.agent, "build");
        assert!(ctx.call_id.is_none());
        assert!(ctx.extra.is_none());
    }

    #[test]
    fn test_with_call_id() {
        let ctx = ToolContext::new(
            "session-123".to_string(),
            "message-456".to_string(),
            "build".to_string(),
        )
        .with_call_id("call-789");

        assert_eq!(ctx.call_id, Some("call-789".to_string()));
    }

    #[test]
    fn test_with_extra() {
        let mut extra = HashMap::new();
        extra.insert("key".to_string(), serde_json::json!("value"));

        let ctx = ToolContext::new(
            "session-123".to_string(),
            "message-456".to_string(),
            "build".to_string(),
        )
        .with_extra(extra.clone());

        assert_eq!(ctx.extra, Some(extra));
    }

    #[tokio::test]
    async fn test_metadata_callback() {
        let ctx = ToolContext::new(
            "session-123".to_string(),
            "message-456".to_string(),
            "build".to_string(),
        );

        let reported = Arc::new(RwLock::new(None));
        let reported_clone = reported.clone();

        ctx.set_metadata_callback(move |update| {
            let reported = reported_clone.clone();
            let _ = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    *reported.write().await = Some(update);
                });
            });
        })
        .await;

        ctx.report_title("Test Title").await;

        // Give callback time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let update = reported.read().await.clone();
        assert!(update.is_some());
    }

    #[test]
    fn test_abort_signal() {
        let ctx = ToolContext::new(
            "session-123".to_string(),
            "message-456".to_string(),
            "build".to_string(),
        );

        assert!(!ctx.is_aborted());
        ctx.signal_abort();
        // After signaling, notified() should complete immediately
        assert!(ctx.is_aborted());
    }
}
