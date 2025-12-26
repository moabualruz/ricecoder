//! Session manager for lifecycle management and session switching

use std::collections::{HashMap, HashSet};

use chrono::{Duration, Utc};
use tracing::{debug, error, warn};

use crate::{
    bus::{BusEvent, EventBus, SessionEvent},
    error::{SessionError, SessionResult},
    models::{MessagePart, Session, SessionContext},
    share::ShareService,
    snapshot::SnapshotManager,
    store::SessionStore,
    token_estimator::{TokenEstimator, TokenUsageTracker},
};

/// Manages session lifecycle and switching
pub struct SessionManager {
    /// All sessions indexed by ID
    sessions: HashMap<String, Session>,
    /// Currently active session ID
    active_session_id: Option<String>,
    /// Maximum number of concurrent sessions
    session_limit: usize,
    /// Token estimator for tracking usage
    token_estimator: TokenEstimator,
    /// Token usage trackers per session
    token_trackers: HashMap<String, TokenUsageTracker>,
    /// Event bus for session events
    event_bus: EventBus,
    /// Share service for session sharing
    share_service: ShareService,
    /// Snapshot manager for session snapshots
    snapshot_manager: SnapshotManager,
    /// Parent-child session relationships
    session_children: HashMap<String, Vec<String>>,
    /// Session store for disk persistence
    store: Option<SessionStore>,
}

impl SessionManager {
    /// Create a new session manager with a session limit
    pub fn new(session_limit: usize) -> Self {
        // Try to create session store for disk persistence
        let store = match SessionStore::new() {
            Ok(s) => {
                debug!("SessionStore initialized for disk persistence");
                Some(s)
            }
            Err(e) => {
                warn!("Failed to initialize SessionStore, sessions will be in-memory only: {}", e);
                None
            }
        };

        let mut manager = Self {
            sessions: HashMap::new(),
            active_session_id: None,
            session_limit,
            token_estimator: TokenEstimator::new(),
            token_trackers: HashMap::new(),
            event_bus: EventBus::new(),
            share_service: ShareService::new(),
            snapshot_manager: SnapshotManager::disabled(),
            session_children: HashMap::new(),
            store,
        };

        // Load existing sessions from disk synchronously at startup
        manager.load_sessions_from_disk();

        manager
    }

