//! Sessions command - Manage ricecoder sessions

use crate::commands::Command;
use crate::error::{CliError, CliResult};

fn format_timestamp(ts: i64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(|| DateTime::<Utc>::UNIX_EPOCH);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}
use ricecoder_sessions::{SessionContext, SessionMode, SharePermissions};
use ricecoder_agents::use_cases::{SessionLifecycleUseCase, SessionSharingUseCase};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

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
    /// Share a session with a shareable link
    Share {
        expires_in: Option<u64>,
        no_history: bool,
        no_context: bool,
    },
    /// List all active shares
    ShareList,
    /// Revoke a share
    ShareRevoke { share_id: String },
    /// Show share information
    ShareInfo { share_id: String },
    /// View a shared session
    ShareView { share_id: String },
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

    /// Get session use cases from DI container
    fn get_use_cases(&self) -> CliResult<(Arc<SessionLifecycleUseCase>, Arc<SessionSharingUseCase>)> {
        // Get services from DI container
        let session_lifecycle = crate::di::get_service::<SessionLifecycleUseCase>()
            .ok_or_else(|| CliError::Internal("SessionLifecycleUseCase not available in DI container".to_string()))?;

        let session_sharing = crate::di::get_service::<SessionSharingUseCase>()
            .ok_or_else(|| CliError::Internal("SessionSharingUseCase not available in DI container".to_string()))?;

        Ok((session_lifecycle, session_sharing))
    }



    /// List all sessions
    fn list_sessions(&self) -> CliResult<()> {
        let (session_lifecycle, _) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let sessions = rt.block_on(session_lifecycle.list_sessions(None))
            .map_err(|e| CliError::Internal(format!("Failed to list sessions: {}", e)))?;

        if sessions.is_empty() {
            println!("No sessions found. Create one with: rice sessions create <name>");
            return Ok(());
        }

        println!("Sessions:");
        println!();

        for session in sessions {
            println!("  {} - {}", session.id, session.name);
            println!("    Created: {}", format_timestamp(session.created_at.timestamp()));
            println!("    Status: {:?}", session.status);
            println!();
        }

        Ok(())
    }
}

impl Command for SessionsCommand {
    fn execute(&self) -> CliResult<()> {
        println!("Sessions command not implemented yet");
        Ok(())
    }
}
