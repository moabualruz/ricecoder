//! Watch mode for continuous file monitoring and index updates
//!
//! This module provides continuous file system monitoring with automatic
//! index rebuilding when files change.

use crate::error::RiceGrepError;
use crate::search::{IndexManager, RegexSearchEngine, SearchEngine, ProgressVerbosity};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::time;

/// Watch mode configuration
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Paths to watch
    pub paths: Vec<PathBuf>,
    /// Timeout between updates (in seconds)
    pub timeout: Option<u64>,
    /// Clear screen between updates
    pub clear_screen: bool,
    /// Debounce delay for file changes (in milliseconds)
    pub debounce_ms: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            paths: vec![],
            timeout: None,
            clear_screen: false,
            debounce_ms: 500, // 500ms debounce
        }
    }
}

/// Watch mode engine for continuous monitoring
pub struct WatchEngine {
    config: WatchConfig,
    index_manager: IndexManager,
}

impl WatchEngine {
    /// Create a new watch engine
    pub fn new(config: WatchConfig, index_dir: PathBuf) -> Self {
        Self {
            config,
            index_manager: IndexManager::new(index_dir),
        }
    }

    /// Start watch mode
    pub async fn start(&mut self) -> Result<(), RiceGrepError> {
        println!("🔍 Starting watch mode...");
        println!("📁 Watching paths: {:?}", self.config.paths);
        if let Some(timeout) = self.config.timeout {
            println!("⏱️  Timeout: {} seconds", timeout);
        }
        println!("💡 File changes will automatically update the search index");
        println!("🛑 Press Ctrl+C to stop");
        println!();

        // Create the file watcher
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Watch the specified paths
        for path in &self.config.paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
            println!("Watching: {}", path.display());
        }

        // Initial search and display
        self.perform_search_and_display().await?;

        // Set up timeout if specified
        let start_time = std::time::Instant::now();
        let timeout_duration = self.config.timeout.map(|t| Duration::from_secs(t));

        loop {
            // Check for timeout
            if let Some(duration) = timeout_duration {
                if start_time.elapsed() >= duration {
                    println!("\nWatch timeout reached, exiting...");
                    break;
                }
            }

            // Wait for file change events with timeout
            match rx.recv_timeout(Duration::from_millis(self.config.debounce_ms)) {
                Ok(Ok(event)) => {
                    println!("\n📝 File change detected: {:?}", event.kind);
                    println!("🔄 Updating search index...");

                    let update_start = std::time::Instant::now();
                    let mut search_engine = RegexSearchEngine::new();
                    search_engine.build_index(&self.config.paths, ProgressVerbosity::Normal).await?;
                    let update_time = update_start.elapsed();

                    println!("✅ Index updated in {:.2}s", update_time.as_secs_f64());

                    // Perform search and display results
                    self.perform_search_and_display().await?;
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {}", e);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // No events in the timeout period, continue
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    println!("\nWatch channel disconnected, exiting...");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Perform search and display results
    async fn perform_search_and_display(&self) -> Result<(), RiceGrepError> {
        // For now, this is a placeholder - in a real implementation,
        // we'd store the search query and re-execute it
        println!("Search results would be displayed here");
        println!("(Watch mode search integration pending)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_watch_config() {
        let config = WatchConfig {
            paths: vec![PathBuf::from("/tmp")],
            timeout: Some(60),
            clear_screen: true,
            debounce_ms: 1000,
        };

        let temp_dir = tempdir().unwrap();
        let mut engine = WatchEngine::new(config, temp_dir.path().to_path_buf());

        // Just test that it can be created
        assert!(!engine.config.paths.is_empty());
    }
}