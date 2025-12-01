//! Claude IDE configuration adapter
//!
//! Reads and converts Claude IDE configuration files (CLAUDE.md)
//! into RiceCoder's internal configuration format.

use crate::config::{Config, SteeringRule};
use crate::error::StorageResult;
use crate::types::DocumentFormat;
use std::path::Path;
use tracing::debug;

use super::adapter::IndustryFileAdapter;

/// Claude IDE adapter
pub struct ClaudeAdapter;

impl ClaudeAdapter {
    /// Create a new Claude adapter
    pub fn new() -> Self {
        ClaudeAdapter
    }

    /// Read CLAUDE.md file
    fn read_claude_md(&self, project_root: &Path) -> StorageResult<Option<String>> {
        let claude_path = project_root.join("CLAUDE.md");

        if !claude_path.exists() {
            debug!("No CLAUDE.md file found at {:?}", claude_path);
            return Ok(None);
        }

        debug!("Reading CLAUDE.md from {:?}", claude_path);
        let content = std::fs::read_to_string(&claude_path).map_err(|e| {
            crate::error::StorageError::io_error(
                claude_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        Ok(Some(content))
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IndustryFileAdapter for ClaudeAdapter {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn can_handle(&self, project_root: &Path) -> bool {
        project_root.join("CLAUDE.md").exists()
    }

    fn read_config(&self, project_root: &Path) -> StorageResult<Config> {
        let mut config = Config::default();

        if let Ok(Some(claude_content)) = self.read_claude_md(project_root) {
            debug!("Adding Claude instructions as steering rule");
            config.steering.push(SteeringRule {
                name: "claude-instructions".to_string(),
                content: claude_content,
                format: DocumentFormat::Markdown,
            });
        }

        Ok(config)
    }

    fn priority(&self) -> u32 {
        // Claude has medium priority
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_claude_adapter_detects_claude_md() {
        let temp_dir = TempDir::new().unwrap();
        let claude_path = temp_dir.path().join("CLAUDE.md");
        fs::write(&claude_path, "# Claude Instructions").unwrap();

        let adapter = ClaudeAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_claude_adapter_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = ClaudeAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_claude_adapter_reads_claude_md() {
        let temp_dir = TempDir::new().unwrap();
        let claude_path = temp_dir.path().join("CLAUDE.md");
        let instructions = "# Claude Instructions\nBe helpful and thorough";
        fs::write(&claude_path, instructions).unwrap();

        let adapter = ClaudeAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert_eq!(config.steering[0].name, "claude-instructions");
        assert_eq!(config.steering[0].content, instructions);
        assert_eq!(config.steering[0].format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_claude_adapter_priority() {
        let adapter = ClaudeAdapter::new();
        assert_eq!(adapter.priority(), 50);
    }

    #[test]
    fn test_claude_adapter_name() {
        let adapter = ClaudeAdapter::new();
        assert_eq!(adapter.name(), "claude");
    }
}
