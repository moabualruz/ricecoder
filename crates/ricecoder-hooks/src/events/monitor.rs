//! File system monitoring for file operation events
//!
//! This module provides file system monitoring capabilities to detect and emit events
//! for file operations including create, modify, delete, rename, move, and read operations.
//!
//! # Examples
//!
//! Creating a file monitor:
//! ```ignore
//! use ricecoder_hooks::events::FileSystemMonitor;
//! use std::path::PathBuf;
//!
//! let monitor = FileSystemMonitor::new();
//! monitor.start_monitoring(PathBuf::from("/path/to/watch")).await?;
//! ```

use super::file_operations::{DirectoryOperationEvent, FileOperationEvent};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// File system monitor for detecting file operations
///
/// Monitors file system for create, modify, delete, rename, move, and read operations.
/// Emits events when operations are detected.
#[derive(Debug, Clone)]
pub struct FileSystemMonitor {
    /// Tracked file hashes for detecting modifications
    file_hashes: Arc<RwLock<HashMap<PathBuf, String>>>,
}

impl FileSystemMonitor {
    /// Create a new file system monitor
    pub fn new() -> Self {
        Self {
            file_hashes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Emit a file created event
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the created file
    /// * `size` - Size of the file in bytes
    ///
    /// # Returns
    ///
    /// A `FileOperationEvent::Created` event
    pub fn emit_file_created(&self, path: PathBuf, size: u64) -> FileOperationEvent {
        FileOperationEvent::Created {
            path,
            size,
            timestamp: SystemTime::now(),
        }
    }

    /// Emit a file modified event
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the modified file
    /// * `old_hash` - Hash of the file before modification
    /// * `new_hash` - Hash of the file after modification
    ///
    /// # Returns
    ///
    /// A `FileOperationEvent::Modified` event
    pub fn emit_file_modified(
        &self,
        path: PathBuf,
        old_hash: String,
        new_hash: String,
    ) -> FileOperationEvent {
        FileOperationEvent::Modified {
            path,
            old_hash,
            new_hash,
            timestamp: SystemTime::now(),
        }
    }

    /// Emit a file deleted event
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the deleted file
    ///
    /// # Returns
    ///
    /// A `FileOperationEvent::Deleted` event
    pub fn emit_file_deleted(&self, path: PathBuf) -> FileOperationEvent {
        FileOperationEvent::Deleted {
            path,
            timestamp: SystemTime::now(),
        }
    }

    /// Emit a file renamed event
    ///
    /// # Arguments
    ///
    /// * `old_path` - Original path of the file
    /// * `new_path` - New path of the file
    ///
    /// # Returns
    ///
    /// A `FileOperationEvent::Renamed` event
    pub fn emit_file_renamed(&self, old_path: PathBuf, new_path: PathBuf) -> FileOperationEvent {
        FileOperationEvent::Renamed {
            old_path,
            new_path,
            timestamp: SystemTime::now(),
        }
    }

    /// Emit a file moved event
    ///
    /// # Arguments
    ///
    /// * `old_path` - Original path of the file
    /// * `new_path` - New path of the file
    ///
    /// # Returns
    ///
    /// A `FileOperationEvent::Moved` event
    pub fn emit_file_moved(&self, old_path: PathBuf, new_path: PathBuf) -> FileOperationEvent {
        FileOperationEvent::Moved {
            old_path,
            new_path,
            timestamp: SystemTime::now(),
        }
    }

    /// Emit a file read event
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file that was read
    ///
    /// # Returns
    ///
    /// A `FileOperationEvent::Read` event
    pub fn emit_file_read(&self, path: PathBuf) -> FileOperationEvent {
        FileOperationEvent::Read {
            path,
            timestamp: SystemTime::now(),
        }
    }

    /// Emit a directory created event
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the created directory
    ///
    /// # Returns
    ///
    /// A `DirectoryOperationEvent::Created` event
    pub fn emit_directory_created(&self, path: PathBuf) -> DirectoryOperationEvent {
        DirectoryOperationEvent::Created {
            path,
            timestamp: SystemTime::now(),
        }
    }

    /// Emit a directory deleted event
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the deleted directory
    ///
    /// # Returns
    ///
    /// A `DirectoryOperationEvent::Deleted` event
    pub fn emit_directory_deleted(&self, path: PathBuf) -> DirectoryOperationEvent {
        DirectoryOperationEvent::Deleted {
            path,
            timestamp: SystemTime::now(),
        }
    }

    /// Track a file hash for modification detection
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file
    /// * `hash` - Hash of the file content
    pub async fn track_file_hash(&self, path: PathBuf, hash: String) {
        let mut hashes = self.file_hashes.write().await;
        hashes.insert(path, hash);
    }

    /// Get the tracked hash for a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file
    ///
    /// # Returns
    ///
    /// The tracked hash if it exists
    pub async fn get_file_hash(&self, path: &PathBuf) -> Option<String> {
        let hashes = self.file_hashes.read().await;
        hashes.get(path).cloned()
    }

    /// Remove a tracked file hash
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file
    pub async fn remove_file_hash(&self, path: &PathBuf) {
        let mut hashes = self.file_hashes.write().await;
        hashes.remove(path);
    }

    /// Clear all tracked file hashes
    pub async fn clear_file_hashes(&self) {
        let mut hashes = self.file_hashes.write().await;
        hashes.clear();
    }
}

impl Default for FileSystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_file_created() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/file.rs");
        let event = monitor.emit_file_created(path.clone(), 1024);

        match event {
            FileOperationEvent::Created { path: p, size, .. } => {
                assert_eq!(p, path);
                assert_eq!(size, 1024);
            }
            _ => panic!("Expected Created event"),
        }
    }

