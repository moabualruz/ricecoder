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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();

        assert!(id.starts_with("session-"));
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_create_multiple_sessions() {
        let mut manager = TuiSessionManager::new();

        let id1 = manager.create_session("Session 1".to_string()).unwrap();
        let id2 = manager.create_session("Session 2".to_string()).unwrap();
        let id3 = manager.create_session("Session 3".to_string()).unwrap();

        assert_eq!(manager.session_count(), 3);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_delete_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        assert_eq!(manager.session_count(), 1);

        manager.delete_session(&id).unwrap();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_delete_nonexistent_session() {
        let mut manager = TuiSessionManager::new();

        let result = manager.delete_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Old Name".to_string()).unwrap();
        manager.rename_session(&id, "New Name".to_string()).unwrap();

        let data = manager.get_session_data(&id).unwrap();
        assert_eq!(data.name, "New Name");
    }

    #[test]
    fn test_rename_nonexistent_session() {
        let mut manager = TuiSessionManager::new();

        let result = manager.rename_session("nonexistent", "New Name".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_add_content_to_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.add_content_to_session(&id, "Hello").unwrap();
        manager.add_content_to_session(&id, "World").unwrap();

        let content = manager.get_session_content(&id).unwrap();
        assert!(content.contains("Hello"));
        assert!(content.contains("World"));
    }

    #[test]
    fn test_get_all_session_ids() {
        let mut manager = TuiSessionManager::new();

        let id1 = manager.create_session("Session 1".to_string()).unwrap();
        let id2 = manager.create_session("Session 2".to_string()).unwrap();

        let ids = manager.all_session_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_get_all_session_names() {
        let mut manager = TuiSessionManager::new();

        manager.create_session("Session 1".to_string()).unwrap();
        manager.create_session("Session 2".to_string()).unwrap();

        let names = manager.all_session_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Session 1".to_string()));
        assert!(names.contains(&"Session 2".to_string()));
    }

    #[test]
    fn test_clear_all_sessions() {
        let mut manager = TuiSessionManager::new();

        manager.create_session("Session 1".to_string()).unwrap();
        manager.create_session("Session 2".to_string()).unwrap();

        assert_eq!(manager.session_count(), 2);

        manager.clear_all_sessions();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_data_persistence() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.add_content_to_session(&id, "Message 1").unwrap();
        manager.add_content_to_session(&id, "Message 2").unwrap();

        let data = manager.get_session_data(&id).unwrap();
        assert_eq!(data.name, "Test Session");
        assert!(data.content.contains("Message 1"));
        assert!(data.content.contains("Message 2"));
    }
}