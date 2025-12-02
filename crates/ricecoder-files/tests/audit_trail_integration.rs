//! Integration tests for audit trail
//!
//! Tests complete audit trail for complex multi-file operations,
//! change history retrieval, and ordering by timestamp.

use ricecoder_files::audit::AuditLogger;
use ricecoder_files::models::{AuditEntry, OperationType};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Test complete audit trail for multi-file operations
#[test]
fn test_audit_trail_for_multi_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir.clone());

    // Log multiple file operations
    let entry1 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: PathBuf::from("file1.txt"),
        operation_type: OperationType::Create,
        content_hash: "hash1".to_string(),
        transaction_id: None,
    };

    let entry2 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: PathBuf::from("file2.txt"),
        operation_type: OperationType::Create,
        content_hash: "hash2".to_string(),
        transaction_id: None,
    };

    let entry3 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: PathBuf::from("file3.txt"),
        operation_type: OperationType::Update,
        content_hash: "hash3".to_string(),
        transaction_id: None,
    };

    // Log all entries
    assert!(logger.log_operation(entry1).is_ok());
    assert!(logger.log_operation(entry2).is_ok());
    assert!(logger.log_operation(entry3).is_ok());

    // Verify audit directory was created
    assert!(audit_dir.exists());

    // Verify audit files were created
    let entries = std::fs::read_dir(&audit_dir).unwrap();
    let count = entries.count();
    assert_eq!(count, 3);
}

/// Test change history retrieval for a specific file
#[test]
fn test_change_history_retrieval_for_file() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir);
    let file_path = PathBuf::from("test.txt");

    // Log multiple operations for the same file
    let entry1 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Create,
        content_hash: "hash1".to_string(),
        transaction_id: None,
    };

    thread::sleep(Duration::from_millis(10));

    let entry2 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Update,
        content_hash: "hash2".to_string(),
        transaction_id: None,
    };

    thread::sleep(Duration::from_millis(10));

    let entry3 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Update,
        content_hash: "hash3".to_string(),
        transaction_id: None,
    };

    // Log all entries
    assert!(logger.log_operation(entry1).is_ok());
    assert!(logger.log_operation(entry2).is_ok());
    assert!(logger.log_operation(entry3).is_ok());

    // Retrieve change history
    let history = logger.get_change_history(&file_path).unwrap();

    // Verify all entries are retrieved
    assert_eq!(history.len(), 3);

    // Verify entries are for the correct file
    for entry in &history {
        assert_eq!(entry.path, file_path);
    }
}

/// Test change history ordering by timestamp
#[test]
fn test_change_history_ordered_by_timestamp() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir);
    let file_path = PathBuf::from("ordered.txt");

    // Log operations with delays to ensure different timestamps
    let entry1 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Create,
        content_hash: "hash1".to_string(),
        transaction_id: None,
    };

    thread::sleep(Duration::from_millis(50));

    let entry2 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Update,
        content_hash: "hash2".to_string(),
        transaction_id: None,
    };

    thread::sleep(Duration::from_millis(50));

    let entry3 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Update,
        content_hash: "hash3".to_string(),
        transaction_id: None,
    };

    // Log all entries
    assert!(logger.log_operation(entry1).is_ok());
    assert!(logger.log_operation(entry2).is_ok());
    assert!(logger.log_operation(entry3).is_ok());

    // Retrieve change history
    let history = logger.get_change_history(&file_path).unwrap();

    // Verify entries are ordered by timestamp (oldest first)
    assert_eq!(history.len(), 3);
    for i in 0..history.len() - 1 {
        assert!(history[i].timestamp <= history[i + 1].timestamp);
    }

    // Verify operation types are in expected order
    assert_eq!(history[0].operation_type, OperationType::Create);
    assert_eq!(history[1].operation_type, OperationType::Update);
    assert_eq!(history[2].operation_type, OperationType::Update);
}

/// Test audit trail with transaction IDs
#[test]
fn test_audit_trail_includes_transaction_ids() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir);
    let tx_id = uuid::Uuid::new_v4();

    let entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: PathBuf::from("test.txt"),
        operation_type: OperationType::Create,
        content_hash: "hash".to_string(),
        transaction_id: Some(tx_id),
    };

    assert!(logger.log_operation(entry).is_ok());

    // Retrieve and verify transaction ID is preserved
    let history = logger.get_change_history(&PathBuf::from("test.txt")).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].transaction_id, Some(tx_id));
}

