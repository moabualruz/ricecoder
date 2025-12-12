//! Session manager for lifecycle management and session switching

use crate::error::{SessionError, SessionResult};
use crate::models::{Session, SessionContext, MessagePart};
use crate::token_estimator::{TokenEstimator, TokenUsageTracker};
use std::collections::{HashMap, HashSet};
use chrono::{Duration, Utc};

/// Manages session lifecycle and switching
#[derive(Debug)]
pub struct SessionManager {
    /// All sessions indexed by ID
    sessions: HashMap<String, Session>,
    /// Currently active session ID
    active_session_id: Option<String>,
    /// Maximum number of concurrent sessions
    session_limit: usize,
    /// Token estimator for tracking usage
    token_estimator: TokenEstimator,
    /// Token usage trackers per session
    token_trackers: HashMap<String, TokenUsageTracker>,
}

impl SessionManager {
    /// Create a new session manager with a session limit
    pub fn new(session_limit: usize) -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            session_limit,
            token_estimator: TokenEstimator::new(),
            token_trackers: HashMap::new(),
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

        // Create token usage tracker for this session
        let tracker = self.token_estimator.create_usage_tracker(&session.context.model)?;
        self.token_trackers.insert(session_id.clone(), tracker);

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

    /// Record token usage for a prompt message
    pub fn record_prompt_tokens(&mut self, session_id: &str, tokens: usize) -> SessionResult<()> {
        if let Some(tracker) = self.token_trackers.get_mut(session_id) {
            tracker.record_prompt(tokens);
            Ok(())
        } else {
            Err(SessionError::NotFound(format!("Token tracker for session {} not found", session_id)))
        }
    }

    /// Record token usage for a completion message
    pub fn record_completion_tokens(&mut self, session_id: &str, tokens: usize) -> SessionResult<()> {
        if let Some(tracker) = self.token_trackers.get_mut(session_id) {
            tracker.record_completion(tokens);
            Ok(())
        } else {
            Err(SessionError::NotFound(format!("Token tracker for session {} not found", session_id)))
        }
    }

    /// Get token usage for a session
    pub fn get_session_token_usage(&self, session_id: &str) -> SessionResult<crate::token_estimator::TokenUsage> {
        if let Some(tracker) = self.token_trackers.get(session_id) {
            Ok(tracker.current_usage())
        } else {
            Err(SessionError::NotFound(format!("Token tracker for session {} not found", session_id)))
        }
    }

    /// Get token usage for the active session
    pub fn get_active_session_token_usage(&self) -> SessionResult<crate::token_estimator::TokenUsage> {
        let session_id = self
            .active_session_id
            .as_ref()
            .ok_or(SessionError::Invalid("No active session".to_string()))?;

        self.get_session_token_usage(session_id)
    }

    /// Check if a session is approaching token limits
    pub fn check_session_token_limits(&self, session_id: &str) -> SessionResult<crate::token_estimator::TokenLimitStatus> {
        let usage = self.get_session_token_usage(session_id)?;
        Ok(self.token_estimator.check_token_limits(usage.total_tokens, &usage.model))
    }

    /// Estimate tokens for content using the active session's model
    pub fn estimate_tokens_for_active_session(&mut self, content: &str) -> SessionResult<crate::token_estimator::TokenEstimate> {
        let session = self.get_active_session()?;
        self.token_estimator.estimate_tokens(content, Some(&session.context.model))
    }

    /// Estimate tokens for content using a specific model
    pub fn estimate_tokens_with_model(&mut self, content: &str, model: &str) -> SessionResult<crate::token_estimator::TokenEstimate> {
        self.token_estimator.estimate_tokens(content, Some(model))
    }

