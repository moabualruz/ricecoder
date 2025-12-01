//! Aider configuration adapter
//!
//! Reads and converts Aider configuration files (.aider.conf.yml)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// Aider adapter
pub struct AiderAdapter;

impl AiderAdapter {
    /// Create a new Aider adapter
    pub fn new() -> Self {
        AiderAdapter
    }

    /// Read .aider.conf.yml file
    fn read_aider_config(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let aider_config_path = project_root.join(".aider.conf.yml");

        if !aider_config_path.exists() {
            debug!("No .aider.conf.yml file found at {:?}", aider_config_path);
            return Ok(None);
        }

        debug!("Reading .aider.conf.yml from {:?}", aider_config_path);
        let content = std::fs::read_to_string(&aider_config_path).map_err(|e| {
            crate::error::StorageError::io_error(
                aider_config_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        Ok(Some(content))
    }
}

impl Default for AiderAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for AiderAdapter {
    fn name(&self) -> &'static str {
        "aider"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root.join(".aider.conf.yml").exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(aider_config)) = self.read_aider_config(project_root) {
            debug!("Adding Aider configuration as steering rule");
            config.steering.push(SteeringRule {
                name: "aider-config".to_string(),
                content: aider_config,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Aider has medium priority
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_aider_adapter_detects_config() {
        let temp_dir = TempDir::new().unwrap();
        let aider_config_path = temp_dir.path().join(".aider.conf.yml");
        fs::write(&aider_config_path, "model: gpt-4").unwrap();

        let adapter = AiderAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_aider_adapter_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = AiderAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_aider_adapter_reads_config() {
        let temp_dir = TempDir::new().unwrap();
        let aider_config_path = temp_dir.path().join(".aider.conf.yml");
        let config_content = "model: gpt-4\ntemperature: 0.7";
        fs::write(&aider_config_path, config_content).unwrap();

        let adapter = AiderAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "aider-config");
        assert_eq!(config.steering[0].content, config_content);
        assert_eq!(config.steering[0].format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_aider_adapter_priority() {
        let adapter = AiderAdapter::new();
        assert_eq!(adapter.priority(), 50);
    }

    #[test]
    fn test_aider_adapter_name() {
        let adapter = AiderAdapter::new();
        assert_eq!(adapter.name(), "aider");
    }
}
