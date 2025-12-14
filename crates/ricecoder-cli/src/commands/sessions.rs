//! Sessions command - Manage ricecoder sessions

use crate::commands::Command;
use crate::error::{CliError, CliResult};
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

    /// Get session use cases (create them if needed)
    fn get_use_cases(&self) -> CliResult<(Arc<SessionLifecycleUseCase>, Arc<SessionSharingUseCase>)> {
        // For now, create the infrastructure components here
        // In a real application, these would be injected from a DI container
        let session_store = Arc::new(ricecoder_sessions::SessionStore::new()
            .map_err(|e| CliError::Internal(format!("Failed to create session store: {}", e)))?);

        let session_manager = Arc::new(ricecoder_sessions::SessionManager::new(10)); // max 10 sessions

        let session_lifecycle = Arc::new(SessionLifecycleUseCase::new(
            session_manager.clone(),
            session_store.clone(),
        ));

        let share_service = Arc::new(ricecoder_sessions::ShareService::new());
        let session_sharing = Arc::new(SessionSharingUseCase::new(
            share_service,
            session_store,
        ));

        Ok((session_lifecycle, session_sharing))
    }
}

impl Command for SessionsCommand {
    fn execute(&self) -> CliResult<()> {
        match &self.action {
            SessionsAction::List => self.list_sessions(),
            SessionsAction::Create { name } => self.create_session(name),
            SessionsAction::Delete { id } => self.delete_session(id),
            SessionsAction::Rename { id, name } => self.rename_session(id, name),
            SessionsAction::Switch { id } => self.switch_session(id),
            SessionsAction::Info { id } => self.show_session_info(id),
            SessionsAction::Share {
                expires_in,
                no_history,
                no_context,
            } => self.handle_share(*expires_in, *no_history, *no_context),
            SessionsAction::ShareList => self.handle_share_list(),
            SessionsAction::ShareRevoke { share_id } => self.handle_share_revoke(share_id),
            SessionsAction::ShareInfo { share_id } => self.handle_share_info(share_id),
            SessionsAction::ShareView { share_id } => self.handle_share_view(share_id),
        }
    }
}

    /// List all sessions
    fn list_sessions(&self) -> CliResult<()> {
        let (session_lifecycle, _) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let sessions = rt.block_on(session_lifecycle.list_sessions())?;

        if sessions.is_empty() {
            println!("No sessions found. Create one with: rice sessions create <name>");
            return Ok(());
        }

        println!("Sessions:");
        println!();

        for session in sessions {
            println!("  {} - {}", session.id, session.name);
            println!("    Created: {}", format_timestamp(session.created_at.timestamp() as u64));
            println!("    Status: {:?}", session.status);
            println!();
        }

        Ok(())
    }

    /// Create a new session
    fn create_session(&self, name: &str) -> CliResult<()> {
        let (session_lifecycle, _) = self.get_use_cases()?;
        // Create a basic session context for CLI-created sessions
        let context = SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat);

        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let session_id = rt.block_on(
            session_lifecycle.create_session(name.to_string(), context)
        )?;

        println!("Created session: {} ({})", session_id, name);
        Ok(())
    }

    /// Delete a session
    fn delete_session(&self, id: &str) -> CliResult<()> {
        let (session_lifecycle, _) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        rt.block_on(session_lifecycle.delete_session(id))?;
        println!("Deleted session: {}", id);
        Ok(())
    }

    /// Rename a session (placeholder - not implemented in use cases yet)
    fn rename_session(&self, _id: &str, _name: &str) -> CliResult<()> {
        println!("Session renaming not yet implemented in the application layer.");
        println!("This feature will be available once session state management is fully implemented.");
        Ok(())
    }

    /// Switch to a session (placeholder - not implemented in use cases yet)
    fn switch_session(&self, _id: &str) -> CliResult<()> {
        println!("Session switching not yet implemented in the application layer.");
        println!("This feature will be available once multi-session support is fully implemented.");
        Ok(())
    }

    /// Show session info
    fn show_session_info(&self, id: &str) -> CliResult<()> {
        let (session_lifecycle, _) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let session = rt.block_on(session_lifecycle.load_session(id))?;

        println!("Session: {}", session.id);
        println!("  Name: {}", session.name);
        println!("  Created: {}", format_timestamp(session.created_at.timestamp() as u64));
        println!("  Status: {:?}", session.status);
        println!("  Provider: {}", session.context.provider);
        println!("  Model: {}", session.context.model);
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
    fn handle_share(&self, expires_in: Option<u64>, no_history: bool, no_context: bool) -> CliResult<()> {
        let (_, session_sharing) = self.get_use_cases()?;
        // For now, use a placeholder session ID - in a real implementation,
        // this would get the current active session
        let session_id = "placeholder-session";

        // Build permission flags
        let include_history = !no_history;
        let include_context = !no_context;

        // Create permissions
        let permissions = SharePermissions {
            read_only: true,
            include_history,
            include_context,
        };

        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let share_id = rt.block_on(
            session_sharing.create_share_link(session_id, permissions, expires_in)
        )?;

        // Display share information
        println!("Share link: {}", share_id);
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
    fn handle_share_list(&self) -> CliResult<()> {
        let (_, session_sharing) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let shares = rt.block_on(session_sharing.list_active_shares())?;

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
    fn handle_share_revoke(&self, share_id: &str) -> CliResult<()> {
        let (_, session_sharing) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        rt.block_on(session_sharing.revoke_share_link(share_id))?;
        println!("Share {} revoked successfully", share_id);
        Ok(())
    }

    /// Handle share info command - show share details
    fn handle_share_info(&self, share_id: &str) -> CliResult<()> {
        let (_, session_sharing) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let share = rt.block_on(session_sharing.get_share_info(share_id))?;

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
    fn handle_share_view(&self, share_id: &str) -> CliResult<()> {
        let (_, session_sharing) = self.get_use_cases()?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let session = rt.block_on(session_sharing.access_shared_session(share_id))?;

        println!("Shared Session: {}", session.name);
        println!("  ID: {}", session.id);
        println!("  Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("  Status: {:?}", session.status);
        println!("  Provider: {}", session.context.provider);
        println!("  Model: {}", session.context.model);
        println!("  Mode: {:?}", session.context.mode);
        println!();
        println!("Note: This is a read-only view of the shared session.");

        Ok(())
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
