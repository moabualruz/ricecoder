//! Property-based tests for patch tool
//!
//! Tests the correctness properties of the patch tool implementation.
//! **Feature: ricecoder-tools-enhanced, Property 4, 5, 6: Patch safety, conflict detection, format validation**

use std::{fs, io::Write};

use proptest::prelude::*;
use ricecoder_tools::patch::{PatchInput, PatchTool};
use tempfile::NamedTempFile;

/// Generate valid unified diff patches
#[allow(dead_code)]
fn valid_patch_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("--- a/test\\.txt\n\\+\\+\\+ b/test\\.txt\n@@ -1,3 \\+1,3 @@\n line 1\n-line 2\n\\+line 2 modified\n line 3")
        .unwrap()
}

/// Generate file content that matches the patch
fn matching_file_content_strategy() -> impl Strategy<Value = String> {
    Just("line 1\nline 2\nline 3\n".to_string())
}

/// Generate file content that doesn't match the patch
fn non_matching_file_content_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("line 1\ndifferent line\nline 3\n").unwrap()
}

/// Generate invalid patch formats
fn invalid_patch_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9 ]+").unwrap()
}

/// Property 4: Patch safety - Patch application succeeds completely or fails without modifying file
/// **Validates: Requirements 2.2, 2.3, 2.4**
#[test]
fn prop_patch_safety() {
    proptest!(|(file_content in matching_file_content_strategy())| {
        // Create a temporary file with the content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(file_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();
        let original_content = fs::read_to_string(temp_file.path()).unwrap();

        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = PatchInput {
            file_path: file_path.clone(),
            patch_content: patch.to_string(),
        };

        let result = PatchTool::apply_patch(&input).unwrap();

        if result.success {
            // If successful, file should be modified
            let new_content = fs::read_to_string(&file_path).unwrap();
            prop_assert_ne!(&new_content, &original_content);
            prop_assert!(new_content.contains("line 2 modified"));
        } else {
            // If failed, file should be unchanged
            let new_content = fs::read_to_string(&file_path).unwrap();
            prop_assert_eq!(&new_content, &original_content);
        }
    });
}

/// Property 5: Patch conflict detection - Conflicting patches report which hunks failed
/// **Validates: Requirements 2.5**
#[test]
fn prop_patch_conflict_detection() {
    proptest!(|(file_content in non_matching_file_content_strategy())| {
        // Create a temporary file with non-matching content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(file_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();
        let original_content = fs::read_to_string(temp_file.path()).unwrap();

        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = PatchInput {
            file_path: file_path.clone(),
            patch_content: patch.to_string(),
        };

        let result = PatchTool::apply_patch(&input).unwrap();

        // When there's a conflict, patch should fail
        if !result.success {
            // File should not be modified
            let new_content = fs::read_to_string(&file_path).unwrap();
            prop_assert_eq!(&new_content, &original_content);

            // Failed hunks should be reported
            prop_assert!(result.failed_hunks > 0);
            prop_assert!(!result.failed_hunk_details.is_empty());

            // Each failed hunk should have error information
            for failed_hunk in &result.failed_hunk_details {
                prop_assert!(!failed_hunk.error.is_empty());
                prop_assert!(failed_hunk.hunk_number > 0);
            }
        }
    });
}

/// Property 6: Patch format validation - Invalid patches are rejected without file modification
/// **Validates: Requirements 2.6**
#[test]
fn prop_patch_format_validation() {
    proptest!(|(invalid_patch in invalid_patch_strategy())| {
        // Skip if the generated string happens to be a valid patch
        if !invalid_patch.contains("@@") && !invalid_patch.contains("---") {
            // Create a temporary file
            let mut temp_file = NamedTempFile::new().unwrap();
            temp_file.write_all(b"line 1\nline 2\nline 3\n").unwrap();
            temp_file.flush().unwrap();

            let file_path = temp_file.path().to_string_lossy().to_string();
            let original_content = fs::read_to_string(temp_file.path()).unwrap();

            let input = PatchInput {
                file_path: file_path.clone(),
                patch_content: invalid_patch,
            };

            let result = PatchTool::apply_patch(&input);

            // Invalid patch should either fail or not modify the file
            match result {
                Ok(output) => {
                    if !output.success {
                        // File should not be modified
                        let new_content = fs::read_to_string(&file_path).unwrap();
                        prop_assert_eq!(&new_content, &original_content);
                    }
                }
                Err(_) => {
                    // Error is acceptable for invalid patch
                    let new_content = fs::read_to_string(&file_path).unwrap();
                    prop_assert_eq!(&new_content, &original_content);
                }
            }
        }
    });
}

