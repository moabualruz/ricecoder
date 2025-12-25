//! Managed child process wrapper

use std::time::Duration;
use tokio::process::Child;
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::{
    config::ProcessConfig,
    error::{ProcessError, Result},
};

/// SIGKILL escalation timeout (200ms - matches OpenCode)
const SIGKILL_TIMEOUT_MS: u64 = 200;

/// Wrapper around tokio::process::Child with lifecycle management
pub struct ManagedChild {
    /// Underlying tokio child process
    child: Child,
    /// Process configuration
    config: ProcessConfig,
    /// Process ID
    pid: u32,
}

impl ManagedChild {
    /// Create new managed child
    pub(crate) fn new(child: Child, config: ProcessConfig) -> Self {
        let pid = child.id().unwrap_or(0);
        Self { child, config, pid }
    }

    /// Get process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Get process configuration
    pub fn config(&self) -> &ProcessConfig {
        &self.config
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        }
    }

    /// Wait for process to exit
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        match self.config.timeout {
            Some(timeout) => {
                tokio::time::timeout(timeout, self.child.wait())
                    .await
                    .map_err(|_| ProcessError::Timeout {
                        seconds: timeout.as_secs(),
                    })?
                    .map_err(Into::into)
            }
            None => self.child.wait().await.map_err(Into::into),
        }
    }

    /// Gracefully shutdown process
    ///
    /// Attempts SIGTERM first, then SIGKILL after timeout.
    pub async fn shutdown(&mut self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        debug!(pid = %self.pid, "Shutting down process");

        // Try graceful kill first
        if let Err(e) = self.child.kill().await {
            warn!(pid = %self.pid, error = %e, "Failed to kill process");
            return Err(ProcessError::KillFailed(e.to_string()));
        }

        // Wait for process to exit
        let timeout = Duration::from_secs(5);
        match tokio::time::timeout(timeout, self.child.wait()).await {
            Ok(Ok(_)) => {
                debug!(pid = %self.pid, "Process shut down gracefully");
                Ok(())
            }
            Ok(Err(e)) => {
                warn!(pid = %self.pid, error = %e, "Error waiting for process");
                Err(ProcessError::KillFailed(e.to_string()))
            }
            Err(_) => {
                warn!(pid = %self.pid, "Timeout waiting for process to exit");
                Err(ProcessError::Timeout {
                    seconds: timeout.as_secs(),
                })
            }
        }
    }

    /// Kill process tree (process and all descendants)
    ///
    /// - Windows: Uses `taskkill /pid <pid> /f /t`
    /// - Unix: Kills process group via negative PID with SIGTERM→SIGKILL
    pub async fn kill_tree(&mut self) -> Result<()> {
        debug!(pid = %self.pid, "Killing process tree");

        #[cfg(windows)]
        {
            use tokio::process::Command;

            let mut killer = Command::new("taskkill")
                .args(["/pid", &self.pid.to_string(), "/f", "/t"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| ProcessError::KillFailed(e.to_string()))?;

            let _ = killer.wait().await;
            debug!(pid = %self.pid, "Windows process tree killed");
            return Ok(());
        }

        #[cfg(unix)]
        {
            use nix::sys::signal::{killpg, Signal};
            use nix::unistd::Pid;

            let pgid = Pid::from_raw(self.pid as i32);

            // Try SIGTERM first
            match killpg(pgid, Signal::SIGTERM) {
                Ok(_) => debug!(pid = %self.pid, "Sent SIGTERM to process group"),
                Err(e) => {
                    warn!(pid = %self.pid, error = %e, "Failed to send SIGTERM, trying process only");
                    let _ = self.child.kill().await;
                }
            }

            // Wait for escalation timeout
            sleep(Duration::from_millis(SIGKILL_TIMEOUT_MS)).await;

            // Try SIGKILL
            match killpg(pgid, Signal::SIGKILL) {
                Ok(_) => debug!(pid = %self.pid, "Sent SIGKILL to process group"),
                Err(e) => {
                    warn!(pid = %self.pid, error = %e, "Failed to send SIGKILL, trying process only");
                    let _ = self.child.kill().await;
                }
            }

        }

        #[allow(unreachable_code)]
        Ok(())
    }

    /// Take stdin handle
    pub fn stdin(&mut self) -> Option<tokio::process::ChildStdin> {
        self.child.stdin.take()
    }

    /// Take stdout handle
    pub fn stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout.take()
    }

    /// Take stderr handle
    pub fn stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.child.stderr.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProcessManager;

    #[tokio::test]
    async fn test_is_running() {
        let manager = ProcessManager::new();
        let config = ProcessConfig::new("sleep").args(&["1"]);

        let mut child = manager.spawn(config).await.unwrap();
        assert!(child.is_running());

        child.wait().await.unwrap();
        assert!(!child.is_running());
    }

    #[tokio::test]
    async fn test_shutdown() {
        let manager = ProcessManager::new();
        let config = ProcessConfig::new("sleep").args(&["10"]);

        let mut child = manager.spawn(config).await.unwrap();
        assert!(child.is_running());

        child.shutdown().await.unwrap();
        assert!(!child.is_running());
    }
}
