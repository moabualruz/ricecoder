//! Hot-reload support for configuration and provider availability changes
//!
//! This module provides file watching and change detection for configuration files,
//! as well as callbacks for provider availability changes. It enables runtime updates
//! without requiring application restart.

use crate::error::{IdeError, IdeResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Callback type for configuration changes
pub type ConfigChangeCallback = Box<dyn Fn() + Send + Sync>;

/// Callback type for provider availability changes
pub type ProviderAvailabilityCallback = Box<dyn Fn(&str, bool) + Send + Sync>;

/// Configuration hot-reload manager
pub struct HotReloadManager {
    /// Path to the configuration file being watched
    config_path: PathBuf,
    /// Last known modification time
    last_modified: Arc<RwLock<Option<std::time::SystemTime>>>,
    /// Configuration change callbacks
    config_callbacks: Arc<RwLock<Vec<Arc<ConfigChangeCallback>>>>,
    /// Provider availability callbacks
    provider_callbacks: Arc<RwLock<Vec<Arc<ProviderAvailabilityCallback>>>>,
}

impl HotReloadManager {
    /// Create a new hot-reload manager
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        HotReloadManager {
            config_path: config_path.as_ref().to_path_buf(),
            last_modified: Arc::new(RwLock::new(None)),
            config_callbacks: Arc::new(RwLock::new(Vec::new())),
            provider_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a callback for configuration changes
    pub async fn on_config_change(&self, callback: ConfigChangeCallback) -> IdeResult<()> {
        debug!("Registering configuration change callback");
        let mut callbacks = self.config_callbacks.write().await;
        callbacks.push(Arc::new(callback));
        Ok(())
    }

    /// Register a callback for provider availability changes
    pub async fn on_provider_availability_change(
        &self,
        callback: ProviderAvailabilityCallback,
    ) -> IdeResult<()> {
        debug!("Registering provider availability change callback");
        let mut callbacks = self.provider_callbacks.write().await;
        callbacks.push(Arc::new(callback));
        Ok(())
    }

    /// Check if configuration file has changed
    pub async fn check_config_changed(&self) -> IdeResult<bool> {
        let metadata = tokio::fs::metadata(&self.config_path).await.map_err(|e| {
            IdeError::config_error(format!(
                "Failed to check configuration file metadata: {}",
                e
            ))
        })?;

        let modified = metadata.modified().map_err(|e| {
            IdeError::config_error(format!("Failed to get file modification time: {}", e))
        })?;

        let mut last_modified = self.last_modified.write().await;
        let changed = match *last_modified {
            None => {
                *last_modified = Some(modified);
                true
            }
            Some(prev) => {
                if modified > prev {
                    *last_modified = Some(modified);
                    true
                } else {
                    false
                }
            }
        };

        if changed {
            debug!("Configuration file has changed");
        }

        Ok(changed)
    }

    /// Notify all configuration change callbacks
    pub async fn notify_config_changed(&self) -> IdeResult<()> {
        info!("Notifying configuration change callbacks");
        let callbacks = self.config_callbacks.read().await;
        for callback in callbacks.iter() {
            callback();
        }
        Ok(())
    }

    /// Notify all provider availability change callbacks
    pub async fn notify_provider_availability_changed(
        &self,
        language: &str,
        available: bool,
    ) -> IdeResult<()> {
        info!(
            "Notifying provider availability change: {} (available: {})",
            language, available
        );
        let callbacks = self.provider_callbacks.read().await;
        for callback in callbacks.iter() {
            callback(language, available);
        }
        Ok(())
    }

    /// Start watching configuration file for changes
    pub async fn start_watching(&self, check_interval_ms: u64) -> IdeResult<()> {
        info!(
            "Starting configuration file watcher with {}ms interval",
            check_interval_ms
        );

        let config_path = self.config_path.clone();
        let last_modified = self.last_modified.clone();
        let config_callbacks = self.config_callbacks.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;

                match tokio::fs::metadata(&config_path).await {
                    Ok(metadata) => {
                        if let Ok(modified) = metadata.modified() {
                            let mut last_mod = last_modified.write().await;
                            if let Some(prev) = *last_mod {
                                if modified > prev {
                                    *last_mod = Some(modified);
                                    info!("Configuration file changed, notifying callbacks");
                                    let callbacks = config_callbacks.read().await;
                                    for callback in callbacks.iter() {
                                        callback();
                                    }
                                }
                            } else {
                                *last_mod = Some(modified);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to check configuration file: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_hot_reload_manager_creation() {
        let manager = HotReloadManager::new("/tmp/config.yaml");
        assert_eq!(manager.config_path, PathBuf::from("/tmp/config.yaml"));
    }

    #[tokio::test]
    async fn test_register_config_change_callback() {
        let manager = HotReloadManager::new("/tmp/config.yaml");
        let callback = Box::new(|| {});
        assert!(manager.on_config_change(callback).await.is_ok());
    }

    #[tokio::test]
    async fn test_register_provider_availability_callback() {
        let manager = HotReloadManager::new("/tmp/config.yaml");
        let callback = Box::new(|_: &str, _: bool| {});
        assert!(manager
            .on_provider_availability_change(callback)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_notify_config_changed() {
        let manager = HotReloadManager::new("/tmp/config.yaml");
        let call_count = Arc::new(AtomicUsize::new(0));
        let count_clone = call_count.clone();

        let callback = Box::new(move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        manager.on_config_change(callback).await.unwrap();
        manager.notify_config_changed().await.unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_notify_provider_availability_changed() {
        let manager = HotReloadManager::new("/tmp/config.yaml");
        let call_count = Arc::new(AtomicUsize::new(0));
        let count_clone = call_count.clone();

        let callback = Box::new(move |_: &str, _: bool| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        manager
            .on_provider_availability_change(callback)
            .await
            .unwrap();
        manager
            .notify_provider_availability_changed("rust", true)
            .await
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_multiple_callbacks() {
        let manager = HotReloadManager::new("/tmp/config.yaml");
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        let c1 = count1.clone();
        manager
            .on_config_change(Box::new(move || {
                c1.fetch_add(1, Ordering::SeqCst);
            }))
            .await
            .unwrap();

        let c2 = count2.clone();
        manager
            .on_config_change(Box::new(move || {
                c2.fetch_add(1, Ordering::SeqCst);
            }))
            .await
            .unwrap();

        manager.notify_config_changed().await.unwrap();

        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }
}
