//! Webhook Operations - Advanced webhook operations and utilities

use crate::errors::{GitHubError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Webhook retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl WebhookRetryConfig {
    /// Create a new retry configuration
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }

    /// Calculate the delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let delay =
            (self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32)) as u64;
        delay.min(self.max_delay_ms)
    }
}

impl Default for WebhookRetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Webhook error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookErrorDetails {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error context
    pub context: HashMap<String, String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WebhookErrorDetails {
    /// Create a new error details
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add context information
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Webhook error handling result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookErrorHandlingResult {
    /// Whether the error was handled
    pub handled: bool,
    /// Whether to retry
    pub should_retry: bool,
    /// Error details
    pub error: WebhookErrorDetails,
    /// Retry attempt number
    pub retry_attempt: u32,
}

impl WebhookErrorHandlingResult {
    /// Create a new error handling result
    pub fn new(error: WebhookErrorDetails) -> Self {
        Self {
            handled: false,
            should_retry: false,
            error,
            retry_attempt: 0,
        }
    }

    /// Mark as handled
    pub fn with_handled(mut self, handled: bool) -> Self {
        self.handled = handled;
        self
    }

    /// Mark for retry
    pub fn with_retry(mut self, should_retry: bool) -> Self {
        self.should_retry = should_retry;
        self
    }

    /// Set retry attempt
    pub fn with_attempt(mut self, attempt: u32) -> Self {
        self.retry_attempt = attempt;
        self
    }
}

/// Webhook event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventLogEntry {
    /// Event ID
    pub event_id: String,
    /// Event type
    pub event_type: String,
    /// Event action
    pub action: Option<String>,
    /// Processing status
    pub status: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Error if any
    pub error: Option<String>,
    /// Processing duration in milliseconds
    pub duration_ms: u64,
}

impl WebhookEventLogEntry {
    /// Create a new log entry
    pub fn new(event_id: impl Into<String>, event_type: impl Into<String>) -> Self {
        Self {
            event_id: event_id.into(),
            event_type: event_type.into(),
            action: None,
            status: "pending".to_string(),
            timestamp: chrono::Utc::now(),
            error: None,
            duration_ms: 0,
        }
    }

    /// Set the action
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Mark as processed
    pub fn with_processed(mut self, duration_ms: u64) -> Self {
        self.status = "processed".to_string();
        self.duration_ms = duration_ms;
        self
    }

    /// Mark as failed
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.status = "failed".to_string();
        self.error = Some(error.into());
        self
    }

    /// Mark as filtered
    pub fn with_filtered(mut self) -> Self {
        self.status = "filtered".to_string();
        self
    }
}

/// Webhook event logger
pub struct WebhookEventLogger {
    entries: Vec<WebhookEventLogEntry>,
    max_entries: usize,
}

impl WebhookEventLogger {
    /// Create a new event logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Log an event
    pub fn log(&mut self, entry: WebhookEventLogEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Get all entries
    pub fn entries(&self) -> &[WebhookEventLogEntry] {
        &self.entries
    }

    /// Get entries by status
    pub fn entries_by_status(&self, status: &str) -> Vec<&WebhookEventLogEntry> {
        self.entries.iter().filter(|e| e.status == status).collect()
    }

    /// Get entries by event type
    pub fn entries_by_type(&self, event_type: &str) -> Vec<&WebhookEventLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get statistics
    pub fn statistics(&self) -> WebhookEventStatistics {
        let total = self.entries.len();
        let processed = self.entries_by_status("processed").len();
        let failed = self.entries_by_status("failed").len();
        let filtered = self.entries_by_status("filtered").len();

        WebhookEventStatistics {
            total_events: total,
            processed_events: processed,
            failed_events: failed,
            filtered_events: filtered,
        }
    }
}

impl Clone for WebhookEventLogger {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            max_entries: self.max_entries,
        }
    }
}

/// Webhook event statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventStatistics {
    /// Total events
    pub total_events: usize,
    /// Processed events
    pub processed_events: usize,
    /// Failed events
    pub failed_events: usize,
    /// Filtered events
    pub filtered_events: usize,
}

impl WebhookEventStatistics {
    /// Get the success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_events == 0 {
            return 0.0;
        }
        (self.processed_events as f64) / (self.total_events as f64)
    }

    /// Get the failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_events == 0 {
            return 0.0;
        }
        (self.failed_events as f64) / (self.total_events as f64)
    }
}

/// Webhook operations
pub struct WebhookOperations;

impl WebhookOperations {
    /// Handle a webhook error
    pub fn handle_error(
        error: &GitHubError,
        retry_config: &WebhookRetryConfig,
        attempt: u32,
    ) -> WebhookErrorHandlingResult {
        let error_details = match error {
            GitHubError::RateLimitExceeded => {
                WebhookErrorDetails::new("RATE_LIMIT", "GitHub API rate limit exceeded")
            }
            GitHubError::AuthError(msg) => {
                WebhookErrorDetails::new("AUTH_ERROR", format!("Authentication failed: {}", msg))
            }
            GitHubError::NetworkError(msg) => {
                WebhookErrorDetails::new("NETWORK_ERROR", format!("Network error: {}", msg))
            }
            GitHubError::Timeout => WebhookErrorDetails::new("TIMEOUT", "Operation timed out"),
            _ => WebhookErrorDetails::new("UNKNOWN_ERROR", error.to_string()),
        };

        let should_retry = attempt < retry_config.max_retries
            && matches!(
                error,
                GitHubError::NetworkError(_)
                    | GitHubError::Timeout
                    | GitHubError::RateLimitExceeded
            );

        WebhookErrorHandlingResult::new(error_details)
            .with_handled(true)
            .with_retry(should_retry)
            .with_attempt(attempt)
    }

