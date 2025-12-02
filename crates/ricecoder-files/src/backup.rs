//! Backup management with retention and restoration

use crate::error::FileError;
use crate::models::BackupMetadata;
use crate::verifier::ContentVerifier;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Manages backup creation, retention, and restoration
#[derive(Debug, Clone)]
pub struct BackupManager {
    backup_dir: PathBuf,
    retention_count: usize,
}

impl BackupManager {
    /// Creates a new BackupManager instance
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory where backups will be stored
    /// * `retention_count` - Number of backups to keep per file (default: 10)
    pub fn new(backup_dir: PathBuf, retention_count: usize) -> Self {
        BackupManager {
            backup_dir,
            retention_count,
        }
    }

    /// Creates a timestamped backup of a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to backup
    ///
    /// # Returns
    ///
    /// BackupMetadata containing backup location and hash, or an error
    pub async fn create_backup(&self, path: &Path) -> Result<BackupMetadata, FileError> {
        // Read the original file
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| FileError::BackupFailed(format!(
                "Failed to read file for backup: {}",
                e
            )))?;

        // Compute hash of the content
        let content_hash = ContentVerifier::compute_hash(&content);

        // Create backup directory structure
        fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| FileError::BackupFailed(format!(
                "Failed to create backup directory: {}",
                e
            )))?;

        // Generate timestamped backup filename
        let timestamp = Utc::now();
        let filename = format!(
            "{}.{}.bak",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file"),
            timestamp.format("%Y%m%d_%H%M%S_%f")
        );

        let backup_path = self.backup_dir.join(&filename);

        // Write backup file
        fs::write(&backup_path, &content)
            .await
            .map_err(|e| FileError::BackupFailed(format!(
                "Failed to write backup file: {}",
                e
            )))?;

        Ok(BackupMetadata {
            original_path: path.to_path_buf(),
            backup_path,
            timestamp,
            content_hash,
        })
    }

    /// Enforces retention policy by keeping only the last N backups per file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the original file (used to identify related backups)
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn enforce_retention_policy(&self, path: &Path) -> Result<(), FileError> {
        // Get the base filename for matching backups
        let base_filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| FileError::BackupFailed("Invalid file path".to_string()))?;

        // Read all entries in backup directory
        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| FileError::BackupFailed(format!(
                "Failed to read backup directory: {}",
                e
            )))?;

        let mut backups = Vec::new();

        // Collect all backups for this file
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| FileError::BackupFailed(format!(
                "Failed to read backup entry: {}",
                e
            )))?
        {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Check if this backup belongs to our file
                if filename.starts_with(base_filename) && filename.ends_with(".bak") {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        backups.push((path, metadata.modified().unwrap_or_else(|_| {
                            std::time::SystemTime::now()
                        })));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // Delete old backups beyond retention count
        for (backup_path, _) in backups.iter().skip(self.retention_count) {
            fs::remove_file(backup_path)
                .await
                .map_err(|e| FileError::BackupFailed(format!(
                    "Failed to delete old backup: {}",
                    e
                )))?;
        }

        Ok(())
    }

    /// Restores a file from a backup
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file
    /// * `target_path` - Path where the file should be restored
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn restore_from_backup(
        &self,
        backup_path: &Path,
        target_path: &Path,
    ) -> Result<(), FileError> {
        // Read backup content
        let content = fs::read_to_string(backup_path)
            .await
            .map_err(|e| FileError::BackupFailed(format!(
                "Failed to read backup file: {}",
                e
            )))?;

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| FileError::BackupFailed(format!(
                    "Failed to create target directory: {}",
                    e
                )))?;
        }

        // Write to target location
        fs::write(target_path, &content)
            .await
            .map_err(|e| FileError::BackupFailed(format!(
                "Failed to restore file: {}",
                e
            )))?;

        Ok(())
    }

    /// Verifies backup integrity by comparing stored hash with computed hash
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file
    /// * `stored_hash` - Previously stored hash to compare against
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn verify_backup_integrity(
        &self,
        backup_path: &Path,
        stored_hash: &str,
    ) -> Result<(), FileError> {
        let verifier = ContentVerifier::new();
        verifier.verify_backup(backup_path, stored_hash).await
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new(PathBuf::from(".ricecoder/backups"), 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("test.txt");

        // Create a test file
        fs::write(&file_path, "test content").await.unwrap();

        let manager = BackupManager::new(backup_dir, 10);
        let metadata = manager.create_backup(&file_path).await.unwrap();

        // Verify backup was created
        assert!(metadata.backup_path.exists());
        let backup_content = fs::read_to_string(&metadata.backup_path)
            .await
            .unwrap();
        assert_eq!(backup_content, "test content");
    }

    #[tokio::test]
    async fn test_backup_metadata_hash() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("test.txt");

        let content = "test content";
        fs::write(&file_path, content).await.unwrap();

        let manager = BackupManager::new(backup_dir, 10);
        let metadata = manager.create_backup(&file_path).await.unwrap();

        // Verify hash matches
        let expected_hash = ContentVerifier::compute_hash(content);
        assert_eq!(metadata.content_hash, expected_hash);
    }

    #[tokio::test]
    async fn test_restore_from_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let original_path = temp_dir.path().join("original.txt");
        let restore_path = temp_dir.path().join("restored.txt");

        let content = "original content";
        fs::write(&original_path, content).await.unwrap();

        let manager = BackupManager::new(backup_dir, 10);
        let metadata = manager.create_backup(&original_path).await.unwrap();

        // Restore from backup
        manager
            .restore_from_backup(&metadata.backup_path, &restore_path)
            .await
            .unwrap();

        // Verify restored content matches original
        let restored_content = fs::read_to_string(&restore_path).await.unwrap();
        assert_eq!(restored_content, content);
    }

    #[tokio::test]
    async fn test_verify_backup_integrity_success() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("test.txt");

        let content = "test content";
        fs::write(&file_path, content).await.unwrap();

        let manager = BackupManager::new(backup_dir, 10);
        let metadata = manager.create_backup(&file_path).await.unwrap();

        // Verify backup integrity
        let result = manager
            .verify_backup_integrity(&metadata.backup_path, &metadata.content_hash)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_backup_integrity_corruption() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("test.txt");

        let content = "test content";
        fs::write(&file_path, content).await.unwrap();

        let manager = BackupManager::new(backup_dir, 10);
        let metadata = manager.create_backup(&file_path).await.unwrap();

        // Modify the backup file to simulate corruption
        fs::write(&metadata.backup_path, "corrupted content")
            .await
            .unwrap();

        // Verify backup integrity fails
        let result = manager
            .verify_backup_integrity(&metadata.backup_path, &metadata.content_hash)
            .await;
        assert!(result.is_err());
        match result {
            Err(FileError::BackupCorrupted) => (),
            _ => panic!("Expected BackupCorrupted error"),
        }
    }

    #[tokio::test]
    async fn test_enforce_retention_policy() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "content").await.unwrap();

        let manager = BackupManager::new(backup_dir.clone(), 3);

        // Create multiple backups
        for i in 0..5 {
            fs::write(&file_path, &format!("content {}", i))
                .await
                .unwrap();
            manager.create_backup(&file_path).await.unwrap();
            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Enforce retention policy
        manager.enforce_retention_policy(&file_path).await.unwrap();

        // Count remaining backups
        let mut entries = fs::read_dir(&backup_dir).await.unwrap();
        let mut count = 0;
        while let Some(_entry) = entries.next_entry().await.unwrap() {
            count += 1;
        }

        // Should have at most 3 backups
        assert!(count <= 3);
    }
}
