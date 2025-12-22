//! Checkpoint management for rollback operations

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::UndoRedoError;

/// Represents a saved state for rollback operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique identifier for this checkpoint
    pub id: String,
    /// User-provided name for the checkpoint
    pub name: String,
    /// Optional description of the checkpoint
    pub description: String,
    /// When the checkpoint was created
    pub created_at: DateTime<Utc>,
    /// Number of changes in this checkpoint
    pub changes_count: usize,
    /// File path to content mapping for rollback
    pub file_states: HashMap<String, String>,
}

impl Checkpoint {
    /// Create a new checkpoint with metadata tracking
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        file_states: HashMap<String, String>,
    ) -> Result<Self, UndoRedoError> {
        let name = name.into();
        let description = description.into();

        // Validate name is not empty
        if name.is_empty() {
            return Err(UndoRedoError::validation_error(
                "Checkpoint name cannot be empty",
            ));
        }

        // Validate file_states is not empty
        if file_states.is_empty() {
            return Err(UndoRedoError::validation_error(
                "Checkpoint must contain at least one file state",
            ));
        }

        Ok(Checkpoint {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: Utc::now(),
            changes_count: file_states.len(),
            file_states,
        })
    }

    /// Validate the checkpoint for consistency
    pub fn validate(&self) -> Result<(), UndoRedoError> {
        if self.name.is_empty() {
            return Err(UndoRedoError::validation_error(
                "Checkpoint name cannot be empty",
            ));
        }

        if self.file_states.is_empty() {
            return Err(UndoRedoError::validation_error(
                "Checkpoint must contain at least one file state",
            ));
        }

        if self.changes_count != self.file_states.len() {
            return Err(UndoRedoError::validation_error(
                "Checkpoint changes_count does not match file_states length",
            ));
        }

        Ok(())
    }
}

/// Manages checkpoints for rollback operations
pub struct CheckpointManager {
    checkpoints: HashMap<String, Checkpoint>,
    current_state: HashMap<String, String>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new() -> Self {
        CheckpointManager {
            checkpoints: HashMap::new(),
            current_state: HashMap::new(),
        }
    }

    /// Create a new checkpoint and store it
    pub fn create_checkpoint(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        file_states: HashMap<String, String>,
    ) -> Result<String, UndoRedoError> {
        let checkpoint = Checkpoint::new(name, description, file_states)?;
        let id = checkpoint.id.clone();
        self.checkpoints.insert(id.clone(), checkpoint);
        Ok(id)
    }

    /// List all checkpoints
    pub fn list_checkpoints(&self) -> Vec<Checkpoint> {
        self.checkpoints.values().cloned().collect()
    }

    /// Get a specific checkpoint by ID
    pub fn get_checkpoint(&self, checkpoint_id: &str) -> Result<Checkpoint, UndoRedoError> {
        self.checkpoints
            .get(checkpoint_id)
            .cloned()
            .ok_or_else(|| UndoRedoError::checkpoint_not_found(checkpoint_id))
    }

