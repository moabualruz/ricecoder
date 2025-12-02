//! Audit logging for comprehensive audit trails

use crate::error::FileError;
use crate::models::AuditEntry;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

/// Manages audit trails for all file operations
///
/// Logs all file operations persistently in JSON format and provides
/// access to change history with timestamps.
#[derive(Debug)]
pub struct AuditLogger {
    /// Base directory for storing audit logs
    audit_dir: PathBuf,
}

impl AuditLogger {
    /// Creates a new AuditLogger instance
    ///
    /// # Arguments
    ///
    /// * `audit_dir` - Directory where audit entries will be stored
    ///
    /// # Returns
    ///
    /// A new AuditLogger instance
    pub fn new(audit_dir: PathBuf) -> Self {
        AuditLogger { audit_dir }
    }

    /// Creates a new AuditLogger with default audit directory
    ///
    /// Uses `.ricecoder/audit/` as the default audit directory
    pub fn with_default_dir() -> Self {
        let audit_dir = PathBuf::from(".ricecoder/audit");
        AuditLogger { audit_dir }
    }

    /// Logs a file operation to the audit trail
    ///
    /// Stores the audit entry persistently in JSON format.
    /// Creates the audit directory if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `entry` - The audit entry to log
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn log_operation(&self, entry: AuditEntry) -> Result<(), FileError> {
        // Ensure audit directory exists
        fs::create_dir_all(&self.audit_dir).map_err(|e| {
            error!("Failed to create audit directory: {}", e);
            FileError::IoError(e)
        })?;

        // Generate filename based on timestamp and path
        let filename = self.generate_audit_filename(&entry);
        let filepath = self.audit_dir.join(&filename);

        // Serialize entry to JSON
        let json = serde_json::to_string_pretty(&entry).map_err(|e| {
            error!("Failed to serialize audit entry: {}", e);
            FileError::InvalidContent(format!("Failed to serialize audit entry: {}", e))
        })?;

        // Write to file
        fs::write(&filepath, json).map_err(|e| {
            error!("Failed to write audit entry to {}: {}", filepath.display(), e);
            FileError::IoError(e)
        })?;

        debug!(
            "Logged audit entry for {:?} at {}",
            entry.path,
            filepath.display()
        );
        Ok(())
    }

    /// Retrieves the change history for a specific file
    ///
    /// Returns all audit entries for the given path, ordered by timestamp.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to get history for
    ///
    /// # Returns
    ///
    /// A vector of audit entries ordered by timestamp (oldest first)
    pub fn get_change_history(&self, path: &Path) -> Result<Vec<AuditEntry>, FileError> {
        // Check if audit directory exists
        if !self.audit_dir.exists() {
            debug!("Audit directory does not exist, returning empty history");
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();

        // Read all files in audit directory
        let entries_iter = fs::read_dir(&self.audit_dir).map_err(|e| {
            error!("Failed to read audit directory: {}", e);
            FileError::IoError(e)
        })?;

        for entry_result in entries_iter {
            let entry = entry_result.map_err(|e| {
                error!("Failed to read audit entry: {}", e);
                FileError::IoError(e)
            })?;

            let file_path = entry.path();

            // Skip directories
            if file_path.is_dir() {
                continue;
            }

            // Read and parse JSON file
            let content = fs::read_to_string(&file_path).map_err(|e| {
                error!("Failed to read audit file {}: {}", file_path.display(), e);
                FileError::IoError(e)
            })?;

            match serde_json::from_str::<AuditEntry>(&content) {
                Ok(audit_entry) => {
                    // Only include entries for the requested path
                    if audit_entry.path == path {
                        entries.push(audit_entry);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to parse audit entry from {}: {}",
                        file_path.display(),
                        e
                    );
                    // Continue processing other files
                }
            }
        }

        // Sort by timestamp (oldest first)
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        info!(
            "Retrieved {} audit entries for {:?}",
            entries.len(),
            path
        );
        Ok(entries)
    }

    /// Retrieves all audit entries
    ///
    /// Returns all audit entries in the audit trail, ordered by timestamp.
    ///
    /// # Returns
    ///
    /// A vector of all audit entries ordered by timestamp (oldest first)
    pub fn get_all_entries(&self) -> Result<Vec<AuditEntry>, FileError> {
        // Check if audit directory exists
        if !self.audit_dir.exists() {
            debug!("Audit directory does not exist, returning empty entries");
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();

        // Read all files in audit directory
        let entries_iter = fs::read_dir(&self.audit_dir).map_err(|e| {
            error!("Failed to read audit directory: {}", e);
            FileError::IoError(e)
        })?;

        for entry_result in entries_iter {
            let entry = entry_result.map_err(|e| {
                error!("Failed to read audit entry: {}", e);
                FileError::IoError(e)
            })?;

            let file_path = entry.path();

            // Skip directories
            if file_path.is_dir() {
                continue;
            }

            // Read and parse JSON file
            let content = fs::read_to_string(&file_path).map_err(|e| {
                error!("Failed to read audit file {}: {}", file_path.display(), e);
                FileError::IoError(e)
            })?;

            match serde_json::from_str::<AuditEntry>(&content) {
                Ok(audit_entry) => {
                    entries.push(audit_entry);
                }
                Err(e) => {
                    error!(
                        "Failed to parse audit entry from {}: {}",
                        file_path.display(),
                        e
                    );
                    // Continue processing other files
                }
            }
        }

        // Sort by timestamp (oldest first)
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        info!("Retrieved {} total audit entries", entries.len());
        Ok(entries)
    }

    /// Generates a unique filename for an audit entry
    ///
    /// Uses timestamp and path hash to create a unique filename
    fn generate_audit_filename(&self, entry: &AuditEntry) -> String {
        // Use timestamp and a hash of the path to create unique filename
        let timestamp = entry.timestamp.format("%Y%m%d_%H%M%S_%3f");
        let path_hash = format!("{:x}", fxhash::hash64(&entry.path.to_string_lossy()));
        format!("audit_{}_{}.json", timestamp, path_hash)
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::with_default_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OperationType;
    use chrono::Utc;
    use tempfile::TempDir;

    #[test]
    fn test_audit_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());
        assert_eq!(logger.audit_dir, temp_dir.path());
    }

    #[test]
    fn test_log_operation_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let audit_dir = temp_dir.path().join("audit");
        let logger = AuditLogger::new(audit_dir.clone());

        let entry = AuditEntry {
            timestamp: Utc::now(),
            path: PathBuf::from("test.txt"),
            operation_type: OperationType::Create,
            content_hash: "abc123".to_string(),
            transaction_id: None,
        };

        logger.log_operation(entry).unwrap();
        assert!(audit_dir.exists());
    }

