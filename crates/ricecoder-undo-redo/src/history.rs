//! History management and navigation

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{change::Change, error::UndoRedoError};

/// Represents a single entry in the change history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The change associated with this entry
    pub change: Change,
    /// Position in the history
    pub index: usize,
    /// Whether this change is currently undone
    pub is_undone: bool,
    /// Session ID this change belongs to (for session-scoped history)
    pub session_id: Option<String>,
}

/// Configuration for history management
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    /// Maximum number of undo operations to keep
    pub max_undo_stack_size: usize,
    /// Maximum number of redo operations to keep
    pub max_redo_stack_size: usize,
    /// Whether to enable session-scoped history
    pub session_scoped: bool,
    /// Whether to enable persistence
    pub enable_persistence: bool,
    /// Directory for persistent storage
    pub persistence_dir: Option<PathBuf>,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        HistoryConfig {
            max_undo_stack_size: 100,
            max_redo_stack_size: 50,
            session_scoped: false,
            enable_persistence: false,
            persistence_dir: None,
        }
    }
}

/// File change tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeInfo {
    /// File path that was changed
    pub file_path: PathBuf,
    /// Type of change
    pub change_type: crate::change::ChangeType,
    /// Session ID (if session-scoped)
    pub session_id: Option<String>,
    /// Timestamp of change
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(change: Change, index: usize) -> Self {
        HistoryEntry {
            change,
            index,
            is_undone: false,
            session_id: None,
        }
    }
}

/// Manages undo/redo stacks and change history
pub struct HistoryManager {
    undo_stack: Vec<Change>,
    redo_stack: Vec<Change>,
    all_changes: Vec<HistoryEntry>,
    config: HistoryConfig,
    /// Track changes by session (session_id -> changes)
    session_changes: HashMap<String, Vec<HistoryEntry>>,
    /// Track file changes for quick lookup
    file_changes: HashMap<PathBuf, Vec<FileChangeInfo>>,
    /// Current session ID (if session-scoped)
    current_session_id: Option<String>,
}

impl HistoryManager {
    /// Create a new history manager with default configuration
    pub fn new() -> Self {
        Self::with_config(HistoryConfig::default())
    }

    /// Create a new history manager with custom configuration
    pub fn with_config(config: HistoryConfig) -> Self {
        HistoryManager {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            all_changes: Vec::new(),
            config,
            session_changes: HashMap::new(),
            file_changes: HashMap::new(),
            current_session_id: None,
        }
    }

    /// Set the current session ID for session-scoped history
    pub fn set_current_session(&mut self, session_id: String) {
        self.current_session_id = Some(session_id);
    }

    /// Clear the current session
    pub fn clear_current_session(&mut self) {
        self.current_session_id = None;
    }

    /// Record a change in history
    pub fn record_change(&mut self, change: Change) -> Result<(), UndoRedoError> {
        change.validate()?;

        // Enforce stack size limits
        if self.undo_stack.len() >= self.config.max_undo_stack_size {
            // Remove oldest undo entry
            let removed = self.undo_stack.remove(0);
            // Mark as removed in all_changes if found
            if let Some(entry) = self
                .all_changes
                .iter_mut()
                .find(|e| e.change.id == removed.id)
            {
                entry.is_undone = true; // Mark as effectively undone
            }
        }

        // Add to undo stack
        self.undo_stack.push(change.clone());

        // Clear redo stack when new change is recorded
        self.redo_stack.clear();

        // Add to all_changes history
        let index = self.all_changes.len();
        let session_id = self.current_session_id.clone();
        let mut entry = HistoryEntry::new(change.clone(), index);
        entry.session_id = session_id.clone();
        self.all_changes.push(entry);

        // Add to session-scoped history if enabled
        if self.config.session_scoped {
            if let Some(session_id) = &session_id {
                let session_entry = HistoryEntry::new(change.clone(), index);
                self.session_changes
                    .entry(session_id.clone())
                    .or_insert_with(Vec::new)
                    .push(session_entry);
            }
        }

        // Track file changes
        let file_change_info = FileChangeInfo {
            file_path: PathBuf::from(change.file_path.clone()),
            change_type: change.change_type,
            session_id: session_id,
            timestamp: change.timestamp,
        };
        self.file_changes
            .entry(file_change_info.file_path.clone())
            .or_insert_with(Vec::new)
            .push(file_change_info);

        Ok(())
    }

