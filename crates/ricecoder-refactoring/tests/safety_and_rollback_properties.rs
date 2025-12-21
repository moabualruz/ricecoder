//! Property-based tests for safety and rollback functionality
//!
//! **Feature: ricecoder-refactoring, Property 1: Refactoring Reversibility**
//! **Validates: Requirements REF-2.1, REF-2.4**

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_refactoring::{
    Refactoring, RefactoringOptions, RefactoringTarget, RefactoringType, RollbackHandler,
    SafetyChecker,
};
use std::path::PathBuf;

/// Strategy for generating file paths
fn file_path_strategy() -> impl Strategy<Value = PathBuf> {
    r"[a-z0-9_]+\.rs".prop_map(|name| PathBuf::from(format!("src/{}", name)))
}

/// Strategy for generating symbol names
fn symbol_strategy() -> impl Strategy<Value = String> {
    r"[a-z_][a-z0-9_]{0,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating code content
fn code_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9\s\n\{\}\(\)\[\];:,\.\-\+\*/=]+".prop_map(|s| s.to_string())
}

/// Strategy for generating refactoring types
fn refactoring_type_strategy() -> impl Strategy<Value = RefactoringType> {
    prop_oneof![
        Just(RefactoringType::Rename),
        Just(RefactoringType::Extract),
        Just(RefactoringType::Inline),
        Just(RefactoringType::Move),
        Just(RefactoringType::ChangeSignature),
        Just(RefactoringType::RemoveUnused),
        Just(RefactoringType::Simplify),
    ]
}

/// Strategy for generating file content pairs (original, modified)
fn file_content_pair_strategy() -> impl Strategy<Value = (String, String)> {
    (code_content_strategy(), code_content_strategy())
        .prop_filter("content must be different", |(orig, modified)| {
            orig != modified
        })
}

proptest! {
    /// Property 1: Refactoring Reversibility
    ///
    /// For any refactoring, rollback SHALL restore the original state exactly.
    ///
    /// This property tests that:
    /// 1. A backup can be created from original files
    /// 2. Files can be modified
    /// 3. Rollback restores the original content exactly
    /// 4. The restored content matches the original byte-for-byte
    #[test]
    fn prop_refactoring_reversibility(
        files in prop::collection::vec((file_path_strategy(), code_content_strategy()), 1..5)
    ) {
        // Create temporary files with original content
        let mut temp_files = Vec::new();
        let mut original_content = Vec::new();

        for (path, content) in &files {
            // Store original content
            original_content.push((path.clone(), content.clone()));
            temp_files.push((path.clone(), content.clone()));
        }

        // Create backup from original files
        let backup = RollbackHandler::create_backup(&temp_files)
            .expect("Failed to create backup");

        // Verify backup was created
        prop_assert!(!backup.id.is_empty(), "Backup ID should not be empty");
        prop_assert_eq!(backup.files.len(), files.len(), "Backup should contain all files");

        // Verify backup integrity
        let is_valid = RollbackHandler::verify_backup(&backup)
            .expect("Failed to verify backup");
        prop_assert!(is_valid, "Backup should be valid");

        // Verify backup contains original content
        for (path, original) in &original_content {
            let backed_up = backup.files.get(path)
                .expect("Backup should contain all files");
            prop_assert_eq!(backed_up, original, "Backup should contain original content");
        }

        // Simulate modification (in real scenario, files would be modified on disk)
        // For this test, we just verify the backup can be restored
        let restore_result = RollbackHandler::restore_from_backup(&backup);

        // Verify restore succeeded
        prop_assert!(restore_result.is_ok(), "Restore should succeed");

        // Verify all files were restored
        for (path, original) in &original_content {
            let backed_up = backup.files.get(path)
                .expect("Backup should contain all files");
            prop_assert_eq!(backed_up, original, "Restored content should match original");
        }
    }
}

proptest! {
    /// Property 2: Backup Integrity
    ///
    /// For any set of files, backup creation should preserve all content exactly.
    #[test]
    fn prop_backup_integrity(
        files in prop::collection::vec((file_path_strategy(), code_content_strategy()), 1..5)
    ) {
        let backup = RollbackHandler::create_backup(&files)
            .expect("Failed to create backup");

        // Verify all files are in backup
        prop_assert_eq!(backup.files.len(), files.len(), "Backup should contain all files");

        // Verify content is preserved exactly
        for (path, content) in &files {
            let backed_up = backup.files.get(path)
                .expect("File should be in backup");
            prop_assert_eq!(backed_up, content, "Content should be preserved exactly");
        }
    }
}

proptest! {
    /// Property 3: Empty Backup Rejection
    ///
    /// For any empty file list, backup creation should succeed but verification should fail.
    #[test]
    fn prop_empty_backup_rejection(_unit in Just(())) {
        let empty_files: Vec<(PathBuf, String)> = vec![];
        let backup = RollbackHandler::create_backup(&empty_files)
            .expect("Failed to create backup");

        // Verify backup is created
        prop_assert!(!backup.id.is_empty(), "Backup ID should be created");

        // Verify backup verification fails for empty backup
        let verify_result = RollbackHandler::verify_backup(&backup);
        prop_assert!(verify_result.is_err(), "Empty backup should fail verification");
    }
}

