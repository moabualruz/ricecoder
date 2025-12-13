//! Session management operations
//!
//! This module provides session management operations including creation, deletion,
//! renaming, and persistence of sessions.

use crate::models::{Session, SessionStatus};
use crate::error::{SessionError, SessionResult};
use std::collections::HashMap;

/// Session manager for handling session lifecycle
pub struct TuiSessionManager {
    /// Session storage (ID -> Session data)
    pub storage: HashMap<String, TuiSessionData>,
    /// Next session ID counter
    next_id: u64,
}

/// Session data for persistence
#[derive(Debug, Clone)]
pub struct TuiSessionData {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Session content/messages
    pub content: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

impl TuiSessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, name: String) -> SessionResult<String> {
        let id = format!("session-{}", self.next_id);
        self.next_id += 1;

        // Store session data
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.storage.insert(
            id.clone(),
            TuiSessionData {
                id: id.clone(),
                name,
                content: String::new(),
                created_at: now,
                modified_at: now,
            },
        );

        Ok(id)
    }

    /// Delete a session by ID
    pub fn delete_session(&mut self, id: &str) -> SessionResult<()> {
        // Remove from storage
        if self.storage.remove(id).is_none() {
            return Err(SessionError::NotFound(format!("Session not found: {}", id)));
        }

        Ok(())
    }

    /// Rename a session
    pub fn rename_session(&mut self, id: &str, new_name: String) -> SessionResult<()> {
        // Update in storage
        if let Some(data) = self.storage.get_mut(id) {
            data.name = new_name;
            data.modified_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Ok(())
        } else {
            Err(SessionError::NotFound(format!("Session not found: {}", id)))
        }
    }

    /// Add content to a session
    pub fn add_content_to_session(&mut self, id: &str, content: &str) -> SessionResult<()> {
        if let Some(data) = self.storage.get_mut(id) {
            data.content.push_str(content);
            data.content.push('\n');
            data.modified_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Ok(())
        } else {
            Err(SessionError::NotFound(format!("Session not found: {}", id)))
        }
    }

    /// Get session content
    pub fn get_session_content(&self, id: &str) -> Option<String> {
        self.storage.get(id).map(|d| d.content.clone())
    }

    /// Get all session IDs
    pub fn all_session_ids(&self) -> Vec<String> {
        self.storage.keys().cloned().collect()
    }

    /// Get all session names
    pub fn all_session_names(&self) -> Vec<String> {
        self.storage.values().map(|d| d.name.clone()).collect()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.storage.len()
    }

    /// Check if there are any sessions
    pub fn has_sessions(&self) -> bool {
        !self.storage.is_empty()
    }

    /// Clear all sessions
    pub fn clear_all_sessions(&mut self) {
        self.storage.clear();
    }

    /// Get session data by ID
    pub fn get_session_data(&self, id: &str) -> Option<&TuiSessionData> {
        self.storage.get(id)
    }
}

impl Default for TuiSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

