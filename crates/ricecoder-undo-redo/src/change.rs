//! Change tracking and recording

use crate::error::UndoRedoError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Type of change made to a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// File was created
    Create,
    /// File was modified
    Modify,
    /// File was deleted
    Delete,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeType::Create => write!(f, "Create"),
            ChangeType::Modify => write!(f, "Modify"),
            ChangeType::Delete => write!(f, "Delete"),
        }
    }
}

/// Represents a single modification to a file or system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Unique identifier for this change
    pub id: String,
    /// When the change occurred
    pub timestamp: DateTime<Utc>,
    /// Path to the modified file
    pub file_path: String,
    /// State before the change
    pub before: String,
    /// State after the change
    pub after: String,
    /// Human-readable description of the change
    pub description: String,
    /// Type of change
    pub change_type: ChangeType,
}

impl Change {
    /// Create a new change with automatic UUID and timestamp
    pub fn new(
        file_path: impl Into<String>,
        before: impl Into<String>,
        after: impl Into<String>,
        description: impl Into<String>,
        change_type: ChangeType,
    ) -> Result<Self, UndoRedoError> {
        let file_path = file_path.into();
        let before = before.into();
        let after = after.into();
        let description = description.into();

        // Validate file path is not empty
        if file_path.is_empty() {
            return Err(UndoRedoError::validation_error("file_path cannot be empty"));
        }

        // Validate before/after state consistency
        match change_type {
            ChangeType::Create => {
                if !before.is_empty() {
                    return Err(UndoRedoError::validation_error(
                        "Create change must have empty before state",
                    ));
                }
            }
            ChangeType::Delete => {
                if !after.is_empty() {
                    return Err(UndoRedoError::validation_error(
                        "Delete change must have empty after state",
                    ));
                }
            }
            ChangeType::Modify => {
                if before.is_empty() || after.is_empty() {
                    return Err(UndoRedoError::validation_error(
                        "Modify change must have non-empty before and after states",
                    ));
                }
            }
        }

        Ok(Change {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            file_path,
            before,
            after,
            description,
            change_type,
        })
    }

    /// Validate the change for consistency
    pub fn validate(&self) -> Result<(), UndoRedoError> {
        if self.file_path.is_empty() {
            return Err(UndoRedoError::validation_error("file_path cannot be empty"));
        }

        match self.change_type {
            ChangeType::Create => {
                if !self.before.is_empty() {
                    return Err(UndoRedoError::validation_error(
                        "Create change must have empty before state",
                    ));
                }
            }
            ChangeType::Delete => {
                if !self.after.is_empty() {
                    return Err(UndoRedoError::validation_error(
                        "Delete change must have empty after state",
                    ));
                }
            }
            ChangeType::Modify => {
                if self.before.is_empty() || self.after.is_empty() {
                    return Err(UndoRedoError::validation_error(
                        "Modify change must have non-empty before and after states",
                    ));
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} - {} ({})",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.change_type,
            self.file_path,
            self.description
        )
    }
}

/// Tracks changes to files and system state
pub struct ChangeTracker {
    pending_changes: Vec<Change>,
}

impl ChangeTracker {
    /// Create a new change tracker
    pub fn new() -> Self {
        ChangeTracker {
            pending_changes: Vec::new(),
        }
    }

    /// Track a single change
    pub fn track_change(
        &mut self,
        file_path: impl Into<String>,
        before: impl Into<String>,
        after: impl Into<String>,
        description: impl Into<String>,
        change_type: ChangeType,
    ) -> Result<String, UndoRedoError> {
        let change = Change::new(file_path, before, after, description, change_type)?;
        let id = change.id.clone();
        self.pending_changes.push(change);
        Ok(id)
    }

    /// Track multiple changes atomically
    pub fn track_batch(&mut self, changes: Vec<Change>) -> Result<(), UndoRedoError> {
        // Validate all changes first
        for change in &changes {
            change.validate()?;
        }

        // Add all changes if validation passes
        self.pending_changes.extend(changes);
        Ok(())
    }

    /// Get all pending changes
    pub fn get_pending_changes(&self) -> Vec<Change> {
        self.pending_changes.clone()
    }

    /// Clear pending changes
    pub fn clear_pending(&mut self) {
        self.pending_changes.clear();
    }

    /// Get the number of pending changes
    pub fn pending_count(&self) -> usize {
        self.pending_changes.len()
    }
}

impl Default for ChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_create_valid() {
        let change = Change::new(
            "test.txt",
            "",
            "content",
            "Create test file",
            ChangeType::Create,
        );
        assert!(change.is_ok());
        let change = change.unwrap();
        assert_eq!(change.file_path, "test.txt");
        assert_eq!(change.before, "");
        assert_eq!(change.after, "content");
        assert_eq!(change.change_type, ChangeType::Create);
    }

