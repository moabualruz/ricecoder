//! Webhook Handler - Processes GitHub webhooks and triggers workflows

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::errors::{GitHubError, Result};

/// Webhook event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// Push event
    Push,
    /// Pull request event
    PullRequest,
    /// Issues event
    Issues,
    /// Discussion event
    Discussion,
    /// Release event
    Release,
    /// Workflow run event
    WorkflowRun,
    /// Repository event
    Repository,
    /// Unknown event
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for WebhookEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookEventType::Push => write!(f, "push"),
            WebhookEventType::PullRequest => write!(f, "pull_request"),
            WebhookEventType::Issues => write!(f, "issues"),
            WebhookEventType::Discussion => write!(f, "discussion"),
            WebhookEventType::Release => write!(f, "release"),
            WebhookEventType::WorkflowRun => write!(f, "workflow_run"),
            WebhookEventType::Repository => write!(f, "repository"),
            WebhookEventType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Webhook action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAction {
    /// Action type (e.g., "opened", "closed", "synchronize")
    pub action: String,
}

/// Webhook event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Event type
    pub event_type: WebhookEventType,
    /// Event action
    pub action: Option<String>,
    /// Event payload
    pub payload: Value,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WebhookEvent {
    /// Create a new webhook event
    pub fn new(event_type: WebhookEventType, payload: Value) -> Self {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            event_type,
            action,
            payload,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get the repository name from the payload
    pub fn repository_name(&self) -> Option<String> {
        self.payload
            .get("repository")
            .and_then(|r| r.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    }

    /// Get the repository owner from the payload
    pub fn repository_owner(&self) -> Option<String> {
        self.payload
            .get("repository")
            .and_then(|r| r.get("owner"))
            .and_then(|o| o.get("login"))
            .and_then(|l| l.as_str())
            .map(|s| s.to_string())
    }
}

/// Event filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Event types to process
    pub event_types: Vec<WebhookEventType>,
    /// Actions to process (if empty, all actions are processed)
    pub actions: Vec<String>,
    /// Repository filter (if empty, all repositories are processed)
    pub repositories: Vec<String>,
}

impl EventFilter {
    /// Create a new event filter
    pub fn new() -> Self {
        Self {
            event_types: vec![],
            actions: vec![],
            repositories: vec![],
        }
    }

    /// Add an event type to the filter
    pub fn with_event_type(mut self, event_type: WebhookEventType) -> Self {
        self.event_types.push(event_type);
        self
    }

    /// Add an action to the filter
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Add a repository to the filter
    pub fn with_repository(mut self, repo: impl Into<String>) -> Self {
        self.repositories.push(repo.into());
        self
    }

    /// Check if an event matches the filter
    pub fn matches(&self, event: &WebhookEvent) -> bool {
        // Check event type
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type) {
            return false;
        }

        // Check action
        if !self.actions.is_empty() {
            if let Some(action) = &event.action {
                if !self.actions.contains(action) {
                    return false;
                }
            } else if !self.actions.is_empty() {
                return false;
            }
        }

        // Check repository
        if !self.repositories.is_empty() {
            if let Some(repo) = event.repository_name() {
                if !self.repositories.contains(&repo) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTrigger {
    /// Workflow name or ID
    pub workflow_id: String,
    /// Event filter
    pub filter: EventFilter,
    /// Workflow input parameters
    pub inputs: HashMap<String, String>,
}

impl WorkflowTrigger {
    /// Create a new workflow trigger
    pub fn new(workflow_id: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            filter: EventFilter::new(),
            inputs: HashMap::new(),
        }
    }

    /// Set the filter
    pub fn with_filter(mut self, filter: EventFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Add an input parameter
    pub fn with_input(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.inputs.insert(key.into(), value.into());
        self
    }
}

/// Webhook processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookProcessingResult {
    /// Whether the event was processed
    pub processed: bool,
    /// Whether the event matched any filters
    pub matched_filter: bool,
    /// Workflows triggered
    pub workflows_triggered: Vec<String>,
    /// Error message if any
    pub error: Option<String>,
}

impl WebhookProcessingResult {
    /// Create a new processing result
    pub fn new() -> Self {
        Self {
            processed: false,
            matched_filter: false,
            workflows_triggered: vec![],
            error: None,
        }
    }

    /// Mark as processed
    pub fn with_processed(mut self, processed: bool) -> Self {
        self.processed = processed;
        self
    }

    /// Mark as matched
    pub fn with_matched(mut self, matched: bool) -> Self {
        self.matched_filter = matched;
        self
    }

    /// Add a triggered workflow
    pub fn add_workflow(mut self, workflow: impl Into<String>) -> Self {
        self.workflows_triggered.push(workflow.into());
        self
    }

    /// Set error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
}

impl Default for WebhookProcessingResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Webhook handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookHandlerConfig {
    /// Webhook secret for signature verification
    pub secret: Option<String>,
    /// Workflow triggers
    pub triggers: Vec<WorkflowTrigger>,
    /// Enable event logging
    pub log_events: bool,
    /// Enable event filtering
    pub enable_filtering: bool,
}

impl WebhookHandlerConfig {
    /// Create a new webhook handler configuration
    pub fn new() -> Self {
        Self {
            secret: None,
            triggers: vec![],
            log_events: true,
            enable_filtering: true,
        }
    }

    /// Set the webhook secret
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }

    /// Add a workflow trigger
    pub fn add_trigger(mut self, trigger: WorkflowTrigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// Enable or disable event logging
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_events = enabled;
        self
    }

    /// Enable or disable event filtering
    pub fn with_filtering(mut self, enabled: bool) -> Self {
        self.enable_filtering = enabled;
        self
    }
}

impl Default for WebhookHandlerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Webhook handler
pub struct WebhookHandler {
    config: Arc<RwLock<WebhookHandlerConfig>>,
    event_log: Arc<RwLock<Vec<WebhookEvent>>>,
}

impl WebhookHandler {
    /// Create a new webhook handler
    pub fn new(config: WebhookHandlerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            event_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Process a webhook event
    pub async fn process_event(&self, event: WebhookEvent) -> Result<WebhookProcessingResult> {
        let config = self.config.read().await;

        // Log the event if enabled
        if config.log_events {
            info!(
                event_type = %event.event_type,
                action = ?event.action,
                timestamp = %event.timestamp,
                "Webhook event received"
            );
        }

        let mut result = WebhookProcessingResult::new();

        // Check if filtering is enabled
        if config.enable_filtering {
            // Find matching triggers
            let mut matched = false;
            for trigger in &config.triggers {
                if trigger.filter.matches(&event) {
                    matched = true;
                    result = result.add_workflow(trigger.workflow_id.clone());
                    debug!(
                        workflow_id = %trigger.workflow_id,
                        "Workflow trigger matched"
                    );
                }
            }
            result = result.with_matched(matched);
        } else {
            // If filtering is disabled, trigger all workflows
            for trigger in &config.triggers {
                result = result.add_workflow(trigger.workflow_id.clone());
            }
            result = result.with_matched(true);
        }

        result = result.with_processed(true);

        // Store event in log
        let mut log = self.event_log.write().await;
        log.push(event);

        Ok(result)
    }

    /// Get the event log
    pub async fn get_event_log(&self) -> Vec<WebhookEvent> {
        self.event_log.read().await.clone()
    }

    /// Clear the event log
    pub async fn clear_event_log(&self) {
        self.event_log.write().await.clear();
    }

    /// Get the number of events in the log
    pub async fn event_log_size(&self) -> usize {
        self.event_log.read().await.len()
    }

    /// Update the configuration
    pub async fn update_config(&self, config: WebhookHandlerConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> WebhookHandlerConfig {
        self.config.read().await.clone()
    }

    /// Verify webhook signature
    pub fn verify_signature(&self, payload: &[u8], signature: &str, secret: &str) -> Result<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| GitHubError::invalid_input("Invalid secret"))?;
        mac.update(payload);

        let computed = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        Ok(computed == signature)
    }
}

impl Clone for WebhookHandler {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            event_log: Arc::clone(&self.event_log),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_event_filter_matches_event_type() {
        let filter = EventFilter::new().with_event_type(WebhookEventType::Push);
        let event = WebhookEvent::new(WebhookEventType::Push, json!({}));
        assert!(filter.matches(&event));

        let event = WebhookEvent::new(WebhookEventType::PullRequest, json!({}));
        assert!(!filter.matches(&event));
    }

    #[test]
    fn test_event_filter_matches_action() {
        let filter = EventFilter::new()
            .with_event_type(WebhookEventType::PullRequest)
            .with_action("opened");

        let event = WebhookEvent::new(WebhookEventType::PullRequest, json!({"action": "opened"}));
        assert!(filter.matches(&event));

        let event = WebhookEvent::new(WebhookEventType::PullRequest, json!({"action": "closed"}));
        assert!(!filter.matches(&event));
    }

    #[test]
    fn test_event_filter_matches_repository() {
        let filter = EventFilter::new().with_repository("my-repo");

        let event = WebhookEvent::new(
            WebhookEventType::Push,
            json!({"repository": {"name": "my-repo"}}),
        );
        assert!(filter.matches(&event));

        let event = WebhookEvent::new(
            WebhookEventType::Push,
            json!({"repository": {"name": "other-repo"}}),
        );
        assert!(!filter.matches(&event));
    }

    #[test]
    fn test_webhook_event_extraction() {
        let payload = json!({
            "action": "opened",
            "repository": {
                "name": "test-repo",
                "owner": {
                    "login": "test-owner"
                }
            }
        });

        let event = WebhookEvent::new(WebhookEventType::PullRequest, payload);
        assert_eq!(event.action, Some("opened".to_string()));
        assert_eq!(event.repository_name(), Some("test-repo".to_string()));
        assert_eq!(event.repository_owner(), Some("test-owner".to_string()));
    }

    #[tokio::test]
    async fn test_webhook_handler_processes_event() {
        let config = WebhookHandlerConfig::new();
        let handler = WebhookHandler::new(config);

        let event = WebhookEvent::new(WebhookEventType::Push, json!({}));
        let result = handler.process_event(event.clone()).await.unwrap();

        assert!(result.processed);
        assert_eq!(handler.event_log_size().await, 1);
    }

    #[tokio::test]
    async fn test_webhook_handler_filters_events() {
        let trigger = WorkflowTrigger::new("test-workflow")
            .with_filter(EventFilter::new().with_event_type(WebhookEventType::Push));

        let config = WebhookHandlerConfig::new().add_trigger(trigger);
        let handler = WebhookHandler::new(config);

        let event = WebhookEvent::new(WebhookEventType::Push, json!({}));
        let result = handler.process_event(event).await.unwrap();

        assert!(result.matched_filter);
        assert_eq!(result.workflows_triggered.len(), 1);
    }

    #[tokio::test]
    async fn test_webhook_handler_no_match() {
        let trigger = WorkflowTrigger::new("test-workflow")
            .with_filter(EventFilter::new().with_event_type(WebhookEventType::Push));

        let config = WebhookHandlerConfig::new().add_trigger(trigger);
        let handler = WebhookHandler::new(config);

        let event = WebhookEvent::new(WebhookEventType::PullRequest, json!({}));
        let result = handler.process_event(event).await.unwrap();

        assert!(!result.matched_filter);
        assert_eq!(result.workflows_triggered.len(), 0);
    }
}
