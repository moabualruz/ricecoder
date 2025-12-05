//! Process lifecycle management

use crate::error::{ExternalLspError, Result};
use crate::types::{ClientState, LspServerConfig};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

/// Manages LSP server process lifecycle
pub struct ProcessManager {
    /// Configuration for the LSP server
    config: LspServerConfig,
    /// Current process handle
    process: Option<Child>,
    /// Current state
    state: ClientState,
    /// Number of restart attempts
    restart_count: u32,
    /// Time of last restart attempt
    last_restart_attempt: Option<Instant>,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new(config: LspServerConfig) -> Self {
        Self {
            config,
            process: None,
            state: ClientState::Stopped,
            restart_count: 0,
            last_restart_attempt: None,
        }
    }

    /// Get the current state
    pub fn state(&self) -> ClientState {
        self.state
    }

    /// Get the restart count
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    /// Spawn the LSP server process
    pub async fn spawn(&mut self) -> Result<()> {
        if self.state != ClientState::Stopped {
            return Err(ExternalLspError::ProtocolError(
                format!("Cannot spawn process in state: {:?}", self.state),
            ));
        }

        self.state = ClientState::Starting;
        debug!(
            language = %self.config.language,
            executable = %self.config.executable,
            "Starting LSP server process"
        );

        // Build the command
        let mut cmd = Command::new(&self.config.executable);
        cmd.args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        // Spawn the process
        match cmd.spawn() {
            Ok(child) => {
                info!(
                    language = %self.config.language,
                    executable = %self.config.executable,
                    pid = ?child.id(),
                    "LSP server process spawned successfully"
                );

                // Store the process handle
                self.process = Some(child);
                self.state = ClientState::Running;
                self.restart_count = 0;
                Ok(())
            }
            Err(e) => {
                error!(
                    language = %self.config.language,
                    executable = %self.config.executable,
                    error = %e,
                    "Failed to spawn LSP server process"
                );
                self.state = ClientState::Stopped;
                Err(ExternalLspError::SpawnFailed(e))
            }
        }
    }

    /// Gracefully shutdown the process
    pub async fn shutdown(&mut self) -> Result<()> {
        if self.state == ClientState::Stopped {
            return Ok(());
        }

        self.state = ClientState::ShuttingDown;
        debug!(
            language = %self.config.language,
            "Shutting down LSP server process"
        );

        if let Some(mut child) = self.process.take() {
            // Try graceful shutdown first
            if let Err(e) = child.kill().await {
                warn!(
                    language = %self.config.language,
                    error = %e,
                    "Failed to kill LSP server process"
                );
            }

            // Wait for process to exit
            match tokio::time::timeout(Duration::from_secs(5), child.wait()).await {
                Ok(Ok(_)) => {
                    info!(
                        language = %self.config.language,
                        "LSP server process shut down gracefully"
                    );
                }
                Ok(Err(e)) => {
                    warn!(
                        language = %self.config.language,
                        error = %e,
                        "Error waiting for LSP server process to exit"
                    );
                }
                Err(_) => {
                    warn!(
                        language = %self.config.language,
                        "Timeout waiting for LSP server process to exit"
                    );
                }
            }
        }

        self.state = ClientState::Stopped;
        Ok(())
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.process = None;
                    self.state = ClientState::Crashed;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(e) => {
                    error!(
                        language = %self.config.language,
                        error = %e,
                        "Error checking process status"
                    );
                    false
                }
            }
        } else {
            false
        }
    }

    /// Mark the process as unhealthy
    pub fn mark_unhealthy(&mut self) {
        self.state = ClientState::Unhealthy;
        debug!(
            language = %self.config.language,
            "Marked LSP server as unhealthy"
        );
    }

    /// Check if restart is allowed
    pub fn can_restart(&self) -> bool {
        self.restart_count < self.config.max_restarts
    }

    /// Prepare for restart with exponential backoff
    pub fn prepare_restart(&mut self) -> Result<Duration> {
        if !self.can_restart() {
            return Err(ExternalLspError::ServerCrashed {
                reason: format!(
                    "Max restart attempts ({}) exceeded",
                    self.config.max_restarts
                ),
            });
        }

        self.restart_count += 1;
        let backoff = calculate_exponential_backoff(self.restart_count);
        self.last_restart_attempt = Some(Instant::now());

        debug!(
            language = %self.config.language,
            restart_count = self.restart_count,
            backoff_ms = backoff.as_millis(),
            "Preparing to restart LSP server with exponential backoff"
        );

        Ok(backoff)
    }

    /// Get the process stdin if available
    pub fn stdin(&mut self) -> Option<tokio::process::ChildStdin> {
        self.process.as_mut().and_then(|child| child.stdin.take())
    }

    /// Get the process stdout if available
    pub fn stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.process.as_mut().and_then(|child| child.stdout.take())
    }

    /// Get the process stderr if available
    pub fn stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.process.as_mut().and_then(|child| child.stderr.take())
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new(LspServerConfig {
            language: "unknown".to_string(),
            extensions: vec![],
            executable: String::new(),
            args: vec![],
            env: Default::default(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        })
    }
}

/// Calculate exponential backoff duration
/// Formula: min(base * 2^attempt, max_backoff)
fn calculate_exponential_backoff(attempt: u32) -> Duration {
    const BASE_BACKOFF_MS: u64 = 100;
    const MAX_BACKOFF_MS: u64 = 30000; // 30 seconds

    let backoff_ms = BASE_BACKOFF_MS
        .saturating_mul(2_u64.saturating_pow(attempt))
        .min(MAX_BACKOFF_MS);

    Duration::from_millis(backoff_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_calculation() {
        assert_eq!(calculate_exponential_backoff(0), Duration::from_millis(100));
        assert_eq!(calculate_exponential_backoff(1), Duration::from_millis(200));
        assert_eq!(calculate_exponential_backoff(2), Duration::from_millis(400));
        assert_eq!(calculate_exponential_backoff(3), Duration::from_millis(800));
        // Should cap at max
        assert_eq!(
            calculate_exponential_backoff(20),
            Duration::from_millis(30000)
        );
    }

    #[test]
    fn test_process_manager_creation() {
        let config = LspServerConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            executable: "rust-analyzer".to_string(),
            args: vec![],
            env: Default::default(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        };

        let manager = ProcessManager::new(config);
        assert_eq!(manager.state(), ClientState::Stopped);
        assert_eq!(manager.restart_count(), 0);
        assert!(manager.can_restart());
    }

    #[test]
    fn test_restart_limit() {
        let config = LspServerConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            executable: "rust-analyzer".to_string(),
            args: vec![],
            env: Default::default(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 2,
            idle_timeout_ms: 300000,
            output_mapping: None,
        };

        let mut manager = ProcessManager::new(config);
        assert!(manager.can_restart());

        // Simulate restart attempts
        let _ = manager.prepare_restart();
        assert!(manager.can_restart());

        let _ = manager.prepare_restart();
        assert!(!manager.can_restart());
    }
}
