//! Property-based tests for transaction rollback functionality
//! **Feature: ricecoder-files, Property 5: Transaction Rollback**

use ricecoder_files::{
    FileOperation, OperationType, TransactionManager, TransactionStatus,
};
use tempfile::TempDir;
use tokio::fs;

/// Property 5: Transaction Rollback
/// *For any* transaction, rollback SHALL restore all files to their pre-transaction state.
/// **Validates: Requirements 2.2, 2.4**
#[tokio::test]
async fn test_rollback_restores_all_files_to_pre_transaction_state() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    // Create original files with known content
    fs::write(&file1, "original1").await.unwrap();
    fs::write(&file2, "original2").await.unwrap();

    let manager = TransactionManager::new(
        ricecoder_files::BackupManager::new(backup_dir, 10)
    );

    // Begin transaction
    let tx_id = manager.begin_transaction().await.unwrap();

    // Add operations to modify both files
    let op1 = FileOperation {
        path: file1.clone(),
        operation: OperationType::Update,
        content: Some("modified1".to_string()),
        backup_path: None,
        content_hash: Some("hash1".to_string()),
    };

    let op2 = FileOperation {
        path: file2.clone(),
        operation: OperationType::Update,
        content: Some("modified2".to_string()),
        backup_path: None,
        content_hash: Some("hash2".to_string()),
    };

    manager.add_operation(tx_id, op1).await.unwrap();
    manager.add_operation(tx_id, op2).await.unwrap();

    // Commit the transaction
    manager.commit(tx_id).await.unwrap();

    // Verify files were modified
    let content1 = fs::read_to_string(&file1).await.unwrap();
    let content2 = fs::read_to_string(&file2).await.unwrap();
    assert_eq!(content1, "modified1");
    assert_eq!(content2, "modified2");

    // Rollback the transaction
    manager.rollback(tx_id).await.unwrap();

    // Verify files were restored to pre-transaction state
    let restored1 = fs::read_to_string(&file1).await.unwrap();
    let restored2 = fs::read_to_string(&file2).await.unwrap();
    assert_eq!(restored1, "original1");
    assert_eq!(restored2, "original2");

    // Verify transaction status is RolledBack
    let status = manager.get_status(tx_id).await.unwrap();
    assert_eq!(status, TransactionStatus::RolledBack);
}

/// Property 5: Partial transaction failure triggers automatic rollback
/// *For any* transaction where one operation fails, all completed operations SHALL be rolled back.
/// **Validates: Requirements 2.2, 2.4**
#[tokio::test]
async fn test_partial_transaction_failure_triggers_rollback() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    // Create original file1
    fs::write(&file1, "original1").await.unwrap();

    let manager = TransactionManager::new(
        ricecoder_files::BackupManager::new(backup_dir, 10)
    );

    // Begin transaction
    let tx_id = manager.begin_transaction().await.unwrap();

    // Add first operation (will succeed)
    let op1 = FileOperation {
        path: file1.clone(),
        operation: OperationType::Update,
        content: Some("modified1".to_string()),
        backup_path: None,
        content_hash: Some("hash1".to_string()),
    };

    // Add second operation with invalid content (will fail)
    let op2 = FileOperation {
        path: file2.clone(),
        operation: OperationType::Update,
        content: Some("x".repeat(1_000_000_001)), // Content too large
        backup_path: None,
        content_hash: Some("hash2".to_string()),
    };

    manager.add_operation(tx_id, op1).await.unwrap();
    manager.add_operation(tx_id, op2).await.unwrap();

    // Attempt to commit - should fail
    let result = manager.commit(tx_id).await;
    assert!(result.is_err());

    // Verify file1 was restored to original state (rollback occurred)
    let content1 = fs::read_to_string(&file1).await.unwrap();
    assert_eq!(content1, "original1");

    // Verify file2 was not created
    assert!(!file2.exists());
}

