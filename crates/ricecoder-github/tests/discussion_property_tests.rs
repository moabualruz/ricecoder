//! Property-based tests for Discussion Manager
//!
//! These tests verify correctness properties that should hold across all valid inputs

use proptest::prelude::*;
use ricecoder_github::{DiscussionManager, DiscussionOperations};

// Strategy for generating valid discussion titles
fn valid_title_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,!?]{5,100}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("title must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid discussion bodies
fn valid_body_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,;:(){}!?@#$%^&*+=|/'`~\n]{10,500}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("body must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid categories
fn category_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("general".to_string()),
        Just("help".to_string()),
        Just("feature-request".to_string()),
        Just("bug-report".to_string()),
    ]
}

// Strategy for generating valid discussion numbers
fn discussion_number_strategy() -> impl Strategy<Value = u32> {
    1u32..100000u32
}

// Strategy for generating valid response content
fn response_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,;:(){}!?@#$%^&*+=|/'`~]{5,500}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("content must not be empty", |s| !s.is_empty())
}

// **Feature: ricecoder-github, Property 36: Discussion Creation**
// *For any* discussion topic, the system SHALL create a GitHub Discussion with correct content.
// **Validates: Requirements 8.1**
proptest! {
    #[test]
    fn prop_discussion_creation_with_valid_inputs(
        title in valid_title_strategy(),
        body in valid_body_strategy(),
        category in category_strategy()
    ) {
        let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

        // Execute discussion creation
        let result = futures::executor::block_on(async {
            manager.create_discussion(&title, &body, &category).await
        });

        // Property: Creation should succeed with valid inputs
        prop_assert!(result.is_ok());

        let creation_result = result.unwrap();

        // Property: Discussion ID should not be zero
        prop_assert!(creation_result.discussion_id > 0);

        // Property: Discussion number should not be zero
        prop_assert!(creation_result.number > 0);

        // Property: URL should contain owner and repo
        prop_assert!(creation_result.url.contains("testowner"));
        prop_assert!(creation_result.url.contains("testrepo"));

        // Property: URL should be a valid GitHub discussions URL
        prop_assert!(creation_result.url.starts_with("https://github.com/"));
        prop_assert!(creation_result.url.contains("/discussions/"));

        // Property: Category should match input
        prop_assert_eq!(creation_result.category, category);
    }
}

// **Feature: ricecoder-github, Property 37: Discussion Response Posting**
// *For any* discussion response, the system SHALL post the response to the correct discussion thread.
// **Validates: Requirements 8.2**
proptest! {
    #[test]
    fn prop_discussion_response_posting(
        discussion_number in discussion_number_strategy(),
        content in response_content_strategy()
    ) {
        let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

        // Execute response posting
        let result = futures::executor::block_on(async {
            manager.post_response(discussion_number, &content).await
        });

        // Property: Response posting should succeed with valid inputs
        prop_assert!(result.is_ok());

        let response = result.unwrap();

        // Property: Response ID should not be zero
        prop_assert!(response.response_id > 0);

        // Property: Discussion number should match input
        prop_assert_eq!(response.discussion_number, discussion_number);

        // Property: Content should match input
        prop_assert_eq!(response.content, content);

        // Property: Author should be set
        prop_assert!(!response.author.is_empty());
    }
}

// **Feature: ricecoder-github, Property 38: Discussion Insight Extraction**
// *For any* discussion thread, the system SHALL extract insights from the discussion content.
// **Validates: Requirements 8.3**
proptest! {
    #[test]
    fn prop_discussion_insight_extraction(
        discussion_number in discussion_number_strategy()
    ) {
        let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

        // Execute insight extraction
        let result = futures::executor::block_on(async {
            manager.extract_insights(discussion_number).await
        });

        // Property: Insight extraction should succeed with valid discussion number
        prop_assert!(result.is_ok());

        let insights = result.unwrap();

        // Property: Should extract at least one insight
        prop_assert!(!insights.is_empty());

        // Property: All insights should have non-empty type
        for insight in &insights {
            prop_assert!(!insight.insight_type.is_empty());
            prop_assert!(!insight.content.is_empty());
        }

        // Property: All confidence scores should be between 0.0 and 1.0
        for insight in &insights {
            prop_assert!(insight.confidence >= 0.0 && insight.confidence <= 1.0);
        }
    }
}

