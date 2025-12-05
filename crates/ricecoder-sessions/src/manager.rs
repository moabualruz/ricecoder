//! Session manager for lifecycle management and session switching

use crate::error::{SessionError, SessionResult};
use crate::models::{Session, SessionContext};
use std::collections::HashMap;

/// Manages session lifecycle and switching
#[derive(Debug, Clone)]
pub struct SessionManager {
    /// All sessions indexed by ID
    sessions: HashMap<String, Session>,
    /// Currently active session ID
    active_session_id: Option<String>,
    /// Maximum number of concurrent sessions
    session_limit: usize,
}

impl SessionManager {
    /// Create a new session manager with a session limit
    pub fn new(session_limit: usize) -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            session_limit,
        }
    }

    /// Create a new session
    pub fn create_session(
        &mut self,
        name: String,
        context: SessionContext,
    ) -> SessionResult<Session> {
        // Check session limit
        if self.sessions.len() >= self.session_limit {
            return Err(SessionError::LimitReached {
                max: self.session_limit,
            });
        }

        let session = Session::new(name, context);
        let session_id = session.id.clone();

        self.sessions.insert(session_id.clone(), session.clone());

        // Set as active if it's the first session
        if self.active_session_id.is_none() {
            self.active_session_id = Some(session_id);
        }

        Ok(session)
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> SessionResult<()> {
        if !self.sessions.contains_key(session_id) {
            return Err(SessionError::NotFound(session_id.to_string()));
        }

        self.sessions.remove(session_id);

        // If the deleted session was active, switch to another session
        if self.active_session_id.as_deref() == Some(session_id) {
            self.active_session_id = self.sessions.keys().next().cloned();
        }

        Ok(())
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> SessionResult<Session> {
        self.sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))
    }

    /// Get the active session
    pub fn get_active_session(&self) -> SessionResult<Session> {
        let session_id = self
            .active_session_id
            .as_ref()
            .ok_or(SessionError::Invalid("No active session".to_string()))?;

        self.get_session(session_id)
    }

    /// Switch to a different session
    pub fn switch_session(&mut self, session_id: &str) -> SessionResult<Session> {
        // Verify the session exists
        let session = self.get_session(session_id)?;

        self.active_session_id = Some(session_id.to_string());

        Ok(session)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<Session> {
        self.sessions.values().cloned().collect()
    }

    /// Get the ID of the active session
    pub fn active_session_id(&self) -> Option<&str> {
        self.active_session_id.as_deref()
    }

    /// Update a session
    pub fn update_session(&mut self, session: Session) -> SessionResult<()> {
        if !self.sessions.contains_key(&session.id) {
            return Err(SessionError::NotFound(session.id.clone()));
        }

        self.sessions.insert(session.id.clone(), session);
        Ok(())
    }

    /// Get the session limit
    pub fn session_limit(&self) -> usize {
        self.session_limit
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if session limit is reached
    pub fn is_limit_reached(&self) -> bool {
        self.sessions.len() >= self.session_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionMode;

    fn create_test_context() -> SessionContext {
        SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
    }

    #[test]
    fn test_create_session() {
        let mut manager = SessionManager::new(5);
        let context = create_test_context();

        let session = manager
            .create_session("Test Session".to_string(), context)
            .unwrap();

        assert_eq!(session.name, "Test Session");
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_session_limit_enforcement() {
        let mut manager = SessionManager::new(2);
        let context = create_test_context();

        // Create first session
        manager
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();

        // Create second session
        manager
            .create_session("Session 2".to_string(), context.clone())
            .unwrap();

        // Third session should fail
        let result = manager.create_session("Session 3".to_string(), context);
        assert!(matches!(result, Err(SessionError::LimitReached { max: 2 })));
    }

    #[test]
    fn test_switch_session() {
        let mut manager = SessionManager::new(5);
        let context = create_test_context();

        let _session1 = manager
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        let session2 = manager
            .create_session("Session 2".to_string(), context)
            .unwrap();

        // Switch to session 2
        manager.switch_session(&session2.id).unwrap();

        let active = manager.get_active_session().unwrap();
        assert_eq!(active.id, session2.id);
    }

    #[test]
    fn test_delete_session() {
        let mut manager = SessionManager::new(5);
        let context = create_test_context();

        let session = manager
            .create_session("Test Session".to_string(), context)
            .unwrap();

        manager.delete_session(&session.id).unwrap();

        assert_eq!(manager.session_count(), 0);
        assert!(manager.get_session(&session.id).is_err());
    }

    #[test]
    fn test_list_sessions() {
        let mut manager = SessionManager::new(5);
        let context = create_test_context();

        manager
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        manager
            .create_session("Session 2".to_string(), context)
            .unwrap();

        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
    }
}