/// Property test: Rollback with multiple files
/// Tests that rollback correctly restores all files in a transaction
#[tokio::test]
async fn test_rollback_with_multiple_files_property() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    let manager = TransactionManager::new(
        ricecoder_files::BackupManager::new(backup_dir, 10)
    );

    // Create 5 files with original content
    let mut files = Vec::new();
    let mut original_contents = Vec::new();

    for i in 0..5 {
        let file_path = temp_dir.path().join(format!("file{}.txt", i));
        let original_content = format!("original{}", i);
        fs::write(&file_path, &original_content).await.unwrap();
        files.push(file_path);
        original_contents.push(original_content);
    }

    // Begin transaction
    let tx_id = manager.begin_transaction().await.unwrap();

    // Add operations to modify all files
    for (i, file_path) in files.iter().enumerate() {
        let op = FileOperation {
            path: file_path.clone(),
            operation: OperationType::Update,
            content: Some(format!("modified{}", i)),
            backup_path: None,
            content_hash: Some(format!("hash{}", i)),
        };
        manager.add_operation(tx_id, op).await.unwrap();
    }

    // Commit the transaction
    manager.commit(tx_id).await.unwrap();

    // Verify all files were modified
    for (i, file_path) in files.iter().enumerate() {
        let content = fs::read_to_string(file_path).await.unwrap();
        assert_eq!(content, format!("modified{}", i));
    }

    // Rollback the transaction
    manager.rollback(tx_id).await.unwrap();

    // Verify all files were restored to original state
    for (i, file_path) in files.iter().enumerate() {
        let content = fs::read_to_string(file_path).await.unwrap();
        assert_eq!(content, original_contents[i]);
    }
}

/// Property test: Rollback idempotence
/// Rolling back the same transaction twice should not cause errors
#[tokio::test]
async fn test_rollback_idempotence() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, "original").await.unwrap();

    let manager = TransactionManager::new(
        ricecoder_files::BackupManager::new(backup_dir, 10)
    );

    let tx_id = manager.begin_transaction().await.unwrap();

    let op = FileOperation {
        path: file_path.clone(),
        operation: OperationType::Update,
        content: Some("modified".to_string()),
        backup_path: None,
        content_hash: Some("hash".to_string()),
    };

    manager.add_operation(tx_id, op).await.unwrap();
    manager.commit(tx_id).await.unwrap();

    // First rollback should succeed
    let result1 = manager.rollback(tx_id).await;
    assert!(result1.is_ok());

    // Second rollback should fail (already rolled back)
    let result2 = manager.rollback(tx_id).await;
    assert!(result2.is_err());

    // Verify file is in correct state
    let content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(content, "original");
}

/// Property test: Rollback with created files
/// Files created during a transaction should be deleted on rollback
#[tokio::test]
async fn test_rollback_deletes_created_files() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let file1 = temp_dir.path().join("existing.txt");
    let file2 = temp_dir.path().join("new.txt");

    // Create only the first file
    fs::write(&file1, "original").await.unwrap();

    let manager = TransactionManager::new(
        ricecoder_files::BackupManager::new(backup_dir, 10)
    );

    let tx_id = manager.begin_transaction().await.unwrap();

    // Add operation to modify existing file
    let op1 = FileOperation {
        path: file1.clone(),
        operation: OperationType::Update,
        content: Some("modified".to_string()),
        backup_path: None,
        content_hash: Some("hash1".to_string()),
    };

    // Add operation to create new file
    let op2 = FileOperation {
        path: file2.clone(),
        operation: OperationType::Create,
        content: Some("new content".to_string()),
        backup_path: None,
        content_hash: Some("hash2".to_string()),
    };

    manager.add_operation(tx_id, op1).await.unwrap();
    manager.add_operation(tx_id, op2).await.unwrap();

    // Commit the transaction
    manager.commit(tx_id).await.unwrap();

    // Verify both files exist
    assert!(file1.exists());
    assert!(file2.exists());

    // Rollback the transaction
    manager.rollback(tx_id).await.unwrap();

    // Verify file1 was restored and file2 was deleted
    assert!(file1.exists());
    assert!(!file2.exists());

    let content1 = fs::read_to_string(&file1).await.unwrap();
    assert_eq!(content1, "original");
}
