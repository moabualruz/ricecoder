//! Session-aware file read/write tracking
//!
//! Provides session-based tracking of file reads and writes to detect
//! external modifications and prevent accidental overwrites.
//! Matches OpenCode's FileTime module functionality.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tracing::{debug, warn};

use crate::error::FileError;

/// Record of a file read operation
#[derive(Debug, Clone)]
pub struct FileReadRecord {
    /// Path of the file
    pub path: PathBuf,
    /// When the file was read
    pub read_at: SystemTime,
    /// Last modification time of the file at read time
    pub mtime: SystemTime,
}

/// Session-aware file tracker
///
/// Tracks file reads and validates writes to detect external modifications
/// Implements OpenCode's "read before write" + mtime check pattern
pub struct SessionFileTracker {
    /// Map of (session_id, file_path) -> read record
    records: Arc<RwLock<HashMap<(String, PathBuf), FileReadRecord>>>,
}

impl SessionFileTracker {
    /// Create a new session file tracker
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a file read operation
    pub fn record_read(
        &self,
        session_id: &str,
        path: &PathBuf,
        mtime: SystemTime,
    ) -> Result<(), FileError> {
        let record = FileReadRecord {
            path: path.clone(),
            read_at: SystemTime::now(),
            mtime,
        };

        let key = (session_id.to_string(), path.clone());
        let mut records = self.records.write().map_err(|_| {
            FileError::LockError("Failed to acquire write lock for file records".to_string())
        })?;

        records.insert(key, record);
        debug!("Recorded file read: {} in session {}", path.display(), session_id);
        Ok(())
    }

    /// Assert that a file was read before writing and hasn't been modified externally
    ///
    /// Matches OpenCode's FileTime.assert() behavior
    pub fn assert_can_write(
        &self,
        session_id: &str,
        path: &PathBuf,
        current_mtime: SystemTime,
    ) -> Result<(), FileError> {
        let key = (session_id.to_string(), path.clone());
        let records = self.records.read().map_err(|_| {
            FileError::LockError("Failed to acquire read lock for file records".to_string())
        })?;

        // Check if file was read in this session
        let record = records.get(&key).ok_or_else(|| {
            FileError::WritePreconditionFailed(format!(
                "File {} must be read before writing in session {}",
                path.display(),
                session_id
            ))
        })?;

        // Check if file was modified since read
        if current_mtime > record.mtime {
            return Err(FileError::ExternalModification {
                path: path.clone(),
                read_at: record.read_at,
                modified_at: current_mtime,
            });
        }

        Ok(())
    }

    /// Clear all records for a session
    pub fn clear_session(&self, session_id: &str) -> Result<(), FileError> {
        let mut records = self.records.write().map_err(|_| {
            FileError::LockError("Failed to acquire write lock for file records".to_string())
        })?;

        records.retain(|(sid, _), _| sid != session_id);
        debug!("Cleared file records for session {}", session_id);
        Ok(())
    }

    /// Get the number of tracked files for a session
    pub fn session_file_count(&self, session_id: &str) -> Result<usize, FileError> {
        let records = self.records.read().map_err(|_| {
            FileError::LockError("Failed to acquire read lock for file records".to_string())
        })?;

        let count = records.keys().filter(|(sid, _)| sid == session_id).count();
        Ok(count)
    }
}

impl Default for SessionFileTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_session_tracker_record_read() {
        let tracker = SessionFileTracker::new();
        let path = PathBuf::from("/test/file.txt");
        let mtime = SystemTime::now();

        tracker.record_read("session1", &path, mtime).unwrap();
        assert_eq!(tracker.session_file_count("session1").unwrap(), 1);
    }

    #[test]
    fn test_assert_can_write_success() {
        let tracker = SessionFileTracker::new();
        let path = PathBuf::from("/test/file.txt");
        let mtime = SystemTime::now();

        tracker.record_read("session1", &path, mtime).unwrap();

        // Same mtime - should succeed
        assert!(tracker.assert_can_write("session1", &path, mtime).is_ok());
    }

    #[test]
    fn test_assert_can_write_external_modification() {
        let tracker = SessionFileTracker::new();
        let path = PathBuf::from("/test/file.txt");
        let mtime = SystemTime::now();

        tracker.record_read("session1", &path, mtime).unwrap();

        // Later mtime - should fail
        let new_mtime = mtime + Duration::from_secs(10);
        let result = tracker.assert_can_write("session1", &path, new_mtime);
        assert!(result.is_err());
        assert!(matches!(result, Err(FileError::ExternalModification { .. })));
    }

    #[test]
    fn test_assert_can_write_not_read() {
        let tracker = SessionFileTracker::new();
        let path = PathBuf::from("/test/file.txt");
        let mtime = SystemTime::now();

        // File not read - should fail
        let result = tracker.assert_can_write("session1", &path, mtime);
        assert!(result.is_err());
        assert!(matches!(result, Err(FileError::WritePreconditionFailed(_))));
    }

    #[test]
    fn test_clear_session() {
        let tracker = SessionFileTracker::new();
        let path = PathBuf::from("/test/file.txt");
        let mtime = SystemTime::now();

        tracker.record_read("session1", &path, mtime).unwrap();
        assert_eq!(tracker.session_file_count("session1").unwrap(), 1);

        tracker.clear_session("session1").unwrap();
        assert_eq!(tracker.session_file_count("session1").unwrap(), 0);
    }
}
