//! Session Aggregate Root
//!
//! Session aggregate with full DDD compliance
//! - Aggregate root with encapsulated message entities
//! - Invariant enforcement (state machine, message limits)
//! - Domain event emission
//! - Immutable identity

use crate::errors::{DomainError, DomainResult};
use crate::events::session::*;
use crate::events::DomainEvent;
use crate::value_objects::{ProjectId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session Aggregate Root
///
/// Manages AI conversation state, message history, and context for a project.
/// All operations emit domain events for auditability and event sourcing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Immutable identity
    id: SessionId,

    /// Parent project reference
    project_id: ProjectId,

    /// Ordered message history
    messages: Vec<Message>,

    /// Session lifecycle status
    state: SessionState,

    /// Maximum message limit for this session
    max_messages: usize,

    /// Creation timestamp (immutable)
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,

    /// Version for optimistic concurrency control
    version: u64,
}

/// Session lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Active,
    Paused,
    Completed,
    Archived,
}

/// Message entity within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message identity
    id: String,

    /// Message content
    content: String,

    /// Message role (User, Assistant, System)
    role: MessageRole,

    /// Creation timestamp
    created_at: DateTime<Utc>,
}

/// Message role enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl Message {
    /// Create new message
    fn new(content: String, role: MessageRole) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            role,
            created_at: Utc::now(),
        }
    }

    /// Reconstitute message from persistence
    pub fn reconstitute(
        id: String,
        content: String,
        role: MessageRole,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            content,
            role,
            created_at,
        }
    }

    /// Get message ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get message content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get message role
    pub fn role(&self) -> MessageRole {
        self.role
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl Session {
    /// Create new session with invariant validation
    ///
    /// # Invariants
    /// - Project ID must exist (validated at application layer)
    /// - Initial state is always Active
    ///
    /// # Returns
    /// Tuple of (Session, Vec<DomainEvent>) for event sourcing
    ///
    /// # Errors
    /// Returns DomainError if invariants violated
    pub fn create(
        project_id: ProjectId,
        max_messages: usize,
    ) -> DomainResult<(Self, Vec<Box<dyn DomainEvent>>)> {
        // Enforce immutable SessionId
        let id = SessionId::new();
        let now = Utc::now();

        let session = Self {
            id,
            project_id,
            messages: Vec::new(),
            state: SessionState::Active,
            max_messages,
            created_at: now,
            updated_at: now,
            version: 1,
        };

        // Emit domain events
        let event = SessionStarted::new(
            id.as_uuid(),
            format!("Session for project {}", project_id.to_string()),
            "default".into(),
        );
        let events: Vec<Box<dyn DomainEvent>> = vec![Box::new(event)];

        Ok((session, events))
    }

    /// Reconstitute a session from persistence
    ///
    /// Bypasses validation since data was validated during original creation.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SessionId,
        project_id: ProjectId,
        messages: Vec<Message>,
        state: SessionState,
        max_messages: usize,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        version: u64,
    ) -> Self {
        Self {
            id,
            project_id,
            messages,
            state,
            max_messages,
            created_at,
            updated_at,
            version,
        }
    }

    /// Add message to session
    ///
    /// # Invariants
    /// - Session must be in Active state
    /// - Message count must not exceed limit
    /// - Updates timestamp on success
    /// - Emits MessageAdded event
    pub fn add_message(
        &mut self,
        content: String,
        role: MessageRole,
    ) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        // Enforce active state only
        if self.state != SessionState::Active {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!("Cannot add message to {:?} session", self.state),
            });
        }

        // Enforce message limit
        if self.messages.len() >= self.max_messages {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!("Session message limit reached: {}", self.max_messages),
            });
        }

        let message = Message::new(content.clone(), role);
        let message_id_str = message.id().to_string();
        let message_uuid = uuid::Uuid::parse_str(&message_id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());

        self.messages.push(message);
        self.updated_at = Utc::now();
        self.version += 1;

        // Emit domain event
        let role_str = match role {
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
            MessageRole::System => "system".to_string(),
        };
        let event = MessageAdded::new(self.id.as_uuid(), message_uuid, role_str, content);
        Ok(vec![Box::new(event)])
    }

    /// Pause session
    ///
    /// # Business Rules
    /// - Only active sessions can be paused
    pub fn pause(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.state != SessionState::Active {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Only active sessions can be paused".into(),
            });
        }

        self.state = SessionState::Paused;
        self.updated_at = Utc::now();
        self.version += 1;

        let event = SessionPaused::new(self.id.as_uuid());
        Ok(vec![Box::new(event)])
    }

    /// Resume paused session
    ///
    /// # Business Rules
    /// - Only paused sessions can be resumed
    pub fn resume(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.state != SessionState::Paused {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Only paused sessions can be resumed".into(),
            });
        }

        self.state = SessionState::Active;
        self.updated_at = Utc::now();
        self.version += 1;

        let event = SessionResumed::new(self.id.as_uuid());
        Ok(vec![Box::new(event)])
    }

    /// Complete session
    ///
    /// # Business Rules
    /// - Only active or paused sessions can be completed
    pub fn complete(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.state == SessionState::Completed || self.state == SessionState::Archived {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!("Cannot complete {:?} session", self.state),
            });
        }

        self.state = SessionState::Completed;
        self.updated_at = Utc::now();
        self.version += 1;

        let event = SessionCompleted::new(self.id.as_uuid(), self.messages.len());
        Ok(vec![Box::new(event)])
    }

    /// Archive session
    ///
    /// # Business Rules
    /// - Only completed sessions can be archived
    pub fn archive(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.state != SessionState::Completed {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Only completed sessions can be archived".into(),
            });
        }

        self.state = SessionState::Archived;
        self.updated_at = Utc::now();
        self.version += 1;

        let event = SessionArchived::new(self.id.as_uuid());
        Ok(vec![Box::new(event)])
    }

    // === Getters (Prevent direct access) ===

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn max_messages(&self) -> usize {
        self.max_messages
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn is_active(&self) -> bool {
        self.state == SessionState::Active
    }

    pub fn is_paused(&self) -> bool {
        self.state == SessionState::Paused
    }

    pub fn is_completed(&self) -> bool {
        self.state == SessionState::Completed
    }

    pub fn is_archived(&self) -> bool {
        self.state == SessionState::Archived
    }

    /// Get version for optimistic concurrency control
    pub fn version(&self) -> u64 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session_success() {
        let project_id = ProjectId::new();
        let (session, events) = Session::create(project_id, 100).unwrap();

        assert_eq!(session.project_id(), project_id);
        assert_eq!(session.state(), SessionState::Active);
        assert_eq!(session.message_count(), 0);
        assert_eq!(session.max_messages(), 100);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "SessionStarted");
    }

    #[test]
    fn test_add_message_success() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        let events = session
            .add_message("Hello".into(), MessageRole::User)
            .unwrap();

        assert_eq!(session.message_count(), 1);
        assert_eq!(session.messages()[0].content(), "Hello");
        assert_eq!(session.messages()[0].role(), MessageRole::User);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "MessageAdded");
    }

    #[test]
    fn test_add_message_to_paused_session_fails() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        session.pause().unwrap();
        let result = session.add_message("Hello".into(), MessageRole::User);

        assert!(result.is_err());
    }

    #[test]
    fn test_add_message_exceeds_limit_fails() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 1).unwrap();

        session
            .add_message("First".into(), MessageRole::User)
            .unwrap();
        let result = session.add_message("Second".into(), MessageRole::User);

        assert!(result.is_err());
    }

    #[test]
    fn test_pause_and_resume() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        assert!(session.is_active());

        session.pause().unwrap();
        assert!(session.is_paused());

        session.resume().unwrap();
        assert!(session.is_active());
    }

    #[test]
    fn test_cannot_pause_paused_session() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        session.pause().unwrap();
        let result = session.pause();

        assert!(result.is_err());
    }

    #[test]
    fn test_complete_session() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        session
            .add_message("Hello".into(), MessageRole::User)
            .unwrap();

        let events = session.complete().unwrap();
        assert!(session.is_completed());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "SessionCompleted");
    }

    #[test]
    fn test_archive_completed_session() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        session.complete().unwrap();
        let events = session.archive().unwrap();

        assert!(session.is_archived());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "SessionArchived");
    }

    #[test]
    fn test_cannot_archive_active_session() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        let result = session.archive();
        assert!(result.is_err());
    }

    #[test]
    fn test_message_entity_creation() {
        let message = Message::new("Test content".into(), MessageRole::Assistant);

        assert_eq!(message.content(), "Test content");
        assert_eq!(message.role(), MessageRole::Assistant);
        assert!(!message.id().is_empty());
    }

    #[test]
    fn test_multiple_messages_maintained_in_order() {
        let project_id = ProjectId::new();
        let (mut session, _) = Session::create(project_id, 100).unwrap();

        session
            .add_message("First".into(), MessageRole::User)
            .unwrap();
        session
            .add_message("Second".into(), MessageRole::Assistant)
            .unwrap();
        session
            .add_message("Third".into(), MessageRole::User)
            .unwrap();

        assert_eq!(session.messages().len(), 3);
        assert_eq!(session.messages()[0].content(), "First");
        assert_eq!(session.messages()[1].content(), "Second");
        assert_eq!(session.messages()[2].content(), "Third");
    }

    #[test]
    fn test_session_timestamp_updates() {
        let project_id = ProjectId::new();
        let (session, _) = Session::create(project_id, 100).unwrap();
        let initial_updated_at = session.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let mut session = session;
        session
            .add_message("Hello".into(), MessageRole::User)
            .unwrap();

        assert!(session.updated_at() > initial_updated_at);
    }
}
