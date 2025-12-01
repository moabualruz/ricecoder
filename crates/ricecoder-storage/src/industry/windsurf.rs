//! Windsurf IDE configuration adapter
//!
//! Reads and converts Windsurf IDE configuration files (.windsurfrules)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// Windsurf IDE adapter
pub struct WindsurfAdapter;

impl WindsurfAdapter {
    /// Create a new Windsurf adapter
    pub fn new() -> Self {
        WindsurfAdapter
    }

    /// Read .windsurfrules file
    fn read_windsurfrules(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let windsurfrules_path = project_root.join(".windsurfrules");

        if !windsurfrules_path.exists() {
            debug!("No .windsurfrules file found at {:?}", windsurfrules_path);
            return Ok(None);
        }

        debug!("Reading .windsurfrules from {:?}", windsurfrules_path);
        let content = std::fs::read_to_string(&windsurfrules_path).map_err(|e| {
            crate::error::StorageError::io_error(
                windsurfrules_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        Ok(Some(content))
    }
}

impl Default for WindsurfAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for WindsurfAdapter {
    fn name(&self) -> &'static str {
        "windsurf"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root.join(".windsurfrules").exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(rules_content)) = self.read_windsurfrules(project_root) {
            debug!("Adding Windsurf rules as steering rule");
            config.steering.push(SteeringRule {
                name: "windsurf-rules".to_string(),
                content: rules_content,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Windsurf has medium priority
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_windsurf_adapter_detects_windsurfrules() {
        let temp_dir = TempDir::new().unwrap();
        let windsurfrules_path = temp_dir.path().join(".windsurfrules");
        fs::write(&windsurfrules_path, "# Windsurf rules").unwrap();

        let adapter = WindsurfAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_windsurf_adapter_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = WindsurfAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_windsurf_adapter_reads_windsurfrules() {
        let temp_dir = TempDir::new().unwrap();
        let windsurfrules_path = temp_dir.path().join(".windsurfrules");
        let rules = "# Windsurf Rules\nBe productive";
        fs::write(&windsurfrules_path, rules).unwrap();

        let adapter = WindsurfAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "windsurf-rules");
        assert_eq!(config.steering[0].content, rules);
        assert_eq!(config.steering[0].format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_windsurf_adapter_priority() {
        let adapter = WindsurfAdapter::new();
        assert_eq!(adapter.priority(), 50);
    }

    #[test]
    fn test_windsurf_adapter_name() {
        let adapter = WindsurfAdapter::new();
        assert_eq!(adapter.name(), "windsurf");
    }
}