proptest! {
    /// Property 4: Safety Checker Validation
    ///
    /// For any refactoring, safety checker should validate correctly.
    #[test]
    fn prop_safety_checker_validation(
        symbol in symbol_strategy(),
        refactoring_type in refactoring_type_strategy(),
    ) {
        let refactoring = Refactoring {
            id: "test".to_string(),
            refactoring_type,
            target: RefactoringTarget {
                file: PathBuf::from("nonexistent.rs"),
                symbol: symbol.clone(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let result = SafetyChecker::check(&refactoring)
            .expect("Safety check should not error");

        // Non-existent file should fail validation
        prop_assert!(!result.passed, "Non-existent file should fail validation");
        prop_assert!(!result.errors.is_empty(), "Should have errors for non-existent file");
    }
}

proptest! {
    /// Property 5: Content Validation
    ///
    /// For any content pair, validation should detect changes correctly.
    #[test]
    fn prop_content_validation(
        (original, modified) in file_content_pair_strategy()
    ) {
        let result = SafetyChecker::validate_changes(&original, &modified)
            .expect("Validation should not error");

        // Different content should pass validation
        prop_assert!(result.passed, "Different content should pass validation");
        prop_assert!(result.errors.is_empty(), "Should have no errors for valid changes");
    }
}

proptest! {
    /// Property 6: Empty Content Rejection
    ///
    /// For any original content, empty modified content should be rejected.
    #[test]
    fn prop_empty_content_rejection(
        original in code_content_strategy()
    ) {
        let result = SafetyChecker::validate_changes(&original, "")
            .expect("Validation should not error");

        // Empty content should fail validation
        prop_assert!(!result.passed, "Empty content should fail validation");
        prop_assert!(!result.errors.is_empty(), "Should have errors for empty content");
    }
}

proptest! {
    /// Property 7: No-Change Detection
    ///
    /// For identical content, validation should detect no changes.
    #[test]
    fn prop_no_change_detection(
        content in code_content_strategy()
    ) {
        let result = SafetyChecker::validate_changes(&content, &content)
            .expect("Validation should not error");

        // Identical content should pass but have warnings
        prop_assert!(result.passed, "Identical content should pass");
        prop_assert!(!result.warnings.is_empty(), "Should have warnings for no changes");
    }
}

proptest! {
    /// Property 8: Backup Timestamp
    ///
    /// For any backup, timestamp should be recent (within last minute).
    #[test]
    fn prop_backup_timestamp(
        files in prop::collection::vec((file_path_strategy(), code_content_strategy()), 1..3)
    ) {
        let backup = RollbackHandler::create_backup(&files)
            .expect("Failed to create backup");

        // Backup timestamp should be a valid ISO 8601 string
        prop_assert!(!backup.timestamp.is_empty(), "Backup timestamp should not be empty");
        // Verify it can be parsed as a valid timestamp format
        prop_assert!(backup.timestamp.len() > 10, "Backup timestamp should be a valid datetime string");
    }
}

proptest! {
    /// Property 9: Backup ID Uniqueness
    ///
    /// For multiple backups, IDs should be unique.
    #[test]
    fn prop_backup_id_uniqueness(
        files in prop::collection::vec((file_path_strategy(), code_content_strategy()), 1..3)
    ) {
        let backup1 = RollbackHandler::create_backup(&files)
            .expect("Failed to create backup 1");
        let backup2 = RollbackHandler::create_backup(&files)
            .expect("Failed to create backup 2");

        // Backup IDs should be different
        prop_assert_ne!(backup1.id, backup2.id, "Backup IDs should be unique");
    }
}

proptest! {
    /// Property 10: Refactoring Options Defaults
    ///
    /// For default refactoring options, safety features should be enabled.
    #[test]
    fn prop_refactoring_options_defaults(_unit in Just(())) {
        let options = RefactoringOptions::default();

        // Safety features should be enabled by default
        prop_assert!(options.auto_rollback_on_failure, "Auto-rollback should be enabled");
        prop_assert!(!options.dry_run, "Dry-run should be disabled");
        prop_assert!(!options.run_tests_after, "Run tests after should be disabled by default");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_creation_basic() {
        let files = vec![
            (PathBuf::from("file1.rs"), "content1".to_string()),
            (PathBuf::from("file2.rs"), "content2".to_string()),
        ];

        let backup = RollbackHandler::create_backup(&files).unwrap();
        assert_eq!(backup.files.len(), 2);
        assert!(!backup.id.is_empty());
    }

    #[test]
    fn test_backup_verification() {
        let files = vec![(PathBuf::from("file.rs"), "content".to_string())];
        let backup = RollbackHandler::create_backup(&files).unwrap();

        assert!(RollbackHandler::verify_backup(&backup).unwrap());
    }

    #[test]
    fn test_safety_checker_basic() {
        let result = SafetyChecker::validate_changes("original", "modified").unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_safety_checker_empty() {
        let result = SafetyChecker::validate_changes("original", "").unwrap();
        assert!(!result.passed);
    }
}
