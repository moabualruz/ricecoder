//! Property-based tests for Gist Manager
//!
//! These tests verify correctness properties that should hold across all valid inputs

use proptest::prelude::*;
use ricecoder_github::{GistManager, GistOptions};

// Strategy for generating valid filenames
fn valid_filename_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]{1,50}\.(rs|py|js|ts|go|java|cpp|c|h|md|txt|json|yaml)"
        .prop_map(|s| s.to_string())
}

// Strategy for generating valid code content
fn valid_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,;:(){}!?@#$%^&*+=|/'`~]{10,500}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("content must not be empty", |s| !s.is_empty())
}

// Strategy for generating programming languages
fn language_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some("rust".to_string())),
        Just(Some("python".to_string())),
        Just(Some("javascript".to_string())),
        Just(Some("typescript".to_string())),
        Just(Some("go".to_string())),
        Just(Some("java".to_string())),
    ]
}

// Strategy for generating gist descriptions
fn description_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,]{0,100}".prop_map(|s| s.trim().to_string())
}

// Strategy for generating tags
fn tags_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(r"[a-z0-9\-]{1,20}", 0..5)
}

// Strategy for generating gist IDs
fn gist_id_strategy() -> impl Strategy<Value = String> {
    r"[a-f0-9]{8,16}".prop_map(|s| s.to_string())
}

// Strategy for generating gist options
fn gist_options_strategy() -> impl Strategy<Value = GistOptions> {
    (description_strategy(), any::<bool>(), tags_strategy()).prop_map(
        |(description, public, tags)| {
            let mut opts = GistOptions::new(description).with_public(public);
            for tag in tags {
                opts = opts.with_tag(tag);
            }
            opts
        },
    )
}

// **Feature: ricecoder-github, Property 31: Gist Creation**
// *For any* code snippet, the system SHALL create a Gist with the correct content and metadata.
// **Validates: Requirements 7.1**
proptest! {
    #[test]
    fn prop_gist_creation_with_valid_inputs(
        filename in valid_filename_strategy(),
        content in valid_content_strategy(),
        language in language_strategy(),
        options in gist_options_strategy()
    ) {
        let manager = GistManager::new("test_token", "testuser");

        // Execute gist creation
        let result = futures::executor::block_on(async {
            manager.create_gist(&filename, &content, language.clone(), options.clone()).await
        });

        // Property: Creation should succeed with valid inputs
        prop_assert!(result.is_ok());

        let gist = result.unwrap();

        // Property: Gist ID should not be empty
        prop_assert!(!gist.gist_id.is_empty());

        // Property: URL should contain username and gist ID
        prop_assert!(gist.url.contains("testuser"));
        prop_assert!(gist.url.contains(&gist.gist_id));

        // Property: URL should be a valid GitHub gist URL
        prop_assert!(gist.url.starts_with("https://gist.github.com/"));

        // Property: Raw URL should be present
        prop_assert!(gist.raw_url.is_some());

        // Property: HTML URL should be present
        prop_assert!(gist.html_url.is_some());
    }
}

// **Feature: ricecoder-github, Property 32: Gist URL Generation**
// *For any* created Gist, the system SHALL generate a shareable URL.
// **Validates: Requirements 7.2**
proptest! {
    #[test]
    fn prop_gist_url_generation(gist_id in gist_id_strategy()) {
        let manager = GistManager::new("test_token", "testuser");

        // Generate URL
        let url = manager.generate_gist_url(&gist_id);

        // Property: URL should contain username
        prop_assert!(url.contains("testuser"));

        // Property: URL should contain gist ID
        prop_assert!(url.contains(&gist_id));

        // Property: URL should be a valid GitHub gist URL
        prop_assert!(url.starts_with("https://gist.github.com/"));

        // Property: URL should have correct format
        let expected = format!("https://gist.github.com/testuser/{}", gist_id);
        prop_assert_eq!(url, expected);
    }
}

