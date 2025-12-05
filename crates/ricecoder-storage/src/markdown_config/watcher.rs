//! File watcher for hot-reload of configuration files
//!
//! This module provides the [`FileWatcher`] which monitors configuration directories
//! for file changes and automatically reloads configurations when files are modified.
//!
//! # Hot-Reload Behavior
//!
//! When a configuration file is modified:
//!
//! 1. File change is detected (within 5 seconds)
//! 2. Configuration is re-parsed and validated
//! 3. If valid, configuration is updated in the registry
//! 4. If invalid, error is logged and previous configuration is retained
//!
//! # Debouncing
//!
//! Rapid file changes are debounced to avoid excessive reloads. The default
//! debounce delay is 500ms, which can be customized with [`FileWatcher::with_debounce`].
//!
//! # Usage
//!
//! ```ignore
//! use ricecoder_storage::markdown_config::{ConfigurationLoader, ConfigRegistry, FileWatcher};
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let registry = Arc::new(ConfigRegistry::new());
//!     let loader = Arc::new(ConfigurationLoader::new(registry.clone()));
//!
//!     let paths = vec![
//!         PathBuf::from("~/.ricecoder/agents"),
//!         PathBuf::from("projects/ricecoder/.agent"),
//!     ];
//!
//!     let mut watcher = FileWatcher::new(loader, paths);
//!
//!     // Start watching for changes
//!     watcher.watch().await?;
//!
//!     Ok(())
//! }
//! ```

use crate::markdown_config::error::{MarkdownConfigError, MarkdownConfigResult};
use crate::markdown_config::loader::ConfigurationLoader;
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// File watcher for monitoring configuration file changes
///
/// Monitors configuration directories for file modifications and triggers
/// configuration reloads when changes are detected. Includes debouncing to
/// avoid excessive reloads during rapid file changes.
pub struct FileWatcher {
    loader: Arc<ConfigurationLoader>,
    paths: Vec<PathBuf>,
    debounce_ms: u64,
    is_watching: Arc<RwLock<bool>>,
}

impl FileWatcher {
    /// Create a new file watcher
    ///
    /// # Arguments
    /// * `loader` - The configuration loader to use for reloading
    /// * `paths` - Directories to monitor for changes
    /// * `debounce_ms` - Debounce delay in milliseconds (default: 500ms)
    pub fn new(loader: Arc<ConfigurationLoader>, paths: Vec<PathBuf>) -> Self {
        Self {
            loader,
            paths,
            debounce_ms: 500,
            is_watching: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new file watcher with custom debounce delay
    pub fn with_debounce(
        loader: Arc<ConfigurationLoader>,
        paths: Vec<PathBuf>,
        debounce_ms: u64,
    ) -> Self {
        Self {
            loader,
            paths,
            debounce_ms,
            is_watching: Arc::new(RwLock::new(false)),
        }
    }

    /// Start watching configuration directories for changes
    ///
    /// This method runs indefinitely, monitoring for file changes and
    /// triggering reloads when detected. It should be run in a separate task.
    pub async fn watch(&self) -> MarkdownConfigResult<()> {
        // Create a channel for file system events
        let (tx, rx) = mpsc::channel();

        // Create a watcher using the recommended API
        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    if let Err(e) = tx.send(event) {
                        error!("Failed to send file watch event: {}", e);
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            }
        })
        .map_err(|e| {
            MarkdownConfigError::watch_error(format!("Failed to create file watcher: {}", e))
        })?;

        // Watch all configured paths
        for path in &self.paths {
            if path.exists() {
                watcher
                    .watch(path, RecursiveMode::Recursive)
                    .map_err(|e| {
                        MarkdownConfigError::watch_error(format!(
                            "Failed to watch path {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                debug!("Watching configuration directory: {}", path.display());
            }
        }

        // Mark as watching
        *self.is_watching.write().await = true;

        // Process file system events
        let mut last_reload = std::time::Instant::now();

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    // Check if this is a modification event for a markdown config file
                    if self.is_config_file_event(&event) {
                        let now = std::time::Instant::now();
                        let elapsed = now.duration_since(last_reload);

                        // Debounce: only reload if enough time has passed
                        if elapsed.as_millis() as u64 >= self.debounce_ms {
                            debug!("Configuration file changed, reloading...");
                            self.reload_configurations().await;
                            last_reload = now;
                        } else {
                            debug!(
                                "Debouncing configuration reload ({}ms remaining)",
                                self.debounce_ms - elapsed.as_millis() as u64
                            );
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout is normal, just continue
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel disconnected, stop watching
                    break;
                }
            }
        }

        *self.is_watching.write().await = false;
        Ok(())
    }

    /// Check if an event is for a configuration file
    fn is_config_file_event(&self, event: &notify::Event) -> bool {
        use notify::EventKind;

        // Only care about write/create events
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {}
            _ => return false,
        }

        // Check if any path is a configuration file
        event.paths.iter().any(|path| {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    return name_str.ends_with(".agent.md")
                        || name_str.ends_with(".mode.md")
                        || name_str.ends_with(".command.md");
                }
            }
            false
        })
    }