    #[test]
    fn test_change_modify_valid() {
        let change = Change::new(
            "test.txt",
            "old content",
            "new content",
            "Modify test file",
            ChangeType::Modify,
        );
        assert!(change.is_ok());
    }

    #[test]
    fn test_change_delete_valid() {
        let change = Change::new(
            "test.txt",
            "content",
            "",
            "Delete test file",
            ChangeType::Delete,
        );
        assert!(change.is_ok());
    }

    #[test]
    fn test_change_empty_file_path() {
        let change = Change::new("", "before", "after", "desc", ChangeType::Modify);
        assert!(change.is_err());
    }

    #[test]
    fn test_change_create_with_before_state() {
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "desc",
            ChangeType::Create,
        );
        assert!(change.is_err());
    }

    #[test]
    fn test_change_delete_with_after_state() {
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "desc",
            ChangeType::Delete,
        );
        assert!(change.is_err());
    }

    #[test]
    fn test_change_tracker_track_single() {
        let mut tracker = ChangeTracker::new();
        let result = tracker.track_change(
            "test.txt",
            "before",
            "after",
            "Modify",
            ChangeType::Modify,
        );
        assert!(result.is_ok());
        assert_eq!(tracker.pending_count(), 1);
    }

    #[test]
    fn test_change_tracker_track_batch() {
        let mut tracker = ChangeTracker::new();
        let changes = vec![
            Change::new("file1.txt", "", "content1", "Create 1", ChangeType::Create).unwrap(),
            Change::new("file2.txt", "", "content2", "Create 2", ChangeType::Create).unwrap(),
        ];
        let result = tracker.track_batch(changes);
        assert!(result.is_ok());
        assert_eq!(tracker.pending_count(), 2);
    }

    #[test]
    fn test_change_tracker_clear_pending() {
        let mut tracker = ChangeTracker::new();
        tracker
            .track_change("test.txt", "before", "after", "Modify", ChangeType::Modify)
            .unwrap();
        assert_eq!(tracker.pending_count(), 1);
        tracker.clear_pending();
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn test_change_display() {
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "Modify test",
            ChangeType::Modify,
        )
        .unwrap();
        let display = format!("{}", change);
        assert!(display.contains("Modify"));
        assert!(display.contains("test.txt"));
    }

    #[test]
    fn test_change_serialization() {
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "Modify",
            ChangeType::Modify,
        )
        .unwrap();
        let json = serde_json::to_string(&change).unwrap();
        let deserialized: Change = serde_json::from_str(&json).unwrap();
        assert_eq!(change.id, deserialized.id);
        assert_eq!(change.file_path, deserialized.file_path);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for generating valid file paths
    fn file_path_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9_\-./]{1,50}\.rs"
            .prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid content
    fn content_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9\s\n\t]{0,200}"
            .prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid descriptions
    fn description_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9\s]{1,50}"
            .prop_map(|s| s.to_string())
    }

    proptest! {
        /// **Feature: ricecoder-undo-redo, Property 2: History Completeness**
        /// *For any* file modification operation, the change SHALL be recorded in the history
        /// with complete before/after state.
        /// **Validates: Requirements 1.1, 1.2, 1.3**
        #[test]
        fn prop_history_completeness_create(
            file_path in file_path_strategy(),
            content in content_strategy(),
            description in description_strategy(),
        ) {
            let mut tracker = ChangeTracker::new();

            // Track a create change
            let result = tracker.track_change(
                &file_path,
                "",
                &content,
                &description,
                ChangeType::Create,
            );

            prop_assert!(result.is_ok(), "Change tracking should succeed");

            let pending = tracker.get_pending_changes();
            prop_assert_eq!(pending.len(), 1, "Should have exactly one pending change");

            let change = &pending[0];
            prop_assert_eq!(&change.file_path, &file_path, "File path should match");
            prop_assert_eq!(&change.before, "", "Before state should be empty for Create");
            prop_assert_eq!(&change.after, &content, "After state should match content");
            prop_assert_eq!(&change.description, &description, "Description should match");
            prop_assert_eq!(change.change_type, ChangeType::Create, "Change type should be Create");
        }

        /// **Feature: ricecoder-undo-redo, Property 2: History Completeness**
        /// *For any* file modification operation, the change SHALL be recorded in the history
        /// with complete before/after state.
        /// **Validates: Requirements 1.1, 1.2, 1.3**
        #[test]
        fn prop_history_completeness_modify(
            file_path in file_path_strategy(),
            before_content in content_strategy(),
            after_content in content_strategy(),
            description in description_strategy(),
        ) {
            // Ensure before and after are different and non-empty for modify
            prop_assume!(before_content != after_content);
            prop_assume!(!before_content.is_empty());
            prop_assume!(!after_content.is_empty());

            let mut tracker = ChangeTracker::new();

            // Track a modify change
            let result = tracker.track_change(
                &file_path,
                &before_content,
                &after_content,
                &description,
                ChangeType::Modify,
            );

            prop_assert!(result.is_ok(), "Change tracking should succeed");

            let pending = tracker.get_pending_changes();
            prop_assert_eq!(pending.len(), 1, "Should have exactly one pending change");

            let change = &pending[0];
            prop_assert_eq!(&change.file_path, &file_path, "File path should match");
            prop_assert_eq!(&change.before, &before_content, "Before state should match");
            prop_assert_eq!(&change.after, &after_content, "After state should match");
            prop_assert_eq!(&change.description, &description, "Description should match");
            prop_assert_eq!(change.change_type, ChangeType::Modify, "Change type should be Modify");
        }

        /// **Feature: ricecoder-undo-redo, Property 2: History Completeness**
        /// *For any* file modification operation, the change SHALL be recorded in the history
        /// with complete before/after state.
        /// **Validates: Requirements 1.1, 1.2, 1.3**
        #[test]
        fn prop_history_completeness_delete(
            file_path in file_path_strategy(),
            content in content_strategy(),
            description in description_strategy(),
        ) {
            let mut tracker = ChangeTracker::new();

            // Track a delete change
            let result = tracker.track_change(
                &file_path,
                &content,
                "",
                &description,
                ChangeType::Delete,
            );

            prop_assert!(result.is_ok(), "Change tracking should succeed");

            let pending = tracker.get_pending_changes();
            prop_assert_eq!(pending.len(), 1, "Should have exactly one pending change");

            let change = &pending[0];
            prop_assert_eq!(&change.file_path, &file_path, "File path should match");
            prop_assert_eq!(&change.before, &content, "Before state should match content");
            prop_assert_eq!(&change.after, "", "After state should be empty for Delete");
            prop_assert_eq!(&change.description, &description, "Description should match");
            prop_assert_eq!(change.change_type, ChangeType::Delete, "Change type should be Delete");
        }

        /// **Feature: ricecoder-undo-redo, Property 2: History Completeness**
        /// *For any* sequence of file modifications, each modification SHALL be recorded
        /// in the history with complete before/after state.
        /// **Validates: Requirements 1.1, 1.2, 1.3**
        #[test]
        fn prop_history_completeness_batch(
            changes_data in prop::collection::vec(
                (file_path_strategy(), content_strategy(), content_strategy()),
                1..10
            ),
        ) {
            let mut tracker = ChangeTracker::new();
            let mut expected_changes = Vec::new();

            // Create changes with different types
            for (idx, (file_path, before, after)) in changes_data.iter().enumerate() {
                let change_type = match idx % 3 {
                    0 => ChangeType::Create,
                    1 => ChangeType::Modify,
                    _ => ChangeType::Delete,
                };

                // Skip invalid combinations
                if (change_type == ChangeType::Create && !before.is_empty()) ||
                   (change_type == ChangeType::Delete && !after.is_empty()) ||
                   (change_type == ChangeType::Modify && (before.is_empty() || after.is_empty())) {
                    continue;
                }

                if let Ok(change) = Change::new(
                    file_path.clone(),
                    before.clone(),
                    after.clone(),
                    format!("Change {}", idx),
                    change_type,
                ) {
                    expected_changes.push(change);
                }
            }

            // Track all changes
            let result = tracker.track_batch(expected_changes.clone());
            prop_assert!(result.is_ok(), "Batch tracking should succeed");

            let pending = tracker.get_pending_changes();
            prop_assert_eq!(
                pending.len(),
                expected_changes.len(),
                "All changes should be recorded"
            );

            // Verify each change is recorded with complete state
            for (recorded, expected) in pending.iter().zip(expected_changes.iter()) {
                prop_assert_eq!(&recorded.file_path, &expected.file_path, "File path should match");
                prop_assert_eq!(&recorded.before, &expected.before, "Before state should match");
                prop_assert_eq!(&recorded.after, &expected.after, "After state should match");
                prop_assert_eq!(recorded.change_type, expected.change_type, "Change type should match");
            }
        }
    }
}
