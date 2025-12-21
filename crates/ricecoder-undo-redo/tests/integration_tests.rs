//! Integration tests for end-to-end workflows

#[cfg(test)]
mod integration_tests {
    use crate::{
        Change, ChangeTracker, ChangeType, Checkpoint, CheckpointManager, HistoryManager,
        HistorySnapshot, HistoryStore, StorageManager,
    };
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Test complete undo/redo workflow (track → undo → redo)
    #[test]
    fn test_complete_undo_redo_workflow() {
        let mut manager = HistoryManager::new();

        // Track changes
        let change1 = Change::new(
            "file1.txt",
            "",
            "initial content",
            "Create file1",
            ChangeType::Create,
        )
        .unwrap();

        let change2 = Change::new(
            "file1.txt",
            "initial content",
            "modified content",
            "Modify file1",
            ChangeType::Modify,
        )
        .unwrap();

        // Record changes in history
        manager.record_change(change1.clone()).unwrap();
        manager.record_change(change2.clone()).unwrap();

        // Verify we can undo
        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // Perform undo
        let undone1 = manager.undo().unwrap();
        assert_eq!(undone1.id, change2.id);
        assert!(manager.can_undo());
        assert!(manager.can_redo());

        // Perform another undo
        let undone2 = manager.undo().unwrap();
        assert_eq!(undone2.id, change1.id);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Perform redo
        let redone1 = manager.redo().unwrap();
        assert_eq!(redone1.id, change1.id);
        assert!(manager.can_undo());
        assert!(manager.can_redo());

        // Perform another redo
        let redone2 = manager.redo().unwrap();
        assert_eq!(redone2.id, change2.id);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // Verify total changes
        assert_eq!(manager.total_changes(), 2);
    }

    /// Test checkpoint creation and rollback workflow
    #[test]
    fn test_checkpoint_creation_and_rollback_workflow() {
        let mut manager = CheckpointManager::new();

        // Set initial state
        let mut initial_state = HashMap::new();
        initial_state.insert("file1.txt".to_string(), "initial content".to_string());
        initial_state.insert("file2.txt".to_string(), "file2 content".to_string());
        manager.set_current_state(initial_state.clone());

        // Create first checkpoint
        let mut checkpoint1_state = HashMap::new();
        checkpoint1_state.insert("file1.txt".to_string(), "checkpoint1 content".to_string());
        checkpoint1_state.insert("file2.txt".to_string(), "file2 content".to_string());
        let cp1_id = manager
            .create_checkpoint(
                "Checkpoint 1",
                "First checkpoint",
                checkpoint1_state.clone(),
            )
            .unwrap();

        // Modify state
        let mut modified_state = HashMap::new();
        modified_state.insert("file1.txt".to_string(), "modified content".to_string());
        modified_state.insert("file2.txt".to_string(), "modified file2".to_string());
        modified_state.insert("file3.txt".to_string(), "new file".to_string());
        manager.set_current_state(modified_state);

        // Create second checkpoint
        let mut checkpoint2_state = HashMap::new();
        checkpoint2_state.insert("file1.txt".to_string(), "modified content".to_string());
        checkpoint2_state.insert("file2.txt".to_string(), "modified file2".to_string());
        checkpoint2_state.insert("file3.txt".to_string(), "new file".to_string());
        let cp2_id = manager
            .create_checkpoint(
                "Checkpoint 2",
                "Second checkpoint",
                checkpoint2_state.clone(),
            )
            .unwrap();

        // Verify both checkpoints exist
        assert_eq!(manager.checkpoint_count(), 2);

        // Rollback to checkpoint 1
        manager.rollback_to(&cp1_id).unwrap();
        let current = manager.get_current_state();
        assert_eq!(
            current.get("file1.txt"),
            Some(&"checkpoint1 content".to_string())
        );
        assert_eq!(current.get("file2.txt"), Some(&"file2 content".to_string()));
        // Note: file3.txt remains in current state (rollback doesn't delete files not in checkpoint)
        assert_eq!(current.get("file3.txt"), Some(&"new file".to_string()));

        // Verify checkpoint 2 is still intact
        let cp2 = manager.get_checkpoint(&cp2_id).unwrap();
        assert_eq!(cp2.file_states.len(), 3);

        // Rollback to checkpoint 2
        manager.rollback_to(&cp2_id).unwrap();
        let current = manager.get_current_state();
        assert_eq!(
            current.get("file1.txt"),
            Some(&"modified content".to_string())
        );
        assert_eq!(
            current.get("file2.txt"),
            Some(&"modified file2".to_string())
        );
        assert_eq!(current.get("file3.txt"), Some(&"new file".to_string()));

        // Verify checkpoint 1 is still intact
        let cp1 = manager.get_checkpoint(&cp1_id).unwrap();
        assert_eq!(cp1.file_states.len(), 2);
    }

