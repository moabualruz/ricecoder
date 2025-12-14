//! Activity logger implementation

use crate::error::{ActivityLogError, ActivityLogResult};
use crate::events::{ActivityEvent, EventFilter, LogLevel};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Logger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Minimum log level to process
    pub min_level: LogLevel,
    /// Whether to enable console output
    pub console_output: bool,
    /// Whether to enable file output
    pub file_output: bool,
    /// File output path (if file_output is true)
    pub file_path: Option<String>,
    /// Maximum file size in MB
    pub max_file_size_mb: u64,
    /// Whether to enable structured JSON output
    pub json_format: bool,
    /// Whether to include timestamps
    pub include_timestamps: bool,
    /// Custom fields to include in all logs
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            console_output: true,
            file_output: false,
            file_path: None,
            max_file_size_mb: 100,
            json_format: false,
            include_timestamps: true,
            custom_fields: HashMap::new(),
        }
    }
}

/// Storage backend for activity logs
#[async_trait]
pub trait LogStorage: Send + Sync {
    /// Store an activity event
    async fn store_event(&self, event: &ActivityEvent) -> ActivityLogResult<()>;

    /// Retrieve events matching a filter
    async fn retrieve_events(&self, filter: &EventFilter) -> ActivityLogResult<Vec<ActivityEvent>>;

    /// Get event count matching a filter
    async fn count_events(&self, filter: &EventFilter) -> ActivityLogResult<u64>;

    /// Delete events matching a filter
    async fn delete_events(&self, filter: &EventFilter) -> ActivityLogResult<u64>;

    /// Perform maintenance operations (cleanup, archiving, etc.)
    async fn maintenance(&self) -> ActivityLogResult<()>;
}

/// In-memory storage implementation for testing and development
pub struct MemoryStorage {
    events: RwLock<Vec<ActivityEvent>>,
    max_events: usize,
}

impl MemoryStorage {
    /// Create a new memory storage with max capacity
    pub fn new(max_events: usize) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            max_events,
        }
    }
}

#[async_trait]
impl LogStorage for MemoryStorage {
    async fn store_event(&self, event: &ActivityEvent) -> ActivityLogResult<()> {
        let mut events = self.events.write().await;

        // Add new event
        events.push(event.clone());

        // Maintain max capacity (remove oldest)
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }

        Ok(())
    }

    async fn retrieve_events(&self, filter: &EventFilter) -> ActivityLogResult<Vec<ActivityEvent>> {
        let events = self.events.read().await;
        let mut filtered: Vec<_> = events
            .iter()
            .filter(|event| filter.matches(event))
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = filter.limit {
            filtered.truncate(limit);
        }

        Ok(filtered)
    }

    async fn count_events(&self, filter: &EventFilter) -> ActivityLogResult<u64> {
        let events = self.events.read().await;
        let count = events
            .iter()
            .filter(|event| filter.matches(event))
            .count() as u64;
        Ok(count)
    }

    async fn delete_events(&self, filter: &EventFilter) -> ActivityLogResult<u64> {
        let mut events = self.events.write().await;
        let initial_len = events.len();

        events.retain(|event| !filter.matches(event));

        let deleted = (initial_len - events.len()) as u64;
        Ok(deleted)
    }

    async fn maintenance(&self) -> ActivityLogResult<()> {
        // Memory storage doesn't need maintenance
        Ok(())
    }
}

/// Main activity logger
pub struct ActivityLogger {
    config: LoggerConfig,
    storage: Arc<dyn LogStorage>,
}

impl ActivityLogger {
    /// Create a new activity logger with default configuration
    pub fn new() -> Self {
        Self::with_config(LoggerConfig::default())
    }

    /// Create a new activity logger with custom configuration
    pub fn with_config(config: LoggerConfig) -> Self {
        Self::with_storage(config, Arc::new(MemoryStorage::new(10000)))
    }

    /// Create a new activity logger with custom storage
    pub fn with_storage(config: LoggerConfig, storage: Arc<dyn LogStorage>) -> Self {
        Self { config, storage }
    }

    /// Log an activity event
    pub async fn log_activity(&self, event: ActivityEvent) -> ActivityLogResult<()> {
        // Validate the event
        event.validate()?;

        // Check if we should log this level
        if !event.should_log(self.config.min_level) {
            return Ok(());
        }

        // Store the event
        self.storage.store_event(&event).await?;

        // Output to configured destinations
        self.output_event(&event).await?;

        Ok(())
    }

    /// Log a simple activity with basic information
    pub async fn log_simple(
        &self,
        level: LogLevel,
        category: crate::events::EventCategory,
        action: String,
        actor: String,
        resource: String,
    ) -> ActivityLogResult<()> {
        let event = ActivityEvent::new(level, category, action, actor, resource);
        self.log_activity(event).await
    }

    /// Log with additional details
    pub async fn log_with_details(
        &self,
        level: LogLevel,
        category: crate::events::EventCategory,
        action: String,
        actor: String,
        resource: String,
        details: serde_json::Value,
    ) -> ActivityLogResult<()> {
        let event = ActivityEvent::new(level, category, action, actor, resource)
            .with_details(details);
        self.log_activity(event).await
    }

    /// Query events matching a filter
    pub async fn query_events(&self, filter: &EventFilter) -> ActivityLogResult<Vec<ActivityEvent>> {
        self.storage.retrieve_events(filter).await
    }

    /// Get event count matching a filter
    pub async fn count_events(&self, filter: &EventFilter) -> ActivityLogResult<u64> {
        self.storage.count_events(filter).await
    }

    /// Delete events matching a filter
    pub async fn delete_events(&self, filter: &EventFilter) -> ActivityLogResult<u64> {
        self.storage.delete_events(filter).await
    }

    /// Perform maintenance operations
    pub async fn maintenance(&self) -> ActivityLogResult<()> {
        self.storage.maintenance().await
    }

    /// Get logger statistics
    pub async fn get_stats(&self) -> ActivityLogResult<LoggerStats> {
        let total_events = self.storage.count_events(&EventFilter::new().with_limit(usize::MAX)).await?;
        let recent_events = self.storage.retrieve_events(
            &EventFilter::new()
                .with_limit(100)
        ).await?;

        let mut level_counts = HashMap::new();
        let mut category_counts = HashMap::new();

        for event in &recent_events {
            *level_counts.entry(event.level).or_insert(0u64) += 1;
            *category_counts.entry(event.category.clone()).or_insert(0u64) += 1;
        }

        Ok(LoggerStats {
            total_events,
            level_counts,
            category_counts,
        })
    }

    /// Output event to configured destinations
    async fn output_event(&self, event: &ActivityEvent) -> ActivityLogResult<()> {
        // Console output
        if self.config.console_output {
            self.output_to_console(event);
        }

        // File output (placeholder - would implement file writing)
        if self.config.file_output {
            // TODO: Implement file output
        }

        Ok(())
    }

    /// Output event to console
    fn output_to_console(&self, event: &ActivityEvent) {
        let message = if self.config.json_format {
            serde_json::to_string(event).unwrap_or_else(|_| "Failed to serialize event".to_string())
        } else {
            event.description()
        };

        match event.level {
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error | LogLevel::Critical => error!("{}", message),
        }
    }
}

/// Logger statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerStats {
    /// Total number of events logged
    pub total_events: u64,
    /// Event count by log level
    pub level_counts: HashMap<LogLevel, u64>,
    /// Event count by category
    pub category_counts: HashMap<crate::events::EventCategory, u64>,
}

impl Default for ActivityLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new(10000)
    }
}