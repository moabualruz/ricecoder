//! Integration tests for conflict resolution workflows
//!
//! Tests the three conflict resolution strategies: skip, overwrite, and merge.

use ricecoder_files::conflict::ConflictResolver;
use ricecoder_files::models::ConflictResolution;
use tempfile::TempDir;
use tokio::fs;

/// Test skip strategy: abort write and preserve existing file
#[tokio::test]
async fn test_conflict_skip_strategy_preserves_existing() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create existing file
    fs::write(&file_path, "existing content").await.unwrap();

    let resolver = ConflictResolver::new();
    let new_content = "new content";

    // Detect conflict
    let conflict = resolver
        .detect_conflict(&file_path, new_content)
        .await
        .unwrap();

    assert!(conflict.is_some());
    let conflict_info = conflict.unwrap();

    // Apply skip strategy
    let result = resolver.resolve(ConflictResolution::Skip, &conflict_info);
    assert!(result.is_err()); // Skip returns error to abort write

    // Verify file was not modified
    let content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(content, "existing content");
}

/// Test overwrite strategy: replace existing file with new content
#[tokio::test]
async fn test_conflict_overwrite_strategy_replaces_content() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create existing file
    fs::write(&file_path, "existing content").await.unwrap();

    let resolver = ConflictResolver::new();
    let new_content = "new content";

    // Detect conflict
    let conflict = resolver
        .detect_conflict(&file_path, new_content)
        .await
        .unwrap();

    assert!(conflict.is_some());
    let conflict_info = conflict.unwrap();

    // Apply overwrite strategy
    let result = resolver.resolve(ConflictResolution::Overwrite, &conflict_info);
    assert!(result.is_ok()); // Overwrite succeeds

    // Manually write the new content (resolver doesn't do the write)
    fs::write(&file_path, new_content).await.unwrap();

    // Verify file was replaced
    let content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(content, "new content");
}

/// Test merge strategy: combine changes from both versions
#[tokio::test]
async fn test_conflict_merge_strategy_combines_content() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create existing file
    fs::write(&file_path, "existing content").await.unwrap();

    let resolver = ConflictResolver::new();
    let new_content = "new content";

    // Detect conflict
    let conflict = resolver
        .detect_conflict(&file_path, new_content)
        .await
        .unwrap();

    assert!(conflict.is_some());
    let conflict_info = conflict.unwrap();

    // Apply merge strategy
    let result = resolver.resolve(ConflictResolution::Merge, &conflict_info);
    assert!(result.is_ok()); // Merge succeeds

    // Merge content using the resolver's merge function
    let merged = ConflictResolver::merge_content(&conflict_info.existing_content, new_content);

    // Verify merged content contains both versions
    assert!(merged.contains("existing content"));
    assert!(merged.contains("new content"));
    assert!(merged.contains("<<<<<<< EXISTING"));
    assert!(merged.contains("======="));
    assert!(merged.contains(">>>>>>> NEW"));
}

/// Test conflict detection with identical content
#[tokio::test]
async fn test_conflict_detection_with_identical_content() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create existing file
    fs::write(&file_path, "same content").await.unwrap();

    let resolver = ConflictResolver::new();
    let new_content = "same content";

    // Detect conflict - should still detect it even if content is identical
    let conflict = resolver
        .detect_conflict(&file_path, new_content)
        .await
        .unwrap();

    assert!(conflict.is_some());
    let conflict_info = conflict.unwrap();

    // Verify conflict info contains the content
    assert_eq!(conflict_info.existing_content, "same content");
    assert_eq!(conflict_info.new_content, "same content");
}

/// Test conflict detection with non-existent file
#[tokio::test]
async fn test_conflict_detection_no_conflict_for_new_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.txt");

    let resolver = ConflictResolver::new();
    let new_content = "new content";

    // Detect conflict - should return None for non-existent file
    let conflict = resolver
        .detect_conflict(&file_path, new_content)
        .await
        .unwrap();

    assert!(conflict.is_none());
}

/// Test merge with identical content
#[test]
fn test_merge_identical_content_returns_content() {
    let existing = "same content";
    let new = "same content";

    let merged = ConflictResolver::merge_content(existing, new);

    // When content is identical, merge should return the content as-is
    assert_eq!(merged, "same content");
}

/// Test merge with different content
#[test]
fn test_merge_different_content_includes_markers() {
    let existing = "line 1\nline 2";
    let new = "line 1\nline 3";

    let merged = ConflictResolver::merge_content(existing, new);

    // Verify merge markers are present
    assert!(merged.contains("<<<<<<< EXISTING"));
    assert!(merged.contains("======="));
    assert!(merged.contains(">>>>>>> NEW"));

    // Verify both versions are in the merged content
    assert!(merged.contains("line 2"));
    assert!(merged.contains("line 3"));
}

/// Test conflict info contains correct path
#[tokio::test]
async fn test_conflict_info_contains_correct_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create existing file
    fs::write(&file_path, "existing").await.unwrap();

    let resolver = ConflictResolver::new();
    let conflict = resolver
        .detect_conflict(&file_path, "new")
        .await
        .unwrap()
        .unwrap();

    // Verify conflict info has correct path
    assert_eq!(conflict.path, file_path);
}

/// Test multiple conflict resolutions in sequence
#[tokio::test]
async fn test_sequential_conflict_resolutions() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create existing file
    fs::write(&file_path, "original").await.unwrap();

    let resolver = ConflictResolver::new();

    // First conflict with skip strategy
    let conflict1 = resolver
        .detect_conflict(&file_path, "attempt1")
        .await
        .unwrap()
        .unwrap();

    let result1 = resolver.resolve(ConflictResolution::Skip, &conflict1);
    assert!(result1.is_err());

    // File should still have original content
    let content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(content, "original");

    // Second conflict with overwrite strategy
    let conflict2 = resolver
        .detect_conflict(&file_path, "attempt2")
        .await
        .unwrap()
        .unwrap();

    let result2 = resolver.resolve(ConflictResolution::Overwrite, &conflict2);
    assert!(result2.is_ok());

    // Manually apply the overwrite
    fs::write(&file_path, "attempt2").await.unwrap();

    // Verify file was updated
    let content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(content, "attempt2");
}
