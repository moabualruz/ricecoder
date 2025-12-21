//! Session sharing functionality

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

use ricecoder_domain::value_objects::ValidUrl;
use ricecoder_security::access_control::{AccessControl, Permission, Principal, ResourceType};
use ricecoder_security::audit::{AuditEvent, AuditEventType, AuditLogger};
use ricecoder_security::compliance::DataType;
use ricecoder_security::encryption::{EncryptedData, KeyManager};

use crate::{Session, SessionError, SessionResult};

/// Service for managing session sharing
pub struct ShareService {
    /// In-memory store of active shares
    shares: std::sync::Arc<std::sync::Mutex<HashMap<String, SessionShare>>>,
    /// Analytics tracker
    analytics: Arc<ShareAnalytics>,
    /// Base URL for generating share URLs
    base_url: String,
    /// Audit logger for compliance
    audit_logger: Option<Arc<AuditLogger>>,
    /// Access control system for RBAC
    access_control: Arc<AccessControl>,
    /// Key manager for enterprise encryption
    key_manager: Option<KeyManager>,
}

impl ShareService {
    /// Create a new share service with default base URL
    pub fn new() -> Self {
        Self {
            shares: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            analytics: Arc::new(ShareAnalytics::new()),
            base_url: "https://ricecoder.com".to_string(),
            audit_logger: None,
            access_control: Arc::new(AccessControl::new()),
            key_manager: None,
        }
    }

