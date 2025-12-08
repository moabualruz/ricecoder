//! Unit tests for Webhook Handler
//!
//! These tests verify specific functionality and edge cases

use ricecoder_github::{
    EventFilter, WebhookEvent, WebhookEventType, WebhookHandler, WebhookHandlerConfig,
    WebhookOperations, WebhookRetryConfig, WorkflowTrigger,
};
use serde_json::json;

#[tokio::test]
async fn test_webhook_handler_creation() {
    let config = WebhookHandlerConfig::new();
    let handler = WebhookHandler::new(config);

    // Verify handler is created
    assert_eq!(handler.event_log_size().await, 0);
}

#[tokio::test]
async fn test_webhook_handler_processes_event() {
    let config = WebhookHandlerConfig::new();
    let handler = WebhookHandler::new(config);

    let event = WebhookEvent::new(
        WebhookEventType::Push,
        json!({
            "repository": {
                "name": "test-repo",
                "owner": {
                    "login": "test-owner"
                }
            }
        }),
    );

    let result = handler.process_event(event).await.unwrap();

    assert!(result.processed);
    assert_eq!(handler.event_log_size().await, 1);
}

#[tokio::test]
async fn test_webhook_handler_filters_events() {
    let trigger = WorkflowTrigger::new("test-workflow")
        .with_filter(EventFilter::new().with_event_type(WebhookEventType::Push));

    let config = WebhookHandlerConfig::new().add_trigger(trigger);
    let handler = WebhookHandler::new(config);

    // Push event should match
    let push_event = WebhookEvent::new(WebhookEventType::Push, json!({}));
    let result = handler.process_event(push_event).await.unwrap();
    assert!(result.matched_filter);
    assert_eq!(result.workflows_triggered.len(), 1);

    // PR event should not match
    let pr_event = WebhookEvent::new(WebhookEventType::PullRequest, json!({}));
    let result = handler.process_event(pr_event).await.unwrap();
    assert!(!result.matched_filter);
    assert_eq!(result.workflows_triggered.len(), 0);
}

#[tokio::test]
async fn test_webhook_handler_multiple_triggers() {
    let trigger1 = WorkflowTrigger::new("workflow-1")
        .with_filter(EventFilter::new().with_event_type(WebhookEventType::Push));

    let trigger2 = WorkflowTrigger::new("workflow-2")
        .with_filter(EventFilter::new().with_event_type(WebhookEventType::Push));

    let config = WebhookHandlerConfig::new()
        .add_trigger(trigger1)
        .add_trigger(trigger2);

    let handler = WebhookHandler::new(config);

    let event = WebhookEvent::new(WebhookEventType::Push, json!({}));
    let result = handler.process_event(event).await.unwrap();

    assert!(result.matched_filter);
    assert_eq!(result.workflows_triggered.len(), 2);
    assert!(result.workflows_triggered.contains(&"workflow-1".to_string()));
    assert!(result.workflows_triggered.contains(&"workflow-2".to_string()));
}

#[tokio::test]
async fn test_webhook_handler_event_log() {
    let config = WebhookHandlerConfig::new();
    let handler = WebhookHandler::new(config);

    let event1 = WebhookEvent::new(WebhookEventType::Push, json!({}));
    let event2 = WebhookEvent::new(WebhookEventType::PullRequest, json!({}));

    handler.process_event(event1).await.unwrap();
    handler.process_event(event2).await.unwrap();

    let log = handler.get_event_log().await;
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].event_type, WebhookEventType::Push);
    assert_eq!(log[1].event_type, WebhookEventType::PullRequest);
}

#[tokio::test]
async fn test_webhook_handler_clear_log() {
    let config = WebhookHandlerConfig::new();
    let handler = WebhookHandler::new(config);

    let event = WebhookEvent::new(WebhookEventType::Push, json!({}));
    handler.process_event(event).await.unwrap();

    assert_eq!(handler.event_log_size().await, 1);

    handler.clear_event_log().await;
    assert_eq!(handler.event_log_size().await, 0);
}

#[tokio::test]
async fn test_webhook_handler_update_config() {
    let config = WebhookHandlerConfig::new();
    let handler = WebhookHandler::new(config);

    let new_config = WebhookHandlerConfig::new().with_logging(false);
    handler.update_config(new_config).await;

    let config = handler.get_config().await;
    assert!(!config.log_events);
}

