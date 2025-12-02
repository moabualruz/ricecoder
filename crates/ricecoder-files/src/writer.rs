//! Safe file writing with atomic operations and conflict resolution

use crate::conflict::ConflictResolver;
use crate::error::FileError;
use crate::models::{ConflictResolution, FileOperation, OperationType};
use crate::verifier::ContentVerifier;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

/// Implements atomic file write pattern with content verification
#[derive(Debug, Clone)]
pub struct SafeWriter {
    verifier: ContentVerifier,
    conflict_resolver: ConflictResolver,
}

impl SafeWriter {
    /// Creates a new SafeWriter instance
    pub fn new() -> Self {
        SafeWriter {
            verifier: ContentVerifier::new(),
            conflict_resolver: ConflictResolver::new(),
        }
    }

    /// Writes a file safely with atomic operations and conflict resolution
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to write
    /// * `content` - Content to write
    /// * `conflict_resolution` - Strategy for handling conflicts
    ///
    /// # Returns
    ///
    /// A FileOperation describing what was done, or an error
    pub async fn write(
        &self,
        path: &Path,
        content: &str,
        conflict_resolution: ConflictResolution,
    ) -> Result<FileOperation, FileError> {
        // 1. Validate content
        self.validate_content(content)?;

        // 2. Check for conflicts
        if let Some(conflict_info) = self.conflict_resolver.detect_conflict(path, content).await? {
            self.conflict_resolver.resolve(conflict_resolution, &conflict_info)?;
        }

        // 3. Create backup if file exists
        let _backup_path = if path.exists() {
            let backup = self.create_backup(path).await?;
            Some(backup)
        } else {
            None
        };

        // 4. Write to temp file and atomically rename
        let operation = match self.write_atomic(path, content).await {
            Ok(op) => op,
            Err(e) => {
                // If write fails, we don't need to restore backup here
                // The caller can handle rollback if needed
                return Err(e);
            }
        };

        // 5. Verify written content
        self.verifier.verify_write(path, content).await?;

        Ok(operation)
    }

    /// Validates content before writing
    ///
    /// # Arguments
    ///
    /// * `content` - Content to validate
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, error otherwise
    fn validate_content(&self, content: &str) -> Result<(), FileError> {
        // Basic validation: check for valid UTF-8 (already guaranteed by &str)
        // Additional validation can be added here
        if content.len() > 1_000_000_000 {
            return Err(FileError::InvalidContent(
                "Content exceeds maximum size".to_string(),
            ));
        }
        Ok(())
    }

    /// Writes content to a temporary file and atomically renames it
    ///
    /// # Arguments
    ///
    /// * `path` - Target path
    /// * `content` - Content to write
    ///
    /// # Returns
    ///
    /// FileOperation describing the write, or an error
    async fn write_atomic(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<FileOperation, FileError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).await?;
            }
        }

        // Generate temporary file path
        let temp_path = self.temp_path(path);

        // Write to temporary file
        fs::write(&temp_path, content).await?;

        // Atomically rename to target path
        fs::rename(&temp_path, path).await?;

        // Compute content hash
        let content_hash = ContentVerifier::compute_hash(content);

        Ok(FileOperation {
            path: path.to_path_buf(),
            operation: OperationType::Update,
            content: Some(content.to_string()),
            backup_path: None,
            content_hash: Some(content_hash),
        })
    }

    /// Generates a temporary file path
    ///
    /// # Arguments
    ///
    /// * `path` - Original file path
    ///
    /// # Returns
    ///
    /// Temporary file path
    fn temp_path(&self, path: &Path) -> std::path::PathBuf {
        let mut temp_path = path.to_path_buf();
        let file_name = format!(
            ".tmp-{}-{}",
            Uuid::new_v4().to_string(),
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
        );
        temp_path.set_file_name(file_name);
        temp_path
    }

    /// Creates a backup of an existing file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to backup
    ///
    /// # Returns
    ///
    /// Path to the backup file, or an error
    async fn create_backup(&self, path: &Path) -> Result<std::path::PathBuf, FileError> {
        let content = fs::read_to_string(path).await?;
        let backup_path = self.backup_path(path);

        // Create backup directory if needed
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&backup_path, &content).await?;
        Ok(backup_path)
    }

    /// Generates a backup file path
    ///
    /// # Arguments
    ///
    /// * `path` - Original file path
    ///
    /// # Returns
    ///
    /// Backup file path
    fn backup_path(&self, path: &Path) -> std::path::PathBuf {
        let mut backup_path = path.to_path_buf();
        let file_name = format!(
            ".backup-{}-{}",
            Uuid::new_v4().to_string(),
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
        );
        backup_path.set_file_name(file_name);
        backup_path
    }
}

impl Default for SafeWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_new_file() {
        let writer = SafeWriter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("new.txt");

        let content = "test content";
        let result = writer
            .write(&path, content, ConflictResolution::Overwrite)
            .await;

        assert!(result.is_ok());
        let operation = result.unwrap();
        assert_eq!(operation.path, path);
        assert!(operation.content_hash.is_some());

        // Verify file was written
        let written = fs::read_to_string(&path).await.unwrap();
        assert_eq!(written, content);
    }

    #[tokio::test]
    async fn test_write_with_overwrite_strategy() {
        let writer = SafeWriter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("existing.txt");

        // Create existing file
        fs::write(&path, "old content").await.unwrap();

        let new_content = "new content";
        let result = writer
            .write(&path, new_content, ConflictResolution::Overwrite)
            .await;

        assert!(result.is_ok());

        // Verify file was overwritten
        let written = fs::read_to_string(&path).await.unwrap();
        assert_eq!(written, new_content);
    }

    #[tokio::test]
    async fn test_write_with_skip_strategy() {
        let writer = SafeWriter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("existing.txt");

        // Create existing file
        fs::write(&path, "old content").await.unwrap();

        let new_content = "new content";
        let result = writer
            .write(&path, new_content, ConflictResolution::Skip)
            .await;

        assert!(result.is_err());
        match result {
            Err(FileError::ConflictDetected(_)) => (),
            _ => panic!("Expected ConflictDetected error"),
        }

        // Verify file was not changed
        let written = fs::read_to_string(&path).await.unwrap();
        assert_eq!(written, "old content");
    }

    #[tokio::test]
    async fn test_write_creates_parent_directories() {
        let writer = SafeWriter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("subdir/nested/file.txt");

        let content = "test content";
        let result = writer
            .write(&path, content, ConflictResolution::Overwrite)
            .await;

        assert!(result.is_ok());
        assert!(path.exists());

        let written = fs::read_to_string(&path).await.unwrap();
        assert_eq!(written, content);
    }

    #[tokio::test]
    async fn test_write_invalid_content_too_large() {
        let writer = SafeWriter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("large.txt");

        // Create content larger than 1GB (simulated)
        let large_content = "x".repeat(1_000_000_001);

        let result = writer
            .write(&path, &large_content, ConflictResolution::Overwrite)
            .await;

        assert!(result.is_err());
        match result {
            Err(FileError::InvalidContent(_)) => (),
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_validate_content_valid() {
        let writer = SafeWriter::new();
        let result = writer.validate_content("valid content");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_content_empty() {
        let writer = SafeWriter::new();
        let result = writer.validate_content("");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_with_merge_strategy() {
        let writer = SafeWriter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("merge.txt");

        // Create existing file
        fs::write(&path, "existing content").await.unwrap();

        let new_content = "new content";
        let result = writer
            .write(&path, new_content, ConflictResolution::Merge)
            .await;

        assert!(result.is_ok());
    }
}
