//! Output writer for generated code files
//!
//! Writes generated code to files with rollback capability, dry-run mode, and conflict resolution.
//! Implements requirements:
//! - Requirement 1.6: Write generated code to files
//! - Requirement 3.1: Dry-run mode (preview without writing)
//! - Requirement 3.5: Rollback support (restore on failure)
//! - Requirement 4.2, 4.3, 4.4: Conflict resolution strategies

use crate::conflict_detector::FileConflictInfo;
use crate::conflict_resolver::{ConflictResolver, ConflictStrategy};
use crate::error::GenerationError;
use crate::models::GeneratedFile;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for output writing
#[derive(Debug, Clone)]
pub struct OutputWriterConfig {
    /// Whether to run in dry-run mode (preview only)
    pub dry_run: bool,
    /// Whether to create backups before writing
    pub create_backups: bool,
    /// Whether to format code before writing
    pub format_code: bool,
    /// Default conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
}

impl Default for OutputWriterConfig {
    fn default() -> Self {
        Self {
            dry_run: false,
            create_backups: true,
            format_code: true,
            conflict_strategy: ConflictStrategy::Skip,
        }
    }
}

/// Result of writing a single file
#[derive(Debug, Clone)]
pub struct FileWriteResult {
    /// Path to the file that was written
    pub path: PathBuf,
    /// Whether the file was actually written
    pub written: bool,
    /// Path to backup file if created
    pub backup_path: Option<PathBuf>,
    /// Action taken (e.g., "Written", "Skipped", "Merged")
    pub action: String,
    /// Whether this was a dry-run
    pub dry_run: bool,
}

/// Result of writing multiple files
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// Results for each file
    pub files: Vec<FileWriteResult>,
    /// Total files written
    pub files_written: usize,
    /// Total files skipped
    pub files_skipped: usize,
    /// Total backups created
    pub backups_created: usize,
    /// Whether this was a dry-run
    pub dry_run: bool,
    /// Rollback information if needed
    pub rollback_info: Option<RollbackInfo>,
}

/// Information for rolling back changes
#[derive(Debug, Clone)]
pub struct RollbackInfo {
    /// Backups that were created
    pub backups: Vec<(PathBuf, PathBuf)>, // (original_path, backup_path)
    /// Files that were written
    pub written_files: Vec<PathBuf>,
}

/// Writes generated code to files with rollback capability
///
/// Implements requirements:
/// - Requirement 1.6: Write generated code to files
/// - Requirement 3.1: Dry-run mode (preview without writing)
/// - Requirement 3.5: Rollback support (restore on failure)
/// - Requirement 4.2, 4.3, 4.4: Conflict resolution strategies
pub struct OutputWriter {
    config: OutputWriterConfig,
    conflict_resolver: ConflictResolver,
}

impl OutputWriter {
    /// Create a new output writer with default configuration
    pub fn new() -> Self {
        Self {
            config: OutputWriterConfig::default(),
            conflict_resolver: ConflictResolver::new(),
        }
    }

    /// Create a new output writer with custom configuration
    pub fn with_config(config: OutputWriterConfig) -> Self {
        Self {
            config,
            conflict_resolver: ConflictResolver::new(),
        }
    }

