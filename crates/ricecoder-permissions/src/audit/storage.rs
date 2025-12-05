//! Audit log persistence to JSON files

use super::models::AuditLogEntry;
use std::fs;
use std::path::{Path, PathBuf};

/// Audit log storage for persisting logs to disk
pub struct AuditStorage {
    path: PathBuf,
}

impl AuditStorage {
    /// Create a new audit storage with the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Save audit logs to a JSON file
    pub fn save_logs(&self, entries: &[AuditLogEntry]) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Serialize entries to JSON
        let json = serde_json::to_string_pretty(entries)
            .map_err(|e| format!("Failed to serialize logs: {}", e))?;

        // Write to file
        fs::write(&self.path, json).map_err(|e| format!("Failed to write logs to file: {}", e))?;

        Ok(())
    }

    /// Load audit logs from a JSON file
    pub fn load_logs(&self) -> Result<Vec<AuditLogEntry>, String> {
        // Check if file exists
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        // Read file
        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read logs from file: {}", e))?;

        // Deserialize from JSON
        let entries: Vec<AuditLogEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize logs: {}", e))?;

        Ok(entries)
    }

    /// Append audit logs to the existing file
    pub fn append_logs(&self, new_entries: &[AuditLogEntry]) -> Result<(), String> {
        // Load existing logs
        let mut entries = self.load_logs()?;

        // Append new entries
        entries.extend_from_slice(new_entries);

        // Save all logs
        self.save_logs(&entries)?;

        Ok(())
    }

    /// Clear all logs from the file
    pub fn clear_logs(&self) -> Result<(), String> {
        self.save_logs(&[])?;
        Ok(())
    }

    /// Get the path to the storage file
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::models::{AuditAction, AuditResult};
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_logs() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.json");
        let storage = AuditStorage::new(&log_path);

        // Create test entries
        let entries = vec![
            AuditLogEntry::new(
                "tool1".to_string(),
                AuditAction::Allowed,
                AuditResult::Success,
            ),
            AuditLogEntry::new(
                "tool2".to_string(),
                AuditAction::Denied,
                AuditResult::Blocked,
            ),
        ];

        // Save logs
        let result = storage.save_logs(&entries);
        assert!(result.is_ok());

        // Load logs
        let loaded = storage.load_logs().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].tool, "tool1");
        assert_eq!(loaded[1].tool, "tool2");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("nonexistent.json");
        let storage = AuditStorage::new(&log_path);

        // Load from nonexistent file should return empty vec
        let loaded = storage.load_logs().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_append_logs() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.json");
        let storage = AuditStorage::new(&log_path);

        // Save initial logs
        let initial_entries = vec![AuditLogEntry::new(
            "tool1".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        )];
        storage.save_logs(&initial_entries).unwrap();

        // Append new logs
        let new_entries = vec![AuditLogEntry::new(
            "tool2".to_string(),
            AuditAction::Denied,
            AuditResult::Blocked,
        )];
        storage.append_logs(&new_entries).unwrap();

        // Load and verify
        let loaded = storage.load_logs().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].tool, "tool1");
        assert_eq!(loaded[1].tool, "tool2");
    }

    #[test]
    fn test_clear_logs() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.json");
        let storage = AuditStorage::new(&log_path);

        // Save logs
        let entries = vec![AuditLogEntry::new(
            "tool1".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        )];
        storage.save_logs(&entries).unwrap();

        // Verify logs exist
        let loaded = storage.load_logs().unwrap();
        assert_eq!(loaded.len(), 1);

        // Clear logs
        storage.clear_logs().unwrap();

        // Verify logs are cleared
        let loaded = storage.load_logs().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_storage_path() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.json");
        let storage = AuditStorage::new(&log_path);

        assert_eq!(storage.path(), log_path.as_path());
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("subdir").join("audit.json");
        let storage = AuditStorage::new(&log_path);

        let entries = vec![AuditLogEntry::new(
            "tool1".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        )];

        // Save should create the directory
        let result = storage.save_logs(&entries);
        assert!(result.is_ok());
        assert!(log_path.exists());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.json");
        let storage = AuditStorage::new(&log_path);

        // Create entry with all fields
        let mut entry = AuditLogEntry::new(
            "test_tool".to_string(),
            AuditAction::Prompted,
            AuditResult::Success,
        );
        entry.agent = Some("agent1".to_string());
        entry.context = Some("User approved".to_string());

        let entries = vec![entry.clone()];

        // Save and load
        storage.save_logs(&entries).unwrap();
        let loaded = storage.load_logs().unwrap();

        // Verify all fields are preserved
        assert_eq!(loaded[0].tool, entry.tool);
        assert_eq!(loaded[0].action, entry.action);
        assert_eq!(loaded[0].result, entry.result);
        assert_eq!(loaded[0].agent, entry.agent);
        assert_eq!(loaded[0].context, entry.context);
    }
}