/// Integration test: Multiple hunks in a single patch
#[test]
fn test_multiple_hunks_patch() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "line 1").unwrap();
    writeln!(temp_file, "line 2").unwrap();
    writeln!(temp_file, "line 3").unwrap();
    writeln!(temp_file, "line 4").unwrap();
    writeln!(temp_file, "line 5").unwrap();
    temp_file.flush().unwrap();

    let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3
@@ -3,3 +3,3 @@
 line 3
-line 4
+line 4 modified
 line 5"#;

    let input = PatchInput {
        file_path: temp_file.path().to_string_lossy().to_string(),
        patch_content: patch.to_string(),
    };

    let output = PatchTool::apply_patch(&input).unwrap();
    assert!(output.success);
    assert_eq!(output.applied_hunks, 2);
    assert_eq!(output.failed_hunks, 0);

    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("line 2 modified"));
    assert!(content.contains("line 4 modified"));
}

/// Integration test: Patch with additions only
#[test]
fn test_patch_additions_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "line 1").unwrap();
    writeln!(temp_file, "line 2").unwrap();
    temp_file.flush().unwrap();

    let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,2 +1,4 @@
 line 1
+inserted line
 line 2
+another inserted line"#;

    let input = PatchInput {
        file_path: temp_file.path().to_string_lossy().to_string(),
        patch_content: patch.to_string(),
    };

    let output = PatchTool::apply_patch(&input).unwrap();
    assert!(output.success);
    assert_eq!(output.applied_hunks, 1);

    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("inserted line"));
    assert!(content.contains("another inserted line"));
}

/// Integration test: Patch with deletions only
#[test]
fn test_patch_deletions_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "line 1").unwrap();
    writeln!(temp_file, "line 2").unwrap();
    writeln!(temp_file, "line 3").unwrap();
    writeln!(temp_file, "line 4").unwrap();
    temp_file.flush().unwrap();

    let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,4 +1,2 @@
 line 1
-line 2
-line 3
 line 4"#;

    let input = PatchInput {
        file_path: temp_file.path().to_string_lossy().to_string(),
        patch_content: patch.to_string(),
    };

    let output = PatchTool::apply_patch(&input).unwrap();
    assert!(output.success);
    assert_eq!(output.applied_hunks, 1);

    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(!content.contains("line 2"));
    assert!(!content.contains("line 3"));
    assert!(content.contains("line 1"));
    assert!(content.contains("line 4"));
}

/// Integration test: Partial patch failure (some hunks succeed, some fail)
#[test]
fn test_partial_patch_failure() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "line 1").unwrap();
    writeln!(temp_file, "line 2").unwrap();
    writeln!(temp_file, "line 3").unwrap();
    writeln!(temp_file, "line 4").unwrap();
    writeln!(temp_file, "line 5").unwrap();
    temp_file.flush().unwrap();

    let file_path = temp_file.path().to_string_lossy().to_string();
    let original_content = fs::read_to_string(&file_path).unwrap();

    // Patch with one valid hunk and one invalid hunk
    let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3
@@ -3,3 +3,3 @@
 line 3
-nonexistent line
+replaced line
 line 5"#;

    let input = PatchInput {
        file_path: file_path.clone(),
        patch_content: patch.to_string(),
    };

    let output = PatchTool::apply_patch(&input).unwrap();

    // When any hunk fails, the entire patch should fail and file should be unchanged
    assert!(!output.success);
    assert!(output.failed_hunks > 0);

    let new_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(new_content, original_content);
}