    /// Validate webhook payload
    pub fn validate_payload(payload: &Value) -> Result<()> {
        if !payload.is_object() {
            return Err(GitHubError::invalid_input(
                "Webhook payload must be a JSON object",
            ));
        }

        // Check for required fields
        if payload.get("action").is_none() && payload.get("repository").is_none() {
            return Err(GitHubError::invalid_input(
                "Webhook payload must contain 'action' or 'repository' field",
            ));
        }

        Ok(())
    }

    /// Extract event metadata
    pub fn extract_metadata(payload: &Value) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        if let Some(repo) = payload.get("repository") {
            if let Some(name) = repo.get("name").and_then(|n| n.as_str()) {
                metadata.insert("repository".to_string(), name.to_string());
            }
            if let Some(owner) = repo
                .get("owner")
                .and_then(|o| o.get("login"))
                .and_then(|l| l.as_str())
            {
                metadata.insert("owner".to_string(), owner.to_string());
            }
        }

        if let Some(action) = payload.get("action").and_then(|a| a.as_str()) {
            metadata.insert("action".to_string(), action.to_string());
        }

        if let Some(sender) = payload
            .get("sender")
            .and_then(|s| s.get("login"))
            .and_then(|l| l.as_str())
        {
            metadata.insert("sender".to_string(), sender.to_string());
        }

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_calculate_delay() {
        let config = WebhookRetryConfig::new();
        assert_eq!(config.calculate_delay(0), 100);
        assert_eq!(config.calculate_delay(1), 200);
        assert_eq!(config.calculate_delay(2), 400);
    }

    #[test]
    fn test_retry_config_max_delay() {
        let config = WebhookRetryConfig {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 500,
            backoff_multiplier: 2.0,
        };
        assert_eq!(config.calculate_delay(0), 100);
        assert_eq!(config.calculate_delay(1), 200);
        assert_eq!(config.calculate_delay(2), 400);
        assert_eq!(config.calculate_delay(3), 500); // Capped at max_delay_ms
    }

    #[test]
    fn test_event_logger_log_entry() {
        let mut logger = WebhookEventLogger::new(10);
        let entry = WebhookEventLogEntry::new("event-1", "push");
        logger.log(entry);

        assert_eq!(logger.entries().len(), 1);
    }

    #[test]
    fn test_event_logger_max_entries() {
        let mut logger = WebhookEventLogger::new(2);
        logger.log(WebhookEventLogEntry::new("event-1", "push"));
        logger.log(WebhookEventLogEntry::new("event-2", "push"));
        logger.log(WebhookEventLogEntry::new("event-3", "push"));

        assert_eq!(logger.entries().len(), 2);
        assert_eq!(logger.entries()[0].event_id, "event-2");
        assert_eq!(logger.entries()[1].event_id, "event-3");
    }

    #[test]
    fn test_event_logger_statistics() {
        let mut logger = WebhookEventLogger::new(10);
        let mut entry1 = WebhookEventLogEntry::new("event-1", "push");
        entry1.status = "processed".to_string();
        logger.log(entry1);

        let mut entry2 = WebhookEventLogEntry::new("event-2", "push");
        entry2.status = "failed".to_string();
        logger.log(entry2);

        let stats = logger.statistics();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.processed_events, 1);
        assert_eq!(stats.failed_events, 1);
    }

    #[test]
    fn test_event_statistics_success_rate() {
        let stats = WebhookEventStatistics {
            total_events: 10,
            processed_events: 8,
            failed_events: 2,
            filtered_events: 0,
        };
        assert_eq!(stats.success_rate(), 0.8);
        assert_eq!(stats.failure_rate(), 0.2);
    }

    #[test]
    fn test_webhook_operations_validate_payload() {
        let valid_payload = serde_json::json!({"action": "opened"});
        assert!(WebhookOperations::validate_payload(&valid_payload).is_ok());

        let invalid_payload = serde_json::json!("not an object");
        assert!(WebhookOperations::validate_payload(&invalid_payload).is_err());
    }

    #[test]
    fn test_webhook_operations_extract_metadata() {
        let payload = serde_json::json!({
            "action": "opened",
            "repository": {
                "name": "test-repo",
                "owner": {
                    "login": "test-owner"
                }
            },
            "sender": {
                "login": "test-user"
            }
        });

        let metadata = WebhookOperations::extract_metadata(&payload);
        assert_eq!(metadata.get("action"), Some(&"opened".to_string()));
        assert_eq!(metadata.get("repository"), Some(&"test-repo".to_string()));
        assert_eq!(metadata.get("owner"), Some(&"test-owner".to_string()));
        assert_eq!(metadata.get("sender"), Some(&"test-user".to_string()));
    }
}
