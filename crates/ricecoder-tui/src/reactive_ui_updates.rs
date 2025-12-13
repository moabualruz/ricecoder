//! Reactive UI updates system for RiceCoder TUI
//!
//! This module implements:
//! - Automatic UI updates on state changes with reactive rendering
//! - Efficient change detection and batched updates
//! - Live data synchronization with file watching and session sync
//! - Conflict resolution for concurrent edits

use crate::error_handling::{ErrorManager, RiceError, ErrorCategory, ErrorSeverity};
use crate::tea::{AppModel, StateDiff, StateChange, ReactiveState};
use crate::real_time_updates::{RealTimeUpdates, StreamData, StreamType};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::time::{Instant, Duration, interval};
use tokio_util::sync::CancellationToken;

/// Reactive update types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    /// Immediate update (user interaction)
    Immediate,
    /// Batched update (multiple changes)
    Batched,
    /// Background update (file changes, etc.)
    Background,
    /// Periodic update (status checks)
    Periodic,
}

/// Reactive update priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdatePriority {
    /// Critical updates (errors, security)
    Critical = 0,
    /// High priority (user interactions)
    High = 1,
    /// Normal priority (state changes)
    Normal = 2,
    /// Low priority (background updates)
    Low = 3,
}

/// Reactive update batch
#[derive(Debug, Clone)]
pub struct UpdateBatch {
    pub updates: Vec<StateChange>,
    pub priority: UpdatePriority,
    pub timestamp: Instant,
    pub source: String,
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub change_type: FileChangeType,
    pub timestamp: Instant,
}

/// File change types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Session synchronization event
#[derive(Debug, Clone)]
pub struct SessionSyncEvent {
    pub session_id: String,
    pub change_type: SessionChangeType,
    pub data: Vec<u8>,
    pub timestamp: Instant,
}

/// Session change types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionChangeType {
    MessageAdded,
    MessageModified,
    MessageDeleted,
    MetadataChanged,
    SettingsChanged,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Use local changes (overwrite remote)
    UseLocal,
    /// Use remote changes (overwrite local)
    UseRemote,
    /// Merge changes if possible
    Merge,
    /// Prompt user for resolution
    Prompt,
    /// Create separate versions
    Fork,
}

/// Reactive UI renderer
pub struct ReactiveRenderer {
    reactive_state: Arc<RwLock<ReactiveState>>,
    update_batches: Arc<RwLock<Vec<UpdateBatch>>>,
    pending_updates: Arc<RwLock<HashSet<StateChange>>>,
    update_sender: broadcast::Sender<(UpdateType, StateDiff)>,
    batch_interval: Duration,
    max_batch_size: usize,
    is_running: Arc<RwLock<bool>>,
    cancellation_token: CancellationToken,
}

impl ReactiveRenderer {
    /// Create a new reactive renderer
    pub fn new(reactive_state: Arc<RwLock<ReactiveState>>, batch_interval: Duration) -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            reactive_state,
            update_batches: Arc::new(RwLock::new(Vec::new())),
            pending_updates: Arc::new(RwLock::new(HashSet::new())),
            update_sender: tx,
            batch_interval,
            max_batch_size: 10,
            is_running: Arc::new(RwLock::new(false)),
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Start the reactive rendering loop
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.is_running.write().await = true;

        let update_batches = Arc::clone(&self.update_batches);
        let pending_updates = Arc::clone(&self.pending_updates);
        let update_sender = self.update_sender.clone();
        let batch_interval = self.batch_interval;
        let max_batch_size = self.max_batch_size;
        let cancellation_token = self.cancellation_token.clone();

