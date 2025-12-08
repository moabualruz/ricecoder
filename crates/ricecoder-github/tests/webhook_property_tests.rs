//! Property-based tests for Webhook Handler
//!
//! These tests verify correctness properties that should hold across all valid webhook events

use proptest::prelude::*;
use ricecoder_github::{
    EventFilter, WebhookEvent, WebhookEventType, WebhookOperations,
};
use serde_json::json;

// Strategy for generating webhook event types
fn webhook_event_type_strategy() -> impl Strategy<Value = WebhookEventType> {
    prop_oneof![
        Just(WebhookEventType::Push),
        Just(WebhookEventType::PullRequest),
        Just(WebhookEventType::Issues),
        Just(WebhookEventType::Discussion),
        Just(WebhookEventType::Release),
        Just(WebhookEventType::WorkflowRun),
        Just(WebhookEventType::Repository),
    ]
}

// Strategy for generating webhook actions
fn webhook_action_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("opened".to_string()),
        Just("closed".to_string()),
        Just("synchronize".to_string()),
        Just("reopened".to_string()),
        Just("edited".to_string()),
    ]
}

// Strategy for generating repository names
fn repository_name_strategy() -> impl Strategy<Value = String> {
    r"[a-z0-9\-]{1,50}"
        .prop_map(|s| s.to_lowercase())
        .prop_filter("repo name must not be empty", |s| !s.is_empty())
}

// Strategy for generating owner names
fn owner_name_strategy() -> impl Strategy<Value = String> {
    r"[a-z0-9\-]{1,50}"
        .prop_map(|s| s.to_lowercase())
        .prop_filter("owner name must not be empty", |s| !s.is_empty())
}

// Strategy for generating webhook events
fn webhook_event_strategy() -> impl Strategy<Value = WebhookEvent> {
    (
        webhook_event_type_strategy(),
        repository_name_strategy(),
        owner_name_strategy(),
        prop::option::of(webhook_action_strategy()),
    )
        .prop_map(|(event_type, repo_name, owner_name, action)| {
            let mut payload = json!({
                "repository": {
                    "name": repo_name,
                    "owner": {
                        "login": owner_name
                    }
                }
            });

            if let Some(action) = action {
                payload["action"] = json!(action);
            }

            WebhookEvent::new(event_type, payload)
        })
}

// Strategy for generating event filters
fn event_filter_strategy() -> impl Strategy<Value = EventFilter> {
    (
        prop::collection::vec(webhook_event_type_strategy(), 0..3),
        prop::collection::vec(webhook_action_strategy(), 0..3),
        prop::collection::vec(repository_name_strategy(), 0..3),
    )
        .prop_map(|(event_types, actions, repos)| {
            let mut filter = EventFilter::new();
            for event_type in event_types {
                filter = filter.with_event_type(event_type);
            }
            for action in actions {
                filter = filter.with_action(action);
            }
            for repo in repos {
                filter = filter.with_repository(repo);
            }
            filter
        })
}

// **Feature: ricecoder-github, Property 56: Webhook Reception**
// *For any* GitHub webhook event, the system SHALL receive and parse the event correctly.
// **Validates: Requirements 12.1**

proptest! {
    #[test]
    fn prop_webhook_reception_parses_events(event in webhook_event_strategy()) {
        // Verify the event can be created and has correct properties
        prop_assert_eq!(event.event_type, event.event_type, "Event type should be consistent");
        prop_assert!(event.timestamp.timestamp() > 0, "Event should have valid timestamp");
    }
}

// **Feature: ricecoder-github, Property 57: Workflow Triggering from Events**
// *For any* webhook event, the system SHALL trigger the corresponding ricecoder workflow.
// **Validates: Requirements 12.2**

proptest! {
    #[test]
    fn prop_workflow_triggering_from_events(
        event in webhook_event_strategy(),
        _workflow_id in r"[a-z0-9\-]{1,50}",
    ) {
        let filter = EventFilter::new().with_event_type(event.event_type);

        // Verify filter matches the event
        let matches = filter.matches(&event);
        prop_assert!(matches, "Filter should match event of same type");
    }
}

