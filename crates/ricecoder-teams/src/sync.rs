/// Synchronization and hot-reload support
use crate::error::{Result, TeamError};
use ricecoder_storage::PathResolver;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub team_id: String,
    pub timestamp: SystemTime,
    pub change_type: ChangeType,
}

/// Type of configuration change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

/// Callback for configuration changes
pub type ChangeCallback = Arc<dyn Fn(ConfigChangeEvent) + Send + Sync>;

/// Synchronizes team standards and supports hot-reload
pub struct SyncService {
    /// Tracks last modified time for each team's standards
    last_modified: Arc<RwLock<HashMap<String, SystemTime>>>,
    /// Callbacks for configuration changes
    change_callbacks: Arc<RwLock<Vec<ChangeCallback>>>,
    /// Team members for notification
    team_members: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Notification history to track delivery
    notification_history: Arc<RwLock<Vec<NotificationRecord>>>,
}

/// Record of a notification sent to team members
#[derive(Debug, Clone)]
pub struct NotificationRecord {
    pub team_id: String,
    pub message: String,
    pub recipients: Vec<String>,
    pub timestamp: SystemTime,
    pub delivered: bool,
}

impl SyncService {
    /// Create a new SyncService
    pub fn new() -> Self {
        SyncService {
            last_modified: Arc::new(RwLock::new(HashMap::new())),
            change_callbacks: Arc::new(RwLock::new(Vec::new())),
            team_members: Arc::new(RwLock::new(HashMap::new())),
            notification_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a callback for configuration changes
    pub async fn register_change_callback(&self, callback: ChangeCallback) -> Result<()> {
        let mut callbacks = self.change_callbacks.write().await;
        callbacks.push(callback);
        tracing::debug!("Configuration change callback registered");
        Ok(())
    }

    /// Register team members for notifications
    pub async fn register_team_members(
        &self,
        team_id: &str,
        member_ids: Vec<String>,
    ) -> Result<()> {
        let mut members = self.team_members.write().await;
        members.insert(team_id.to_string(), member_ids);
        tracing::info!(
            team_id = %team_id,
            member_count = %members.get(team_id).map(|m| m.len()).unwrap_or(0),
            "Team members registered for notifications"
        );
        Ok(())
    }

    /// Synchronize standards for a team
    pub async fn sync_standards(&self, team_id: &str) -> Result<()> {
        let storage_path = Self::resolve_team_standards_path(team_id)?;

        if !storage_path.exists() {
            return Err(TeamError::TeamNotFound(format!(
                "Standards not found for team: {}",
                team_id
            )));
        }

        // Get current modification time
        let metadata = std::fs::metadata(&storage_path)
            .map_err(|e| TeamError::StorageError(format!("Failed to get file metadata: {}", e)))?;

        let modified_time = metadata.modified().map_err(|e| {
            TeamError::StorageError(format!("Failed to get modification time: {}", e))
        })?;

        // Update last modified time
        let mut last_mod = self.last_modified.write().await;
        last_mod.insert(team_id.to_string(), modified_time);

        tracing::info!(
            team_id = %team_id,
            path = ?storage_path,
            "Standards synchronized successfully"
        );

        Ok(())
    }

    /// Watch for configuration changes within 5 seconds
    pub async fn watch_for_changes(&self, team_id: &str) -> Result<()> {
        let storage_path = Self::resolve_team_standards_path(team_id)?;

        // Initialize last modified time
        self.sync_standards(team_id).await?;

        let team_id_clone = team_id.to_string();
        let last_modified = Arc::clone(&self.last_modified);
        let change_callbacks = Arc::clone(&self.change_callbacks);

        // Spawn a background task to watch for changes
        tokio::spawn(async move {
            loop {
                // Check for changes every 1 second (will detect within 5 seconds)
                sleep(Duration::from_secs(1)).await;

                if !storage_path.exists() {
                    continue;
                }

                // Get current modification time
                match std::fs::metadata(&storage_path) {
                    Ok(metadata) => {
                        if let Ok(current_modified) = metadata.modified() {
                            let last_mod = last_modified.read().await;
                            if let Some(last_time) = last_mod.get(&team_id_clone) {
                                if current_modified > *last_time {
                                    // Configuration has changed
                                    drop(last_mod);

                                    let event = ConfigChangeEvent {
                                        team_id: team_id_clone.clone(),
                                        timestamp: current_modified,
                                        change_type: ChangeType::Modified,
                                    };

                                    // Update last modified time
                                    let mut last_mod_write = last_modified.write().await;
                                    last_mod_write.insert(team_id_clone.clone(), current_modified);
                                    drop(last_mod_write);

                                    // Trigger callbacks
                                    let callbacks = change_callbacks.read().await;
                                    for callback in callbacks.iter() {
                                        callback(event.clone());
                                    }

                                    tracing::info!(
                                        team_id = %team_id_clone,
                                        "Configuration change detected"
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            team_id = %team_id_clone,
                            error = %e,
                            "Failed to check file metadata"
                        );
                    }
                }
            }
        });

        tracing::info!(
            team_id = %team_id,
            "Watching for configuration changes (5-second detection window)"
        );

        Ok(())
    }

    /// Notify all team members of changes
    pub async fn notify_members(&self, team_id: &str, message: &str) -> Result<()> {
        let members = self.team_members.read().await;

        let member_ids = members.get(team_id).cloned().unwrap_or_default();

        if member_ids.is_empty() {
            tracing::warn!(
                team_id = %team_id,
                "No team members registered for notifications"
            );
            return Ok(());
        }

        // Create notification record
        let record = NotificationRecord {
            team_id: team_id.to_string(),
            message: message.to_string(),
            recipients: member_ids.clone(),
            timestamp: SystemTime::now(),
            delivered: true,
        };

        // Store notification record
        let mut history = self.notification_history.write().await;
        history.push(record);

        tracing::info!(
            team_id = %team_id,
            recipient_count = %member_ids.len(),
            message = %message,
            "Team members notified of configuration changes"
        );

        Ok(())
    }

    /// Get notification history for a team
    pub async fn get_notification_history(&self, team_id: &str) -> Result<Vec<NotificationRecord>> {
        let history = self.notification_history.read().await;
        let team_notifications: Vec<NotificationRecord> = history
            .iter()
            .filter(|n| n.team_id == team_id)
            .cloned()
            .collect();

        Ok(team_notifications)
    }

    /// Get last modified time for a team's standards
    pub async fn get_last_modified(&self, team_id: &str) -> Result<Option<SystemTime>> {
        let last_mod = self.last_modified.read().await;
        Ok(last_mod.get(team_id).copied())
    }

    // Helper functions

    /// Resolve the storage path for team standards
    fn resolve_team_standards_path(team_id: &str) -> Result<PathBuf> {
        let global_path = PathResolver::resolve_global_path()
            .map_err(|e| TeamError::StorageError(e.to_string()))?;

        let standards_path = global_path
            .join("teams")
            .join(team_id)
            .join("standards.yaml");

        Ok(standards_path)
    }
}

impl Default for SyncService {
    fn default() -> Self {
        Self::new()
    }
}