// **Feature: ricecoder-github, Property 39: Discussion Summary Generation**
// *For any* discussion, the system SHALL generate a summary containing key points and decisions.
// **Validates: Requirements 8.4**
proptest! {
    #[test]
    fn prop_discussion_summary_generation(
        discussion_number in discussion_number_strategy(),
        title in valid_title_strategy()
    ) {
        let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

        // Execute summary generation
        let result = futures::executor::block_on(async {
            manager.generate_summary(discussion_number, &title).await
        });

        // Property: Summary generation should succeed with valid inputs
        prop_assert!(result.is_ok());

        let summary = result.unwrap();

        // Property: Discussion number should match input
        prop_assert_eq!(summary.discussion_number, discussion_number);

        // Property: Title should match input
        prop_assert_eq!(summary.title, title);

        // Property: Summary content should not be empty
        prop_assert!(!summary.content.is_empty());

        // Property: Should have at least one insight
        prop_assert!(!summary.insights.is_empty());

        // Property: Should have at least one participant
        prop_assert!(!summary.participants.is_empty());

        // Property: Status should not be empty
        prop_assert!(!summary.status.is_empty());
    }
}

// **Feature: ricecoder-github, Property 40: Discussion Status Monitoring**
// *For any* discussion, the system SHALL track and report status and updates.
// **Validates: Requirements 8.5**
proptest! {
    #[test]
    fn prop_discussion_status_monitoring(
        discussion_number in discussion_number_strategy()
    ) {
        let manager = DiscussionManager::new("test_token", "testowner", "testrepo");

        // Execute status monitoring
        let result = futures::executor::block_on(async {
            manager.monitor_status(discussion_number).await
        });

        // Property: Status monitoring should succeed with valid discussion number
        prop_assert!(result.is_ok());

        let status = result.unwrap();

        // Property: Discussion number should match input
        prop_assert_eq!(status.discussion_number, discussion_number);

        // Property: Status should not be empty
        prop_assert!(!status.status.is_empty());

        // Property: Comment count is u32, so it's always non-negative

        // Property: Last activity should be recent (within last hour)
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(status.last_activity);
        prop_assert!(duration.num_seconds() >= 0 && duration.num_seconds() < 3600);
    }
}

// Additional property tests for DiscussionOperations

// **Feature: ricecoder-github, Property: Discussion Categorization Consistency**
// *For any* discussion with the same title and body, categorization should be consistent.
proptest! {
    #[test]
    fn prop_discussion_categorization_consistency(
        discussion_number in discussion_number_strategy(),
        title in valid_title_strategy(),
        body in valid_body_strategy()
    ) {
        // Execute categorization twice
        let result1 = futures::executor::block_on(async {
            DiscussionOperations::categorize_discussion(discussion_number, &title, &body).await
        });

        let result2 = futures::executor::block_on(async {
            DiscussionOperations::categorize_discussion(discussion_number, &title, &body).await
        });

        // Property: Both categorizations should succeed
        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let cat1 = result1.unwrap();
        let cat2 = result2.unwrap();

        // Property: Categories should be identical
        prop_assert_eq!(cat1.category, cat2.category);

        // Property: Confidence scores should be identical
        prop_assert_eq!(cat1.confidence, cat2.confidence);
    }
}

// **Feature: ricecoder-github, Property: Discussion Thread Retrieval**
// *For any* valid discussion number and title, thread retrieval should succeed.
proptest! {
    #[test]
    fn prop_discussion_thread_retrieval(
        discussion_number in discussion_number_strategy(),
        title in valid_title_strategy()
    ) {
        // Execute thread retrieval
        let result = futures::executor::block_on(async {
            DiscussionOperations::get_thread(discussion_number, &title).await
        });

        // Property: Thread retrieval should succeed
        prop_assert!(result.is_ok());

        let thread = result.unwrap();

        // Property: Discussion number should match input
        prop_assert_eq!(thread.discussion_number, discussion_number);

        // Property: Title should match input
        prop_assert_eq!(thread.title, title);

        // Property: Should have at least one comment
        prop_assert!(!thread.comments.is_empty());

        // Property: Comment count should match comments length
        prop_assert_eq!(thread.comment_count, thread.comments.len() as u32);

        // Property: All comments should have non-empty content
        for comment in &thread.comments {
            prop_assert!(!comment.content.is_empty());
            prop_assert!(!comment.author.is_empty());
        }
    }
}

// **Feature: ricecoder-github, Property: Discussion Categories Availability**
// *For any* request for categories, the system SHALL return available categories.
proptest! {
    #[test]
    fn prop_discussion_categories_availability(_dummy in Just(())) {
        // Execute category retrieval
        let result = futures::executor::block_on(async {
            DiscussionOperations::get_categories().await
        });

        // Property: Category retrieval should succeed
        prop_assert!(result.is_ok());

        let categories = result.unwrap();

        // Property: Should have at least one category
        prop_assert!(!categories.is_empty());

        // Property: All categories should have non-empty names
        for category in &categories {
            prop_assert!(!category.name.is_empty());
            prop_assert!(!category.description.is_empty());
        }

        // Property: Should have expected categories
        let names: Vec<_> = categories.iter().map(|c| c.name.as_str()).collect();
        prop_assert!(names.contains(&"general"));
        prop_assert!(names.contains(&"help"));
    }
}
