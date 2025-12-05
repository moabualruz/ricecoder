//! Integration between ricecoder-sessions and TUI session widgets
//!
//! This module bridges the core session management (ricecoder-sessions) with
//! the TUI session display widgets, ensuring session status is properly displayed
//! and session switching is routed correctly.

use crate::sessions::{Session as TuiSession, SessionStatus as TuiSessionStatus, SessionWidget};
use ricecoder_sessions::{Session, SessionManager, SessionStatus as CoreSessionStatus};

/// Integrates core session management with TUI display
pub struct SessionIntegration {
    /// Core session manager (single source of truth)
    pub manager: SessionManager,
    /// TUI session widget for display
    pub widget: SessionWidget,
}

impl SessionIntegration {
    /// Create a new session integration with a session limit
    pub fn new(session_limit: usize) -> Self {
        Self {
            manager: SessionManager::new(session_limit),
            widget: SessionWidget::new(),
        }
    }

    /// Sync core sessions to TUI widget for display
    /// This should be called whenever sessions change to keep the UI in sync
    pub fn sync_to_widget(&mut self) {
        // Get all sessions from the core manager
        let sessions = self.manager.list_sessions();

        // Clear the widget and rebuild it
        self.widget.clear();

        for session in sessions {
            let tui_session = self.convert_to_tui_session(&session);
            self.widget.add_session(tui_session);
        }

        // Set the selected index to match the active session
        if let Some(active_id) = self.manager.active_session_id() {
            if let Some(index) = self.widget.find_session_index(active_id) {
                self.widget.switch_to_session(index);
            }
        }
    }

    /// Convert a core Session to a TUI Session for display
    fn convert_to_tui_session(&self, session: &Session) -> TuiSession {
        let tui_status = match session.status {
            CoreSessionStatus::Active => TuiSessionStatus::Active,
            CoreSessionStatus::Paused => TuiSessionStatus::Idle,
            CoreSessionStatus::Archived => TuiSessionStatus::Idle,
        };

        let mut tui_session = TuiSession::new(session.id.clone(), session.name.clone());
        tui_session.set_status(tui_status);
        tui_session.message_count = session.history.len();
        tui_session.last_activity = session.updated_at.timestamp().max(0) as u64;

        tui_session
    }

    /// Handle session switching from the TUI widget
    /// Updates the core manager to maintain consistency
    pub fn handle_session_switch(&mut self, session_id: &str) -> Result<(), String> {
        // Switch in the core manager
        self.manager
            .switch_session(session_id)
            .map_err(|e| e.to_string())?;

        // Sync the widget to reflect the change
        self.sync_to_widget();

        Ok(())
    }

    /// Create a new session and add it to the manager
    pub fn create_session(
        &mut self,
        name: String,
        context: ricecoder_sessions::SessionContext,
    ) -> Result<String, String> {
        // Create in the core manager
        let session = self
            .manager
            .create_session(name.clone(), context.clone())
            .map_err(|e| e.to_string())?;

        let session_id = session.id.clone();

        // Sync the widget to reflect the change
        self.sync_to_widget();

        Ok(session_id)
    }

    /// Delete a session from the manager
    pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        // Delete from the core manager
        self.manager
            .delete_session(session_id)
            .map_err(|e| e.to_string())?;

        // Sync the widget to reflect the change
        self.sync_to_widget();

