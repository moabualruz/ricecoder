//! Property-based tests for conflict resolution strategies
//!
//! **Feature: ricecoder-generation, Property 5: Conflict Resolution Strategies**
//! **Validates: Requirements 4.2, 4.3, 4.4, 4.5**
//!
//! Property: For any detected conflict, the system SHALL apply the selected strategy (skip, overwrite, merge) correctly and report the action taken.

use proptest::prelude::*;
use ricecoder_generation::conflict_detector::{DiffLine, FileConflictInfo, FileDiff};
use ricecoder_generation::{ConflictResolver, ConflictStrategy};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Strategy for generating file paths
fn file_path_strategy() -> impl Strategy<Value = String> {
    r"[a-z]{1,10}\.rs".prop_map(|s| s.to_string())
}

/// Strategy for generating file content
fn file_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9\n ]{10,50}".prop_map(|s| s.to_string())
}

/// Strategy for generating conflict info
fn conflict_info_strategy() -> impl Strategy<Value = (String, String, String)> {
    (
        file_path_strategy(),
        file_content_strategy(),
        file_content_strategy(),
    )
        .prop_map(|(path, old_content, new_content)| (path, old_content, new_content))
}

proptest! {
    /// Property: Skip strategy doesn't write files
    ///
    /// For any conflict, applying the skip strategy should not write the file.
    #[test]
    fn prop_skip_strategy_doesnt_write(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();
        let result = resolver.resolve(&conflict, ConflictStrategy::Skip, &new_content)
            .expect("Resolution failed");

        // Skip strategy should not write
        prop_assert!(!result.written, "Skip strategy wrote file");

        // File should still have old content
        let current_content = fs::read_to_string(&file_path).expect("Failed to read file");
        prop_assert_eq!(
            current_content, old_content,
            "File content changed with skip strategy"
        );
    }

    /// Property: Overwrite strategy writes files
    ///
    /// For any conflict, applying the overwrite strategy should write the new content.
    #[test]
    fn prop_overwrite_strategy_writes(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();
        let result = resolver.resolve(&conflict, ConflictStrategy::Overwrite, &new_content)
            .expect("Resolution failed");

        // Overwrite strategy should write
        prop_assert!(result.written, "Overwrite strategy didn't write file");

        // File should have new content
        let current_content = fs::read_to_string(&file_path).expect("Failed to read file");
        prop_assert_eq!(
            current_content, new_content,
            "File content not updated with overwrite strategy"
        );
    }

    /// Property: Overwrite strategy creates backup
    ///
    /// For any conflict, applying the overwrite strategy should create a backup.
    #[test]
    fn prop_overwrite_strategy_creates_backup(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();
        let result = resolver.resolve(&conflict, ConflictStrategy::Overwrite, &new_content)
            .expect("Resolution failed");

        // Backup should be created
        prop_assert!(
            result.backup_path.is_some(),
            "Overwrite strategy didn't create backup"
        );

        // Backup file should exist and contain old content
        if let Some(backup_path) = &result.backup_path {
            let backup_file = PathBuf::from(backup_path);
            prop_assert!(
                backup_file.exists(),
                "Backup file doesn't exist: {}",
                backup_path
            );

            let backup_content = fs::read_to_string(&backup_file)
                .expect("Failed to read backup file");
            prop_assert_eq!(
                backup_content, old_content,
                "Backup content doesn't match original"
            );
        }
    }

    /// Property: Merge strategy writes files
    ///
    /// For any conflict, applying the merge strategy should write merged content.
    #[test]
    fn prop_merge_strategy_writes(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();
        let result = resolver.resolve(&conflict, ConflictStrategy::Merge, &new_content)
            .expect("Resolution failed");

        // Merge strategy should write
        prop_assert!(result.written, "Merge strategy didn't write file");

        // File should exist and have content
        let current_content = fs::read_to_string(&file_path).expect("Failed to read file");
        prop_assert!(
            !current_content.is_empty(),
            "Merged file is empty"
        );
    }

    /// Property: Merge strategy creates backup
    ///
    /// For any conflict, applying the merge strategy should create a backup.
    #[test]
    fn prop_merge_strategy_creates_backup(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();
        let result = resolver.resolve(&conflict, ConflictStrategy::Merge, &new_content)
            .expect("Resolution failed");

        // Backup should be created
        prop_assert!(
            result.backup_path.is_some(),
            "Merge strategy didn't create backup"
        );
    }

    /// Property: Resolution result includes action description
    ///
    /// For any conflict resolution, the result should include a description of the action taken.
    #[test]
    fn prop_resolution_includes_action(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();

        // Test skip strategy
        let result = resolver.resolve(&conflict, ConflictStrategy::Skip, &new_content)
            .expect("Resolution failed");
        prop_assert!(
            !result.action.is_empty(),
            "Skip strategy action is empty"
        );

        // Test overwrite strategy
        let result = resolver.resolve(&conflict, ConflictStrategy::Overwrite, &new_content)
            .expect("Resolution failed");
        prop_assert!(
            !result.action.is_empty(),
            "Overwrite strategy action is empty"
        );

        // Test merge strategy
        let result = resolver.resolve(&conflict, ConflictStrategy::Merge, &new_content)
            .expect("Resolution failed");
        prop_assert!(
            !result.action.is_empty(),
            "Merge strategy action is empty"
        );
    }

    /// Property: Different strategies produce different results
    ///
    /// For any conflict, different strategies should produce different outcomes.
    #[test]
    fn prop_different_strategies_produce_different_results(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();

        // Skip strategy
        let skip_result = resolver.resolve(&conflict, ConflictStrategy::Skip, &new_content)
            .expect("Resolution failed");

        // Overwrite strategy
        let overwrite_result = resolver.resolve(&conflict, ConflictStrategy::Overwrite, &new_content)
            .expect("Resolution failed");

        // Skip should not write, overwrite should write
        prop_assert_ne!(
            skip_result.written, overwrite_result.written,
            "Skip and overwrite strategies produced same result"
        );
    }

    /// Property: Resolution is deterministic
    ///
    /// For any conflict, applying the same strategy twice should produce the same result.
    #[test]
    fn prop_resolution_is_deterministic(
        (path, old_content, new_content) in conflict_info_strategy()
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &old_content).expect("Failed to write existing file");

        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: old_content.clone(),
            new_content: new_content.clone(),
            diff: FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 1,
            },
        };

        let resolver = ConflictResolver::new();

        // Apply skip strategy twice
        let result1 = resolver.resolve(&conflict, ConflictStrategy::Skip, &new_content)
            .expect("Resolution failed");
        let result2 = resolver.resolve(&conflict, ConflictStrategy::Skip, &new_content)
            .expect("Resolution failed");

        prop_assert_eq!(
            result1.written, result2.written,
            "Skip strategy produced different results"
        );
    }
}
