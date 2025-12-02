//! Transaction management with rollback support

use crate::backup::BackupManager;
use crate::error::FileError;
use crate::models::{FileOperation, FileTransaction, TransactionStatus};
use crate::writer::SafeWriter;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Manages atomic multi-file operations with all-or-nothing semantics
#[derive(Debug, Clone)]
pub struct TransactionManager {
    transactions: Arc<RwLock<HashMap<Uuid, FileTransaction>>>,
    writer: SafeWriter,
    backup_manager: BackupManager,
}

impl TransactionManager {
    /// Creates a new TransactionManager instance
    ///
    /// # Arguments
    ///
    /// * `backup_manager` - BackupManager for handling backups during rollback
    pub fn new(backup_manager: BackupManager) -> Self {
        TransactionManager {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            writer: SafeWriter::new(),
            backup_manager,
        }
    }

    /// Begins a new transaction with a unique ID
    ///
    /// # Returns
    ///
    /// A unique transaction ID, or an error
    pub async fn begin_transaction(&self) -> Result<Uuid, FileError> {
        let tx_id = Uuid::new_v4();
        let transaction = FileTransaction {
            id: tx_id,
            operations: Vec::new(),
            status: TransactionStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
        };

        let mut transactions = self.transactions.write().await;
        transactions.insert(tx_id, transaction);

        Ok(tx_id)
    }

    /// Adds an operation to a transaction
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID
    /// * `op` - FileOperation to add
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn add_operation(&self, tx_id: Uuid, op: FileOperation) -> Result<(), FileError> {
        let mut transactions = self.transactions.write().await;

        let transaction = transactions
            .get_mut(&tx_id)
            .ok_or_else(|| FileError::TransactionFailed("Transaction not found".to_string()))?;

        if transaction.status != TransactionStatus::Pending {
            return Err(FileError::TransactionFailed(
                "Cannot add operations to non-pending transaction".to_string(),
            ));
        }

        transaction.operations.push(op);
        Ok(())
    }

    /// Commits a transaction by executing all operations
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn commit(&self, tx_id: Uuid) -> Result<(), FileError> {
        let mut transactions = self.transactions.write().await;

        let transaction = transactions
            .get_mut(&tx_id)
            .ok_or_else(|| FileError::TransactionFailed("Transaction not found".to_string()))?;

        if transaction.status != TransactionStatus::Pending {
            return Err(FileError::TransactionFailed(
                "Transaction is not pending".to_string(),
            ));
        }

        // Create backups for all files that will be modified (for rollback)
        let mut pre_transaction_backups: HashMap<std::path::PathBuf, Option<std::path::PathBuf>> = HashMap::new();

        for op in &transaction.operations {
            if op.path.exists() {
                let backup_metadata = self.backup_manager.create_backup(&op.path).await?;
                pre_transaction_backups.insert(op.path.clone(), Some(backup_metadata.backup_path));
            } else {
                // File doesn't exist, so we need to delete it on rollback
                pre_transaction_backups.insert(op.path.clone(), None);
            }
        }

        // Store backups in transaction for rollback
        transaction.operations.iter_mut().for_each(|op| {
            if let Some(backup_opt) = pre_transaction_backups.get(&op.path) {
                op.backup_path = backup_opt.clone();
            }
        });

        // Execute all operations
        let mut executed_count = 0;
        for op in &transaction.operations {
            match self
                .writer
                .write(&op.path, op.content.as_ref().unwrap_or(&String::new()), crate::models::ConflictResolution::Overwrite)
                .await
            {
                Ok(_) => {
                    executed_count += 1;
                }
                Err(e) => {
                    // Rollback all completed operations
                    self.rollback_operations(&pre_transaction_backups, executed_count, &transaction.operations)
                        .await?;

                    return Err(FileError::TransactionFailed(format!(
                        "Operation failed after {} successful operations: {}",
                        executed_count, e
                    )));
                }
            }
        }

        // Mark transaction as committed
        transaction.status = TransactionStatus::Committed;
        transaction.completed_at = Some(Utc::now());

        // Enforce retention policy for all modified files
        for op in &transaction.operations {
            let _ = self.backup_manager.enforce_retention_policy(&op.path).await;
        }

        Ok(())
    }

