//! PTY configuration value object

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// PTY session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConfig {
    /// Command to execute (default: shell)
    pub command: Option<String>,
    /// Command arguments
    pub args: Option<Vec<String>>,
    /// Working directory (default: current)
    pub cwd: Option<PathBuf>,
    /// Session title
    pub title: Option<String>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            command: None,
            args: None,
            cwd: None,
            title: None,
            env: None,
        }
    }
}

impl PtyConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set command
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set args
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = Some(args);
        self
    }

    /// Set working directory
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set environment variables
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = Some(env);
        self
    }
}
