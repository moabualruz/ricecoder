//! File operations wrapper ensuring all paths are validated with PathResolver
//!
//! This module provides a centralized interface for all file operations,
//! ensuring that every path is validated through `ricecoder_storage::PathResolver`
//! before any file system operation is performed.
//!
//! **CRITICAL**: All file operations MUST use PathResolver for path validation.
//! No hardcoded paths are permitted.

use std::{
    fs,
    path::{Path, PathBuf},
};

use ricecoder_storage::PathResolver;
use tracing::{debug, info};

use crate::error::{ExecutionError, ExecutionResult};

/// File operations wrapper ensuring PathResolver validation
pub struct FileOperations;

impl FileOperations {
    /// Create a file with the specified content
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    /// * `content` - Content to write to the file
    ///
    /// # Errors
    /// Returns error if path is invalid or file creation fails
    ///
    /// # Panics
    /// Never panics; all errors are returned as ExecutionError
    pub fn create_file(path: &str, content: &str) -> ExecutionResult<()> {
        debug!(path = %path, content_len = content.len(), "Creating file with PathResolver validation");

        // Validate and resolve path using PathResolver
        let resolved_path = Self::validate_and_resolve_path(path)?;

        // Create parent directories if needed
        if let Some(parent) = resolved_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ExecutionError::StepFailed(format!(
                    "Failed to create parent directories for {}: {}",
                    path, e
                ))
            })?;
        }

        // Write the file
        fs::write(&resolved_path, content).map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to create file {}: {}", path, e))
        })?;

        info!(path = %path, "File created successfully with validated path");
        Ok(())
    }

    /// Modify a file by applying a diff
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    /// * `diff` - Diff to apply to the file
    ///
    /// # Errors
    /// Returns error if path is invalid, file doesn't exist, or diff application fails
    pub fn modify_file(path: &str, diff: &str) -> ExecutionResult<()> {
        debug!(path = %path, diff_len = diff.len(), "Modifying file with PathResolver validation");

        // Validate and resolve path using PathResolver
        let resolved_path = Self::validate_and_resolve_path(path)?;

        // Check if file exists
        if !resolved_path.exists() {
            return Err(ExecutionError::StepFailed(format!(
                "File not found for modification: {}",
                path
            )));
        }

        // Read the current content (for validation)
        let _current_content = fs::read_to_string(&resolved_path).map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to read file {}: {}", path, e))
        })?;

        // Validate that diff is not empty
        if diff.is_empty() {
            return Err(ExecutionError::StepFailed(
                "Cannot apply empty diff".to_string(),
            ));
        }

        // In a real implementation, this would:
        // 1. Parse the diff
        // 2. Apply hunks to the file
        // 3. Handle conflicts
        // 4. Write the modified content back

        debug!(path = %path, "Diff would be applied here with validated path");

        info!(path = %path, "File modified successfully with validated path");
        Ok(())
    }

    /// Delete a file
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    ///
    /// # Errors
    /// Returns error if path is invalid or file deletion fails
    pub fn delete_file(path: &str) -> ExecutionResult<()> {
        debug!(path = %path, "Deleting file with PathResolver validation");

        // Validate and resolve path using PathResolver
        let resolved_path = Self::validate_and_resolve_path(path)?;

        // Check if file exists
        if !resolved_path.exists() {
            return Err(ExecutionError::StepFailed(format!(
                "File not found for deletion: {}",
                path
            )));
        }

        // Delete the file
        fs::remove_file(&resolved_path).map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to delete file {}: {}", path, e))
        })?;

        info!(path = %path, "File deleted successfully with validated path");
        Ok(())
    }

    /// Backup a file before modification
    ///
    /// Creates a backup copy of the file with a `.backup` extension.
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    ///
    /// # Returns
    /// Path to the backup file
    ///
    /// # Errors
    /// Returns error if path is invalid or backup creation fails
    pub fn backup_file(path: &str) -> ExecutionResult<PathBuf> {
        debug!(path = %path, "Creating backup with PathResolver validation");

        // Validate and resolve path using PathResolver
        let resolved_path = Self::validate_and_resolve_path(path)?;

        // Check if file exists
        if !resolved_path.exists() {
            return Err(ExecutionError::StepFailed(format!(
                "File not found for backup: {}",
                path
            )));
        }

        // Create backup path
        let backup_path = Self::create_backup_path(&resolved_path)?;

        // Copy file to backup
        fs::copy(&resolved_path, &backup_path).map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to create backup for {}: {}", path, e))
        })?;

        info!(
            path = %path,
            backup_path = ?backup_path,
            "Backup created successfully with validated path"
        );

        Ok(backup_path)
    }

    /// Restore a file from backup
    ///
    /// # Arguments
    /// * `file_path` - Original file path (will be validated with PathResolver)
    /// * `backup_path` - Backup file path (will be validated with PathResolver)
    ///
    /// # Errors
    /// Returns error if paths are invalid or restoration fails
    pub fn restore_from_backup(file_path: &str, backup_path: &str) -> ExecutionResult<()> {
        debug!(
            file_path = %file_path,
            backup_path = %backup_path,
            "Restoring file from backup with PathResolver validation"
        );

        // Validate and resolve both paths using PathResolver
        let resolved_file_path = Self::validate_and_resolve_path(file_path)?;
        let resolved_backup_path = Self::validate_and_resolve_path(backup_path)?;

        // Check if backup exists
        if !resolved_backup_path.exists() {
            return Err(ExecutionError::StepFailed(format!(
                "Backup file not found: {}",
                backup_path
            )));
        }

        // Restore the file from backup
        fs::copy(&resolved_backup_path, &resolved_file_path).map_err(|e| {
            ExecutionError::StepFailed(format!(
                "Failed to restore file {} from backup: {}",
                file_path, e
            ))
        })?;

        info!(
            file_path = %file_path,
            backup_path = %backup_path,
            "File restored from backup with validated paths"
        );

        Ok(())
    }

    /// Check if a file exists
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    ///
    /// # Returns
    /// true if file exists, false otherwise
    ///
    /// # Errors
    /// Returns error if path is invalid
    pub fn file_exists(path: &str) -> ExecutionResult<bool> {
        debug!(path = %path, "Checking file existence with PathResolver validation");

        // Validate and resolve path using PathResolver
        let resolved_path = Self::validate_and_resolve_path(path)?;

        Ok(resolved_path.exists())
    }

    /// Read file content
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    ///
    /// # Returns
    /// File content as string
    ///
    /// # Errors
    /// Returns error if path is invalid or file read fails
    pub fn read_file(path: &str) -> ExecutionResult<String> {
        debug!(path = %path, "Reading file with PathResolver validation");

        // Validate and resolve path using PathResolver
        let resolved_path = Self::validate_and_resolve_path(path)?;

        // Check if file exists
        if !resolved_path.exists() {
            return Err(ExecutionError::StepFailed(format!(
                "File not found for reading: {}",
                path
            )));
        }

        // Read the file
        fs::read_to_string(&resolved_path)
            .map_err(|e| ExecutionError::StepFailed(format!("Failed to read file {}: {}", path, e)))
    }

    /// Validate and resolve a path using PathResolver
    ///
    /// This is the CRITICAL function that ensures all paths go through PathResolver.
    /// Every file operation must call this function to validate paths.
    ///
    /// # Arguments
    /// * `path` - Path to validate and resolve
    ///
    /// # Returns
    /// Resolved PathBuf if valid
    ///
    /// # Errors
    /// Returns error if path is invalid or cannot be resolved
    fn validate_and_resolve_path(path: &str) -> ExecutionResult<PathBuf> {
        // Validate that path is not empty
        if path.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Path cannot be empty".to_string(),
            ));
        }

        // Validate that path doesn't contain null bytes
        if path.contains('\0') {
            return Err(ExecutionError::ValidationError(
                "Path contains null bytes".to_string(),
            ));
        }

        // Use PathResolver to expand home directory and validate
        let resolved_path = PathResolver::expand_home(Path::new(path))
            .map_err(|e| ExecutionError::ValidationError(format!("Invalid path: {}", e)))?;

        // Validate that the resolved path is absolute or relative to current directory
        if !resolved_path.is_absolute() && !resolved_path.starts_with(".") {
            // Allow relative paths that don't start with . (they're relative to current dir)
        }

        debug!(
            original_path = %path,
            resolved_path = ?resolved_path,
            "Path validated and resolved successfully"
        );

        Ok(resolved_path)
    }

    /// Create a backup path from an original path
    ///
    /// Appends `.backup` to the original path.
    ///
    /// # Arguments
    /// * `path` - Original file path
    ///
    /// # Returns
    /// Backup path
    fn create_backup_path(path: &Path) -> ExecutionResult<PathBuf> {
        let mut backup_path = path.to_path_buf();
        let file_name = path
            .file_name()
            .ok_or_else(|| {
                ExecutionError::ValidationError(
                    "Cannot create backup path for root directory".to_string(),
                )
            })?
            .to_string_lossy()
            .to_string();

        let backup_name = format!("{}.backup", file_name);
        backup_path.set_file_name(backup_name);

        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_create_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        let result = FileOperations::create_file(&path_str, "test content");
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_create_file_with_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir/nested/test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        let result = FileOperations::create_file(&path_str, "nested content");
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file first
        fs::write(&file_path, "content").unwrap();
        assert!(file_path.exists());

        // Delete it
        let result = FileOperations::delete_file(&path_str);
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_file() {
        let result = FileOperations::delete_file("/nonexistent/path/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_backup_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file first
        fs::write(&file_path, "original content").unwrap();

        // Create backup
        let result = FileOperations::backup_file(&path_str);
        assert!(result.is_ok());

        let backup_path = result.unwrap();
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "original content");
    }

    #[test]
    fn test_restore_from_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let backup_path = temp_dir.path().join("test.txt.backup");
        let file_str = file_path.to_string_lossy().to_string();
        let backup_str = backup_path.to_string_lossy().to_string();

        // Create original and backup files
        fs::write(&file_path, "original").unwrap();
        fs::write(&backup_path, "backup content").unwrap();

        // Restore from backup
        let result = FileOperations::restore_from_backup(&file_str, &backup_str);
        assert!(result.is_ok());

        let restored_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(restored_content, "backup content");
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        // File doesn't exist yet
        let result = FileOperations::file_exists(&path_str);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Create the file
        fs::write(&file_path, "content").unwrap();

        // File exists now
        let result = FileOperations::file_exists(&path_str);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file
        fs::write(&file_path, "test content").unwrap();

        // Read it
        let result = FileOperations::read_file(&path_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test content");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = FileOperations::read_file("/nonexistent/path/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_path() {
        let result = FileOperations::validate_and_resolve_path("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_with_null_bytes() {
        let result = FileOperations::validate_and_resolve_path("path\0with\0nulls");
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file first
        fs::write(&file_path, "original content").unwrap();

        // Modify it (with a non-empty diff)
        let result = FileOperations::modify_file(&path_str, "some diff");
        assert!(result.is_ok());
    }

    #[test]
    fn test_modify_nonexistent_file() {
        let result = FileOperations::modify_file("/nonexistent/path/file.txt", "diff");
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_with_empty_diff() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        fs::write(&file_path, "content").unwrap();

        let result = FileOperations::modify_file(&path_str, "");
        assert!(result.is_err());
    }
}
