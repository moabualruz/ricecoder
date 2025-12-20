//! Local metadata storage for RiceGrep
//!
//! This module provides file-based storage for user preferences,
//! search history, and other metadata that should persist locally
//! even when database features are disabled.

use crate::error::RiceGrepError;
use crate::database::{SearchHistory, UserPreferences, IndexMetadata, IndexStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Local metadata storage manager
pub struct LocalMetadataStore {
    /// Base directory for metadata storage
    metadata_dir: PathBuf,
    /// In-memory cache for faster access
    cache: HashMap<String, serde_json::Value>,
}

/// User preferences stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalUserPreferences {
    pub settings: HashMap<String, serde_json::Value>,
    pub ai_preferences: HashMap<String, serde_json::Value>,
    pub last_updated: DateTime<Utc>,
}

/// Local search history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSearchHistory {
    pub entries: Vec<LocalSearchEntry>,
    pub max_entries: usize,
}

/// Individual search history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSearchEntry {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub results_count: usize,
    pub execution_time_ms: u64,
    pub ai_used: bool,
    pub success: bool,
}

/// Local index metadata cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalIndexCache {
    pub indices: HashMap<String, LocalIndexInfo>,
}

/// Local index information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalIndexInfo {
    pub path: String,
    pub last_modified: DateTime<Utc>,
    pub file_count: usize,
    pub total_size: u64,
    pub checksum: String,
}

impl LocalMetadataStore {
    /// Create a new local metadata store
    pub fn new() -> Result<Self, RiceGrepError> {
        let metadata_dir = Self::get_metadata_dir()?;
        Self::ensure_metadata_dir(&metadata_dir)?;

        Ok(Self {
            metadata_dir,
            cache: HashMap::new(),
        })
    }

    /// Get the metadata directory path
    fn get_metadata_dir() -> Result<PathBuf, RiceGrepError> {
        let home = dirs::home_dir()
            .ok_or_else(|| RiceGrepError::ConfigError("Could not find home directory".to_string()))?;

        Ok(home.join(".ricecoder").join(".ricegrep").join("metadata"))
    }

    /// Ensure metadata directory exists
    fn ensure_metadata_dir(dir: &Path) -> Result<(), RiceGrepError> {
        if !dir.exists() {
            fs::create_dir_all(dir)
                .map_err(|e| RiceGrepError::Io {
                    message: format!("Failed to create metadata directory: {}", e),
                })?;
        }
        Ok(())
    }

    /// Get file path for a metadata type
    fn get_metadata_file(&self, name: &str) -> PathBuf {
        self.metadata_dir.join(format!("{}.json", name))
    }

    /// Load user preferences from local storage
    pub fn load_user_preferences(&self) -> Result<LocalUserPreferences, RiceGrepError> {
        let file_path = self.get_metadata_file("user_preferences");

        if !file_path.exists() {
            return Ok(LocalUserPreferences {
                settings: HashMap::new(),
                ai_preferences: HashMap::new(),
                last_updated: Utc::now(),
            });
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to read user preferences: {}", e),
            })?;

