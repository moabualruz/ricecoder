//! Property-based tests for dry-run safety
//!
//! **Feature: ricecoder-generation, Property 3: Dry-Run Safety**
//! **Validates: Requirements 3.1**
//!
//! Property: For any dry-run generation, no files SHALL be written or modified.

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
    /// Property: Dry-run doesn't write new files
    ///
    /// For any generated file in dry-run mode, the file should not be written to disk.
    #[test]
    fn prop_dry_run_doesnt_write_new_files(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        // Ensure file doesn't exist
        if file_path.exists() {
            fs::remove_file(&file_path).expect("Failed to remove file");
        }

        let config = OutputWriterConfig {
            dry_run: true,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Skip,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file.clone()], temp_dir.path(), &[])
            .expect("Write failed");

        // File should not be written in dry-run mode
        prop_assert!(
            !file_path.exists(),
            "File was written in dry-run mode: {}",
            file.path
        );

        // Result should indicate dry-run
        prop_assert!(
            result.dry_run,
            "Result doesn't indicate dry-run"
        );

        // No files should be marked as written
        prop_assert_eq!(
            result.files_written, 0,
            "Files marked as written in dry-run mode"
        );
    }

    /// Property: Dry-run doesn't modify existing files
    ///
    /// For any existing file in dry-run mode, the file should not be modified.
    #[test]
    fn prop_dry_run_doesnt_modify_existing_files(
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
            dry_run: true,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // File should not be modified in dry-run mode
        let current_content = fs::read_to_string(&file_path).expect("Failed to read file");
        prop_assert_eq!(
            current_content, original_content,
            "File was modified in dry-run mode"
        );

        // Result should indicate dry-run
        prop_assert!(
            result.dry_run,
            "Result doesn't indicate dry-run"
        );
    }

    /// Property: Dry-run doesn't create backups
    ///
    /// For any file in dry-run mode, no backups should be created.
    #[test]
    fn prop_dry_run_doesnt_create_backups(
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
            content: new_content,
            language: "rust".to_string(),
        };

        let config = OutputWriterConfig {
            dry_run: true,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Overwrite,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // No backups should be created in dry-run mode
        prop_assert_eq!(
            result.backups_created, 0,
            "Backups created in dry-run mode"
        );
    }

    /// Property: Dry-run mode is idempotent
    ///
    /// For any file in dry-run mode, running dry-run multiple times should
    /// produce the same result (no changes).
    #[test]
    fn prop_dry_run_is_idempotent(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);

        let config = OutputWriterConfig {
            dry_run: true,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Skip,
        };
        let writer = OutputWriter::with_config(config);

        // Run dry-run twice
        let result1 = writer.write(&[file.clone()], temp_dir.path(), &[])
            .expect("Write failed");
        let result2 = writer.write(&[file.clone()], temp_dir.path(), &[])
            .expect("Write failed");

        // Both should indicate dry-run
        prop_assert!(result1.dry_run, "First run doesn't indicate dry-run");
        prop_assert!(result2.dry_run, "Second run doesn't indicate dry-run");

        // Both should have same results
        prop_assert_eq!(
            result1.files_written, result2.files_written,
            "Files written differs between runs"
        );
        prop_assert_eq!(
            result1.backups_created, result2.backups_created,
            "Backups created differs between runs"
        );

        // File should not exist
        prop_assert!(
            !file_path.exists(),
            "File exists after dry-run"
        );
    }

    /// Property: Dry-run with multiple files doesn't write any
    ///
    /// For any collection of files in dry-run mode, no files should be written.
    #[test]
    fn prop_dry_run_with_multiple_files_doesnt_write(
        files in prop::collection::vec(generated_file_strategy(), 1..3)
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let config = OutputWriterConfig {
            dry_run: true,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Skip,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&files, temp_dir.path(), &[])
            .expect("Write failed");

        // No files should be written in dry-run mode
        prop_assert_eq!(
            result.files_written, 0,
            "Files written in dry-run mode"
        );

        // Verify no files were actually created
        for file in &files {
            let file_path = temp_dir.path().join(&file.path);
            prop_assert!(
                !file_path.exists(),
                "File was created in dry-run mode: {}",
                file.path
            );
        }
    }

    /// Property: Dry-run result indicates dry-run status
    ///
    /// For any file in dry-run mode, the result should clearly indicate it was a dry-run.
    #[test]
    fn prop_dry_run_result_indicates_status(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let config = OutputWriterConfig {
            dry_run: true,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Skip,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // Result should indicate dry-run
        prop_assert!(
            result.dry_run,
            "Result doesn't indicate dry-run"
        );

        // All file results should indicate dry-run
        for file_result in &result.files {
            prop_assert!(
                file_result.dry_run,
                "File result doesn't indicate dry-run"
            );
        }
    }

    /// Property: Non-dry-run mode writes files
    ///
    /// For any file in non-dry-run mode, the file should be written to disk.
    #[test]
    fn prop_non_dry_run_writes_files(file in generated_file_strategy()) {
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

        // File should be written in non-dry-run mode
        prop_assert!(
            file_path.exists(),
            "File was not written in non-dry-run mode: {}",
            file.path
        );

        // Result should indicate it's not a dry-run
        prop_assert!(
            !result.dry_run,
            "Result indicates dry-run when it shouldn't"
        );

        // At least one file should be marked as written
        prop_assert!(
            result.files_written > 0,
            "No files marked as written in non-dry-run mode"
        );
    }

    /// Property: Dry-run doesn't create parent directories
    ///
    /// For any file in dry-run mode, parent directories should not be created.
    #[test]
    fn prop_dry_run_doesnt_create_directories(file in generated_file_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(&file.path);
        let parent_dir = file_path.parent().expect("No parent directory");

        // Ensure parent doesn't exist
        if parent_dir.exists() {
            fs::remove_dir_all(parent_dir).expect("Failed to remove directory");
        }

        let config = OutputWriterConfig {
            dry_run: true,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Skip,
        };
        let writer = OutputWriter::with_config(config);

        let result = writer.write(&[file], temp_dir.path(), &[])
            .expect("Write failed");

        // Parent directory should not be created in dry-run mode
        prop_assert!(
            !parent_dir.exists(),
            "Parent directory was created in dry-run mode"
        );

        // Result should indicate dry-run
        prop_assert!(
            result.dry_run,
            "Result doesn't indicate dry-run"
        );
    }
}