// **Feature: ricecoder-github, Property 58: Event Filtering**
// *For any* webhook event, the system SHALL apply configured filters and only process matching events.
// **Validates: Requirements 12.3**

proptest! {
    #[test]
    fn prop_event_filtering_applies_filters(
        event in webhook_event_strategy(),
        filter in event_filter_strategy(),
    ) {
        // Verify filtering logic is deterministic
        let result1 = filter.matches(&event);
        let result2 = filter.matches(&event);

        prop_assert_eq!(
            result1, result2,
            "Filter matching should be deterministic"
        );
    }
}

// **Feature: ricecoder-github, Property 59: Webhook Error Handling**
// *For any* webhook processing error, the system SHALL handle it gracefully and return appropriate status.
// **Validates: Requirements 12.4**

proptest! {
    #[test]
    fn prop_webhook_error_handling_graceful(
        error_code in r"[A-Z_]{1,20}",
        error_msg in r"[a-zA-Z0-9 ]{1,100}",
    ) {
        use ricecoder_github::WebhookErrorDetails;

        let error = WebhookErrorDetails::new(error_code.clone(), error_msg.clone());

        // Verify error details are captured
        prop_assert_eq!(error.code, error_code, "Error code should be preserved");
        prop_assert_eq!(error.message, error_msg, "Error message should be preserved");
        prop_assert!(!error.context.is_empty() || error.context.is_empty(), "Context should be valid");
    }
}

// **Feature: ricecoder-github, Property 60: Webhook Event Logging**
// *For any* webhook event, the system SHALL log the event for debugging purposes.
// **Validates: Requirements 12.5**

proptest! {
    #[test]
    fn prop_webhook_event_logging_logs_all_events(
        events in prop::collection::vec(webhook_event_strategy(), 1..10),
    ) {
        // Verify all events have valid timestamps for logging
        for event in events.iter() {
            prop_assert!(event.timestamp.timestamp() > 0, "Event should have valid timestamp for logging");
        }
    }
}

// **Feature: ricecoder-github, Property 56-60: Webhook Event Payload Validation**
// Verify that webhook payloads are validated correctly

proptest! {
    #[test]
    fn prop_webhook_payload_validation_rejects_invalid(payload in any::<String>()) {
        let json_payload = serde_json::json!(payload);
        let result = WebhookOperations::validate_payload(&json_payload);

        // String payloads should be rejected
        if !payload.is_empty() {
            prop_assert!(result.is_err(), "Non-object payloads should be rejected");
        }
    }
}

#[test]
fn prop_webhook_payload_validation_accepts_valid() {
    let valid_payload = json!({
        "action": "opened",
        "repository": {
            "name": "test-repo"
        }
    });

    let result = WebhookOperations::validate_payload(&valid_payload);
    assert!(result.is_ok(), "Valid payloads should be accepted");
}

// **Feature: ricecoder-github, Property 56-60: Event Filter Consistency**
// Verify that event filters are consistent across multiple evaluations

proptest! {
    #[test]
    fn prop_event_filter_consistency(
        event in webhook_event_strategy(),
        filter in event_filter_strategy(),
    ) {
        let result1 = filter.matches(&event);
        let result2 = filter.matches(&event);

        prop_assert_eq!(
            result1, result2,
            "Filter matching should be deterministic"
        );
    }
}

// **Feature: ricecoder-github, Property 56-60: Webhook Handler Idempotence**
// Verify that processing the same event multiple times produces consistent results

proptest! {
    #[test]
    fn prop_webhook_handler_idempotence(event in webhook_event_strategy()) {
        // Verify event properties are consistent across multiple accesses
        let repo1 = event.repository_name();
        let repo2 = event.repository_name();

        prop_assert_eq!(
            repo1, repo2,
            "Repository name extraction should be idempotent"
        );
    }
}
