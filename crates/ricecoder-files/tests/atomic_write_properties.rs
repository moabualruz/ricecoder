//! Property-based tests for atomic write integrity
//! **Feature: ricecoder-files, Property 1: Atomic Write Integrity**
//! **Validates: Requirements 1.1, 1.4**

use ricecoder_files::ConflictResolution;
use tokio::fs;

/// Property 1a: Write either completes fully or not at all (no partial writes)
///
/// For any valid file path and content, after a successful write,
/// the file should contain exactly the written content (no partial writes).
#[tokio::test]
async fn prop_atomic_write_completes_fully_simple_content() {
    use ricecoder_files::SafeWriter;

    let writer = SafeWriter::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let content = "test content";

    // Perform write
    let result = writer
        .write(&path, content, ConflictResolution::Overwrite)
        .await;

    // Write should succeed
    assert!(result.is_ok(), "Write should succeed");

    // File should exist
    assert!(path.exists(), "File should exist after write");

    // File should contain exactly the written content
    let written = fs::read_to_string(&path)
        .await
        .expect("Should be able to read written file");

    assert_eq!(written, content, "Written content should match source exactly");
}

/// Property 1a: Write either completes fully or not at all (no partial writes)
/// Testing with empty content
#[tokio::test]
async fn prop_atomic_write_completes_fully_empty_content() {
    use ricecoder_files::SafeWriter;

    let writer = SafeWriter::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let content = "";

    // Perform write
    let result = writer
        .write(&path, content, ConflictResolution::Overwrite)
        .await;

    // Write should succeed
    assert!(result.is_ok(), "Write should succeed");

    // File should exist
    assert!(path.exists(), "File should exist after write");

    // File should contain exactly the written content
    let written = fs::read_to_string(&path)
        .await
        .expect("Should be able to read written file");

    assert_eq!(written, content, "Written content should match source exactly");
}

/// Property 1a: Write either completes fully or not at all (no partial writes)
/// Testing with multiline content
#[tokio::test]
async fn prop_atomic_write_completes_fully_multiline_content() {
    use ricecoder_files::SafeWriter;

    let writer = SafeWriter::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let content = "line 1\nline 2\nline 3\n";

    // Perform write
    let result = writer
        .write(&path, content, ConflictResolution::Overwrite)
        .await;

    // Write should succeed
    assert!(result.is_ok(), "Write should succeed");

    // File should exist
    assert!(path.exists(), "File should exist after write");

    // File should contain exactly the written content
    let written = fs::read_to_string(&path)
        .await
        .expect("Should be able to read written file");

    assert_eq!(written, content, "Written content should match source exactly");
}

/// Property 1b: Original file unchanged if write fails
///
/// For any existing file, if a write operation fails (e.g., due to conflict with Skip strategy),
/// the original file content should remain unchanged.
#[tokio::test]
async fn prop_atomic_write_preserves_original_on_failure() {
    use ricecoder_files::SafeWriter;

    let writer = SafeWriter::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let original_content = "original content";
    let new_content = "new content";

    // Create original file
    fs::write(&path, original_content)
        .await
        .expect("Should be able to write original file");

    // Attempt write with Skip strategy (should fail due to conflict)
    let result = writer
        .write(&path, new_content, ConflictResolution::Skip)
        .await;

    // Write should fail
    assert!(result.is_err(), "Write should fail with Skip strategy on conflict");

    // Original file should still exist
    assert!(path.exists(), "Original file should still exist");

    // Original file content should be unchanged
    let current = fs::read_to_string(&path)
        .await
        .expect("Should be able to read file");

    assert_eq!(
        current, original_content,
        "Original content should be preserved after failed write"
    );
}

/// Property 1c: Multiple writes are atomic
///
/// For any sequence of writes to the same file, each write should be atomic.
/// After each write, the file should contain exactly the written content.
#[tokio::test]
async fn prop_atomic_write_multiple_writes() {
    use ricecoder_files::SafeWriter;

    let writer = SafeWriter::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");

    let content1 = "first content";
    let content2 = "second content";
    let content3 = "third content";

    // First write
    let result1 = writer
        .write(&path, content1, ConflictResolution::Overwrite)
        .await;
    assert!(result1.is_ok(), "First write should succeed");

    let written1 = fs::read_to_string(&path)
        .await
        .expect("Should be able to read file");
    assert_eq!(written1, content1, "File should contain first content");

    // Second write
    let result2 = writer
        .write(&path, content2, ConflictResolution::Overwrite)
        .await;
    assert!(result2.is_ok(), "Second write should succeed");

    let written2 = fs::read_to_string(&path)
        .await
        .expect("Should be able to read file");
    assert_eq!(written2, content2, "File should contain second content");

    // Third write
    let result3 = writer
        .write(&path, content3, ConflictResolution::Overwrite)
        .await;
    assert!(result3.is_ok(), "Third write should succeed");

    let written3 = fs::read_to_string(&path)
        .await
        .expect("Should be able to read file");
    assert_eq!(written3, content3, "File should contain third content");
}

/// Property 1d: Write creates parent directories
///
/// For any file path with non-existent parent directories,
/// a successful write should create all necessary parent directories.
#[tokio::test]
async fn prop_atomic_write_creates_parent_dirs() {
    use ricecoder_files::SafeWriter;

    let writer = SafeWriter::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("a/b/c/d/test.txt");

    let content = "test content";

    // Ensure parent doesn't exist
    assert!(!path.parent().unwrap().exists(), "Parent should not exist initially");

    // Perform write
    let result = writer
        .write(&path, content, ConflictResolution::Overwrite)
        .await;

    assert!(result.is_ok(), "Write should succeed");
    assert!(path.exists(), "File should exist");
    assert!(path.parent().unwrap().exists(), "Parent directories should be created");

    let written = fs::read_to_string(&path)
        .await
        .expect("Should be able to read file");
    assert_eq!(written, content, "Content should match");
}
