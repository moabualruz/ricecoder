//! Unit tests for Discussion Manager
//!
//! These tests verify specific functionality and edge cases

use ricecoder_github::{DiscussionManager, DiscussionOperations};

#[tokio::test]
async fn test_create_discussion_success() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager
        .create_discussion("Test Discussion", "This is a test", "general")
        .await;

    assert!(result.is_ok());
    let creation = result.unwrap();
    assert!(creation.discussion_id > 0);
    assert!(creation.number > 0);
    assert!(creation.url.contains("testowner"));
    assert!(creation.url.contains("testrepo"));
    assert_eq!(creation.category, "general");
}

#[tokio::test]
async fn test_create_discussion_empty_title() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager
        .create_discussion("", "This is a test", "general")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_discussion_empty_body() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager
        .create_discussion("Test Discussion", "", "general")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_discussion_empty_category() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager
        .create_discussion("Test Discussion", "This is a test", "")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_post_response_success() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.post_response(1, "This is a response").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.discussion_number, 1);
    assert_eq!(response.content, "This is a response");
    assert!(!response.author.is_empty());
}

#[tokio::test]
async fn test_post_response_empty_content() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.post_response(1, "").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_post_response_invalid_discussion_number() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.post_response(0, "This is a response").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_extract_insights_success() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.extract_insights(1).await;

    assert!(result.is_ok());
    let insights = result.unwrap();
    assert!(!insights.is_empty());

    for insight in insights {
        assert!(!insight.insight_type.is_empty());
        assert!(!insight.content.is_empty());
        assert!(insight.confidence >= 0.0 && insight.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_extract_insights_invalid_discussion_number() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.extract_insights(0).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_summary_success() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager
        .generate_summary(1, "Test Discussion")
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.discussion_number, 1);
    assert_eq!(summary.title, "Test Discussion");
    assert!(!summary.content.is_empty());
    assert!(!summary.insights.is_empty());
    assert!(!summary.participants.is_empty());
}

#[tokio::test]
async fn test_generate_summary_invalid_discussion_number() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.generate_summary(0, "Test Discussion").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_summary_empty_title() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.generate_summary(1, "").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_monitor_status_success() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.monitor_status(1).await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.discussion_number, 1);
    assert!(!status.status.is_empty());
    // comment_count is u32, so it's always >= 0
}

#[tokio::test]
async fn test_monitor_status_invalid_discussion_number() {
    let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

    let result = manager.monitor_status(0).await;

    assert!(result.is_err());
}

// DiscussionOperations tests

#[tokio::test]
async fn test_categorize_discussion_bug() {
    let result = DiscussionOperations::categorize_discussion(
        1,
        "Bug in feature X",
        "There is an error when using feature X",
    )
    .await;

    assert!(result.is_ok());
    let categorization = result.unwrap();
    assert_eq!(categorization.category, "bug-report");
    assert!(categorization.confidence > 0.0);
}

#[tokio::test]
async fn test_categorize_discussion_feature() {
    let result = DiscussionOperations::categorize_discussion(
        1,
        "Feature request: Add X",
        "I would like to request feature X",
    )
    .await;

    assert!(result.is_ok());
    let categorization = result.unwrap();
    assert_eq!(categorization.category, "feature-request");
}

#[tokio::test]
async fn test_categorize_discussion_help() {
    let result = DiscussionOperations::categorize_discussion(
        1,
        "Help with feature X",
        "I need help using feature X",
    )
    .await;

    assert!(result.is_ok());
    let categorization = result.unwrap();
    assert_eq!(categorization.category, "help");
}

#[tokio::test]
async fn test_categorize_discussion_general() {
    let result = DiscussionOperations::categorize_discussion(
        1,
        "General discussion",
        "This is a general discussion",
    )
    .await;

    assert!(result.is_ok());
    let categorization = result.unwrap();
    assert_eq!(categorization.category, "general");
}

#[tokio::test]
async fn test_categorize_discussion_invalid_number() {
    let result = DiscussionOperations::categorize_discussion(
        0,
        "Test",
        "Test body",
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_categorize_discussion_empty_title() {
    let result = DiscussionOperations::categorize_discussion(
        1,
        "",
        "Test body",
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_track_updates_success() {
    let result = DiscussionOperations::track_updates(1, 5).await;

    assert!(result.is_ok());
    let tracking = result.unwrap();
    assert_eq!(tracking.discussion_number, 1);
    assert!(tracking.new_comments > 0);
}

#[tokio::test]
async fn test_track_updates_invalid_number() {
    let result = DiscussionOperations::track_updates(0, 5).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_thread_success() {
    let result = DiscussionOperations::get_thread(1, "Test Discussion").await;

    assert!(result.is_ok());
    let thread = result.unwrap();
    assert_eq!(thread.discussion_number, 1);
    assert_eq!(thread.title, "Test Discussion");
    assert!(!thread.comments.is_empty());
    assert_eq!(thread.comment_count, thread.comments.len() as u32);
}

#[tokio::test]
async fn test_get_thread_invalid_number() {
    let result = DiscussionOperations::get_thread(0, "Test Discussion").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_thread_empty_title() {
    let result = DiscussionOperations::get_thread(1, "").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_mark_resolved_success() {
    let result = DiscussionOperations::mark_resolved(1, 42).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mark_resolved_invalid_discussion_number() {
    let result = DiscussionOperations::mark_resolved(0, 42).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_mark_resolved_invalid_comment_id() {
    let result = DiscussionOperations::mark_resolved(1, 0).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_categories_success() {
    let result = DiscussionOperations::get_categories().await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert!(!categories.is_empty());

    let names: Vec<_> = categories.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"general"));
    assert!(names.contains(&"help"));
    assert!(names.contains(&"feature-request"));
    assert!(names.contains(&"bug-report"));
}
