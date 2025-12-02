//! Property-based tests for audit trail completeness
//! **Feature: ricecoder-files, Property 7: Audit Trail Completeness**

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_files::{AuditEntry, AuditLogger, OperationType};
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

// Property 7: Audit Trail Completeness
// For any file operation, an audit entry SHALL be created with timestamp, path, operation type, and content hash.
// For any set of audit entries, they SHALL be retrievable and ordered by timestamp.

#[test]
fn prop_all_operations_logged_with_required_fields() {
    proptest!(|(
        path_str in r"[a-zA-Z0-9_\-./]+\.txt",
        content_hash in r"[a-f0-9]{64}",
    )| {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let path = PathBuf::from(&path_str);
        let now = Utc::now();

        let entry = AuditEntry {
            timestamp: now,
            path: path.clone(),
            operation_type: OperationType::Create,
            content_hash: content_hash.clone(),
            transaction_id: None,
        };

        // Log the operation
        logger.log_operation(entry.clone()).unwrap();

        // Retrieve the history
        let history = logger.get_change_history(&path).unwrap();

        // Verify the entry was logged
        prop_assert_eq!(history.len(), 1);
        prop_assert_eq!(&history[0].path, &path);
        prop_assert_eq!(&history[0].content_hash, &content_hash);
        prop_assert_eq!(history[0].timestamp, now);
    });
}

#[test]
fn prop_audit_entries_ordered_by_timestamp() {
    proptest!(|(
        path_str in r"[a-zA-Z0-9_\-./]+\.txt",
        num_entries in 2..10usize,
    )| {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let path = PathBuf::from(&path_str);
        let mut base_time = Utc::now();

        // Create multiple entries with increasing timestamps
        for i in 0..num_entries {
            let entry = AuditEntry {
                timestamp: base_time,
                path: path.clone(),
                operation_type: OperationType::Update,
                content_hash: format!("{:064x}", i),
                transaction_id: None,
            };

            logger.log_operation(entry).unwrap();
            base_time = base_time + chrono::Duration::milliseconds(1);
        }

        // Retrieve the history
        let history = logger.get_change_history(&path).unwrap();

        // Verify all entries were logged
        prop_assert_eq!(history.len(), num_entries);

        // Verify entries are ordered by timestamp
        for i in 1..history.len() {
            prop_assert!(history[i - 1].timestamp <= history[i].timestamp);
        }
    });
}

#[test]
fn prop_audit_entries_contain_all_required_fields() {
    proptest!(|(
        path_str in r"[a-zA-Z0-9_\-./]+\.txt",
        content_hash in r"[a-f0-9]{64}",
    )| {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let path = PathBuf::from(&path_str);
        let now = Utc::now();
        let tx_id = Uuid::new_v4();

        let entry = AuditEntry {
            timestamp: now,
            path: path.clone(),
            operation_type: OperationType::Update,
            content_hash: content_hash.clone(),
            transaction_id: Some(tx_id),
        };

        logger.log_operation(entry).unwrap();

        let history = logger.get_change_history(&path).unwrap();
        prop_assert_eq!(history.len(), 1);

        let retrieved = &history[0];
        // Verify all required fields are present
        prop_assert_eq!(&retrieved.path, &path);
        prop_assert_eq!(&retrieved.content_hash, &content_hash);
        prop_assert_eq!(retrieved.timestamp, now);
        prop_assert_eq!(retrieved.transaction_id, Some(tx_id));
    });
}

#[test]
fn prop_multiple_files_tracked_independently() {
    proptest!(|(
        path1_str in r"[a-zA-Z0-9_\-./]+\.txt",
        path2_str in r"[a-zA-Z0-9_\-./]+\.txt",
    )| {
        prop_assume!(path1_str != path2_str);

        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let path1 = PathBuf::from(&path1_str);
        let path2 = PathBuf::from(&path2_str);

        let entry1 = AuditEntry {
            timestamp: Utc::now(),
            path: path1.clone(),
            operation_type: OperationType::Create,
            content_hash: "hash1".to_string(),
            transaction_id: None,
        };

        let entry2 = AuditEntry {
            timestamp: Utc::now(),
            path: path2.clone(),
            operation_type: OperationType::Create,
            content_hash: "hash2".to_string(),
            transaction_id: None,
        };

        logger.log_operation(entry1).unwrap();
        logger.log_operation(entry2).unwrap();

        let history1 = logger.get_change_history(&path1).unwrap();
        let history2 = logger.get_change_history(&path2).unwrap();

        // Each file should have exactly one entry
        prop_assert_eq!(history1.len(), 1);
        prop_assert_eq!(history2.len(), 1);

        // Entries should be for the correct files
        prop_assert_eq!(&history1[0].path, &path1);
        prop_assert_eq!(&history2[0].path, &path2);
    });
}

// **Validates: Requirements 3.1**
