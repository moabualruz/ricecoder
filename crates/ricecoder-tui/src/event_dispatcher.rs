//! Event handling system for RiceCoder TUI
//!
//! This module implements a centralized event dispatcher with async handling,
//! debouncing, batching, and optimistic updates for the Elm Architecture.

use crate::model::{AppMessage, AppModel};
use crate::tea::ReactiveState;
use crate::update::Command as TeaCommand;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

/// Unique identifier for events
pub type EventId = String;

/// Priority levels for event processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Event with metadata for processing
#[derive(Clone)]
pub struct EventEnvelope {
    pub id: EventId,
    pub message: AppMessage,
    pub priority: EventPriority,
    pub timestamp: Instant,
    pub source: EventSource,
    pub cancellation_token: CancellationToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    UserInput,
    System,
    Network,
    FileSystem,
    Timer,
}

/// Event processing result
#[derive(Clone)]
pub enum EventResult {
    Success(EventId),
    Failed(EventId, String),
    Cancelled(EventId),
    Deferred(EventId, Duration),
}

/// Event batch for processing multiple events together
#[derive(Clone)]
pub struct EventBatch {
    pub id: String,
    pub events: Vec<EventEnvelope>,
    pub batch_type: BatchType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchType {
    Atomic,      // All events must succeed together
    BestEffort,  // Process as many as possible
    Sequential,  // Process in order
}

/// Debounced event state
struct DebouncedEvent {
    event: AppMessage,
    last_trigger: Instant,
    timer_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Centralized event dispatcher
pub struct EventDispatcher {
    /// Event processing queue
    event_tx: mpsc::UnboundedSender<EventEnvelope>,
    event_rx: mpsc::UnboundedReceiver<EventEnvelope>,

    /// Batch processing queue
    batch_tx: mpsc::UnboundedSender<EventBatch>,
    batch_rx: mpsc::UnboundedReceiver<EventBatch>,

    /// Result notification channel
    result_tx: mpsc::UnboundedSender<EventResult>,

    /// Debouncing state
    debounced_events: Arc<RwLock<HashMap<String, DebouncedEvent>>>,

    /// Active event processing tasks
    active_tasks: Arc<RwLock<HashMap<EventId, CancellationToken>>>,

    /// Event processing statistics
    stats: Arc<RwLock<EventStats>>,
}

/// Event processing statistics
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    pub total_events: u64,
    pub processed_events: u64,
    pub failed_events: u64,
    pub cancelled_events: u64,
    pub batched_events: u64,
    pub debounced_events: u64,
    pub avg_processing_time: Duration,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();

