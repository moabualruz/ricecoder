//! Runtime session state management (ephemeral busy/idle/retry state)
//!
//! Implements OpenCode-compatible session status model with in-memory state tracking
//! for coordinating prompt loops and handling retry scenarios.
//!
//! SSTATE-005: Session status model parity (busy/retry/idle)

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{SessionError, SessionResult};

/// Runtime session status (ephemeral, in-memory only)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum RuntimeStatus {
    /// Session is idle (ready for new prompts)
    #[serde(rename = "idle")]
    Idle,
    
    /// Session is busy processing
    #[serde(rename = "busy")]
    Busy,
    
    /// Session is in retry state with backoff
    #[serde(rename = "retry")]
    Retry {
        /// Current retry attempt number
        attempt: u32,
        /// User-facing retry message
        message: String,
        /// Next retry time (milliseconds since epoch)
        next: i64,
    },
}

impl Default for RuntimeStatus {
    fn default() -> Self {
        RuntimeStatus::Idle
    }
}

/// Runtime state manager for session coordination
///
/// This is an in-memory state tracker that complements the persisted SessionStatus
/// (Active/Paused/Archived). It handles ephemeral workflow coordination.
#[derive(Debug, Clone)]
pub struct RuntimeStateManager {
    /// In-memory state map (sessionID -> RuntimeStatus)
    state: Arc<Mutex<HashMap<String, RuntimeStatus>>>,
    
    /// Optional event publisher (for TUI updates)
    event_tx: Arc<Mutex<Option<tokio::sync::broadcast::Sender<RuntimeStateEvent>>>>,
}

/// Runtime state change events
#[derive(Debug, Clone)]
pub struct RuntimeStateEvent {
    pub session_id: String,
    pub status: RuntimeStatus,
    pub timestamp: DateTime<Utc>,
}

impl RuntimeStateManager {
    /// Create a new runtime state manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            event_tx: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Create with event publisher for TUI integration
    pub fn with_events(event_tx: tokio::sync::broadcast::Sender<RuntimeStateEvent>) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            event_tx: Arc::new(Mutex::new(Some(event_tx))),
        }
    }
    
    /// Get runtime status for a session
    pub fn get(&self, session_id: &str) -> RuntimeStatus {
        let state = self.state.lock().unwrap();
        state.get(session_id).cloned().unwrap_or(RuntimeStatus::Idle)
    }
    
    /// Set runtime status for a session
    pub fn set(&self, session_id: &str, status: RuntimeStatus) {
        let mut state = self.state.lock().unwrap();
        
        // Publish event before updating state
        let event = RuntimeStateEvent {
            session_id: session_id.to_string(),
            status: status.clone(),
            timestamp: Utc::now(),
        };
        
        if let Some(tx) = self.event_tx.lock().unwrap().as_ref() {
            let _ = tx.send(event); // Ignore send errors (no subscribers is OK)
        }
        
        // Update state (delete idle entries to save memory)
        match status {
            RuntimeStatus::Idle => {
                state.remove(session_id);
            }
            _ => {
                state.insert(session_id.to_string(), status);
            }
        }
    }
    
    /// Check if session is idle
    pub fn is_idle(&self, session_id: &str) -> bool {
        matches!(self.get(session_id), RuntimeStatus::Idle)
    }
    
    /// Check if session is busy
    pub fn is_busy(&self, session_id: &str) -> bool {
        matches!(self.get(session_id), RuntimeStatus::Busy)
    }
    
    /// Check if session is in retry state
    pub fn is_retrying(&self, session_id: &str) -> bool {
        matches!(self.get(session_id), RuntimeStatus::Retry { .. })
    }
    
    /// Assert session is not busy (for operations that require idle state)
    pub fn assert_not_busy(&self, session_id: &str) -> SessionResult<()> {
        if self.is_busy(session_id) {
            Err(SessionError::SessionBusy(session_id.to_string()))
        } else {
            Ok(())
        }
    }
    
    /// List all sessions with non-idle status
    pub fn list_active(&self) -> Vec<(String, RuntimeStatus)> {
        let state = self.state.lock().unwrap();
        state.iter()
            .map(|(id, status)| (id.clone(), status.clone()))
            .collect()
    }
    
    /// Clear all runtime state
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.clear();
    }
}

impl Default for RuntimeStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_idle() {
        let manager = RuntimeStateManager::new();
        assert!(manager.is_idle("session-1"));
        assert!(!manager.is_busy("session-1"));
    }
    
    #[test]
    fn test_set_busy() {
        let manager = RuntimeStateManager::new();
        manager.set("session-1", RuntimeStatus::Busy);
        assert!(manager.is_busy("session-1"));
        assert!(!manager.is_idle("session-1"));
    }
    
    #[test]
    fn test_set_retry() {
        let manager = RuntimeStateManager::new();
        let next_time = Utc::now().timestamp_millis() + 5000;
        manager.set("session-1", RuntimeStatus::Retry {
            attempt: 1,
            message: "Rate limited".to_string(),
            next: next_time,
        });
        assert!(manager.is_retrying("session-1"));
    }
    
    #[test]
    fn test_set_idle_removes_entry() {
        let manager = RuntimeStateManager::new();
        manager.set("session-1", RuntimeStatus::Busy);
        manager.set("session-1", RuntimeStatus::Idle);
        
        let state = manager.state.lock().unwrap();
        assert!(!state.contains_key("session-1"));
    }
    
    #[test]
    fn test_assert_not_busy() {
        let manager = RuntimeStateManager::new();
        assert!(manager.assert_not_busy("session-1").is_ok());
        
        manager.set("session-1", RuntimeStatus::Busy);
        assert!(manager.assert_not_busy("session-1").is_err());
        
        manager.set("session-1", RuntimeStatus::Idle);
        assert!(manager.assert_not_busy("session-1").is_ok());
    }
    
    #[test]
    fn test_list_active() {
        let manager = RuntimeStateManager::new();
        manager.set("session-1", RuntimeStatus::Busy);
        manager.set("session-2", RuntimeStatus::Retry {
            attempt: 1,
            message: "Retrying".to_string(),
            next: Utc::now().timestamp_millis() + 1000,
        });
        manager.set("session-3", RuntimeStatus::Idle);
        
        let active = manager.list_active();
        assert_eq!(active.len(), 2); // Only busy and retry, not idle
    }
}