    /// Reload all configurations from watched directories
    async fn reload_configurations(&self) {
        match self.loader.load_all(&self.paths).await {
            Ok((success, errors, error_list)) => {
                debug!(
                    "Configuration reload complete: {} successful, {} failed",
                    success, errors
                );

                if !error_list.is_empty() {
                    for (path, error) in error_list {
                        warn!(
                            "Failed to load configuration from {}: {}",
                            path.display(),
                            error
                        );
                    }
                }
            }
            Err(e) => {
                error!("Failed to reload configurations: {}", e);
            }
        }
    }

    /// Check if watcher is currently watching
    pub async fn is_watching(&self) -> bool {
        *self.is_watching.read().await
    }

    /// Stop watching (by dropping the watcher)
    pub async fn stop(&self) {
        *self.is_watching.write().await = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_config::registry::ConfigRegistry;

    #[test]
    fn test_file_watcher_creation() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let paths = vec![PathBuf::from("/tmp")];

        let watcher = FileWatcher::new(loader, paths.clone());
        assert_eq!(watcher.paths, paths);
        assert_eq!(watcher.debounce_ms, 500);
    }

    #[test]
    fn test_file_watcher_custom_debounce() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let paths = vec![PathBuf::from("/tmp")];

        let watcher = FileWatcher::with_debounce(loader, paths, 1000);
        assert_eq!(watcher.debounce_ms, 1000);
    }

    #[test]
    fn test_is_config_file_event() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let paths = vec![PathBuf::from("/tmp")];
        let watcher = FileWatcher::new(loader, paths);

        // Test agent file
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/tmp/test.agent.md")],
            attrs: Default::default(),
        };
        assert!(watcher.is_config_file_event(&event));

        // Test mode file
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/tmp/test.mode.md")],
            attrs: Default::default(),
        };
        assert!(watcher.is_config_file_event(&event));

        // Test command file
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/tmp/test.command.md")],
            attrs: Default::default(),
        };
        assert!(watcher.is_config_file_event(&event));

        // Test non-config file
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/tmp/test.md")],
            attrs: Default::default(),
        };
        assert!(!watcher.is_config_file_event(&event));

        // Test non-modify event
        let event = notify::Event {
            kind: notify::EventKind::Access(notify::event::AccessKind::Read),
            paths: vec![PathBuf::from("/tmp/test.agent.md")],
            attrs: Default::default(),
        };
        assert!(!watcher.is_config_file_event(&event));
    }

    #[tokio::test]
    async fn test_watcher_is_watching() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let paths = vec![PathBuf::from("/tmp")];

        let watcher = FileWatcher::new(loader, paths);
        assert!(!watcher.is_watching().await);
    }
}
