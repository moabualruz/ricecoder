//! Session management operations
//!
//! This module provides session management operations including creation, deletion,
//! renaming, and persistence of sessions.

use crate::sessions::{Session, SessionStatus, SessionWidget};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Session manager for handling session lifecycle
pub struct SessionManager {
    /// Session widget
    pub widget: SessionWidget,
    /// Session storage (ID -> Session data)
    pub storage: HashMap<String, SessionData>,
    /// Next session ID counter
    next_id: u64,
}

/// Session data for persistence
#[derive(Debug, Clone)]
pub struct SessionData {
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

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            widget: SessionWidget::new(),
            storage: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, name: String) -> Result<String> {
        let id = format!("session-{}", self.next_id);
        self.next_id += 1;

        let mut session = Session::new(id.clone(), name.clone());
        session.set_status(SessionStatus::Active);

        // Store session data
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.storage.insert(
            id.clone(),
            SessionData {
                id: id.clone(),
                name,
                content: String::new(),
                created_at: now,
                modified_at: now,
            },
        );

        // Add to widget
        self.widget.add_session(session);

        Ok(id)
    }

    /// Delete a session by ID
    pub fn delete_session(&mut self, id: &str) -> Result<()> {
        // Find and remove from widget
        if let Some(index) = self.widget.find_session_index(id) {
            self.widget.remove_session(index);
        } else {
            return Err(anyhow!("Session not found: {}", id));
        }

        // Remove from storage
        self.storage.remove(id);

        Ok(())
    }

    /// Rename a session
    pub fn rename_session(&mut self, id: &str, new_name: String) -> Result<()> {
        // Update in widget
        if let Some(index) = self.widget.find_session_index(id) {
            self.widget.rename_session(index, new_name.clone());
        } else {
            return Err(anyhow!("Session not found: {}", id));
        }

        // Update in storage
        if let Some(data) = self.storage.get_mut(id) {
            data.name = new_name;
            data.modified_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
        }

        Ok(())
    }

    /// Switch to a session
    pub fn switch_session(&mut self, id: &str) -> Result<()> {
        if !self.widget.switch_to_session_by_id(id) {
            return Err(anyhow!("Session not found: {}", id));
        }

        // Mark as active
        if let Some(session) = self.widget.current_session_mut() {
            session.set_status(SessionStatus::Active);
        }

        Ok(())
    }

    /// Get the current session ID
    pub fn current_session_id(&self) -> Option<String> {
        self.widget.current_session().map(|s| s.id.clone())
    }

    /// Get the current session name
    pub fn current_session_name(&self) -> Option<String> {
        self.widget.current_session().map(|s| s.name.clone())
    }

    /// Add a message to the current session
    pub fn add_message_to_current(&mut self, message: &str) -> Result<()> {
        if let Some(session) = self.widget.current_session_mut() {
            session.add_message();

            // Update storage
            if let Some(data) = self.storage.get_mut(&session.id) {
                data.content.push_str(message);
                data.content.push('\n');
                data.modified_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
            }

            Ok(())
        } else {
            Err(anyhow!("No active session"))
        }
    }

    /// Get session content
    pub fn get_session_content(&self, id: &str) -> Option<String> {
        self.storage.get(id).map(|d| d.content.clone())
    }

    /// Get all session IDs
    pub fn all_session_ids(&self) -> Vec<String> {
        self.widget
            .session_ids()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get all session names
    pub fn all_session_names(&self) -> Vec<String> {
        self.widget
            .session_names()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.widget.session_count()
    }

    /// Check if there are any sessions
    pub fn has_sessions(&self) -> bool {
        self.widget.has_sessions()
    }

    /// Clear all sessions
    pub fn clear_all_sessions(&mut self) {
        self.widget.clear();
        self.storage.clear();
    }

    /// Get session by ID
    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.widget.get_session(id)
    }

    /// Get session data by ID
    pub fn get_session_data(&self, id: &str) -> Option<&SessionData> {
        self.storage.get(id)
    }

    /// Mark session as dirty
    pub fn mark_session_dirty(&mut self, id: &str) -> Result<()> {
        if let Some(session) = self.widget.get_session_mut(id) {
            session.mark_dirty();
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", id))
        }
    }

    /// Mark session as clean
    pub fn mark_session_clean(&mut self, id: &str) -> Result<()> {
        if let Some(session) = self.widget.get_session_mut(id) {
            session.mark_clean();
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", id))
        }
    }

    /// Get the widget reference
    pub fn widget(&self) -> &SessionWidget {
        &self.widget
    }

    /// Get mutable widget reference
    pub fn widget_mut(&mut self) -> &mut SessionWidget {
        &mut self.widget
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut manager = SessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();

        assert!(id.starts_with("session-"));
        assert_eq!(manager.session_count(), 1);
        assert_eq!(
            manager.current_session_name(),
            Some("Test Session".to_string())
        );
    }

    #[test]
    fn test_create_multiple_sessions() {
        let mut manager = SessionManager::new();

        let id1 = manager.create_session("Session 1".to_string()).unwrap();
        let id2 = manager.create_session("Session 2".to_string()).unwrap();
        let id3 = manager.create_session("Session 3".to_string()).unwrap();

        assert_eq!(manager.session_count(), 3);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_delete_session() {
        let mut manager = SessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        assert_eq!(manager.session_count(), 1);

        manager.delete_session(&id).unwrap();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_delete_nonexistent_session() {
        let mut manager = SessionManager::new();

        let result = manager.delete_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_session() {
        let mut manager = SessionManager::new();

        let id = manager.create_session("Old Name".to_string()).unwrap();
        manager.rename_session(&id, "New Name".to_string()).unwrap();

        assert_eq!(manager.current_session_name(), Some("New Name".to_string()));
    }

    #[test]
    fn test_rename_nonexistent_session() {
        let mut manager = SessionManager::new();

        let result = manager.rename_session("nonexistent", "New Name".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_switch_session() {
        let mut manager = SessionManager::new();

        let id1 = manager.create_session("Session 1".to_string()).unwrap();
        let id2 = manager.create_session("Session 2".to_string()).unwrap();

        // After creating id2, it should be the current session
        assert_eq!(manager.current_session_id(), Some(id2.clone()));

        // Switch to id1
        manager.switch_session(&id1).unwrap();
        assert_eq!(manager.current_session_id(), Some(id1.clone()));

        // Switch back to id2
        manager.switch_session(&id2).unwrap();
        assert_eq!(manager.current_session_id(), Some(id2.clone()));
    }

    #[test]
    fn test_switch_nonexistent_session() {
        let mut manager = SessionManager::new();

        let result = manager.switch_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_message_to_current() {
        let mut manager = SessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.add_message_to_current("Hello").unwrap();

        let content = manager.get_session_content(&id).unwrap();
        assert!(content.contains("Hello"));
    }

    #[test]
    fn test_add_message_no_session() {
        let mut manager = SessionManager::new();

        let result = manager.add_message_to_current("Hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_session_ids() {
        let mut manager = SessionManager::new();

        let id1 = manager.create_session("Session 1".to_string()).unwrap();
        let id2 = manager.create_session("Session 2".to_string()).unwrap();

        let ids = manager.all_session_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_get_all_session_names() {
        let mut manager = SessionManager::new();

        manager.create_session("Session 1".to_string()).unwrap();
        manager.create_session("Session 2".to_string()).unwrap();

        let names = manager.all_session_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Session 1".to_string()));
        assert!(names.contains(&"Session 2".to_string()));
    }

    #[test]
    fn test_mark_session_dirty() {
        let mut manager = SessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.mark_session_dirty(&id).unwrap();

        let session = manager.get_session(&id).unwrap();
        assert!(session.has_changes);
    }

    #[test]
    fn test_mark_session_clean() {
        let mut manager = SessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.mark_session_dirty(&id).unwrap();
        manager.mark_session_clean(&id).unwrap();

        let session = manager.get_session(&id).unwrap();
        assert!(!session.has_changes);
    }

    #[test]
    fn test_clear_all_sessions() {
        let mut manager = SessionManager::new();

        manager.create_session("Session 1".to_string()).unwrap();
        manager.create_session("Session 2".to_string()).unwrap();

        assert_eq!(manager.session_count(), 2);

        manager.clear_all_sessions();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_data_persistence() {
        let mut manager = SessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.add_message_to_current("Message 1").unwrap();
        manager.add_message_to_current("Message 2").unwrap();

        let data = manager.get_session_data(&id).unwrap();
        assert_eq!(data.name, "Test Session");
        assert!(data.content.contains("Message 1"));
        assert!(data.content.contains("Message 2"));
    }
}
