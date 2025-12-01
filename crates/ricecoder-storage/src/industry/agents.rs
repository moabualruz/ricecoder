//! Generic agent configuration adapter
//!
//! Reads and converts generic agent configuration files (AGENTS.md)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// Generic agents adapter
pub struct AgentsAdapter;

impl AgentsAdapter {
    /// Create a new agents adapter
    pub fn new() -> Self {
        AgentsAdapter
    }

    /// Read AGENTS.md file
    fn read_agents_md(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let agents_path = project_root.join("AGENTS.md");

        if !agents_path.exists() {
            debug!("No AGENTS.md file found at {:?}", agents_path);
            return Ok(None);
        }

        debug!("Reading AGENTS.md from {:?}", agents_path);
        let content = std::fs::read_to_string(&agents_path).map_err(|e| {
            crate::error::StorageError::io_error(
                agents_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        Ok(Some(content))
    }
}

impl Default for AgentsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for AgentsAdapter {
    fn name(&self) -> &'static str {
        "agents"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root.join("AGENTS.md").exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(agents_content)) = self.read_agents_md(project_root) {
            debug!("Adding agent instructions as steering rule");
            config.steering.push(SteeringRule {
                name: "agent-instructions".to_string(),
                content: agents_content,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Generic agents have lower priority than specific tools
        40
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_agents_adapter_detects_agents_md() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Agent Instructions").unwrap();

        let adapter = AgentsAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_agents_adapter_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = AgentsAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_agents_adapter_reads_agents_md() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        let instructions = "# Agent Instructions\nGeneric agent guidelines";
        fs::write(&agents_path, instructions).unwrap();

        let adapter = AgentsAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "agent-instructions");
        assert_eq!(config.steering[0].content, instructions);
        assert_eq!(config.steering[0].format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_agents_adapter_priority() {
        let adapter = AgentsAdapter::new();
        assert_eq!(adapter.priority(), 40);
    }

    #[test]
    fn test_agents_adapter_name() {
        let adapter = AgentsAdapter::new();
        assert_eq!(adapter.name(), "agents");
    }
}
