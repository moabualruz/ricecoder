//! Unit tests for Gist Manager
//!
//! These tests verify specific functionality and edge cases

use ricecoder_github::{GistManager, GistOptions};

#[tokio::test]
async fn test_create_gist_with_valid_inputs() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager
        .create_gist(
            "test.rs",
            "fn main() { println!(\"Hello\"); }",
            Some("rust".to_string()),
            GistOptions::default(),
        )
        .await;

    assert!(result.is_ok());
    let gist = result.unwrap();
    assert!(!gist.gist_id.is_empty());
    assert!(gist.url.contains("testuser"));
    assert!(gist.url.starts_with("https://gist.github.com/"));
}

#[tokio::test]
async fn test_create_gist_with_empty_filename() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager
        .create_gist("", "content", None, GistOptions::default())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_gist_with_empty_content() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager
        .create_gist("test.rs", "", None, GistOptions::default())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_gist_with_custom_options() {
    let manager = GistManager::new("test_token", "testuser");
    let options = GistOptions::new("Test gist")
        .with_public(false)
        .with_tag("rust")
        .with_tag("example")
        .with_category("snippet");

    let result = manager
        .create_gist(
            "example.rs",
            "fn example() {}",
            Some("rust".to_string()),
            options,
        )
        .await;

    assert!(result.is_ok());
}

#[test]
fn test_generate_gist_url() {
    let manager = GistManager::new("token", "testuser");
    let url = manager.generate_gist_url("abc123def456");

    assert_eq!(url, "https://gist.github.com/testuser/abc123def456");
    assert!(url.contains("testuser"));
    assert!(url.contains("abc123def456"));
}

#[test]
fn test_generate_gist_url_with_different_usernames() {
    let manager1 = GistManager::new("token", "user1");
    let manager2 = GistManager::new("token", "user2");

    let url1 = manager1.generate_gist_url("gist1");
    let url2 = manager2.generate_gist_url("gist1");

    assert!(url1.contains("user1"));
    assert!(url2.contains("user2"));
    assert_ne!(url1, url2);
}

#[tokio::test]
async fn test_update_gist_with_valid_inputs() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager
        .update_gist(
            "abc123",
            "test.rs",
            "fn main() {}",
            Some("rust".to_string()),
        )
        .await;

    assert!(result.is_ok());
    let update = result.unwrap();
    assert_eq!(update.gist_id, "abc123");
    assert!(update.url.contains("abc123"));
}

#[tokio::test]
async fn test_update_gist_with_empty_id() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.update_gist("", "test.rs", "content", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_gist_with_empty_filename() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.update_gist("abc123", "", "content", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_gist_with_empty_content() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.update_gist("abc123", "test.rs", "", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_gist_with_valid_id() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.delete_gist("abc123").await;

    assert!(result.is_ok());
    let lifecycle = result.unwrap();
    assert_eq!(lifecycle.gist_id, "abc123");
    assert_eq!(lifecycle.operation, "delete");
    assert!(lifecycle.success);
}

#[tokio::test]
async fn test_delete_gist_with_empty_id() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.delete_gist("").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_archive_gist_with_valid_id() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.archive_gist("abc123").await;

    assert!(result.is_ok());
    let lifecycle = result.unwrap();
    assert_eq!(lifecycle.gist_id, "abc123");
    assert_eq!(lifecycle.operation, "archive");
    assert!(lifecycle.success);
}

#[tokio::test]
async fn test_archive_gist_with_empty_id() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.archive_gist("").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_gist_metadata() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.get_gist_metadata("abc123").await;

    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert!(metadata.tags.is_empty());
    assert!(!metadata.created_at.is_empty());
    assert!(!metadata.updated_at.is_empty());
}

#[tokio::test]
async fn test_get_gist_metadata_with_empty_id() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.get_gist_metadata("").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_gist_metadata() {
    let manager = GistManager::new("test_token", "testuser");
    let metadata = ricecoder_github::GistMetadata {
        tags: vec!["rust".to_string(), "example".to_string()],
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        category: Some("snippet".to_string()),
    };

    let result = manager
        .update_gist_metadata("abc123", metadata.clone())
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.tags.len(), 2);
    assert_eq!(updated.category, Some("snippet".to_string()));
}

#[tokio::test]
async fn test_update_gist_metadata_with_empty_id() {
    let manager = GistManager::new("test_token", "testuser");
    let metadata = ricecoder_github::GistMetadata {
        tags: vec![],
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        category: None,
    };

    let result = manager.update_gist_metadata("", metadata).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_gist_visibility_public() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.set_gist_visibility("abc123", true).await;

    assert!(result.is_ok());
    let gist = result.unwrap();
    assert_eq!(gist.id, "abc123");
    assert!(gist.public);
}

#[tokio::test]
async fn test_set_gist_visibility_private() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.set_gist_visibility("abc123", false).await;

    assert!(result.is_ok());
    let gist = result.unwrap();
    assert_eq!(gist.id, "abc123");
    assert!(!gist.public);
}

#[tokio::test]
async fn test_set_gist_visibility_with_empty_id() {
    let manager = GistManager::new("test_token", "testuser");
    let result = manager.set_gist_visibility("", true).await;

    assert!(result.is_err());
}

#[test]
fn test_gist_options_builder() {
    let options = GistOptions::new("Test gist")
        .with_public(false)
        .with_tag("rust")
        .with_tag("example")
        .with_category("snippet");

    assert_eq!(options.description, "Test gist");
    assert!(!options.public);
    assert_eq!(options.tags.len(), 2);
    assert_eq!(options.category, Some("snippet".to_string()));
}

#[test]
fn test_gist_options_default() {
    let options = GistOptions::default();

    assert_eq!(options.description, "");
    assert!(options.public);
    assert!(options.tags.is_empty());
    assert_eq!(options.category, None);
}

#[test]
fn test_gist_manager_creation() {
    let manager = GistManager::new("token123", "testuser");
    // Just verify it can be created without panicking
    let _url = manager.generate_gist_url("test");
}
