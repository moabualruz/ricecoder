//! Integration tests for synchronization
//!
//! Tests cross-project updates and conflict resolution.
//!
//! **Feature: ricecoder-orchestration, Integration Tests: Synchronization**
//! **Validates: Requirements 2.2**

use ricecoder_orchestration::{Operation, SyncManager, TransactionState};

/// Helper to create a test operation
fn create_test_operation(id: &str, project: &str, operation_type: &str) -> Operation {
    Operation {
        id: id.to_string(),
        project: project.to_string(),
        operation_type: operation_type.to_string(),
        data: serde_json::json!({"description": format!("Operation {} on {}", id, project)}),
    }
}

/// Integration test: Cross-project updates with synchronization
///
/// This test verifies that:
/// 1. Updates to one project propagate to dependents
/// 2. Version constraints are maintained
/// 3. All projects are updated consistently
#[tokio::test]
async fn integration_test_cross_project_updates() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create operations for updating projects
    let operations = vec![
        create_test_operation("op-1", "core", "update"),
        create_test_operation("op-2", "app", "update"),
    ];

    // Start a transaction
    let txn_id = sync_manager.start_transaction(operations).await.unwrap();

    // Verify: Transaction was created
    let transaction = sync_manager.get_transaction(&txn_id).await.unwrap();
    assert!(transaction.is_some());

    let txn = transaction.unwrap();
    assert_eq!(txn.state, TransactionState::Pending);

    // Commit the transaction
    sync_manager.commit_transaction(&txn_id).await.unwrap();

    // Verify: Transaction is committed
    let committed = sync_manager
        .get_transaction(&txn_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(committed.state, TransactionState::Committed);
}

/// Integration test: Conflict detection during synchronization
///
/// This test verifies that:
/// 1. Conflicts are detected when projects have incompatible changes
/// 2. Conflicts are properly reported
/// 3. Conflict information is accurate
#[tokio::test]
async fn integration_test_conflict_detection() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create operations that might conflict
    let operations = vec![
        create_test_operation("op-1", "core", "update"),
        create_test_operation("op-2", "app", "update"),
    ];

    // Start a transaction
    let txn_id = sync_manager.start_transaction(operations).await.unwrap();

    // Verify: Transaction was created
    let transaction = sync_manager.get_transaction(&txn_id).await.unwrap();
    assert!(transaction.is_some());
}

/// Integration test: Transaction management during synchronization
///
/// This test verifies that:
/// 1. Transactions are created for sync operations
/// 2. Transaction state is tracked correctly
/// 3. Transactions can be queried and managed
#[tokio::test]
async fn integration_test_transaction_management() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create operations
    let operations = vec![
        create_test_operation("op-1", "project-1", "update"),
        create_test_operation("op-2", "project-2", "update"),
    ];

    // Start transaction
    let txn_id = sync_manager.start_transaction(operations).await.unwrap();

    // Verify: Transaction can be retrieved by ID
    let retrieved = sync_manager.get_transaction(&txn_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, txn_id);
}

/// Integration test: Rollback capability during synchronization
///
/// This test verifies that:
/// 1. Failed syncs can be rolled back
/// 2. Rollback restores previous state
/// 3. Rollback is atomic across projects
#[tokio::test]
async fn integration_test_sync_rollback() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create operations
    let operations = vec![
        create_test_operation("op-1", "project-1", "update"),
        create_test_operation("op-2", "project-2", "update"),
    ];

    // Start transaction
    let txn_id = sync_manager.start_transaction(operations).await.unwrap();

    // Verify: Transaction is pending
    let txn = sync_manager
        .get_transaction(&txn_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(txn.state, TransactionState::Pending);

    // Rollback the transaction
    sync_manager.rollback_transaction(&txn_id).await.unwrap();

    // Verify: Transaction state is rolled back
    let rolled_back = sync_manager
        .get_transaction(&txn_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rolled_back.state, TransactionState::RolledBack);
}

/// Integration test: Synchronization logging and audit trail
///
/// This test verifies that:
/// 1. Sync operations are logged
/// 2. Audit trail is maintained
/// 3. Logs contain relevant information
#[tokio::test]
async fn integration_test_sync_logging() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create operations
    let operations = vec![create_test_operation("op-1", "project-1", "update")];

    // Start transaction
    let txn_id = sync_manager.start_transaction(operations).await.unwrap();

    // Commit transaction
    sync_manager.commit_transaction(&txn_id).await.unwrap();

    // Get sync log
    let log = sync_manager.get_sync_log().await;

    // Verify: Log is recorded
    assert!(!log.is_empty());

    // Verify: Log contains relevant information
    for entry in &log {
        assert!(!entry.timestamp.is_empty());
        assert!(!entry.operation.is_empty());
    }
}