    #[test]
    fn test_log_operation_writes_json() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let entry = AuditEntry {
            timestamp: Utc::now(),
            path: PathBuf::from("test.txt"),
            operation_type: OperationType::Create,
            content_hash: "abc123".to_string(),
            transaction_id: None,
        };

        logger.log_operation(entry.clone()).unwrap();

        // Verify file was created
        let files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!files.is_empty());
    }

    #[test]
    fn test_get_change_history_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let history = logger.get_change_history(Path::new("test.txt")).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_get_change_history_filters_by_path() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let entry1 = AuditEntry {
            timestamp: Utc::now(),
            path: PathBuf::from("file1.txt"),
            operation_type: OperationType::Create,
            content_hash: "hash1".to_string(),
            transaction_id: None,
        };

        let entry2 = AuditEntry {
            timestamp: Utc::now(),
            path: PathBuf::from("file2.txt"),
            operation_type: OperationType::Update,
            content_hash: "hash2".to_string(),
            transaction_id: None,
        };

        logger.log_operation(entry1).unwrap();
        logger.log_operation(entry2).unwrap();

        let history = logger.get_change_history(Path::new("file1.txt")).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].path, PathBuf::from("file1.txt"));
    }

    #[test]
    fn test_get_change_history_ordered_by_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let entry1 = AuditEntry {
            timestamp: now,
            path: PathBuf::from("test.txt"),
            operation_type: OperationType::Create,
            content_hash: "hash1".to_string(),
            transaction_id: None,
        };

        let entry2 = AuditEntry {
            timestamp: now + chrono::Duration::seconds(1),
            path: PathBuf::from("test.txt"),
            operation_type: OperationType::Update,
            content_hash: "hash2".to_string(),
            transaction_id: None,
        };

        logger.log_operation(entry1).unwrap();
        logger.log_operation(entry2).unwrap();

        let history = logger.get_change_history(Path::new("test.txt")).unwrap();
        assert_eq!(history.len(), 2);
        assert!(history[0].timestamp <= history[1].timestamp);
    }

    #[test]
    fn test_get_all_entries() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let entry1 = AuditEntry {
            timestamp: Utc::now(),
            path: PathBuf::from("file1.txt"),
            operation_type: OperationType::Create,
            content_hash: "hash1".to_string(),
            transaction_id: None,
        };

        let entry2 = AuditEntry {
            timestamp: Utc::now(),
            path: PathBuf::from("file2.txt"),
            operation_type: OperationType::Update,
            content_hash: "hash2".to_string(),
            transaction_id: None,
        };

        logger.log_operation(entry1).unwrap();
        logger.log_operation(entry2).unwrap();

        let all_entries = logger.get_all_entries().unwrap();
        assert_eq!(all_entries.len(), 2);
    }
}
