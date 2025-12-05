//! Sessions command - Manage ricecoder sessions

use crate::commands::Command;
use crate::error::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Sessions command action
#[derive(Debug, Clone)]
pub enum SessionsAction {
    /// List all sessions
    List,
    /// Create a new session
    Create { name: String },
    /// Delete a session
    Delete { id: String },
    /// Rename a session
    Rename { id: String, name: String },
    /// Switch to a session
    Switch { id: String },
    /// Show session info
    Info { id: String },
}

/// Session data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Number of messages
    pub message_count: usize,
}

/// Sessions command handler
pub struct SessionsCommand {
    action: SessionsAction,
}

impl SessionsCommand {
    /// Create a new sessions command
    pub fn new(action: SessionsAction) -> Self {
        Self { action }
    }

    /// Get the sessions directory
    fn sessions_dir() -> CliResult<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| CliError::Internal("Could not determine home directory".to_string()))?;
        let sessions_dir = home.join(".ricecoder").join("sessions");

        // Create directory if it doesn't exist
        fs::create_dir_all(&sessions_dir).map_err(|e| {
            CliError::Internal(format!("Failed to create sessions directory: {}", e))
        })?;

        Ok(sessions_dir)
    }

    /// Get the sessions index file
    fn sessions_index() -> CliResult<PathBuf> {
        let sessions_dir = Self::sessions_dir()?;
        Ok(sessions_dir.join("index.json"))
    }

    /// Load all sessions from index
    fn load_sessions() -> CliResult<Vec<SessionInfo>> {
        let index_path = Self::sessions_index()?;

        if !index_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&index_path)
            .map_err(|e| CliError::Internal(format!("Failed to read sessions index: {}", e)))?;

        // Handle empty file
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let sessions: Vec<SessionInfo> = serde_json::from_str(&content)
            .map_err(|e| CliError::Internal(format!("Failed to parse sessions index: {}", e)))?;

        Ok(sessions)
    }

    /// Save sessions to index
    fn save_sessions(sessions: &[SessionInfo]) -> CliResult<()> {
        let index_path = Self::sessions_index()?;

        let content = serde_json::to_string_pretty(sessions)
            .map_err(|e| CliError::Internal(format!("Failed to serialize sessions: {}", e)))?;

        fs::write(&index_path, content)
            .map_err(|e| CliError::Internal(format!("Failed to write sessions index: {}", e)))?;

        Ok(())
    }
}

impl Command for SessionsCommand {
    fn execute(&self) -> CliResult<()> {
        match &self.action {
            SessionsAction::List => list_sessions(),
            SessionsAction::Create { name } => create_session(name),
            SessionsAction::Delete { id } => delete_session(id),
            SessionsAction::Rename { id, name } => rename_session(id, name),
            SessionsAction::Switch { id } => switch_session(id),
            SessionsAction::Info { id } => show_session_info(id),
        }
    }
}

/// List all sessions
fn list_sessions() -> CliResult<()> {
    let sessions = SessionsCommand::load_sessions()?;

    if sessions.is_empty() {
        println!("No sessions found. Create one with: rice sessions create <name>");
        return Ok(());
    }

    println!("Sessions:");
    println!();

    for session in sessions {
        println!("  {} - {}", session.id, session.name);
        println!("    Messages: {}", session.message_count);
        println!("    Created: {}", format_timestamp(session.created_at));
        println!("    Modified: {}", format_timestamp(session.modified_at));
        println!();
    }

    Ok(())
}

/// Create a new session
fn create_session(name: &str) -> CliResult<()> {
    let mut sessions = SessionsCommand::load_sessions()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let id = format!("session-{}", now);

    let session = SessionInfo {
        id: id.clone(),
        name: name.to_string(),
        created_at: now,
        modified_at: now,
        message_count: 0,
    };

    sessions.push(session);
    SessionsCommand::save_sessions(&sessions)?;

    println!("Created session: {} ({})", id, name);
    Ok(())
}

/// Delete a session
fn delete_session(id: &str) -> CliResult<()> {
    let mut sessions = SessionsCommand::load_sessions()?;

    let initial_len = sessions.len();
    sessions.retain(|s| s.id != id);

    if sessions.len() == initial_len {
        return Err(CliError::Internal(format!("Session not found: {}", id)));
    }

    SessionsCommand::save_sessions(&sessions)?;
    println!("Deleted session: {}", id);
    Ok(())
}

/// Rename a session
fn rename_session(id: &str, name: &str) -> CliResult<()> {
    let mut sessions = SessionsCommand::load_sessions()?;

    let session = sessions
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| CliError::Internal(format!("Session not found: {}", id)))?;

    let old_name = session.name.clone();
    session.name = name.to_string();
    session.modified_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    SessionsCommand::save_sessions(&sessions)?;
    println!("Renamed session from '{}' to '{}'", old_name, name);
    Ok(())
}

/// Switch to a session
fn switch_session(id: &str) -> CliResult<()> {
    let sessions = SessionsCommand::load_sessions()?;

    let session = sessions
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| CliError::Internal(format!("Session not found: {}", id)))?;

    // Store current session in config
    let config_path = dirs::home_dir()
        .ok_or_else(|| CliError::Internal("Could not determine home directory".to_string()))?
        .join(".ricecoder")
        .join("current_session.txt");

    fs::write(&config_path, &session.id)
        .map_err(|e| CliError::Internal(format!("Failed to save current session: {}", e)))?;

    println!("Switched to session: {} ({})", session.id, session.name);
    Ok(())
}

/// Show session info
fn show_session_info(id: &str) -> CliResult<()> {
    let sessions = SessionsCommand::load_sessions()?;

    let session = sessions
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| CliError::Internal(format!("Session not found: {}", id)))?;

    println!("Session: {}", session.id);
    println!("  Name: {}", session.name);
    println!("  Messages: {}", session.message_count);
    println!("  Created: {}", format_timestamp(session.created_at));
    println!("  Modified: {}", format_timestamp(session.modified_at));
    Ok(())
}

/// Format timestamp as human-readable string
fn format_timestamp(secs: u64) -> String {
    use std::time::UNIX_EPOCH;

    let duration = std::time::Duration::from_secs(secs);
    let datetime = UNIX_EPOCH + duration;

    // Simple formatting - just show seconds since epoch for now
    // In production, use chrono or similar
    format!(
        "{} seconds ago",
        std::time::SystemTime::now()
            .duration_since(datetime)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sessions_command_creation() {
        let cmd = SessionsCommand::new(SessionsAction::List);
        assert!(matches!(cmd.action, SessionsAction::List));
    }

    #[test]
    fn test_create_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Create {
            name: "test".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Create { .. }));
    }

    #[test]
    fn test_delete_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Delete {
            id: "session-1".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Delete { .. }));
    }

    #[test]
    fn test_session_info_serialization() {
        let session = SessionInfo {
            id: "session-1".to_string(),
            name: "Test Session".to_string(),
            created_at: 1000,
            modified_at: 2000,
            message_count: 5,
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: SessionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.name, deserialized.name);
        assert_eq!(session.message_count, deserialized.message_count);
    }

    #[test]
    fn test_rename_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Rename {
            id: "session-1".to_string(),
            name: "New Name".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Rename { .. }));
    }

    #[test]
    fn test_switch_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Switch {
            id: "session-1".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Switch { .. }));
    }

    #[test]
    fn test_info_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Info {
            id: "session-1".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Info { .. }));
    }
}
