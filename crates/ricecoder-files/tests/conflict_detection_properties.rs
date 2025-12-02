//! Property-based tests for conflict detection
//! **Feature: ricecoder-files, Property 6: Conflict Detection**
//! **Validates: Requirements 1.2**

use ricecoder_files::ConflictResolver;
use tokio::fs;

/// Property 6a: Conflicts detected before overwrite
///
/// For any existing file with different content, detect_conflict should
/// return Some(ConflictInfo) before any overwrite occurs.
#[tokio::test]
async fn prop_conflict_detected_before_overwrite() {
    let resolver = ConflictResolver::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let existing_content = "existing content";
    let new_content = "new content";

    // Create existing file
    fs::write(&path, existing_content)
        .await
        .expect("Should be able to write file");

    // Detect conflict
    let result = resolver
        .detect_conflict(&path, new_content)
        .await;

    assert!(result.is_ok(), "detect_conflict should succeed");

    let conflict = result.unwrap();
    assert!(conflict.is_some(), "Conflict should be detected");

    // Verify file was not modified
    let current = fs::read_to_string(&path)
        .await
        .expect("Should be able to read file");
    assert_eq!(
        current, existing_content,
        "File should not be modified during conflict detection"
    );
}

/// Property 6b: Conflict info contains correct file paths and content
///
/// For any detected conflict, ConflictInfo should contain:
/// - Correct path
/// - Correct existing content
/// - Correct new content
#[tokio::test]
async fn prop_conflict_info_contains_correct_data() {
    let resolver = ConflictResolver::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let existing_content = "existing content";
    let new_content = "new content";

    // Create existing file
    fs::write(&path, existing_content)
        .await
        .expect("Should be able to write file");

    // Detect conflict
    let result = resolver
        .detect_conflict(&path, new_content)
        .await;

    assert!(result.is_ok(), "detect_conflict should succeed");

    let conflict_info = result
        .unwrap()
        .expect("Conflict should be detected");

    // Verify path
    assert_eq!(
        conflict_info.path, path,
        "Conflict path should match"
    );

    // Verify existing content
    assert_eq!(
        conflict_info.existing_content, existing_content,
        "Existing content should match"
    );

    // Verify new content
    assert_eq!(
        conflict_info.new_content, new_content,
        "New content should match"
    );
}

/// Property 6c: No conflict detected for non-existent files
///
/// For any non-existent file path, detect_conflict should return None.
#[tokio::test]
async fn prop_no_conflict_for_nonexistent_file() {
    let resolver = ConflictResolver::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("nonexistent.txt");

    let new_content = "new content";

    // Ensure file doesn't exist
    assert!(!path.exists(), "File should not exist");

    // Detect conflict
    let result = resolver
        .detect_conflict(&path, new_content)
        .await;

    assert!(result.is_ok(), "detect_conflict should succeed");

    let conflict = result.unwrap();
    assert!(conflict.is_none(), "No conflict should be detected for non-existent file");
}

/// Property 6d: Conflict detected for identical content
///
/// For any existing file, even if the new content is identical,
/// detect_conflict should still return Some(ConflictInfo) because
/// the file exists (conflict is about existence, not content difference).
#[tokio::test]
async fn prop_conflict_detected_for_identical_content() {
    let resolver = ConflictResolver::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let content = "identical content";

    // Create existing file
    fs::write(&path, content)
        .await
        .expect("Should be able to write file");

    // Detect conflict with identical content
    let result = resolver
        .detect_conflict(&path, content)
        .await;

    assert!(result.is_ok(), "detect_conflict should succeed");

    let conflict = result.unwrap();
    assert!(conflict.is_some(), "Conflict should be detected even for identical content");

    let conflict_info = conflict.unwrap();
    assert_eq!(
        conflict_info.existing_content, content,
        "Existing content should match"
    );
    assert_eq!(
        conflict_info.new_content, content,
        "New content should match"
    );
}

/// Property 6e: Multiple conflict detections are consistent
///
/// For any file, multiple calls to detect_conflict with the same
/// new content should return consistent results.
#[tokio::test]
async fn prop_conflict_detection_consistent() {
    let resolver = ConflictResolver::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let existing_content = "existing content";
    let new_content = "new content";

    // Create existing file
    fs::write(&path, existing_content)
        .await
        .expect("Should be able to write file");

    // First detection
    let result1 = resolver
        .detect_conflict(&path, new_content)
        .await;

    assert!(result1.is_ok(), "First detection should succeed");

    let conflict1 = result1.unwrap();
    assert!(conflict1.is_some(), "First detection should find conflict");

    // Second detection
    let result2 = resolver
        .detect_conflict(&path, new_content)
        .await;

    assert!(result2.is_ok(), "Second detection should succeed");

    let conflict2 = result2.unwrap();
    assert!(conflict2.is_some(), "Second detection should find conflict");

    // Both should have same data
    let info1 = conflict1.unwrap();
    let info2 = conflict2.unwrap();

    assert_eq!(info1.path, info2.path, "Paths should match");
    assert_eq!(
        info1.existing_content, info2.existing_content,
        "Existing content should match"
    );
    assert_eq!(
        info1.new_content, info2.new_content,
        "New content should match"
    );
}

/// Property 6f: Conflict detection with empty content
///
/// For any existing file, detect_conflict with empty new content
/// should still detect a conflict.
#[tokio::test]
async fn prop_conflict_detected_with_empty_new_content() {
    let resolver = ConflictResolver::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let existing_content = "existing content";
    let new_content = "";

    // Create existing file
    fs::write(&path, existing_content)
        .await
        .expect("Should be able to write file");

    // Detect conflict
    let result = resolver
        .detect_conflict(&path, new_content)
        .await;

    assert!(result.is_ok(), "detect_conflict should succeed");

    let conflict = result.unwrap();
    assert!(conflict.is_some(), "Conflict should be detected");

    let conflict_info = conflict.unwrap();
    assert_eq!(
        conflict_info.existing_content, existing_content,
        "Existing content should match"
    );
    assert_eq!(
        conflict_info.new_content, new_content,
        "New content should be empty"
    );
}
