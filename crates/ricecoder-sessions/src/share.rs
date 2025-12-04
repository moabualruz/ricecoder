//! Session sharing functionality

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{Session, SessionError, SessionResult};

/// Service for managing session sharing
#[derive(Debug, Clone)]
pub struct ShareService {
    /// In-memory store of active shares
    shares: std::sync::Arc<std::sync::Mutex<HashMap<String, SessionShare>>>,
}

impl ShareService {
    /// Create a new share service
    pub fn new() -> Self {
        Self {
            shares: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Generate a share link for a session with optional expiration
    pub fn generate_share_link(
        &self,
        session_id: &str,
        permissions: SharePermissions,
        expires_in: Option<Duration>,
    ) -> SessionResult<SessionShare> {
        let share_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = expires_in.map(|duration| now + duration);

        let share = SessionShare {
            id: share_id.clone(),
            session_id: session_id.to_string(),
            created_at: now,
            expires_at,
            permissions,
        };

        // Store the share
        let mut shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;
        shares.insert(share_id.clone(), share.clone());

        Ok(share)
    }

    /// Get a share by ID, checking expiration
    pub fn get_share(&self, share_id: &str) -> SessionResult<SessionShare> {
        let shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;

        let share = shares
            .get(share_id)
            .ok_or_else(|| SessionError::ShareNotFound(share_id.to_string()))?;

        // Check if share has expired
        if let Some(expires_at) = share.expires_at {
            if Utc::now() > expires_at {
                return Err(SessionError::ShareExpired(share_id.to_string()));
            }
        }

        Ok(share.clone())
    }

    /// Create a read-only view of a session based on share permissions
    pub fn create_shared_session_view(
        &self,
        session: &Session,
        permissions: &SharePermissions,
    ) -> Session {
        let mut shared_session = session.clone();

        // Apply permissions
        if !permissions.include_history {
            shared_session.history.clear();
        }

        if !permissions.include_context {
            shared_session.context.files.clear();
            shared_session.context.custom.clear();
        }

        shared_session
    }

    /// Import a shared session, creating a new local session
    pub fn import_shared_session(
        &self,
        share_id: &str,
        shared_session: &Session,
    ) -> SessionResult<Session> {
        // Verify the share exists and hasn't expired
        let _share = self.get_share(share_id)?;

        // Create a new local session with the shared session's data
        let mut imported_session = shared_session.clone();

        // Generate a new ID for the imported session
        imported_session.id = Uuid::new_v4().to_string();

        // Update timestamps
        let now = Utc::now();
        imported_session.created_at = now;
        imported_session.updated_at = now;

        Ok(imported_session)
    }

    /// Revoke a share by ID
    pub fn revoke_share(&self, share_id: &str) -> SessionResult<()> {
        let mut shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;

        shares
            .remove(share_id)
            .ok_or_else(|| SessionError::ShareNotFound(share_id.to_string()))?;

        Ok(())
    }

    /// List all active shares
    pub fn list_shares(&self) -> SessionResult<Vec<SessionShare>> {
        let shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;

        let now = Utc::now();
        let active_shares: Vec<SessionShare> = shares
            .values()
            .filter(|share| {
                // Include only non-expired shares
                share.expires_at.is_none() || share.expires_at.is_some_and(|exp| now <= exp)
            })
            .cloned()
            .collect();

        Ok(active_shares)
    }

    /// Clean up expired shares
    pub fn cleanup_expired_shares(&self) -> SessionResult<usize> {
        let mut shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;

        let now = Utc::now();
        let initial_count = shares.len();

        shares.retain(|_, share| {
            share.expires_at.is_none() || share.expires_at.is_some_and(|exp| now <= exp)
        });

        Ok(initial_count - shares.len())
    }
}

impl Default for ShareService {
    fn default() -> Self {
        Self::new()
    }
}

/// A shared session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionShare {
    /// Unique identifier for the share
    pub id: String,
    /// ID of the shared session
    pub session_id: String,
    /// When the share was created
    pub created_at: DateTime<Utc>,
    /// When the share expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Share permissions
    pub permissions: SharePermissions,
}

/// Permissions for a shared session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePermissions {
    /// Whether the share is read-only
    pub read_only: bool,
    /// Whether to include conversation history
    pub include_history: bool,
    /// Whether to include session context
    pub include_context: bool,
}
