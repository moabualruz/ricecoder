//! PTY session events for reactive patterns

use serde::{Deserialize, Serialize};

/// Events emitted by PTY sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEvent {
    /// Session was created
    Created {
        /// Session ID
        session_id: String,
    },
    /// Session produced output
    Output {
        /// Session ID
        session_id: String,
        /// Output data
        data: String,
    },
    /// Session title changed
    TitleChanged {
        /// Session ID
        session_id: String,
        /// New title
        title: String,
    },
    /// Session was resized
    Resized {
        /// Session ID
        session_id: String,
        /// New width in columns
        cols: u16,
        /// New height in rows
        rows: u16,
    },
    /// Session exited
    Exited {
        /// Session ID
        session_id: String,
        /// Exit code (if available)
        exit_code: Option<i32>,
    },
    /// Session was closed/destroyed
    Closed {
        /// Session ID
        session_id: String,
    },
}

impl SessionEvent {
    /// Get the session ID associated with this event
    pub fn session_id(&self) -> &str {
        match self {
            Self::Created { session_id } => session_id,
            Self::Output { session_id, .. } => session_id,
            Self::TitleChanged { session_id, .. } => session_id,
            Self::Resized { session_id, .. } => session_id,
            Self::Exited { session_id, .. } => session_id,
            Self::Closed { session_id } => session_id,
        }
    }

    /// Check if this is an output event
    pub fn is_output(&self) -> bool {
        matches!(self, Self::Output { .. })
    }

    /// Check if this is a terminal event (exit or close)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Exited { .. } | Self::Closed { .. })
    }
}
