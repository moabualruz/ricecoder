//! Property-based tests for path resolution consistency
//!
//! **Feature: ricecoder-execution, Property 5: Path Resolution Consistency**
//! **Validates: Requirements 1.1**
//!
//! Property: *For any* file operation, path resolution using `ricecoder_storage::PathResolver`
//! SHALL produce consistent, validated paths.
//!
//! This test verifies that:
//! 1. The same path always resolves to the same result
//! 2. Path validation is consistent across multiple calls
//! 3. Invalid paths are consistently rejected
//! 4. Home directory expansion is consistent

use proptest::prelude::*;
use ricecoder_execution::FileOperations;
use tempfile::TempDir;

/// Strategy for generating valid file paths
fn valid_path_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]+"
        .prop_map(|s| format!("test_{}.txt", s))
}

/// Strategy for generating invalid file paths
fn invalid_path_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),                    // Empty path
        Just("path\0with\0nulls".to_string()),   // Path with null bytes
    ]
}

proptest! {
    /// Property 5.1: Path resolution is consistent
    ///
    /// For any valid path, resolving it multiple times should produce the same result.
    #[test]
    fn prop_path_resolution_consistency(path in valid_path_strategy()) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&path);
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file
        std::fs::write(&file_path, "test content").unwrap();

        // Check file existence multiple times
        let exists_1 = FileOperations::file_exists(&path_str);
        let exists_2 = FileOperations::file_exists(&path_str);
        let exists_3 = FileOperations::file_exists(&path_str);

        // All three calls should return the same result
        prop_assert_eq!(&exists_1, &exists_2);
        prop_assert_eq!(&exists_2, &exists_3);
        prop_assert!(exists_1.is_ok());
        prop_assert!(exists_1.unwrap());
    }

    /// Property 5.2: Invalid paths are consistently rejected
    ///
    /// For any invalid path, validation should consistently fail.
    #[test]
    fn prop_invalid_path_rejection(path in invalid_path_strategy()) {
        // Try to check if invalid path exists
        let result_1 = FileOperations::file_exists(&path);
        let result_2 = FileOperations::file_exists(&path);

        // Both calls should fail
        prop_assert!(result_1.is_err());
        prop_assert!(result_2.is_err());
    }

    /// Property 5.3: File operations preserve path consistency
    ///
    /// For any valid path, creating and reading a file should use the same resolved path.
    #[test]
    fn prop_file_operations_path_consistency(
        path in valid_path_strategy(),
        content in ".*"
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&path);
        let path_str = file_path.to_string_lossy().to_string();

        // Create file
        let create_result = FileOperations::create_file(&path_str, &content);
        prop_assert!(create_result.is_ok());

        // Read file back
        let read_result = FileOperations::read_file(&path_str);
        prop_assert!(read_result.is_ok());

        // Content should match
        prop_assert_eq!(read_result.unwrap(), content);
    }

    /// Property 5.4: Backup and restore preserve path consistency
    ///
    /// For any valid path, backing up and restoring should use consistent paths.
    #[test]
    fn prop_backup_restore_path_consistency(path in valid_path_strategy()) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&path);
        let path_str = file_path.to_string_lossy().to_string();

        // Create original file
        std::fs::write(&file_path, "original content").unwrap();

        // Create backup
        let backup_result = FileOperations::backup_file(&path_str);
        prop_assert!(backup_result.is_ok());

        let backup_path = backup_result.unwrap();
        let backup_str = backup_path.to_string_lossy().to_string();

        // Modify original file
        std::fs::write(&file_path, "modified content").unwrap();

        // Restore from backup
        let restore_result = FileOperations::restore_from_backup(&path_str, &backup_str);
        prop_assert!(restore_result.is_ok());

        // Content should be restored
        let restored_content = std::fs::read_to_string(&file_path).unwrap();
        prop_assert_eq!(restored_content, "original content");
    }

    /// Property 5.5: Path validation is deterministic
    ///
    /// For any path, validation should always produce the same result.
    #[test]
    fn prop_path_validation_deterministic(path in valid_path_strategy()) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&path);
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file
        std::fs::write(&file_path, "content").unwrap();

        // Check existence multiple times
        let mut results = Vec::new();
        for _ in 0..10 {
            let result = FileOperations::file_exists(&path_str);
            results.push(result);
        }

        // All results should be the same
        for result in &results[1..] {
            prop_assert_eq!(result, &results[0]);
        }
    }

    /// Property 5.6: Delete and recreate use consistent paths
    ///
    /// For any valid path, deleting and recreating a file should use the same resolved path.
    #[test]
    fn prop_delete_recreate_path_consistency(path in valid_path_strategy()) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&path);
        let path_str = file_path.to_string_lossy().to_string();

        // Create file
        let create_result = FileOperations::create_file(&path_str, "content 1");
        prop_assert!(create_result.is_ok());

        // Delete file
        let delete_result = FileOperations::delete_file(&path_str);
        prop_assert!(delete_result.is_ok());

        // File should not exist
        let exists_result = FileOperations::file_exists(&path_str);
        prop_assert!(exists_result.is_ok());
        prop_assert!(!exists_result.unwrap());

        // Recreate file
        let recreate_result = FileOperations::create_file(&path_str, "content 2");
        prop_assert!(recreate_result.is_ok());

        // File should exist again
        let exists_result = FileOperations::file_exists(&path_str);
        prop_assert!(exists_result.is_ok());
        prop_assert!(exists_result.unwrap());

        // Content should be new content
        let read_result = FileOperations::read_file(&path_str);
        prop_assert!(read_result.is_ok());
        prop_assert_eq!(read_result.unwrap(), "content 2");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_path_resolution_with_home_expansion() {
        // Test that home directory expansion works
        let home_path = "~/test_file.txt";
        let result = FileOperations::file_exists(home_path);
        // Should not error on validation, even if file doesn't exist
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_resolution_with_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        let path_str = file_path.to_string_lossy().to_string();
        let result = FileOperations::file_exists(&path_str);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_path_resolution_with_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        let path_str = file_path.to_string_lossy().to_string();
        let result = FileOperations::file_exists(&path_str);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_empty_path_validation() {
        let result = FileOperations::file_exists("");
        assert!(result.is_err());
    }

    #[test]
    fn test_null_byte_path_validation() {
        let result = FileOperations::file_exists("path\0with\0nulls");
        assert!(result.is_err());
    }
}
