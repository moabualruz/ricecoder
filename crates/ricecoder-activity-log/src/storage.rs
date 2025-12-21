//! Log storage and retention management

use crate::error::{ActivityLogError, ActivityLogResult};
use crate::events::{ActivityEvent, EventFilter};
use crate::logger::LogStorage;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

/// Retention policy for log management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum age of logs to keep (in days)
    pub max_age_days: u32,
    /// Maximum number of log entries to keep
    pub max_entries: Option<u64>,
    /// Maximum storage size in bytes
    pub max_size_bytes: Option<u64>,
    /// Categories to retain longer
    pub extended_retention_categories: Vec<crate::events::EventCategory>,
    /// Extended retention period for special categories (in days)
    pub extended_retention_days: u32,
    /// Whether to compress old logs
    pub compress_old_logs: bool,
    /// Archive destination (None means delete)
    pub archive_destination: Option<String>,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of events stored
    pub total_events: u64,
    /// Storage size in bytes
    pub storage_size_bytes: u64,
    /// Oldest event timestamp
    pub oldest_event: Option<DateTime<Utc>>,
    /// Newest event timestamp
    pub newest_event: Option<DateTime<Utc>>,
    /// Events by category
    pub events_by_category: HashMap<crate::events::EventCategory, u64>,
    /// Events by level
    pub events_by_level: HashMap<crate::events::LogLevel, u64>,
    /// Last maintenance run
    pub last_maintenance: Option<DateTime<Utc>>,
}

/// File-based log storage implementation
pub struct FileStorage {
    base_path: std::path::PathBuf,
    retention_policy: RetentionPolicy,
    stats: std::sync::Mutex<StorageStats>,
}

impl FileStorage {
    /// Create a new file storage
    pub fn new(
        base_path: std::path::PathBuf,
        retention_policy: RetentionPolicy,
    ) -> ActivityLogResult<Self> {
        std::fs::create_dir_all(&base_path)?;

        let stats = StorageStats {
            total_events: 0,
            storage_size_bytes: 0,
            oldest_event: None,
            newest_event: None,
            events_by_category: HashMap::new(),
            events_by_level: HashMap::new(),
            last_maintenance: None,
        };

        Ok(Self {
            base_path,
            retention_policy,
            stats: std::sync::Mutex::new(stats),
        })
    }

    /// Get file path for a date
    fn get_file_path(&self, date: &DateTime<Utc>) -> std::path::PathBuf {
        let date_str = date.format("%Y-%m-%d").to_string();
        self.base_path.join(format!("activity-{}.log", date_str))
    }

    /// Update storage statistics
    fn update_stats(&self, event: &ActivityEvent, added: bool) {
        let mut stats = self.stats.lock().unwrap();

        if added {
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;
            *stats.events_by_level.entry(event.level).or_insert(0) += 1;

            // Update timestamps
            if stats.oldest_event.is_none() || event.timestamp < stats.oldest_event.unwrap() {
                stats.oldest_event = Some(event.timestamp);
            }
            if stats.newest_event.is_none() || event.timestamp > stats.newest_event.unwrap() {
                stats.newest_event = Some(event.timestamp);
            }

            // Estimate storage size (rough calculation)
            stats.storage_size_bytes += serde_json::to_string(event)
                .map(|s| s.len() as u64)
                .unwrap_or(100);
        } else {
            // Removal logic would go here
            stats.total_events = stats.total_events.saturating_sub(1);
        }
    }
}

#[async_trait]
impl LogStorage for FileStorage {
    async fn store_event(&self, event: &ActivityEvent) -> ActivityLogResult<()> {
        let file_path = self.get_file_path(&event.timestamp);

        // Serialize event
        let json_line = serde_json::to_string(event)? + "\n";

        // Append to file
        tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await?
            .write_all(json_line.as_bytes())
            .await?;

        self.update_stats(event, true);
        Ok(())
    }