        tokio::spawn(async move {
            let mut interval = interval(batch_interval);

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        // Process batched updates
                        let batches = update_batches.read().await.clone();
                        if !batches.is_empty() {
                            Self::process_update_batches(batches, &update_sender).await;
                            *update_batches.write().await = Vec::new();
                        }

                        // Process pending updates if batch is full
                        let pending_count = pending_updates.read().await.len();
                        if pending_count >= max_batch_size {
                            Self::process_pending_updates(&pending_updates, &update_sender).await;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the reactive rendering loop
    pub async fn stop(&self) {
        self.cancellation_token.cancel();
        *self.is_running.write().await = false;
    }

    /// Queue an update for reactive rendering
    pub async fn queue_update(&self, update_type: UpdateType, diff: StateDiff) {
        match update_type {
            UpdateType::Immediate => {
                // Send immediately
                let _ = self.update_sender.send((update_type, diff));
            }
            UpdateType::Batched => {
                // Add to batch
                let batch = UpdateBatch {
                    updates: diff.changes.clone(),
                    priority: UpdatePriority::Normal,
                    timestamp: Instant::now(),
                    source: "batched".to_string(),
                };

                self.update_batches.write().await.push(batch);
            }
            UpdateType::Background | UpdateType::Periodic => {
                // Add to pending updates
                for change in &diff.changes {
                    self.pending_updates.write().await.insert(change.clone());
                }
            }
        }
    }

    /// Force immediate render of all pending updates
    pub async fn force_render(&self) {
        Self::process_pending_updates(&self.pending_updates, &self.update_sender).await;
        let batches = self.update_batches.read().await.clone();
        if !batches.is_empty() {
            Self::process_update_batches(batches, &self.update_sender).await;
            *self.update_batches.write().await = Vec::new();
        }
    }

    /// Process update batches
    async fn process_update_batches(batches: Vec<UpdateBatch>, sender: &broadcast::Sender<(UpdateType, StateDiff)>) {
        // Sort by priority
        let mut sorted_batches = batches;
        sorted_batches.sort_by_key(|b| b.priority);

        for batch in sorted_batches {
            let diff = StateDiff {
                changes: batch.updates,
            };

            let _ = sender.send((UpdateType::Batched, diff));
        }
    }

    /// Process pending updates
    async fn process_pending_updates(
        pending: &Arc<RwLock<HashSet<StateChange>>>,
        sender: &broadcast::Sender<(UpdateType, StateDiff)>
    ) {
        let updates: Vec<StateChange> = pending.read().await.iter().cloned().collect();
        if !updates.is_empty() {
            let diff = StateDiff { changes: updates };
            let _ = sender.send((UpdateType::Background, diff));
            pending.write().await.clear();
        }
    }

    /// Subscribe to reactive updates
    pub fn subscribe(&self) -> broadcast::Receiver<(UpdateType, StateDiff)> {
        self.update_sender.subscribe()
    }

    /// Check if renderer is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

/// Live data synchronizer
pub struct LiveDataSynchronizer {
    file_watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
    watched_paths: Arc<RwLock<HashSet<PathBuf>>>,
    session_sync: Arc<RwLock<HashMap<String, SessionSyncState>>>,
    conflict_resolver: ConflictResolver,
    update_sender: broadcast::Sender<LiveDataEvent>,
    error_manager: ErrorManager,
    is_running: Arc<RwLock<bool>>,
    cancellation_token: CancellationToken,
}

/// Live data events
#[derive(Debug, Clone)]
pub enum LiveDataEvent {
    FileChanged(FileChangeEvent),
    SessionChanged(SessionSyncEvent),
    ConflictDetected(ConflictInfo),
}

/// Conflict information
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub resource_id: String,
    pub local_version: Vec<u8>,
    pub remote_version: Vec<u8>,
    pub conflict_type: ConflictType,
}

/// Conflict types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    FileModified,
    SessionData,
    Settings,
}

/// Session synchronization state
#[derive(Debug, Clone)]
struct SessionSyncState {
    last_sync: Instant,
    version: u64,
    pending_changes: Vec<SessionSyncEvent>,
}

impl LiveDataSynchronizer {
    /// Create a new live data synchronizer
    pub fn new(error_manager: ErrorManager) -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            file_watcher: Arc::new(RwLock::new(None)),
            watched_paths: Arc::new(RwLock::new(HashSet::new())),
            session_sync: Arc::new(RwLock::new(HashMap::new())),
            conflict_resolver: ConflictResolver::new(),
            update_sender: tx,
            error_manager,
            is_running: Arc::new(RwLock::new(false)),
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Start live synchronization
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.is_running.write().await = true;

        // Start file watching
        self.start_file_watching().await?;

        // Start session synchronization
        self.start_session_sync().await?;

        Ok(())
    }

    /// Stop live synchronization
    pub async fn stop(&self) {
        self.cancellation_token.cancel();
        *self.is_running.write().await = false;

        // Stop file watcher
        if let Some(watcher) = self.file_watcher.write().await.take() {
            drop(watcher);
        }
    }

    /// Watch a file or directory for changes
    pub async fn watch_path(&self, path: PathBuf, recursive: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mode = if recursive { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };

        if let Some(watcher) = self.file_watcher.read().await.as_ref() {
            watcher.watch(&path, mode)?;
            self.watched_paths.write().await.insert(path);
        }

        Ok(())
    }

    /// Stop watching a path
    pub async fn unwatch_path(&self, path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(watcher) = self.file_watcher.read().await.as_ref() {
            watcher.unwatch(path)?;
            self.watched_paths.write().await.remove(path);
        }

        Ok(())
    }

    /// Synchronize session data
    pub async fn sync_session(&self, session_id: String, event: SessionSyncEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.session_sync.write().await;

        let state = sessions.entry(session_id.clone()).or_insert(SessionSyncState {
            last_sync: Instant::now(),
            version: 0,
            pending_changes: Vec::new(),
        });

        // Check for conflicts
        if let Some(conflict) = self.detect_session_conflict(&session_id, &event).await? {
            let _ = self.update_sender.send(LiveDataEvent::ConflictDetected(conflict));
            return Ok(());
        }

        // Apply change
        state.pending_changes.push(event.clone());
        state.version += 1;
        state.last_sync = Instant::now();

        let _ = self.update_sender.send(LiveDataEvent::SessionChanged(event));

        Ok(())
    }

    /// Start file watching system
    async fn start_file_watching(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, mut rx) = mpsc::channel(100);
        let update_sender = self.update_sender.clone();
        let cancellation_token = self.cancellation_token.clone();

        let mut watcher = RecommendedWatcher::new(move |res| {
            let _ = tx.blocking_send(res);
        }, notify::Config::default())?;

        *self.file_watcher.write().await = Some(watcher);

        tokio::spawn(async move {
            let mut pending_events = Vec::new();
            let mut debounce_timer = None;

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => break,
                    result = rx.recv() => {
                        match result {
                            Some(Ok(event)) => {
                                // Collect events for debouncing
                                for path in event.paths {
                                    let change_type = match event.kind {
                                        notify::EventKind::Create(_) => FileChangeType::Created,
                                        notify::EventKind::Modify(_) => FileChangeType::Modified,
                                        notify::EventKind::Remove(_) => FileChangeType::Deleted,
                                        notify::EventKind::Other => continue,
                                    };

                                    let file_event = FileChangeEvent {
                                        path,
                                        change_type,
                                        timestamp: Instant::now(),
                                    };

                                    pending_events.push(file_event);
                                }

                                // Start debounce timer if not already running
                                if debounce_timer.is_none() {
                                    let update_sender_clone = update_sender.clone();
                                    debounce_timer = Some(tokio::spawn(async move {
                                        tokio::time::sleep(Duration::from_millis(100)).await; // 100ms debounce
                                        // Send all pending events
                                        for event in &pending_events {
                                            let _ = update_sender_clone.send(LiveDataEvent::FileChanged(event.clone()));
                                        }
                                        pending_events.clear();
                                    }));
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("File watching error: {}", e);
                            }
                            None => break,
                        }
                    }
                    // Handle debounce timer completion
                    Some(_) = async { if let Some(timer) = &mut debounce_timer { timer.await } else { std::future::pending().await } }, if debounce_timer.is_some() => {
                        debounce_timer = None;
                    }
                }
            }
        });

