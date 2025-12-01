//! Cline IDE configuration adapter
//!
//! Reads and converts Cline IDE configuration files (.clinerules)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// Cline IDE adapter
pub struct ClineAdapter;

impl ClineAdapter {
    /// Create a new Cline adapter
    pub fn new() -> Self {
        ClineAdapter
    }

    /// Read .clinerules file
    fn read_clinerules(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let clinerules_path = project_root.join(".clinerules");

        if !clinerules_path.exists() {
            debug!("No .clinerules file found at {:?}", clinerules_path);
            return Ok(None);
        }

        debug!("Reading .clinerules from {:?}", clinerules_path);
        let content = std::fs::read_to_string(&clinerules_path).map_err(|e| {
            crate::error::StorageError::io_error(
                clinerules_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        Ok(Some(content))
    }
}

impl Default for ClineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for ClineAdapter {
    fn name(&self) -> &'static str {
        "cline"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root.join(".clinerules").exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(rules_content)) = self.read_clinerules(project_root) {
            debug!("Adding Cline rules as steering rule");
            config.steering.push(SteeringRule {
                name: "cline-rules".to_string(),
                content: rules_content,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Cline has medium priority
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cline_adapter_detects_clinerules() {
        let temp_dir = TempDir::new().unwrap();
        let clinerules_path = temp_dir.path().join(".clinerules");
        fs::write(&clinerules_path, "# Cline rules").unwrap();

        let adapter = ClineAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_cline_adapter_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = ClineAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_cline_adapter_reads_clinerules() {
        let temp_dir = TempDir::new().unwrap();
        let clinerules_path = temp_dir.path().join(".clinerules");
        let rules = "# Cline Rules\nBe efficient";
        fs::write(&clinerules_path, rules).unwrap();

        let adapter = ClineAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "cline-rules");
        assert_eq!(config.steering[0].content, rules);
        assert_eq!(config.steering[0].format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_cline_adapter_priority() {
        let adapter = ClineAdapter::new();
        assert_eq!(adapter.priority(), 50);
    }

    #[test]
    fn test_cline_adapter_name() {
        let adapter = ClineAdapter::new();
        assert_eq!(adapter.name(), "cline");
    }
}
