//! Session routing for message handling

use std::collections::HashMap;

use crate::{
    error::{SessionError, SessionResult},
    models::{Message, MessageRole, Session, SessionContext},
    token_estimator::{TokenEstimator, TokenUsageTracker},
};

/// Routes messages to the appropriate session
/// Manages active session state and ensures messages are routed to the correct session
#[derive(Debug)]
pub struct SessionRouter {
    /// All sessions indexed by ID
    sessions: HashMap<String, Session>,
    /// Currently active session ID
    active_session_id: Option<String>,
    /// Tracks which session each message belongs to
    message_session_map: HashMap<String, String>, // message_id -> session_id
    /// Token estimator for tracking usage
    token_estimator: TokenEstimator,
    /// Token usage trackers per session
    token_trackers: HashMap<String, TokenUsageTracker>,
}

impl SessionRouter {
    /// Create a new session router
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            message_session_map: HashMap::new(),
            token_estimator: TokenEstimator::new(),
            token_trackers: HashMap::new(),
        }
    }

    /// Create a new session and set it as active
    pub fn create_session(
        &mut self,
        name: String,
        context: SessionContext,
    ) -> SessionResult<Session> {
        let session = Session::new(name, context);
        let session_id = session.id.clone();

        // Create token usage tracker for this session
        let tracker = self
            .token_estimator
            .create_usage_tracker(&session.context.model)?;
        self.token_trackers.insert(session_id.clone(), tracker);

        self.sessions.insert(session_id.clone(), session.clone());

        // Set as active if it's the first session
        if self.active_session_id.is_none() {
            self.active_session_id = Some(session_id);
        }

        Ok(session)
    }

    /// Route a message to the active session
    /// Returns the session ID the message was routed to
    pub fn route_to_active_session(&mut self, message_content: &str) -> SessionResult<String> {
        let session_id = self
            .active_session_id
            .as_ref()
            .ok_or(SessionError::Invalid("No active session".to_string()))?
            .clone();

        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or(SessionError::NotFound(session_id.clone()))?;

        // Estimate tokens for the message
        let token_estimate = self
            .token_estimator
            .estimate_tokens(message_content, Some(&session.context.model))?;

        // Create a message with token count and add it to the session history
        let mut message = Message::new(MessageRole::User, message_content.to_string());
        message.metadata.tokens = Some(token_estimate.tokens);

        let message_id = message.id.clone();

        session.history.push(message);
        session.updated_at = chrono::Utc::now();

        // Track token usage
        if let Some(tracker) = self.token_trackers.get_mut(&session_id) {
            tracker.record_prompt(token_estimate.tokens);
        }

        // Track which session this message belongs to
        self.message_session_map
            .insert(message_id, session_id.clone());

        Ok(session_id)
    }

    /// Route a message to a specific session
    /// Returns the session ID the message was routed to
    pub fn route_to_session(
        &mut self,
        session_id: &str,
        message_content: &str,
    ) -> SessionResult<String> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(SessionError::NotFound(session_id.to_string()))?;

        // Estimate tokens for the message
        let token_estimate = self
            .token_estimator
            .estimate_tokens(message_content, Some(&session.context.model))?;

        // Create a message with token count and add it to the session history
        let mut message = Message::new(MessageRole::User, message_content.to_string());
        message.metadata.tokens = Some(token_estimate.tokens);

        let message_id = message.id.clone();

        session.history.push(message);
        session.updated_at = chrono::Utc::now();

        // Track token usage
        if let Some(tracker) = self.token_trackers.get_mut(session_id) {
            tracker.record_prompt(token_estimate.tokens);
        }

        // Track which session this message belongs to
        self.message_session_map
            .insert(message_id, session_id.to_string());

        Ok(session_id.to_string())
    }

    /// Get the active session
    pub fn get_active_session(&self) -> SessionResult<Session> {
        let session_id = self
            .active_session_id
            .as_ref()
            .ok_or(SessionError::Invalid("No active session".to_string()))?;

        self.sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(session_id.clone()))
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> SessionResult<Session> {
        self.sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))
    }

    /// Switch to a different session
    pub fn switch_session(&mut self, session_id: &str) -> SessionResult<Session> {
        // Verify the session exists
        let session = self.get_session(session_id)?;

        self.active_session_id = Some(session_id.to_string());

        Ok(session)
    }

    /// Get the ID of the active session
    pub fn active_session_id(&self) -> Option<&str> {
        self.active_session_id.as_deref()
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<Session> {
        self.sessions.values().cloned().collect()
    }

    /// Record an AI completion response for token tracking
    pub fn record_completion(
        &mut self,
        session_id: &str,
        completion_text: &str,
    ) -> SessionResult<()> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SessionError::NotFound(session_id.to_string()))?;

        // Estimate tokens for the completion
        let token_estimate = self
            .token_estimator
            .estimate_tokens(completion_text, Some(&session.context.model))?;

        // Create completion message and add to session history
        let mut message = Message::new(MessageRole::Assistant, completion_text.to_string());
        message.metadata.tokens = Some(token_estimate.tokens);

        if let Some(session) = self.sessions.get_mut(session_id) {
            session.history.push(message);
            session.updated_at = chrono::Utc::now();
        }

        // Track token usage
        if let Some(tracker) = self.token_trackers.get_mut(session_id) {
            tracker.record_completion(token_estimate.tokens);
        }

        Ok(())
    }

    /// Get token usage for a session
    pub fn get_session_token_usage(
        &self,
        session_id: &str,
    ) -> SessionResult<crate::token_estimator::TokenUsage> {
        let tracker = self
            .token_trackers
            .get(session_id)
            .ok_or(SessionError::NotFound(format!(
                "Token tracker for session {} not found",
                session_id
            )))?;

        Ok(tracker.current_usage())
    }

    /// Get token usage for the active session
    pub fn get_active_session_token_usage(
        &self,
    ) -> SessionResult<crate::token_estimator::TokenUsage> {
        let session_id = self
            .active_session_id
            .as_ref()
            .ok_or(SessionError::Invalid("No active session".to_string()))?;

        self.get_session_token_usage(session_id)
    }

    /// Check if a session is approaching token limits
    pub fn check_session_token_limits(
        &self,
        session_id: &str,
    ) -> SessionResult<crate::token_estimator::TokenLimitStatus> {
        let usage = self.get_session_token_usage(session_id)?;
        Ok(self
            .token_estimator
            .check_token_limits(usage.total_tokens, &usage.model))
    }

    /// Get which session a message belongs to
    pub fn get_message_session(&self, message_id: &str) -> Option<String> {
        self.message_session_map.get(message_id).cloned()
    }

    /// Verify that a message belongs to a specific session
    pub fn verify_message_in_session(&self, message_id: &str, session_id: &str) -> bool {
        self.message_session_map
            .get(message_id)
            .map(|id| id == session_id)
            .unwrap_or(false)
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> SessionResult<()> {
        if !self.sessions.contains_key(session_id) {
            return Err(SessionError::NotFound(session_id.to_string()));
        }

        // Remove all messages from this session from the tracking map
        self.message_session_map.retain(|_, sid| sid != session_id);

        self.sessions.remove(session_id);

        // If the deleted session was active, switch to another session
        if self.active_session_id.as_deref() == Some(session_id) {
            self.active_session_id = self.sessions.keys().next().cloned();
        }

        Ok(())
    }

    /// Update a session
    pub fn update_session(&mut self, session: Session) -> SessionResult<()> {
        if !self.sessions.contains_key(&session.id) {
            return Err(SessionError::NotFound(session.id.clone()));
        }

        self.sessions.insert(session.id.clone(), session);
        Ok(())
    }

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionRouter {
    fn default() -> Self {
        Self::new()
    }
}
