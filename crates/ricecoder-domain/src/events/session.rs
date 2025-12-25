//! Session aggregate domain events
//!
//! Session aggregate events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DomainEvent, EventMetadata};

/// Event emitted when a session is started
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionStarted {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Session aggregate ID
    pub session_id: Uuid,
    
    /// Session title
    pub title: String,
    
    /// AI provider ID
    pub provider_id: String,
}

impl SessionStarted {
    /// Create new SessionStarted event
    pub fn new(session_id: Uuid, title: String, provider_id: String) -> Self {
        Self {
            metadata: EventMetadata::new(),
            session_id,
            title,
            provider_id,
        }
    }
}

impl DomainEvent for SessionStarted {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SessionStarted"
    }
}

/// Event emitted when a message is added to a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageAdded {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Session aggregate ID
    pub session_id: Uuid,
    
    /// Message ID
    pub message_id: Uuid,
    
    /// Message role (user, assistant, system)
    pub role: String,
    
    /// Message content
    pub content: String,
}

impl MessageAdded {
    /// Create new MessageAdded event
    pub fn new(session_id: Uuid, message_id: Uuid, role: String, content: String) -> Self {
        Self {
            metadata: EventMetadata::new(),
            session_id,
            message_id,
            role,
            content,
        }
    }
}

impl DomainEvent for MessageAdded {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "MessageAdded"
    }
}

/// Event emitted when a session is paused
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionPaused {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Session aggregate ID
    pub session_id: Uuid,
}

impl SessionPaused {
    /// Create new SessionPaused event
    pub fn new(session_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            session_id,
        }
    }
}

impl DomainEvent for SessionPaused {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SessionPaused"
    }
}

/// Event emitted when a session is resumed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionResumed {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Session aggregate ID
    pub session_id: Uuid,
}

impl SessionResumed {
    /// Create new SessionResumed event
    pub fn new(session_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            session_id,
        }
    }
}

impl DomainEvent for SessionResumed {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SessionResumed"
    }
}

/// Event emitted when a session is completed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionCompleted {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Session aggregate ID
    pub session_id: Uuid,
    
    /// Number of messages in session
    pub message_count: usize,
}

impl SessionCompleted {
    /// Create new SessionCompleted event
    pub fn new(session_id: Uuid, message_count: usize) -> Self {
        Self {
            metadata: EventMetadata::new(),
            session_id,
            message_count,
        }
    }
}

impl DomainEvent for SessionCompleted {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SessionCompleted"
    }
}

/// Event emitted when a session is archived
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionArchived {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Session aggregate ID
    pub session_id: Uuid,
}

impl SessionArchived {
    /// Create new SessionArchived event
    pub fn new(session_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            session_id,
        }
    }
}

impl DomainEvent for SessionArchived {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SessionArchived"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_started_event() {
        let session_id = Uuid::new_v4();
        let event = SessionStarted::new(
            session_id,
            "Test Session".to_string(),
            "openai".to_string(),
        );

        assert_eq!(event.aggregate_id(), session_id);
        assert_eq!(event.title, "Test Session");
        assert_eq!(event.provider_id, "openai");
        assert_eq!(event.event_type(), "SessionStarted");
    }

    #[test]
    fn test_message_added_event() {
        let session_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let event = MessageAdded::new(
            session_id,
            message_id,
            "user".to_string(),
            "Hello".to_string(),
        );

        assert_eq!(event.aggregate_id(), session_id);
        assert_eq!(event.message_id, message_id);
        assert_eq!(event.role, "user");
        assert_eq!(event.content, "Hello");
        assert_eq!(event.event_type(), "MessageAdded");
    }

    #[test]
    fn test_session_paused_event() {
        let session_id = Uuid::new_v4();
        let event = SessionPaused::new(session_id);

        assert_eq!(event.aggregate_id(), session_id);
        assert_eq!(event.event_type(), "SessionPaused");
    }

    #[test]
    fn test_session_resumed_event() {
        let session_id = Uuid::new_v4();
        let event = SessionResumed::new(session_id);

        assert_eq!(event.aggregate_id(), session_id);
        assert_eq!(event.event_type(), "SessionResumed");
    }

    #[test]
    fn test_session_completed_event() {
        let session_id = Uuid::new_v4();
        let event = SessionCompleted::new(session_id, 42);

        assert_eq!(event.aggregate_id(), session_id);
        assert_eq!(event.message_count, 42);
        assert_eq!(event.event_type(), "SessionCompleted");
    }

    #[test]
    fn test_session_archived_event() {
        let session_id = Uuid::new_v4();
        let event = SessionArchived::new(session_id);

        assert_eq!(event.aggregate_id(), session_id);
        assert_eq!(event.event_type(), "SessionArchived");
    }

    #[test]
    fn test_event_serialization() {
        let session_id = Uuid::new_v4();
        let event = SessionStarted::new(
            session_id,
            "Test".to_string(),
            "openai".to_string(),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SessionStarted = serde_json::from_str(&json).unwrap();

        assert_eq!(event.session_id, deserialized.session_id);
        assert_eq!(event.title, deserialized.title);
    }
}