        serde_json::from_str(&content)
            .map_err(|e| RiceGrepError::ConfigError {
                message: format!("Failed to parse user preferences: {}", e),
            })
    }

    /// Save user preferences to local storage
    pub fn save_user_preferences(&self, prefs: &LocalUserPreferences) -> Result<(), RiceGrepError> {
        let file_path = self.get_metadata_file("user_preferences");
        let content = serde_json::to_string_pretty(prefs)
            .map_err(|e| RiceGrepError::ConfigError {
                message: format!("Failed to serialize user preferences: {}", e),
            })?;

        fs::write(&file_path, content)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to write user preferences: {}", e),
            })
    }

    /// Load search history from local storage
    pub fn load_search_history(&self) -> Result<LocalSearchHistory, RiceGrepError> {
        let file_path = self.get_metadata_file("search_history");

        if !file_path.exists() {
            return Ok(LocalSearchHistory {
                entries: Vec::new(),
                max_entries: 1000,
            });
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to read search history: {}", e),
            })?;

        serde_json::from_str(&content)
            .map_err(|e| RiceGrepError::ConfigError {
                message: format!("Failed to parse search history: {}", e),
            })
    }

    /// Save search history to local storage
    pub fn save_search_history(&self, history: &LocalSearchHistory) -> Result<(), RiceGrepError> {
        let file_path = self.get_metadata_file("search_history");
        let content = serde_json::to_string_pretty(history)
            .map_err(|e| RiceGrepError::ConfigError {
                message: format!("Failed to serialize search history: {}", e),
            })?;

        fs::write(&file_path, content)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to write search history: {}", e),
            })
    }

    /// Add a search entry to history
    pub fn add_search_entry(&self, query: &str, results_count: usize, execution_time_ms: u64, ai_used: bool, success: bool) -> Result<(), RiceGrepError> {
        let mut history = self.load_search_history()?;

        // Remove duplicate entries
        history.entries.retain(|entry| entry.query != query);

        // Add new entry
        let entry = LocalSearchEntry {
            query: query.to_string(),
            timestamp: Utc::now(),
            results_count,
            execution_time_ms,
            ai_used,
            success,
        };

        history.entries.push(entry);

        // Keep only the most recent entries
        if history.entries.len() > history.max_entries {
            history.entries = history.entries
                .into_iter()
                .rev()
                .take(history.max_entries)
                .rev()
                .collect();
        }

        self.save_search_history(&history)
    }

    /// Get recent search queries for autocomplete
    pub fn get_recent_queries(&self, limit: usize) -> Result<Vec<String>, RiceGrepError> {
        let history = self.load_search_history()?;
        Ok(history.entries
            .iter()
            .rev()
            .take(limit)
            .map(|entry| entry.query.clone())
            .collect())
    }

    /// Load index cache from local storage
    pub fn load_index_cache(&self) -> Result<LocalIndexCache, RiceGrepError> {
        let file_path = self.get_metadata_file("index_cache");

        if !file_path.exists() {
            return Ok(LocalIndexCache {
                indices: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to read index cache: {}", e),
            })?;

        serde_json::from_str(&content)
            .map_err(|e| RiceGrepError::ConfigError {
                message: format!("Failed to parse index cache: {}", e),
            })
    }

    /// Save index cache to local storage
    pub fn save_index_cache(&self, cache: &LocalIndexCache) -> Result<(), RiceGrepError> {
        let file_path = self.get_metadata_file("index_cache");
        let content = serde_json::to_string_pretty(cache)
            .map_err(|e| RiceGrepError::ConfigError {
                message: format!("Failed to serialize index cache: {}", e),
            })?;

        fs::write(&file_path, content)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to write index cache: {}", e),
            })
    }

    /// Update index info in cache
    pub fn update_index_info(&self, path: &str, file_count: usize, total_size: u64, checksum: &str) -> Result<(), RiceGrepError> {
        let mut cache = self.load_index_cache()?;

        let info = LocalIndexInfo {
            path: path.to_string(),
            last_modified: Utc::now(),
            file_count,
            total_size,
            checksum: checksum.to_string(),
        };

        cache.indices.insert(path.to_string(), info);
        self.save_index_cache(&cache)
    }

    /// Get index info from cache
    pub fn get_index_info(&self, path: &str) -> Result<Option<LocalIndexInfo>, RiceGrepError> {
        let cache = self.load_index_cache()?;
        Ok(cache.indices.get(path).cloned())
    }

    /// Clear all local metadata
    pub fn clear_all(&self) -> Result<(), RiceGrepError> {
        let entries = fs::read_dir(&self.metadata_dir)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to read metadata directory: {}", e),
            })?;

        for entry in entries {
            let entry = entry.map_err(|e| RiceGrepError::Io {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                fs::remove_file(entry.path())
                    .map_err(|e| RiceGrepError::Io {
                        message: format!("Failed to remove metadata file: {}", e),
                    })?;
            }
        }

        self.cache.clear();
        Ok(())
    }

    /// Get metadata directory size
    pub fn get_storage_size(&self) -> Result<u64, RiceGrepError> {
        let mut total_size = 0u64;

        let entries = fs::read_dir(&self.metadata_dir)
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to read metadata directory: {}", e),
            })?;

        for entry in entries {
            let entry = entry.map_err(|e| RiceGrepError::Io {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }
}

/// Hybrid metadata manager (local + optional database)
pub struct HybridMetadataManager {
    local_store: LocalMetadataStore,
    database_manager: Option<Arc<crate::database::DatabaseManager>>,
}

impl HybridMetadataManager {
    /// Create a new hybrid metadata manager
    pub fn new(database_manager: Option<Arc<crate::database::DatabaseManager>>) -> Result<Self, RiceGrepError> {
        let local_store = LocalMetadataStore::new()?;

        Ok(Self {
            local_store,
            database_manager,
        })
    }

    /// Store search history (local + optional database)
    pub async fn store_search_history(&self, query: &str, results_count: usize, execution_time_ms: u64, ai_used: bool, success: bool) -> Result<(), RiceGrepError> {
        // Always store locally for immediate access
        self.local_store.add_search_entry(query, results_count, execution_time_ms, ai_used, success)?;

        // Also store in database if available
        if let Some(db) = &self.database_manager {
            let search_history = SearchHistory {
                id: Uuid::new_v4(),
                user_id: std::env::var("RICEGREP_USER_ID").ok(),
                query: query.to_string(),
                results_count,
                execution_time_ms,
                timestamp: Utc::now(),
                ai_used,
                success,
            };

            if let Err(e) = db.store_search_history(search_history).await {
                eprintln!("Warning: Failed to store search history in database: {}", e);
                // Don't fail the operation, just log the warning
            }
        }

        Ok(())
    }

    /// Get recent search queries (from local storage)
    pub fn get_recent_queries(&self, limit: usize) -> Result<Vec<String>, RiceGrepError> {
        self.local_store.get_recent_queries(limit)
    }

    /// Store user preference (local + optional database)
    pub async fn store_user_preference(&self, key: &str, value: serde_json::Value, is_ai_pref: bool) -> Result<(), RiceGrepError> {
        // Load current preferences
        let mut prefs = self.local_store.load_user_preferences()?;

        // Update the preference
        if is_ai_pref {
            prefs.ai_preferences.insert(key.to_string(), value.clone());
        } else {
            prefs.settings.insert(key.to_string(), value.clone());
        }
        prefs.last_updated = Utc::now();

        // Save locally
        self.local_store.save_user_preferences(&prefs)?;

        // Also store in database if available
        if let Some(db) = &self.database_manager {
            let db_pref = UserPreferences {
                user_id: "local".to_string(), // Could be enhanced with actual user ID
                setting_key: key.to_string(),
                setting_value: value,
                category: if is_ai_pref { "ai" } else { "general" }.to_string(),
                last_updated: prefs.last_updated,
                is_ai_preference: is_ai_pref,
            };

            if let Err(e) = db.store_user_preference(db_pref).await {
                eprintln!("Warning: Failed to store user preference in database: {}", e);
            }
        }

        Ok(())
    }

    /// Get user preference (try local first, then database)
    pub async fn get_user_preference(&self, key: &str) -> Result<Option<serde_json::Value>, RiceGrepError> {
        // Try local storage first
        let prefs = self.local_store.load_user_preferences()?;
        if let Some(value) = prefs.settings.get(key) {
            return Ok(Some(value.clone()));
        }
        if let Some(value) = prefs.ai_preferences.get(key) {
            return Ok(Some(value.clone()));
        }

        // Try database if available
        if let Some(db) = &self.database_manager {
            if let Ok(Some(db_pref)) = db.get_user_preference("local", key).await {
                return Ok(Some(db_pref.setting_value));
            }
        }

        Ok(None)
    }

    /// Sync local data to database (if database is available)
    pub async fn sync_to_database(&self) -> Result<(), RiceGrepError> {
        if let Some(db) = &self.database_manager {
            // Sync user preferences
            let prefs = self.local_store.load_user_preferences()?;
            for (key, value) in &prefs.settings {
                let db_pref = UserPreferences {
                    user_id: "local".to_string(),
                    setting_key: key.clone(),
                    setting_value: value.clone(),
                    category: "general".to_string(),
                    last_updated: prefs.last_updated,
                    is_ai_preference: false,
                };
                if let Err(e) = db.store_user_preference(db_pref).await {
                    eprintln!("Warning: Failed to sync preference {}: {}", key, e);
                }
            }

            // Sync AI preferences
            for (key, value) in &prefs.ai_preferences {
                let db_pref = UserPreferences {
                    user_id: "local".to_string(),
                    setting_key: key.clone(),
                    setting_value: value.clone(),
                    category: "ai".to_string(),
                    last_updated: prefs.last_updated,
                    is_ai_preference: true,
                };
                if let Err(e) = db.store_user_preference(db_pref).await {
                    eprintln!("Warning: Failed to sync AI preference {}: {}", key, e);
                }
            }

            // Sync recent search history
            let history = self.local_store.load_search_history()?;
            for entry in &history.entries {
                let search_history = SearchHistory {
                    id: Uuid::new_v4(),
                    user_id: std::env::var("RICEGREP_USER_ID").ok(),
                    query: entry.query.clone(),
                    results_count: entry.results_count,
                    execution_time_ms: entry.execution_time_ms,
                    timestamp: entry.timestamp,
                    ai_used: entry.ai_used,
                    success: entry.success,
                };
                if let Err(e) = db.store_search_history(search_history).await {
                    eprintln!("Warning: Failed to sync search history: {}", e);
                }
            }
        }

        Ok(())
    }
}