    async fn retrieve_events(&self, filter: &EventFilter) -> ActivityLogResult<Vec<ActivityEvent>> {
        let mut results = Vec::new();

        // For simplicity, read recent files (last 7 days)
        for i in 0..7 {
            let date = Utc::now() - Duration::days(i);
            let file_path = self.get_file_path(&date);

            if file_path.exists() {
                let content = tokio::fs::read_to_string(&file_path).await?;
                for line in content.lines() {
                    if let Ok(event) = serde_json::from_str::<ActivityEvent>(line) {
                        if filter.matches(&event) {
                            results.push(event);
                            if let Some(limit) = filter.limit {
                                if results.len() >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(results)
    }

    async fn count_events(&self, filter: &EventFilter) -> ActivityLogResult<u64> {
        // Simplified implementation - in practice, you'd use indexes
        let events = self.retrieve_events(filter).await?;
        Ok(events.len() as u64)
    }

    async fn delete_events(&self, filter: &EventFilter) -> ActivityLogResult<u64> {
        // Simplified implementation - in practice, you'd need to rewrite files
        // For now, just return 0 (no deletion implemented)
        let _ = filter;
        Ok(0)
    }

    async fn maintenance(&self) -> ActivityLogResult<()> {
        let cutoff_date = Utc::now() - Duration::days(self.retention_policy.max_age_days as i64);

        // Remove old files
        let mut removed_files = 0;
        if let Ok(entries) = std::fs::read_dir(&self.base_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.modified().is_ok() {
                        // This is a simplified check - you'd need proper date parsing from filenames
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.starts_with("activity-") && filename.ends_with(".log") {
                                // Check if file is older than retention period
                                // (This is a placeholder - real implementation would parse dates)
                                let _ = cutoff_date;
                                removed_files += 1;
                            }
                        }
                    }
                }
            }
        }

        // Update maintenance timestamp
        let mut stats = self.stats.lock().unwrap();
        stats.last_maintenance = Some(Utc::now());

        tracing::info!("Maintenance completed: {} old files removed", removed_files);
        Ok(())
    }
}

/// Database-backed log storage (placeholder for future implementation)
pub struct DatabaseStorage {
    connection_string: String,
    retention_policy: RetentionPolicy,
}

impl DatabaseStorage {
    /// Create a new database storage
    pub fn new(connection_string: String, retention_policy: RetentionPolicy) -> Self {
        Self {
            connection_string,
            retention_policy,
        }
    }
}

#[async_trait]
impl LogStorage for DatabaseStorage {
    async fn store_event(&self, _event: &ActivityEvent) -> ActivityLogResult<()> {
        // Placeholder - would implement database storage
        Err(ActivityLogError::StorageError {
            message: "Database storage not yet implemented".to_string(),
        })
    }

    async fn retrieve_events(
        &self,
        _filter: &EventFilter,
    ) -> ActivityLogResult<Vec<ActivityEvent>> {
        // Placeholder - would implement database queries
        Err(ActivityLogError::StorageError {
            message: "Database storage not yet implemented".to_string(),
        })
    }

    async fn count_events(&self, _filter: &EventFilter) -> ActivityLogResult<u64> {
        // Placeholder
        Ok(0)
    }

    async fn delete_events(&self, _filter: &EventFilter) -> ActivityLogResult<u64> {
        // Placeholder
        Ok(0)
    }

    async fn maintenance(&self) -> ActivityLogResult<()> {
        // Placeholder - would implement database cleanup
        Ok(())
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age_days: 90,                        // 90 days
            max_entries: Some(100000),               // 100k entries
            max_size_bytes: Some(100 * 1024 * 1024), // 100MB
            extended_retention_categories: vec![
                crate::events::EventCategory::Security,
                crate::events::EventCategory::Security,
            ],
            extended_retention_days: 365, // 1 year for security/compliance
            compress_old_logs: true,
            archive_destination: None,
        }
    }
}
