//! Session persistence to disk with encryption support

use crate::error::{SessionError, SessionResult};
use crate::models::Session;
use base64::engine::general_purpose;
use chrono::{DateTime, Utc};
use ricecoder_security::encryption::{CustomerKeyManager, EncryptedData, KeyManager};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};

/// Manages session persistence to disk with optional encryption
#[derive(Debug, Clone)]
pub struct SessionStore {
    /// Base directory for storing sessions
    sessions_dir: PathBuf,
    /// Archive directory for deleted sessions
    archive_dir: PathBuf,
    /// Optional encryption key manager for enterprise security
    key_manager: Option<Arc<KeyManager>>,
    /// Optional customer key manager for SOC 2 compliance
    customer_key_manager: Option<Arc<CustomerKeyManager>>,
}

impl SessionStore {
    /// Create a new session store with default directories
    pub fn new() -> SessionResult<Self> {
        let sessions_dir = Self::get_sessions_dir()?;
        let archive_dir = Self::get_archive_dir()?;

        // Ensure directories exist
        fs::create_dir_all(&sessions_dir)?;
        fs::create_dir_all(&archive_dir)?;

        debug!(
            "SessionStore initialized with sessions_dir: {:?}",
            sessions_dir
        );

        Ok(Self {
            sessions_dir,
            archive_dir,
            key_manager: None,
            customer_key_manager: None,
        })
    }

    /// Create a session store with custom directories (for testing)
    pub fn with_dirs(sessions_dir: PathBuf, archive_dir: PathBuf) -> SessionResult<Self> {
        fs::create_dir_all(&sessions_dir)?;
        fs::create_dir_all(&archive_dir)?;

        Ok(Self {
            sessions_dir,
            archive_dir,
            key_manager: None,
            customer_key_manager: None,
        })
    }

    /// Create a session store with encryption enabled
    pub fn with_encryption(master_password: &str) -> SessionResult<Self> {
        let mut store = Self::new()?;
        let key_manager = Arc::new(KeyManager::new(master_password)?);
        store.key_manager = Some(key_manager);
        Ok(store)
    }

    /// Create a session store with enterprise encryption
    pub fn with_enterprise_encryption(master_password: &str) -> SessionResult<Self> {
        let mut store = Self::new()?;
        let key_manager = Arc::new(KeyManager::new(master_password)?);
        let customer_key_manager = Arc::new(CustomerKeyManager::new(master_password)?);
        store.key_manager = Some(key_manager);
        store.customer_key_manager = Some(customer_key_manager);
        Ok(store)
    }

    /// Enable encryption on an existing store
    pub fn enable_encryption(&mut self, master_password: &str) -> SessionResult<()> {
        let key_manager = Arc::new(KeyManager::new(master_password)?);
        self.key_manager = Some(key_manager);
        Ok(())
    }

    /// Enable enterprise encryption on an existing store
    pub fn enable_enterprise_encryption(&mut self, master_password: &str) -> SessionResult<()> {
        let key_manager = Arc::new(KeyManager::new(master_password)?);
        let customer_key_manager = Arc::new(CustomerKeyManager::new(master_password)?);
        self.key_manager = Some(key_manager);
        self.customer_key_manager = Some(customer_key_manager);
        Ok(())
    }

