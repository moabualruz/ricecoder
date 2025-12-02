//! Conversation history storage and retrieval for spec writing sessions

use crate::models::{ConversationMessage, MessageRole, SpecWritingSession, SpecPhase, ApprovalGate};
use crate::error::SpecError;
use chrono::Utc;
use std::collections::HashMap;

/// Manages conversation history and session lifecycle
#[derive(Debug, Clone)]
pub struct ConversationManager {
    /// In-memory storage of sessions (session_id -> session)
    sessions: HashMap<String, SpecWritingSession>,
}

impl ConversationManager {
    /// Create a new conversation manager
    pub fn new() -> Self {
        ConversationManager {
            sessions: HashMap::new(),
        }
    }

    /// Create a new spec writing session
    pub fn create_session(
        &mut self,
        session_id: String,
        spec_id: String,
    ) -> Result<SpecWritingSession, SpecError> {
        if self.sessions.contains_key(&session_id) {
            return Err(SpecError::ConversationError(
                format!("Session {} already exists", session_id),
            ));
        }

        let now = Utc::now();
        let session = SpecWritingSession {
            id: session_id.clone(),
            spec_id,
            phase: SpecPhase::Discovery,
            conversation_history: vec![],
            approval_gates: vec![],
            created_at: now,
            updated_at: now,
        };

        self.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// Retrieve a session by ID
    pub fn get_session(&self, session_id: &str) -> Result<SpecWritingSession, SpecError> {
        self.sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SpecError::ConversationError(format!("Session {} not found", session_id)))
    }

    /// Add a message to a session's conversation history
    pub fn add_message(
        &mut self,
        session_id: &str,
        message_id: String,
        role: MessageRole,
        content: String,
    ) -> Result<ConversationMessage, SpecError> {
        let session = self.sessions.get_mut(session_id).ok_or_else(|| {
            SpecError::ConversationError(format!("Session {} not found", session_id))
        })?;

        let message = ConversationMessage {
            id: message_id,
            spec_id: session.spec_id.clone(),
            role,
            content,
            timestamp: Utc::now(),
        };

        session.conversation_history.push(message.clone());
        session.updated_at = Utc::now();

        Ok(message)
    }

    /// Get conversation history for a session
    pub fn get_conversation_history(
        &self,
        session_id: &str,
    ) -> Result<Vec<ConversationMessage>, SpecError> {
        let session = self.get_session(session_id)?;
        Ok(session.conversation_history)
    }

