//! Process manager - lifecycle orchestration

use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};

use crate::{
    child::ManagedChild,
    config::ProcessConfig,
    error::{ProcessError, Result},
};

/// Manages process lifecycle
pub struct ProcessManager;

impl ProcessManager {
    /// Create new process manager
    pub fn new() -> Self {
        Self
    }

    /// Spawn a managed process
    ///
    /// # Arguments
    /// * `config` - Process configuration
    ///
    /// # Returns
    /// Managed child process wrapper
    ///
    /// # Examples
    /// ```no_run
    /// use ricecoder_process::{ProcessManager, ProcessConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProcessManager::new();
    /// let config = ProcessConfig::new("echo").args(&["hello"]);
    /// let child = manager.spawn(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn spawn(&self, config: ProcessConfig) -> Result<ManagedChild> {
        debug!(
            command = %config.command,
            args = ?config.args,
            "Spawning process"
        );

        // Build command
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);

        // Set working directory
        if let Some(ref dir) = config.working_dir {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(if config.capture_stdout {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stderr(if config.capture_stderr {
            Stdio::piped()
        } else {
            Stdio::null()
        });

        // Spawn process
        let child = cmd.spawn()?;
        let pid = child.id().ok_or_else(|| {
            ProcessError::SpawnFailed(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get process ID",
            ))
        })?;

        info!(pid = %pid, command = %config.command, "Process spawned");

        Ok(ManagedChild::new(child, config))
    }

    /// Gracefully shutdown a process
    ///
    /// Attempts SIGTERM first, then SIGKILL after timeout.
    ///
    /// # Arguments
    /// * `child` - Managed child process
    ///
    /// # Examples
    /// ```no_run
    /// use ricecoder_process::{ProcessManager, ProcessConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProcessManager::new();
    /// let config = ProcessConfig::new("sleep").args(&["1000"]);
    /// let child = manager.spawn(config).await?;
    /// manager.shutdown(child).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&self, mut child: ManagedChild) -> Result<()> {
        child.shutdown().await
    }

    /// Kill a process tree (process and all descendants)
    ///
    /// # Arguments
    /// * `child` - Managed child process
    ///
    /// # Examples
    /// ```no_run
    /// use ricecoder_process::{ProcessManager, ProcessConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProcessManager::new();
    /// let config = ProcessConfig::new("bash").args(&["-c", "sleep 1000"]);
    /// let child = manager.spawn(config).await?;
    /// manager.kill_tree(child).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn kill_tree(&self, mut child: ManagedChild) -> Result<()> {
        child.kill_tree().await
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_echo() {
        let manager = ProcessManager::new();
        let config = ProcessConfig::new("echo").args(&["hello"]);

        let child = manager.spawn(config).await.unwrap();
        assert!(child.pid() > 0);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let manager = ProcessManager::new();
        let config = ProcessConfig::new("sleep").args(&["10"]);

        let child = manager.spawn(config).await.unwrap();
        manager.shutdown(child).await.unwrap();
    }
}
