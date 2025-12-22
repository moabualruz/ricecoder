//! File watching and change detection
//!
//! This module provides file system watching capabilities with debouncing,
//! event batching, and change detection for external file modifications.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::error::FileError;

/// File change event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChangeEvent {
    /// File was created
    Created(PathBuf),
    /// File was modified
    Modified(PathBuf),
    /// File was deleted
    Deleted(PathBuf),
}

/// Batched file change events
#[derive(Debug, Clone)]
pub struct FileChangeBatch {
    /// Timestamp when the batch was created
    pub timestamp: Instant,
    /// List of file change events in this batch
    pub events: Vec<FileChangeEvent>,
    /// Number of events in this batch
    pub count: usize,
}

/// File watcher configuration
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce delay for file change events (default: 100ms)
    pub debounce_delay: Duration,
    /// Maximum batch size before forcing a flush (default: 100)
    pub max_batch_size: usize,
    /// Whether to watch subdirectories recursively (default: true)
    pub recursive: bool,
    /// File extensions to watch (empty means all files)
    pub file_extensions: Vec<String>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_delay: Duration::from_millis(100),
            max_batch_size: 100,
            recursive: true,
            file_extensions: Vec::new(),
        }
    }
}

/// File watcher for detecting external file changes
pub struct FileWatcher {
    /// Watcher configuration
    config: WatcherConfig,
    /// Internal file system watcher
    watcher: Option<RecommendedWatcher>,
    /// Broadcast channel for batched events
    event_sender: broadcast::Sender<FileChangeBatch>,
    /// Running state
    running: Arc<Mutex<bool>>,
    /// Watched directories
    watched_dirs: Arc<Mutex<Vec<PathBuf>>>,
    /// Pending events buffer
    pending_events: Arc<Mutex<HashMap<PathBuf, (FileChangeEvent, Instant)>>>,
}

impl FileWatcher {
    /// Create a new file watcher with default configuration
    pub fn new() -> Result<Self, FileError> {
        Self::with_config(WatcherConfig::default())
    }

    /// Create a new file watcher with custom configuration
    pub fn with_config(config: WatcherConfig) -> Result<Self, FileError> {
        let (event_tx, _) = broadcast::channel(100);

        Ok(Self {
            config,
            watcher: None,
            event_sender: event_tx,
            running: Arc::new(Mutex::new(false)),
            watched_dirs: Arc::new(Mutex::new(Vec::new())),
            pending_events: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start watching a directory
    pub fn watch(&mut self, path: &Path) -> Result<(), FileError> {
        if !path.exists() {
            return Err(FileError::NotFound(path.to_path_buf()));
        }

        if !path.is_dir() {
            return Err(FileError::InvalidPath(format!(
                "Path is not a directory: {}",
                path.display()
            )));
        }

        // Initialize watcher if not already done
        if self.watcher.is_none() {
            let watcher = RecommendedWatcher::new(
                |res| {
                    // For now, we just log the event. In a real implementation,
                    // you'd want to handle this properly with channels.
                    match res {
                        Ok(event) => debug!("File event: {:?}", event),
                        Err(e) => error!("File watcher error: {}", e),
                    }
                },
                Config::default(),
            )
            .map_err(|e| FileError::WatcherError(format!("Failed to create watcher: {}", e)))?;

            self.watcher = Some(watcher);
        }

        // Start watching the directory
        let watcher = self.watcher.as_mut().unwrap();
        let mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watcher
            .watch(path, mode)
            .map_err(|e| FileError::WatcherError(format!("Failed to watch path: {}", e)))?;

        // Track watched directories
        self.watched_dirs.lock().unwrap().push(path.to_path_buf());

        info!("Started watching directory: {}", path.display());
        Ok(())
    }

    /// Stop watching a directory
    pub fn unwatch(&mut self, path: &Path) -> Result<(), FileError> {
        if let Some(watcher) = &mut self.watcher {
            watcher
                .unwatch(path)
                .map_err(|e| FileError::WatcherError(format!("Failed to unwatch path: {}", e)))?;

            // Remove from tracked directories
            self.watched_dirs.lock().unwrap().retain(|p| p != path);

            info!("Stopped watching directory: {}", path.display());
            Ok(())
        } else {
            Err(FileError::WatcherError(
                "Watcher not initialized".to_string(),
            ))
        }
    }

    /// Start the watcher
    pub fn start(&mut self) -> Result<(), FileError> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(FileError::WatcherError(
                "Watcher already running".to_string(),
            ));
        }
        *running = true;
        drop(running);

        info!("File watcher started");
        Ok(())
    }

    /// Stop the watcher
    pub fn stop(&self) -> Result<(), FileError> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        info!("File watcher stopped");
        Ok(())
    }

    /// Get a receiver for file change batches
    pub fn subscribe(&self) -> broadcast::Receiver<FileChangeBatch> {
        self.event_sender.subscribe()
    }

    /// Check if the watcher is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Get the list of watched directories
    pub fn watched_directories(&self) -> Vec<PathBuf> {
        self.watched_dirs.lock().unwrap().clone()
    }

    /// Process pending events (call this periodically)
    pub fn process_pending_events(&self) -> Result<(), FileError> {
        if !self.is_running() {
            return Ok(());
        }

        let mut pending = self.pending_events.lock().unwrap();
        let now = Instant::now();
        let mut batch_events = Vec::new();

        // Collect events that are old enough to flush
        let mut to_remove = Vec::new();
        for (path, (event, timestamp)) in pending.iter() {
            if now.duration_since(*timestamp) >= self.config.debounce_delay {
                batch_events.push(event.clone());
                to_remove.push(path.clone());
            }
        }

        // Remove flushed events
        for path in to_remove {
            pending.remove(&path);
        }

        // Send batch if we have events
        if !batch_events.is_empty() {
            let count = batch_events.len();
            let batch = FileChangeBatch {
                timestamp: now,
                events: batch_events,
                count,
            };

            if let Err(e) = self.event_sender.send(batch) {
                warn!("Failed to send file change batch: {}", e);
            } else {
                debug!("Sent file change batch with {} events", count);
            }
        }

        Ok(())
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_file_watcher_creation() {
        let watcher = FileWatcher::new().unwrap();
        assert!(!watcher.is_running());
        assert!(watcher.watched_directories().is_empty());
    }

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert_eq!(config.debounce_delay, Duration::from_millis(100));
        assert_eq!(config.max_batch_size, 100);
        assert!(config.recursive);
        assert!(config.file_extensions.is_empty());
    }

    #[test]
    fn test_watch_nonexistent_directory() {
        let mut watcher = FileWatcher::new().unwrap();
        let result = watcher.watch(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_watch_file_instead_of_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        let result = watcher.watch(&file_path);
        assert!(result.is_err());
    }
}
