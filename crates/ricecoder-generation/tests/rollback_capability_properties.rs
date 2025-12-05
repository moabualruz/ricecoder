//! Property-based tests for rollback capability
//!
//! **Feature: ricecoder-generation, Property 9: Rollback Capability**
//! **Validates: Requirements 3.5**
//!
//! Property: For any generation failure, the system SHALL support rollback to restore the previous state of all files.

use proptest::prelude::*;
use ricecoder_generation::conflict_resolver::ConflictStrategy;
use ricecoder_generation::{GeneratedFile, OutputWriter, OutputWriterConfig};
use std::fs;
use tempfile::TempDir;

/// Strategy for generating file paths
fn file_path_strategy() -> impl Strategy<Value = String> {
    r"[a-z]{1,10}\.rs".prop_map(|s| s.to_string())
}

/// Strategy for generating file content
fn file_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9\n ]{10,50}".prop_map(|s| s.to_string())
}

/// Strategy for generating generated files
fn generated_file_strategy() -> impl Strategy<Value = GeneratedFile> {
    (file_path_strategy(), file_content_strategy(), Just("rust")).prop_map(
        |(path, content, language)| GeneratedFile {
            path,
            content,
            language: language.to_string(),
        },
    )
}

proptest! {
    /// Property: Backups are created before writing
    ///
    /// For any existing file being overwritten, a backup should be created.
    #[test]
    fn prop_backups_created_before_writing(
        (path, original_content, new_content) in (
            file_path_strategy(),
            file_content_strategy(),
            file_content_strategy()
        )
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &original_content).expect("Failed to write existing file");

        let file = GeneratedFile {
            path,
            content: new_content.clone(),
            language: "rust".to_string(),
        };

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // Backup should be created
        prop_assert!(
            result.backups_created > 0,
            "No backups created for existing file"
        );

        // Backup file should exist
        if let Some(rollback_info) = &result.rollback_info {
            prop_assert!(
                !rollback_info.backups.is_empty(),
                "Rollback info has no backups"
            );

            for (_, backup_path) in &rollback_info.backups {
                prop_assert!(
                    backup_path.exists(),
                    "Backup file doesn't exist: {}",
                    backup_path.display()
                );
            }
        }
    }

    /// Property: Backup contains original content
    ///
    /// For any backup created, it should contain the original file content.
    #[test]
    fn prop_backup_contains_original_content(
        (path, original_content, new_content) in (
            file_path_strategy(),
            file_content_strategy(),
            file_content_strategy()
        )
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &original_content).expect("Failed to write existing file");

        let file = GeneratedFile {
            path,
            content: new_content.clone(),
            language: "rust".to_string(),
        };

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // Verify backup content
        if let Some(rollback_info) = &result.rollback_info {
            for (_, backup_path) in &rollback_info.backups {
                let backup_content = fs::read_to_string(backup_path)
                    .expect("Failed to read backup file");
                prop_assert_eq!(
                    &backup_content, &original_content,
                    "Backup content doesn't match original"
                );
            }
        }
    }

    /// Property: Rollback info includes written files
    ///
    /// For any successful write, the rollback info should include the files that were written.
    #[test]
    fn prop_rollback_info_includes_written_files(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        // Ensure file doesn't exist
        if file_path.exists() {
            fs::remove_file(&file_path).expect("Failed to remove file");
        }

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Skip,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file.clone()], temp_dir.path(), &[])
            .expect("Write failed");

        // If file was written, rollback info should include it
        if result.files_written > 0 {
            if let Some(rollback_info) = &result.rollback_info {
                prop_assert!(
                    !rollback_info.written_files.is_empty(),
                    "Rollback info has no written files"
                );
            }
        }
    }

    /// Property: Rollback info is present when backups are created
    ///
    /// For any write that creates backups, rollback info should be present.
    #[test]
    fn prop_rollback_info_present_with_backups(
        (path, original_content, new_content) in (
            file_path_strategy(),
            file_content_strategy(),
            file_content_strategy()
        )
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &original_content).expect("Failed to write existing file");

        let file = GeneratedFile {
            path,
            content: new_content.clone(),
            language: "rust".to_string(),
        };

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // If backups were created, rollback info should be present
        if result.backups_created > 0 {
            prop_assert!(
                result.rollback_info.is_some(),
                "Rollback info missing when backups created"
            );
        }
    }

    /// Property: Multiple files have independent rollback info
    ///
    /// For any collection of files, each should have independent rollback information.
    #[test]
    fn prop_multiple_files_have_independent_rollback(
        files in prop::collection::vec(generated_file_strategy(), 1..3)
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create some existing files
        for (idx, file) in files.iter().enumerate() {
            if idx % 2 == 0 {
                let file_path = temp_dir.path().join(&file.path);
                fs::write(&file_path, "existing").expect("Failed to write existing file");
            }
        }

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&files, temp_dir.path(), &[])
            .expect("Write failed");

        // Count expected backups
        let expected_backups = files.iter().enumerate()
            .filter(|(idx, _)| idx % 2 == 0)
            .count();

        // Verify backup count
        if expected_backups > 0 {
            prop_assert!(
                result.rollback_info.is_some(),
                "Rollback info missing"
            );

            if let Some(rollback_info) = &result.rollback_info {
                prop_assert_eq!(
                    rollback_info.backups.len(), expected_backups,
                    "Backup count doesn't match expected"
                );
            }
        }
    }

    /// Property: Backup files have .bak extension
    ///
    /// For any backup created, it should have a .bak extension.
    #[test]
    fn prop_backup_files_have_bak_extension(
        (path, original_content, new_content) in (
            file_path_strategy(),
            file_content_strategy(),
            file_content_strategy()
        )
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &original_content).expect("Failed to write existing file");

        let file = GeneratedFile {
            path,
            content: new_content.clone(),
            language: "rust".to_string(),
        };

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // Verify backup extension
        if let Some(rollback_info) = &result.rollback_info {
            for (_, backup_path) in &rollback_info.backups {
                let extension = backup_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");
                prop_assert_eq!(
                    extension, "bak",
                    "Backup file doesn't have .bak extension: {}",
                    backup_path.display()
                );
            }
        }
    }

    /// Property: Rollback info is deterministic
    ///
    /// For any file, creating rollback info twice should produce the same result.
    #[test]
    fn prop_rollback_info_is_deterministic(
        (path, original_content, new_content) in (
            file_path_strategy(),
            file_content_strategy(),
            file_content_strategy()
        )
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &original_content).expect("Failed to write existing file");

        let file = GeneratedFile {
            path,
            content: new_content.clone(),
            language: "rust".to_string(),
        };

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result1 = writer.write(&[file.clone()], temp_dir.path(), &[])
            .expect("Write failed");

        // Restore original for second write
        fs::write(&file_path, &original_content).expect("Failed to write existing file");

        let result2 = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // Both should have same backup count
        prop_assert_eq!(
            result1.backups_created, result2.backups_created,
            "Backup count differs between writes"
        );

        // Both should have rollback info if backups were created
        if result1.backups_created > 0 {
            prop_assert!(
                result1.rollback_info.is_some(),
                "First write missing rollback info"
            );
            prop_assert!(
                result2.rollback_info.is_some(),
                "Second write missing rollback info"
            );
        }
    }

    /// Property: Backup path is different from original path
    ///
    /// For any backup created, the backup path should be different from the original path.
    #[test]
    fn prop_backup_path_differs_from_original(
        (path, original_content, new_content) in (
            file_path_strategy(),
            file_content_strategy(),
            file_content_strategy()
        )
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&path);

        // Create existing file
        fs::write(&file_path, &original_content).expect("Failed to write existing file");

        let file = GeneratedFile {
            path,
            content: new_content.clone(),
            language: "rust".to_string(),
        };

        let config = OutputWriterConfig {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // Verify backup path differs from original
        if let Some(rollback_info) = &result.rollback_info {
            for (original, backup) in &rollback_info.backups {
                prop_assert_ne!(
                    original, backup,
                    "Backup path is same as original path"
                );
            }
        }
    }
}