    /// Get a specific message from a session
    pub fn get_message(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<ConversationMessage, SpecError> {
        let session = self.get_session(session_id)?;
        session
            .conversation_history
            .iter()
            .find(|m| m.id == message_id)
            .cloned()
            .ok_or_else(|| {
                SpecError::ConversationError(format!("Message {} not found", message_id))
            })
    }

    /// Update a session's phase
    pub fn update_phase(
        &mut self,
        session_id: &str,
        new_phase: SpecPhase,
    ) -> Result<SpecWritingSession, SpecError> {
        let session = self.sessions.get_mut(session_id).ok_or_else(|| {
            SpecError::ConversationError(format!("Session {} not found", session_id))
        })?;

        session.phase = new_phase;
        session.updated_at = Utc::now();

        Ok(session.clone())
    }

    /// Add an approval gate to a session
    pub fn add_approval_gate(
        &mut self,
        session_id: &str,
        gate: ApprovalGate,
    ) -> Result<(), SpecError> {
        let session = self.sessions.get_mut(session_id).ok_or_else(|| {
            SpecError::ConversationError(format!("Session {} not found", session_id))
        })?;

        session.approval_gates.push(gate);
        session.updated_at = Utc::now();

        Ok(())
    }

    /// Approve a phase in a session
    pub fn approve_phase(
        &mut self,
        session_id: &str,
        phase: SpecPhase,
        approved_by: Option<String>,
        feedback: Option<String>,
    ) -> Result<(), SpecError> {
        let session = self.sessions.get_mut(session_id).ok_or_else(|| {
            SpecError::ConversationError(format!("Session {} not found", session_id))
        })?;

        // Find or create the approval gate for this phase
        if let Some(gate) = session.approval_gates.iter_mut().find(|g| g.phase == phase) {
            gate.approved = true;
            gate.approved_at = Some(Utc::now());
            gate.approved_by = approved_by;
            gate.feedback = feedback;
        } else {
            let gate = ApprovalGate {
                phase,
                approved: true,
                approved_at: Some(Utc::now()),
                approved_by,
                feedback,
            };
            session.approval_gates.push(gate);
        }

        session.updated_at = Utc::now();
        Ok(())
    }

    /// Get approval status for a phase
    pub fn get_approval_status(
        &self,
        session_id: &str,
        phase: SpecPhase,
    ) -> Result<bool, SpecError> {
        let session = self.get_session(session_id)?;
        Ok(session
            .approval_gates
            .iter()
            .find(|g| g.phase == phase)
            .map(|g| g.approved)
            .unwrap_or(false))
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> Result<(), SpecError> {
        self.sessions
            .remove(session_id)
            .ok_or_else(|| SpecError::ConversationError(format!("Session {} not found", session_id)))?;
        Ok(())
    }

    /// List all session IDs
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Get the number of messages in a session
    pub fn message_count(&self, session_id: &str) -> Result<usize, SpecError> {
        let session = self.get_session(session_id)?;
        Ok(session.conversation_history.len())
    }

    /// Clear conversation history for a session (but keep the session)
    pub fn clear_history(&mut self, session_id: &str) -> Result<(), SpecError> {
        let session = self.sessions.get_mut(session_id).ok_or_else(|| {
            SpecError::ConversationError(format!("Session {} not found", session_id))
        })?;

        session.conversation_history.clear();
        session.updated_at = Utc::now();

        Ok(())
    }
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut manager = ConversationManager::new();
        let session = manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        assert_eq!(session.id, "session-1");
        assert_eq!(session.spec_id, "spec-1");
        assert_eq!(session.phase, SpecPhase::Discovery);
        assert!(session.conversation_history.is_empty());
        assert!(session.approval_gates.is_empty());
    }

    #[test]
    fn test_create_duplicate_session_fails() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        let result = manager.create_session("session-1".to_string(), "spec-2".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_session() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        let session = manager.get_session("session-1").unwrap();
        assert_eq!(session.id, "session-1");
        assert_eq!(session.spec_id, "spec-1");
    }

    #[test]
    fn test_get_nonexistent_session_fails() {
        let manager = ConversationManager::new();
        let result = manager.get_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_message() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        let message = manager
            .add_message(
                "session-1",
                "msg-1".to_string(),
                MessageRole::User,
                "Hello".to_string(),
            )
            .unwrap();

        assert_eq!(message.id, "msg-1");
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello");
    }

    #[test]
    fn test_add_message_to_nonexistent_session_fails() {
        let mut manager = ConversationManager::new();
        let result = manager.add_message(
            "nonexistent",
            "msg-1".to_string(),
            MessageRole::User,
            "Hello".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_conversation_history() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        manager
            .add_message(
                "session-1",
                "msg-1".to_string(),
                MessageRole::User,
                "Hello".to_string(),
            )
            .unwrap();

        manager
            .add_message(
                "session-1",
                "msg-2".to_string(),
                MessageRole::Assistant,
                "Hi there".to_string(),
            )
            .unwrap();

        let history = manager.get_conversation_history("session-1").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, MessageRole::User);
        assert_eq!(history[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_get_message() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        manager
            .add_message(
                "session-1",
                "msg-1".to_string(),
                MessageRole::User,
                "Hello".to_string(),
            )
            .unwrap();

        let message = manager.get_message("session-1", "msg-1").unwrap();
        assert_eq!(message.id, "msg-1");
        assert_eq!(message.content, "Hello");
    }

    #[test]
    fn test_get_nonexistent_message_fails() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        let result = manager.get_message("session-1", "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_phase() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        let session = manager
            .update_phase("session-1", SpecPhase::Requirements)
            .unwrap();

        assert_eq!(session.phase, SpecPhase::Requirements);
    }

    #[test]
    fn test_add_approval_gate() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        let gate = ApprovalGate {
            phase: SpecPhase::Requirements,
            approved: false,
            approved_at: None,
            approved_by: None,
            feedback: None,
        };

        manager.add_approval_gate("session-1", gate).unwrap();

        let session = manager.get_session("session-1").unwrap();
        assert_eq!(session.approval_gates.len(), 1);
    }

    #[test]
    fn test_approve_phase() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        manager
            .approve_phase(
                "session-1",
                SpecPhase::Requirements,
                Some("reviewer".to_string()),
                Some("Looks good".to_string()),
            )
            .unwrap();

        let approved = manager
            .get_approval_status("session-1", SpecPhase::Requirements)
            .unwrap();

        assert!(approved);
    }

    #[test]
    fn test_get_approval_status() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        let approved = manager
            .get_approval_status("session-1", SpecPhase::Requirements)
            .unwrap();

        assert!(!approved);

        manager
            .approve_phase("session-1", SpecPhase::Requirements, None, None)
            .unwrap();

        let approved = manager
            .get_approval_status("session-1", SpecPhase::Requirements)
            .unwrap();

        assert!(approved);
    }

