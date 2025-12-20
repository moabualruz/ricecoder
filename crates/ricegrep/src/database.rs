//! Database storage for RiceGrep enterprise features
//!
//! This module provides persistent storage for search history, user preferences,
//! and index metadata using SQLite as the backend.

use crate::error::RiceGrepError;
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// SQLite database file path
    pub database_path: PathBuf,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from(".ricecoder/ricegrep.db"),
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

/// Database manager using SQLite
pub struct DatabaseManager {
    conn: Arc<Connection>,
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// Create a new database manager
    pub fn new(config: DatabaseConfig) -> Result<Self, RiceGrepError> {
        let conn = Connection::open(&config.database_path)
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to open database: {}", e),
            })?;

        // Create tables
        Self::create_tables(&conn)?;

        Ok(Self {
            conn: Arc::new(conn),
            config,
        })
    }

    /// Create database tables
    fn create_tables(conn: &Connection) -> Result<(), RiceGrepError> {
        // Search history table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS search_history (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                query TEXT NOT NULL,
                results_count INTEGER NOT NULL,
                execution_time_ms INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                ai_used BOOLEAN NOT NULL,
                success BOOLEAN NOT NULL
            )
            "#,
            [],
        )
        .map_err(|e| RiceGrepError::Database {
            message: format!("Failed to create search_history table: {}", e),
        })?;

        // User preferences table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS user_preferences (
                user_id TEXT NOT NULL,
                setting_key TEXT NOT NULL,
                setting_value TEXT NOT NULL,
                category TEXT NOT NULL,
                last_updated INTEGER NOT NULL,
                is_ai_preference BOOLEAN NOT NULL,
                PRIMARY KEY (user_id, setting_key)
            )
            "#,
            [],
        )
        .map_err(|e| RiceGrepError::Database {
            message: format!("Failed to create user_preferences table: {}", e),
        })?;

        // Index metadata table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS index_metadata (
                index_id TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                file_count INTEGER NOT NULL,
                total_size_bytes INTEGER NOT NULL,
                last_rebuild INTEGER NOT NULL,
                rebuild_duration_ms INTEGER NOT NULL,
                performance_score REAL NOT NULL,
                status TEXT NOT NULL
            )
            "#,
            [],
        )
        .map_err(|e| RiceGrepError::Database {
            message: format!("Failed to create index_metadata table: {}", e),
        })?;

        Ok(())
    }





    /// Store search history
    pub fn store_search_history(&self, history: SearchHistory) -> Result<(), RiceGrepError> {
        self.conn
            .execute(
                r#"
                INSERT INTO search_history (
                    id, user_id, query, results_count, execution_time_ms,
                    timestamp, ai_used, success
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    history.id.to_string(),
                    history.user_id,
                    history.query,
                    history.results_count as i32,
                    history.execution_time_ms as i64,
                    history.timestamp.timestamp_millis(),
                    history.ai_used,
                    history.success,
                ],
            )
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to store search history: {}", e),
            })?;

        Ok(())
    }

    /// Retrieve search history for a user
    pub fn get_search_history(
        &self,
        user_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchHistory>, RiceGrepError> {
        let mut stmt = if let Some(user_id) = user_id {
            self.conn.prepare(
                "SELECT id, user_id, query, results_count, execution_time_ms, timestamp, ai_used, success FROM search_history WHERE user_id = ? ORDER BY timestamp DESC LIMIT ?"
            )?
        } else {
            self.conn.prepare(
                "SELECT id, user_id, query, results_count, execution_time_ms, timestamp, ai_used, success FROM search_history ORDER BY timestamp DESC LIMIT ?"
            )?
        };

        let mut results = Vec::new();
        let mut rows = if let Some(user_id) = user_id {
            stmt.query(params![user_id, limit as i64])?
        } else {
            stmt.query(params![limit as i64])?
        };

        while let Some(row) = rows.next()? {
            results.push(Self::map_search_history_row(&row)?);
        }

        Ok(results)
    }

    /// Helper function to map database rows to SearchHistory
    fn map_search_history_row(row: &rusqlite::Row) -> rusqlite::Result<SearchHistory> {
        Ok(SearchHistory {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
            user_id: row.get(1)?,
            query: row.get(2)?,
            results_count: row.get::<_, i32>(3)? as usize,
            execution_time_ms: row.get::<_, i64>(4)? as u64,
            timestamp: DateTime::from_timestamp_millis(row.get::<_, i64>(5)?).unwrap_or_else(|| Utc::now()),
            ai_used: row.get(6)?,
            success: row.get(7)?,
        })
    }

    /// Get index metadata
    pub fn get_index_metadata(&self, index_id: Uuid) -> Result<Option<IndexMetadata>, RiceGrepError> {
        let mut stmt = self.conn.prepare(
            "SELECT index_id, path, file_count, total_size_bytes, last_rebuild, rebuild_duration_ms, performance_score, status FROM index_metadata WHERE index_id = ?"
        )?;

        let result = stmt.query_row(params![index_id.to_string()], |row| {
            let status_str: String = row.get(7)?;
            let status = match status_str.as_str() {
                "active" => IndexStatus::Active,
                "rebuilding" => IndexStatus::Rebuilding,
                "corrupted" => IndexStatus::Corrupted,
                "outdated" => IndexStatus::Outdated,
                _ => IndexStatus::Corrupted,
            };

            Ok(IndexMetadata {
                index_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                path: row.get(1)?,
                file_count: row.get::<_, i32>(2)? as usize,
                total_size_bytes: row.get::<_, i64>(3)? as u64,
                last_rebuild: DateTime::from_timestamp_millis(row.get::<_, i64>(4)?).unwrap_or_else(|| Utc::now()),
                rebuild_duration_ms: row.get::<_, i64>(5)? as u64,
                performance_score: row.get(6)?,
                status,
            })
        });

        match result {
            Ok(metadata) => Ok(Some(metadata)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(RiceGrepError::Database {
                message: format!("Failed to retrieve index metadata: {}", e),
            }),
        }
    }

    /// Get all index metadata
    pub fn get_all_index_metadata(&self) -> Result<Vec<IndexMetadata>, RiceGrepError> {
        let mut stmt = self.conn.prepare(
            "SELECT index_id, path, file_count, total_size_bytes, last_rebuild, rebuild_duration_ms, performance_score, status FROM index_metadata"
        )?;

        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(7)?;
            let status = match status_str.as_str() {
                "active" => IndexStatus::Active,
                "rebuilding" => IndexStatus::Rebuilding,
                "corrupted" => IndexStatus::Corrupted,
                "outdated" => IndexStatus::Outdated,
                _ => IndexStatus::Corrupted,
            };

            Ok(IndexMetadata {
                index_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                path: row.get(1)?,
                file_count: row.get::<_, i32>(2)? as usize,
                total_size_bytes: row.get::<_, i64>(3)? as u64,
                last_rebuild: DateTime::from_timestamp_millis(row.get::<_, i64>(4)?).unwrap_or_else(|| Utc::now()),
                rebuild_duration_ms: row.get::<_, i64>(5)? as u64,
                performance_score: row.get(6)?,
                status,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Store index metadata
    pub fn store_index_metadata(&self, metadata: IndexMetadata) -> Result<(), RiceGrepError> {
        let status_str = match metadata.status {
            IndexStatus::Active => "active",
            IndexStatus::Rebuilding => "rebuilding",
            IndexStatus::Corrupted => "corrupted",
            IndexStatus::Outdated => "outdated",
        };

        self.conn
            .execute(
                r#"
                INSERT INTO index_metadata (
                    index_id, path, file_count, total_size_bytes,
                    last_rebuild, rebuild_duration_ms, performance_score, status
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    metadata.index_id.to_string(),
                    metadata.path,
                    metadata.file_count as i32,
                    metadata.total_size_bytes as i64,
                    metadata.last_rebuild.timestamp_millis(),
                    metadata.rebuild_duration_ms as i64,
                    metadata.performance_score,
                    status_str,
                ],
            )
            .map_err(|e| RiceGrepError::Database {
                message: format!("Failed to store index metadata: {}", e),
            })?;

        Ok(())
    }
}