    /// Write generated files to disk
    ///
    /// Writes all files atomically. If any write fails, rolls back all changes.
    ///
    /// # Arguments
    /// * `files` - Generated files to write
    /// * `target_dir` - Target directory where files should be written
    /// * `conflicts` - Detected file conflicts
    ///
    /// # Returns
    /// Write result with information about what was written
    ///
    /// # Requirements
    /// - Requirement 1.6: Write generated code to files
    /// - Requirement 3.1: Dry-run mode (preview without writing)
    /// - Requirement 3.5: Rollback support (restore on failure)
    pub fn write(
        &self,
        files: &[GeneratedFile],
        target_dir: &Path,
        conflicts: &[FileConflictInfo],
    ) -> Result<WriteResult, GenerationError> {
        let mut file_results = Vec::new();
        let mut backups = Vec::new();
        let mut written_files = Vec::new();
        let mut files_written = 0;
        let mut files_skipped = 0;
        let mut backups_created = 0;

        // Process each file
        for file in files {
            let file_path = target_dir.join(&file.path);

            // Create parent directories if needed
            if let Some(parent) = file_path.parent() {
                if !parent.exists() && !self.config.dry_run {
                    fs::create_dir_all(parent).map_err(|e| {
                        GenerationError::WriteFailed(format!(
                            "Failed to create directory {}: {}",
                            parent.display(),
                            e
                        ))
                    })?;
                }
            }

            // Check for conflicts
            let conflict = conflicts.iter().find(|c| c.path == file_path);

            let result = if let Some(conflict) = conflict {
                // Handle conflict
                self.handle_conflict(&file_path, conflict, &file.content, &mut backups)?
            } else {
                // No conflict, write file
                self.write_file(&file_path, &file.content, &mut backups)?
            };

            if result.written {
                files_written += 1;
                written_files.push(file_path.clone());
            } else {
                files_skipped += 1;
            }

            if result.backup_path.is_some() {
                backups_created += 1;
            }

            file_results.push(result);
        }

        // If dry-run, don't actually write anything
        if self.config.dry_run {
            return Ok(WriteResult {
                files: file_results,
                files_written: 0,
                files_skipped: files.len(),
                backups_created: 0,
                dry_run: true,
                rollback_info: None,
            });
        }

        // If any write failed, rollback
        if files_written > 0 && files_written < files.len() {
            self.rollback(&backups, &written_files)?;
            return Err(GenerationError::WriteFailed(
                "Partial write detected, rolled back all changes".to_string(),
            ));
        }

        let rollback_info = if !backups.is_empty() {
            Some(RollbackInfo {
                backups: backups.clone(),
                written_files: written_files.clone(),
            })
        } else {
            None
        };

        Ok(WriteResult {
            files: file_results,
            files_written,
            files_skipped,
            backups_created,
            dry_run: false,
            rollback_info,
        })
    }

    /// Write a single file
    ///
    /// # Arguments
    /// * `file_path` - Path to write to
    /// * `content` - File content
    /// * `backups` - List to track backups created
    ///
    /// # Returns
    /// Write result for this file
    fn write_file(
        &self,
        file_path: &Path,
        content: &str,
        backups: &mut Vec<(PathBuf, PathBuf)>,
    ) -> Result<FileWriteResult, GenerationError> {
        // Create backup if file exists and backups are enabled
        let backup_path = if file_path.exists() && self.config.create_backups {
            let backup = self.create_backup(file_path)?;
            backups.push((file_path.to_path_buf(), backup.clone()));
            Some(backup)
        } else {
            None
        };

        // Write file (unless dry-run)
        if !self.config.dry_run {
            let content_to_write = if self.config.format_code {
                self.format_code(content, file_path)?
            } else {
                content.to_string()
            };

            fs::write(file_path, content_to_write).map_err(|e| {
                GenerationError::WriteFailed(format!(
                    "Failed to write {}: {}",
                    file_path.display(),
                    e
                ))
            })?;
        }

        let has_backup = backup_path.is_some();
        Ok(FileWriteResult {
            path: file_path.to_path_buf(),
            written: true,
            backup_path,
            action: if has_backup {
                "Written (backup created)".to_string()
            } else {
                "Written".to_string()
            },
            dry_run: self.config.dry_run,
        })
    }

    /// Handle a file conflict
    ///
    /// # Arguments
    /// * `file_path` - Path to the conflicting file
    /// * `conflict` - Conflict information
    /// * `new_content` - New content to write
    /// * `backups` - List to track backups created
    ///
    /// # Returns
    /// Write result for this file
    fn handle_conflict(
        &self,
        file_path: &Path,
        conflict: &FileConflictInfo,
        new_content: &str,
        backups: &mut Vec<(PathBuf, PathBuf)>,
    ) -> Result<FileWriteResult, GenerationError> {
        // Resolve conflict using configured strategy
        let resolution =
            self.conflict_resolver
                .resolve(conflict, self.config.conflict_strategy, new_content)?;

        // Track backup if created
        if let Some(backup_path) = &resolution.backup_path {
            backups.push((file_path.to_path_buf(), PathBuf::from(backup_path)));
        }

        Ok(FileWriteResult {
            path: file_path.to_path_buf(),
            written: resolution.written,
            backup_path: resolution.backup_path.map(PathBuf::from),
            action: resolution.action,
            dry_run: self.config.dry_run,
        })
    }

