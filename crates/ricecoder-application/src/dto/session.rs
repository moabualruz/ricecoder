//! Session-related DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ricecoder_domain::session::{Session, SessionState, MessageRole};
use ricecoder_domain::value_objects::ProjectId;

/// Command to create a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionCommand {
    pub project_id: String,
    pub max_messages: Option<usize>,
}

/// Command to add a message to a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMessageCommand {
    pub session_id: String,
    pub content: String,
    pub role: String,
}

/// Session summary DTO (list view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryDto {
    pub id: String,
    pub project_id: String,
    pub state: String,
    pub message_count: usize,
    pub created_at: DateTime<Utc>,
}

impl SessionSummaryDto {
    /// Create from domain aggregate
    pub fn from_domain(session: &Session) -> Self {
        Self {
            id: session.id().to_string(),
            project_id: session.project_id().to_string(),
            state: format!("{:?}", session.state()),
            message_count: session.message_count(),
            created_at: session.created_at(),
        }
    }
}

/// Session detail DTO (single view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailDto {
    pub id: String,
    pub project_id: String,
    pub state: String,
    pub messages: Vec<MessageDto>,
    pub max_messages: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl SessionDetailDto {
    /// Create from domain aggregate
    pub fn from_domain(session: &Session) -> Self {
        Self {
            id: session.id().to_string(),
            project_id: session.project_id().to_string(),
            state: format!("{:?}", session.state()),
            messages: session
                .messages()
                .iter()
                .map(|m| MessageDto {
                    id: m.id().to_string(),
                    content: m.content().to_string(),
                    role: format!("{:?}", m.role()),
                    created_at: m.created_at(),
                })
                .collect(),
            max_messages: session.max_messages(),
            created_at: session.created_at(),
            updated_at: session.updated_at(),
            is_active: session.is_active(),
        }
    }
}

/// Message DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDto {
    pub id: String,
    pub content: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session_command_serialization() {
        let cmd = CreateSessionCommand {
            project_id: "proj-123".into(),
            max_messages: Some(100),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: CreateSessionCommand = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.project_id, "proj-123");
        assert_eq!(parsed.max_messages, Some(100));
    }
}
