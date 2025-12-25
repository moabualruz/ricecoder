//! File Change Tracking with Debouncing
//!
//! Tracks file changes for deduplication and batching during watch mode.

use ricegrep::indexing_optimization::{DebounceBuffer, FileChangeEvent, FileChangeKind};
use std::path::PathBuf;
use std::time::Duration;

/// Tracks file changes for deduplication and batching
#[derive(Debug)]
pub struct ChangeTracker {
    /// Internal debounce buffer for event batching
    debounce_buffer: DebounceBuffer,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            debounce_buffer: DebounceBuffer::new(),
        }
    }

    /// Create with custom debounce window duration
    pub fn with_duration(duration: Duration) -> Self {
        Self {
            debounce_buffer: DebounceBuffer::with_duration(duration),
        }
    }

    /// Record a file change
    pub fn record_change(&mut self, path: PathBuf, kind: FileChangeKind) {
        let event = FileChangeEvent {
            path,
            kind,
            timestamp: std::time::Instant::now(),
        };
        self.debounce_buffer.collect_event(event);
    }

    /// Check if debounce window has expired and ready for processing
    pub fn is_ready(&self) -> bool {
        self.debounce_buffer.is_ready()
    }

    /// Get and clear all accumulated changes
    pub fn take_changes(&mut self) -> Vec<PathBuf> {
        self.debounce_buffer
            .take_events()
            .into_iter()
            .map(|e| e.path)
            .collect()
    }

    /// Check if there are pending changes
    pub fn has_changes(&self) -> bool {
        self.debounce_buffer.has_events()
    }

    /// Get count of tracked changes
    pub fn change_count(&self) -> usize {
        self.debounce_buffer.event_count()
    }

    /// Get remaining time until debounce window expires
    pub fn remaining(&self) -> Option<Duration> {
        self.debounce_buffer.remaining()
    }

    /// Get current debouncing statistics
    pub fn stats(&self) -> ricegrep::indexing_optimization::DebouncingStats {
        self.debounce_buffer.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_tracker_basic() {
        let mut tracker = ChangeTracker::new();
        let path = PathBuf::from("test.txt");

        assert!(!tracker.has_changes());
        tracker.record_change(path.clone(), FileChangeKind::Modify);

        // Should only have 1 entry (latest timestamp)
        assert_eq!(tracker.change_count(), 1);
    }

    #[test]
    fn test_change_tracker_multiple_files() {
        let mut tracker = ChangeTracker::new();
        let path1 = PathBuf::from("file1.txt");
        let path2 = PathBuf::from("file2.txt");
        let path3 = PathBuf::from("file3.txt");

        tracker.record_change(path1.clone(), FileChangeKind::Modify);
        tracker.record_change(path2.clone(), FileChangeKind::Create);
        tracker.record_change(path3.clone(), FileChangeKind::Modify);

        assert_eq!(tracker.change_count(), 3);
        assert!(tracker.has_changes());
    }

    #[test]
    fn test_change_tracker_take_changes() {
        let mut tracker = ChangeTracker::new();
        let path1 = PathBuf::from("file1.txt");
        let path2 = PathBuf::from("file2.txt");

        tracker.record_change(path1.clone(), FileChangeKind::Modify);
        tracker.record_change(path2.clone(), FileChangeKind::Modify);

        let changes = tracker.take_changes();

        // Should have returned 2 changes
        assert_eq!(changes.len(), 2);

        // Should be empty after taking
        assert!(!tracker.has_changes());
        assert_eq!(tracker.change_count(), 0);
    }

    #[test]
    fn test_change_tracker_timestamps() {
        let mut tracker = ChangeTracker::new();
        let path = PathBuf::from("test.txt");

        let _before = std::time::SystemTime::now();
        tracker.record_change(path.clone(), FileChangeKind::Modify);
        let _after = std::time::SystemTime::now();

        // Verify that the change was recorded within timestamp bounds
        let changes = tracker.take_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], path);
    }

    #[test]
    fn test_change_tracker_rapid_changes() {
        let mut tracker = ChangeTracker::new();
        let path = PathBuf::from("rapid_changes.txt");

        // Rapid updates to same file
        for _ in 0..100 {
            tracker.record_change(path.clone(), FileChangeKind::Modify);
        }

        // Should only have 1 entry with latest timestamp
        assert_eq!(tracker.change_count(), 1);
        let changes = tracker.take_changes();
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_change_tracker_deduplication() {
        let mut tracker = ChangeTracker::new();
        let path = PathBuf::from("test.txt");

        // Record same file multiple times
        tracker.record_change(path.clone(), FileChangeKind::Modify);
        tracker.record_change(path.clone(), FileChangeKind::Modify);
        tracker.record_change(path.clone(), FileChangeKind::Modify);

        // Should only have 1 entry (latest timestamp)
        assert_eq!(tracker.change_count(), 1);
    }
}
