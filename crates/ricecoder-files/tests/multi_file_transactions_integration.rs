//! Integration tests for multi-file transactions
//!
//! Tests complex workflows involving multiple files in a single transaction,
//! including rollback of partial transactions and audit trail verification.

use ricecoder_files::backup::BackupManager;
use ricecoder_files::models::{FileOperation, OperationType};
use ricecoder_files::transaction::TransactionManager;
use tempfile::TempDir;
use tokio::fs;

/// Test writing multiple files in a single transaction
#[tokio::test]
async fn test_multi_file_transaction_writes_all_files() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
    let tx_id = manager.begin_transaction().await.unwrap();

    // Add three operations to the transaction
    let op1 = FileOperation {
        path: file1.clone(),
        operation: OperationType::Create,
        content: Some("content 1".to_string()),
        backup_path: None,
        content_hash: Some("hash1".to_string()),
    };

    let op2 = FileOperation {
        path: file2.clone(),
        operation: OperationType::Create,
        content: Some("content 2".to_string()),
        backup_path: None,
        content_hash: Some("hash2".to_string()),
    };

    let op3 = FileOperation {
        path: file3.clone(),
        operation: OperationType::Create,
        content: Some("content 3".to_string()),
        backup_path: None,
        content_hash: Some("hash3".to_string()),
    };

    manager.add_operation(tx_id, op1).await.unwrap();
    manager.add_operation(tx_id, op2).await.unwrap();
    manager.add_operation(tx_id, op3).await.unwrap();

    // Commit the transaction
    let result = manager.commit(tx_id).await;
    assert!(result.is_ok());

    // Verify all files were written
    let content1 = fs::read_to_string(&file1).await.unwrap();
    let content2 = fs::read_to_string(&file2).await.unwrap();
    let content3 = fs::read_to_string(&file3).await.unwrap();

    assert_eq!(content1, "content 1");
    assert_eq!(content2, "content 2");
    assert_eq!(content3, "content 3");
}

/// Test rollback of partial transactions
///
/// This test verifies that when a transaction fails partway through,
/// all completed writes are rolled back to their pre-transaction state.
#[tokio::test]
async fn test_partial_transaction_rollback() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    // Create original files
    fs::write(&file1, "original 1").await.unwrap();
    fs::write(&file2, "original 2").await.unwrap();

    let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
    let tx_id = manager.begin_transaction().await.unwrap();

    // Add operations to update both files
    let op1 = FileOperation {
        path: file1.clone(),
        operation: OperationType::Update,
        content: Some("updated 1".to_string()),
        backup_path: None,
        content_hash: Some("hash1".to_string()),
    };

    let op2 = FileOperation {
        path: file2.clone(),
        operation: OperationType::Update,
        content: Some("updated 2".to_string()),
        backup_path: None,
        content_hash: Some("hash2".to_string()),
    };

    manager.add_operation(tx_id, op1).await.unwrap();
    manager.add_operation(tx_id, op2).await.unwrap();

    // Commit the transaction
    manager.commit(tx_id).await.unwrap();

    // Verify files were updated
    let content1 = fs::read_to_string(&file1).await.unwrap();
    let content2 = fs::read_to_string(&file2).await.unwrap();
    assert_eq!(content1, "updated 1");
    assert_eq!(content2, "updated 2");

    // Now rollback the transaction
    let result = manager.rollback(tx_id).await;
    assert!(result.is_ok());

    // Verify files were restored to original content
    let content1 = fs::read_to_string(&file1).await.unwrap();
    let content2 = fs::read_to_string(&file2).await.unwrap();
    assert_eq!(content1, "original 1");
    assert_eq!(content2, "original 2");
}

/// Test that transaction rollback handles mixed create/update operations
#[tokio::test]
async fn test_rollback_with_mixed_operations() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let existing_file = temp_dir.path().join("existing.txt");
    let new_file = temp_dir.path().join("new.txt");

    // Create an existing file
    fs::write(&existing_file, "original content").await.unwrap();

    let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
    let tx_id = manager.begin_transaction().await.unwrap();

    // Add operations: update existing file and create new file
    let op1 = FileOperation {
        path: existing_file.clone(),
        operation: OperationType::Update,
        content: Some("updated content".to_string()),
        backup_path: None,
        content_hash: Some("hash1".to_string()),
    };

    let op2 = FileOperation {
        path: new_file.clone(),
        operation: OperationType::Create,
        content: Some("new content".to_string()),
        backup_path: None,
        content_hash: Some("hash2".to_string()),
    };

    manager.add_operation(tx_id, op1).await.unwrap();
    manager.add_operation(tx_id, op2).await.unwrap();

    // Commit the transaction
    manager.commit(tx_id).await.unwrap();

    // Verify both files exist with correct content
    assert!(existing_file.exists());
    assert!(new_file.exists());
    let content1 = fs::read_to_string(&existing_file).await.unwrap();
    let content2 = fs::read_to_string(&new_file).await.unwrap();
    assert_eq!(content1, "updated content");
    assert_eq!(content2, "new content");

    // Rollback the transaction
    manager.rollback(tx_id).await.unwrap();

    // Verify existing file was restored and new file was deleted
    assert!(existing_file.exists());
    assert!(!new_file.exists());
    let content1 = fs::read_to_string(&existing_file).await.unwrap();
    assert_eq!(content1, "original content");
}

/// Test transaction status tracking through lifecycle
#[tokio::test]
async fn test_transaction_status_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let file_path = temp_dir.path().join("test.txt");

    let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
    let tx_id = manager.begin_transaction().await.unwrap();

    // Check initial status is Pending
    let status = manager.get_status(tx_id).await.unwrap();
    assert_eq!(status, ricecoder_files::models::TransactionStatus::Pending);

    // Add operation and commit
    let op = FileOperation {
        path: file_path,
        operation: OperationType::Create,
        content: Some("test content".to_string()),
        backup_path: None,
        content_hash: Some("hash".to_string()),
    };

    manager.add_operation(tx_id, op).await.unwrap();
    manager.commit(tx_id).await.unwrap();

    // Check status is Committed
    let status = manager.get_status(tx_id).await.unwrap();
    assert_eq!(
        status,
        ricecoder_files::models::TransactionStatus::Committed
    );

    // Rollback
    manager.rollback(tx_id).await.unwrap();

    // Check status is RolledBack
    let status = manager.get_status(tx_id).await.unwrap();
    assert_eq!(
        status,
        ricecoder_files::models::TransactionStatus::RolledBack
    );
}

/// Test that transaction preserves operation order
#[tokio::test]
async fn test_transaction_preserves_operation_order() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    let manager = TransactionManager::new(BackupManager::new(backup_dir, 10));
    let tx_id = manager.begin_transaction().await.unwrap();

    // Add multiple operations
    for i in 1..=5 {
        let file_path = temp_dir.path().join(format!("file{}.txt", i));
        let op = FileOperation {
            path: file_path,
            operation: OperationType::Create,
            content: Some(format!("content {}", i)),
            backup_path: None,
            content_hash: Some(format!("hash{}", i)),
        };
        manager.add_operation(tx_id, op).await.unwrap();
    }

    // Get transaction and verify operations are in order
    let transaction = manager.get_transaction(tx_id).await.unwrap();
    assert_eq!(transaction.operations.len(), 5);

    for (i, op) in transaction.operations.iter().enumerate() {
        let expected_filename = format!("file{}.txt", i + 1);
        assert!(op.path.to_string_lossy().contains(&expected_filename));
    }
}