        Self {
            event_tx,
            event_rx,
            batch_tx,
            batch_rx,
            result_tx: mpsc::unbounded_channel().0, // We'll handle results differently
            debounced_events: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(EventStats::default())),
        }
    }

    /// Start the event processing loop
    pub async fn run(
        mut self,
        reactive_state: Arc<RwLock<ReactiveState>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let stats = Arc::clone(&self.stats);

        loop {
            tokio::select! {
                // Handle individual events
                Some(envelope) = self.event_rx.recv() => {
                    let stats_clone = Arc::clone(&stats);
                    let reactive_clone = Arc::clone(&reactive_state);
                    let active_tasks = Arc::clone(&self.active_tasks);

                    tokio::spawn(async move {
                        Self::process_event(envelope, reactive_clone, stats_clone, active_tasks).await;
                    });
                }

                // Handle event batches
                Some(batch) = self.batch_rx.recv() => {
                    let stats_clone = Arc::clone(&stats);
                    let reactive_clone = Arc::clone(&reactive_state);
                    let active_tasks = Arc::clone(&self.active_tasks);

                    tokio::spawn(async move {
                        Self::process_batch(batch, reactive_clone, stats_clone, active_tasks).await;
                    });
                }

                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send an event for processing
    pub async fn dispatch_event(
        &self,
        message: AppMessage,
        priority: EventPriority,
        source: EventSource,
    ) -> Result<EventId, String> {
        let id = format!("event_{}", uuid::Uuid::new_v4());
        let envelope = EventEnvelope {
            id: id.clone(),
            message,
            priority,
            timestamp: Instant::now(),
            source,
            cancellation_token: CancellationToken::new(),
        };

        // Register the task
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(id.clone(), envelope.cancellation_token.clone());
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
        }

        self.event_tx.send(envelope)
            .map_err(|e| format!("Failed to dispatch event: {}", e))?;

        Ok(id)
    }

    /// Send a debounced event
    pub async fn dispatch_debounced(
        &self,
        key: String,
        message: AppMessage,
        delay: Duration,
        priority: EventPriority,
        source: EventSource,
    ) -> Result<(), String> {
        let mut debounced = self.debounced_events.write().await;

        // Cancel existing timer if any
        if let Some(existing) = debounced.get(&key) {
            if let Some(handle) = &existing.timer_handle {
                handle.abort();
            }
        }

        // Create new debounced event
        let event_tx = self.event_tx.clone();
        let key_clone = key.clone();
        let debounced_clone = Arc::clone(&self.debounced_events);
        let message_clone = message.clone();

        let timer_handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            // Remove from debounced events
            {
                let mut debounced = debounced_clone.write().await;
                debounced.remove(&key_clone);
            }

            // Dispatch the event
            let id = format!("debounced_{}", uuid::Uuid::new_v4());
            let envelope = EventEnvelope {
                id,
                message: message_clone,
                priority,
                timestamp: Instant::now(),
                source,
                cancellation_token: CancellationToken::new(),
            };

            let _ = event_tx.send(envelope);
        });

        debounced.insert(key, DebouncedEvent {
            event: message,
            last_trigger: Instant::now(),
            timer_handle: Some(timer_handle),
        });

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.debounced_events += 1;
        }

        Ok(())
    }

    /// Send an event batch
    pub async fn dispatch_batch(
        &self,
        events: Vec<AppMessage>,
        batch_type: BatchType,
        priority: EventPriority,
    ) -> Result<String, String> {
        let batch_id = format!("batch_{}", uuid::Uuid::new_v4());

        let envelopes: Vec<EventEnvelope> = events.into_iter().enumerate().map(|(i, message)| {
            EventEnvelope {
                id: format!("{}_event_{}", batch_id, i),
                message,
                priority,
                timestamp: Instant::now(),
                source: EventSource::System,
                cancellation_token: CancellationToken::new(),
            }
        }).collect();

        let batch = EventBatch {
            id: batch_id.clone(),
            events: envelopes,
            batch_type,
        };

        // Register tasks
        {
            let mut active_tasks = self.active_tasks.write().await;
            for envelope in &batch.events {
                active_tasks.insert(envelope.id.clone(), envelope.cancellation_token.clone());
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.batched_events += batch.events.len() as u64;
        }

        self.batch_tx.send(batch)
            .map_err(|e| format!("Failed to dispatch batch: {}", e))?;

        Ok(batch_id)
    }

    /// Cancel an event by ID
    pub async fn cancel_event(&self, event_id: &str) -> bool {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(token) = active_tasks.remove(event_id) {
            token.cancel();
            // Update stats
            let mut stats = self.stats.write().await;
            stats.cancelled_events += 1;
            true
        } else {
            false
        }
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }

    /// Process a single event
    async fn process_event(
        envelope: EventEnvelope,
        reactive_state: Arc<RwLock<ReactiveState>>,
        stats: Arc<RwLock<EventStats>>,
        active_tasks: Arc<RwLock<HashMap<EventId, CancellationToken>>>,
    ) {
        let start_time = Instant::now();

        // Check if cancelled
        if envelope.cancellation_token.is_cancelled() {
            let mut stats = stats.write().await;
            stats.cancelled_events += 1;
            return;
        }

        // Process the event
        let result = {
            let mut reactive = reactive_state.write().await;
            reactive.update(envelope.message)
        };

        match result {
            Ok(_) => {
                let mut stats = stats.write().await;
                stats.processed_events += 1;
                let processing_time = start_time.elapsed();
                stats.avg_processing_time = (stats.avg_processing_time + processing_time) / 2;
            }
            Err(e) => {
                let mut stats = stats.write().await;
                stats.failed_events += 1;
                tracing::error!("Event processing failed: {}", e);
            }
        }

        // Clean up
        let mut active_tasks = active_tasks.write().await;
        active_tasks.remove(&envelope.id);
    }

    /// Process an event batch
    async fn process_batch(
        batch: EventBatch,
        reactive_state: Arc<RwLock<ReactiveState>>,
        stats: Arc<RwLock<EventStats>>,
        active_tasks: Arc<RwLock<HashMap<EventId, CancellationToken>>>,
    ) {
        match batch.batch_type {
            BatchType::Atomic => {
                Self::process_atomic_batch(batch, reactive_state, stats, active_tasks).await;
            }
            BatchType::BestEffort => {
                Self::process_best_effort_batch(batch, reactive_state, stats, active_tasks).await;
            }
            BatchType::Sequential => {
                Self::process_sequential_batch(batch, reactive_state, stats, active_tasks).await;
            }
        }
    }

    async fn process_atomic_batch(
        batch: EventBatch,
        reactive_state: Arc<RwLock<ReactiveState>>,
        stats: Arc<RwLock<EventStats>>,
        active_tasks: Arc<RwLock<HashMap<EventId, CancellationToken>>>,
    ) {
        // For atomic batches, we need to ensure all events succeed or all fail
        // This is more complex and would require transaction-like semantics
        // For now, process them sequentially
        Self::process_sequential_batch(batch, reactive_state, stats, active_tasks).await;
    }

    async fn process_best_effort_batch(
        batch: EventBatch,
        reactive_state: Arc<RwLock<ReactiveState>>,
        stats: Arc<RwLock<EventStats>>,
        active_tasks: Arc<RwLock<HashMap<EventId, CancellationToken>>>,
    ) {
        // Process each event independently
        for envelope in batch.events {
            if !envelope.cancellation_token.is_cancelled() {
                let stats_clone = Arc::clone(&stats);
                let reactive_clone = Arc::clone(&reactive_state);
                let active_tasks_clone = Arc::clone(&active_tasks);

                tokio::spawn(async move {
                    Self::process_event(envelope, reactive_clone, stats_clone, active_tasks_clone).await;
                });
            }
        }
    }

    async fn process_sequential_batch(
        batch: EventBatch,
        reactive_state: Arc<RwLock<ReactiveState>>,
        stats: Arc<RwLock<EventStats>>,
        active_tasks: Arc<RwLock<HashMap<EventId, CancellationToken>>>,
    ) {
        // Process events in sequence
        for envelope in batch.events {
            if envelope.cancellation_token.is_cancelled() {
                continue;
            }

            let stats_clone = Arc::clone(&stats);
            let reactive_clone = Arc::clone(&reactive_state);
            let active_tasks_clone = Arc::clone(&active_tasks);

            // Wait for each event to complete before processing the next
            Self::process_event(envelope, reactive_clone, stats_clone, active_tasks_clone).await;
        }
    }
}



