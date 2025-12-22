//! Configuration hot-reload integration
//!
//! This module integrates configuration loading, hot-reload watching, and provider
//! chain updates. It enables runtime configuration changes without restart.

use std::{path::Path, sync::Arc};

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::{
    config::ConfigManager, error::IdeResult, hot_reload::HotReloadManager, lsp_monitor::LspMonitor,
    provider_chain::ProviderChainManager, types::IdeIntegrationConfig,
};

/// Configuration hot-reload coordinator
pub struct ConfigHotReloadCoordinator {
    /// Current configuration
    config: Arc<RwLock<IdeIntegrationConfig>>,
    /// Hot-reload manager
    hot_reload: Arc<HotReloadManager>,
    /// LSP monitor
    lsp_monitor: Arc<RwLock<Option<LspMonitor>>>,
    /// Provider chain manager
    provider_chain: Arc<RwLock<Option<ProviderChainManager>>>,
    /// Configuration file path
    config_path: String,
}

impl ConfigHotReloadCoordinator {
    /// Create a new configuration hot-reload coordinator
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        let config_path_str = config_path.as_ref().to_string_lossy().to_string();
        ConfigHotReloadCoordinator {
            config: Arc::new(RwLock::new(ConfigManager::default_config())),
            hot_reload: Arc::new(HotReloadManager::new(&config_path_str)),
            lsp_monitor: Arc::new(RwLock::new(None)),
            provider_chain: Arc::new(RwLock::new(None)),
            config_path: config_path_str,
        }
    }

    /// Load initial configuration
    pub async fn load_config(&self) -> IdeResult<IdeIntegrationConfig> {
        debug!("Loading initial configuration from {}", self.config_path);
        let config = ConfigManager::load_from_file(&self.config_path).await?;
        let mut current_config = self.config.write().await;
        *current_config = config.clone();
        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Get current configuration
    pub async fn get_config(&self) -> IdeIntegrationConfig {
        self.config.read().await.clone()
    }

    /// Reload configuration from file
    pub async fn reload_config(&self) -> IdeResult<()> {
        debug!("Reloading configuration from {}", self.config_path);
        let new_config = ConfigManager::load_from_file(&self.config_path).await?;

        let mut current_config = self.config.write().await;
        *current_config = new_config.clone();

        info!("Configuration reloaded successfully");

        // Update provider chain if it exists
        if let Some(provider_chain) = self.provider_chain.read().await.as_ref() {
            provider_chain.update_config(new_config).await?;
        }

        Ok(())
    }

    /// Start watching configuration file for changes
    pub async fn start_watching(&self, check_interval_ms: u64) -> IdeResult<()> {
        info!(
            "Starting configuration file watcher with {}ms interval",
            check_interval_ms
        );

        // Register callback for configuration changes
        let coordinator = self.clone_arc();
        self.hot_reload
            .on_config_change(Box::new(move || {
                let coordinator = coordinator.clone();
                tokio::spawn(async move {
                    if let Err(e) = coordinator.reload_config().await {
                        warn!("Failed to reload configuration: {}", e);
                    }
                });
            }))
            .await?;

        // Start file watcher
        self.hot_reload.start_watching(check_interval_ms).await?;

        Ok(())
    }

    /// Start LSP health checks
    pub async fn start_lsp_health_checks(&self, interval_ms: u64) -> IdeResult<()> {
        let config = self.config.read().await;

        if !config.providers.external_lsp.enabled {
            debug!("External LSP is disabled, skipping health checks");
            return Ok(());
        }

        let monitor = LspMonitor::new(config.providers.external_lsp.servers.clone());

        // Register callback for availability changes
        let coordinator = self.clone_arc();
        let callback = Arc::new(move |language: &str, available: bool| {
            let coordinator = coordinator.clone();
            let language = language.to_string();
            tokio::spawn(async move {
                if let Err(e) = coordinator
                    .hot_reload
                    .notify_provider_availability_changed(&language, available)
                    .await
                {
                    warn!("Failed to notify provider availability change: {}", e);
                }
            });
        });

        monitor.on_availability_changed(callback).await?;

        // Start health checks
        monitor.start_health_checks(interval_ms).await?;

        let mut lsp_monitor = self.lsp_monitor.write().await;
        *lsp_monitor = Some(monitor);

        info!("LSP health checks started");
        Ok(())
    }

    /// Set provider chain manager for updates
    pub async fn set_provider_chain(&self, provider_chain: ProviderChainManager) {
        let mut pc = self.provider_chain.write().await;
        *pc = Some(provider_chain);
    }

    /// Clone as Arc for use in callbacks
    fn clone_arc(&self) -> Arc<Self> {
        Arc::new(ConfigHotReloadCoordinator {
            config: self.config.clone(),
            hot_reload: self.hot_reload.clone(),
            lsp_monitor: self.lsp_monitor.clone(),
            provider_chain: self.provider_chain.clone(),
            config_path: self.config_path.clone(),
        })
    }
}

impl Clone for ConfigHotReloadCoordinator {
    fn clone(&self) -> Self {
        ConfigHotReloadCoordinator {
            config: self.config.clone(),
            hot_reload: self.hot_reload.clone(),
            lsp_monitor: self.lsp_monitor.clone(),
            provider_chain: self.provider_chain.clone(),
            config_path: self.config_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = ConfigHotReloadCoordinator::new("/tmp/config.yaml");
        assert_eq!(coordinator.config_path, "/tmp/config.yaml");
    }

    #[tokio::test]
    async fn test_get_default_config() {
        let coordinator = ConfigHotReloadCoordinator::new("/tmp/config.yaml");
        let config = coordinator.get_config().await;
        assert!(config.providers.external_lsp.enabled);
        assert!(config.providers.builtin_providers.enabled);
    }

    #[tokio::test]
    async fn test_clone_coordinator() {
        let coordinator = ConfigHotReloadCoordinator::new("/tmp/config.yaml");
        let cloned = coordinator.clone();
        assert_eq!(cloned.config_path, coordinator.config_path);
    }
}
