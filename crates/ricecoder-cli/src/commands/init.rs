// Initialize a new ricecoder project

use super::Command;
use crate::error::{CliError, CliResult};

/// Initialize a new ricecoder project
pub struct InitCommand {
    pub project_path: Option<String>,
}

impl InitCommand {
    pub fn new(project_path: Option<String>) -> Self {
        Self { project_path }
    }
}

impl Command for InitCommand {
    fn execute(&self) -> CliResult<()> {
        let path = self.project_path.as_deref().unwrap_or(".");

        // Create .agent/ directory structure
        std::fs::create_dir_all(format!("{}/.agent", path)).map_err(CliError::Io)?;

        // Create default configuration
        let config_content = r#"# RiceCoder Project Configuration
# This file configures ricecoder for your project

[project]
name = "My Project"
description = "A ricecoder project"

[providers]
default = "openai"

[storage]
mode = "merged"
"#;

        std::fs::write(format!("{}/.agent/ricecoder.toml", path), config_content)
            .map_err(CliError::Io)?;

        println!("✓ Initialized ricecoder project at {}", path);
        Ok(())
    }
}