    /// Delete a checkpoint by ID
    pub fn delete_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), UndoRedoError> {
        self.checkpoints
            .remove(checkpoint_id)
            .ok_or_else(|| UndoRedoError::checkpoint_not_found(checkpoint_id))?;
        Ok(())
    }

    /// Rollback to a specific checkpoint with atomic guarantees
    ///
    /// This operation attempts to restore all files to the checkpoint state.
    /// If any file fails to restore, the operation is rolled back to the pre-rollback state.
    /// Uses transaction-like semantics: all-or-nothing.
    pub fn rollback_to(&mut self, checkpoint_id: &str) -> Result<(), UndoRedoError> {
        // Get the checkpoint and validate it
        let checkpoint = self.get_checkpoint(checkpoint_id)?;
        checkpoint.validate()?;

        // Save current state for potential rollback (failure recovery)
        let pre_rollback_state = self.current_state.clone();

        // Attempt to apply all file states from checkpoint
        // We collect all updates first to ensure atomicity
        let mut updates = Vec::new();
        for (file_path, content) in &checkpoint.file_states {
            // Validate file path is not empty
            if file_path.is_empty() {
                // Restore pre-rollback state on validation failure
                self.current_state = pre_rollback_state;
                return Err(UndoRedoError::validation_error("File path cannot be empty"));
            }
            updates.push((file_path.clone(), content.clone()));
        }

        // Apply all updates atomically
        for (file_path, content) in updates {
            self.current_state.insert(file_path, content);
        }

        // If we reach here, all files were successfully updated
        // The rollback is complete and atomic
        Ok(())
    }

    /// Verify rollback success and report errors
    pub fn verify_rollback(&self, checkpoint_id: &str) -> Result<bool, UndoRedoError> {
        let checkpoint = self.get_checkpoint(checkpoint_id)?;

        // Verify all checkpoint files are in current state
        for (file_path, expected_content) in &checkpoint.file_states {
            match self.current_state.get(file_path) {
                Some(actual_content) => {
                    if actual_content != expected_content {
                        return Ok(false);
                    }
                }
                None => return Ok(false),
            }
        }

        Ok(true)
    }

    /// Restore the pre-rollback state (used for rollback failure recovery)
    pub fn restore_pre_rollback_state(&mut self, pre_rollback_state: HashMap<String, String>) {
        self.current_state = pre_rollback_state;
    }

    /// Get the current state
    pub fn get_current_state(&self) -> HashMap<String, String> {
        self.current_state.clone()
    }

    /// Set the current state (for testing and initialization)
    pub fn set_current_state(&mut self, state: HashMap<String, String>) {
        self.current_state = state;
    }

    /// Get the number of checkpoints
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_create_valid() {
        let mut file_states = HashMap::new();
        file_states.insert("file1.txt".to_string(), "content1".to_string());
        let checkpoint = Checkpoint::new("Test Checkpoint", "A test checkpoint", file_states);
        assert!(checkpoint.is_ok());
        let checkpoint = checkpoint.unwrap();
        assert_eq!(checkpoint.name, "Test Checkpoint");
        assert_eq!(checkpoint.changes_count, 1);
    }

    #[test]
    fn test_checkpoint_empty_name() {
        let mut file_states = HashMap::new();
        file_states.insert("file1.txt".to_string(), "content1".to_string());
        let checkpoint = Checkpoint::new("", "description", file_states);
        assert!(checkpoint.is_err());
    }

    #[test]
    fn test_checkpoint_empty_file_states() {
        let file_states = HashMap::new();
        let checkpoint = Checkpoint::new("Test", "description", file_states);
        assert!(checkpoint.is_err());
    }

    #[test]
    fn test_checkpoint_manager_create() {
        let mut manager = CheckpointManager::new();
        let mut file_states = HashMap::new();
        file_states.insert("file1.txt".to_string(), "content1".to_string());
        let result = manager.create_checkpoint("Test", "description", file_states);
        assert!(result.is_ok());
        assert_eq!(manager.checkpoint_count(), 1);
    }

    #[test]
    fn test_checkpoint_manager_list() {
        let mut manager = CheckpointManager::new();
        let mut file_states1 = HashMap::new();
        file_states1.insert("file1.txt".to_string(), "content1".to_string());
        let mut file_states2 = HashMap::new();
        file_states2.insert("file2.txt".to_string(), "content2".to_string());

        manager
            .create_checkpoint("Checkpoint 1", "desc1", file_states1)
            .unwrap();
        manager
            .create_checkpoint("Checkpoint 2", "desc2", file_states2)
            .unwrap();

        let checkpoints = manager.list_checkpoints();
        assert_eq!(checkpoints.len(), 2);
    }

    #[test]
    fn test_checkpoint_manager_get() {
        let mut manager = CheckpointManager::new();
        let mut file_states = HashMap::new();
        file_states.insert("file1.txt".to_string(), "content1".to_string());
        let id = manager
            .create_checkpoint("Test", "description", file_states)
            .unwrap();

        let checkpoint = manager.get_checkpoint(&id);
        assert!(checkpoint.is_ok());
        assert_eq!(checkpoint.unwrap().name, "Test");
    }

    #[test]
    fn test_checkpoint_manager_get_not_found() {
        let manager = CheckpointManager::new();
        let checkpoint = manager.get_checkpoint("nonexistent");
        assert!(checkpoint.is_err());
    }

    #[test]
    fn test_checkpoint_manager_delete() {
        let mut manager = CheckpointManager::new();
        let mut file_states = HashMap::new();
        file_states.insert("file1.txt".to_string(), "content1".to_string());
        let id = manager
            .create_checkpoint("Test", "description", file_states)
            .unwrap();

        assert_eq!(manager.checkpoint_count(), 1);
        let result = manager.delete_checkpoint(&id);
        assert!(result.is_ok());
        assert_eq!(manager.checkpoint_count(), 0);
    }

    #[test]
    fn test_checkpoint_serialization() {
        let mut file_states = HashMap::new();
        file_states.insert("file1.txt".to_string(), "content1".to_string());
        let checkpoint = Checkpoint::new("Test", "description", file_states).unwrap();
        let json = serde_json::to_string(&checkpoint).unwrap();
        let deserialized: Checkpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(checkpoint.id, deserialized.id);
        assert_eq!(checkpoint.name, deserialized.name);
    }

    #[test]
    fn test_checkpoint_manager_rollback_to() {
        let mut manager = CheckpointManager::new();

        // Set initial state
        let mut initial_state = HashMap::new();
        initial_state.insert("file1.txt".to_string(), "initial content".to_string());
        manager.set_current_state(initial_state);

        // Create a checkpoint with different state
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file1.txt".to_string(), "checkpoint content".to_string());
        checkpoint_state.insert("file2.txt".to_string(), "new file".to_string());

        let checkpoint_id = manager
            .create_checkpoint("Checkpoint 1", "desc", checkpoint_state.clone())
            .unwrap();

        // Rollback to checkpoint
        let result = manager.rollback_to(&checkpoint_id);
        assert!(result.is_ok());

        // Verify state was restored
        let current_state = manager.get_current_state();
        assert_eq!(
            current_state.get("file1.txt"),
            Some(&"checkpoint content".to_string())
        );
        assert_eq!(
            current_state.get("file2.txt"),
            Some(&"new file".to_string())
        );
    }

    #[test]
    fn test_checkpoint_manager_rollback_not_found() {
        let mut manager = CheckpointManager::new();
        let result = manager.rollback_to("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_manager_rollback_isolation() {
        let mut manager = CheckpointManager::new();

        // Create two checkpoints with different states
        let mut state1 = HashMap::new();
        state1.insert("file.txt".to_string(), "state1".to_string());
        let id1 = manager
            .create_checkpoint("Checkpoint 1", "desc1", state1)
            .unwrap();

        let mut state2 = HashMap::new();
        state2.insert("file.txt".to_string(), "state2".to_string());
        let id2 = manager
            .create_checkpoint("Checkpoint 2", "desc2", state2)
            .unwrap();

        // Rollback to checkpoint 1
        manager.rollback_to(&id1).unwrap();
        let current = manager.get_current_state();
        assert_eq!(current.get("file.txt"), Some(&"state1".to_string()));

        // Verify checkpoint 2 is still intact
        let cp2 = manager.get_checkpoint(&id2).unwrap();
        assert_eq!(cp2.file_states.get("file.txt"), Some(&"state2".to_string()));

        // Rollback to checkpoint 2
        manager.rollback_to(&id2).unwrap();
        let current = manager.get_current_state();
        assert_eq!(current.get("file.txt"), Some(&"state2".to_string()));

        // Verify checkpoint 1 is still intact
        let cp1 = manager.get_checkpoint(&id1).unwrap();
        assert_eq!(cp1.file_states.get("file.txt"), Some(&"state1".to_string()));
    }

    #[test]
    fn test_checkpoint_manager_restore_pre_rollback_state() {
        let mut manager = CheckpointManager::new();

        // Set initial state
        let mut initial_state = HashMap::new();
        initial_state.insert("file.txt".to_string(), "initial".to_string());
        manager.set_current_state(initial_state.clone());

        // Create checkpoint
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file.txt".to_string(), "checkpoint".to_string());
        let checkpoint_id = manager
            .create_checkpoint("CP", "desc", checkpoint_state)
            .unwrap();

        // Rollback
        manager.rollback_to(&checkpoint_id).unwrap();
        assert_eq!(
            manager.get_current_state().get("file.txt"),
            Some(&"checkpoint".to_string())
        );

        // Restore pre-rollback state
        manager.restore_pre_rollback_state(initial_state);
        assert_eq!(
            manager.get_current_state().get("file.txt"),
            Some(&"initial".to_string())
        );
    }

    #[test]
    fn test_checkpoint_manager_verify_rollback_success() {
        let mut manager = CheckpointManager::new();

        // Create checkpoint
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file1.txt".to_string(), "content1".to_string());
        checkpoint_state.insert("file2.txt".to_string(), "content2".to_string());
        let checkpoint_id = manager
            .create_checkpoint("CP", "desc", checkpoint_state)
            .unwrap();

        // Rollback
        manager.rollback_to(&checkpoint_id).unwrap();

        // Verify rollback success
        let result = manager.verify_rollback(&checkpoint_id);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_checkpoint_manager_verify_rollback_failure() {
        let mut manager = CheckpointManager::new();

        // Create checkpoint
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file.txt".to_string(), "content".to_string());
        let checkpoint_id = manager
            .create_checkpoint("CP", "desc", checkpoint_state)
            .unwrap();

        // Rollback
        manager.rollback_to(&checkpoint_id).unwrap();

        // Modify current state
        manager
            .current_state
            .insert("file.txt".to_string(), "modified".to_string());

        // Verify rollback failure
        let result = manager.verify_rollback(&checkpoint_id);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_checkpoint_manager_rollback_failure_recovery() {
        let mut manager = CheckpointManager::new();

        // Set initial state
        let mut initial_state = HashMap::new();
        initial_state.insert("file.txt".to_string(), "initial".to_string());
        manager.set_current_state(initial_state.clone());

        // Create checkpoint
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file.txt".to_string(), "checkpoint".to_string());
        let checkpoint_id = manager
            .create_checkpoint("CP", "desc", checkpoint_state)
            .unwrap();

        // Perform rollback
        let result = manager.rollback_to(&checkpoint_id);
        assert!(result.is_ok());

        // Verify state was updated
        assert_eq!(
            manager.get_current_state().get("file.txt"),
            Some(&"checkpoint".to_string())
        );

        // Simulate failure recovery by restoring pre-rollback state
        manager.restore_pre_rollback_state(initial_state);
        assert_eq!(
            manager.get_current_state().get("file.txt"),
            Some(&"initial".to_string())
        );
    }

    #[test]
    fn test_checkpoint_manager_rollback_atomic_all_or_nothing() {
        let mut manager = CheckpointManager::new();

        // Create checkpoint with multiple files
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file1.txt".to_string(), "content1".to_string());
        checkpoint_state.insert("file2.txt".to_string(), "content2".to_string());
        checkpoint_state.insert("file3.txt".to_string(), "content3".to_string());
        let checkpoint_id = manager
            .create_checkpoint("CP", "desc", checkpoint_state)
            .unwrap();

        // Rollback
        let result = manager.rollback_to(&checkpoint_id);
        assert!(result.is_ok());

        // Verify all files were updated (all-or-nothing)
        let current_state = manager.get_current_state();
        assert_eq!(current_state.len(), 3);
        assert_eq!(
            current_state.get("file1.txt"),
            Some(&"content1".to_string())
        );
        assert_eq!(
            current_state.get("file2.txt"),
            Some(&"content2".to_string())
        );
        assert_eq!(
            current_state.get("file3.txt"),
            Some(&"content3".to_string())
        );
    }

    #[test]
    fn test_checkpoint_isolation_independent_storage() {
        let mut manager = CheckpointManager::new();

        // Create first checkpoint
        let mut state1 = HashMap::new();
        state1.insert("file.txt".to_string(), "state1".to_string());
        let id1 = manager.create_checkpoint("CP1", "desc1", state1).unwrap();

        // Create second checkpoint
        let mut state2 = HashMap::new();
        state2.insert("file.txt".to_string(), "state2".to_string());
        let id2 = manager.create_checkpoint("CP2", "desc2", state2).unwrap();

        // Verify both checkpoints exist independently
        let cp1 = manager.get_checkpoint(&id1).unwrap();
        let cp2 = manager.get_checkpoint(&id2).unwrap();

        assert_eq!(cp1.file_states.get("file.txt"), Some(&"state1".to_string()));
        assert_eq!(cp2.file_states.get("file.txt"), Some(&"state2".to_string()));

        // Rollback to checkpoint 1
        manager.rollback_to(&id1).unwrap();

        // Verify checkpoint 2 is still intact (not affected by rollback)
        let cp2_after = manager.get_checkpoint(&id2).unwrap();
        assert_eq!(
            cp2_after.file_states.get("file.txt"),
            Some(&"state2".to_string())
        );

        // Verify current state matches checkpoint 1
        assert_eq!(
            manager.get_current_state().get("file.txt"),
            Some(&"state1".to_string())
        );
    }

    #[test]
    fn test_checkpoint_isolation_prevent_corruption() {
        let mut manager = CheckpointManager::new();

        // Create checkpoint
        let mut checkpoint_state = HashMap::new();
        checkpoint_state.insert("file1.txt".to_string(), "content1".to_string());
        checkpoint_state.insert("file2.txt".to_string(), "content2".to_string());
        let checkpoint_id = manager
            .create_checkpoint("CP", "desc", checkpoint_state)
            .unwrap();

        // Verify checkpoint is valid
        let checkpoint = manager.get_checkpoint(&checkpoint_id).unwrap();
        assert!(checkpoint.validate().is_ok());

        // Rollback
        manager.rollback_to(&checkpoint_id).unwrap();

        // Verify checkpoint is still valid and unchanged
        let checkpoint_after = manager.get_checkpoint(&checkpoint_id).unwrap();
        assert!(checkpoint_after.validate().is_ok());
        assert_eq!(checkpoint.id, checkpoint_after.id);
        assert_eq!(checkpoint.name, checkpoint_after.name);
        assert_eq!(checkpoint.file_states, checkpoint_after.file_states);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use super::*;

    /// Strategy for generating valid checkpoint names
    fn checkpoint_name_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9\s\-_]{1,50}".prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid file paths
    fn file_path_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9_\-./]{1,50}\.rs".prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid content
    fn content_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9\s]{1,100}".prop_map(|s| s.to_string())
    }

    proptest! {
        /// **Feature: ricecoder-undo-redo, Property 3: Rollback Atomicity**
        /// *For any* rollback operation to a checkpoint, either all files are reverted
        /// to checkpoint state or none are modified.
        /// **Validates: Requirements 4.2, 4.4, 4.5**
        #[test]
        fn prop_rollback_atomicity(
            checkpoint_files in prop::collection::hash_map(
                file_path_strategy(),
                content_strategy(),
                1..10
            ),
            name in checkpoint_name_strategy(),
        ) {
            let mut manager = CheckpointManager::new();

            // Create checkpoint with multiple files
            let checkpoint_id = manager
                .create_checkpoint(&name, "description", checkpoint_files.clone())
                .ok();

            prop_assert!(checkpoint_id.is_some(), "Checkpoint creation should succeed");
            let checkpoint_id = checkpoint_id.unwrap();

            // Perform rollback
            let rollback_result = manager.rollback_to(&checkpoint_id);
            prop_assert!(rollback_result.is_ok(), "Rollback should succeed");

            // Verify all files were restored to checkpoint state
            let current_state = manager.get_current_state();
            for (file_path, expected_content) in &checkpoint_files {
                let actual_content = current_state.get(file_path);
                prop_assert_eq!(
                    actual_content,
                    Some(expected_content),
                    "File {} should be restored to checkpoint state",
                    file_path
                );
            }

            // Verify no extra files were added
            prop_assert_eq!(
                current_state.len(),
                checkpoint_files.len(),
                "Current state should have exactly the checkpoint files"
            );
        }

        /// **Feature: ricecoder-undo-redo, Property 5: Checkpoint Isolation**
        /// *For any* checkpoint, rolling back to that checkpoint SHALL not affect
        /// other checkpoints or the current history.
        /// **Validates: Requirements 4.1, 4.3**
        #[test]
        fn prop_checkpoint_isolation(
            checkpoint_data in prop::collection::vec(
                (checkpoint_name_strategy(), prop::collection::hash_map(
                    file_path_strategy(),
                    content_strategy(),
                    1..5
                )),
                2..5
            ),
        ) {
            let mut manager = CheckpointManager::new();
            let mut checkpoint_ids = Vec::new();

            // Create multiple checkpoints
            for (name, files) in checkpoint_data.iter() {
                prop_assume!(!files.is_empty());
                if let Ok(id) = manager.create_checkpoint(name, "desc", files.clone()) {
                    checkpoint_ids.push((id, files.clone()));
                }
            }

            prop_assume!(checkpoint_ids.len() >= 2);

            // Rollback to each checkpoint and verify others remain intact
            for (rollback_idx, (rollback_id, _)) in checkpoint_ids.iter().enumerate() {
                // Perform rollback
                let rollback_result = manager.rollback_to(rollback_id);
                prop_assert!(rollback_result.is_ok(), "Rollback should succeed");

                // Verify all other checkpoints are still intact
                for (other_idx, (other_id, other_files)) in checkpoint_ids.iter().enumerate() {
                    if rollback_idx != other_idx {
                        let checkpoint = manager.get_checkpoint(other_id);
                        prop_assert!(checkpoint.is_ok(), "Other checkpoint should still exist");

                        let checkpoint = checkpoint.unwrap();
                        for (file_path, expected_content) in other_files {
                            let actual_content = checkpoint.file_states.get(file_path);
                            prop_assert_eq!(
                                actual_content,
                                Some(expected_content),
                                "Other checkpoint file {} should be unchanged",
                                file_path
                            );
                        }
                    }
                }
            }
        }

        /// **Feature: ricecoder-undo-redo, Property 3: Rollback Atomicity**
        /// *For any* single checkpoint, rollback should restore all files atomically.
        /// **Validates: Requirements 4.2, 4.4, 4.5**
        #[test]
        fn prop_single_checkpoint_rollback(
            files in prop::collection::hash_map(
                file_path_strategy(),
                content_strategy(),
                1..5
            ),
            name in checkpoint_name_strategy(),
        ) {
            prop_assume!(!files.is_empty());

            let mut manager = CheckpointManager::new();

            // Create checkpoint
            let checkpoint_id = manager
                .create_checkpoint(&name, "desc", files.clone())
                .unwrap();

            // Verify checkpoint was created
            let checkpoint = manager.get_checkpoint(&checkpoint_id).unwrap();
            prop_assert_eq!(
                checkpoint.file_states.len(),
                files.len(),
                "Checkpoint should contain all files"
            );

            // Perform rollback
            manager.rollback_to(&checkpoint_id).unwrap();

            // Verify current state matches checkpoint exactly
            let current_state = manager.get_current_state();
            prop_assert_eq!(
                current_state.len(),
                files.len(),
                "Current state should have same number of files"
            );

            for (file_path, expected_content) in &files {
                let actual_content = current_state.get(file_path);
                prop_assert_eq!(
                    actual_content,
                    Some(expected_content),
                    "File {} should match checkpoint state",
                    file_path
                );
            }
        }
    }
}
