//! Event debouncing for incremental indexing.
//!
//! Implements TIER 2 (Medium Path) of the hybrid indexing strategy:
//! - Accumulate file change events over 200-300ms window
//! - Deduplicate events (keep only latest per file)
//! - Process batch when window expires
//! - Reduces index updates by 50-80% for rapid file changes

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Reasons for file changes that trigger debouncing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeKind {
    Create,
    Modify,
    Delete,
}

/// Single file change event
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub kind: FileChangeKind,
    pub timestamp: Instant,
}

/// Accumulates file change events with deduplication
/// 
/// Batches rapid file changes over a configurable window (200-300ms)
/// to reduce redundant index updates. Multiple changes to the same file
/// within the window are deduplicated - only the latest is kept.
#[derive(Debug)]
pub struct DebounceBuffer {
    /// Accumulated changes: path -> latest event
    events: HashMap<PathBuf, FileChangeEvent>,
    
    /// When this buffer started accumulating (Instant)
    window_start: Instant,
    
    /// Debounce window duration (default 250ms)
    window_duration: Duration,
}

impl DebounceBuffer {
    /// Create new debounce buffer with default 250ms window
    pub fn new() -> Self {
        Self::with_duration(Duration::from_millis(250))
    }
    
    /// Create with custom window duration
    pub fn with_duration(window_duration: Duration) -> Self {
        Self {
            events: HashMap::new(),
            window_start: Instant::now(),
            window_duration,
        }
    }
    
    /// Add or update event for a file
    /// 
    /// If file already in buffer, updates with new event (deduplication)
    pub fn collect_event(&mut self, event: FileChangeEvent) {
        self.events.insert(event.path.clone(), event);
    }
    
    /// Check if debounce window has elapsed
    pub fn is_ready(&self) -> bool {
        self.window_start.elapsed() >= self.window_duration
    }
    
    /// Get elapsed time since window started
    pub fn elapsed(&self) -> Duration {
        self.window_start.elapsed()
    }
    
    /// Get remaining time until window expires
    pub fn remaining(&self) -> Option<Duration> {
        self.window_duration.checked_sub(self.elapsed())
    }
    
    /// Get current number of buffered events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
    
    /// Check if buffer has any events
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }
    
    /// Extract accumulated events and reset buffer
    /// 
    /// Returns all deduplicated events and resets window timer
    pub fn take_events(&mut self) -> Vec<FileChangeEvent> {
        self.window_start = Instant::now();
        let events = self.events.values().cloned().collect();
        self.events.clear();
        events
    }
    
    /// Extract events without resetting window
    pub fn peek_events(&self) -> Vec<FileChangeEvent> {
        self.events.values().cloned().collect()
    }
    
    /// Clear buffer without extracting
    pub fn clear(&mut self) {
        self.events.clear();
        self.window_start = Instant::now();
    }
    
    /// Get statistics about current buffer state
    pub fn stats(&self) -> DebouncingStats {
        let changed_files = self.events.keys().cloned().collect();
        let create_count = self.events.values().filter(|e| e.kind == FileChangeKind::Create).count();
        let modify_count = self.events.values().filter(|e| e.kind == FileChangeKind::Modify).count();
        let delete_count = self.events.values().filter(|e| e.kind == FileChangeKind::Delete).count();
        
        DebouncingStats {
            buffered_events: self.events.len(),
            changed_files,
            creates: create_count,
            modifies: modify_count,
            deletes: delete_count,
            window_remaining: self.remaining(),
        }
    }
}

impl Default for DebounceBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DebouncingStats {
    pub buffered_events: usize,
    pub changed_files: Vec<PathBuf>,
    pub creates: usize,
    pub modifies: usize,
    pub deletes: usize,
    pub window_remaining: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_debounce_buffer_creation() {
        let buffer = DebounceBuffer::new();
        assert_eq!(buffer.event_count(), 0);
        assert!(!buffer.has_events());
    }
    