    /// Perform an undo operation
    pub fn undo(&mut self) -> Result<Change, UndoRedoError> {
        let change = self.undo_stack.pop().ok_or(UndoRedoError::NoMoreUndos)?;

        // Mark as undone in history
        if let Some(entry) = self
            .all_changes
            .iter_mut()
            .rev()
            .find(|e| e.change.id == change.id)
        {
            entry.is_undone = true;
        }

        // Add to redo stack
        self.redo_stack.push(change.clone());

        Ok(change)
    }

    /// Perform a redo operation
    pub fn redo(&mut self) -> Result<Change, UndoRedoError> {
        let change = self.redo_stack.pop().ok_or(UndoRedoError::NoMoreRedos)?;

        // Mark as not undone in history
        if let Some(entry) = self
            .all_changes
            .iter_mut()
            .rev()
            .find(|e| e.change.id == change.id)
        {
            entry.is_undone = false;
        }

        // Add back to undo stack
        self.undo_stack.push(change.clone());

        Ok(change)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Get changes for a specific session
    pub fn get_session_changes(&self, session_id: &str) -> Vec<&HistoryEntry> {
        self.session_changes
            .get(session_id)
            .map(|changes| changes.iter().collect())
            .unwrap_or_default()
    }

    /// Get changes for a specific file
    pub fn get_file_changes(&self, file_path: &PathBuf) -> Vec<&FileChangeInfo> {
        self.file_changes
            .get(file_path)
            .map(|changes| changes.iter().collect())
            .unwrap_or_default()
    }

    /// Get all files that have been changed
    pub fn get_changed_files(&self) -> HashSet<PathBuf> {
        self.file_changes.keys().cloned().collect()
    }

    /// Get undo stack size
    pub fn undo_stack_size(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    pub fn redo_stack_size(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get total history size
    pub fn total_history_size(&self) -> usize {
        self.all_changes.len()
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.all_changes.clear();
        self.session_changes.clear();
        self.file_changes.clear();
    }

    /// Clear history for a specific session
    pub fn clear_session_history(&mut self, session_id: &str) {
        if let Some(session_changes) = self.session_changes.get_mut(session_id) {
            session_changes.clear();
        }
        // Also remove from main history (simplified - in practice would need more sophisticated filtering)
        self.all_changes
            .retain(|entry| entry.session_id.as_ref() != Some(&session_id.to_string()));
    }

    /// Save history to persistent storage (if enabled)
    pub fn save_history(&self) -> Result<(), UndoRedoError> {
        if !self.config.enable_persistence {
            return Ok(());
        }

        if let Some(persistence_dir) = &self.config.persistence_dir {
            std::fs::create_dir_all(persistence_dir)?;

            // Save main history
            let history_path = persistence_dir.join("history.json");
            let history_data = serde_json::to_string(&self.all_changes)?;
            std::fs::write(history_path, history_data)?;

            // Save session changes
            let session_path = persistence_dir.join("session_changes.json");
            let session_data = serde_json::to_string(&self.session_changes)?;
            std::fs::write(session_path, session_data)?;

            // Save file changes
            let file_path = persistence_dir.join("file_changes.json");
            let file_data = serde_json::to_string(&self.file_changes)?;
            std::fs::write(file_path, file_data)?;

            Ok(())
        } else {
            Err(UndoRedoError::PersistenceError(
                "Persistence directory not configured".to_string(),
            ))
        }
    }

    /// Load history from persistent storage (if enabled)
    pub fn load_history(&mut self) -> Result<(), UndoRedoError> {
        if !self.config.enable_persistence {
            return Ok(());
        }

        if let Some(persistence_dir) = &self.config.persistence_dir {
            // Load main history
            let history_path = persistence_dir.join("history.json");
            if history_path.exists() {
                let history_data = std::fs::read_to_string(history_path)?;
                self.all_changes = serde_json::from_str(&history_data)?;
            }

            // Load session changes
            let session_path = persistence_dir.join("session_changes.json");
            if session_path.exists() {
                let session_data = std::fs::read_to_string(session_path)?;
                self.session_changes = serde_json::from_str(&session_data)?;
            }

            // Load file changes
            let file_path = persistence_dir.join("file_changes.json");
            if file_path.exists() {
                let file_data = std::fs::read_to_string(file_path)?;
                self.file_changes = serde_json::from_str(&file_data)?;
            }

            // Rebuild undo/redo stacks from loaded history (simplified - would need more sophisticated logic)
            // For now, just clear stacks since we can't reliably reconstruct them
            self.undo_stack.clear();
            self.redo_stack.clear();

            Ok(())
        } else {
            Err(UndoRedoError::PersistenceError(
                "Persistence directory not configured".to_string(),
            ))
        }
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get paginated history
    pub fn get_history(&self, limit: usize, offset: usize) -> Vec<HistoryEntry> {
        self.all_changes
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get details of a specific change
    pub fn get_change_details(&self, change_id: &str) -> Result<HistoryEntry, UndoRedoError> {
        self.all_changes
            .iter()
            .find(|e| e.change.id == change_id)
            .cloned()
            .ok_or_else(|| UndoRedoError::change_not_found(change_id))
    }

    /// Get all changes for a specific file
    pub fn get_changes_by_file(&self, file_path: &str) -> Vec<HistoryEntry> {
        self.all_changes
            .iter()
            .filter(|e| e.change.file_path == file_path)
            .cloned()
            .collect()
    }

    /// Get the total number of changes in history
    pub fn total_changes(&self) -> usize {
        self.all_changes.len()
    }

    /// Get the number of undoable changes
    pub fn undoable_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redoable changes
    pub fn redoable_count(&self) -> usize {
        self.redo_stack.len()
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::ChangeType;

    #[test]
    fn test_history_manager_record_change() {
        let mut manager = HistoryManager::new();
        let change =
            Change::new("test.txt", "before", "after", "Modify", ChangeType::Modify).unwrap();
        let result = manager.record_change(change);
        assert!(result.is_ok());
        assert_eq!(manager.total_changes(), 1);
        assert!(manager.can_undo());
    }

    #[test]
    fn test_history_manager_undo() {
        let mut manager = HistoryManager::new();
        let change =
            Change::new("test.txt", "before", "after", "Modify", ChangeType::Modify).unwrap();
        manager.record_change(change.clone()).unwrap();

        let undone = manager.undo();
        assert!(undone.is_ok());
        assert!(!manager.can_undo());
        assert!(manager.can_redo());
    }

    #[test]
    fn test_history_manager_redo() {
        let mut manager = HistoryManager::new();
        let change =
            Change::new("test.txt", "before", "after", "Modify", ChangeType::Modify).unwrap();
        manager.record_change(change).unwrap();
        manager.undo().unwrap();

        let redone = manager.redo();
        assert!(redone.is_ok());
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_history_manager_no_more_undos() {
        let mut manager = HistoryManager::new();
        let result = manager.undo();
        assert!(result.is_err());
        assert!(matches!(result, Err(UndoRedoError::NoMoreUndos)));
    }

    #[test]
    fn test_history_manager_no_more_redos() {
        let mut manager = HistoryManager::new();
        let result = manager.redo();
        assert!(result.is_err());
        assert!(matches!(result, Err(UndoRedoError::NoMoreRedos)));
    }

    #[test]
    fn test_history_manager_redo_stack_clearing() {
        let mut manager = HistoryManager::new();
        let change1 = Change::new(
            "test.txt",
            "before1",
            "after1",
            "Modify 1",
            ChangeType::Modify,
        )
        .unwrap();
        let change2 = Change::new(
            "test.txt",
            "after1",
            "after2",
            "Modify 2",
            ChangeType::Modify,
        )
        .unwrap();

        manager.record_change(change1).unwrap();
        manager.undo().unwrap();
        assert!(manager.can_redo());

        // Record new change after undo
        manager.record_change(change2).unwrap();
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_history_manager_get_history() {
        let mut manager = HistoryManager::new();
        for i in 0..5 {
            let change = Change::new(
                "test.txt",
                &format!("before{}", i),
                &format!("after{}", i),
                &format!("Modify {}", i),
                ChangeType::Modify,
            )
            .unwrap();
            manager.record_change(change).unwrap();
        }

        let history = manager.get_history(2, 1);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].index, 1);
    }

    #[test]
    fn test_history_manager_get_change_details() {
        let mut manager = HistoryManager::new();
        let change =
            Change::new("test.txt", "before", "after", "Modify", ChangeType::Modify).unwrap();
        let change_id = change.id.clone();
        manager.record_change(change).unwrap();

        let details = manager.get_change_details(&change_id);
        assert!(details.is_ok());
        assert_eq!(details.unwrap().change.id, change_id);
    }

    #[test]
    fn test_history_manager_get_changes_by_file() {
        let mut manager = HistoryManager::new();
        let change1 =
            Change::new("file1.txt", "before", "after", "Modify", ChangeType::Modify).unwrap();
        let change2 =
            Change::new("file2.txt", "before", "after", "Modify", ChangeType::Modify).unwrap();
        let change3 =
            Change::new("file1.txt", "before", "after", "Modify", ChangeType::Modify).unwrap();

        manager.record_change(change1).unwrap();
        manager.record_change(change2).unwrap();
        manager.record_change(change3).unwrap();

        let file1_changes = manager.get_changes_by_file("file1.txt");
        assert_eq!(file1_changes.len(), 2);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use super::*;
    use crate::change::ChangeType;

    /// Strategy for generating valid file paths
    fn file_path_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9_\-./]{1,50}\.rs".prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid content (non-empty, non-whitespace-only)
    fn content_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9]{1,100}".prop_map(|s| s.to_string())
    }

    proptest! {
        /// **Feature: ricecoder-undo-redo, Property 1: Undo/Redo Consistency**
        /// *For any* sequence of changes, performing undo followed by redo SHALL restore
        /// the original state exactly.
        /// **Validates: Requirements 2.1, 2.2**
        #[test]
        fn prop_undo_redo_consistency(
            changes_data in prop::collection::vec(
                (file_path_strategy(), content_strategy(), content_strategy()),
                1..10
            ),
        ) {
            let mut manager = HistoryManager::new();
            let mut recorded_changes = Vec::new();

            // Record a sequence of changes
            for (idx, (file_path, before, after)) in changes_data.iter().enumerate() {
                // Ensure before and after are different for modify operations
                prop_assume!(before != after);

                let change = Change::new(
                    file_path.clone(),
                    before.clone(),
                    after.clone(),
                    format!("Change {}", idx),
                    ChangeType::Modify,
                )
                .unwrap();

                recorded_changes.push(change.clone());
                manager.record_change(change).ok();
            }

            // Verify we recorded all changes
            prop_assert_eq!(
                manager.total_changes(),
                recorded_changes.len(),
                "All changes should be recorded"
            );

            // Perform undo operations for all changes
            let mut undone_changes = Vec::new();
            while manager.can_undo() {
                if let Ok(change) = manager.undo() {
                    undone_changes.push(change);
                }
            }

            // Verify all changes were undone
            prop_assert_eq!(
                undone_changes.len(),
                recorded_changes.len(),
                "All changes should be undoable"
            );

            // Perform redo operations for all undone changes
            let mut redone_changes = Vec::new();
            while manager.can_redo() {
                if let Ok(change) = manager.redo() {
                    redone_changes.push(change);
                }
            }

            // Verify all changes were redone
            prop_assert_eq!(
                redone_changes.len(),
                recorded_changes.len(),
                "All changes should be redoable"
            );

            // Verify state is restored: undone changes in reverse order should match redone changes
            for (undone, redone) in undone_changes.iter().zip(redone_changes.iter().rev()) {
                prop_assert_eq!(
                    &undone.id, &redone.id,
                    "Undone and redone changes should have same ID"
                );
                prop_assert_eq!(
                    &undone.file_path, &redone.file_path,
                    "File paths should match"
                );
                prop_assert_eq!(
                    &undone.before, &redone.before,
                    "Before states should match"
                );
                prop_assert_eq!(
                    &undone.after, &redone.after,
                    "After states should match"
                );
            }

            // Verify state after all redo operations
            // After redoing all changes, we should be able to undo them again
            prop_assert!(manager.can_undo(), "Should be able to undo after redo");
            prop_assert!(!manager.can_redo(), "No more redos should be available after redo");
        }

        /// **Feature: ricecoder-undo-redo, Property 4: Redo Stack Clearing**
        /// *For any* new change recorded after an undo operation, the redo stack SHALL be
        /// cleared and no previously undone changes SHALL be reapplicable.
        /// **Validates: Requirements 2.5**
        #[test]
        fn prop_redo_stack_clearing(
            initial_changes in prop::collection::vec(
                (file_path_strategy(), content_strategy(), content_strategy()),
                1..5
            ),
            new_change_data in (file_path_strategy(), content_strategy(), content_strategy()),
        ) {
            let mut manager = HistoryManager::new();

            // Record initial changes
            for (idx, (file_path, before, after)) in initial_changes.iter().enumerate() {
                prop_assume!(before != after);

                let change = Change::new(
                    file_path.clone(),
                    before.clone(),
                    after.clone(),
                    format!("Initial {}", idx),
                    ChangeType::Modify,
                )
                .unwrap();

                manager.record_change(change).ok();
            }

            let initial_count = manager.total_changes();
            prop_assume!(initial_count > 0);

            // Perform undo
            let undo_result = manager.undo();
            prop_assert!(undo_result.is_ok(), "Undo should succeed");
            prop_assert!(manager.can_redo(), "Redo should be available after undo");

            // Record a new change after undo
            let (file_path, before, after) = new_change_data;
            prop_assume!(before != after);

            let new_change = Change::new(
                file_path,
                before,
                after,
                "New change after undo",
                ChangeType::Modify,
            )
            .unwrap();

            manager.record_change(new_change).ok();

            // Verify redo stack is cleared
            prop_assert!(
                !manager.can_redo(),
                "Redo stack should be cleared after new change"
            );

            // Verify we can still undo the new change
            prop_assert!(
                manager.can_undo(),
                "Should be able to undo the new change"
            );

            // Verify total changes increased by 1
            prop_assert_eq!(
                manager.total_changes(),
                initial_count + 1,
                "Total changes should increase by 1"
            );
        }

        /// **Feature: ricecoder-undo-redo, Property 1: Undo/Redo Consistency**
        /// *For any* single change, undo followed by redo should restore exact state.
        /// **Validates: Requirements 2.1, 2.2**
        #[test]
        fn prop_single_change_undo_redo(
            file_path in file_path_strategy(),
            before in content_strategy(),
            after in content_strategy(),
        ) {
            prop_assume!(before != after);

            let mut manager = HistoryManager::new();
            let change = Change::new(
                &file_path,
                &before,
                &after,
                "Test change",
                ChangeType::Modify,
            )
            .unwrap();

            let original_id = change.id.clone();
            manager.record_change(change).unwrap();

            // Verify can undo
            prop_assert!(manager.can_undo(), "Should be able to undo");
            prop_assert!(!manager.can_redo(), "Should not be able to redo initially");

            // Perform undo
            let undone = manager.undo().unwrap();
            prop_assert_eq!(&undone.id, &original_id, "Undone change should have same ID");
            prop_assert_eq!(&undone.file_path, &file_path, "File path should match");
            prop_assert_eq!(&undone.before, &before, "Before state should match");
            prop_assert_eq!(&undone.after, &after, "After state should match");

            // Verify state after undo
            prop_assert!(!manager.can_undo(), "Should not be able to undo after undo");
            prop_assert!(manager.can_redo(), "Should be able to redo after undo");

            // Perform redo
            let redone = manager.redo().unwrap();
            prop_assert_eq!(&redone.id, &original_id, "Redone change should have same ID");
            prop_assert_eq!(&redone.file_path, &file_path, "File path should match");
            prop_assert_eq!(&redone.before, &before, "Before state should match");
            prop_assert_eq!(&redone.after, &after, "After state should match");

            // Verify state after redo
            prop_assert!(manager.can_undo(), "Should be able to undo after redo");
            prop_assert!(!manager.can_redo(), "Should not be able to redo after redo");
        }
    }
}