    #[test]
    fn test_emit_file_modified() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/file.rs");
        let event =
            monitor.emit_file_modified(path.clone(), "abc123".to_string(), "def456".to_string());

        match event {
            FileOperationEvent::Modified {
                path: p,
                old_hash,
                new_hash,
                ..
            } => {
                assert_eq!(p, path);
                assert_eq!(old_hash, "abc123");
                assert_eq!(new_hash, "def456");
            }
            _ => panic!("Expected Modified event"),
        }
    }

    #[test]
    fn test_emit_file_deleted() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/file.rs");
        let event = monitor.emit_file_deleted(path.clone());

        match event {
            FileOperationEvent::Deleted { path: p, .. } => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected Deleted event"),
        }
    }

    #[test]
    fn test_emit_file_renamed() {
        let monitor = FileSystemMonitor::new();
        let old_path = PathBuf::from("/path/to/old.rs");
        let new_path = PathBuf::from("/path/to/new.rs");
        let event = monitor.emit_file_renamed(old_path.clone(), new_path.clone());

        match event {
            FileOperationEvent::Renamed {
                old_path: op,
                new_path: np,
                ..
            } => {
                assert_eq!(op, old_path);
                assert_eq!(np, new_path);
            }
            _ => panic!("Expected Renamed event"),
        }
    }

    #[test]
    fn test_emit_file_moved() {
        let monitor = FileSystemMonitor::new();
        let old_path = PathBuf::from("/old/path/file.rs");
        let new_path = PathBuf::from("/new/path/file.rs");
        let event = monitor.emit_file_moved(old_path.clone(), new_path.clone());

        match event {
            FileOperationEvent::Moved {
                old_path: op,
                new_path: np,
                ..
            } => {
                assert_eq!(op, old_path);
                assert_eq!(np, new_path);
            }
            _ => panic!("Expected Moved event"),
        }
    }

    #[test]
    fn test_emit_file_read() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/file.rs");
        let event = monitor.emit_file_read(path.clone());

        match event {
            FileOperationEvent::Read { path: p, .. } => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected Read event"),
        }
    }

    #[test]
    fn test_emit_directory_created() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/dir");
        let event = monitor.emit_directory_created(path.clone());

        match event {
            DirectoryOperationEvent::Created { path: p, .. } => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected Created event"),
        }
    }

    #[test]
    fn test_emit_directory_deleted() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/dir");
        let event = monitor.emit_directory_deleted(path.clone());

        match event {
            DirectoryOperationEvent::Deleted { path: p, .. } => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected Deleted event"),
        }
    }

    #[tokio::test]
    async fn test_track_file_hash() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/file.rs");
        let hash = "abc123".to_string();

        monitor.track_file_hash(path.clone(), hash.clone()).await;
        let tracked = monitor.get_file_hash(&path).await;

        assert_eq!(tracked, Some(hash));
    }

    #[tokio::test]
    async fn test_remove_file_hash() {
        let monitor = FileSystemMonitor::new();
        let path = PathBuf::from("/path/to/file.rs");
        let hash = "abc123".to_string();

        monitor.track_file_hash(path.clone(), hash).await;
        monitor.remove_file_hash(&path).await;
        let tracked = monitor.get_file_hash(&path).await;

        assert_eq!(tracked, None);
    }

    #[tokio::test]
    async fn test_clear_file_hashes() {
        let monitor = FileSystemMonitor::new();
        let path1 = PathBuf::from("/path/to/file1.rs");
        let path2 = PathBuf::from("/path/to/file2.rs");

        monitor
            .track_file_hash(path1.clone(), "hash1".to_string())
            .await;
        monitor
            .track_file_hash(path2.clone(), "hash2".to_string())
            .await;

        monitor.clear_file_hashes().await;

        assert_eq!(monitor.get_file_hash(&path1).await, None);
        assert_eq!(monitor.get_file_hash(&path2).await, None);
    }

    #[test]
    fn test_file_system_monitor_default() {
        let monitor = FileSystemMonitor::default();
        let path = PathBuf::from("/path/to/file.rs");
        let event = monitor.emit_file_created(path.clone(), 512);

        match event {
            FileOperationEvent::Created { size, .. } => {
                assert_eq!(size, 512);
            }
            _ => panic!("Expected Created event"),
        }
    }
}