// **Feature: ricecoder-github, Property 33: Gist Update Idempotence**
// *For any* Gist update with the same content, performing the update twice SHALL result in identical Gist state.
// **Validates: Requirements 7.3**
proptest! {
    #[test]
    fn prop_gist_update_idempotence(
        gist_id in gist_id_strategy(),
        filename in valid_filename_strategy(),
        content in valid_content_strategy(),
        language in language_strategy()
    ) {
        let manager = GistManager::new("test_token", "testuser");

        // First update
        let result1 = futures::executor::block_on(async {
            manager.update_gist(&gist_id, &filename, &content, language.clone()).await
        });

        // Second update with same content
        let result2 = futures::executor::block_on(async {
            manager.update_gist(&gist_id, &filename, &content, language.clone()).await
        });

        // Property: Both updates should succeed
        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let update1 = result1.unwrap();
        let update2 = result2.unwrap();

        // Property: Gist IDs should be identical
        prop_assert_eq!(&update1.gist_id, &update2.gist_id);

        // Property: URLs should be identical
        prop_assert_eq!(&update1.url, &update2.url);

        // Property: Both should have the same gist ID
        prop_assert_eq!(&update1.gist_id, &gist_id);
    }
}

// **Feature: ricecoder-github, Property 34: Gist Metadata Support**
// *For any* Gist, the system SHALL store and retrieve descriptions and tags correctly.
// **Validates: Requirements 7.4**
proptest! {
    #[test]
    fn prop_gist_metadata_support(
        gist_id in gist_id_strategy(),
        tags in tags_strategy(),
        category in r"[a-z]{1,20}".prop_map(|s| Some(s))
    ) {
        let manager = GistManager::new("test_token", "testuser");

        // Update metadata
        let result = futures::executor::block_on(async {
            let metadata = ricecoder_github::GistMetadata {
                tags: tags.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                category: category.clone(),
            };
            manager.update_gist_metadata(&gist_id, metadata).await
        });

        // Property: Metadata update should succeed
        prop_assert!(result.is_ok());

        let metadata = result.unwrap();

        // Property: Tags should be preserved
        prop_assert_eq!(metadata.tags, tags);

        // Property: Category should be preserved
        prop_assert_eq!(metadata.category, category);

        // Property: Timestamps should be valid ISO 8601
        prop_assert!(!metadata.created_at.is_empty());
        prop_assert!(!metadata.updated_at.is_empty());
    }
}

// **Feature: ricecoder-github, Property 35: Gist Lifecycle Management**
// *For any* Gist, the system SHALL support deletion and archival operations.
// **Validates: Requirements 7.5**
proptest! {
    #[test]
    fn prop_gist_lifecycle_management(gist_id in gist_id_strategy()) {
        let manager = GistManager::new("test_token", "testuser");

        // Test deletion
        let delete_result = futures::executor::block_on(async {
            manager.delete_gist(&gist_id).await
        });

        // Property: Deletion should succeed
        prop_assert!(delete_result.is_ok());

        let delete_lifecycle = delete_result.unwrap();

        // Property: Operation should be "delete"
        prop_assert_eq!(delete_lifecycle.operation, "delete");

        // Property: Success flag should be true
        prop_assert!(delete_lifecycle.success);

        // Property: Gist ID should be preserved
        prop_assert_eq!(&delete_lifecycle.gist_id, &gist_id);

        // Test archival
        let archive_result = futures::executor::block_on(async {
            manager.archive_gist(&gist_id).await
        });

        // Property: Archival should succeed
        prop_assert!(archive_result.is_ok());

        let archive_lifecycle = archive_result.unwrap();

        // Property: Operation should be "archive"
        prop_assert_eq!(archive_lifecycle.operation, "archive");

        // Property: Success flag should be true
        prop_assert!(archive_lifecycle.success);

        // Property: Gist ID should be preserved
        prop_assert_eq!(archive_lifecycle.gist_id, gist_id);
    }
}