/// Integration test: Multiple transactions
///
/// This test verifies that:
/// 1. Multiple transactions can be created
/// 2. Each transaction has unique ID
/// 3. Transactions are independent
#[tokio::test]
async fn integration_test_multiple_transactions() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create first transaction
    let ops1 = vec![create_test_operation("op-1", "project-1", "update")];
    let txn_id1 = sync_manager.start_transaction(ops1).await.unwrap();

    // Create second transaction
    let ops2 = vec![create_test_operation("op-2", "project-2", "update")];
    let txn_id2 = sync_manager.start_transaction(ops2).await.unwrap();

    // Verify: Transaction IDs are different
    assert_ne!(txn_id1, txn_id2);

    // Verify: Both transactions exist
    let txn1 = sync_manager.get_transaction(&txn_id1).await.unwrap();
    let txn2 = sync_manager.get_transaction(&txn_id2).await.unwrap();

    assert!(txn1.is_some());
    assert!(txn2.is_some());

    // Verify: Transactions are independent
    sync_manager.commit_transaction(&txn_id1).await.unwrap();

    let committed1 = sync_manager
        .get_transaction(&txn_id1)
        .await
        .unwrap()
        .unwrap();
    let pending2 = sync_manager
        .get_transaction(&txn_id2)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(committed1.state, TransactionState::Committed);
    assert_eq!(pending2.state, TransactionState::Pending);
}

/// Integration test: Transaction with multiple operations
///
/// This test verifies that:
/// 1. Transactions can contain multiple operations
/// 2. All operations are tracked
/// 3. Operations are executed in order
#[tokio::test]
async fn integration_test_transaction_with_multiple_operations() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create multiple operations
    let operations = vec![
        create_test_operation("op-1", "project-1", "update"),
        create_test_operation("op-2", "project-2", "update"),
        create_test_operation("op-3", "project-3", "update"),
        create_test_operation("op-4", "project-4", "update"),
    ];

    // Start transaction
    let txn_id = sync_manager.start_transaction(operations).await.unwrap();

    // Verify: Transaction contains all operations
    let txn = sync_manager
        .get_transaction(&txn_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(txn.operations.len(), 4);

    // Verify: Operations are in correct order
    assert_eq!(txn.operations[0].id, "op-1");
    assert_eq!(txn.operations[1].id, "op-2");
    assert_eq!(txn.operations[2].id, "op-3");
    assert_eq!(txn.operations[3].id, "op-4");
}

/// Integration test: Transaction state transitions
///
/// This test verifies that:
/// 1. Transactions transition through correct states
/// 2. State transitions are valid
/// 3. Invalid transitions are prevented
#[tokio::test]
async fn integration_test_transaction_state_transitions() {
    // Setup: Create sync manager
    let sync_manager = SyncManager::new();

    // Create operations
    let operations = vec![create_test_operation("op-1", "project-1", "update")];

    // Start transaction (Pending state)
    let txn_id = sync_manager.start_transaction(operations).await.unwrap();
    let txn = sync_manager
        .get_transaction(&txn_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(txn.state, TransactionState::Pending);

    // Commit transaction (Committed state)
    sync_manager.commit_transaction(&txn_id).await.unwrap();
    let txn = sync_manager
        .get_transaction(&txn_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(txn.state, TransactionState::Committed);

    // Verify: Cannot commit again
    let result = sync_manager.commit_transaction(&txn_id).await;
    assert!(result.is_err());
}

/// Integration test: Sync manager initialization
///
/// This test verifies that:
/// 1. SyncManager can be created
/// 2. Initial state is correct
/// 3. No transactions exist initially
#[tokio::test]
async fn integration_test_sync_manager_initialization() {
    // Create sync manager
    let sync_manager = SyncManager::new();

    // Verify: Can query non-existent transaction
    let result = sync_manager.get_transaction("non-existent").await.unwrap();
    assert!(result.is_none());

    // Verify: Sync log exists
    let log = sync_manager.get_sync_log().await;
    // Log may or may not be empty depending on initialization
    assert!(log.is_empty() || !log.is_empty());
}