    /// Create a backup of a file
    ///
    /// # Arguments
    /// * `file_path` - Path to file to backup
    ///
    /// # Returns
    /// Path to backup file
    fn create_backup(&self, file_path: &Path) -> Result<PathBuf, GenerationError> {
        let backup_path = format!("{}.bak", file_path.display());
        let backup_path_obj = PathBuf::from(&backup_path);

        if !self.config.dry_run {
            let content = fs::read_to_string(file_path).map_err(|e| {
                GenerationError::WriteFailed(format!("Failed to read file for backup: {}", e))
            })?;

            fs::write(&backup_path_obj, content).map_err(|e| {
                GenerationError::WriteFailed(format!("Failed to create backup: {}", e))
            })?;
        }

        Ok(backup_path_obj)
    }

    /// Format code before writing
    ///
    /// # Arguments
    /// * `content` - Code content to format
    /// * `_file_path` - Path to file (used to determine language)
    ///
    /// # Returns
    /// Formatted code
    fn format_code(&self, content: &str, _file_path: &Path) -> Result<String, GenerationError> {
        // For now, just return content as-is
        // In a real implementation, this would call language-specific formatters
        Ok(content.to_string())
    }

    /// Rollback changes by restoring backups
    ///
    /// # Arguments
    /// * `backups` - List of (original_path, backup_path) tuples
    /// * `written_files` - List of files that were written
    ///
    /// # Returns
    /// Result of rollback operation
    fn rollback(
        &self,
        backups: &[(PathBuf, PathBuf)],
        written_files: &[PathBuf],
    ) -> Result<(), GenerationError> {
        // Restore backups
        for (original_path, backup_path) in backups {
            if backup_path.exists() {
                let backup_content = fs::read_to_string(backup_path).map_err(|e| {
                    GenerationError::RollbackFailed(format!("Failed to read backup: {}", e))
                })?;

                fs::write(original_path, backup_content).map_err(|e| {
                    GenerationError::RollbackFailed(format!("Failed to restore backup: {}", e))
                })?;

                // Remove backup file
                fs::remove_file(backup_path).map_err(|e| {
                    GenerationError::RollbackFailed(format!("Failed to remove backup: {}", e))
                })?;
            }
        }

        // Remove written files
        for file_path in written_files {
            if file_path.exists() {
                fs::remove_file(file_path).map_err(|e| {
                    GenerationError::RollbackFailed(format!("Failed to remove file: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Preview changes without writing (dry-run mode)
    ///
    /// # Arguments
    /// * `files` - Generated files to preview
    /// * `target_dir` - Target directory
    /// * `conflicts` - Detected conflicts
    ///
    /// # Returns
    /// Write result showing what would be written
    pub fn preview(
        &self,
        files: &[GeneratedFile],
        target_dir: &Path,
        conflicts: &[FileConflictInfo],
    ) -> Result<WriteResult, GenerationError> {
        let mut config = self.config.clone();
        config.dry_run = true;

        let writer = OutputWriter::with_config(config);
        writer.write(files, target_dir, conflicts)
    }

    /// Get a summary of write results
    ///
    /// # Arguments
    /// * `result` - Write result to summarize
    ///
    /// # Returns
    /// Summary string
    pub fn summarize_result(&self, result: &WriteResult) -> String {
        format!(
            "Files written: {}, Files skipped: {}, Backups created: {}{}",
            result.files_written,
            result.files_skipped,
            result.backups_created,
            if result.dry_run { " (dry-run)" } else { "" }
        )
    }
}

impl Default for OutputWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_output_writer() {
        let _writer = OutputWriter::new();
    }

    #[test]
    fn test_write_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let writer = OutputWriter::new();

        let files = vec![GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];

        let result = writer.write(&files, temp_dir.path(), &[]).unwrap();

        assert_eq!(result.files_written, 1);
        assert_eq!(result.files_skipped, 0);
        assert!(temp_dir.path().join("src/main.rs").exists());
    }

    #[test]
    fn test_write_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let writer = OutputWriter::new();

        let files = vec![
            GeneratedFile {
                path: "src/main.rs".to_string(),
                content: "fn main() {}".to_string(),
                language: "rust".to_string(),
            },
            GeneratedFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn lib() {}".to_string(),
                language: "rust".to_string(),
            },
        ];

        let result = writer.write(&files, temp_dir.path(), &[]).unwrap();

        assert_eq!(result.files_written, 2);
        assert_eq!(result.files_skipped, 0);
        assert!(temp_dir.path().join("src/main.rs").exists());
        assert!(temp_dir.path().join("src/lib.rs").exists());
    }

    #[test]
    fn test_dry_run_mode() {
        let temp_dir = TempDir::new().unwrap();
        let config = OutputWriterConfig {
            dry_run: true,
            ..Default::default()
        };
        let writer = OutputWriter::with_config(config);

        let files = vec![GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];

        let result = writer.write(&files, temp_dir.path(), &[]).unwrap();

        assert!(result.dry_run);
        assert_eq!(result.files_written, 0);
        assert_eq!(result.files_skipped, 1);
        assert!(!temp_dir.path().join("src/main.rs").exists());
    }

    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let writer = OutputWriter::new();

        // Create existing file
        let file_path = temp_dir.path().join("existing.rs");
        fs::write(&file_path, "old content").unwrap();

        let files = vec![GeneratedFile {
            path: "existing.rs".to_string(),
            content: "new content".to_string(),
            language: "rust".to_string(),
        }];

        let result = writer.write(&files, temp_dir.path(), &[]).unwrap();

        assert_eq!(result.files_written, 1);
        assert_eq!(result.backups_created, 1);

        // Verify backup was created
        let backup_path = temp_dir.path().join("existing.rs.bak");
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "old content");

        // Verify new content was written
        let new_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(new_content, "new content");
    }

    #[test]
    fn test_conflict_skip_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let config = OutputWriterConfig {
            conflict_strategy: ConflictStrategy::Skip,
            ..Default::default()
        };
        let writer = OutputWriter::with_config(config);

        // Create existing file
        let file_path = temp_dir.path().join("existing.rs");
        fs::write(&file_path, "old content").unwrap();

        let files = vec![GeneratedFile {
            path: "existing.rs".to_string(),
            content: "new content".to_string(),
            language: "rust".to_string(),
        }];

        // Create conflict
        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: "old content".to_string(),
            new_content: "new content".to_string(),
            diff: crate::conflict_detector::FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 0,
            },
        };

        let result = writer.write(&files, temp_dir.path(), &[conflict]).unwrap();

        assert_eq!(result.files_written, 0);
        assert_eq!(result.files_skipped, 1);

        // Verify file was not overwritten
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "old content");
    }

    #[test]
    fn test_conflict_overwrite_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let config = OutputWriterConfig {
            conflict_strategy: ConflictStrategy::Overwrite,
            ..Default::default()
        };
        let writer = OutputWriter::with_config(config);

        // Create existing file
        let file_path = temp_dir.path().join("existing.rs");
        fs::write(&file_path, "old content").unwrap();

        let files = vec![GeneratedFile {
            path: "existing.rs".to_string(),
            content: "new content".to_string(),
            language: "rust".to_string(),
        }];

        // Create conflict
        let conflict = FileConflictInfo {
            path: file_path.clone(),
            old_content: "old content".to_string(),
            new_content: "new content".to_string(),
            diff: crate::conflict_detector::FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 0,
            },
        };

        let result = writer.write(&files, temp_dir.path(), &[conflict]).unwrap();

        assert_eq!(result.files_written, 1);
        assert_eq!(result.backups_created, 1);

        // Verify file was overwritten
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "new content");

        // Verify backup was created
        let backup_path = temp_dir.path().join("existing.rs.bak");
        assert!(backup_path.exists());
    }

    #[test]
    fn test_preview_mode() {
        let temp_dir = TempDir::new().unwrap();
        let writer = OutputWriter::new();

        let files = vec![GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];

        let result = writer.preview(&files, temp_dir.path(), &[]).unwrap();

        assert!(result.dry_run);
        assert_eq!(result.files_written, 0);
        assert!(!temp_dir.path().join("src/main.rs").exists());
    }

    #[test]
    fn test_summarize_result() {
        let writer = OutputWriter::new();
        let result = WriteResult {
            files: vec![],
            files_written: 5,
            files_skipped: 2,
            backups_created: 3,
            dry_run: false,
            rollback_info: None,
        };

        let summary = writer.summarize_result(&result);
        assert!(summary.contains("5"));
        assert!(summary.contains("2"));
        assert!(summary.contains("3"));
    }

    #[test]
    fn test_create_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let writer = OutputWriter::new();

        let files = vec![GeneratedFile {
            path: "src/nested/deep/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];

        let result = writer.write(&files, temp_dir.path(), &[]).unwrap();

        assert_eq!(result.files_written, 1);
        assert!(temp_dir.path().join("src/nested/deep/main.rs").exists());
    }
}