        Ok(())
    }

    /// Get the currently active session ID
    pub fn active_session_id(&self) -> Option<&str> {
        self.manager.active_session_id()
    }

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.manager.session_count()
    }

    /// Check if the session limit is reached
    pub fn is_limit_reached(&self) -> bool {
        self.manager.is_limit_reached()
    }

    /// Get all sessions
    pub fn list_sessions(&self) -> Vec<Session> {
        self.manager.list_sessions()
    }

    /// Get a specific session
    pub fn get_session(&self, session_id: &str) -> Result<Session, String> {
        self.manager
            .get_session(session_id)
            .map_err(|e| e.to_string())
    }

    /// Get the active session
    pub fn get_active_session(&self) -> Result<Session, String> {
        self.manager.get_active_session().map_err(|e| e.to_string())
    }

    /// Add a message to the active session
    pub fn add_message_to_active(&mut self, message_content: &str) -> Result<String, String> {
        // Get the active session
        let mut session = self
            .manager
            .get_active_session()
            .map_err(|e| e.to_string())?;

        let session_id = session.id.clone();

        // Add the message to the session
        let message = ricecoder_sessions::Message::new(
            ricecoder_sessions::MessageRole::User,
            message_content.to_string(),
        );
        session.history.push(message);
        session.updated_at = chrono::Utc::now();

        // Update the session in the manager
        self.manager
            .update_session(session)
            .map_err(|e| e.to_string())?;

        Ok(session_id)
    }

    /// Add a message to a specific session
    pub fn add_message_to_session(
        &mut self,
        session_id: &str,
        message_content: &str,
    ) -> Result<String, String> {
        // Get the session
        let mut session = self
            .manager
            .get_session(session_id)
            .map_err(|e| e.to_string())?;

        // Add the message to the session
        let message = ricecoder_sessions::Message::new(
            ricecoder_sessions::MessageRole::User,
            message_content.to_string(),
        );
        session.history.push(message);
        session.updated_at = chrono::Utc::now();

        // Update the session in the manager
        self.manager
            .update_session(session)
            .map_err(|e| e.to_string())?;

        Ok(session_id.to_string())
    }

    /// Get the TUI widget reference
    pub fn widget(&self) -> &SessionWidget {
        &self.widget
    }

    /// Get mutable TUI widget reference
    pub fn widget_mut(&mut self) -> &mut SessionWidget {
        &mut self.widget
    }
}

impl Default for SessionIntegration {
    fn default() -> Self {
        Self::new(10) // Default to 10 concurrent sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_sessions::SessionMode;

    fn create_test_context() -> ricecoder_sessions::SessionContext {
        ricecoder_sessions::SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        )
    }

    #[test]
    fn test_create_session_integration() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session_id = integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        assert_eq!(integration.session_count(), 1);
        assert_eq!(integration.active_session_id(), Some(session_id.as_str()));
        assert_eq!(integration.widget.session_count(), 1);
    }

    #[test]
    fn test_sync_to_widget() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        integration
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        integration
            .create_session("Session 2".to_string(), context)
            .unwrap();

        assert_eq!(integration.widget.session_count(), 2);

        let widget_sessions = integration.widget.session_names();
        assert!(widget_sessions.contains(&"Session 1"));
        assert!(widget_sessions.contains(&"Session 2"));
    }

    #[test]
    fn test_session_switch_integration() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session1_id = integration
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        let session2_id = integration
            .create_session("Session 2".to_string(), context)
            .unwrap();

        // Switch to session 1
        integration.handle_session_switch(&session1_id).unwrap();

        assert_eq!(integration.active_session_id(), Some(session1_id.as_str()));
        assert_eq!(
            integration.widget.current_session().unwrap().id,
            session1_id
        );

        // Switch to session 2
        integration.handle_session_switch(&session2_id).unwrap();

        assert_eq!(integration.active_session_id(), Some(session2_id.as_str()));
        assert_eq!(
            integration.widget.current_session().unwrap().id,
            session2_id
        );
    }

    #[test]
    fn test_delete_session_integration() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session_id = integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        assert_eq!(integration.session_count(), 1);

        integration.delete_session(&session_id).unwrap();

        assert_eq!(integration.session_count(), 0);
        assert_eq!(integration.widget.session_count(), 0);
    }

    #[test]
    fn test_session_status_display() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        let tui_session = integration.widget.current_session().unwrap();
        assert_eq!(tui_session.status, TuiSessionStatus::Active);
    }

    #[test]
    fn test_message_routing() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session_id = integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        let routed_id = integration.add_message_to_active("Hello").unwrap();

        assert_eq!(routed_id, session_id);

        let session = integration.get_session(&session_id).unwrap();
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].content, "Hello");
    }

    #[test]
    fn test_session_limit_enforcement() {
        let mut integration = SessionIntegration::new(2);
        let context = create_test_context();

        integration
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        integration
            .create_session("Session 2".to_string(), context.clone())
            .unwrap();

        assert!(integration.is_limit_reached());

        let result = integration.create_session("Session 3".to_string(), context);
        assert!(result.is_err());
    }
}
