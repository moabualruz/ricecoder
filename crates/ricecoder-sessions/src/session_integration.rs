//! Integration between ricecoder-sessions and TUI session widgets
//!
//! This module bridges the core session management (ricecoder-sessions) with
//! the TUI session display widgets, ensuring session status is properly displayed
//! and session switching is routed correctly.

use crate::models::{Session, SessionStatus as CoreSessionStatus};
use crate::SessionManager;

/// Integrates core session management with TUI display
/// Note: This is a simplified version moved from ricecoder-tui.
/// The full integration with TUI widgets remains in ricecoder-tui.
pub struct SessionIntegration {
    /// Core session manager (single source of truth)
    pub manager: SessionManager,
}

impl SessionIntegration {
    /// Create a new session integration with a session limit
    pub fn new(session_limit: usize) -> Self {
        Self {
            manager: SessionManager::new(session_limit),
        }
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
    pub fn get_session(&self, session_id: &str) -> crate::error::SessionResult<Session> {
        self.manager.get_session(session_id)
    }

    /// Get the active session
    pub fn get_active_session(&self) -> crate::error::SessionResult<Session> {
        self.manager.get_active_session()
    }

    /// Create a new session
    pub fn create_session(
        &mut self,
        name: String,
        context: crate::models::SessionContext,
    ) -> crate::error::SessionResult<Session> {
        self.manager.create_session(name, context)
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> crate::error::SessionResult<()> {
        self.manager.delete_session(session_id)
    }

    /// Switch to a session
    pub fn switch_session(&mut self, session_id: &str) -> crate::error::SessionResult<()> {
        self.manager.switch_session(session_id)?;
        Ok(())
    }

    /// Add a message to the active session
    pub fn add_message_to_active(&mut self, message_content: &str) -> crate::error::SessionResult<String> {
        // Get session info first
        let session = self.manager.get_active_session()?;
        let session_id = session.id.clone();
        let model = session.context.model.clone();

        // Estimate tokens for the message
        let token_count = {
            let token_estimate = self.manager.estimate_tokens_with_model(message_content, &model)?;
            token_estimate.tokens
        };

        // Get session again and modify it
        let mut session = self.manager.get_active_session()?;
        let mut message = crate::models::Message::new(
            crate::models::MessageRole::User,
            message_content.to_string(),
        );
        message.metadata.tokens = Some(token_count);

        session.history.push(message);
        session.updated_at = chrono::Utc::now();

        self.manager.update_session(session)?;

        // Record prompt tokens
        self.manager.record_prompt_tokens(&session_id, token_count)?;

        Ok(session_id)
    }

    /// Add a message to a specific session
    pub fn add_message_to_session(
        &mut self,
        session_id: &str,
        message_content: &str,
    ) -> crate::error::SessionResult<String> {
        let mut session = self.manager.get_session(session_id)?;
        let message = crate::models::Message::new(
            crate::models::MessageRole::User,
            message_content.to_string(),
        );
        session.history.push(message);
        session.updated_at = chrono::Utc::now();

        self.manager.update_session(session)?;
        Ok(session_id.to_string())
    }

    /// Get token usage for the active session
    pub fn get_active_session_token_usage(&self) -> crate::error::SessionResult<crate::token_estimator::TokenUsage> {
        self.manager.get_active_session_token_usage()
    }

    /// Send a user message and track token usage
    pub fn send_user_message(&mut self, content: &str) -> crate::error::SessionResult<()> {
        // For now, create a simple session if none exists
        if self.manager.list_sessions().is_empty() {
            let context = crate::models::SessionContext::new(
                "openai".to_string(),
                "gpt-4".to_string(),
                crate::models::SessionMode::Chat,
            );
            self.create_session("Default Session".to_string(), context)?;
        }

        // Get active session
        let session = self.manager.get_active_session()?;

        // Estimate tokens for the message
        let token_count = {
            let token_estimate = self.manager.estimate_tokens_for_active_session(content)?;
            token_estimate.tokens
        };

        // Record prompt tokens
        if let Some(session_id) = self.manager.active_session_id() {
            let session_id = session_id.to_string();
            self.manager.record_prompt_tokens(&session_id, token_count)?;
        }

        Ok(())
    }

    /// Record completion tokens for AI responses
    pub fn record_completion_tokens(&mut self, tokens: usize) -> crate::error::SessionResult<()> {
        if let Some(session_id) = self.manager.active_session_id() {
            let session_id = session_id.to_string(); // Clone the string
            self.manager.record_completion_tokens(&session_id, tokens)?;
        }
        Ok(())
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
    use crate::models::SessionMode;

    fn create_test_context() -> crate::models::SessionContext {
        crate::models::SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        )
    }

    #[test]
    fn test_create_session_integration() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session = integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        assert_eq!(integration.session_count(), 1);
        assert_eq!(integration.active_session_id(), Some(session.id.as_str()));
    }

    #[test]
    fn test_message_routing() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session = integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        let routed_id = integration.add_message_to_active("Hello").unwrap();

        assert_eq!(routed_id, session.id);

        let session = integration.get_session(&session.id).unwrap();
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].content(), "Hello");
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