#[cfg(test)]
mod integration_tests_suite {
    use std::collections::HashMap;

    use tempfile::TempDir;

    use crate::{
        Change, ChangeTracker, ChangeType, Checkpoint, CheckpointManager, HistoryManager,
        HistorySnapshot, HistoryStore, StorageManager,
    };

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
            let storage = StorageManager::new(&store_path);
            let mut history = HistoryManager::new();
            history
                .record_change(
                    Change::new(
                        "file1.txt",
                        "",
                        "initial",
                        "Initial change",
                        ChangeType::Create,
                    )
                    .unwrap(),
                )
                .unwrap();
            history
                .save(&storage)
                .expect("failed to save history during session 1");
        }

        // Session 2: Load history and verify
        {
            let storage = StorageManager::new(&store_path);
            let history = HistoryManager::load(&storage).expect("failed to load history");
            assert!(history.can_undo());
            let snapshot = history.snapshots().next().expect("snapshot missing");
            assert_eq!(snapshot.changes.len(), 1);
            assert_eq!(snapshot.changes[0].description, "Initial change");
        }
    }

    /// Test checkpoint manager expiration of stale checkpoints
    #[test]
    fn test_checkpoint_expiration() {
        let mut manager = CheckpointManager::new();
        let mut state = HashMap::new();
        state.insert("file1.txt".to_string(), "content".to_string());
        manager.set_current_state(state.clone());

        // Create checkpoints and expire one
        let cp1 = manager
            .create_checkpoint("First", "first cp", state.clone())
            .unwrap();
        let cp2 = manager
            .create_checkpoint("Second", "second cp", state)
            .unwrap();

        manager.expire_checkpoint(&cp1.id).unwrap();
        assert!(manager.get_checkpoint(&cp1.id).is_none());
        assert!(manager.get_checkpoint(&cp2.id).is_some());
    }
}
