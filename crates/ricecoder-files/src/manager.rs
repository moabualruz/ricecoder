//! FileManager coordinator for file operations
//!
//! The FileManager serves as the central coordinator for all file operations,
//! composing the SafeWriter, TransactionManager, and BackupManager to provide
//! a unified high-level API for safe file operations.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::{
    backup::BackupManager,
    error::FileError,
    models::{ConflictResolution, FileOperation, OperationType},
    transaction::TransactionManager,
    verifier::ContentVerifier,
    writer::SafeWriter,
};

/// Central coordinator for all file operations
///
/// FileManager orchestrates safe writes, transactions, conflict resolution,
/// and audit logging. It provides a high-level API for file operations.
///
/// # Example
///
/// ```ignore
/// use ricecoder_files::FileManager;
///
/// let manager = FileManager::new();
/// let op = manager.write_file(&path, "content").await?;
/// ```
#[derive(Debug)]
pub struct FileManager {
    writer: SafeWriter,
    transaction_manager: TransactionManager,
    backup_dir: PathBuf,
}

impl FileManager {
    /// Creates a new FileManager instance with default backup directory
    pub fn new() -> Self {
        let backup_dir = PathBuf::from(".ricecoder/backups");
        let backup_manager = BackupManager::new(backup_dir.clone(), 10);
        FileManager {
            writer: SafeWriter::new(),
            transaction_manager: TransactionManager::new(backup_manager),
            backup_dir,
        }
    }

    /// Creates a new FileManager with a custom backup directory
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory where backups will be stored
    pub fn with_backup_dir(backup_dir: PathBuf) -> Self {
        let backup_manager = BackupManager::new(backup_dir.clone(), 10);
        FileManager {
            writer: SafeWriter::new(),
            transaction_manager: TransactionManager::new(backup_manager),
            backup_dir,
        }
    }

    /// Writes a file safely with atomic operations
    ///
    /// Uses the SafeWriter internally to ensure atomic writes with
    /// optional backup creation for existing files.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to write
    /// * `content` - Content to write
    ///
    /// # Returns
    ///
    /// A FileOperation describing what was done, or an error
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<FileOperation, FileError> {
        self.writer
            .write(path, content, ConflictResolution::Overwrite)
            .await
    }

    /// Writes a file with a specific conflict resolution strategy
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to write
    /// * `content` - Content to write
    /// * `resolution` - Conflict resolution strategy to use
    ///
    /// # Returns
    ///
    /// A FileOperation describing what was done, or an error
    pub async fn write_file_with_strategy(
        &self,
        path: &Path,
        content: &str,
        resolution: ConflictResolution,
    ) -> Result<FileOperation, FileError> {
        self.writer.write(path, content, resolution).await
    }

    /// Reads a file's content
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// The file content as a string
    pub async fn read_file(&self, path: &Path) -> Result<String, FileError> {
        tokio::fs::read_to_string(path)
            .await
            .map_err(FileError::IoError)
    }

    /// Deletes a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to delete
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn delete_file(&self, path: &Path) -> Result<(), FileError> {
        tokio::fs::remove_file(path)
            .await
            .map_err(FileError::IoError)
    }

    /// Checks if a file exists
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check
    ///
    /// # Returns
    ///
    /// True if the file exists
    pub fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// Begins a new transaction
    ///
    /// # Returns
    ///
    /// A unique transaction ID
    pub async fn begin_transaction(&self) -> Result<Uuid, FileError> {
        self.transaction_manager.begin_transaction().await
    }

    /// Adds a file operation to a transaction
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID
    /// * `path` - Path to the file
    /// * `content` - Content to write
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn add_to_transaction(
        &self,
        tx_id: Uuid,
        path: &Path,
        content: &str,
    ) -> Result<(), FileError> {
        let op = FileOperation {
            path: path.to_path_buf(),
            operation: if path.exists() {
                OperationType::Update
            } else {
                OperationType::Create
            },
            content: Some(content.to_string()),
            backup_path: None,
            content_hash: Some(ContentVerifier::compute_hash(content)),
        };
        self.transaction_manager.add_operation(tx_id, op).await
    }

    /// Commits a transaction
    ///
    /// Executes all operations in the transaction atomically.
    /// If any operation fails, all changes are rolled back.
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID to commit
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn commit_transaction(&self, tx_id: Uuid) -> Result<(), FileError> {
        self.transaction_manager.commit(tx_id).await
    }

    /// Rolls back a transaction
    ///
    /// Restores all files to their pre-transaction state.
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID to rollback
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn rollback_transaction(&self, tx_id: Uuid) -> Result<(), FileError> {
        self.transaction_manager.rollback(tx_id).await
    }

    /// Gets the backup directory path
    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_file_manager_creation() {
        let manager = FileManager::new();
        assert_eq!(manager.backup_dir(), Path::new(".ricecoder/backups"));
    }

    #[test]
    fn test_file_manager_with_custom_backup_dir() {
        let backup_dir = PathBuf::from("/custom/backups");
        let manager = FileManager::with_backup_dir(backup_dir.clone());
        assert_eq!(manager.backup_dir(), backup_dir.as_path());
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::with_backup_dir(temp_dir.path().join("backups"));

        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        // Write file
        let op = manager.write_file(&file_path, content).await.unwrap();
        assert_eq!(op.path, file_path);
        assert!(op.content_hash.is_some());

        // Read file
        let read_content = manager.read_file(&file_path).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::new();

        let file_path = temp_dir.path().join("test.txt");
        assert!(!manager.file_exists(&file_path));

        tokio::fs::write(&file_path, "content").await.unwrap();
        assert!(manager.file_exists(&file_path));
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::new();

        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();
        assert!(manager.file_exists(&file_path));

        manager.delete_file(&file_path).await.unwrap();
        assert!(!manager.file_exists(&file_path));
    }

    #[tokio::test]
    async fn test_transaction_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::with_backup_dir(temp_dir.path().join("backups"));

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        // Begin transaction
        let tx_id = manager.begin_transaction().await.unwrap();

        // Add operations
        manager
            .add_to_transaction(tx_id, &file1, "content1")
            .await
            .unwrap();
        manager
            .add_to_transaction(tx_id, &file2, "content2")
            .await
            .unwrap();

        // Commit transaction
        manager.commit_transaction(tx_id).await.unwrap();

        // Verify files were created
        assert!(file1.exists());
        assert!(file2.exists());
    }

    #[tokio::test]
    async fn test_write_with_skip_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::new();

        let file_path = temp_dir.path().join("existing.txt");
        tokio::fs::write(&file_path, "original").await.unwrap();

        // Attempt to write with Skip strategy should fail
        let result = manager
            .write_file_with_strategy(&file_path, "new content", ConflictResolution::Skip)
            .await;

        assert!(result.is_err());
        match result {
            Err(FileError::ConflictDetected(_)) => (),
            _ => panic!("Expected ConflictDetected error"),
        }

        // Verify original content unchanged
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "original");
    }
}