    #[test]
    fn test_take_events_resets_window() {
        let mut buffer = DebounceBuffer::with_duration(Duration::from_millis(100));
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("test.txt"),
            kind: FileChangeKind::Modify,
            timestamp: Instant::now(),
        });
        
        // Should not be ready yet (just started)
        assert!(!buffer.is_ready());
        
        let events = buffer.take_events();
        
        assert_eq!(events.len(), 1);
        assert_eq!(buffer.event_count(), 0);
        // After taking, window should be reset, so not ready immediately
        assert!(!buffer.is_ready());
        assert!(buffer.remaining().is_some());
    }
    
    #[test]
    fn test_deduplicate_same_file() {
        let mut buffer = DebounceBuffer::new();
        
        let event1 = FileChangeEvent {
            path: PathBuf::from("test.txt"),
            kind: FileChangeKind::Modify,
            timestamp: Instant::now(),
        };
        
        buffer.collect_event(event1);
        assert_eq!(buffer.event_count(), 1);
        
        // Add another event for same file
        let event2 = FileChangeEvent {
            path: PathBuf::from("test.txt"),
            kind: FileChangeKind::Modify,
            timestamp: Instant::now(),
        };
        
        let event2_timestamp = event2.timestamp;
        buffer.collect_event(event2);
        
        // Should still be 1 (deduplicated)
        assert_eq!(buffer.event_count(), 1);
        
        // Should have latest timestamp
        let events = buffer.peek_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].timestamp, event2_timestamp);
    }
    
    #[test]
    fn test_multiple_files() {
        let mut buffer = DebounceBuffer::new();
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("file1.txt"),
            kind: FileChangeKind::Modify,
            timestamp: Instant::now(),
        });
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("file2.txt"),
            kind: FileChangeKind::Create,
            timestamp: Instant::now(),
        });
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("file3.txt"),
            kind: FileChangeKind::Delete,
            timestamp: Instant::now(),
        });
        
        assert_eq!(buffer.event_count(), 3);
    }
    
    #[test]
    fn test_is_ready_timing() {
        let mut buffer = DebounceBuffer::with_duration(Duration::from_millis(50));
        
        // Should not be ready immediately
        assert!(!buffer.is_ready());
        assert!(buffer.remaining().is_some());
        
        // Wait for window to expire
        thread::sleep(Duration::from_millis(60));
        
        // Should be ready now
        assert!(buffer.is_ready());
        // Remaining should be zero or negative (None or Zero)
        let remaining = buffer.remaining();
        assert!(remaining.is_none() || remaining == Some(Duration::ZERO));
    }
    
    #[test]
    fn test_peek_does_not_reset() {
        let mut buffer = DebounceBuffer::new();
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("test.txt"),
            kind: FileChangeKind::Modify,
            timestamp: Instant::now(),
        });
        
        let elapsed_before = buffer.elapsed();
        let events = buffer.peek_events();
        let elapsed_after = buffer.elapsed();
        
        assert_eq!(events.len(), 1);
        // Peek should not reset, so elapsed grows
        assert!(elapsed_after >= elapsed_before);
    }
    
    #[test]
    fn test_clear_buffer() {
        let mut buffer = DebounceBuffer::new();
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("test.txt"),
            kind: FileChangeKind::Modify,
            timestamp: Instant::now(),
        });
        
        assert!(buffer.has_events());
        buffer.clear();
        assert!(!buffer.has_events());
        assert_eq!(buffer.event_count(), 0);
    }
    
    #[test]
    fn test_stats() {
        let mut buffer = DebounceBuffer::new();
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("file1.txt"),
            kind: FileChangeKind::Create,
            timestamp: Instant::now(),
        });
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("file2.txt"),
            kind: FileChangeKind::Modify,
            timestamp: Instant::now(),
        });
        
        buffer.collect_event(FileChangeEvent {
            path: PathBuf::from("file3.txt"),
            kind: FileChangeKind::Delete,
            timestamp: Instant::now(),
        });
        
        let stats = buffer.stats();
        assert_eq!(stats.buffered_events, 3);
        assert_eq!(stats.creates, 1);
        assert_eq!(stats.modifies, 1);
        assert_eq!(stats.deletes, 1);
        assert_eq!(stats.changed_files.len(), 3);
    }
    
    #[test]
    fn test_rapid_edits_deduplication() {
        let mut buffer = DebounceBuffer::new();
        
        // Simulate 10 rapid edits to same file
        for _ in 0..10 {
            buffer.collect_event(FileChangeEvent {
                path: PathBuf::from("test.txt"),
                kind: FileChangeKind::Modify,
                timestamp: Instant::now(),
            });
        }
        
        // Should deduplicate to 1
        assert_eq!(buffer.event_count(), 1);
        
        let events = buffer.take_events();
        assert_eq!(events.len(), 1);
    }
}
