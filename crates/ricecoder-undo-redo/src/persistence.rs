//! Persistence layer for history storage

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{change::Change, checkpoint::Checkpoint, error::UndoRedoError};

/// Serializable history snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnapshot {
    /// All recorded changes
    pub changes: Vec<Change>,
    /// All checkpoints
    pub checkpoints: HashMap<String, Checkpoint>,
    /// Timestamp when snapshot was created
    pub snapshot_time: DateTime<Utc>,
}

impl HistorySnapshot {
    /// Create a new history snapshot
    pub fn new(changes: Vec<Change>, checkpoints: HashMap<String, Checkpoint>) -> Self {
        HistorySnapshot {
            changes,
            checkpoints,
            snapshot_time: Utc::now(),
        }
    }

    /// Validate the snapshot for consistency
    pub fn validate(&self) -> Result<(), UndoRedoError> {
        // Validate all changes
        for change in &self.changes {
            change.validate()?;
        }

        // Validate all checkpoints
        for checkpoint in self.checkpoints.values() {
            checkpoint.validate()?;
        }

        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total size in bytes
    pub size_bytes: u64,
    /// Number of entries
    pub entry_count: usize,
    /// Oldest entry timestamp
    pub oldest_entry: Option<DateTime<Utc>>,
    /// Newest entry timestamp
    pub newest_entry: Option<DateTime<Utc>>,
}

/// Manages history persistence to disk
pub struct HistoryStore {
    storage_path: PathBuf,
    history_data: Option<HistorySnapshot>,
    max_retries: usize,
}

impl HistoryStore {
    /// Create a new history store with the given storage path
    pub fn new(storage_path: impl AsRef<Path>) -> Result<Self, UndoRedoError> {
        let storage_path = storage_path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    UndoRedoError::storage_error(format!(
                        "Failed to create storage directory: {}",
                        e
                    ))
                })?;
            }
        }

        Ok(HistoryStore {
            storage_path,
            history_data: None,
            max_retries: 3,
        })
    }

    /// Save history to disk with retry logic
    pub fn save_history(&mut self, snapshot: HistorySnapshot) -> Result<(), UndoRedoError> {
        // Validate snapshot before saving
        snapshot.validate()?;

        let mut last_error = None;

        // Retry with exponential backoff
        for attempt in 0..self.max_retries {
            match self.save_history_attempt(&snapshot) {
                Ok(_) => {
                    self.history_data = Some(snapshot);
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries - 1 {
                        // Exponential backoff: 100ms, 200ms, 400ms
                        let backoff_ms = 100 * (2_u64.pow(attempt as u32));
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            UndoRedoError::storage_error("Failed to save history after retries")
        }))
    }

    /// Attempt to save history once
    fn save_history_attempt(&self, snapshot: &HistorySnapshot) -> Result<(), UndoRedoError> {
        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(&self.storage_path, json).map_err(|e| {
            UndoRedoError::storage_error(format!("Failed to write history file: {}", e))
        })?;
        Ok(())
    }

    /// Load history from disk with fallback to empty history
    pub fn load_history(&mut self) -> Result<HistorySnapshot, UndoRedoError> {
        // If file doesn't exist, return empty history
        if !self.storage_path.exists() {
            let empty_snapshot = HistorySnapshot::new(Vec::new(), HashMap::new());
            self.history_data = Some(empty_snapshot.clone());
            return Ok(empty_snapshot);
        }

        // Try to read and deserialize
        match self.load_history_attempt() {
            Ok(snapshot) => {
                self.history_data = Some(snapshot.clone());
                Ok(snapshot)
            }
            Err(e) => {
                // Log warning and return empty history on load failure
                eprintln!(
                    "Warning: Failed to load history: {}. Starting with empty history.",
                    e
                );
                let empty_snapshot = HistorySnapshot::new(Vec::new(), HashMap::new());
                self.history_data = Some(empty_snapshot.clone());
                Ok(empty_snapshot)
            }
        }
    }

    /// Attempt to load history once
    fn load_history_attempt(&self) -> Result<HistorySnapshot, UndoRedoError> {
        let content = fs::read_to_string(&self.storage_path).map_err(|e| {
            UndoRedoError::storage_error(format!("Failed to read history file: {}", e))
        })?;

        let snapshot: HistorySnapshot = serde_json::from_str(&content).map_err(|e| {
            UndoRedoError::storage_error(format!("Failed to deserialize history: {}", e))
        })?;

        // Validate loaded snapshot
        snapshot.validate()?;

        Ok(snapshot)
    }

    /// Clean up entries older than the specified number of days
    pub fn cleanup_old_entries(&mut self, retention_days: i64) -> Result<(), UndoRedoError> {
        let cutoff_time = Utc::now() - Duration::days(retention_days);

        if let Some(snapshot) = &mut self.history_data {
            // Filter out old changes
            let original_count = snapshot.changes.len();
            snapshot
                .changes
                .retain(|change| change.timestamp > cutoff_time);
            let removed_count = original_count - snapshot.changes.len();

            if removed_count > 0 {
                eprintln!(
                    "Cleaned up {} old history entries (older than {} days)",
                    removed_count, retention_days
                );
            }

            // Filter out old checkpoints
            let original_checkpoint_count = snapshot.checkpoints.len();
            snapshot
                .checkpoints
                .retain(|_, cp| cp.created_at > cutoff_time);
            let removed_checkpoint_count = original_checkpoint_count - snapshot.checkpoints.len();

            if removed_checkpoint_count > 0 {
                eprintln!(
                    "Cleaned up {} old checkpoints (older than {} days)",
                    removed_checkpoint_count, retention_days
                );
            }
        }

        Ok(())
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<StorageStats, UndoRedoError> {
        if !self.storage_path.exists() {
            return Ok(StorageStats {
                size_bytes: 0,
                entry_count: 0,
                oldest_entry: None,
                newest_entry: None,
            });
        }

        let metadata = fs::metadata(&self.storage_path).map_err(|e| {
            UndoRedoError::storage_error(format!("Failed to get file metadata: {}", e))
        })?;

        let size_bytes = metadata.len();

        let (entry_count, oldest_entry, newest_entry) = if let Some(snapshot) = &self.history_data {
            let entry_count = snapshot.changes.len() + snapshot.checkpoints.len();

            let oldest_entry = snapshot
                .changes
                .iter()
                .map(|c| c.timestamp)
                .chain(snapshot.checkpoints.values().map(|cp| cp.created_at))
                .min();

            let newest_entry = snapshot
                .changes
                .iter()
                .map(|c| c.timestamp)
                .chain(snapshot.checkpoints.values().map(|cp| cp.created_at))
                .max();

            (entry_count, oldest_entry, newest_entry)
        } else {
            (0, None, None)
        };

        Ok(StorageStats {
            size_bytes,
            entry_count,
            oldest_entry,
            newest_entry,
        })
    }

    /// Check if storage is full (exceeds 1GB limit)
    pub fn is_storage_full(&self) -> Result<bool, UndoRedoError> {
        const MAX_STORAGE_BYTES: u64 = 1024 * 1024 * 1024; // 1GB
        let stats = self.get_storage_stats()?;
        Ok(stats.size_bytes > MAX_STORAGE_BYTES)
    }

    /// Handle storage full gracefully by logging warning
    pub fn handle_storage_full(&self) -> Result<(), UndoRedoError> {
        if self.is_storage_full()? {
            eprintln!("Warning: History storage is full (>1GB). Consider cleaning up old entries.");
        }
        Ok(())
    }

    /// Get the current history data
    pub fn get_history_data(&self) -> Option<HistorySnapshot> {
        self.history_data.clone()
    }

    /// Set the history data
    pub fn set_history_data(&mut self, snapshot: HistorySnapshot) {
        self.history_data = Some(snapshot);
    }
}

/// Manages storage lifecycle including cleanup and size limits
pub struct StorageManager {
    store: HistoryStore,
    retention_days: i64,
    max_storage_bytes: u64,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new(
        storage_path: impl AsRef<Path>,
        retention_days: i64,
        max_storage_bytes: u64,
    ) -> Result<Self, UndoRedoError> {
        let store = HistoryStore::new(storage_path)?;
        Ok(StorageManager {
            store,
            retention_days,
            max_storage_bytes,
        })
    }

    /// Create a new storage manager with default settings (30 days retention, 1GB limit)
    pub fn with_defaults(storage_path: impl AsRef<Path>) -> Result<Self, UndoRedoError> {
        Self::new(storage_path, 30, 1024 * 1024 * 1024)
    }

    /// Perform automatic cleanup on session start
    pub fn cleanup_on_session_start(&mut self) -> Result<(), UndoRedoError> {
        // Load history first
        self.store.load_history()?;

        // Clean up old entries
        self.store.cleanup_old_entries(self.retention_days)?;

        // Check storage size and handle if full
        if self.store.is_storage_full()? {
            self.store.handle_storage_full()?;
        }

        Ok(())
    }

    /// Perform automatic cleanup on session end
    pub fn cleanup_on_session_end(&mut self) -> Result<(), UndoRedoError> {
        // Clean up old entries
        self.store.cleanup_old_entries(self.retention_days)?;

        // Check storage size and handle if full
        if self.store.is_storage_full()? {
            self.store.handle_storage_full()?;
        }

        Ok(())
    }

    /// Enforce storage size limit by removing oldest entries
    pub fn enforce_storage_limit(&mut self) -> Result<(), UndoRedoError> {
        let stats = self.store.get_storage_stats()?;

        if stats.size_bytes > self.max_storage_bytes {
            eprintln!(
                "Storage limit exceeded: {} bytes > {} bytes. Removing oldest entries.",
                stats.size_bytes, self.max_storage_bytes
            );

            // Remove entries older than retention period
            self.store.cleanup_old_entries(self.retention_days)?;

            // If still over limit, remove more aggressively
            let stats_after = self.store.get_storage_stats()?;
            if stats_after.size_bytes > self.max_storage_bytes {
                eprintln!("Still over limit after cleanup. Removing entries older than 7 days.");
                self.store.cleanup_old_entries(7)?;
            }
        }

        Ok(())
    }

    /// Get the underlying history store
    pub fn get_store(&self) -> &HistoryStore {
        &self.store
    }

    /// Get the underlying history store mutably
    pub fn get_store_mut(&mut self) -> &mut HistoryStore {
        &mut self.store
    }

    /// Set retention days
    pub fn set_retention_days(&mut self, days: i64) {
        self.retention_days = days;
    }

    /// Set maximum storage bytes
    pub fn set_max_storage_bytes(&mut self, bytes: u64) {
        self.max_storage_bytes = bytes;
    }

    /// Get retention days
    pub fn get_retention_days(&self) -> i64 {
        self.retention_days
    }

    /// Get maximum storage bytes
    pub fn get_max_storage_bytes(&self) -> u64 {
        self.max_storage_bytes
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::change::ChangeType;

    #[test]
    fn test_history_store_create() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");
        let store = HistoryStore::new(&store_path);
        assert!(store.is_ok());
    }

    #[test]
    fn test_history_store_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        // Create and save
        let mut store = HistoryStore::new(&store_path).unwrap();
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "Test change",
            ChangeType::Modify,
        )
        .unwrap();
        let snapshot = HistorySnapshot::new(vec![change.clone()], HashMap::new());
        store.save_history(snapshot).unwrap();

        // Load and verify
        let mut store2 = HistoryStore::new(&store_path).unwrap();
        let loaded = store2.load_history().unwrap();
        assert_eq!(loaded.changes.len(), 1);
        assert_eq!(loaded.changes[0].id, change.id);
    }

    #[test]
    fn test_history_store_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("nonexistent.json");

        let mut store = HistoryStore::new(&store_path).unwrap();
        let loaded = store.load_history().unwrap();
        assert_eq!(loaded.changes.len(), 0);
        assert_eq!(loaded.checkpoints.len(), 0);
    }

    #[test]
    fn test_history_store_cleanup_old_entries() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut store = HistoryStore::new(&store_path).unwrap();

        // Create changes with different timestamps
        let mut changes = Vec::new();
        for i in 0..3 {
            let change = Change::new(
                "test.txt",
                "before",
                "after",
                &format!("Change {}", i),
                ChangeType::Modify,
            )
            .unwrap();
            changes.push(change);
        }

        let snapshot = HistorySnapshot::new(changes, HashMap::new());
        store.set_history_data(snapshot);

        // Cleanup entries older than 0 days (should remove all)
        store.cleanup_old_entries(0).unwrap();

        let stats = store.get_storage_stats().unwrap();
        assert_eq!(stats.entry_count, 0);
    }

    #[test]
    fn test_history_store_get_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut store = HistoryStore::new(&store_path).unwrap();
        let change =
            Change::new("test.txt", "before", "after", "Test", ChangeType::Modify).unwrap();
        let snapshot = HistorySnapshot::new(vec![change], HashMap::new());
        store.save_history(snapshot).unwrap();

        let stats = store.get_storage_stats().unwrap();
        assert!(stats.size_bytes > 0);
        assert_eq!(stats.entry_count, 1);
        assert!(stats.oldest_entry.is_some());
        assert!(stats.newest_entry.is_some());
    }

    #[test]
    fn test_history_store_is_storage_full() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut store = HistoryStore::new(&store_path).unwrap();
        let change =
            Change::new("test.txt", "before", "after", "Test", ChangeType::Modify).unwrap();
        let snapshot = HistorySnapshot::new(vec![change], HashMap::new());
        store.save_history(snapshot).unwrap();

        let is_full = store.is_storage_full().unwrap();
        assert!(!is_full); // Small test file should not be full
    }

    #[test]
    fn test_history_snapshot_validate() {
        let change =
            Change::new("test.txt", "before", "after", "Test", ChangeType::Modify).unwrap();
        let snapshot = HistorySnapshot::new(vec![change], HashMap::new());
        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn test_history_store_save_with_retry() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut store = HistoryStore::new(&store_path).unwrap();
        let change =
            Change::new("test.txt", "before", "after", "Test", ChangeType::Modify).unwrap();
        let snapshot = HistorySnapshot::new(vec![change], HashMap::new());

        // Should succeed on first attempt
        let result = store.save_history(snapshot);
        assert!(result.is_ok());
    }

    #[test]
    fn test_history_store_load_corrupted_file() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        // Write corrupted JSON
        fs::write(&store_path, "{ invalid json }").unwrap();

        let mut store = HistoryStore::new(&store_path).unwrap();
        // Should fall back to empty history on load failure
        let loaded = store.load_history().unwrap();
        assert_eq!(loaded.changes.len(), 0);
    }

    #[test]
    fn test_history_store_multiple_changes_and_checkpoints() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut store = HistoryStore::new(&store_path).unwrap();

        // Create multiple changes
        let mut changes = Vec::new();
        for i in 0..5 {
            let change = Change::new(
                &format!("file{}.txt", i),
                "before",
                "after",
                &format!("Change {}", i),
                ChangeType::Modify,
            )
            .unwrap();
            changes.push(change);
        }

        // Create checkpoints
        let mut checkpoints = HashMap::new();
        for i in 0..2 {
            let mut file_states = HashMap::new();
            file_states.insert(format!("file{}.txt", i), format!("content{}", i));
            let checkpoint =
                Checkpoint::new(format!("Checkpoint {}", i), "description", file_states).unwrap();
            checkpoints.insert(checkpoint.id.clone(), checkpoint);
        }

        let snapshot = HistorySnapshot::new(changes, checkpoints);
        store.save_history(snapshot).unwrap();

        // Load and verify
        let mut store2 = HistoryStore::new(&store_path).unwrap();
        let loaded = store2.load_history().unwrap();
        assert_eq!(loaded.changes.len(), 5);
        assert_eq!(loaded.checkpoints.len(), 2);
    }

    #[test]
    fn test_storage_manager_create() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");
        let manager = StorageManager::with_defaults(&store_path);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_storage_manager_cleanup_on_session_start() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut manager = StorageManager::with_defaults(&store_path).unwrap();
        let result = manager.cleanup_on_session_start();
        assert!(result.is_ok());
    }

    #[test]
    fn test_storage_manager_cleanup_on_session_end() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut manager = StorageManager::with_defaults(&store_path).unwrap();
        let result = manager.cleanup_on_session_end();
        assert!(result.is_ok());
    }

    #[test]
    fn test_storage_manager_enforce_storage_limit() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut manager = StorageManager::new(&store_path, 30, 1024 * 1024 * 1024).unwrap();

        // Create some changes
        let change =
            Change::new("test.txt", "before", "after", "Test", ChangeType::Modify).unwrap();
        let snapshot = HistorySnapshot::new(vec![change], HashMap::new());
        manager.get_store_mut().save_history(snapshot).unwrap();

        // Enforce limit (should not fail for small file)
        let result = manager.enforce_storage_limit();
        assert!(result.is_ok());
    }

    #[test]
    fn test_storage_manager_retention_days() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut manager = StorageManager::with_defaults(&store_path).unwrap();
        assert_eq!(manager.get_retention_days(), 30);

        manager.set_retention_days(7);
        assert_eq!(manager.get_retention_days(), 7);
    }

    #[test]
    fn test_storage_manager_max_storage_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("history.json");

        let mut manager = StorageManager::with_defaults(&store_path).unwrap();
        assert_eq!(manager.get_max_storage_bytes(), 1024 * 1024 * 1024);

        manager.set_max_storage_bytes(512 * 1024 * 1024);
        assert_eq!(manager.get_max_storage_bytes(), 512 * 1024 * 1024);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use tempfile::TempDir;

    use super::*;
    use crate::change::ChangeType;

    /// Strategy for generating valid file paths
    fn file_path_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9_\-./]{1,50}\.rs".prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid content
    fn content_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9\s]{1,100}".prop_map(|s| s.to_string())
    }

    proptest! {
        /// **Feature: ricecoder-undo-redo, Property 6: Persistence Round-Trip**
        /// *For any* history state, saving and loading SHALL produce an equivalent history
        /// with all changes intact.
        /// **Validates: Requirements 5.1, 5.2, 5.3**
        #[test]
        fn prop_persistence_round_trip_small(
            changes_data in prop::collection::vec(
                (file_path_strategy(), content_strategy(), content_strategy()),
                1..10
            ),
        ) {
            let temp_dir = TempDir::new().unwrap();
            let store_path = temp_dir.path().join("history.json");

            let mut changes = Vec::new();
            for (idx, (file_path, before, after)) in changes_data.iter().enumerate() {
                prop_assume!(before != after);

                if let Ok(change) = Change::new(
                    file_path.clone(),
                    before.clone(),
                    after.clone(),
                    format!("Change {}", idx),
                    ChangeType::Modify,
                ) {
                    changes.push(change);
                }
            }

            // Save
            let mut store = HistoryStore::new(&store_path).unwrap();
            let snapshot = HistorySnapshot::new(changes.clone(), HashMap::new());
            store.save_history(snapshot).unwrap();

            // Load
            let mut store2 = HistoryStore::new(&store_path).unwrap();
            let loaded = store2.load_history().unwrap();

            // Verify
            prop_assert_eq!(
                loaded.changes.len(),
                changes.len(),
                "Loaded changes count should match saved"
            );

            for (saved, loaded_change) in changes.iter().zip(loaded.changes.iter()) {
                prop_assert_eq!(&saved.id, &loaded_change.id, "Change IDs should match");
                prop_assert_eq!(
                    &saved.file_path, &loaded_change.file_path,
                    "File paths should match"
                );
                prop_assert_eq!(
                    &saved.before, &loaded_change.before,
                    "Before states should match"
                );
                prop_assert_eq!(
                    &saved.after, &loaded_change.after,
                    "After states should match"
                );
            }
        }

        /// **Feature: ricecoder-undo-redo, Property 6: Persistence Round-Trip**
        /// *For any* history state with varying sizes, saving and loading SHALL preserve
        /// all data exactly.
        /// **Validates: Requirements 5.1, 5.2, 5.3**
        #[test]
        fn prop_persistence_round_trip_medium(
            changes_data in prop::collection::vec(
                (file_path_strategy(), content_strategy(), content_strategy()),
                1..100
            ),
        ) {
            let temp_dir = TempDir::new().unwrap();
            let store_path = temp_dir.path().join("history.json");

            let mut changes = Vec::new();
            for (idx, (file_path, before, after)) in changes_data.iter().enumerate() {
                prop_assume!(before != after);

                if let Ok(change) = Change::new(
                    file_path.clone(),
                    before.clone(),
                    after.clone(),
                    format!("Change {}", idx),
                    ChangeType::Modify,
                ) {
                    changes.push(change);
                }
            }

            // Save
            let mut store = HistoryStore::new(&store_path).unwrap();
            let snapshot = HistorySnapshot::new(changes.clone(), HashMap::new());
            store.save_history(snapshot).unwrap();

            // Load
            let mut store2 = HistoryStore::new(&store_path).unwrap();
            let loaded = store2.load_history().unwrap();

            // Verify
            prop_assert_eq!(
                loaded.changes.len(),
                changes.len(),
                "Loaded changes count should match saved"
            );

            for (saved, loaded_change) in changes.iter().zip(loaded.changes.iter()) {
                prop_assert_eq!(&saved.id, &loaded_change.id, "Change IDs should match");
                prop_assert_eq!(
                    &saved.file_path, &loaded_change.file_path,
                    "File paths should match"
                );
                prop_assert_eq!(
                    &saved.before, &loaded_change.before,
                    "Before states should match"
                );
                prop_assert_eq!(
                    &saved.after, &loaded_change.after,
                    "After states should match"
                );
            }
        }
    }
}