    /// Test persistence across sessions (save → load)
    #[test]
    fn test_persistence_across_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        // Session 1: Create and save history
        {
            let mut store = HistoryStore::new(&store_path).unwrap();

            // Create changes
            let change1 = Change::new(
                "file1.txt",
                "",
                "content1",
                "Create file1",
                ChangeType::Create,
            )
            .unwrap();

            let change2 = Change::new(
                "file2.txt",
                "",
                "content2",
                "Create file2",
                ChangeType::Create,
            )
            .unwrap();

            // Create checkpoints
            let mut checkpoint_state = HashMap::new();
            checkpoint_state.insert("file1.txt".to_string(), "content1".to_string());
            checkpoint_state.insert("file2.txt".to_string(), "content2".to_string());
            let checkpoint =
                Checkpoint::new("Session 1 Checkpoint", "desc", checkpoint_state).unwrap();

            // Save snapshot
            let snapshot = HistorySnapshot::new(vec![change1, change2], {
                let mut map = HashMap::new();
                map.insert(checkpoint.id.clone(), checkpoint);
                map
            });

            store.save_history(snapshot).unwrap();
        }

        // Session 2: Load and verify history
        {
            let mut store = HistoryStore::new(&store_path).unwrap();
            let loaded = store.load_history().unwrap();

            // Verify changes were loaded
            assert_eq!(loaded.changes.len(), 2);
            assert_eq!(loaded.changes[0].file_path, "file1.txt");
            assert_eq!(loaded.changes[1].file_path, "file2.txt");

            // Verify checkpoints were loaded
            assert_eq!(loaded.checkpoints.len(), 1);
            let checkpoint = loaded.checkpoints.values().next().unwrap();
            assert_eq!(checkpoint.name, "Session 1 Checkpoint");
            assert_eq!(checkpoint.file_states.len(), 2);
        }
    }

    /// Test concurrent change tracking
    #[test]
    fn test_concurrent_change_tracking() {
        let mut tracker = ChangeTracker::new();

        // Simulate concurrent changes by tracking multiple changes
        let changes = vec![
            Change::new(
                "file1.txt",
                "",
                "content1",
                "Create file1",
                ChangeType::Create,
            )
            .unwrap(),
            Change::new(
                "file2.txt",
                "",
                "content2",
                "Create file2",
                ChangeType::Create,
            )
            .unwrap(),
            Change::new(
                "file3.txt",
                "",
                "content3",
                "Create file3",
                ChangeType::Create,
            )
            .unwrap(),
        ];

        // Track batch of changes
        tracker.track_batch(changes.clone()).unwrap();

        // Verify all changes were tracked
        let pending = tracker.get_pending_changes();
        assert_eq!(pending.len(), 3);

        // Verify each change is recorded
        for (original, tracked) in changes.iter().zip(pending.iter()) {
            assert_eq!(original.id, tracked.id);
            assert_eq!(original.file_path, tracked.file_path);
        }
    }

    /// Test error recovery (corrupted history, storage full)
    #[test]
    fn test_error_recovery_corrupted_history() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        // Write corrupted JSON
        std::fs::write(&store_path, "{ invalid json }").unwrap();

        // Try to load corrupted history
        let mut store = HistoryStore::new(&store_path).unwrap();
        let loaded = store.load_history().unwrap();

        // Should fall back to empty history
        assert_eq!(loaded.changes.len(), 0);
        assert_eq!(loaded.checkpoints.len(), 0);
    }

    /// Test error recovery with storage management
    #[test]
    fn test_error_recovery_storage_management() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut manager = StorageManager::with_defaults(&store_path).unwrap();

        // Create some changes
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "Test change",
            ChangeType::Modify,
        )
        .unwrap();

        let snapshot = HistorySnapshot::new(vec![change], HashMap::new());
        manager.get_store_mut().save_history(snapshot).unwrap();

        // Cleanup on session start should not fail
        let result = manager.cleanup_on_session_start();
        assert!(result.is_ok());

        // Cleanup on session end should not fail
        let result = manager.cleanup_on_session_end();
        assert!(result.is_ok());

        // Enforce storage limit should not fail
        let result = manager.enforce_storage_limit();
        assert!(result.is_ok());
    }

    /// Test complete workflow: track → checkpoint → undo → redo → rollback
    #[test]
    fn test_complete_workflow_track_checkpoint_undo_redo_rollback() {
        let mut tracker = ChangeTracker::new();
        let mut history_manager = HistoryManager::new();
        let mut checkpoint_manager = CheckpointManager::new();

        // Step 1: Track initial changes
        let change1 = Change::new(
            "file1.txt",
            "",
            "initial",
            "Create file1",
            ChangeType::Create,
        )
        .unwrap();

        tracker
            .track_change(
                "file1.txt",
                "",
                "initial",
                "Create file1",
                ChangeType::Create,
            )
            .unwrap();

        history_manager.record_change(change1).unwrap();

        // Step 2: Create checkpoint
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file1.txt".to_string(), "initial".to_string());
        checkpoint_manager.set_current_state(checkpoint_state.clone());

        let cp_id = checkpoint_manager
            .create_checkpoint("Checkpoint 1", "Initial state", checkpoint_state)
            .unwrap();

        // Step 3: Track more changes
        let change2 = Change::new(
            "file1.txt",
            "initial",
            "modified",
            "Modify file1",
            ChangeType::Modify,
        )
        .unwrap();

        history_manager.record_change(change2).unwrap();

        // Step 4: Undo the modification
        let undone = history_manager.undo().unwrap();
        assert_eq!(undone.description, "Modify file1");

        // Step 5: Redo the modification
        let redone = history_manager.redo().unwrap();
        assert_eq!(redone.description, "Modify file1");

        // Step 6: Rollback to checkpoint
        let mut modified_state = HashMap::new();
        modified_state.insert("file1.txt".to_string(), "modified".to_string());
        checkpoint_manager.set_current_state(modified_state);

        checkpoint_manager.rollback_to(&cp_id).unwrap();

        // Verify state was restored to checkpoint
        let current = checkpoint_manager.get_current_state();
        assert_eq!(current.get("file1.txt"), Some(&"initial".to_string()));

        // Verify history still has all changes
        assert_eq!(history_manager.total_changes(), 2);
    }

    /// Test persistence with multiple checkpoints and changes
    #[test]
    fn test_persistence_with_multiple_checkpoints_and_changes() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        // Session 1: Create complex history
        {
            let mut store = HistoryStore::new(&store_path).unwrap();

            // Create multiple changes
            let mut changes = Vec::new();
            for i in 0..5 {
                let change = Change::new(
                    &format!("file{}.txt", i),
                    "",
                    &format!("content{}", i),
                    &format!("Create file{}", i),
                    ChangeType::Create,
                )
                .unwrap();
                changes.push(change);
            }

            // Create multiple checkpoints
            let mut checkpoints = HashMap::new();
            for i in 0..3 {
                let mut file_states = HashMap::new();
                for j in 0..=i {
                    file_states.insert(format!("file{}.txt", j), format!("content{}", j));
                }
                let checkpoint = Checkpoint::new(
                    format!("Checkpoint {}", i),
                    &format!("State after {} files", i + 1),
                    file_states,
                )
                .unwrap();
                checkpoints.insert(checkpoint.id.clone(), checkpoint);
            }

            let snapshot = HistorySnapshot::new(changes, checkpoints);
            store.save_history(snapshot).unwrap();
        }

        // Session 2: Load and verify
        {
            let mut store = HistoryStore::new(&store_path).unwrap();
            let loaded = store.load_history().unwrap();

            // Verify all changes were loaded
            assert_eq!(loaded.changes.len(), 5);
            for i in 0..5 {
                assert_eq!(loaded.changes[i].file_path, format!("file{}.txt", i));
            }

            // Verify all checkpoints were loaded
            assert_eq!(loaded.checkpoints.len(), 3);

            // Verify checkpoint integrity
            for checkpoint in loaded.checkpoints.values() {
                assert!(checkpoint.validate().is_ok());
            }
        }
    }

    /// Test undo/redo with checkpoint isolation
    #[test]
    fn test_undo_redo_with_checkpoint_isolation() {
        let mut history_manager = HistoryManager::new();
        let mut checkpoint_manager = CheckpointManager::new();

        // Record changes
        let change1 = Change::new("file.txt", "", "state1", "Create", ChangeType::Create).unwrap();

        let change2 = Change::new(
            "file.txt",
            "state1",
            "state2",
            "Modify to state2",
            ChangeType::Modify,
        )
        .unwrap();

        history_manager.record_change(change1).unwrap();
        history_manager.record_change(change2).unwrap();

        // Create checkpoint at state2
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file.txt".to_string(), "state2".to_string());
        let cp_id = checkpoint_manager
            .create_checkpoint("CP at state2", "desc", checkpoint_state)
            .unwrap();

        // Undo to state1
        history_manager.undo().unwrap();

        // Verify checkpoint is still at state2
        let checkpoint = checkpoint_manager.get_checkpoint(&cp_id).unwrap();
        assert_eq!(
            checkpoint.file_states.get("file.txt"),
            Some(&"state2".to_string())
        );

        // Redo to state2
        history_manager.redo().unwrap();

        // Verify checkpoint is still at state2
        let checkpoint = checkpoint_manager.get_checkpoint(&cp_id).unwrap();
        assert_eq!(
            checkpoint.file_states.get("file.txt"),
            Some(&"state2".to_string())
        );
    }

    /// Test change tracking with batch operations
    #[test]
    fn test_change_tracking_with_batch_operations() {
        let mut tracker = ChangeTracker::new();
        let mut manager = HistoryManager::new();

        // Create batch of changes
        let batch1 = vec![
            Change::new("file1.txt", "", "content1", "Create 1", ChangeType::Create).unwrap(),
            Change::new("file2.txt", "", "content2", "Create 2", ChangeType::Create).unwrap(),
            Change::new("file3.txt", "", "content3", "Create 3", ChangeType::Create).unwrap(),
        ];

        // Track batch
        tracker.track_batch(batch1.clone()).unwrap();
        assert_eq!(tracker.pending_count(), 3);

        // Record all changes in history
        for change in batch1 {
            manager.record_change(change).unwrap();
        }

        // Verify all changes are in history
        assert_eq!(manager.total_changes(), 3);

        // Undo all changes
        for _ in 0..3 {
            assert!(manager.undo().is_ok());
        }

        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Redo all changes
        for _ in 0..3 {
            assert!(manager.redo().is_ok());
        }

        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }
}
