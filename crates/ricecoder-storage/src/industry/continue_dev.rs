//! Continue.dev configuration adapter
//!
//! Reads and converts Continue.dev configuration files (.continue/ directory)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// Continue.dev adapter
pub struct ContinueDevAdapter;

impl ContinueDevAdapter {
    /// Create a new Continue.dev adapter
    pub fn new() -> Self {
        ContinueDevAdapter
    }

    /// Read .continue/ directory configuration
    fn read_continue_config(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let continue_dir = project_root.join(".continue");

        if !continue_dir.exists() {
            debug!("No .continue directory found at {:?}", continue_dir);
            return Ok(None);
        }

        debug!("Reading .continue configuration from {:?}", continue_dir);

        // Try to read config.json or other config files
        let config_path = continue_dir.join("config.json");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).map_err(|e| {
                crate::error::StorageError::io_error(
                    config_path.clone(),
                    crate::error::IoOperation::Read,
                    e,
                )
            })?;
            return Ok(Some(content));
        }

        // If no config.json, try to read all files in the directory
        let mut combined_content = String::new();
        if let Ok(entries) = std::fs::read_dir(&continue_dir) {
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

impl Default for ContinueDevAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for ContinueDevAdapter {
    fn name(&self) -> &'static str {
        "continue"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root.join(".continue").exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(continue_config)) = self.read_continue_config(project_root) {
            debug!("Adding Continue.dev configuration as steering rule");
            config.steering.push(SteeringRule {
                name: "continue-config".to_string(),
                content: continue_config,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Continue.dev has medium priority
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_continue_adapter_detects_directory() {
        let temp_dir = TempDir::new().unwrap();
        let continue_dir = temp_dir.path().join(".continue");
        fs::create_dir(&continue_dir).unwrap();

        let adapter = ContinueDevAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_continue_adapter_no_directory() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = ContinueDevAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_continue_adapter_reads_config_json() {
        let temp_dir = TempDir::new().unwrap();
        let continue_dir = temp_dir.path().join(".continue");
        fs::create_dir(&continue_dir).unwrap();
        let config_path = continue_dir.join("config.json");
        let config_content = r#"{"models": ["gpt-4"]}"#;
        fs::write(&config_path, config_content).unwrap();

        let adapter = ContinueDevAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "continue-config");
        assert_eq!(config.steering[0].content, config_content);
    }

    #[test]
    fn test_continue_adapter_reads_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let continue_dir = temp_dir.path().join(".continue");
        fs::create_dir(&continue_dir).unwrap();
        fs::write(continue_dir.join("file1.txt"), "content1").unwrap();
        fs::write(continue_dir.join("file2.txt"), "content2").unwrap();

        let adapter = ContinueDevAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "continue-config");
        assert!(config.steering[0].content.contains("content1"));
        assert!(config.steering[0].content.contains("content2"));
    }

    #[test]
    fn test_continue_adapter_priority() {
        let adapter = ContinueDevAdapter::new();
        assert_eq!(adapter.priority(), 50);
    }

    #[test]
    fn test_continue_adapter_name() {
        let adapter = ContinueDevAdapter::new();
        assert_eq!(adapter.name(), "continue");
    }
}
