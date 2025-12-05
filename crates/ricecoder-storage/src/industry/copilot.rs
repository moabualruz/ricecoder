//! GitHub Copilot configuration adapter
//!
//! Reads and converts GitHub Copilot configuration files (.github/copilot-instructions.md)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// GitHub Copilot adapter
pub struct CopilotAdapter;

impl CopilotAdapter {
    /// Create a new Copilot adapter
    pub fn new() -> Self {
        CopilotAdapter
    }

    /// Read .github/copilot-instructions.md file
    fn read_copilot_instructions(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let copilot_path = project_root.join(".github/copilot-instructions.md");

        if !copilot_path.exists() {
            debug!(
                "No .github/copilot-instructions.md file found at {:?}",
                copilot_path
            );
            return Ok(None);
        }

        debug!(
            "Reading .github/copilot-instructions.md from {:?}",
            copilot_path
        );
        let content = std::fs::read_to_string(&copilot_path).map_err(|e| {
            crate::error::StorageError::io_error(
                copilot_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        Ok(Some(content))
    }
}

impl Default for CopilotAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for CopilotAdapter {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root
            .join(".github/copilot-instructions.md")
            .exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(instructions)) = self.read_copilot_instructions(project_root) {
            debug!("Adding Copilot instructions as steering rule");
            config.steering.push(SteeringRule {
                name: "copilot-instructions".to_string(),
                content: instructions,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Copilot has medium priority
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_copilot_adapter_detects_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let github_dir = temp_dir.path().join(".github");
        fs::create_dir(&github_dir).unwrap();
        let copilot_path = github_dir.join("copilot-instructions.md");
        fs::write(&copilot_path, "# Copilot Instructions").unwrap();

        let adapter = CopilotAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_copilot_adapter_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = CopilotAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_copilot_adapter_reads_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let github_dir = temp_dir.path().join(".github");
        fs::create_dir(&github_dir).unwrap();
        let copilot_path = github_dir.join("copilot-instructions.md");
        let instructions = "# Copilot Instructions\nBe helpful";
        fs::write(&copilot_path, instructions).unwrap();

        let adapter = CopilotAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "copilot-instructions");
        assert_eq!(config.steering[0].content, instructions);
        assert_eq!(config.steering[0].format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_copilot_adapter_priority() {
        let adapter = CopilotAdapter::new();
        assert_eq!(adapter.priority(), 50);
    }

    #[test]
    fn test_copilot_adapter_name() {
        let adapter = CopilotAdapter::new();
        assert_eq!(adapter.name(), "copilot");
    }
}
