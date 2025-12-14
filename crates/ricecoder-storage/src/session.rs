//! Session management and persistence
//!
//! This module provides session state management, persistence, and lifecycle handling
//! for RiceCoder sessions. Sessions track user interactions, state, and can be shared
//! or persisted across application restarts.

use crate::error::{StorageError, StorageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Session identifier
pub type SessionId = String;

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Unique session identifier
    pub id: SessionId,
    /// Session name (user-friendly)
    pub name: String,
    /// Session description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last modified timestamp
    pub modified_at: SystemTime,
    /// Session state
    pub state: SessionState,
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Session tags for organization
    pub tags: Vec<String>,
    /// Whether the session is shared/public
    pub is_shared: bool,
    /// Owner/creator of the session
    pub owner: String,
    /// Access permissions if shared
    pub permissions: Option<SessionPermissions>,
}

/// Session state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Current working directory
    pub working_directory: Option<PathBuf>,
    /// Active files in the session
    pub active_files: Vec<PathBuf>,
    /// Session variables/environment
    pub variables: HashMap<String, String>,
    /// Command history
    pub command_history: Vec<String>,
    /// Last command executed
    pub last_command: Option<String>,
    /// Session statistics
    pub stats: SessionStats,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total commands executed
    pub commands_executed: u64,
    /// Total files processed
    pub files_processed: u64,
    /// Session duration in seconds
    pub duration_seconds: u64,
    /// Memory usage (approximate)
    pub memory_usage_kb: u64,
    /// CPU time used (approximate)
    pub cpu_time_seconds: f64,
}

/// Session permissions for shared sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissions {
    /// Whether others can view the session
    pub can_view: bool,
    /// Whether others can execute commands in the session
    pub can_execute: bool,
    /// Whether others can modify the session
    pub can_modify: bool,
    /// Whether others can share the session further
    pub can_share: bool,
    /// List of allowed users (empty means public)
    pub allowed_users: Vec<String>,
}