#[test]
fn test_event_filter_matches_event_type() {
    let filter = EventFilter::new().with_event_type(WebhookEventType::Push);
    let event = WebhookEvent::new(WebhookEventType::Push, json!({}));

    assert!(filter.matches(&event));
}

#[test]
fn test_event_filter_matches_action() {
    let filter = EventFilter::new()
        .with_event_type(WebhookEventType::PullRequest)
        .with_action("opened");

    let event = WebhookEvent::new(
        WebhookEventType::PullRequest,
        json!({"action": "opened"}),
    );

    assert!(filter.matches(&event));
}

#[test]
fn test_event_filter_matches_repository() {
    let filter = EventFilter::new().with_repository("my-repo");

    let event = WebhookEvent::new(
        WebhookEventType::Push,
        json!({"repository": {"name": "my-repo"}}),
    );

    assert!(filter.matches(&event));
}

#[test]
fn test_event_filter_no_match_different_type() {
    let filter = EventFilter::new().with_event_type(WebhookEventType::Push);
    let event = WebhookEvent::new(WebhookEventType::PullRequest, json!({}));

    assert!(!filter.matches(&event));
}

#[test]
fn test_event_filter_no_match_different_action() {
    let filter = EventFilter::new()
        .with_event_type(WebhookEventType::PullRequest)
        .with_action("opened");

    let event = WebhookEvent::new(
        WebhookEventType::PullRequest,
        json!({"action": "closed"}),
    );

    assert!(!filter.matches(&event));
}

#[test]
fn test_event_filter_no_match_different_repository() {
    let filter = EventFilter::new().with_repository("my-repo");

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

#[test]
fn test_webhook_operations_validate_payload_valid() {
    let payload = json!({"action": "opened"});
    assert!(WebhookOperations::validate_payload(&payload).is_ok());
}

#[test]
fn test_webhook_operations_validate_payload_invalid() {
    let payload = json!("not an object");
    assert!(WebhookOperations::validate_payload(&payload).is_err());
}

#[test]
fn test_webhook_operations_extract_metadata() {
    let payload = json!({
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

#[test]
fn test_webhook_retry_config_calculate_delay() {
    let config = WebhookRetryConfig::new();

    assert_eq!(config.calculate_delay(0), 100);
    assert_eq!(config.calculate_delay(1), 200);
    assert_eq!(config.calculate_delay(2), 400);
}

#[test]
fn test_webhook_retry_config_max_delay() {
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
fn test_workflow_trigger_creation() {
    let trigger = WorkflowTrigger::new("test-workflow");

    assert_eq!(trigger.workflow_id, "test-workflow");
    assert!(trigger.inputs.is_empty());
}

#[test]
fn test_workflow_trigger_with_input() {
    let trigger = WorkflowTrigger::new("test-workflow")
        .with_input("key1", "value1")
        .with_input("key2", "value2");

    assert_eq!(trigger.inputs.len(), 2);
    assert_eq!(trigger.inputs.get("key1"), Some(&"value1".to_string()));
    assert_eq!(trigger.inputs.get("key2"), Some(&"value2".to_string()));
}

#[test]
fn test_webhook_handler_config_creation() {
    let config = WebhookHandlerConfig::new();

    assert!(config.secret.is_none());
    assert!(config.triggers.is_empty());
    assert!(config.log_events);
    assert!(config.enable_filtering);
}

#[test]
fn test_webhook_handler_config_with_secret() {
    let config = WebhookHandlerConfig::new().with_secret("my-secret");

    assert_eq!(config.secret, Some("my-secret".to_string()));
}

#[test]
fn test_webhook_handler_config_with_logging() {
    let config = WebhookHandlerConfig::new().with_logging(false);

    assert!(!config.log_events);
}

#[test]
fn test_webhook_handler_config_with_filtering() {
    let config = WebhookHandlerConfig::new().with_filtering(false);

    assert!(!config.enable_filtering);
}

#[tokio::test]
async fn test_webhook_handler_no_filtering() {
    let trigger = WorkflowTrigger::new("test-workflow")
        .with_filter(EventFilter::new().with_event_type(WebhookEventType::Push));

    let config = WebhookHandlerConfig::new()
        .add_trigger(trigger)
        .with_filtering(false);

    let handler = WebhookHandler::new(config);

    // Even though filter is for Push, with filtering disabled, all events should trigger
    let event = WebhookEvent::new(WebhookEventType::PullRequest, json!({}));
    let result = handler.process_event(event).await.unwrap();

    assert!(result.matched_filter);
    assert_eq!(result.workflows_triggered.len(), 1);
}
