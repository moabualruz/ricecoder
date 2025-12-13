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

impl Command for SessionsCommand {
    fn execute(&self) -> CliResult<()> {
        match &self.action {
            SessionsAction::List => list_sessions(),
            SessionsAction::Create { name } => create_session(name),
            SessionsAction::Delete { id } => delete_session(id),
            SessionsAction::Rename { id, name } => rename_session(id, name),
            SessionsAction::Switch { id } => switch_session(id),
            SessionsAction::Info { id } => show_session_info(id),
            SessionsAction::Share {
                expires_in,
                no_history,
                no_context,
            } => handle_share(*expires_in, *no_history, *no_context),
            SessionsAction::ShareList => handle_share_list(),
            SessionsAction::ShareRevoke { share_id } => handle_share_revoke(share_id),
            SessionsAction::ShareInfo { share_id } => handle_share_info(share_id),
            SessionsAction::ShareView { share_id } => handle_share_view(share_id),
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

/// Handle share command - generate a shareable link
fn handle_share(expires_in: Option<u64>, no_history: bool, no_context: bool) -> CliResult<()> {
    use ricecoder_sessions::{ShareService, SharePermissions};
    use chrono::Duration;

    // Create share service
    let share_service = ShareService::new();

    // Build permission flags
    let include_history = !no_history;
    let include_context = !no_context;

    // Get current session ID (for now, use a placeholder)
    let session_id = "current-session";

    // Create permissions
    let permissions = SharePermissions {
        read_only: true,
        include_history,
        include_context,
    };

    // Convert expires_in to Duration
    let expires_in_duration = expires_in.map(|secs| Duration::seconds(secs as i64));

    // Generate share link
    let share = share_service
        .generate_share_link(session_id, permissions, expires_in_duration)
        .map_err(|e| CliError::Internal(format!("Failed to generate share link: {}", e)))?;

    // Display share information
    println!("Share link: {}", share.id);
    println!();
    println!("Permissions:");
    println!("  History: {}", if include_history { "Yes" } else { "No" });
    println!("  Context: {}", if include_context { "Yes" } else { "No" });

    if let Some(expiration) = expires_in {
        println!("  Expires in: {} seconds", expiration);
    } else {
        println!("  Expires: Never");
    }

    println!();
    println!("Share this link with others to grant access to your session.");

    Ok(())
}

/// Handle share list command - list all active shares
fn handle_share_list() -> CliResult<()> {
    use ricecoder_sessions::ShareService;

    // Create share service
    let share_service = ShareService::new();

    // List all active shares
    let shares = share_service
        .list_shares()
        .map_err(|e| CliError::Internal(format!("Failed to list shares: {}", e)))?;

    if shares.is_empty() {
        println!("Active shares:");
        println!();
        println!("  No shares found. Create one with: rice sessions share");
        println!();
        return Ok(());
    }

    println!("Active shares:");
    println!();
    println!("{:<40} {:<20} {:<20} {:<20} {:<30}", "Share ID", "Session ID", "Created", "Expires", "Permissions");
    println!("{}", "-".repeat(130));

    for share in shares {
        let created = share.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let expires = share
            .expires_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Never".to_string());
        let permissions = format!(
            "History: {}, Context: {}",
            if share.permissions.include_history { "Yes" } else { "No" },
            if share.permissions.include_context { "Yes" } else { "No" }
        );

        println!(
            "{:<40} {:<20} {:<20} {:<20} {:<30}",
            &share.id[..40.min(share.id.len())],
            &share.session_id[..20.min(share.session_id.len())],
            created,
            expires,
            &permissions[..30.min(permissions.len())]
        );
    }

    println!();

    Ok(())
}

/// Handle share revoke command - revoke a share
fn handle_share_revoke(share_id: &str) -> CliResult<()> {
    use ricecoder_sessions::ShareService;

    // Create share service
    let share_service = ShareService::new();

    // Revoke the share
    share_service
        .revoke_share(share_id)
        .map_err(|e| CliError::Internal(format!("Failed to revoke share: {}", e)))?;

    println!("Share {} revoked successfully", share_id);
    Ok(())
}

/// Handle share info command - show share details
fn handle_share_info(share_id: &str) -> CliResult<()> {
    use ricecoder_sessions::ShareService;

    // Create share service
    let share_service = ShareService::new();

    // Get share details
    let share = share_service
        .get_share(share_id)
        .map_err(|e| CliError::Internal(format!("Failed to get share info: {}", e)))?;

    println!("Share: {}", share.id);
    println!("  Session: {}", share.session_id);
    println!("  Created: {}", share.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!(
        "  Expires: {}",
        share
            .expires_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Never".to_string())
    );
    println!("  Permissions:");
    println!("    History: {}", if share.permissions.include_history { "Yes" } else { "No" });
    println!("    Context: {}", if share.permissions.include_context { "Yes" } else { "No" });
    println!("    Read-Only: {}", if share.permissions.read_only { "Yes" } else { "No" });
    println!("  Status: Active");

    Ok(())
}

/// Handle share view command - view a shared session
fn handle_share_view(share_id: &str) -> CliResult<()> {
    use ricecoder_sessions::{ShareService, Session, SessionContext, SessionMode};

    // Create share service
    let share_service = ShareService::new();

    // Validate share exists and is not expired
    let share = share_service
        .get_share(share_id)
        .map_err(|e| match e {
            ricecoder_sessions::SessionError::ShareNotFound(_) => {
                CliError::Internal(format!("Share not found: {}", share_id))
            }
            ricecoder_sessions::SessionError::ShareExpired(_) => {
                CliError::Internal(format!("Share has expired: {}", share_id))
            }
            _ => CliError::Internal(format!("Failed to access share: {}", e)),
        })?;

    // For now, create a mock session to display
    // In a real implementation, this would retrieve the actual session from storage
    let mock_session = Session::new(
        format!("Shared Session ({})", &share.session_id[..8.min(share.session_id.len())]),
        SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat),
    );

    // Create filtered session view based on permissions
    let shared_session = share_service.create_shared_session_view(&mock_session, &share.permissions);

    // Display shared session with read-only mode enforced
    display_shared_session(&shared_session, &share)
}

/// Display a shared session with read-only mode enforced
fn display_shared_session(
    session: &ricecoder_sessions::Session,
    share: &ricecoder_sessions::SessionShare,
) -> CliResult<()> {
    use ricecoder_sessions::MessageRole;

    // Display header with permission indicators
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║ Shared Session: {} [Read-Only]", session.name);
    println!("║ Permissions: [History: {}] [Context: {}]",
        if share.permissions.include_history { "Yes" } else { "No" },
        if share.permissions.include_context { "Yes" } else { "No" }
    );
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Display session metadata
    println!("Session Information:");
    println!("  Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S"));
    if let Some(expires_at) = share.expires_at {
        println!("  Expires: {}", expires_at.format("%Y-%m-%d %H:%M:%S"));
    } else {
        println!("  Expires: Never");
    }
    println!("  Status: Read-Only");
    println!();

    // Display messages if history is included
    if share.permissions.include_history {
        if session.history.is_empty() {
            println!("Messages: (empty)");
        } else {
            println!("Messages ({} total):", session.history.len());
            println!();

            // Pagination: show first 10 messages
            let messages_per_page = 10;
            let total_messages = session.history.len();
            let pages = (total_messages + messages_per_page - 1) / messages_per_page;
            let current_page = 1;
            let start_idx = (current_page - 1) * messages_per_page;
            let end_idx = (start_idx + messages_per_page).min(total_messages);

            for (idx, msg) in session.history[start_idx..end_idx].iter().enumerate() {
                let role_str = match msg.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant",
                    MessageRole::System => "System",
                };

                println!("[{}] {}: {}", start_idx + idx + 1, role_str, msg.content);
                println!("    Timestamp: {}", msg.timestamp.format("%Y-%m-%d %H:%M:%S"));
                println!();
            }

            // Display pagination info
            println!("Message {} - {} of {} (Page {} of {})", 
                start_idx + 1,
                end_idx,
                total_messages,
                current_page,
                pages
            );
            
            if pages > 1 {
                println!("(Use 'rice sessions share view <share_id> --page <N>' to view other pages)");
            }
        }
    } else {
        println!("Messages: (history excluded from share)");
    }

    println!();

    // Display context if included
    if share.permissions.include_context {
        println!("Context:");
        if let Some(project_path) = &session.context.project_path {
            println!("  Project: {}", project_path);
        }
        println!("  Provider: {}", session.context.provider);
        println!("  Model: {}", session.context.model);

        if !session.context.files.is_empty() {
            println!("  Files:");
            for file in &session.context.files {
                println!("    - {}", file);
            }
        } else {
            println!("  Files: (none)");
        }
    } else {
        println!("Context: (context excluded from share)");
    }

    println!();
    println!("This is a read-only view. You cannot modify this shared session.");
    println!();

    Ok(())
}


}