/// Session manager for handling session lifecycle
pub struct SessionManager {
    /// Storage path for sessions
    storage_path: PathBuf,
    /// In-memory session cache
    sessions: Arc<RwLock<HashMap<SessionId, SessionData>>>,
    /// Auto-save interval
    auto_save_interval: Duration,
    /// Maximum number of sessions to keep in memory
    max_cached_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            auto_save_interval: Duration::from_secs(300), // 5 minutes
            max_cached_sessions: 100,
        }
    }

    /// Create a new session
    pub fn create_session(&self, name: String, owner: String) -> StorageResult<SessionData> {
        let now = SystemTime::now();
        let timestamp = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        let id = format!("session_{}_{}_{}", owner, timestamp.as_secs(), timestamp.subsec_nanos());
        let now = SystemTime::now();

        let session = SessionData {
            id: id.clone(),
            name,
            description: None,
            created_at: now,
            modified_at: now,
            state: SessionState {
                working_directory: std::env::current_dir().ok(),
                active_files: Vec::new(),
                variables: HashMap::new(),
                command_history: Vec::new(),
                last_command: None,
                stats: SessionStats {
                    commands_executed: 0,
                    files_processed: 0,
                    duration_seconds: 0,
                    memory_usage_kb: 0,
                    cpu_time_seconds: 0.0,
                },
            },
            metadata: HashMap::new(),
            tags: Vec::new(),
            is_shared: false,
            owner,
            permissions: None,
        };

        // Cache in memory
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(id.clone(), session.clone());
        }

        // Persist to disk
        self.save_session(&session)?;

        Ok(session)
    }

    /// Load a session by ID
    pub fn load_session(&self, session_id: &str) -> StorageResult<SessionData> {
        // Check cache first
        {
            let sessions = self.sessions.read().unwrap();
            if let Some(session) = sessions.get(session_id) {
                return Ok(session.clone());
            }
        }

        // Load from disk
        let session_path = self.storage_path.join(format!("{}.json", session_id));
        let content = std::fs::read_to_string(&session_path)
            .map_err(|e| StorageError::io_error(session_path.clone(), crate::error::IoOperation::Read, e))?;

        let session: SessionData = serde_json::from_str(&content)
            .map_err(|e| StorageError::parse_error(session_path.clone(), "json".to_string(), e.to_string()))?;

        // Cache in memory
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.to_string(), session.clone());
        }

        Ok(session)
    }

    /// Save a session
    pub fn save_session(&self, session: &SessionData) -> StorageResult<()> {
        // Ensure storage directory exists
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|e| StorageError::directory_creation_failed(self.storage_path.clone(), e))?;

        let session_path = self.storage_path.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)
            .map_err(|e| StorageError::parse_error(session_path.clone(), "json".to_string(), e.to_string()))?;

        std::fs::write(&session_path, content)
            .map_err(|e| StorageError::io_error(session_path, crate::error::IoOperation::Write, e))?;

        Ok(())
    }

    /// Update session state
    pub fn update_session_state(&self, session_id: &str, new_state: SessionState) -> StorageResult<()> {
        let mut session = self.load_session(session_id)?;
        session.state = new_state;
        session.modified_at = SystemTime::now();

        // Update cache
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.to_string(), session.clone());
        }

        // Persist
        self.save_session(&session)
    }

    /// Update session metadata
    pub fn update_session_metadata(&self, session_id: &str, key: String, value: serde_json::Value) -> StorageResult<()> {
        let mut session = self.load_session(session_id)?;
        session.metadata.insert(key, value);
        session.modified_at = SystemTime::now();

        // Update cache
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.to_string(), session.clone());
        }

        // Persist
        self.save_session(&session)
    }

    /// Add command to session history
    pub fn add_command_to_history(&self, session_id: &str, command: String) -> StorageResult<()> {
        let mut session = self.load_session(session_id)?;
        session.state.command_history.push(command.clone());
        session.state.last_command = Some(command);
        session.state.stats.commands_executed += 1;
        session.modified_at = SystemTime::now();

        // Update cache
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.to_string(), session.clone());
        }

        // Persist
        self.save_session(&session)
    }

    /// Share a session with permissions
    pub fn share_session(&self, session_id: &str, permissions: SessionPermissions) -> StorageResult<()> {
        let mut session = self.load_session(session_id)?;
        session.is_shared = true;
        session.permissions = Some(permissions);
        session.modified_at = SystemTime::now();

        // Update cache
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.to_string(), session.clone());
        }

        // Persist
        self.save_session(&session)
    }

    /// Unshare a session
    pub fn unshare_session(&self, session_id: &str) -> StorageResult<()> {
        let mut session = self.load_session(session_id)?;
        session.is_shared = false;
        session.permissions = None;
        session.modified_at = SystemTime::now();

        // Update cache
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.to_string(), session.clone());
        }

        // Persist
        self.save_session(&session)
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> StorageResult<()> {
        // Remove from cache
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.remove(session_id);
        }

        // Remove from disk
        let session_path = self.storage_path.join(format!("{}.json", session_id));
        if session_path.exists() {
            std::fs::remove_file(&session_path)
                .map_err(|e| StorageError::io_error(session_path, crate::error::IoOperation::Delete, e))?;
        }

        Ok(())
    }

    /// List all sessions for a user
    pub fn list_user_sessions(&self, owner: &str) -> StorageResult<Vec<SessionData>> {
        let mut user_sessions = Vec::new();

        // Check cache first
        {
            let sessions = self.sessions.read().unwrap();
            for session in sessions.values() {
                if session.owner == owner {
                    user_sessions.push(session.clone());
                }
            }
        }

        // Also check disk for sessions not in cache
        if let Ok(entries) = std::fs::read_dir(&self.storage_path) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") && file_name.starts_with("session_") {
                        let session_id = file_name.trim_end_matches(".json");
                        if let Ok(session) = self.load_session(session_id) {
                            if session.owner == owner && !user_sessions.iter().any(|s| s.id == session.id) {
                                user_sessions.push(session);
                            }
                        }
                    }
                }
            }
        }

        Ok(user_sessions)
    }

    /// List shared sessions accessible to a user
    pub fn list_shared_sessions(&self, user: &str) -> StorageResult<Vec<SessionData>> {
        let mut shared_sessions = Vec::new();

        // Check all sessions (this is a simplified implementation)
        if let Ok(entries) = std::fs::read_dir(&self.storage_path) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") && file_name.starts_with("session_") {
                        let session_id = file_name.trim_end_matches(".json");
                        if let Ok(session) = self.load_session(session_id) {
                            if session.is_shared {
                                if let Some(perms) = &session.permissions {
                                    // Check if user has access
                                    if perms.allowed_users.is_empty() || perms.allowed_users.contains(&user.to_string()) {
                                        shared_sessions.push(session);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(shared_sessions)
    }

    /// Clean up old sessions
    pub fn cleanup_old_sessions(&self, max_age: Duration) -> StorageResult<usize> {
        let cutoff = SystemTime::now() - max_age;
        let mut cleaned_count = 0;

        // Check all sessions on disk
        if let Ok(entries) = std::fs::read_dir(&self.storage_path) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") && file_name.starts_with("session_") {
                        let session_id = file_name.trim_end_matches(".json");
                        if let Ok(session) = self.load_session(session_id) {
                            if session.modified_at < cutoff {
                                let _ = self.delete_session(&session.id);
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    /// Get session statistics
    pub fn get_session_stats(&self) -> StorageResult<SessionManagerStats> {
        let mut total_sessions = 0;
        let mut active_sessions = 0;
        let mut shared_sessions = 0;
        let mut total_commands = 0;

        // Check all sessions
        if let Ok(entries) = std::fs::read_dir(&self.storage_path) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") && file_name.starts_with("session_") {
                        total_sessions += 1;
                        let session_id = file_name.trim_end_matches(".json");
                        if let Ok(session) = self.load_session(session_id) {
                            if session.is_shared {
                                shared_sessions += 1;
                            }
                            total_commands += session.state.stats.commands_executed;

                            // Consider active if modified within last hour
                            if session.modified_at > SystemTime::now() - Duration::from_secs(3600) {
                                active_sessions += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(SessionManagerStats {
            total_sessions,
            active_sessions,
            shared_sessions,
            total_commands,
        })
    }
}

/// Session manager statistics
#[derive(Debug, Clone)]
pub struct SessionManagerStats {
    /// Total number of sessions
    pub total_sessions: usize,
    /// Number of active sessions (modified within last hour)
    pub active_sessions: usize,
    /// Number of shared sessions
    pub shared_sessions: usize,
    /// Total commands executed across all sessions
    pub total_commands: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());

        let session = manager.create_session("Test Session".to_string(), "test_user".to_string()).unwrap();

        assert_eq!(session.name, "Test Session");
        assert_eq!(session.owner, "test_user");
        assert!(!session.is_shared);
        assert!(session.state.command_history.is_empty());
    }

    #[test]
    fn test_session_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());

        // Create and save session
        let session = manager.create_session("Test Session".to_string(), "test_user".to_string()).unwrap();
        let session_id = session.id.clone();

        // Load session in new manager instance
        let manager2 = SessionManager::new(temp_dir.path().to_path_buf());
        let loaded_session = manager2.load_session(&session_id).unwrap();

        assert_eq!(loaded_session.id, session.id);
        assert_eq!(loaded_session.name, session.name);
        assert_eq!(loaded_session.owner, session.owner);
    }

    #[test]
    fn test_session_state_update() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());

        let session = manager.create_session("Test Session".to_string(), "test_user".to_string()).unwrap();
        let session_id = session.id;

        // Update session state
        let new_state = SessionState {
            working_directory: Some(PathBuf::from("/tmp")),
            active_files: vec![PathBuf::from("file1.rs")],
            variables: HashMap::from([("TEST_VAR".to_string(), "test_value".to_string())]),
            command_history: vec!["echo hello".to_string()],
            last_command: Some("echo hello".to_string()),
            stats: SessionStats {
                commands_executed: 1,
                files_processed: 1,
                duration_seconds: 60,
                memory_usage_kb: 1024,
                cpu_time_seconds: 0.5,
            },
        };

        manager.update_session_state(&session_id, new_state.clone()).unwrap();

        // Load and verify
        let updated_session = manager.load_session(&session_id).unwrap();
        assert_eq!(updated_session.state.working_directory, new_state.working_directory);
        assert_eq!(updated_session.state.command_history, new_state.command_history);
    }

    #[test]
    fn test_session_sharing() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());

        let session = manager.create_session("Test Session".to_string(), "test_user".to_string()).unwrap();
        let session_id = session.id;

        // Share session
        let permissions = SessionPermissions {
            can_view: true,
            can_execute: false,
            can_modify: false,
            can_share: false,
            allowed_users: vec!["user2".to_string()],
        };

        manager.share_session(&session_id, permissions.clone()).unwrap();

        // Load and verify
        let shared_session = manager.load_session(&session_id).unwrap();
        assert!(shared_session.is_shared);
        assert_eq!(shared_session.permissions.unwrap().can_view, permissions.can_view);

        // Test shared session listing
        let shared_sessions = manager.list_shared_sessions("user2").unwrap();
        assert!(shared_sessions.iter().any(|s| s.id == session_id));
    }

    #[test]
    fn test_command_history() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());

        let session = manager.create_session("Test Session".to_string(), "test_user".to_string()).unwrap();
        let session_id = session.id;

        // Add commands
        manager.add_command_to_history(&session_id, "echo hello".to_string()).unwrap();
        manager.add_command_to_history(&session_id, "ls -la".to_string()).unwrap();

        // Load and verify
        let updated_session = manager.load_session(&session_id).unwrap();
        assert_eq!(updated_session.state.command_history.len(), 2);
        assert_eq!(updated_session.state.last_command, Some("ls -la".to_string()));
        assert_eq!(updated_session.state.stats.commands_executed, 2);
    }

    #[test]
    fn test_session_performance() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());

        // Test session creation performance
        let start = std::time::Instant::now();
        let session = manager.create_session("Performance Test".to_string(), "perf_user".to_string()).unwrap();
        let creation_time = start.elapsed();

        // Should be fast (< 10ms)
        assert!(creation_time < std::time::Duration::from_millis(10));

        let session_id = session.id;

        // Test session loading performance
        let start = std::time::Instant::now();
        let loaded_session = manager.load_session(&session_id).unwrap();
        let load_time = start.elapsed();

        // Should be fast (< 5ms)
        assert!(load_time < std::time::Duration::from_millis(5));
        assert_eq!(loaded_session.id, session_id);
    }

    #[test]
    fn test_bulk_session_operations() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());

        // Create multiple sessions
        let mut session_ids = Vec::new();
        for i in 0..10 {
            let session = manager.create_session(format!("Bulk Session {}", i), "bulk_user".to_string()).unwrap();
            session_ids.push(session.id);
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Test bulk loading
        let start = std::time::Instant::now();
        for session_id in &session_ids {
            let _session = manager.load_session(session_id).unwrap();
        }
        let bulk_load_time = start.elapsed();

        // Should be reasonably fast (< 50ms for 10 sessions)
        assert!(bulk_load_time < std::time::Duration::from_millis(50));

        // Test listing user sessions
        let user_sessions = manager.list_user_sessions("bulk_user").unwrap();
        assert_eq!(user_sessions.len(), 10);
    }
}