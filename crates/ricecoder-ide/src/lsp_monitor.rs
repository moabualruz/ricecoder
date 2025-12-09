//! LSP server availability monitoring
//!
//! This module monitors external LSP server availability and detects when servers
//! become available or unavailable. It supports periodic health checks and automatic
//! provider switching based on availability changes.

use crate::error::{IdeError, IdeResult};
use crate::types::LspServerConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// LSP server health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspHealthStatus {
    /// Server is healthy and responding
    Healthy,
    /// Server is unhealthy or not responding
    Unhealthy,
    /// Server status is unknown
    Unknown,
}

/// LSP server availability monitor
pub struct LspMonitor {
    /// Server configurations by language
    servers: HashMap<String, LspServerConfig>,
    /// Current health status by language
    health_status: Arc<RwLock<HashMap<String, LspHealthStatus>>>,
    /// Availability change callbacks
    availability_callbacks: Arc<RwLock<Vec<Arc<dyn Fn(&str, bool) + Send + Sync>>>>,
}

impl LspMonitor {
    /// Create a new LSP monitor
    pub fn new(servers: HashMap<String, LspServerConfig>) -> Self {
        let mut health_status = HashMap::new();
        for language in servers.keys() {
            health_status.insert(language.clone(), LspHealthStatus::Unknown);
        }

        LspMonitor {
            servers,
            health_status: Arc::new(RwLock::new(health_status)),
            availability_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a callback for availability changes
    pub async fn on_availability_changed(
        &self,
        callback: Arc<dyn Fn(&str, bool) + Send + Sync>,
    ) -> IdeResult<()> {
        debug!("Registering LSP availability change callback");
        let mut callbacks = self.availability_callbacks.write().await;
        callbacks.push(callback);
        Ok(())
    }

    /// Check health of a specific LSP server
    pub async fn check_server_health(&self, language: &str) -> IdeResult<LspHealthStatus> {
        debug!("Checking health of LSP server for language: {}", language);

        let server_config = self
            .servers
            .get(language)
            .ok_or_else(|| IdeError::config_error(format!("No LSP server configured for {}", language)))?;

        // Simulate health check by attempting to spawn the server process
        // In a real implementation, this would send a health check request to the LSP server
        let status = match tokio::process::Command::new(&server_config.command)
            .args(&server_config.args)
            .arg("--version")
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    debug!("LSP server for {} is healthy", language);
                    LspHealthStatus::Healthy
                } else {
                    warn!("LSP server for {} returned non-zero exit code", language);
                    LspHealthStatus::Unhealthy
                }
            }
            Err(e) => {
                warn!("Failed to check LSP server health for {}: {}", language, e);
                LspHealthStatus::Unhealthy
            }
        };

        // Update status and notify if changed
        let mut health_status = self.health_status.write().await;
        let old_status = health_status.get(language).copied().unwrap_or(LspHealthStatus::Unknown);

        if old_status != status {
            health_status.insert(language.to_string(), status);
            let is_available = status == LspHealthStatus::Healthy;
            info!(
                "LSP server availability changed for {}: {}",
                language,
                if is_available { "available" } else { "unavailable" }
            );

            // Notify callbacks
            let callbacks = self.availability_callbacks.read().await;
            for callback in callbacks.iter() {
                callback(language, is_available);
            }
        }

        Ok(status)
    }

    /// Get current health status of a server
    pub async fn get_server_status(&self, language: &str) -> IdeResult<LspHealthStatus> {
        let health_status = self.health_status.read().await;
        Ok(health_status
            .get(language)
            .copied()
            .unwrap_or(LspHealthStatus::Unknown))
    }

    /// Check health of all configured servers
    pub async fn check_all_servers(&self) -> IdeResult<HashMap<String, LspHealthStatus>> {
        debug!("Checking health of all LSP servers");
        let mut results = HashMap::new();

        for language in self.servers.keys() {
            let status = self.check_server_health(language).await?;
            results.insert(language.clone(), status);
        }

        Ok(results)
    }

    /// Start periodic health checks
    pub async fn start_health_checks(&self, interval_ms: u64) -> IdeResult<()> {
        info!("Starting LSP health checks with {}ms interval", interval_ms);

        let servers = self.servers.clone();
        let health_status = self.health_status.clone();
        let availability_callbacks = self.availability_callbacks.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;

                for (language, server_config) in &servers {
                    match tokio::process::Command::new(&server_config.command)
                        .args(&server_config.args)
                        .arg("--version")
                        .output()
                        .await
                    {
                        Ok(output) => {
                            let new_status = if output.status.success() {
                                LspHealthStatus::Healthy
                            } else {
                                LspHealthStatus::Unhealthy
                            };

                            let mut status = health_status.write().await;
                            let old_status = status.get(language).copied().unwrap_or(LspHealthStatus::Unknown);

                            if old_status != new_status {
                                status.insert(language.clone(), new_status);
                                let is_available = new_status == LspHealthStatus::Healthy;
                                info!(
                                    "LSP server availability changed for {}: {}",
                                    language,
                                    if is_available { "available" } else { "unavailable" }
                                );

                                let callbacks = availability_callbacks.read().await;
                                for callback in callbacks.iter() {
                                    callback(language, is_available);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to check LSP server health for {}: {}", language, e);
                            let mut status = health_status.write().await;
                            let old_status = status.get(language).copied().unwrap_or(LspHealthStatus::Unknown);

                            if old_status != LspHealthStatus::Unhealthy {
                                status.insert(language.clone(), LspHealthStatus::Unhealthy);
                                let callbacks = availability_callbacks.read().await;
                                for callback in callbacks.iter() {
                                    callback(language, false);
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Get all available languages
    pub fn available_languages(&self) -> Vec<String> {
        self.servers.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_server_config(language: &str) -> LspServerConfig {
        LspServerConfig {
            language: language.to_string(),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            timeout_ms: 5000,
        }
    }

    #[tokio::test]
    async fn test_lsp_monitor_creation() {
        let mut servers = HashMap::new();
        servers.insert("rust".to_string(), create_test_server_config("rust"));

        let monitor = LspMonitor::new(servers);
        assert_eq!(monitor.available_languages().len(), 1);
    }

    #[tokio::test]
    async fn test_get_server_status_unknown() {
        let mut servers = HashMap::new();
        servers.insert("rust".to_string(), create_test_server_config("rust"));

        let monitor = LspMonitor::new(servers);
        let status = monitor.get_server_status("rust").await.unwrap();
        assert_eq!(status, LspHealthStatus::Unknown);
    }

    #[tokio::test]
    async fn test_register_availability_callback() {
        let mut servers = HashMap::new();
        servers.insert("rust".to_string(), create_test_server_config("rust"));

        let monitor = LspMonitor::new(servers);
        let callback = Arc::new(|_: &str, _: bool| {});
        assert!(monitor.on_availability_changed(callback).await.is_ok());
    }

    #[tokio::test]
    async fn test_check_all_servers() {
        let mut servers = HashMap::new();
        servers.insert("rust".to_string(), create_test_server_config("rust"));
        servers.insert("typescript".to_string(), create_test_server_config("typescript"));

        let monitor = LspMonitor::new(servers);
        let results = monitor.check_all_servers().await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_check_nonexistent_server() {
        let servers = HashMap::new();
        let monitor = LspMonitor::new(servers);
        let result = monitor.check_server_health("rust").await;
        assert!(result.is_err());
    }
}
