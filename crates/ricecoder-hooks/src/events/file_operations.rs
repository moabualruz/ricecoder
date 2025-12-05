//! File and directory operation event types
//!
//! This module defines event types for file system operations including create, modify,
//! delete, rename, move, and read operations on both files and directories.
//!
//! # Examples
//!
//! File created event:
//! ```ignore
//! use ricecoder_hooks::events::FileOperationEvent;
//! use std::path::PathBuf;
//! use std::time::SystemTime;
//!
//! let event = FileOperationEvent::Created {
//!     path: PathBuf::from("/path/to/file.rs"),
//!     size: 1024,
//!     timestamp: SystemTime::now(),
//! };
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// File operation event types
///
/// Represents different types of file system operations that can trigger hooks.
/// Each variant includes relevant metadata for the operation.
///
/// # Variants
///
/// * `Created` - File was created
/// * `Modified` - File was modified
/// * `Deleted` - File was deleted
/// * `Renamed` - File was renamed
/// * `Moved` - File was moved to a different directory
/// * `Read` - File was read
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation_type")]
pub enum FileOperationEvent {
    /// File was created
    ///
    /// # Fields
    ///
    /// * `path` - Path to the created file
    /// * `size` - Size of the file in bytes
    /// * `timestamp` - Time when the file was created
    #[serde(rename = "file_created")]
    Created {
        /// Path to the file
        path: PathBuf,

        /// File size in bytes
        size: u64,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },

    /// File was modified
    ///
    /// # Fields
    ///
    /// * `path` - Path to the modified file
    /// * `old_hash` - Hash of the file before modification
    /// * `new_hash` - Hash of the file after modification
    /// * `timestamp` - Time when the file was modified
    #[serde(rename = "file_modified")]
    Modified {
        /// Path to the file
        path: PathBuf,

        /// Hash of the file before modification
        old_hash: String,

        /// Hash of the file after modification
        new_hash: String,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },

    /// File was deleted
    ///
    /// # Fields
    ///
    /// * `path` - Path to the deleted file
    /// * `timestamp` - Time when the file was deleted
    #[serde(rename = "file_deleted")]
    Deleted {
        /// Path to the file
        path: PathBuf,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },

    /// File was renamed
    ///
    /// # Fields
    ///
    /// * `old_path` - Original path of the file
    /// * `new_path` - New path of the file
    /// * `timestamp` - Time when the file was renamed
    #[serde(rename = "file_renamed")]
    Renamed {
        /// Original path
        old_path: PathBuf,

        /// New path
        new_path: PathBuf,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },

    /// File was moved to a different directory
    ///
    /// # Fields
    ///
    /// * `old_path` - Original path of the file
    /// * `new_path` - New path of the file
    /// * `timestamp` - Time when the file was moved
    #[serde(rename = "file_moved")]
    Moved {
        /// Original path
        old_path: PathBuf,

        /// New path
        new_path: PathBuf,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },

    /// File was read
    ///
    /// # Fields
    ///
    /// * `path` - Path to the file that was read
    /// * `timestamp` - Time when the file was read
    #[serde(rename = "file_read")]
    Read {
        /// Path to the file
        path: PathBuf,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },
}

/// Directory operation event types
///
/// Represents different types of directory operations that can trigger hooks.
/// Each variant includes relevant metadata for the operation.
///
/// # Variants
///
/// * `Created` - Directory was created
/// * `Deleted` - Directory was deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation_type")]
pub enum DirectoryOperationEvent {
    /// Directory was created
    ///
    /// # Fields
    ///
    /// * `path` - Path to the created directory
    /// * `timestamp` - Time when the directory was created
    #[serde(rename = "directory_created")]
    Created {
        /// Path to the directory
        path: PathBuf,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },

    /// Directory was deleted
    ///
    /// # Fields
    ///
    /// * `path` - Path to the deleted directory
    /// * `timestamp` - Time when the directory was deleted
    #[serde(rename = "directory_deleted")]
    Deleted {
        /// Path to the directory
        path: PathBuf,

        /// Timestamp of the operation
        timestamp: SystemTime,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_created_event_serialization() {
        let event = FileOperationEvent::Created {
            path: PathBuf::from("/path/to/file.rs"),
            size: 1024,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("file_created"));
        assert!(json.contains("file.rs"));
        assert!(json.contains("1024"));
    }

    #[test]
    fn test_file_modified_event_serialization() {
        let event = FileOperationEvent::Modified {
            path: PathBuf::from("/path/to/file.rs"),
            old_hash: "abc123".to_string(),
            new_hash: "def456".to_string(),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("file_modified"));
        assert!(json.contains("abc123"));
        assert!(json.contains("def456"));
    }

    #[test]
    fn test_file_deleted_event_serialization() {
        let event = FileOperationEvent::Deleted {
            path: PathBuf::from("/path/to/file.rs"),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("file_deleted"));
        assert!(json.contains("file.rs"));
    }

    #[test]
    fn test_file_renamed_event_serialization() {
        let event = FileOperationEvent::Renamed {
            old_path: PathBuf::from("/path/to/old.rs"),
            new_path: PathBuf::from("/path/to/new.rs"),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("file_renamed"));
        assert!(json.contains("old.rs"));
        assert!(json.contains("new.rs"));
    }

    #[test]
    fn test_file_moved_event_serialization() {
        let event = FileOperationEvent::Moved {
            old_path: PathBuf::from("/old/path/file.rs"),
            new_path: PathBuf::from("/new/path/file.rs"),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("file_moved"));
        assert!(json.contains("old/path"));
        assert!(json.contains("new/path"));
    }

    #[test]
    fn test_file_read_event_serialization() {
        let event = FileOperationEvent::Read {
            path: PathBuf::from("/path/to/file.rs"),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("file_read"));
        assert!(json.contains("file.rs"));
    }

    #[test]
    fn test_directory_created_event_serialization() {
        let event = DirectoryOperationEvent::Created {
            path: PathBuf::from("/path/to/dir"),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("directory_created"));
        assert!(json.contains("dir"));
    }

    #[test]
    fn test_directory_deleted_event_serialization() {
        let event = DirectoryOperationEvent::Deleted {
            path: PathBuf::from("/path/to/dir"),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("directory_deleted"));
        assert!(json.contains("dir"));
    }

    #[test]
    fn test_file_operation_event_clone() {
        let event = FileOperationEvent::Created {
            path: PathBuf::from("/path/to/file.rs"),
            size: 1024,
            timestamp: SystemTime::now(),
        };

        let cloned = event.clone();
        match cloned {
            FileOperationEvent::Created { size, .. } => {
                assert_eq!(size, 1024);
            }
            _ => panic!("Expected Created variant"),
        }
    }

    #[test]
    fn test_directory_operation_event_clone() {
        let event = DirectoryOperationEvent::Created {
            path: PathBuf::from("/path/to/dir"),
            timestamp: SystemTime::now(),
        };

        let cloned = event.clone();
        match cloned {
            DirectoryOperationEvent::Created { .. } => {
                // Success
            }
            _ => panic!("Expected Created variant"),
        }
    }
}