    /// Load all sessions from disk into memory
    fn load_sessions_from_disk(&mut self) {
        if let Some(ref store) = self.store {
            // Use blocking runtime for sync context
            let rt = match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    // Already in async context, spawn blocking
                    let store_clone = store.clone();
                    let sessions = std::thread::spawn(move || {
                        tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(store_clone.list())
                    })
                    .join()
                    .unwrap_or_else(|_| Ok(Vec::new()));
                    sessions
                }
                Err(_) => {
                    // Not in async context, create runtime
                    match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt.block_on(store.list()),
                        Err(e) => {
                            error!("Failed to create runtime for session loading: {}", e);
                            Ok(Vec::new())
                        }
                    }
                }
            };

            match rt {
                Ok(sessions) => {
                    debug!("Loaded {} sessions from disk", sessions.len());
                    for session in sessions {
                        // Create token tracker for loaded session
                        if let Ok(tracker) = self.token_estimator.create_usage_tracker(&session.context.model) {
                            self.token_trackers.insert(session.id.clone(), tracker);
                        }
                        self.sessions.insert(session.id.clone(), session);
                    }
                    // Set first session as active if we have any
                    if self.active_session_id.is_none() {
                        self.active_session_id = self.sessions.keys().next().cloned();
                    }
                }
                Err(e) => {
                    warn!("Failed to load sessions from disk: {}", e);
                }
            }
        }
    }

    /// Create a new session
    pub fn create_session(
        &mut self,
        name: String,
        context: SessionContext,
    ) -> SessionResult<Session> {
        // Check session limit
        if self.sessions.len() >= self.session_limit {
            return Err(SessionError::LimitReached {
                max: self.session_limit,
            });
        }

        let session = Session::new(name, context);
        let session_id = session.id.clone();

        // Create token usage tracker for this session
        let tracker = self
            .token_estimator
            .create_usage_tracker(&session.context.model)?;
        self.token_trackers.insert(session_id.clone(), tracker);

        self.sessions.insert(session_id.clone(), session.clone());

        // Persist to disk
        self.persist_session(&session);

        // Set as active if it's the first session
        if self.active_session_id.is_none() {
            self.active_session_id = Some(session_id.clone());
        }

        // Publish SessionCreated event
        self.event_bus.publish(BusEvent::Session(SessionEvent::Created {
            session_id: session_id.clone(),
        }));

        Ok(session)
    }

    /// Persist a session to disk (fire-and-forget)
    fn persist_session(&self, session: &Session) {
        if let Some(ref store) = self.store {
            let store_clone = store.clone();
            let session_clone = session.clone();
            
            // Spawn async task to persist without blocking
            tokio::spawn(async move {
                if let Err(e) = store_clone.save(&session_clone).await {
                    error!("Failed to persist session {} to disk: {}", session_clone.id, e);
                } else {
                    debug!("Session {} persisted to disk", session_clone.id);
                }
            });
        }
    }

    /// Close a session (alias for delete_session for interface compatibility)
    pub fn close_session(&mut self, session_id: &str) -> SessionResult<()> {
        self.delete_session(session_id)
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> SessionResult<()> {
        if !self.sessions.contains_key(session_id) {
            return Err(SessionError::NotFound(session_id.to_string()));
        }

        self.sessions.remove(session_id);
        self.token_trackers.remove(session_id);
        self.session_children.remove(session_id);

        // Delete from disk
        self.delete_session_from_disk(session_id);

        // If the deleted session was active, switch to another session
        if self.active_session_id.as_deref() == Some(session_id) {
            self.active_session_id = self.sessions.keys().next().cloned();
        }

        // Publish SessionDeleted event
        self.event_bus.publish(BusEvent::Session(SessionEvent::Deleted {
            session_id: session_id.to_string(),
        }));

        Ok(())
    }

    /// Delete a session from disk (fire-and-forget)
    fn delete_session_from_disk(&self, session_id: &str) {
        if let Some(ref store) = self.store {
            let store_clone = store.clone();
            let session_id = session_id.to_string();
            
            // Spawn async task to delete without blocking
            tokio::spawn(async move {
                if let Err(e) = store_clone.delete(&session_id).await {
                    // Not an error if session doesn't exist on disk
                    debug!("Note: Could not delete session {} from disk: {}", session_id, e);
                } else {
                    debug!("Session {} deleted from disk", session_id);
                }
            });
        }
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> SessionResult<Session> {
        self.sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))
    }

    /// Get the active session
    pub fn get_active_session(&self) -> SessionResult<Session> {
        let session_id = self
            .active_session_id
            .as_ref()
            .ok_or(SessionError::Invalid("No active session".to_string()))?;

        self.get_session(session_id)
    }

    /// Switch to a different session
    pub fn switch_session(&mut self, session_id: &str) -> SessionResult<Session> {
        // Verify the session exists
        let session = self.get_session(session_id)?;

        self.active_session_id = Some(session_id.to_string());

        Ok(session)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<Session> {
        self.sessions.values().cloned().collect()
    }

    /// Get the ID of the active session
    pub fn active_session_id(&self) -> Option<&str> {
        self.active_session_id.as_deref()
    }

    /// Update a session
    pub fn update_session(&mut self, session: Session) -> SessionResult<()> {
        if !self.sessions.contains_key(&session.id) {
            return Err(SessionError::NotFound(session.id.clone()));
        }

        let session_id = session.id.clone();
        self.sessions.insert(session_id.clone(), session.clone());

        // Persist to disk
        self.persist_session(&session);

        // Publish SessionUpdated event
        self.event_bus.publish(BusEvent::Session(SessionEvent::Updated {
            session_id,
        }));

        Ok(())
    }

    /// Get the session limit
    pub fn session_limit(&self) -> usize {
        self.session_limit
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if session limit is reached
    pub fn is_limit_reached(&self) -> bool {
        self.sessions.len() >= self.session_limit
    }

    /// Record token usage for a prompt message
    pub fn record_prompt_tokens(&mut self, session_id: &str, tokens: usize) -> SessionResult<()> {
        if let Some(tracker) = self.token_trackers.get_mut(session_id) {
            tracker.record_prompt(tokens);
            Ok(())
        } else {
            Err(SessionError::NotFound(format!(
                "Token tracker for session {} not found",
                session_id
            )))
        }
    }

    /// Record token usage for a completion message
    pub fn record_completion_tokens(
        &mut self,
        session_id: &str,
        tokens: usize,
    ) -> SessionResult<()> {
        if let Some(tracker) = self.token_trackers.get_mut(session_id) {
            tracker.record_completion(tokens);
            Ok(())
        } else {
            Err(SessionError::NotFound(format!(
                "Token tracker for session {} not found",
                session_id
            )))
        }
    }

    /// Get token usage for a session
    pub fn get_session_token_usage(
        &self,
        session_id: &str,
    ) -> SessionResult<crate::token_estimator::TokenUsage> {
        if let Some(tracker) = self.token_trackers.get(session_id) {
            Ok(tracker.current_usage())
        } else {
            Err(SessionError::NotFound(format!(
                "Token tracker for session {} not found",
                session_id
            )))
        }
    }

    /// Get token usage for the active session
    pub fn get_active_session_token_usage(
        &self,
    ) -> SessionResult<crate::token_estimator::TokenUsage> {
        let session_id = self
            .active_session_id
            .as_ref()
            .ok_or(SessionError::Invalid("No active session".to_string()))?;

        self.get_session_token_usage(session_id)
    }

    /// Check if a session is approaching token limits
    pub fn check_session_token_limits(
        &self,
        session_id: &str,
    ) -> SessionResult<crate::token_estimator::TokenLimitStatus> {
        let usage = self.get_session_token_usage(session_id)?;
        Ok(self
            .token_estimator
            .check_token_limits(usage.total_tokens, &usage.model))
    }

    /// Estimate tokens for content using the active session's model
    pub fn estimate_tokens_for_active_session(
        &mut self,
        content: &str,
    ) -> SessionResult<crate::token_estimator::TokenEstimate> {
        let session = self.get_active_session()?;
        self.token_estimator
            .estimate_tokens(content, Some(&session.context.model))
    }

    /// Estimate tokens for content using a specific model
    pub fn estimate_tokens_with_model(
        &mut self,
        content: &str,
        model: &str,
    ) -> SessionResult<crate::token_estimator::TokenEstimate> {
        self.token_estimator.estimate_tokens(content, Some(model))
    }

    /// Generate a descriptive title for a session based on its content
    pub fn generate_session_title(&self, session_id: &str) -> SessionResult<String> {
        let session = self.get_session(session_id)?;

        // If session already has a custom title, use it
        if !session.name.is_empty() && session.name != "Untitled" {
            return Ok(session.name.clone());
        }

        // Generate title based on message content
        if session.history.is_empty() {
            return Ok("Empty Session".to_string());
        }

        // Analyze the first few messages to determine the topic
        let mut topics = Vec::new();
        let mut tools_used = HashSet::new();

        for message in session.history.iter().take(5) {
            // Extract text content
            for part in &message.parts {
                match part {
                    MessagePart::Text { text, .. } => {
                        // Simple keyword extraction (could be enhanced with NLP)
                        let lower_text = text.to_lowercase();
                        if lower_text.contains("error")
                            || lower_text.contains("bug")
                            || lower_text.contains("fix")
                        {
                            topics.push("Bug Fix");
                        } else if lower_text.contains("feature")
                            || lower_text.contains("implement")
                            || lower_text.contains("add")
                        {
                            topics.push("Feature Development");
                        } else if lower_text.contains("refactor")
                            || lower_text.contains("clean")
                            || lower_text.contains("optimize")
                        {
                            topics.push("Code Refactoring");
                        } else if lower_text.contains("test") || lower_text.contains("testing") {
                            topics.push("Testing");
                        } else if lower_text.contains("documentation")
                            || lower_text.contains("docs")
                            || lower_text.contains("readme")
                        {
                            topics.push("Documentation");
                        }
                    }
                    MessagePart::ToolInvocation(invocation) => {
                        tools_used.insert(invocation.tool_name.clone());
                    }
                    MessagePart::ToolResult(result) => {
                        tools_used.insert(result.tool_name.clone());
                    }
                    _ => {}
                }
            }
        }

        // Generate title based on analysis
        let title = if !topics.is_empty() {
            // Use the most common topic
            let primary_topic = topics.into_iter().next().unwrap();
            if tools_used.is_empty() {
                format!("{} Session", primary_topic)
            } else {
                format!(
                    "{} with {}",
                    primary_topic,
                    tools_used.into_iter().next().unwrap()
                )
            }
        } else if !tools_used.is_empty() {
            format!("Tool Usage: {}", tools_used.into_iter().next().unwrap())
        } else {
            // Fallback: use first message content
            let first_content = session.history[0].content();
            if first_content.len() > 50 {
                format!("{}...", &first_content[..47])
            } else {
                first_content
            }
        };

        Ok(title)
    }

    /// Generate a comprehensive summary for a session
    pub fn generate_session_summary(&self, session_id: &str) -> SessionResult<SessionSummary> {
        let session = self.get_session(session_id)?;

        let mut summary = SessionSummary {
            session_id: session.id.clone(),
            title: self.generate_session_title(session_id)?,
            duration: session.updated_at.signed_duration_since(session.created_at),
            message_count: session.history.len(),
            topics: Vec::new(),
            files_modified: HashSet::new(),
            tools_used: HashSet::new(),
            last_activity: session.updated_at,
        };

        // Analyze all messages for detailed summary
        for message in &session.history {
            // Extract topics and tools
            for part in &message.parts {
                match part {
                    MessagePart::Text { text, .. } => {
                        let lower_text = text.to_lowercase();
                        if lower_text.contains("error")
                            || lower_text.contains("bug")
                            || lower_text.contains("fix")
                        {
                            summary.topics.push("Bug Fixes".to_string());
                        } else if lower_text.contains("feature")
                            || lower_text.contains("implement")
                            || lower_text.contains("add")
                        {
                            summary.topics.push("Feature Development".to_string());
                        } else if lower_text.contains("refactor")
                            || lower_text.contains("clean")
                            || lower_text.contains("optimize")
                        {
                            summary.topics.push("Code Refactoring".to_string());
                        } else if lower_text.contains("test") || lower_text.contains("testing") {
                            summary.topics.push("Testing".to_string());
                        } else if lower_text.contains("documentation")
                            || lower_text.contains("docs")
                        {
                            summary.topics.push("Documentation".to_string());
                        }
                    }
                    MessagePart::ToolInvocation(invocation) => {
                        summary.tools_used.insert(invocation.tool_name.clone());
                    }
                    MessagePart::ToolResult(result) => {
                        summary.tools_used.insert(result.tool_name.clone());
                    }
                    MessagePart::FileReference(file_ref) => {
                        summary.files_modified.insert(file_ref.path.clone());
                    }
                    _ => {}
                }
            }
        }

        // Remove duplicates from topics
        summary.topics.sort();
        summary.topics.dedup();

        Ok(summary)
    }

    /// Update session title (manual override)
    pub fn update_session_title(&mut self, session_id: &str, title: String) -> SessionResult<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        session.name = title;
        session.updated_at = Utc::now();

        // Persist updated session to disk
        let session_clone = session.clone();
        self.persist_session(&session_clone);

        Ok(())
    }

    /// Check if a session has overflowed context window (OpenCode parity)
    pub fn is_session_overflow(
        &self,
        session_id: &str,
        input_tokens: usize,
        cache_read_tokens: usize,
        output_tokens: usize,
    ) -> SessionResult<bool> {
        let session = self.get_session(session_id)?;
        let pricing = self.token_estimator.get_pricing(&session.context.model);
        
        if let Some(pricing) = pricing {
            Ok(crate::token_estimator::is_overflow(
                input_tokens,
                cache_read_tokens,
                output_tokens,
                pricing.max_tokens,
                pricing.max_output_tokens,
            ))
        } else {
            // Unknown model, assume no overflow
            Ok(false)
        }
    }

    /// Get max output tokens for a session (OpenCode parity)
    pub fn get_max_output_tokens(&self, session_id: &str) -> SessionResult<usize> {
        let session = self.get_session(session_id)?;
        let pricing = self.token_estimator.get_pricing(&session.context.model);
        
        if let Some(pricing) = pricing {
            Ok(crate::token_estimator::max_output_tokens(pricing.max_output_tokens))
        } else {
            Ok(crate::token_estimator::OUTPUT_TOKEN_MAX)
        }
    }

    // === GAP 1: Session CRUD API additions ===

    /// Fork a session from a specific message point
    pub fn fork(
        &mut self,
        session_id: &str,
        message_id: Option<&str>,
    ) -> SessionResult<Session> {
        let parent_session = self.get_session(session_id)?;

        // Truncate history at message_id if provided
        let mut forked_session = parent_session.clone();
        if let Some(msg_id) = message_id {
            let message_idx = forked_session
                .history
                .iter()
                .position(|m| m.id == msg_id)
                .ok_or_else(|| SessionError::NotFound(format!("Message {} not found", msg_id)))?;
            forked_session.history.truncate(message_idx + 1);
        }

        // Create new session with forked history
        let forked_session = Session {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} (fork)", parent_session.name),
            history: forked_session.history,
            context: parent_session.context.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: crate::models::SessionStatus::Active,
            background_agents: Vec::new(),
        };

        // Track parent-child relationship
        self.session_children
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(forked_session.id.clone());

        // Store forked session
        self.sessions
            .insert(forked_session.id.clone(), forked_session.clone());

        // Create token tracker for forked session
        let tracker = self
            .token_estimator
            .create_usage_tracker(&forked_session.context.model)?;
        self.token_trackers
            .insert(forked_session.id.clone(), tracker);

        // Publish event
        self.event_bus.publish(BusEvent::Session(SessionEvent::Created {
            session_id: forked_session.id.clone(),
        }));

        Ok(forked_session)
    }

    /// Touch a session to update its timestamp
    pub fn touch(&mut self, session_id: &str) -> SessionResult<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        session.updated_at = Utc::now();

        // Persist updated session to disk
        let session_clone = session.clone();
        self.persist_session(&session_clone);

        // Publish SessionUpdated event
        self.event_bus.publish(BusEvent::Session(SessionEvent::Updated {
            session_id: session_id.to_string(),
        }));

        Ok(())
    }

    /// Get snapshot diff for a session
    pub async fn diff(&self, session_id: &str) -> SessionResult<String> {
        let session = self.get_session(session_id)?;
        
        // Use snapshot manager to get diff
        match self.snapshot_manager.diff(&session.id).await {
            Ok(diff_text) => Ok(diff_text),
            Err(_) => Ok(String::from("No changes detected")),
        }
    }

    /// Get child sessions
    pub fn children(&self, session_id: &str) -> SessionResult<Vec<Session>> {
        let child_ids = self
            .session_children
            .get(session_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let children: Vec<Session> = child_ids
            .iter()
            .filter_map(|id| self.sessions.get(id).cloned())
            .collect();

        Ok(children)
    }

    // === GAP 3: Sharing integration ===

    /// Share a session
    pub fn share(
        &self,
        session_id: &str,
        permissions: crate::share::SharePermissions,
        expires_in: Option<Duration>,
    ) -> SessionResult<crate::share::SessionShare> {
        // Verify session exists
        let _session = self.get_session(session_id)?;

        self.share_service
            .generate_share_link(session_id, permissions, expires_in)
    }

    /// Revoke a share
    pub fn unshare(
        &self,
        share_id: &str,
        revoker_user_id: Option<String>,
    ) -> SessionResult<()> {
        self.share_service.revoke_share(share_id, revoker_user_id)
    }

    /// Get a share by ID
    pub fn get_share(&self, share_id: &str) -> SessionResult<crate::share::SessionShare> {
        self.share_service.get_share(share_id)
    }

    // === Event bus access ===

    /// Get the event bus for external subscribers
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Get the share service for external access
    pub fn share_service(&self) -> &ShareService {
        &self.share_service
    }

    /// Get the snapshot manager for external access
    pub fn snapshot_manager(&self) -> &SnapshotManager {
        &self.snapshot_manager
    }
}

/// Session summary information
#[derive(Debug, Clone)]
pub struct SessionSummary {
    /// Session ID
    pub session_id: String,
    /// Generated or manual title
    pub title: String,
    /// Total session duration
    pub duration: Duration,
    /// Number of messages in the session
    pub message_count: usize,
    /// Topics discussed (derived from content)
    pub topics: Vec<String>,
    /// Files that were modified or referenced
    pub files_modified: HashSet<std::path::PathBuf>,
    /// Tools that were used
    pub tools_used: HashSet<String>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionMode;
}
