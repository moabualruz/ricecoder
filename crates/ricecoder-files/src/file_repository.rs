//! FileSystemRepository Implementation
//!
//! Implements the `FileRepository` port from ricecoder-domain using the
//! standard library's file system operations.
//!
//! File System Repository

use std::path::PathBuf;

use async_trait::async_trait;
use ricecoder_domain::{
    DomainError, DomainResult, FileManager, FileMetadata, FileReader, FileWriter, WriteResult,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::backup::BackupManager;
use crate::error::FileError;

/// Filesystem-based implementation of `FileRepository`
///
/// Provides safe file operations with optional backup support.
///
/// # Example
///
/// ```ignore
/// use ricecoder_files::FileSystemRepository;
/// use ricecoder_domain::FileRepository;
///
/// let repo = FileSystemRepository::new();
/// let content = repo.read_string(&PathBuf::from("file.txt")).await?;
/// ```
#[derive(Debug, Clone)]
pub struct FileSystemRepository {
    /// Directory for storing backups
    backup_dir: Option<PathBuf>,
}

impl FileSystemRepository {
    /// Create a new FileSystemRepository without backup support
    pub fn new() -> Self {
        Self { backup_dir: None }
    }

    /// Create a new FileSystemRepository with backup support
    pub fn with_backup_dir(backup_dir: PathBuf) -> Self {
        Self {
            backup_dir: Some(backup_dir),
        }
    }

    /// Convert FileError to DomainError
    fn to_domain_error(err: FileError) -> DomainError {
        DomainError::FileOperationError {
            operation: "file".to_string(),
            reason: err.to_string(),
        }
    }

    /// Convert std::io::Error to DomainError
    fn io_to_domain_error(err: std::io::Error, context: &str) -> DomainError {
        DomainError::IoError {
            reason: format!("{}: {}", context, err),
        }
    }

    /// Create a backup of a file if backup_dir is set
    async fn create_backup(&self, path: &PathBuf) -> Option<PathBuf> {
        if let Some(ref backup_dir) = self.backup_dir {
            if path.exists() {
                let backup_manager = BackupManager::new(backup_dir.clone(), 10);
                match backup_manager.create_backup(path).await {
                    Ok(info) => Some(info.backup_path),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for FileSystemRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// ISP-compliant: FileReader implementation (read-only operations)
#[async_trait]
impl FileReader for FileSystemRepository {
    async fn read(&self, path: &PathBuf) -> DomainResult<Vec<u8>> {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to open file"))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to read file"))?;

        Ok(buffer)
    }

    async fn read_string(&self, path: &PathBuf) -> DomainResult<String> {
        let bytes = self.read(path).await?;
        String::from_utf8(bytes).map_err(|e| DomainError::ValidationError {
            field: "file_content".to_string(),
            reason: format!("File is not valid UTF-8: {}", e),
        })
    }

    async fn exists(&self, path: &PathBuf) -> DomainResult<bool> {
        Ok(tokio::fs::try_exists(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to check file existence"))?)
    }

    async fn metadata(&self, path: &PathBuf) -> DomainResult<FileMetadata> {
        let meta = tokio::fs::metadata(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to get metadata"))?;

        let modified = meta
            .modified()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

        let created = meta
            .created()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

        Ok(FileMetadata {
            path: path.clone(),
            size: meta.len(),
            is_directory: meta.is_dir(),
            modified,
            created,
            is_readonly: meta.permissions().readonly(),
        })
    }

    async fn list_directory(&self, path: &PathBuf) -> DomainResult<Vec<FileMetadata>> {
        self.list_directory_with_ignore(path, &[]).await
    }
}

impl FileSystemRepository {
    /// List directory contents with ignore pattern support
    ///
    /// # Arguments
    /// * `path` - Directory path to list
    /// * `ignore_patterns` - Glob patterns to ignore (e.g., ["node_modules", "*.log", ".git"])
    ///
    /// # Returns
    /// Vector of FileMetadata for non-ignored entries
    pub async fn list_directory_with_ignore(
        &self,
        path: &PathBuf,
        ignore_patterns: &[&str],
    ) -> DomainResult<Vec<FileMetadata>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to read directory"))?;

        // Compile glob patterns
        let patterns: Vec<glob::Pattern> = ignore_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to read directory entry"))?
        {
            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Check if entry matches any ignore pattern
            let should_ignore = patterns.iter().any(|pattern| pattern.matches(file_name));

            if should_ignore {
                continue;
            }

            match self.metadata(&entry_path).await {
                Ok(meta) => entries.push(meta),
                Err(_) => continue, // Skip entries we can't read
            }
        }

        Ok(entries)
    }
}

/// ISP-compliant: FileWriter implementation (write operations)
#[async_trait]
impl FileWriter for FileSystemRepository {
    async fn write(
        &self,
        path: &PathBuf,
        content: &[u8],
        create_backup: bool,
    ) -> DomainResult<WriteResult> {
        // Create backup if requested
        let backup_path = if create_backup {
            self.create_backup(path).await
        } else {
            None
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| Self::io_to_domain_error(e, "Failed to create parent directory"))?;
        }

        // Write file atomically (write to temp, then rename)
        let temp_path = path.with_extension("tmp");

        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to create temp file"))?;

        file.write_all(content)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to write content"))?;

        file.sync_all()
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to sync file"))?;

        // Rename temp file to target (atomic on most systems)
        tokio::fs::rename(&temp_path, path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to rename temp file"))?;

        Ok(WriteResult {
            path: path.clone(),
            bytes_written: content.len() as u64,
            backup_created: backup_path.is_some(),
            backup_path,
        })
    }

    async fn write_string(
        &self,
        path: &PathBuf,
        content: &str,
        create_backup: bool,
    ) -> DomainResult<WriteResult> {
        self.write(path, content.as_bytes(), create_backup).await
    }

    async fn delete(&self, path: &PathBuf) -> DomainResult<()> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to get file metadata"))?;

        if metadata.is_dir() {
            tokio::fs::remove_dir(path)
                .await
                .map_err(|e| Self::io_to_domain_error(e, "Failed to remove directory"))?;
        } else {
            tokio::fs::remove_file(path)
                .await
                .map_err(|e| Self::io_to_domain_error(e, "Failed to remove file"))?;
        }

        Ok(())
    }
}

/// ISP-compliant: FileManager implementation (file management operations)
#[async_trait]
impl FileManager for FileSystemRepository {
    async fn create_directory(&self, path: &PathBuf) -> DomainResult<()> {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to create directory"))?;
        Ok(())
    }

    async fn copy(&self, from: &PathBuf, to: &PathBuf) -> DomainResult<u64> {
        tokio::fs::copy(from, to)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to copy file"))
    }

    async fn rename(&self, from: &PathBuf, to: &PathBuf) -> DomainResult<()> {
        tokio::fs::rename(from, to)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to rename file"))?;
        Ok(())
    }
}

// FileRepository is automatically implemented via blanket impl in ricecoder-domain
// for any T: FileReader + FileWriter + FileManager

// Tests for FileSystemRepository
// Note: Full integration tests are in tests/ directory. Unit tests here focus on isolated behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_system_repository_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        // Write file
        let result = repo.write_string(&file_path, content, false).await;
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.bytes_written, content.len() as u64);
        assert!(!write_result.backup_created);

        // Read file
        let read_content = repo.read_string(&file_path).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_file_system_repository_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");

        // File should not exist initially
        assert!(!repo.exists(&file_path).await.unwrap());

        // Create file
        fs::write(&file_path, "content").await.unwrap();

        // File should now exist
        assert!(repo.exists(&file_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_system_repository_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";
        fs::write(&file_path, content).await.unwrap();

        let metadata = repo.metadata(&file_path).await.unwrap();
        assert_eq!(metadata.size, content.len() as u64);
        assert!(!metadata.is_directory);
        assert!(!metadata.is_readonly);
    }

    #[tokio::test]
    async fn test_file_system_repository_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), "content1")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2")
            .await
            .unwrap();

        let entries = repo.list_directory(&temp_dir.path().to_path_buf()).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_file_system_repository_delete() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").await.unwrap();

        // File should exist
        assert!(repo.exists(&file_path).await.unwrap());

        // Delete file
        repo.delete(&file_path).await.unwrap();

        // File should not exist
        assert!(!repo.exists(&file_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_system_repository_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let repo = FileSystemRepository::with_backup_dir(backup_dir);

        let file_path = temp_dir.path().join("test.txt");
        let original_content = "Original content";
        fs::write(&file_path, original_content).await.unwrap();

        // Write with backup
        let result = repo.write_string(&file_path, "New content", true).await;
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert!(write_result.backup_created);
        assert!(write_result.backup_path.is_some());
    }

    #[tokio::test]
    async fn test_file_system_repository_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let dir_path = temp_dir.path().join("subdir").join("nested");
        repo.create_directory(&dir_path).await.unwrap();

        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[tokio::test]
    async fn test_file_system_repository_copy() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        let content = "Copy me!";
        fs::write(&source, content).await.unwrap();

        let bytes_copied = repo.copy(&source, &dest).await.unwrap();
        assert_eq!(bytes_copied, content.len() as u64);

        let dest_content = fs::read_to_string(&dest).await.unwrap();
        assert_eq!(dest_content, content);
    }

    #[tokio::test]
    async fn test_file_system_repository_rename() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        let content = "Rename me!";
        fs::write(&source, content).await.unwrap();

        repo.rename(&source, &dest).await.unwrap();

        assert!(!source.exists());
        assert!(dest.exists());

        let dest_content = fs::read_to_string(&dest).await.unwrap();
        assert_eq!(dest_content, content);
    }
}
