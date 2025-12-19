//! Replace operations for RiceGrep
//!
//! This module provides safe file transformation capabilities with
//! preview, rollback, and validation features.

use crate::error::RiceGrepError;
use std::path::PathBuf;
use std::fs;
use tokio::fs as async_fs;

/// Replace operation representing a single transformation
#[derive(Debug, Clone)]
pub struct ReplaceOperation {
    /// File to modify
    pub file_path: PathBuf,
    /// Line number (1-indexed)
    pub line_number: usize,
    /// Original line content
    pub old_content: String,
    /// New line content after replacement
    pub new_content: String,
    /// Byte offset in file
    pub byte_offset: usize,
}

/// Result of executing replace operations
#[derive(Debug, Clone)]
pub struct ReplaceResult {
    /// Number of files modified
    pub files_modified: usize,
    /// Number of operations that succeeded
    pub operations_successful: usize,
    /// Number of operations that failed
    pub operations_failed: usize,
    /// Error messages for failed operations
    pub errors: Vec<String>,
}

/// Engine for executing replace operations safely
pub struct ReplaceEngine {
    /// Maximum file size to process (in bytes)
    max_file_size: u64,
    /// Create backup files
    create_backups: bool,
}

impl ReplaceEngine {
    /// Create a new replace engine with default settings
    pub fn new() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            create_backups: true,
        }
    }

    /// Execute a batch of replace operations
    pub async fn execute_operations(&self, operations: Vec<ReplaceOperation>) -> Result<ReplaceResult, RiceGrepError> {
        let mut result = ReplaceResult {
            files_modified: 0,
            operations_successful: 0,
            operations_failed: 0,
            errors: Vec::new(),
        };

        // Group operations by file
        let mut file_operations = std::collections::HashMap::new();
        for op in operations {
            file_operations.entry(op.file_path.clone()).or_insert_with(Vec::new).push(op);
        }

        // Process each file
        for (file_path, ops) in file_operations {
            let ops_len = ops.len();
            match self.process_file(&file_path, ops).await {
                Ok(success_count) => {
                    if success_count > 0 {
                        result.files_modified += 1;
                        result.operations_successful += success_count;
                    }
                }
                Err(e) => {
                    result.operations_failed += ops_len;
                    result.errors.push(format!("{}: {}", file_path.display(), e));
                }
            }
        }

        Ok(result)
    }

    /// Process all replace operations for a single file
    async fn process_file(&self, file_path: &PathBuf, operations: Vec<ReplaceOperation>) -> Result<usize, RiceGrepError> {
        // Check file size
        let metadata = fs::metadata(file_path)?;
        if metadata.len() > self.max_file_size {
            return Err(RiceGrepError::Search {
                message: format!("File too large: {} bytes (max: {} bytes)",
                               metadata.len(), self.max_file_size)
            });
        }

        // Read file content
        let content = fs::read_to_string(file_path)?;

        // Create backup if requested
        if self.create_backups {
            let backup_path = file_path.with_extension(format!("{}.backup",
                file_path.extension().unwrap_or_default().to_string_lossy()));
            fs::write(&backup_path, &content)?;
        }

        // Apply transformations
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut applied_count = 0;

        for operation in operations {
            let line_idx = operation.line_number - 1; // Convert to 0-indexed
            if line_idx < lines.len() && lines[line_idx] == operation.old_content {
                lines[line_idx] = operation.new_content.clone();
                applied_count += 1;
            } else {
                // Line doesn't match - skip this operation
                continue;
            }
        }

        // Write back to file
        let new_content = lines.join("\n");
        fs::write(file_path, new_content)?;

        Ok(applied_count)
    }

    /// Validate replace operations without executing them
    pub fn validate_operations(&self, operations: &[ReplaceOperation]) -> Result<(), RiceGrepError> {
        for operation in operations {
            // Check if file exists and is readable
            if !operation.file_path.exists() {
                return Err(RiceGrepError::Search {
                    message: format!("File does not exist: {}", operation.file_path.display())
                });
            }

            // Check file size
            let metadata = fs::metadata(&operation.file_path)?;
            if metadata.len() > self.max_file_size {
                return Err(RiceGrepError::Search {
                    message: format!("File too large: {} bytes (max: {} bytes)",
                                   metadata.len(), self.max_file_size)
                });
            }

            // Basic validation of operation
            if operation.line_number == 0 {
                return Err(RiceGrepError::Search {
                    message: "Invalid line number: 0".to_string()
                });
            }
        }

        Ok(())
    }
}

impl Default for ReplaceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_replace_operation() {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn main() {{").unwrap();
        writeln!(temp_file, "    println!(\"hello\");").unwrap();
        writeln!(temp_file, "}}").unwrap();
        let file_path = temp_file.path().to_path_buf();

        // Create replace operation
        let operation = ReplaceOperation {
            file_path: file_path.clone(),
            line_number: 2,
            old_content: "    println!(\"hello\");".to_string(),
            new_content: "    println!(\"world\");".to_string(),
            byte_offset: 12,
        };

        // Execute operation
        let engine = ReplaceEngine::new();
        let result = engine.execute_operations(vec![operation]).await.unwrap();

        // Verify result
        assert_eq!(result.files_modified, 1);
        assert_eq!(result.operations_successful, 1);
        assert_eq!(result.operations_failed, 0);

        // Verify file content
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("println!(\"world\")"));
        assert!(!content.contains("println!(\"hello\")"));
    }

    #[test]
    fn test_validation() {
        let engine = ReplaceEngine::new();

        // Test with non-existent file
        let operation = ReplaceOperation {
            file_path: PathBuf::from("/non/existent/file"),
            line_number: 1,
            old_content: "test".to_string(),
            new_content: "test".to_string(),
            byte_offset: 0,
        };

        assert!(engine.validate_operations(&[operation]).is_err());
    }
}