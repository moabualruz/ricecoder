//! Event types for the session event bus
//!
//! This module defines all event types that can be published and subscribed to
//! through the event bus. Events are used to notify components about session lifecycle
//! changes, message updates, and tool execution status.

use serde::{Deserialize, Serialize};

/// Session lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionEvent {
    /// Session was created
    Created {
        /// Unique session identifier
        session_id: String,
    },
    /// Session was updated
    Updated {
        /// Unique session identifier
        session_id: String,
    },
    /// Session was deleted
    Deleted {
        /// Unique session identifier
        session_id: String,
    },
    /// Session encountered an error
    Error {
        /// Unique session identifier
        session_id: String,
        /// Error message
        error: String,
    },
    /// Session status changed
    StatusChanged {
        /// Unique session identifier
        session_id: String,
        /// New status
        status: String,
    },
    /// Session became idle
    Idle {
        /// Unique session identifier
        session_id: String,
    },
}

/// Message-related events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageEvent {
    /// Message was updated
    Updated {
        /// Message identifier
        message_id: String,
        /// Session identifier
        session_id: String,
    },
    /// Message part was updated (streaming)
    PartUpdated {
        /// Message identifier
        message_id: String,
        /// Part index
        part_index: usize,
        /// Optional delta for streaming updates
        delta: Option<String>,
    },
    /// Message part was removed
    PartRemoved {
        /// Message identifier
        message_id: String,
        /// Part index
        part_index: usize,
    },
}

/// Tool execution events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolEvent {
    /// Tool execution started
    Started {
        /// Tool call identifier
        call_id: String,
        /// Tool name
        tool_name: String,
    },
    /// Tool execution completed
    Completed {
        /// Tool call identifier
        call_id: String,
        /// Tool name
        tool_name: String,
    },
    /// Tool execution failed
    Error {
        /// Tool call identifier
        call_id: String,
        /// Tool name
        tool_name: String,
        /// Error message
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_event_serialization() {
        let event = SessionEvent::Created {
            session_id: "test-123".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SessionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_message_event_serialization() {
        let event = MessageEvent::PartUpdated {
            message_id: "msg-123".to_string(),
            part_index: 0,
            delta: Some("hello".to_string()),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MessageEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_tool_event_serialization() {
        let event = ToolEvent::Error {
            call_id: "call-123".to_string(),
            tool_name: "grep".to_string(),
            error: "not found".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ToolEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }
}
