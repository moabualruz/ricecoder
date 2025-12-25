//! Process configuration

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for spawning a process
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Executable command
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory (None = current dir)
    pub working_dir: Option<PathBuf>,
    /// Environment variables (added to parent env)
    pub env: HashMap<String, String>,
    /// Timeout for process execution (None = no timeout)
    pub timeout: Option<Duration>,
    /// Capture stdout
    pub capture_stdout: bool,
    /// Capture stderr
    pub capture_stderr: bool,
}

impl ProcessConfig {
    /// Create new process configuration
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: vec![],
            working_dir: None,
            env: HashMap::new(),
            timeout: None,
            capture_stdout: true,
            capture_stderr: true,
        }
    }

    /// Set command arguments
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set timeout in seconds
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    /// Set timeout duration
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Enable/disable stdout capture
    pub fn capture_stdout(mut self, capture: bool) -> Self {
        self.capture_stdout = capture;
        self
    }

    /// Enable/disable stderr capture
    pub fn capture_stderr(mut self, capture: bool) -> Self {
        self.capture_stderr = capture;
        self
    }
}
