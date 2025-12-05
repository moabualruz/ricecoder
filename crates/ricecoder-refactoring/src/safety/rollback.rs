//! Rollback handling for refactoring operations

use crate::error::{RefactoringError, Result};
use crate::types::BackupInfo;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Handles rollback of refactoring operations
pub struct RollbackHandler;

impl RollbackHandler {
    /// Create a backup of files before refactoring
    pub fn create_backup(files: &[(PathBuf, String)]) -> Result<BackupInfo> {
        let mut backup_files = HashMap::new();

        for (path, content) in files {
            backup_files.insert(path.clone(), content.clone());
        }

        Ok(BackupInfo {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            files: backup_files,
        })
    }

    /// Restore files from a backup
    pub fn restore_from_backup(backup: &BackupInfo) -> Result<()> {
        for (path, content) in &backup.files {
            std::fs::write(path, content).map_err(|e| {
                RefactoringError::RollbackFailed(format!("Failed to restore {}: {}", path.display(), e))
            })?;
        }

        Ok(())
    }

    /// Verify backup integrity
    pub fn verify_backup(backup: &BackupInfo) -> Result<bool> {
        if backup.files.is_empty() {
            return Err(RefactoringError::RollbackFailed(
                "Backup contains no files".to_string(),
            ));
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_create_backup() -> Result<()> {
        let files = vec![
            (PathBuf::from("file1.rs"), "content1".to_string()),
            (PathBuf::from("file2.rs"), "content2".to_string()),
        ];

        let backup = RollbackHandler::create_backup(&files)?;
        assert_eq!(backup.files.len(), 2);
        assert!(!backup.id.is_empty());

        Ok(())
    }

    #[test]
    fn test_verify_backup() -> Result<()> {
        let files = vec![(PathBuf::from("file1.rs"), "content1".to_string())];
        let backup = RollbackHandler::create_backup(&files)?;

        assert!(RollbackHandler::verify_backup(&backup)?);

        Ok(())
    }

    #[test]
    fn test_verify_backup_empty() -> Result<()> {
        let backup = BackupInfo {
            id: "test".to_string(),
            timestamp: Utc::now(),
            files: HashMap::new(),
        };

        assert!(RollbackHandler::verify_backup(&backup).is_err());

        Ok(())
    }

    #[test]
    fn test_restore_from_backup() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"original")?;
        file.flush()?;

        let path = file.path().to_path_buf();
        let mut backup_files = HashMap::new();
        backup_files.insert(path.clone(), "restored".to_string());

        let backup = BackupInfo {
            id: "test".to_string(),
            timestamp: Utc::now(),
            files: backup_files,
        };

        RollbackHandler::restore_from_backup(&backup)?;

        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, "restored");

        Ok(())
    }
}