    /// Generate a descriptive title for a session based on its content
    pub fn generate_session_title(&self, session_id: &str) -> SessionResult<String> {
        let session = self.get_session(session_id)?;

        // If session already has a custom title, use it
        if !session.name.is_empty() && session.name != "Untitled" {
            return Ok(session.name.clone());
        }

        // Generate title based on message content
        if session.history.is_empty() {
            return Ok("Empty Session".to_string());
        }

        // Analyze the first few messages to determine the topic
        let mut topics = Vec::new();
        let mut tools_used = HashSet::new();

        for message in session.history.iter().take(5) {
            // Extract text content
            for part in &message.parts {
                match part {
                    MessagePart::Text(text) => {
                        // Simple keyword extraction (could be enhanced with NLP)
                        let lower_text = text.to_lowercase();
                        if lower_text.contains("error") || lower_text.contains("bug") || lower_text.contains("fix") {
                            topics.push("Bug Fix");
                        } else if lower_text.contains("feature") || lower_text.contains("implement") || lower_text.contains("add") {
                            topics.push("Feature Development");
                        } else if lower_text.contains("refactor") || lower_text.contains("clean") || lower_text.contains("optimize") {
                            topics.push("Code Refactoring");
                        } else if lower_text.contains("test") || lower_text.contains("testing") {
                            topics.push("Testing");
                        } else if lower_text.contains("documentation") || lower_text.contains("docs") || lower_text.contains("readme") {
                            topics.push("Documentation");
                        }
                    }
                    MessagePart::ToolInvocation(invocation) => {
                        tools_used.insert(invocation.tool_name.clone());
                    }
                    MessagePart::ToolResult(result) => {
                        tools_used.insert(result.tool_name.clone());
                    }
                    _ => {}
                }
            }
        }

        // Generate title based on analysis
        let title = if !topics.is_empty() {
            // Use the most common topic
            let primary_topic = topics.into_iter().next().unwrap();
            if tools_used.is_empty() {
                format!("{} Session", primary_topic)
            } else {
                format!("{} with {}", primary_topic, tools_used.into_iter().next().unwrap())
            }
        } else if !tools_used.is_empty() {
            format!("Tool Usage: {}", tools_used.into_iter().next().unwrap())
        } else {
            // Fallback: use first message content
            let first_content = session.history[0].content();
            if first_content.len() > 50 {
                format!("{}...", &first_content[..47])
            } else {
                first_content
            }
        };

        Ok(title)
    }

    /// Generate a comprehensive summary for a session
    pub fn generate_session_summary(&self, session_id: &str) -> SessionResult<SessionSummary> {
        let session = self.get_session(session_id)?;

        let mut summary = SessionSummary {
            session_id: session.id.clone(),
            title: self.generate_session_title(session_id)?,
            duration: session.updated_at.signed_duration_since(session.created_at),
            message_count: session.history.len(),
            topics: Vec::new(),
            files_modified: HashSet::new(),
            tools_used: HashSet::new(),
            last_activity: session.updated_at,
        };

        // Analyze all messages for detailed summary
        for message in &session.history {
            // Extract topics and tools
            for part in &message.parts {
                match part {
                    MessagePart::Text(text) => {
                        let lower_text = text.to_lowercase();
                        if lower_text.contains("error") || lower_text.contains("bug") || lower_text.contains("fix") {
                            summary.topics.push("Bug Fixes".to_string());
                        } else if lower_text.contains("feature") || lower_text.contains("implement") || lower_text.contains("add") {
                            summary.topics.push("Feature Development".to_string());
                        } else if lower_text.contains("refactor") || lower_text.contains("clean") || lower_text.contains("optimize") {
                            summary.topics.push("Code Refactoring".to_string());
                        } else if lower_text.contains("test") || lower_text.contains("testing") {
                            summary.topics.push("Testing".to_string());
                        } else if lower_text.contains("documentation") || lower_text.contains("docs") {
                            summary.topics.push("Documentation".to_string());
                        }
                    }
                    MessagePart::ToolInvocation(invocation) => {
                        summary.tools_used.insert(invocation.tool_name.clone());
                    }
                    MessagePart::ToolResult(result) => {
                        summary.tools_used.insert(result.tool_name.clone());
                    }
                    MessagePart::FileReference(file_ref) => {
                        summary.files_modified.insert(file_ref.path.clone());
                    }
                    _ => {}
                }
            }
        }

        // Remove duplicates from topics
        summary.topics.sort();
        summary.topics.dedup();

        Ok(summary)
    }

    /// Update session title (manual override)
    pub fn update_session_title(&mut self, session_id: &str, title: String) -> SessionResult<()> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        session.name = title;
        session.updated_at = Utc::now();

        Ok(())
    }
}

/// Session summary information
#[derive(Debug, Clone)]
pub struct SessionSummary {
    /// Session ID
    pub session_id: String,
    /// Generated or manual title
    pub title: String,
    /// Total session duration
    pub duration: Duration,
    /// Number of messages in the session
    pub message_count: usize,
    /// Topics discussed (derived from content)
    pub topics: Vec<String>,
    /// Files that were modified or referenced
    pub files_modified: HashSet<std::path::PathBuf>,
    /// Tools that were used
    pub tools_used: HashSet<String>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<Utc>,
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