    /// Rolls back a transaction to restore pre-transaction state
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn rollback(&self, tx_id: Uuid) -> Result<(), FileError> {
        let mut transactions = self.transactions.write().await;

        let transaction = transactions
            .get_mut(&tx_id)
            .ok_or_else(|| FileError::TransactionFailed("Transaction not found".to_string()))?;

        if transaction.status == TransactionStatus::RolledBack {
            return Err(FileError::TransactionFailed(
                "Transaction already rolled back".to_string(),
            ));
        }

        // Restore all files from their pre-transaction backups
        for op in &transaction.operations {
            if let Some(backup_path_opt) = &op.backup_path {
                // File existed before transaction, restore from backup
                self.backup_manager
                    .restore_from_backup(backup_path_opt, &op.path)
                    .await?;
            } else if op.path.exists() {
                // File didn't exist before transaction, delete it
                fs::remove_file(&op.path)
                    .await
                    .map_err(|e| FileError::RollbackFailed(format!(
                        "Failed to delete file during rollback: {}",
                        e
                    )))?;
            }
        }

        // Mark transaction as rolled back
        transaction.status = TransactionStatus::RolledBack;
        transaction.completed_at = Some(Utc::now());

        Ok(())
    }

    /// Gets the status of a transaction
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID
    ///
    /// # Returns
    ///
    /// Transaction status, or an error if not found
    pub async fn get_status(&self, tx_id: Uuid) -> Result<TransactionStatus, FileError> {
        let transactions = self.transactions.read().await;

        transactions
            .get(&tx_id)
            .map(|t| t.status)
            .ok_or_else(|| FileError::TransactionFailed("Transaction not found".to_string()))
    }

    /// Gets a transaction by ID
    ///
    /// # Arguments
    ///
    /// * `tx_id` - Transaction ID
    ///
    /// # Returns
    ///
    /// Transaction, or an error if not found
    pub async fn get_transaction(&self, tx_id: Uuid) -> Result<FileTransaction, FileError> {
        let transactions = self.transactions.read().await;

        transactions
            .get(&tx_id)
            .cloned()
            .ok_or_else(|| FileError::TransactionFailed("Transaction not found".to_string()))
    }

    /// Helper function to rollback operations after a failure
    async fn rollback_operations(
        &self,
        backups: &HashMap<std::path::PathBuf, Option<std::path::PathBuf>>,
        executed_count: usize,
        operations: &[FileOperation],
    ) -> Result<(), FileError> {
        // Restore the first executed_count operations from their backups
        for op in operations.iter().take(executed_count) {
            if let Some(backup_opt) = backups.get(&op.path) {
                if let Some(backup_path) = backup_opt {
                    self.backup_manager
                        .restore_from_backup(backup_path, &op.path)
                        .await
                        .map_err(|e| FileError::RollbackFailed(format!(
                            "Failed to restore backup during rollback: {}",
                            e
                        )))?;
                } else if op.path.exists() {
                    // File didn't exist before, delete it
                    fs::remove_file(&op.path)
                        .await
                        .map_err(|e| FileError::RollbackFailed(format!(
                            "Failed to delete file during rollback: {}",
                            e
                        )))?;
                }
            }
        }

        Ok(())
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new(BackupManager::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_begin_transaction() {
        let manager = TransactionManager::default();
        let tx_id = manager.begin_transaction().await.unwrap();

        let status = manager.get_status(tx_id).await.unwrap();
        assert_eq!(status, TransactionStatus::Pending);
    }

    #[tokio::test]
    async fn test_add_operation() {
        let manager = TransactionManager::default();
        let tx_id = manager.begin_transaction().await.unwrap();

        let op = FileOperation {
            path: std::path::PathBuf::from("test.txt"),
            operation: crate::models::OperationType::Create,
            content: Some("test content".to_string()),
            backup_path: None,
            content_hash: Some("hash".to_string()),
        };

        let result = manager.add_operation(tx_id, op).await;
        assert!(result.is_ok());

        let transaction = manager.get_transaction(tx_id).await.unwrap();
        assert_eq!(transaction.operations.len(), 1);
    }

    #[tokio::test]
    async fn test_add_operation_to_non_pending_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));

        let tx_id = manager.begin_transaction().await.unwrap();

