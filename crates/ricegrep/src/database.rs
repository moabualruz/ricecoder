//! Database storage for RiceGrep enterprise features
//!
//! This module provides persistent storage for search history, user preferences,
//! and index metadata using Scylla/Cassandra as the backend.

use crate::error::RiceGrepError;
use async_trait::async_trait;
use scylla::{Session, SessionBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Scylla/Cassandra hosts
    pub hosts: Vec<String>,
    /// Keyspace name
    pub keyspace: String,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            hosts: vec!["127.0.0.1:9042".to_string()],
            keyspace: "ricegrep".to_string(),
            connection_timeout_secs: 30,
            request_timeout_secs: 30,
        }
    }
}

/// Search history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistory {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub query: String,
    pub results_count: usize,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub ai_used: bool,
    pub success: bool,
}

/// User preferences record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub user_id: String,
    pub setting_key: String,
    pub setting_value: serde_json::Value,
    pub category: String,
    pub last_updated: DateTime<Utc>,
    pub is_ai_preference: bool,
}

/// Index metadata record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub index_id: Uuid,
    pub path: String,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub last_rebuild: DateTime<Utc>,
    pub rebuild_duration_ms: u64,
    pub performance_score: f64,
    pub status: IndexStatus,
}

/// Index status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexStatus {
    Active,
    Rebuilding,
    Corrupted,
    Outdated,
}