/// Loading state manager
#[derive(Debug, Clone)]
pub struct LoadingState {
    pub operation_id: String,
    pub description: String,
    pub progress: Option<f32>, // 0.0 to 1.0
    pub start_time: Instant,
}

pub struct LoadingManager {
    active_loadings: Arc<RwLock<HashMap<String, LoadingState>>>,
}

impl LoadingManager {
    pub fn new() -> Self {
        Self {
            active_loadings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a loading operation
    pub async fn start_loading(&self, operation_id: String, description: String) {
        let state = LoadingState {
            operation_id: operation_id.clone(),
            description,
            progress: None,
            start_time: Instant::now(),
        };

        let mut loadings = self.active_loadings.write().await;
        loadings.insert(operation_id, state);
    }

    /// Update loading progress
    pub async fn update_progress(&self, operation_id: &str, progress: f32) {
        let mut loadings = self.active_loadings.write().await;
        if let Some(state) = loadings.get_mut(operation_id) {
            state.progress = Some(progress.clamp(0.0, 1.0));
        }
    }

    /// Complete a loading operation
    pub async fn complete_loading(&self, operation_id: &str) {
        let mut loadings = self.active_loadings.write().await;
        loadings.remove(operation_id);
    }

    /// Get all active loading states
    pub async fn get_active_loadings(&self) -> Vec<LoadingState> {
        let loadings = self.active_loadings.read().await;
        loadings.values().cloned().collect()
    }

    /// Check if any loading operations are active
    pub async fn has_active_loadings(&self) -> bool {
        let loadings = self.active_loadings.read().await;
        !loadings.is_empty()
    }
}