    #[test]
    fn test_delete_session() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        manager.delete_session("session-1").unwrap();

        let result = manager.get_session("session-1");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_sessions() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();
        manager
            .create_session("session-2".to_string(), "spec-2".to_string())
            .unwrap();

        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"session-1".to_string()));
        assert!(sessions.contains(&"session-2".to_string()));
    }

    #[test]
    fn test_message_count() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        assert_eq!(manager.message_count("session-1").unwrap(), 0);

        manager
            .add_message(
                "session-1",
                "msg-1".to_string(),
                MessageRole::User,
                "Hello".to_string(),
            )
            .unwrap();

        assert_eq!(manager.message_count("session-1").unwrap(), 1);
    }

    #[test]
    fn test_clear_history() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        manager
            .add_message(
                "session-1",
                "msg-1".to_string(),
                MessageRole::User,
                "Hello".to_string(),
            )
            .unwrap();

        assert_eq!(manager.message_count("session-1").unwrap(), 1);

        manager.clear_history("session-1").unwrap();

        assert_eq!(manager.message_count("session-1").unwrap(), 0);
    }

    #[test]
    fn test_session_lifecycle() {
        let mut manager = ConversationManager::new();

        // Create session
        let session = manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();
        assert_eq!(session.phase, SpecPhase::Discovery);

        // Add messages
        manager
            .add_message(
                "session-1",
                "msg-1".to_string(),
                MessageRole::User,
                "Create a task system".to_string(),
            )
            .unwrap();

        // Update phase
        manager
            .update_phase("session-1", SpecPhase::Requirements)
            .unwrap();

        // Approve phase
        manager
            .approve_phase("session-1", SpecPhase::Requirements, None, None)
            .unwrap();

        // Verify final state
        let session = manager.get_session("session-1").unwrap();
        assert_eq!(session.phase, SpecPhase::Requirements);
        assert_eq!(session.conversation_history.len(), 1);
        assert_eq!(session.approval_gates.len(), 1);
        assert!(session.approval_gates[0].approved);
    }

    #[test]
    fn test_conversation_history_preservation() {
        let mut manager = ConversationManager::new();
        manager
            .create_session("session-1".to_string(), "spec-1".to_string())
            .unwrap();

        // Add multiple messages
        let messages = vec![
            ("msg-1", MessageRole::User, "Hello"),
            ("msg-2", MessageRole::Assistant, "Hi"),
            ("msg-3", MessageRole::User, "How are you?"),
            ("msg-4", MessageRole::Assistant, "I'm good"),
        ];

        for (id, role, content) in messages {
            manager
                .add_message("session-1", id.to_string(), role, content.to_string())
                .unwrap();
        }

        // Retrieve and verify
        let history = manager.get_conversation_history("session-1").unwrap();
        assert_eq!(history.len(), 4);

        // Verify order is preserved
        assert_eq!(history[0].id, "msg-1");
        assert_eq!(history[1].id, "msg-2");
        assert_eq!(history[2].id, "msg-3");
        assert_eq!(history[3].id, "msg-4");

        // Verify roles are preserved
        assert_eq!(history[0].role, MessageRole::User);
        assert_eq!(history[1].role, MessageRole::Assistant);
        assert_eq!(history[2].role, MessageRole::User);
        assert_eq!(history[3].role, MessageRole::Assistant);
    }
}