    /// Get the default sessions directory (~/.ricecoder/sessions/)
    fn get_sessions_dir() -> SessionResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            SessionError::ConfigError("Could not determine home directory".to_string())
        })?;
        Ok(home.join(".ricecoder").join("sessions"))
    }

    /// Get the default archive directory (~/.ricecoder/sessions/archive/)
    fn get_archive_dir() -> SessionResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            SessionError::ConfigError("Could not determine home directory".to_string())
        })?;
        Ok(home.join(".ricecoder").join("sessions").join("archive"))
    }

    /// Get the path for a session file
    fn session_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", session_id))
    }

    /// Get the path for an archived session file
    fn archive_path(&self, session_id: &str) -> PathBuf {
        self.archive_dir.join(format!("{}.json", session_id))
    }

    /// Save a session to disk with optional encryption
    pub async fn save(&self, session: &Session) -> SessionResult<()> {
        let path = self.session_path(&session.id);

        // Serialize session to JSON
        let json_data = serde_json::to_string_pretty(session)?;

        // Encrypt if encryption is enabled
        let data_to_write = if let Some(ref key_manager) = self.key_manager {
            if let Some(ref customer_key_manager) = self.customer_key_manager {
                // Use enterprise encryption with customer-managed keys
                let customer_key = customer_key_manager.generate_customer_key()?;
                let encrypted = customer_key_manager.encrypt_with_customer_key(&json_data, &customer_key)?;

                // Store customer key encrypted with master key
                let encrypted_key = key_manager.encrypt_api_key(&general_purpose::STANDARD.encode(&customer_key))?;

                // Combine encrypted data and encrypted key
                let combined = serde_json::json!({
                    "encrypted_session": encrypted,
                    "encrypted_key": encrypted_key
                });
                serde_json::to_string_pretty(&combined)?
            } else {
                // Use standard encryption
                let encrypted = key_manager.encrypt_api_key(&json_data)?;
                serde_json::to_string_pretty(&encrypted)?
            }
        } else {
            json_data
        };

        // Write to file
        fs::write(&path, data_to_write)?;

        info!("Session saved: {} at {:?}", session.id, path);

        Ok(())
    }

    /// Load a session from disk with optional decryption
    pub async fn load(&self, session_id: &str) -> SessionResult<Session> {
        let path = self.session_path(session_id);

        if !path.exists() {
            return Err(SessionError::NotFound(format!(
                "Session file not found: {}",
                session_id
            )));
        }

        // Read file
        let file_data = fs::read_to_string(&path)?;

        // Decrypt if encryption is enabled
        let json_data = if let Some(ref key_manager) = self.key_manager {
            if let Some(ref customer_key_manager) = self.customer_key_manager {
                // Handle enterprise encryption
                let combined: serde_json::Value = serde_json::from_str(&file_data)?;
                let encrypted_session: EncryptedData = serde_json::from_value(combined["encrypted_session"].clone())?;
                let encrypted_key: EncryptedData = serde_json::from_value(combined["encrypted_key"].clone())?;

                // Decrypt customer key
                let customer_key_b64 = key_manager.decrypt_api_key(&encrypted_key)?;
                let customer_key = general_purpose::STANDARD.decode(&customer_key_b64)?;

                // Decrypt session data
                customer_key_manager.decrypt_with_customer_key(&encrypted_session, &customer_key)?
            } else {
                // Handle standard encryption
                let encrypted: EncryptedData = serde_json::from_str(&file_data)?;
                key_manager.decrypt_api_key(&encrypted)?
            }
        } else {
            file_data
        };

        // Deserialize from JSON
        let session: Session = serde_json::from_str(&json_data)?;

        debug!("Session loaded: {} from {:?}", session_id, path);

        Ok(session)
    }

    /// List all persisted sessions
    pub async fn list(&self) -> SessionResult<Vec<Session>> {
        let mut sessions = Vec::new();

        // Read all JSON files in sessions directory
        let entries = fs::read_dir(&self.sessions_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip directories and non-JSON files
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Try to load the session
            match fs::read_to_string(&path) {
                Ok(json_data) => match serde_json::from_str::<Session>(&json_data) {
                    Ok(session) => sessions.push(session),
                    Err(e) => {
                        error!("Failed to deserialize session from {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    error!("Failed to read session file {:?}: {}", path, e);
                }
            }
        }

        debug!("Listed {} sessions", sessions.len());

        Ok(sessions)
    }

    /// Export a session to a user-specified file (always in plain text)
    pub async fn export(&self, session_id: &str, export_path: &Path) -> SessionResult<()> {
        // Load the session (decryption happens in load)
        let session = self.load(session_id).await?;

        // Serialize to JSON (plain text for export)
        let json_data = serde_json::to_string_pretty(&session)?;

        // Write to export path
        fs::write(export_path, json_data)?;

        info!("Session exported: {} to {:?}", session_id, export_path);

        Ok(())
    }

    /// Delete a session and archive it
    pub async fn delete(&self, session_id: &str) -> SessionResult<()> {
        let session_path = self.session_path(session_id);
        let archive_path = self.archive_path(session_id);

        if !session_path.exists() {
            return Err(SessionError::NotFound(format!(
                "Session not found: {}",
                session_id
            )));
        }

        // Read the session file
        let json_data = fs::read_to_string(&session_path)?;

        // Write to archive
        fs::write(&archive_path, json_data)?;

        // Delete the original
        fs::remove_file(&session_path)?;

        info!("Session deleted and archived: {}", session_id);

        Ok(())
    }

    /// Check if a session exists
    pub fn exists(&self, session_id: &str) -> bool {
        self.session_path(session_id).exists()
    }

    /// Get the sessions directory path
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }

    /// Get the archive directory path
    pub fn archive_dir(&self) -> &Path {
        &self.archive_dir
    }

    /// Clean up old sessions based on age
    pub async fn cleanup_old_sessions(&self, max_age: Duration) -> SessionResult<usize> {
        let mut cleaned_count = 0;
        let cutoff_time = Utc::now() - chrono::Duration::from_std(max_age).unwrap();

        // Get all session files
        let entries = fs::read_dir(&self.sessions_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                // Check file modification time
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(modified_datetime) = DateTime::<Utc>::try_from(modified) {
                            if modified_datetime < cutoff_time {
                                if let Some(file_name) = path.file_stem() {
                                    if let Some(session_id) = file_name.to_str() {
                                        match self.delete(session_id).await {
                                            Ok(_) => {
                                                cleaned_count += 1;
                                                debug!("Cleaned up old session: {}", session_id);
                                            }
                                            Err(e) => {
                                                warn!("Failed to clean up session {}: {}", session_id, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Cleaned up {} old sessions", cleaned_count);
        Ok(cleaned_count)
    }

    /// Clean up sessions that haven't been accessed recently
    pub async fn cleanup_stale_sessions(&self, stale_threshold: Duration) -> SessionResult<usize> {
        let mut cleaned_count = 0;
        let cutoff_time = Utc::now() - chrono::Duration::from_std(stale_threshold).unwrap();

        // Get all session files
        let entries = fs::read_dir(&self.sessions_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                // Load session and check last activity
                if let Some(file_name) = path.file_stem() {
                    if let Some(session_id) = file_name.to_str() {
                        match self.load(session_id).await {
                            Ok(session) => {
                                // Check if session has been inactive too long
                                if session.updated_at < cutoff_time {
                                    match self.delete(session_id).await {
                                        Ok(_) => {
                                            cleaned_count += 1;
                                            debug!("Cleaned up stale session: {}", session_id);
                                        }
                                        Err(e) => {
                                            warn!("Failed to clean up stale session {}: {}", session_id, e);
                                        }
                                    }
                                }
                            }
                            Err(SessionError::NotFound(_)) => {
                                // Session file doesn't exist, which is fine for cleanup
                            }
                            Err(e) => {
                                warn!("Error loading session {} for cleanup: {}", session_id, e);
                                // Try to remove potentially corrupted file
                                if fs::remove_file(&path).is_ok() {
                                    cleaned_count += 1;
                                    debug!("Cleaned up corrupted session file: {}", session_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Cleaned up {} stale sessions", cleaned_count);
        Ok(cleaned_count)
    }

    /// Perform comprehensive garbage collection
    pub async fn garbage_collect(&self, config: &GarbageCollectionConfig) -> SessionResult<GarbageCollectionResult> {
        let mut result = GarbageCollectionResult::default();

        // Clean up old sessions
        if let Some(max_age) = config.max_session_age {
            result.old_sessions_cleaned = self.cleanup_old_sessions(max_age).await?;
        }

        // Clean up stale sessions
        if let Some(stale_threshold) = config.stale_session_threshold {
            result.stale_sessions_cleaned = self.cleanup_stale_sessions(stale_threshold).await?;
        }

        // Clean up archive files
        if let Some(archive_max_age) = config.max_archive_age {
            result.archive_files_cleaned = self.cleanup_old_archive_files(archive_max_age)?;
        }

        // Check current storage usage
        result.current_session_count = self.count_sessions()?;
        result.current_archive_count = self.count_archive_files()?;
        result.total_size_bytes = self.calculate_total_size()?;

        info!("Garbage collection completed: {:?}", result);
        Ok(result)
    }

    /// Clean up old archive files
    fn cleanup_old_archive_files(&self, max_age: Duration) -> SessionResult<usize> {
        let mut cleaned_count = 0;
        let cutoff_time = SystemTime::now() - max_age;

        let entries = fs::read_dir(&self.archive_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff_time {
                            if fs::remove_file(&path).is_ok() {
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    /// Count current sessions
    fn count_sessions(&self) -> SessionResult<usize> {
        let entries = fs::read_dir(&self.sessions_dir)?;
        let count = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file() &&
                entry.path().extension().map_or(false, |ext| ext == "json")
            })
            .count();
        Ok(count)
    }

    /// Count archive files
    fn count_archive_files(&self) -> SessionResult<usize> {
        let entries = fs::read_dir(&self.archive_dir)?;
        let count = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .count();
        Ok(count)
    }

    /// Calculate total storage size
    fn calculate_total_size(&self) -> SessionResult<u64> {
        let mut total_size = 0u64;

        // Sum session files
        let session_entries = fs::read_dir(&self.sessions_dir)?;
        for entry in session_entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        // Sum archive files
        let archive_entries = fs::read_dir(&self.archive_dir)?;
        for entry in archive_entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }
}

/// Configuration for garbage collection
#[derive(Debug, Clone)]
pub struct GarbageCollectionConfig {
    /// Maximum age for sessions before cleanup
    pub max_session_age: Option<Duration>,
    /// Threshold for considering sessions stale (no activity)
    pub stale_session_threshold: Option<Duration>,
    /// Maximum age for archive files
    pub max_archive_age: Option<Duration>,
}

impl Default for GarbageCollectionConfig {
    fn default() -> Self {
        Self {
            max_session_age: Some(Duration::from_secs(30 * 24 * 60 * 60)), // 30 days
            stale_session_threshold: Some(Duration::from_secs(7 * 24 * 60 * 60)), // 7 days
            max_archive_age: Some(Duration::from_secs(90 * 24 * 60 * 60)), // 90 days
        }
    }
}

/// Result of garbage collection operation
#[derive(Debug, Clone, Default)]
pub struct GarbageCollectionResult {
    /// Number of old sessions cleaned up
    pub old_sessions_cleaned: usize,
    /// Number of stale sessions cleaned up
    pub stale_sessions_cleaned: usize,
    /// Number of old archive files cleaned up
    pub archive_files_cleaned: usize,
    /// Current number of active sessions
    pub current_session_count: usize,
    /// Current number of archive files
    pub current_archive_count: usize,
    /// Total storage size in bytes
    pub total_size_bytes: u64,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new().expect("Failed to create default SessionStore")
    }
}
