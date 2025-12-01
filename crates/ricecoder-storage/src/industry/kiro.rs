//! Kiro configuration adapter
//!
//! Reads and converts Kiro configuration files (.kiro/ directory)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// Kiro adapter
pub struct KiroAdapter;

impl KiroAdapter {
    /// Create a new Kiro adapter
    pub fn new() -> Self {
        KiroAdapter
    }

    /// Read .kiro/ directory configuration
    fn read_kiro_config(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let kiro_dir = project_root.join(".kiro");

        if !kiro_dir.exists() {
            debug!("No .kiro directory found at {:?}", kiro_dir);
            return Ok(None);
        }

        debug!("Reading .kiro configuration from {:?}", kiro_dir);

        // Try to read specs and steering files
        let mut combined_content = String::new();

        // Read specs
        let specs_dir = kiro_dir.join("specs");
        if specs_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&specs_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                combined_content.push_str(&format!(
                                    "# Spec: {}\n{}\n\n",
                                    entry.path().display(),
                                    content
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Read steering
        let steering_dir = kiro_dir.join("steering");
        if steering_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&steering_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                combined_content.push_str(&format!(
                                    "# Steering: {}\n{}\n\n",
                                    entry.path().display(),
                                    content
                                ));
                            }
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

impl Default for KiroAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for KiroAdapter {
    fn name(&self) -> &'static str {
        "kiro"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root.join(".kiro").exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(kiro_config)) = self.read_kiro_config(project_root) {
            debug!("Adding Kiro configuration as steering rule");
            config.steering.push(SteeringRule {
                name: "kiro-config".to_string(),
                content: kiro_config,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Kiro has highest priority among industry files (it's the native format)
        100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_kiro_adapter_detects_directory() {
        let temp_dir = TempDir::new().unwrap();
        let kiro_dir = temp_dir.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();

        let adapter = KiroAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_kiro_adapter_no_directory() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = KiroAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_kiro_adapter_reads_specs() {
        let temp_dir = TempDir::new().unwrap();
        let kiro_dir = temp_dir.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();
        let specs_dir = kiro_dir.join("specs");
        fs::create_dir(&specs_dir).unwrap();
        fs::write(specs_dir.join("spec1.md"), "# Spec 1").unwrap();

        let adapter = KiroAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "kiro-config");
        assert!(config.steering[0].content.contains("Spec 1"));
    }

    #[test]
    fn test_kiro_adapter_reads_steering() {
        let temp_dir = TempDir::new().unwrap();
        let kiro_dir = temp_dir.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();
        let steering_dir = kiro_dir.join("steering");
        fs::create_dir(&steering_dir).unwrap();
        fs::write(steering_dir.join("rules.md"), "# Rules").unwrap();

        let adapter = KiroAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "kiro-config");
        assert!(config.steering[0].content.contains("Rules"));
    }

    #[test]
    fn test_kiro_adapter_priority() {
        let adapter = KiroAdapter::new();
        assert_eq!(adapter.priority(), 100);
    }

    #[test]
    fn test_kiro_adapter_name() {
        let adapter = KiroAdapter::new();
        assert_eq!(adapter.name(), "kiro");
    }
}
