//! File Watch Management for MCP
//!
//! Manages watch lifecycle tied to MCP server with graceful shutdown.

mod indexer;
mod tracker;

pub use indexer::update_index_for_changes;
pub use tracker::ChangeTracker;

use anyhow::{Context, Result};
use ricegrep::admin::AdminToolset;
use std::path::PathBuf;
use tokio::time::Duration;

const MCP_AUTO_WATCH_DELAY_SECS: u64 = 5;

/// Manages watch lifecycle tied to MCP server
pub struct WatchManager {
    handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl WatchManager {
    pub fn new() -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        Self {
            handle: None,
            shutdown_tx,
        }
    }

    /// Start watch operation with index directory
    pub fn start_with_index(&mut self, watch_args: crate::WatchArgs, index_dir: PathBuf) {
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            // Wait for delay before starting watch
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(MCP_AUTO_WATCH_DELAY_SECS)) => {
                    // Create toolset for index management wrapped in Arc for thread safety
                    let toolset = std::sync::Arc::new(AdminToolset::new(index_dir, None));
                    let root_path = watch_args.paths.get(0).map(|p| PathBuf::from(p)).unwrap_or_else(|| PathBuf::from("."));

                    // Start watch with shutdown signal
                    let _ = run_watch_with_shutdown(
                        watch_args,
                        shutdown_rx.resubscribe(),
                        toolset,
                        root_path,
                    ).await;
                }
                _ = shutdown_rx.recv() => {
                    // Shutdown during delay
                    tracing::info!("Watch cancelled before start");
                    return;
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Gracefully shutdown watch
    pub async fn shutdown(&mut self) -> Result<()> {
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        // Wait for watch to exit (with timeout)
        if let Some(handle) = self.handle.take() {
            tokio::time::timeout(Duration::from_secs(5), handle)
                .await
                .context("Watch shutdown timed out after 5s")??;
        }

        Ok(())
    }
}

/// Run watch with shutdown signal support
async fn run_watch_with_shutdown(
    args: crate::WatchArgs,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    toolset: std::sync::Arc<AdminToolset>,
    root_path: PathBuf,
) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).context("Failed to create file watcher")?;

    for path in &args.paths {
        watcher
            .watch(path.as_ref(), notify::RecursiveMode::Recursive)
            .context("Failed to watch path")?;
    }

    tracing::info!("Watch started for {:?}", args.paths);

    let mut change_tracker = ChangeTracker::new();
    let recv_timeout = Duration::from_millis(100);

    loop {
        // Check for shutdown signal
        match shutdown_rx.try_recv() {
            Ok(_) | Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                // Flush pending changes before shutdown
                process_tracked_changes(&mut change_tracker, &toolset, &root_path).await;
                tracing::info!("Watch received shutdown signal");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                // Continue watching
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                // Broadcast buffer lagged, still process shutdown
                tracing::warn!("Watch shutdown signal lagged, continuing");
            }
        }

        // Check for file events
        match rx.recv_timeout(recv_timeout) {
            Ok(Ok(event)) => {
                handle_watch_event(&mut change_tracker, &event, &args);
            }
            Ok(Err(e)) => {
                // Include file paths if available in the error event
                let path_context = if !e.paths.is_empty() {
                    format!(
                        " for files: {}",
                        e.paths
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    String::new()
                };
                tracing::error!("Watch error processing event: {}{}", e, path_context);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Process batch when debounce window expires
                if change_tracker.is_ready() && change_tracker.has_changes() {
                    process_tracked_changes(&mut change_tracker, &toolset, &root_path).await;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                tracing::warn!("Watch channel disconnected");
                break;
            }
        }
    }

    Ok(())
}

/// Handle individual watch events and track changes with debouncing
fn handle_watch_event(tracker: &mut ChangeTracker, event: &notify::Event, args: &crate::WatchArgs) {
    use notify::EventKind;
    use ricegrep::indexing_optimization::FileChangeKind;

    match event.kind {
        EventKind::Create(_) => {
            for path in &event.paths {
                if path.is_file() {
                    tracing::debug!("File created: {}", path.display());
                    tracker.record_change(path.clone(), FileChangeKind::Create);
                }
            }
        }
        EventKind::Modify(_) => {
            for path in &event.paths {
                if path.is_file() {
                    tracing::debug!("File modified: {}", path.display());
                    tracker.record_change(path.clone(), FileChangeKind::Modify);
                }
            }
        }
        EventKind::Remove(_) => {
            for path in &event.paths {
                tracing::debug!("File removed: {}", path.display());
                tracker.record_change(path.clone(), FileChangeKind::Delete);
            }
        }
        _ => {
            // Ignore other event kinds (access, metadata changes, etc.)
        }
    }

    // Clear screen on first change if requested
    if args.clear_screen && tracker.change_count() == 1 {
        print!("\x1B[2J\x1B[1;1H");
    }
}

/// Process accumulated changes and update index
async fn process_tracked_changes(
    tracker: &mut ChangeTracker,
    toolset: &std::sync::Arc<AdminToolset>,
    root_path: &std::path::Path,
) {
    if !tracker.has_changes() {
        return;
    }

    let changes = tracker.take_changes();
    let count = changes.len();

    tracing::info!("Tracked {} file change(s)", count);
    for path in changes.iter().take(5) {
        tracing::debug!("  - {}", path.display());
    }

    if count > 5 {
        tracing::debug!("  ... and {} more", count - 5);
    }

    // Update index for changed files
    match update_index_for_changes(toolset, root_path, &changes).await {
        Ok(updated) => {
            if updated > 0 {
                tracing::info!("Updated index for {} file(s)", updated);
            }
        }
        Err(e) => {
            // Include file paths context in error message
            let file_context = if changes.len() <= 3 {
                format!(
                    " (files: {})",
                    changes
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                format!(" ({} files changed)", changes.len())
            };
            tracing::error!("Failed to update index for changes{}: {}", file_context, e);
            // Continue watching even if update fails
        }
    }
}