        // Manually mark as committed to test error case
        {
            let mut transactions = manager.transactions.write().await;
            if let Some(tx) = transactions.get_mut(&tx_id) {
                tx.status = TransactionStatus::Committed;
            }
        }

        let op = FileOperation {
            path: std::path::PathBuf::from("test.txt"),
            operation: crate::models::OperationType::Create,
            content: Some("test content".to_string()),
            backup_path: None,
            content_hash: Some("hash".to_string()),
        };

        let result = manager.add_operation(tx_id, op).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_commit_single_operation() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("test.txt");

        let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
        let tx_id = manager.begin_transaction().await.unwrap();

        let op = FileOperation {
            path: file_path.clone(),
            operation: crate::models::OperationType::Create,
            content: Some("test content".to_string()),
            backup_path: None,
            content_hash: Some("hash".to_string()),
        };

        manager.add_operation(tx_id, op).await.unwrap();
        let result = manager.commit(tx_id).await;

        assert!(result.is_ok());
        let status = manager.get_status(tx_id).await.unwrap();
        assert_eq!(status, TransactionStatus::Committed);

        // Verify file was written
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_commit_multiple_operations() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
        let tx_id = manager.begin_transaction().await.unwrap();

        let op1 = FileOperation {
            path: file1.clone(),
            operation: crate::models::OperationType::Create,
            content: Some("content 1".to_string()),
            backup_path: None,
            content_hash: Some("hash1".to_string()),
        };

        let op2 = FileOperation {
            path: file2.clone(),
            operation: crate::models::OperationType::Create,
            content: Some("content 2".to_string()),
            backup_path: None,
            content_hash: Some("hash2".to_string()),
        };

        manager.add_operation(tx_id, op1).await.unwrap();
        manager.add_operation(tx_id, op2).await.unwrap();

        let result = manager.commit(tx_id).await;
        assert!(result.is_ok());

        // Verify both files were written
        let content1 = fs::read_to_string(&file1).await.unwrap();
        let content2 = fs::read_to_string(&file2).await.unwrap();
        assert_eq!(content1, "content 1");
        assert_eq!(content2, "content 2");
    }

    #[tokio::test]
    async fn test_rollback_restores_files() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("test.txt");

        // Create original file
        fs::write(&file_path, "original content")
            .await
            .unwrap();

        let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
        let tx_id = manager.begin_transaction().await.unwrap();

        let op = FileOperation {
            path: file_path.clone(),
            operation: crate::models::OperationType::Update,
            content: Some("new content".to_string()),
            backup_path: None,
            content_hash: Some("hash".to_string()),
        };

        manager.add_operation(tx_id, op).await.unwrap();
        manager.commit(tx_id).await.unwrap();

        // Verify file was updated
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "new content");

        // Rollback
        let result = manager.rollback(tx_id).await;
        assert!(result.is_ok());

        // Verify file was restored
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "original content");
    }

    #[tokio::test]
    async fn test_rollback_deletes_created_files() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let file_path = temp_dir.path().join("new_file.txt");

        let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
        let tx_id = manager.begin_transaction().await.unwrap();

        let op = FileOperation {
            path: file_path.clone(),
            operation: crate::models::OperationType::Create,
            content: Some("new content".to_string()),
            backup_path: None,
            content_hash: Some("hash".to_string()),
        };

        manager.add_operation(tx_id, op).await.unwrap();
        manager.commit(tx_id).await.unwrap();

        // Verify file was created
        assert!(file_path.exists());

        // Rollback
        let result = manager.rollback(tx_id).await;
        assert!(result.is_ok());

        // Verify file was deleted
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_get_transaction() {
        let manager = TransactionManager::default();
        let tx_id = manager.begin_transaction().await.unwrap();

        let transaction = manager.get_transaction(tx_id).await.unwrap();
        assert_eq!(transaction.id, tx_id);
        assert_eq!(transaction.status, TransactionStatus::Pending);
        assert_eq!(transaction.operations.len(), 0);
    }

    #[tokio::test]
    async fn test_get_nonexistent_transaction() {
        let manager = TransactionManager::default();
        let fake_id = Uuid::new_v4();

        let result = manager.get_transaction(fake_id).await;
        assert!(result.is_err());
    }
}