/// Database session manager
pub struct DatabaseManager {
    session: Arc<Session>,
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// Create a new database manager
    pub async fn new(config: DatabaseConfig) -> Result<Self, RiceGrepError> {
        let session = SessionBuilder::new()
            .known_nodes(&config.hosts)
            .connection_timeout(std::time::Duration::from_secs(config.connection_timeout_secs))
            .build()
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to connect to database: {}", e),
            })?;

        // Create keyspace if it doesn't exist
        session
            .query(
                format!(
                    "CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{
                        'class': 'SimpleStrategy',
                        'replication_factor': 1
                    }}",
                    config.keyspace
                ),
                &[],
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to create keyspace: {}", e),
            })?;

        // Use the keyspace
        session
            .query(format!("USE {}", config.keyspace), &[])
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to use keyspace: {}", e),
            })?;

        // Create tables
        Self::create_tables(&session).await?;

        Ok(Self {
            session: Arc::new(session),
            config,
        })
    }

    /// Create database tables
    async fn create_tables(session: &Session) -> Result<(), RiceGrepError> {
        // Search history table
        session
            .query(
                r#"
                CREATE TABLE IF NOT EXISTS search_history (
                    id uuid PRIMARY KEY,
                    user_id text,
                    query text,
                    results_count int,
                    execution_time_ms bigint,
                    timestamp timestamp,
                    ai_used boolean,
                    success boolean
                )
                "#,
                &[],
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to create search_history table: {}", e),
            })?;

        // User preferences table
        session
            .query(
                r#"
                CREATE TABLE IF NOT EXISTS user_preferences (
                    user_id text,
                    setting_key text,
                    setting_value text,
                    category text,
                    last_updated timestamp,
                    is_ai_preference boolean,
                    PRIMARY KEY (user_id, setting_key)
                )
                "#,
                &[],
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to create user_preferences table: {}", e),
            })?;

        // Index metadata table
        session
            .query(
                r#"
                CREATE TABLE IF NOT EXISTS index_metadata (
                    index_id uuid PRIMARY KEY,
                    path text,
                    file_count int,
                    total_size_bytes bigint,
                    last_rebuild timestamp,
                    rebuild_duration_ms bigint,
                    performance_score double,
                    status text
                )
                "#,
                &[],
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to create index_metadata table: {}", e),
            })?;

        Ok(())
    }

    /// Store search history
    pub async fn store_search_history(&self, history: SearchHistory) -> Result<(), RiceGrepError> {
        self.session
            .query(
                r#"
                INSERT INTO search_history (
                    id, user_id, query, results_count, execution_time_ms,
                    timestamp, ai_used, success
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                (
                    history.id,
                    history.user_id,
                    history.query,
                    history.results_count as i32,
                    history.execution_time_ms as i64,
                    history.timestamp.timestamp_millis(),
                    history.ai_used,
                    history.success,
                ),
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to store search history: {}", e),
            })?;

        Ok(())
    }

    /// Retrieve search history for a user
    pub async fn get_search_history(
        &self,
        user_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchHistory>, RiceGrepError> {
        let rows = if let Some(user_id) = user_id {
            self.session
                .query(
                    "SELECT id, user_id, query, results_count, execution_time_ms, timestamp, ai_used, success FROM search_history WHERE user_id = ? LIMIT ?",
                    (user_id, limit as i32),
                )
        } else {
            self.session
                .query(
                    "SELECT id, user_id, query, results_count, execution_time_ms, timestamp, ai_used, success FROM search_history LIMIT ? ALLOW FILTERING",
                    (limit as i32,),
                )
        }
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to retrieve search history: {}", e),
            })?;

        let mut results = Vec::new();
        for row in rows.rows().unwrap_or_default() {
            let (id, user_id, query, results_count, execution_time_ms, timestamp_ms, ai_used, success): (Uuid, Option<String>, String, i32, i64, i64, bool, bool) = row.into_typed().map_err(|e| RiceGrepError::Database {
                message: format!("Failed to parse search history row: {}", e),
            })?;

            let timestamp = DateTime::from_timestamp_millis(timestamp_ms).ok_or_else(|| RiceGrepError::Database {
                message: "Invalid timestamp in database".to_string(),
            })?;

            results.push(SearchHistory {
                id,
                user_id,
                query,
                results_count: results_count as usize,
                execution_time_ms: execution_time_ms as u64,
                timestamp,
                ai_used,
                success,
            });
        }

        Ok(results)
    }

    /// Store user preference
    pub async fn store_user_preference(&self, pref: UserPreferences) -> Result<(), RiceGrepError> {
        self.session
            .query(
                r#"
                INSERT INTO user_preferences (
                    user_id, setting_key, setting_value, category,
                    last_updated, is_ai_preference
                ) VALUES (?, ?, ?, ?, ?, ?)
                "#,
                (
                    pref.user_id,
                    pref.setting_key,
                    serde_json::to_string(&pref.setting_value).map_err(|e| RiceGrepError::Database {
                        message: format!("Failed to serialize preference value: {}", e),
                    })?,
                    pref.category,
                    pref.last_updated,
                    pref.is_ai_preference,
                ),
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to store user preference: {}", e),
            })?;

        Ok(())
    }

    /// Get user preference
    pub async fn get_user_preference(
        &self,
        user_id: &str,
        setting_key: &str,
    ) -> Result<Option<UserPreferences>, RiceGrepError> {
        let rows = self
            .session
            .query(
                "SELECT user_id, setting_key, setting_value, category, last_updated, is_ai_preference FROM user_preferences WHERE user_id = ? AND setting_key = ?",
                (user_id, setting_key),
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to retrieve user preference: {}", e),
            })?;

        if let Some(row) = rows.first_row() {
            let (user_id, setting_key, setting_value_str, category, last_updated_ms, is_ai_preference): (String, String, String, String, i64, bool) = row.into_typed().map_err(|e| RiceGrepError::Database {
                message: format!("Failed to parse user preference row: {}", e),
            })?;

            let last_updated = DateTime::from_timestamp_millis(last_updated_ms).ok_or_else(|| RiceGrepError::Database {
                message: "Invalid timestamp in database".to_string(),
            })?;

            let setting_value: serde_json::Value = serde_json::from_str(&setting_value_str).map_err(|e| RiceGrepError::Database {
                message: format!("Failed to deserialize preference value: {}", e),
            })?;

            Ok(Some(UserPreferences {
                user_id,
                setting_key,
                setting_value,
                category,
                last_updated,
                is_ai_preference,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all user preferences
    pub async fn get_user_preferences(&self, user_id: &str) -> Result<Vec<UserPreferences>, RiceGrepError> {
        let rows = self
            .session
            .query(
                "SELECT user_id, setting_key, setting_value, category, last_updated, is_ai_preference FROM user_preferences WHERE user_id = ?",
                (user_id,),
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to retrieve user preferences: {}", e),
            })?;

        let mut results = Vec::new();
        for row in rows.rows().unwrap_or_default() {
            let (user_id, setting_key, setting_value_str, category, last_updated_ms, is_ai_preference): (String, String, String, String, i64, bool) = row.into_typed().map_err(|e| RiceGrepError::Database {
                message: format!("Failed to parse user preference row: {}", e),
            })?;

            let last_updated = DateTime::from_timestamp_millis(last_updated_ms).ok_or_else(|| RiceGrepError::Database {
                message: "Invalid timestamp in database".to_string(),
            })?;

            let setting_value: serde_json::Value = serde_json::from_str(&setting_value_str).map_err(|e| RiceGrepError::Database {
                message: format!("Failed to deserialize preference value: {}", e),
            })?;

            results.push(UserPreferences {
                user_id,
                setting_key,
                setting_value,
                category,
                last_updated,
                is_ai_preference,
            });
        }

        Ok(results)
    }

    /// Store index metadata
    pub async fn store_index_metadata(&self, metadata: IndexMetadata) -> Result<(), RiceGrepError> {
        let status_str = match metadata.status {
            IndexStatus::Active => "active",
            IndexStatus::Rebuilding => "rebuilding",
            IndexStatus::Corrupted => "corrupted",
            IndexStatus::Outdated => "outdated",
        };

        self.session
            .query(
                r#"
                INSERT INTO index_metadata (
                    index_id, path, file_count, total_size_bytes,
                    last_rebuild, rebuild_duration_ms, performance_score, status
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                (
                    metadata.index_id,
                    metadata.path,
                    metadata.file_count as i32,
                    metadata.total_size_bytes as i64,
                    metadata.last_rebuild,
                    metadata.rebuild_duration_ms as i64,
                    metadata.performance_score,
                    status_str,
                ),
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to store index metadata: {}", e),
            })?;

        Ok(())
    }

    /// Get index metadata
    pub async fn get_index_metadata(&self, index_id: Uuid) -> Result<Option<IndexMetadata>, RiceGrepError> {
        let rows = self
            .session
            .query(
                "SELECT index_id, path, file_count, total_size_bytes, last_rebuild, rebuild_duration_ms, performance_score, status FROM index_metadata WHERE index_id = ?",
                (index_id,),
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to retrieve index metadata: {}", e),
            })?;

        if let Some(row) = rows.first_row() {
            let (index_id, path, file_count, total_size_bytes, last_rebuild_ms, rebuild_duration_ms, performance_score, status_str): (Uuid, String, i32, i64, i64, i64, f64, String) = row.into_typed().map_err(|e| RiceGrepError::Database {
                message: format!("Failed to parse index metadata row: {}", e),
            })?;

            let last_rebuild = DateTime::from_timestamp_millis(last_rebuild_ms).ok_or_else(|| RiceGrepError::Database {
                message: "Invalid timestamp in database".to_string(),
            })?;

            let status = match status_str.as_str() {
                "active" => IndexStatus::Active,
                "rebuilding" => IndexStatus::Rebuilding,
                "corrupted" => IndexStatus::Corrupted,
                "outdated" => IndexStatus::Outdated,
                _ => IndexStatus::Corrupted,
            };

            Ok(Some(IndexMetadata {
                index_id,
                path,
                file_count: file_count as usize,
                total_size_bytes: total_size_bytes as u64,
                last_rebuild,
                rebuild_duration_ms: rebuild_duration_ms as u64,
                performance_score,
                status,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all index metadata
    pub async fn get_all_index_metadata(&self) -> Result<Vec<IndexMetadata>, RiceGrepError> {
        let rows = self
            .session
            .query(
                "SELECT index_id, path, file_count, total_size_bytes, last_rebuild, rebuild_duration_ms, performance_score, status FROM index_metadata",
                &[],
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to retrieve index metadata: {}", e),
            })?;

        let mut results = Vec::new();
        for row in rows.rows().unwrap_or_default() {
            let (index_id, path, file_count, total_size_bytes, last_rebuild_ms, rebuild_duration_ms, performance_score, status_str): (Uuid, String, i32, i64, i64, i64, f64, String) = row.into_typed().map_err(|e| RiceGrepError::Database {
                message: format!("Failed to parse index metadata row: {}", e),
            })?;

            let last_rebuild = DateTime::from_timestamp_millis(last_rebuild_ms).ok_or_else(|| RiceGrepError::Database {
                message: "Invalid timestamp in database".to_string(),
            })?;

            let status = match status_str.as_str() {
                "active" => IndexStatus::Active,
                "rebuilding" => IndexStatus::Rebuilding,
                "corrupted" => IndexStatus::Corrupted,
                "outdated" => IndexStatus::Outdated,
                _ => IndexStatus::Corrupted,
            };

            results.push(IndexMetadata {
                index_id,
                path,
                file_count: file_count as usize,
                total_size_bytes: total_size_bytes as u64,
                last_rebuild,
                rebuild_duration_ms: rebuild_duration_ms as u64,
                performance_score,
                status,
            });
        }

        Ok(results)
    }
}

/// Migration manager for database schema updates
pub struct MigrationManager {
    session: Arc<Session>,
    current_version: i32,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session,
            current_version: 1,
        }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self) -> Result<(), RiceGrepError> {
        // Create migrations table if it doesn't exist
        self.session
            .query(
                r#"
                CREATE TABLE IF NOT EXISTS schema_migrations (
                    version int PRIMARY KEY,
                    description text,
                    applied_at timestamp
                )
                "#,
                &[],
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to create migrations table: {}", e),
            })?;

        // Get current version
        let current_version = self.get_current_version().await?;

        // Run migrations
        if current_version < 1 {
            self.run_migration_1().await?;
        }

        // Future migrations can be added here

        Ok(())
    }

    /// Get current schema version
    async fn get_current_version(&self) -> Result<i32, RiceGrepError> {
        let rows = self
            .session
            .query("SELECT version FROM schema_migrations LIMIT 1000 ALLOW FILTERING", &[])
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to get current version: {}", e),
            })?;

        let mut max_version = 0;
        for row in rows.rows().unwrap_or_default() {
            let version: i32 = row.into_typed().map_err(|e| RiceGrepError::Database {
                message: format!("Failed to parse version: {}", e),
            })?;
            if version > max_version {
                max_version = version;
            }
        }

        Ok(max_version)
    }

    /// Migration 1: Initial schema
    async fn run_migration_1(&self) -> Result<(), RiceGrepError> {
        // Mark migration as applied
        self.session
            .query(
                "INSERT INTO schema_migrations (version, description, applied_at) VALUES (?, ?, ?)",
                (1, "Initial schema", Utc::now().timestamp_millis()),
            )
            .await
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to record migration 1: {}", e),
            })?;

        Ok(())
    }
}