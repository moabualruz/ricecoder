//! Property-based tests for conflict detection
//!
//! **Feature: ricecoder-generation, Property 4: Conflict Detection**
//! **Validates: Requirements 1.5, 4.1**
//!
//! Property: For any generated file that would overwrite an existing file, the system SHALL detect the conflict and compute a diff before writing.

use proptest::prelude::*;
use ricecoder_generation::{ConflictDetector, GeneratedFile};
use std::fs;
use tempfile::TempDir;

/// Strategy for generating file paths
fn file_path_strategy() -> impl Strategy<Value = String> {
    r"[a-z]{1,10}\.rs|[a-z]{1,10}\.ts|[a-z]{1,10}\.py"
        .prop_map(|s| s.to_string())
}

/// Strategy for generating file content
fn file_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9\n ]{10,100}"
        .prop_map(|s| s.to_string())
}

/// Strategy for generating generated files
fn generated_file_strategy() -> impl Strategy<Value = GeneratedFile> {
    (file_path_strategy(), file_content_strategy(), Just("rust"))
        .prop_map(|(path, content, language)| GeneratedFile {
            path,
            content,
            language: language.to_string(),
        })
}

proptest! {
    /// Property: No conflicts detected for new files
    ///
    /// For any generated file that doesn't exist in the target directory,
    /// the ConflictDetector should report no conflicts.
    #[test]
    fn prop_no_conflicts_for_new_files(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let detector = ConflictDetector::new();

        let conflicts = detector.detect(&[file.clone()], temp_dir.path())
            .expect("Conflict detection failed");

        prop_assert!(
            conflicts.is_empty(),
            "Conflicts detected for new file: {}",
            file.path
        );
    }

    /// Property: Conflicts detected for existing files
    ///
    /// For any generated file that already exists in the target directory,
    /// the ConflictDetector should report a conflict.
    #[test]
    fn prop_conflicts_detected_for_existing_files(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }

        // Create existing file with different content
        let existing_content = "existing content";
        fs::write(&file_path, existing_content).expect("Failed to write existing file");

        let detector = ConflictDetector::new();
        let conflicts = detector.detect(&[file.clone()], temp_dir.path())
            .expect("Conflict detection failed");

        prop_assert!(
            !conflicts.is_empty(),
            "No conflicts detected for existing file: {}",
            file.path
        );

        // Verify conflict details
        let conflict = &conflicts[0];
        prop_assert_eq!(&conflict.old_content, existing_content, "Old content mismatch");
        prop_assert_eq!(&conflict.new_content, &file.content, "New content mismatch");
    }

    /// Property: Diff is computed for conflicts
    ///
    /// For any conflict, the ConflictDetector should compute a diff
    /// between old and new content.
    #[test]
    fn prop_diff_computed_for_conflicts(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }

        // Create existing file
        let existing_content = "line 1\nline 2\nline 3";
        fs::write(&file_path, existing_content).expect("Failed to write existing file");

        let detector = ConflictDetector::new();
        let conflicts = detector.detect(&[file.clone()], temp_dir.path())
            .expect("Conflict detection failed");

        if !conflicts.is_empty() {
            let conflict = &conflicts[0];
            let diff = &conflict.diff;

            // Diff should have total_changes > 0 if content is different
            if existing_content != file.content {
                prop_assert!(
                    diff.total_changes > 0,
                    "Diff has no changes for different content"
                );
            }
        }
    }

    /// Property: Conflict info includes file path
    ///
    /// For any conflict, the ConflictDetector should include the file path
    /// in the conflict information.
    #[test]
    fn prop_conflict_info_includes_path(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }

        // Create existing file
        fs::write(&file_path, "existing").expect("Failed to write existing file");

        let detector = ConflictDetector::new();
        let conflicts = detector.detect(&[file.clone()], temp_dir.path())
            .expect("Conflict detection failed");

        if !conflicts.is_empty() {
            let conflict = &conflicts[0];
            prop_assert_eq!(
                &conflict.path, &file_path,
                "Conflict path doesn't match"
            );
        }
    }

    /// Property: Conflict info includes old and new content
    ///
    /// For any conflict, the ConflictDetector should include both
    /// old and new content in the conflict information.
    #[test]
    fn prop_conflict_info_includes_content(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }

        // Create existing file
        let existing_content = "existing content";
        fs::write(&file_path, existing_content).expect("Failed to write existing file");

        let detector = ConflictDetector::new();
        let conflicts = detector.detect(&[file.clone()], temp_dir.path())
            .expect("Conflict detection failed");

        if !conflicts.is_empty() {
            let conflict = &conflicts[0];
            prop_assert_eq!(
                &conflict.old_content, existing_content,
                "Old content not preserved"
            );
            prop_assert_eq!(
                &conflict.new_content, &file.content,
                "New content not preserved"
            );
        }
    }

    /// Property: Multiple files are checked independently
    ///
    /// For any collection of generated files, the ConflictDetector should
    /// check each file independently and report all conflicts.
    #[test]
    fn prop_multiple_files_checked_independently(
        files in prop::collection::vec(generated_file_strategy(), 1..3)
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create some files
        for (idx, file) in files.iter().enumerate() {
            if idx % 2 == 0 {
                let file_path = temp_dir.path().join(&file.path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).expect("Failed to create parent dir");
                }
                fs::write(&file_path, "existing").expect("Failed to write existing file");
            }
        }

        let detector = ConflictDetector::new();
        let conflicts = detector.detect(&files, temp_dir.path())
            .expect("Conflict detection failed");

        // Count expected conflicts
        let expected_conflicts = files.iter().enumerate()
            .filter(|(idx, _)| idx % 2 == 0)
            .count();

        prop_assert_eq!(
            conflicts.len(), expected_conflicts,
            "Conflict count doesn't match expected"
        );
    }

    /// Property: Empty file list produces no conflicts
    ///
    /// For an empty list of files, the ConflictDetector should report no conflicts.
    #[test]
    fn prop_empty_file_list_produces_no_conflicts(_unit in Just(())) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let detector = ConflictDetector::new();

        let conflicts = detector.detect(&[], temp_dir.path())
            .expect("Conflict detection failed");

        prop_assert!(
            conflicts.is_empty(),
            "Conflicts detected for empty file list"
        );
    }

    /// Property: Conflict detection is deterministic
    ///
    /// For any set of files, detecting conflicts twice should produce
    /// the same result.
    #[test]
    fn prop_conflict_detection_is_deterministic(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }

        // Create existing file
        fs::write(&file_path, "existing").expect("Failed to write existing file");

        let detector = ConflictDetector::new();
        let conflicts1 = detector.detect(&[file.clone()], temp_dir.path())
            .expect("Conflict detection failed");
        let conflicts2 = detector.detect(&[file.clone()], temp_dir.path())
            .expect("Conflict detection failed");

        prop_assert_eq!(
            conflicts1.len(), conflicts2.len(),
            "Conflict count differs between detections"
        );

        for (c1, c2) in conflicts1.iter().zip(conflicts2.iter()) {
            prop_assert_eq!(
                &c1.old_content, &c2.old_content,
                "Old content differs between detections"
            );
            prop_assert_eq!(
                &c1.new_content, &c2.new_content,
                "New content differs between detections"
            );
        }
    }

    /// Property: Identical content produces no diff
    ///
    /// For any file where old and new content are identical,
    /// the diff should have no changes.
    #[test]
    fn prop_identical_content_produces_no_diff(content in r"[a-zA-Z0-9\n ]{10,100}") {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.rs");

        // Create existing file with same content
        fs::write(&file_path, &content).expect("Failed to write existing file");

        let file = GeneratedFile {
            path: "test.rs".to_string(),
            content: content.clone(),
            language: "rust".to_string(),
        };

        let detector = ConflictDetector::new();
        let conflicts = detector.detect(&[file], temp_dir.path())
            .expect("Conflict detection failed");

        if !conflicts.is_empty() {
            let conflict = &conflicts[0];
            prop_assert_eq!(
                conflict.diff.total_changes, 0,
                "Diff has changes for identical content"
            );
        }
    }
}
