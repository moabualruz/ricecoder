//! Session sharing functionality

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use crate::{Session, SessionError, SessionResult};

/// Service for managing session sharing
#[derive(Debug, Clone)]
pub struct ShareService {
    /// In-memory store of active shares
    shares: std::sync::Arc<std::sync::Mutex<HashMap<String, SessionShare>>>,
    /// Analytics tracker
    analytics: Arc<ShareAnalytics>,
}

impl ShareService {
    /// Create a new share service
    pub fn new() -> Self {
        Self {
            shares: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            analytics: Arc::new(ShareAnalytics::new()),
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

        // Track analytics
        self.analytics.record_share_created(&share);

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

        // Track analytics
        self.analytics.record_share_accessed(&session.id, permissions);

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

    /// List all active shares for a specific session
    pub fn list_shares_for_session(&self, session_id: &str) -> SessionResult<Vec<SessionShare>> {
        let shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;

        let now = Utc::now();
        let session_shares: Vec<SessionShare> = shares
            .values()
            .filter(|share| {
                // Filter by session_id and include only non-expired shares
                share.session_id == session_id
                    && (share.expires_at.is_none()
                        || share.expires_at.is_some_and(|exp| now <= exp))
            })
            .cloned()
            .collect();

        Ok(session_shares)
    }

    /// Invalidate all shares for a session when the session is deleted
    pub fn invalidate_session_shares(&self, session_id: &str) -> SessionResult<usize> {
        let mut shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;

        let initial_count = shares.len();

        // Remove all shares associated with this session
        shares.retain(|_, share| share.session_id != session_id);

        Ok(initial_count - shares.len())
    }

    /// Get sharing analytics
    pub fn get_analytics(&self) -> ShareAnalyticsData {
        self.analytics.get_data()
    }

    /// Reset analytics data
    pub fn reset_analytics(&self) {
        self.analytics.reset();
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

/// Analytics data for session sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareAnalyticsData {
    /// Total number of shares created
    pub total_shares_created: u64,
    /// Total number of share accesses
    pub total_share_accesses: u64,
    /// Number of shares by permission type
    pub shares_by_permissions: HashMap<String, u64>,
    /// Top accessed sessions
    pub top_accessed_sessions: Vec<(String, u64)>,
    /// Recent share activity
    pub recent_activity: Vec<ShareActivity>,
}

/// Individual share activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareActivity {
    pub share_id: String,
    pub session_id: String,
    pub activity_type: ShareActivityType,
    pub timestamp: DateTime<Utc>,
    pub permissions: Option<SharePermissions>,
}

/// Types of share activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShareActivityType {
    Created,
    Accessed,
    Expired,
    Revoked,
}

/// Analytics tracker for session sharing
#[derive(Debug)]
struct ShareAnalytics {
    total_shares_created: AtomicU64,
    total_share_accesses: AtomicU64,
    shares_by_permissions: std::sync::Mutex<HashMap<String, u64>>,
    session_access_counts: std::sync::Mutex<HashMap<String, u64>>,
    recent_activity: std::sync::Mutex<Vec<ShareActivity>>,
}

impl ShareAnalytics {
    fn new() -> Self {
        Self {
            total_shares_created: AtomicU64::new(0),
            total_share_accesses: AtomicU64::new(0),
            shares_by_permissions: std::sync::Mutex::new(HashMap::new()),
            session_access_counts: std::sync::Mutex::new(HashMap::new()),
            recent_activity: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn record_share_created(&self, share: &SessionShare) {
        self.total_shares_created.fetch_add(1, Ordering::Relaxed);

        let perm_key = format!("ro={},hist={},ctx={}",
            share.permissions.read_only,
            share.permissions.include_history,
            share.permissions.include_context);

        if let Ok(mut perms) = self.shares_by_permissions.lock() {
            *perms.entry(perm_key).or_insert(0) += 1;
        }

        self.record_activity(ShareActivity {
            share_id: share.id.clone(),
            session_id: share.session_id.clone(),
            activity_type: ShareActivityType::Created,
            timestamp: Utc::now(),
            permissions: Some(share.permissions.clone()),
        });
    }

    fn record_share_accessed(&self, session_id: &str, permissions: &SharePermissions) {
        self.total_share_accesses.fetch_add(1, Ordering::Relaxed);

        if let Ok(mut counts) = self.session_access_counts.lock() {
            *counts.entry(session_id.to_string()).or_insert(0) += 1;
        }

        self.record_activity(ShareActivity {
            share_id: "accessed".to_string(), // Not a specific share ID for access
            session_id: session_id.to_string(),
            activity_type: ShareActivityType::Accessed,
            timestamp: Utc::now(),
            permissions: Some(permissions.clone()),
        });
    }

    fn record_activity(&self, activity: ShareActivity) {
        if let Ok(mut activities) = self.recent_activity.lock() {
            activities.push(activity);
            // Keep only last 100 activities
            if activities.len() > 100 {
                activities.remove(0);
            }
        }
    }

    fn get_data(&self) -> ShareAnalyticsData {
        let total_shares = self.total_shares_created.load(Ordering::Relaxed);
        let total_accesses = self.total_share_accesses.load(Ordering::Relaxed);

        let shares_by_permissions = self.shares_by_permissions.lock()
            .map(|perms| perms.clone())
            .unwrap_or_default();

        let session_counts = self.session_access_counts.lock()
            .map(|counts| {
                let mut vec: Vec<(String, u64)> = counts.iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                vec.sort_by(|a, b| b.1.cmp(&a.1));
                vec.truncate(10); // Top 10
                vec
            })
            .unwrap_or_default();

        let recent_activity = self.recent_activity.lock()
            .map(|activities| activities.clone())
            .unwrap_or_default();

        ShareAnalyticsData {
            total_shares_created: total_shares,
            total_share_accesses: total_accesses,
            shares_by_permissions,
            top_accessed_sessions: session_counts,
            recent_activity,
        }
    }

    fn reset(&self) {
        self.total_shares_created.store(0, Ordering::Relaxed);
        self.total_share_accesses.store(0, Ordering::Relaxed);

        if let Ok(mut perms) = self.shares_by_permissions.lock() {
            perms.clear();
        }

        if let Ok(mut counts) = self.session_access_counts.lock() {
            counts.clear();
        }

        if let Ok(mut activities) = self.recent_activity.lock() {
            activities.clear();
        }
    }
}
