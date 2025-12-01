//! Cursor IDE configuration adapter
//!
//! Reads and converts Cursor IDE configuration files (.cursorrules and .cursor/ directory)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::{debug, warn};

use super::adapter::IndustryFileAdapter;

/// Cursor IDE adapter
pub struct CursorAdapter;

impl CursorAdapter {
    /// Create a new Cursor adapter
    pub fn new() -> Self {
        CursorAdapter
    }

    /// Read .cursorrules file
    fn read_cursorrules(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let cursorrules_path = project_root.join(".cursorrules");

        if !cursorrules_path.exists() {
            debug!("No .cursorrules file found at {:?}", cursorrules_path);
            return Ok(None);
        }

        debug!("Reading .cursorrules from {:?}", cursorrules_path);
        let content = std::fs::read_to_string(&cursorrules_path).map_err(|e| {
            crate::error::StorageError::io_error(
                cursorrules_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        Ok(Some(content))
    }

    /// Read .cursor/ directory settings
    fn read_cursor_settings(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let cursor_dir = project_root.join(".cursor");

        if !cursor_dir.exists() {
            debug!("No .cursor directory found at {:?}", cursor_dir);
            return Ok(None);
        }

        debug!("Reading .cursor settings from {:?}", cursor_dir);

        // Try to read settings.json or other config files in .cursor/
        let settings_path = cursor_dir.join("settings.json");
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path).map_err(|e| {
                crate::error::StorageError::io_error(
                    settings_path.clone(),
                    crate::error::IoOperation::Read,
                    e,
                )
            })?;
            return Ok(Some(content));
        }

        // If no settings.json, try to read all files in the directory
        let mut combined_content = String::new();
        if let Ok(entries) = std::fs::read_dir(&cursor_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            combined_content.push_str(&format!(
                                "# {}\n{}\n\n",
                                entry.path().display(),
                                content
                            ));
                        }
                    }
                }
            }
        }

        if combined_content.is_empty() {
            Ok(None)
        } else {
            Ok(Some(combined_content))
        }
    }
}

impl Default for CursorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for CursorAdapter {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        let cursorrules_exists = project_root.join(".cursorrules").exists();
        let cursor_dir_exists = project_root.join(".cursor").exists();

        cursorrules_exists || cursor_dir_exists
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        // Read .cursorrules
        if let Ok(Some(cursorrules_content)) = self.read_cursorrules(project_root) {
            debug!("Adding Cursor rules as steering rule");
            config.steering.push(SteeringRule {
                name: "cursor-rules".to_string(),
                content: cursorrules_content,
                format: DocumentFormat::Markdown,
            });
        }

        // Read .cursor/ settings
        if let Ok(Some(cursor_settings)) = self.read_cursor_settings(project_root) {
            debug!("Adding Cursor settings as steering rule");
            config.steering.push(SteeringRule {
                name: "cursor-settings".to_string(),
                content: cursor_settings,
                format: DocumentFormat::Markdown,
            });
        }

        if config.steering.is_empty() {
            warn!("Cursor adapter found files but no content was read");
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Cursor has medium priority (after project-specific, before generic)
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cursor_adapter_detects_cursorrules() {
        let temp_dir = TempDir::new().unwrap();
        let cursorrules_path = temp_dir.path().join(".cursorrules");
        fs::write(&cursorrules_path, "# Cursor rules").unwrap();

        let adapter = CursorAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_cursor_adapter_detects_cursor_dir() {
        let temp_dir = TempDir::new().unwrap();
        let cursor_dir = temp_dir.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();

        let adapter = CursorAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_cursor_adapter_no_files() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = CursorAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_cursor_adapter_reads_cursorrules() {
        let temp_dir = TempDir::new().unwrap();
        let cursorrules_path = temp_dir.path().join(".cursorrules");
        let rules_content = "# Cursor rules\nBe helpful";
        fs::write(&cursorrules_path, rules_content).unwrap();

        let adapter = CursorAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "cursor-rules");
        assert_eq!(config.steering[0].content, rules_content);
        assert_eq!(config.steering[0].format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_cursor_adapter_reads_cursor_settings() {
        let temp_dir = TempDir::new().unwrap();
        let cursor_dir = temp_dir.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();
        let settings_path = cursor_dir.join("settings.json");
        let settings_content = r#"{"key": "value"}"#;
        fs::write(&settings_path, settings_content).unwrap();

        let adapter = CursorAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "cursor-settings");
        assert_eq!(config.steering[0].content, settings_content);
    }

    #[test]
    fn test_cursor_adapter_reads_both() {
        let temp_dir = TempDir::new().unwrap();
        let cursorrules_path = temp_dir.path().join(".cursorrules");
        fs::write(&cursorrules_path, "# Rules").unwrap();

        let cursor_dir = temp_dir.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();
        let settings_path = cursor_dir.join("settings.json");
        fs::write(&settings_path, r#"{"key": "value"}"#).unwrap();

        let adapter = CursorAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 2);
        assert_eq!(config.steering[0].name, "cursor-rules");
        assert_eq!(config.steering[1].name, "cursor-settings");
    }

    #[test]
    fn test_cursor_adapter_priority() {
        let adapter = CursorAdapter::new();
        assert_eq!(adapter.priority(), 50);
    }

    #[test]
    fn test_cursor_adapter_name() {
        let adapter = CursorAdapter::new();
        assert_eq!(adapter.name(), "cursor");
    }
}
