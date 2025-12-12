//! Hot configuration reloading system
//!
//! This module provides runtime configuration reloading with file watching,
//! validation, migration, and conflict resolution.

use crate::config::{Config, ConfigLoader, ConfigValidator};
use crate::error::{StorageError, StorageResult};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{Duration, Instant};

/// Configuration change event
#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    /// Configuration file was modified
    FileModified { path: PathBuf, config_type: ConfigType },
    /// Configuration was reloaded successfully
    Reloaded { old_config: Arc<Config>, new_config: Arc<Config> },
    /// Configuration reload failed
    ReloadFailed { path: PathBuf, error: String },
    /// Configuration validation failed
    ValidationFailed { path: PathBuf, errors: Vec<String> },
}

/// Type of configuration file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Global,
    User,
    Project,
}

/// Configuration watcher for file changes
pub struct ConfigWatcher {
    watcher: RecommendedWatcher,
    watched_paths: HashMap<PathBuf, ConfigType>,
    event_sender: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new() -> StorageResult<(Self, broadcast::Receiver<ConfigChangeEvent>)> {
        let (tx, rx) = broadcast::channel(100);

        let tx_clone = tx.clone();
        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // Handle file change events
                        for path in &event.paths {
                            let _ = tx_clone.send(ConfigChangeEvent::FileModified {
                                path: path.clone(),
                                config_type: ConfigType::Global, // Will be determined by path
                            });
                        }
                    }
                    Err(e) => {
                        tracing::error!("File watching error: {}", e);
                    }
                }
            },
            notify::Config::default(),
        )?;

        Ok((
            Self {
                watcher,
                watched_paths: HashMap::new(),
                event_sender: tx,
            },
            rx,
        ))
    }

    /// Watch a configuration file
    pub fn watch_file(&mut self, path: PathBuf, config_type: ConfigType) -> StorageResult<()> {
        if path.exists() {
            self.watcher.watch(&path, RecursiveMode::NonRecursive)?;
            self.watched_paths.insert(path, config_type);
        }
        Ok(())
    }

    /// Stop watching a configuration file
    pub fn unwatch_file(&mut self, path: &Path) -> StorageResult<()> {
        if self.watched_paths.contains_key(path) {
            self.watcher.unwatch(path)?;
            self.watched_paths.remove(path);
        }
        Ok(())
    }

    /// Get the config type for a path
    pub fn get_config_type(&self, path: &Path) -> ConfigType {
        self.watched_paths
            .get(path)
            .copied()
            .unwrap_or(ConfigType::Global)
    }
}

/// Hot reload manager for configuration
pub struct HotReloadManager {
    current_config: Arc<RwLock<Arc<Config>>>,
    watcher: ConfigWatcher,
    event_receiver: broadcast::Receiver<ConfigChangeEvent>,
    validator: ConfigValidator,
    debounce_duration: Duration,
    last_reload: Arc<RwLock<Instant>>,
}

impl HotReloadManager {
    /// Create a new hot reload manager
    pub async fn new(initial_config: Config) -> StorageResult<Self> {
        let (watcher, receiver) = ConfigWatcher::new()?;

        Ok(Self {
            current_config: Arc::new(RwLock::new(Arc::new(initial_config))),
            watcher,
            event_receiver: receiver,
            validator: ConfigValidator::new(),
            debounce_duration: Duration::from_millis(500), // 500ms debounce
            last_reload: Arc::new(RwLock::new(Instant::now())),
        })
    }

    /// Start watching configuration files
    pub async fn start_watching(&mut self) -> StorageResult<()> {
        // Watch standard configuration paths
        let paths = vec![
            (Some(crate::manager::PathResolver::resolve_global_path()?), ConfigType::Global),
            (Some(crate::manager::PathResolver::resolve_user_path()?), ConfigType::User),
            (Some(crate::manager::PathResolver::resolve_project_path()), ConfigType::Project),
        ];

        for (path_option, config_type) in paths {
            if let Some(path) = path_option {
                let config_file = path.join("ricecoder.yaml");
                self.watcher.watch_file(config_file, config_type)?;
            }
        }

        Ok(())
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> Arc<Config> {
        Arc::clone(&*self.current_config.read().await)
    }

    /// Manually trigger a configuration reload
    pub async fn reload_config(&self) -> StorageResult<Arc<Config>> {
        // Check debounce timing
        let now = Instant::now();
        let last_reload = *self.last_reload.read().await;
        if now.duration_since(last_reload) < self.debounce_duration {
            return Ok(self.get_config().await);
        }

        // Load new configuration
        let loader = ConfigLoader::new();
        let new_config = loader.load_merged()?;

        // Validate the new configuration
        self.validator.validate(&new_config)?;

        // Update current config
        let old_config = self.get_config().await;
        let new_config_arc = Arc::new(new_config);

        *self.current_config.write().await = Arc::clone(&new_config_arc);
        *self.last_reload.write().await = now;

        // Notify listeners
        let _ = self.watcher.event_sender.send(ConfigChangeEvent::Reloaded {
            old_config,
            new_config: Arc::clone(&new_config_arc),
        });

        Ok(new_config_arc)
    }

    /// Process pending configuration change events
    pub async fn process_events(&mut self) -> StorageResult<()> {
        // Non-blocking event processing
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                ConfigChangeEvent::FileModified { path, .. } => {
                    tracing::info!("Configuration file changed: {}", path.display());

                    // Attempt to reload configuration
                    match self.reload_config().await {
                        Ok(new_config) => {
                            tracing::info!("Configuration reloaded successfully");
                            // Additional processing could be done here
                            let _ = new_config;
                        }
                        Err(e) => {
                            tracing::error!("Failed to reload configuration: {}", e);
                            let _ = self.watcher.event_sender.send(ConfigChangeEvent::ReloadFailed {
                                path,
                                error: e.to_string(),
                            });
                        }
                    }
                }
                _ => {} // Other events are handled elsewhere
            }
        }