    /// Create a new share service with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            shares: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            analytics: Arc::new(ShareAnalytics::new()),
            base_url,
            audit_logger: None,
            access_control: Arc::new(AccessControl::new()),
            key_manager: None,
        }
    }

    /// Create a new share service with audit logging
    pub fn with_audit_logging(base_url: String, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            shares: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            analytics: Arc::new(ShareAnalytics::new()),
            base_url,
            audit_logger: Some(audit_logger),
            access_control: Arc::new(AccessControl::new()),
            key_manager: None,
        }
    }

    /// Create a new share service with access control
    pub fn with_access_control(base_url: String, access_control: Arc<AccessControl>) -> Self {
        Self {
            shares: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            analytics: Arc::new(ShareAnalytics::new()),
            base_url,
            audit_logger: None,
            access_control,
            key_manager: None,
        }
    }

    /// Create a new share service with full enterprise features
    pub fn with_enterprise_features(
        base_url: String,
        audit_logger: Arc<AuditLogger>,
        access_control: Arc<AccessControl>,
    ) -> Self {
        Self {
            shares: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            analytics: Arc::new(ShareAnalytics::new()),
            base_url,
            audit_logger: Some(audit_logger),
            access_control,
            key_manager: None,
        }
    }

    /// Set the key manager for enterprise encryption
    pub fn set_key_manager(&mut self, key_manager: KeyManager) {
        self.key_manager = Some(key_manager);
    }

    /// Generate a share link for a session with optional expiration
    pub fn generate_share_link(
        &self,
        session_id: &str,
        permissions: SharePermissions,
        expires_in: Option<Duration>,
    ) -> SessionResult<SessionShare> {
        self.generate_share_link_with_policy(session_id, permissions, expires_in, None, None)
    }

    /// Generate a share link with enterprise policy
    pub fn generate_share_link_with_policy(
        &self,
        session_id: &str,
        permissions: SharePermissions,
        expires_in: Option<Duration>,
        policy: Option<EnterpriseSharingPolicy>,
        creator_user_id: Option<String>,
    ) -> SessionResult<SessionShare> {
        // Check RBAC permissions for session sharing
        if let Some(ref user_id) = creator_user_id {
            let principal = Principal {
                id: user_id.clone(),
                roles: vec!["user".to_string()], // In production, load from user store
                attributes: HashMap::new(),
            };

            // Check if user has permission to share sessions
            if !self
                .access_control
                .has_permission(&principal, &Permission::SessionShare)
            {
                return Err(SessionError::PermissionDenied(
                    "User does not have permission to share sessions".to_string(),
                ));
            }

            // Additional checks for enterprise policies
            if let Some(ref policy) = policy {
                // Check if user can create shares with this data classification
                match policy.data_classification {
                    DataClassification::Restricted => {
                        if !self
                            .access_control
                            .has_permission(&principal, &Permission::Admin)
                        {
                            return Err(SessionError::PermissionDenied(
                                "Only administrators can share restricted data".to_string(),
                            ));
                        }
                    }
                    DataClassification::Confidential => {
                        if !self.access_control.has_any_permission(
                            &principal,
                            &[Permission::Admin, Permission::SessionShare],
                        ) {
                            return Err(SessionError::PermissionDenied(
                                "Insufficient permissions for sharing confidential data"
                                    .to_string(),
                            ));
                        }
                    }
                    _ => {} // Public and Internal allow sharing with SessionShare permission
                }
            }
        } else {
            // Anonymous sharing not allowed for enterprise features
            return Err(SessionError::PermissionDenied(
                "User authentication required for session sharing".to_string(),
            ));
        }

        let share_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Apply enterprise governance policies
        let effective_policy = if self.has_enterprise_features() {
            let session_data = serde_json::json!({
                "session_id": session_id,
                "permissions": &permissions,
                "creator_user_id": &creator_user_id
            });
            Some(self.apply_governance_policies(
                creator_user_id.as_ref().unwrap(),
                &session_data,
                policy.as_ref(),
            )?)
        } else {
            policy.clone()
        };

        // Apply enterprise policy constraints
        let effective_expires_at = if let Some(ref policy) = effective_policy {
            if let Some(max_days) = policy.max_expiration_days {
                let max_expires = now + Duration::days(max_days);
                expires_in
                    .map(|duration| {
                        let requested_expires = now + duration;
                        if requested_expires > max_expires {
                            max_expires
                        } else {
                            requested_expires
                        }
                    })
                    .or(Some(max_expires))
            } else {
                expires_in.map(|duration| now + duration)
            }
        } else {
            expires_in.map(|duration| now + duration)
        };

        // Generate share URL
        let share_url = self.generate_share_url(&share_id)?;

        // Encrypt session data for enterprise security if key manager is available
        let encrypted_session_data = if let Some(ref key_manager) = self.key_manager {
            // Serialize session data for encryption
            let session_data = serde_json::json!({
                "session_id": session_id,
                "permissions": permissions,
                "policy": policy,
                "creator_user_id": creator_user_id
            });
            let data_str = serde_json::to_string(&session_data).map_err(|e| {
                SessionError::Invalid(format!("Failed to serialize session data: {}", e))
            })?;

            Some(key_manager.encrypt_api_key(&data_str).map_err(|e| {
                SessionError::Invalid(format!("Failed to encrypt session data: {}", e))
            })?)
        } else {
            None
        };

        let share = SessionShare {
            id: share_id.clone(),
            share_url: Some(share_url),
            session_id: session_id.to_string(),
            created_at: now,
            expires_at: effective_expires_at,
            permissions: permissions.clone(),
            policy: effective_policy.clone(),
            creator_user_id: creator_user_id.clone(),
            encrypted_session_data: encrypted_session_data.clone(),
        };

        // Store the share
        let mut shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;
        shares.insert(share_id.clone(), share.clone());

        // Track analytics
        self.analytics.record_share_created(&share);

        // Log compliance event with governance details
        if let Some(ref audit_logger) = self.audit_logger {
            let permissions_clone = permissions.clone();
            let policy_applied = effective_policy.is_some();
            let governance_applied = self.has_enterprise_features();
            let session_id_clone = session_id.to_string();
            let share_id_clone = share_id.clone();
            let event = AuditEvent {
                event_type: AuditEventType::DataAccess,
                user_id: creator_user_id.clone(),
                session_id: Some(session_id_clone),
                action: "share_created".to_string(),
                resource: format!("session:{}", session_id),
                metadata: serde_json::json!({
                    "share_id": share_id_clone,
                    "permissions": permissions_clone,
                    "expires_at": effective_expires_at,
                    "policy_applied": policy_applied,
                    "governance_applied": governance_applied,
                    "data_classification": effective_policy.as_ref().map(|p| format!("{:?}", p.data_classification)),
                    "encryption_used": encrypted_session_data.is_some()
                }),
            };
            // Note: In a real implementation, this would be async
            // For now, we'll log synchronously
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        Ok(share)
    }

    /// Generate a share URL from share ID
    fn generate_share_url(&self, share_id: &str) -> SessionResult<String> {
        let base_url = Url::parse(&self.base_url)
            .map_err(|e| SessionError::Invalid(format!("Invalid base URL: {}", e)))?;

        let share_url = base_url
            .join(&format!("share/{}", share_id))
            .map_err(|e| SessionError::Invalid(format!("Failed to generate share URL: {}", e)))?;

        Ok(share_url.to_string())
    }

    /// Validate and extract share ID from URL
    pub fn validate_share_url(&self, share_url: &str) -> SessionResult<String> {
        let url = Url::parse(share_url)
            .map_err(|e| SessionError::Invalid(format!("Invalid share URL: {}", e)))?;

        // Ensure the URL is from our domain
        let base_url = Url::parse(&self.base_url)
            .map_err(|e| SessionError::Invalid(format!("Invalid base URL: {}", e)))?;

        if url.host_str() != base_url.host_str() {
            return Err(SessionError::Invalid(
                "Share URL from unauthorized domain".to_string(),
            ));
        }

        // Extract share ID from path
        let path_segments: Vec<&str> = url
            .path_segments()
            .ok_or_else(|| SessionError::Invalid("Invalid share URL path".to_string()))?
            .collect();

        if path_segments.len() != 2 || path_segments[0] != "share" {
            return Err(SessionError::Invalid(
                "Invalid share URL format".to_string(),
            ));
        }

        let share_id = path_segments[1].to_string();

        // Validate share ID format (UUID)
        Uuid::parse_str(&share_id)
            .map_err(|_| SessionError::Invalid("Invalid share ID format".to_string()))?;

        Ok(share_id)
    }

    /// Get a share by URL, checking expiration
    pub fn get_share_by_url(&self, share_url: &str) -> SessionResult<SessionShare> {
        let share_id = self.validate_share_url(share_url)?;
        let share = self.get_share(&share_id)?;

        // Record URL access for analytics
        self.analytics
            .record_url_accessed(&share_id, &share.session_id, &share.permissions);

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

        // Log access for compliance
        if let Some(ref audit_logger) = self.audit_logger {
            let session_id_clone = share.session_id.clone();
            let permissions_clone = share.permissions.clone();
            let share_id_clone = share_id.to_string();
            let event = AuditEvent {
                event_type: AuditEventType::DataAccess,
                user_id: share.creator_user_id.clone(),
                session_id: Some(session_id_clone.clone()),
                action: "share_accessed".to_string(),
                resource: format!("share:{}", share_id),
                metadata: serde_json::json!({
                    "share_id": share_id_clone,
                    "session_id": session_id_clone,
                    "permissions": permissions_clone
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
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
        self.analytics
            .record_share_accessed(&session.id, permissions);

        shared_session
    }

    /// Import a shared session, creating a new local session
    pub fn import_shared_session(
        &self,
        share_id: &str,
        shared_session: &Session,
        importer_user_id: Option<String>,
    ) -> SessionResult<Session> {
        // Verify the share exists and hasn't expired
        let share = self.get_share(share_id)?;

        // Create a new local session with the shared session's data
        let mut imported_session = shared_session.clone();

        // Generate a new ID for the imported session
        imported_session.id = Uuid::new_v4().to_string();

        // Update timestamps
        let now = Utc::now();
        imported_session.created_at = now;
        imported_session.updated_at = now;

        // Log import for compliance
        if let Some(ref audit_logger) = self.audit_logger {
            let original_session_id = share.session_id.clone();
            let imported_session_id = imported_session.id.clone();
            let share_id_clone = share_id.to_string();
            let event = AuditEvent {
                event_type: AuditEventType::DataAccess,
                user_id: importer_user_id.or_else(|| share.creator_user_id.clone()),
                session_id: Some(imported_session_id.clone()),
                action: "session_imported".to_string(),
                resource: format!("session:{}:from_share:{}", imported_session_id, share_id),
                metadata: serde_json::json!({
                    "share_id": share_id_clone,
                    "original_session_id": original_session_id,
                    "imported_session_id": imported_session_id
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        Ok(imported_session)
    }

    /// Revoke a share by ID
    pub fn revoke_share(
        &self,
        share_id: &str,
        revoker_user_id: Option<String>,
    ) -> SessionResult<()> {
        let mut shares = self
            .shares
            .lock()
            .map_err(|e| SessionError::StorageError(format!("Failed to lock shares: {}", e)))?;

        let share = shares
            .remove(share_id)
            .ok_or_else(|| SessionError::ShareNotFound(share_id.to_string()))?;

        // Log revocation for compliance
        if let Some(ref audit_logger) = self.audit_logger {
            let session_id_clone = share.session_id.clone();
            let share_id_clone = share_id.to_string();
            let event = AuditEvent {
                event_type: AuditEventType::DataAccess,
                user_id: revoker_user_id.or_else(|| share.creator_user_id),
                session_id: Some(session_id_clone.clone()),
                action: "share_revoked".to_string(),
                resource: format!("share:{}", share_id),
                metadata: serde_json::json!({
                    "share_id": share_id_clone,
                    "session_id": session_id_clone
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

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

impl Clone for ShareService {
    fn clone(&self) -> Self {
        // Note: KeyManager is not cloneable, so we create a new instance without it
        // In production, KeyManager should be shared via Arc
        Self {
            shares: self.shares.clone(),
            analytics: self.analytics.clone(),
            base_url: self.base_url.clone(),
            audit_logger: self.audit_logger.clone(),
            access_control: self.access_control.clone(),
            key_manager: None, // Cannot clone KeyManager
        }
    }
}

impl ShareService {
    /// Set the base URL for share links
    pub fn set_base_url(&mut self, base_url: String) {
        self.base_url = base_url;
    }

    /// Set the audit logger
    pub fn set_audit_logger(&mut self, audit_logger: Arc<AuditLogger>) {
        self.audit_logger = Some(audit_logger);
    }

    /// Check if enterprise features are enabled
    pub fn has_enterprise_features(&self) -> bool {
        self.audit_logger.is_some() && self.key_manager.is_some()
    }

    /// Validate enterprise sharing policy
    pub fn validate_enterprise_policy(
        &self,
        policy: &EnterpriseSharingPolicy,
    ) -> SessionResult<()> {
        // Ensure policy doesn't allow excessively long expirations
        if let Some(max_days) = policy.max_expiration_days {
            if max_days > 365 {
                // Max 1 year
                return Err(SessionError::Invalid(
                    "Maximum expiration cannot exceed 365 days".to_string(),
                ));
            }
        }

        // Ensure data classification is appropriate
        match policy.data_classification {
            DataClassification::Public
            | DataClassification::Internal
            | DataClassification::Confidential
            | DataClassification::Restricted => {
                // Valid classifications
            }
        }

        Ok(())
    }

    /// Apply enterprise governance policies to sharing
    pub fn apply_governance_policies(
        &self,
        user_id: &str,
        session_data: &serde_json::Value,
        requested_policy: Option<&EnterpriseSharingPolicy>,
    ) -> SessionResult<EnterpriseSharingPolicy> {
        // Get user's principal for governance checks
        let principal = Principal {
            id: user_id.to_string(),
            roles: vec!["user".to_string()], // In production, load from user store
            attributes: HashMap::new(),
        };

        // Start with default policy
        let mut policy = EnterpriseSharingPolicy {
            max_expiration_days: Some(30), // Default 30 days
            requires_approval: false,
            allowed_domains: vec![],
            compliance_logging: true,
            data_classification: DataClassification::Internal,
        };

        // Apply user role-based governance
        if self
            .access_control
            .has_permission(&principal, &Permission::Admin)
        {
            // Admins can share with longer expirations and higher classifications
            policy.max_expiration_days = Some(365);
            policy.data_classification = DataClassification::Confidential;
        } else if self
            .access_control
            .has_permission(&principal, &Permission::SessionShare)
        {
            // Regular users with sharing permission
            policy.max_expiration_days = Some(90);
            policy.data_classification = DataClassification::Internal;
        } else {
            return Err(SessionError::PermissionDenied(
                "User does not have permission to share sessions".to_string(),
            ));
        }

        // Apply session data classification based on content analysis
        if let Some(content) = session_data.get("content") {
            if content.to_string().contains("password") || content.to_string().contains("secret") {
                policy.data_classification = DataClassification::Confidential;
                policy.requires_approval = true;
            }
        }

        // Override with requested policy if user has sufficient permissions
        if let Some(requested) = requested_policy {
            // Validate requested policy against user's permissions
            match requested.data_classification {
                DataClassification::Restricted => {
                    if !self
                        .access_control
                        .has_permission(&principal, &Permission::Admin)
                    {
                        return Err(SessionError::PermissionDenied(
                            "Only administrators can set restricted data classification"
                                .to_string(),
                        ));
                    }
                }
                DataClassification::Confidential => {
                    if !self.access_control.has_any_permission(
                        &principal,
                        &[Permission::Admin, Permission::SessionShare],
                    ) {
                        return Err(SessionError::PermissionDenied(
                            "Insufficient permissions for confidential data classification"
                                .to_string(),
                        ));
                    }
                }
                _ => {} // Allow lower classifications
            }

            // Apply requested policy settings within governance bounds
            if let Some(req_max_days) = requested.max_expiration_days {
                if req_max_days <= policy.max_expiration_days.unwrap_or(30) {
                    policy.max_expiration_days = Some(req_max_days);
                } else {
                    return Err(SessionError::Invalid(format!(
                        "Requested expiration exceeds governance limit of {} days",
                        policy.max_expiration_days.unwrap_or(30)
                    )));
                }
            }

            policy.requires_approval = requested.requires_approval || policy.requires_approval;
            policy.allowed_domains = requested.allowed_domains.clone();
            policy.compliance_logging = requested.compliance_logging;
            policy.data_classification = requested.data_classification;
        }

        Ok(policy)
    }
}

/// A shared session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionShare {
    /// Unique identifier for the share
    pub id: String,
    /// Share URL for URL-based sharing
    pub share_url: Option<String>,
    /// ID of the shared session
    pub session_id: String,
    /// When the share was created
    pub created_at: DateTime<Utc>,
    /// When the share expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Share permissions
    pub permissions: SharePermissions,
    /// Enterprise sharing policy
    pub policy: Option<EnterpriseSharingPolicy>,
    /// Creator user ID for enterprise tracking
    pub creator_user_id: Option<String>,
    /// Encrypted session data for enterprise security
    pub encrypted_session_data: Option<EncryptedData>,
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

/// Enterprise sharing policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSharingPolicy {
    /// Maximum expiration time for shares
    pub max_expiration_days: Option<i64>,
    /// Whether to require approval for sharing
    pub requires_approval: bool,
    /// Allowed domains for sharing
    pub allowed_domains: Vec<String>,
    /// Whether to enable compliance logging
    pub compliance_logging: bool,
    /// Data classification level
    pub data_classification: DataClassification,
}

/// Data classification levels for compliance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClassification {
    /// Public data - no restrictions
    Public,
    /// Internal data - basic restrictions
    Internal,
    /// Confidential data - strict controls
    Confidential,
    /// Restricted data - highest security
    Restricted,
}

/// Analytics data for session sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareAnalyticsData {
    /// Total number of shares created
    pub total_shares_created: u64,
    /// Total number of share accesses
    pub total_share_accesses: u64,
    /// Total number of URL-based accesses
    pub total_url_accesses: u64,
    /// Number of shares by permission type
    pub shares_by_permissions: HashMap<String, u64>,
    /// Top accessed sessions
    pub top_accessed_sessions: Vec<(String, u64)>,
    /// Recent share activity
    pub recent_activity: Vec<ShareActivity>,
    /// Enterprise compliance metrics
    pub enterprise_metrics: Option<EnterpriseShareMetrics>,
}

/// Enterprise-specific sharing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseShareMetrics {
    /// Shares by data classification
    pub shares_by_classification: HashMap<String, u64>,
    /// Compliance events logged
    pub compliance_events: u64,
    /// Policy violations detected
    pub policy_violations: u64,
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
    total_url_accesses: AtomicU64,
    shares_by_permissions: std::sync::Mutex<HashMap<String, u64>>,
    session_access_counts: std::sync::Mutex<HashMap<String, u64>>,
    recent_activity: std::sync::Mutex<Vec<ShareActivity>>,
    enterprise_metrics: std::sync::Mutex<Option<EnterpriseShareMetrics>>,
}

impl ShareAnalytics {
    fn new() -> Self {
        Self {
            total_shares_created: AtomicU64::new(0),
            total_share_accesses: AtomicU64::new(0),
            total_url_accesses: AtomicU64::new(0),
            shares_by_permissions: std::sync::Mutex::new(HashMap::new()),
            session_access_counts: std::sync::Mutex::new(HashMap::new()),
            recent_activity: std::sync::Mutex::new(Vec::new()),
            enterprise_metrics: std::sync::Mutex::new(Some(EnterpriseShareMetrics {
                shares_by_classification: HashMap::new(),
                compliance_events: 0,
                policy_violations: 0,
            })),
        }
    }

    fn record_share_created(&self, share: &SessionShare) {
        self.total_shares_created.fetch_add(1, Ordering::Relaxed);

        let perm_key = format!(
            "ro={},hist={},ctx={}",
            share.permissions.read_only,
            share.permissions.include_history,
            share.permissions.include_context
        );

        if let Ok(mut perms) = self.shares_by_permissions.lock() {
            *perms.entry(perm_key).or_insert(0) += 1;
        }

        // Track enterprise metrics
        if let Ok(mut metrics) = self.enterprise_metrics.lock() {
            if let Some(ref mut enterprise_metrics) = *metrics {
                if let Some(ref policy) = share.policy {
                    let classification_key = format!("{:?}", policy.data_classification);
                    *enterprise_metrics
                        .shares_by_classification
                        .entry(classification_key)
                        .or_insert(0) += 1;
                }
            }
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

    fn record_url_accessed(
        &self,
        share_id: &str,
        session_id: &str,
        permissions: &SharePermissions,
    ) {
        self.total_url_accesses.fetch_add(1, Ordering::Relaxed);
        self.record_share_accessed(session_id, permissions);
    }

    fn record_compliance_event(&self) {
        if let Ok(mut metrics) = self.enterprise_metrics.lock() {
            if let Some(ref mut enterprise_metrics) = *metrics {
                enterprise_metrics.compliance_events += 1;
            }
        }
    }

    fn record_policy_violation(&self) {
        if let Ok(mut metrics) = self.enterprise_metrics.lock() {
            if let Some(ref mut enterprise_metrics) = *metrics {
                enterprise_metrics.policy_violations += 1;
            }
        }
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
        let total_url_accesses = self.total_url_accesses.load(Ordering::Relaxed);

        let shares_by_permissions = self
            .shares_by_permissions
            .lock()
            .map(|perms| perms.clone())
            .unwrap_or_default();

        let session_counts = self
            .session_access_counts
            .lock()
            .map(|counts| {
                let mut vec: Vec<(String, u64)> =
                    counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
                vec.sort_by(|a, b| b.1.cmp(&a.1));
                vec.truncate(10); // Top 10
                vec
            })
            .unwrap_or_default();

        let recent_activity = self
            .recent_activity
            .lock()
            .map(|activities| activities.clone())
            .unwrap_or_default();

        let enterprise_metrics = self
            .enterprise_metrics
            .lock()
            .map(|metrics| metrics.clone())
            .unwrap_or_default();

        ShareAnalyticsData {
            total_shares_created: total_shares,
            total_share_accesses: total_accesses,
            total_url_accesses,
            shares_by_permissions,
            top_accessed_sessions: session_counts,
            recent_activity,
            enterprise_metrics,
        }
    }

    fn reset(&self) {
        self.total_shares_created.store(0, Ordering::Relaxed);
        self.total_share_accesses.store(0, Ordering::Relaxed);
        self.total_url_accesses.store(0, Ordering::Relaxed);

        if let Ok(mut perms) = self.shares_by_permissions.lock() {
            perms.clear();
        }

        if let Ok(mut counts) = self.session_access_counts.lock() {
            counts.clear();
        }

        if let Ok(mut activities) = self.recent_activity.lock() {
            activities.clear();
        }

        if let Ok(mut metrics) = self.enterprise_metrics.lock() {
            *metrics = Some(EnterpriseShareMetrics {
                shares_by_classification: HashMap::new(),
                compliance_events: 0,
                policy_violations: 0,
            });
        }
    }
}
