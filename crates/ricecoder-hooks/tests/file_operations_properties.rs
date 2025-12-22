//! Property-based tests for file operation events
//!
//! **Feature: ricecoder-hooks, Property 9: File operation event accuracy**
//! **Validates: Requirements Hooks-4.5.1 through Hooks-4.5.8**
//!
//! These tests verify that file operation events are emitted with correct event types,
//! accurate file paths, and complete metadata.

use std::path::PathBuf;

use proptest::prelude::*;
use ricecoder_hooks::events::{DirectoryOperationEvent, FileOperationEvent, FileSystemMonitor};

// Strategy for generating valid file paths
fn arb_file_path() -> impl Strategy<Value = PathBuf> {
    r"[a-zA-Z0-9_\-./]{1,50}\.rs".prop_map(PathBuf::from)
}

// Strategy for generating valid directory paths
fn arb_dir_path() -> impl Strategy<Value = PathBuf> {
    r"[a-zA-Z0-9_\-./]{1,50}".prop_map(PathBuf::from)
}

// Strategy for generating file sizes
fn arb_file_size() -> impl Strategy<Value = u64> {
    0u64..1_000_000u64
}

// Strategy for generating hash strings
fn arb_hash() -> impl Strategy<Value = String> {
    r"[a-f0-9]{32}".prop_map(|s| s.to_string())
}

proptest! {
    /// Property 9.1: File created events have correct event type and metadata
    ///
    /// For any file path and size, emitting a file created event should produce
    /// a FileOperationEvent::Created variant with the correct path and size.
    #[test]
    fn prop_file_created_event_accuracy(
        path in arb_file_path(),
        size in arb_file_size()
    ) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_file_created(path.clone(), size);

        // Verify correct event type
        match event {
            FileOperationEvent::Created { path: p, size: s, timestamp } => {
                // Verify path is accurate
                prop_assert_eq!(p, path);
                // Verify size is accurate
                prop_assert_eq!(s, size);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Created event"));
            }
        }
    }

    /// Property 9.2: File modified events have correct event type and metadata
    ///
    /// For any file path and hashes, emitting a file modified event should produce
    /// a FileOperationEvent::Modified variant with correct path and hashes.
    #[test]
    fn prop_file_modified_event_accuracy(
        path in arb_file_path(),
        old_hash in arb_hash(),
        new_hash in arb_hash()
    ) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_file_modified(path.clone(), old_hash.clone(), new_hash.clone());

        // Verify correct event type
        match event {
            FileOperationEvent::Modified { path: p, old_hash: oh, new_hash: nh, timestamp } => {
                // Verify path is accurate
                prop_assert_eq!(p, path);
                // Verify old hash is accurate
                prop_assert_eq!(oh, old_hash);
                // Verify new hash is accurate
                prop_assert_eq!(nh, new_hash);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Modified event"));
            }
        }
    }

    /// Property 9.3: File deleted events have correct event type and metadata
    ///
    /// For any file path, emitting a file deleted event should produce
    /// a FileOperationEvent::Deleted variant with the correct path.
    #[test]
    fn prop_file_deleted_event_accuracy(path in arb_file_path()) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_file_deleted(path.clone());

        // Verify correct event type
        match event {
            FileOperationEvent::Deleted { path: p, timestamp } => {
                // Verify path is accurate
                prop_assert_eq!(p, path);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Deleted event"));
            }
        }
    }

    /// Property 9.4: File renamed events have correct event type and metadata
    ///
    /// For any old and new file paths, emitting a file renamed event should produce
    /// a FileOperationEvent::Renamed variant with correct paths.
    #[test]
    fn prop_file_renamed_event_accuracy(
        old_path in arb_file_path(),
        new_path in arb_file_path()
    ) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_file_renamed(old_path.clone(), new_path.clone());

        // Verify correct event type
        match event {
            FileOperationEvent::Renamed { old_path: op, new_path: np, timestamp } => {
                // Verify old path is accurate
                prop_assert_eq!(op, old_path);
                // Verify new path is accurate
                prop_assert_eq!(np, new_path);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Renamed event"));
            }
        }
    }

    /// Property 9.5: File moved events have correct event type and metadata
    ///
    /// For any old and new file paths, emitting a file moved event should produce
    /// a FileOperationEvent::Moved variant with correct paths.
    #[test]
    fn prop_file_moved_event_accuracy(
        old_path in arb_file_path(),
        new_path in arb_file_path()
    ) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_file_moved(old_path.clone(), new_path.clone());

        // Verify correct event type
        match event {
            FileOperationEvent::Moved { old_path: op, new_path: np, timestamp } => {
                // Verify old path is accurate
                prop_assert_eq!(op, old_path);
                // Verify new path is accurate
                prop_assert_eq!(np, new_path);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Moved event"));
            }
        }
    }

    /// Property 9.6: File read events have correct event type and metadata
    ///
    /// For any file path, emitting a file read event should produce
    /// a FileOperationEvent::Read variant with the correct path.
    #[test]
    fn prop_file_read_event_accuracy(path in arb_file_path()) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_file_read(path.clone());

        // Verify correct event type
        match event {
            FileOperationEvent::Read { path: p, timestamp } => {
                // Verify path is accurate
                prop_assert_eq!(p, path);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Read event"));
            }
        }
    }

    /// Property 9.7: Directory created events have correct event type and metadata
    ///
    /// For any directory path, emitting a directory created event should produce
    /// a DirectoryOperationEvent::Created variant with the correct path.
    #[test]
    fn prop_directory_created_event_accuracy(path in arb_dir_path()) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_directory_created(path.clone());

        // Verify correct event type
        match event {
            DirectoryOperationEvent::Created { path: p, timestamp } => {
                // Verify path is accurate
                prop_assert_eq!(p, path);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Created event"));
            }
        }
    }

    /// Property 9.8: Directory deleted events have correct event type and metadata
    ///
    /// For any directory path, emitting a directory deleted event should produce
    /// a DirectoryOperationEvent::Deleted variant with the correct path.
    #[test]
    fn prop_directory_deleted_event_accuracy(path in arb_dir_path()) {
        let monitor = FileSystemMonitor::new();
        let event = monitor.emit_directory_deleted(path.clone());

        // Verify correct event type
        match event {
            DirectoryOperationEvent::Deleted { path: p, timestamp } => {
                // Verify path is accurate
                prop_assert_eq!(p, path);
                // Verify timestamp is present
                prop_assert!(timestamp <= std::time::SystemTime::now());
            }
            _ => {
                return Err(TestCaseError::fail("Expected Deleted event"));
            }
        }
    }
}