        Ok(())
    }

    /// Set debounce duration for configuration reloading
    pub fn set_debounce_duration(&mut self, duration: Duration) {
        self.debounce_duration = duration;
    }

    /// Get the event receiver for external listeners
    pub fn event_receiver(&self) -> &broadcast::Receiver<ConfigChangeEvent> {
        &self.event_receiver
    }

    /// Stop watching all files
    pub async fn stop_watching(&mut self) -> StorageResult<()> {
        for path in self.watcher.watched_paths.keys().cloned().collect::<Vec<_>>() {
            self.watcher.unwatch_file(&path)?;
        }
        Ok(())
    }
}

/// Configuration migration helper
pub struct ConfigMigrator;

impl ConfigMigrator {
    /// Migrate configuration from an older version
    pub fn migrate_config(config: &mut Config, from_version: &str, to_version: &str) -> StorageResult<()> {
        tracing::info!("Migrating configuration from {} to {}", from_version, to_version);

        // Example migration: add new default values
        if from_version < "1.1.0" && to_version >= "1.1.0" {
            // Add new default temperature if not set
            if config.defaults.temperature.is_none() {
                config.defaults.temperature = Some(0.7);
            }
        }

        // Add more migrations as needed
        Ok(())
    }

    /// Check if migration is needed
    pub fn needs_migration(_current_config: &Config, _target_version: &str) -> bool {
        // For now, always allow migration
        // In practice, this would check version compatibility
        true
    }
}

/// Configuration conflict resolver
pub struct ConfigConflictResolver;

impl ConfigConflictResolver {
    /// Resolve conflicts between different configuration sources
    pub fn resolve_conflicts(configs: &[&Config]) -> Config {
        if configs.is_empty() {
            return Config::default();
        }

        if configs.len() == 1 {
            return configs[0].clone();
        }

        // For multiple configs, merge them with priority
        // configs[0] has highest priority, configs.last() has lowest
        let mut result = configs[0].clone();

        for config in &configs[1..] {
            Self::merge_config(&mut result, config);
        }

        result
    }

    /// Merge two configurations with conflict resolution
    fn merge_config(target: &mut Config, source: &Config) {
        // Merge providers
        for (key, value) in &source.providers.api_keys {
            if !target.providers.api_keys.contains_key(key) {
                target.providers.api_keys.insert(key.clone(), value.clone());
            }
        }

        for (key, value) in &source.providers.endpoints {
            if !target.providers.endpoints.contains_key(key) {
                target.providers.endpoints.insert(key.clone(), value.clone());
            }
        }

        if target.providers.default_provider.is_none() {
            target.providers.default_provider = source.providers.default_provider.clone();
        }

        // Merge defaults
        if target.defaults.model.is_none() {
            target.defaults.model = source.defaults.model.clone();
        }

        if target.defaults.temperature.is_none() {
            target.defaults.temperature = source.defaults.temperature;
        }

        if target.defaults.max_tokens.is_none() {
            target.defaults.max_tokens = source.defaults.max_tokens;
        }

        // Merge steering rules (add missing ones)
        for rule in &source.steering {
            if !target.steering.iter().any(|r| r.name == rule.name) {
                target.steering.push(rule.clone());
            }
        }

        // Merge custom settings (source overrides target for conflicts)
        for (key, value) in &source.custom {
            target.custom.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_hot_reload_manager_creation() {
        let config = Config::default();
        let manager = HotReloadManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[test]
    fn test_config_conflict_resolution() {
        let config1 = Config {
            providers: ProvidersConfig {
                api_keys: [("key1".to_string(), "value1".to_string())].into(),
                endpoints: HashMap::new(),
                default_provider: Some("provider1".to_string()),
            },
            defaults: DefaultsConfig {
                model: Some("model1".to_string()),
                temperature: Some(0.5),
                max_tokens: Some(100),
            },
            steering: vec![],
            custom: HashMap::new(),
        };

        let config2 = Config {
            providers: ProvidersConfig {
                api_keys: [("key2".to_string(), "value2".to_string())].into(),
                endpoints: HashMap::new(),
                default_provider: Some("provider2".to_string()),
            },
            defaults: DefaultsConfig {
                model: Some("model2".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(200),
            },
            steering: vec![],
            custom: HashMap::new(),
        };

        let configs = vec![&config1, &config2];
        let resolved = ConfigConflictResolver::resolve_conflicts(&configs);

        // config1 should take priority for conflicts
        assert_eq!(resolved.providers.default_provider, Some("provider1".to_string()));
        assert_eq!(resolved.defaults.model, Some("model1".to_string()));
        assert_eq!(resolved.defaults.temperature, Some(0.5));

        // But config2 values should be included where config1 doesn't have them
        assert_eq!(resolved.providers.api_keys.len(), 2);
        assert!(resolved.providers.api_keys.contains_key("key1"));
        assert!(resolved.providers.api_keys.contains_key("key2"));
    }
}