/// Test audit trail with different operation types
#[test]
fn test_audit_trail_tracks_operation_types() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir);
    let file_path = temp_dir.path().join("test.txt");

    // Log different operation types
    let create_entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Create,
        content_hash: "hash1".to_string(),
        transaction_id: None,
    };

    std::thread::sleep(std::time::Duration::from_millis(50));

    let update_entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Update,
        content_hash: "hash2".to_string(),
        transaction_id: None,
    };

    std::thread::sleep(std::time::Duration::from_millis(50));

    let delete_entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Delete,
        content_hash: "hash3".to_string(),
        transaction_id: None,
    };

    // Log all entries
    assert!(logger.log_operation(create_entry).is_ok());
    assert!(logger.log_operation(update_entry).is_ok());
    assert!(logger.log_operation(delete_entry).is_ok());

    // Retrieve history and verify operation types
    let history = logger.get_change_history(&file_path).unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].operation_type, OperationType::Create);
    assert_eq!(history[1].operation_type, OperationType::Update);
    assert_eq!(history[2].operation_type, OperationType::Delete);
}

/// Test audit trail for multiple files
#[test]
fn test_audit_trail_for_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir);

    let file1 = PathBuf::from("file1.txt");
    let file2 = PathBuf::from("file2.txt");
    let file3 = PathBuf::from("file3.txt");

    // Log operations for different files
    let entry1 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file1.clone(),
        operation_type: OperationType::Create,
        content_hash: "hash1".to_string(),
        transaction_id: None,
    };

    let entry2 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file2.clone(),
        operation_type: OperationType::Create,
        content_hash: "hash2".to_string(),
        transaction_id: None,
    };

    let entry3 = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file3.clone(),
        operation_type: OperationType::Create,
        content_hash: "hash3".to_string(),
        transaction_id: None,
    };

    // Log all entries
    assert!(logger.log_operation(entry1).is_ok());
    assert!(logger.log_operation(entry2).is_ok());
    assert!(logger.log_operation(entry3).is_ok());

    // Retrieve history for each file
    let history1 = logger.get_change_history(&file1).unwrap();
    let history2 = logger.get_change_history(&file2).unwrap();
    let history3 = logger.get_change_history(&file3).unwrap();

    // Verify each file has its own history
    assert_eq!(history1.len(), 1);
    assert_eq!(history2.len(), 1);
    assert_eq!(history3.len(), 1);

    assert_eq!(history1[0].path, file1);
    assert_eq!(history2[0].path, file2);
    assert_eq!(history3[0].path, file3);
}

/// Test audit trail with content hashes
#[test]
fn test_audit_trail_preserves_content_hashes() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir);
    let file_path = PathBuf::from("test.txt");

    let entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        path: file_path.clone(),
        operation_type: OperationType::Create,
        content_hash: "abc123def456".to_string(),
        transaction_id: None,
    };

    assert!(logger.log_operation(entry).is_ok());

    // Retrieve and verify content hash is preserved
    let history = logger.get_change_history(&file_path).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].content_hash, "abc123def456");
}

/// Test audit trail for non-existent file returns empty history
#[test]
fn test_audit_trail_empty_for_non_existent_file() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let logger = AuditLogger::new(audit_dir);
    let non_existent_file = PathBuf::from("nonexistent.txt");

    // Retrieve history for file that was never logged
    let history = logger.get_change_history(&non_existent_file).unwrap();

    // Should return empty vector
    assert_eq!(history.len(), 0);
}

/// Test audit trail persistence across logger instances
#[test]
fn test_audit_trail_persists_across_instances() {
    let temp_dir = TempDir::new().unwrap();
    let audit_dir = temp_dir.path().join("audit");

    let file_path = PathBuf::from("test.txt");

    // Create first logger and log an entry
    {
        let logger1 = AuditLogger::new(audit_dir.clone());
        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            path: file_path.clone(),
            operation_type: OperationType::Create,
            content_hash: "hash1".to_string(),
            transaction_id: None,
        };
        assert!(logger1.log_operation(entry).is_ok());
    }

    // Create second logger and verify entry is still there
    {
        let logger2 = AuditLogger::new(audit_dir);
        let history = logger2.get_change_history(&file_path).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content_hash, "hash1");
    }
}