        Ok(())
    }

    /// Start session synchronization
    async fn start_session_sync(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session_sync = Arc::clone(&self.session_sync);
        let update_sender = self.update_sender.clone();
        let cancellation_token = self.cancellation_token.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Sync every 30 seconds

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => break,
                    _ = interval.tick() => {
                        // Process pending session changes
                        let mut sessions = session_sync.write().await;
                        for (session_id, state) in sessions.iter_mut() {
                            if !state.pending_changes.is_empty() {
                                // In a real implementation, this would sync with a remote server
                                // For now, just mark as synced
                                state.pending_changes.clear();
                                tracing::debug!("Synced session {}", session_id);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Detect session conflicts
    async fn detect_session_conflict(&self, _session_id: &str, _event: &SessionSyncEvent) -> Result<Option<ConflictInfo>, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would check against remote state
        // For now, return None (no conflicts)
        Ok(None)
    }

    /// Subscribe to live data events
    pub fn subscribe(&self) -> broadcast::Receiver<LiveDataEvent> {
        self.update_sender.subscribe()
    }

    /// Check if synchronizer is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

/// Conflict resolver
pub struct ConflictResolver {
    strategies: HashMap<String, ConflictResolution>,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
        }
    }

    /// Set conflict resolution strategy for a resource
    pub fn set_strategy(&mut self, resource_id: String, strategy: ConflictResolution) {
        self.strategies.insert(resource_id, strategy);
    }

    /// Resolve a conflict
    pub fn resolve(&self, conflict: &ConflictInfo) -> ConflictResolution {
        self.strategies.get(&conflict.resource_id)
            .copied()
            .unwrap_or(ConflictResolution::Prompt)
    }
}

/// Reactive UI coordinator that combines all reactive systems
pub struct ReactiveUICoordinator {
    reactive_renderer: ReactiveRenderer,
    live_synchronizer: LiveDataSynchronizer,
    real_time_updates: Arc<RealTimeUpdates>,
    error_manager: ErrorManager,
    is_running: Arc<RwLock<bool>>,
}

impl ReactiveUICoordinator {
    /// Create a new reactive UI coordinator
    pub fn new(
        reactive_state: Arc<RwLock<ReactiveState>>,
        real_time_updates: RealTimeUpdates,
        error_manager: ErrorManager,
    ) -> Self {
        let reactive_renderer = ReactiveRenderer::new(
            Arc::clone(&reactive_state),
            Duration::from_millis(50), // 50ms batch interval
        );

        let live_synchronizer = LiveDataSynchronizer::new(error_manager.clone());

        Self {
            reactive_renderer,
            live_synchronizer,
            real_time_updates: Arc::new(real_time_updates),
            error_manager,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start all reactive systems
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.is_running.write().await = true;

        // Start reactive renderer
        self.reactive_renderer.start().await?;

        // Start live synchronizer
        self.live_synchronizer.start().await?;

        // Start real-time updates processing
        let real_time_updates = Arc::clone(&self.real_time_updates);
        tokio::spawn(async move {
            let _ = real_time_updates.process_updates().await;
        });

        Ok(())
    }

    /// Stop all reactive systems
    pub async fn stop(&self) {
        *self.is_running.write().await = false;

        self.live_synchronizer.stop().await;
        self.reactive_renderer.stop().await;
    }

    /// Get reactive renderer
    pub fn reactive_renderer(&self) -> &ReactiveRenderer {
        &self.reactive_renderer
    }

    /// Get live synchronizer
    pub fn live_synchronizer(&self) -> &LiveDataSynchronizer {
        &self.live_synchronizer
    }

    /// Get real-time updates coordinator
    pub fn real_time_updates(&self) -> &RealTimeUpdates {
        &self.real_time_updates
    }

    /// Check if coordinator is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

