//! Security entities for session isolation and SOC 2 compliance

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::{Permission, UserRole};

/// Security context for session isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: Option<String>,
    pub role: Option<UserRole>,
    pub permissions: Vec<Permission>,
    pub isolation_level: IsolationLevel,
    pub encryption_enabled: bool,
    pub confidentiality_level: ConfidentialityLevel,
}

impl SecurityContext {
    /// Check if the context has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.implies(permission))
    }

    /// Check if the context has a specific role
    pub fn has_role(&self, role: &UserRole) -> bool {
        self.role.as_ref() == Some(role)
    }

    /// Grant additional permissions
    pub fn grant_permission(&mut self, permission: Permission) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// Revoke a permission
    pub fn revoke_permission(&mut self, permission: &Permission) {
        self.permissions.retain(|p| p != permission);
    }

    /// Check if can access data of given confidentiality level
    pub fn can_access_confidentiality(&self, level: &ConfidentialityLevel) -> bool {
        match (self.role, level) {
            (Some(UserRole::Admin), _) => true,
            (_, ConfidentialityLevel::Public) => true,
            (Some(UserRole::Developer), ConfidentialityLevel::Internal) => true,
            (Some(UserRole::Developer), ConfidentialityLevel::Confidential) => true,
            (Some(UserRole::Analyst), ConfidentialityLevel::Internal) => true,
            _ => false,
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            user_id: None,
            role: Some(UserRole::Guest),
            permissions: UserRole::Guest.default_permissions(),
            isolation_level: IsolationLevel::Standard,
            encryption_enabled: false,
            confidentiality_level: ConfidentialityLevel::Public,
        }
    }
}

/// Isolation level for micro-segmentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    Standard,
    High,
    Critical,
}

/// Confidentiality level for data protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidentialityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Access event for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub resource: String,
    pub success: bool,
    pub details: Option<String>,
}

/// Security event for comprehensive SOC 2 audit trails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub subject: Option<SecuritySubject>,
    pub resource: Option<SecurityResource>,
    pub action: Option<String>,
    pub success: bool,
    pub details: HashMap<String, String>,
    pub compliance_flags: Vec<ComplianceFlag>,
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(event_type: SecurityEventType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            subject: None,
            resource: None,
            action: None,
            success: true,
            details: HashMap::new(),
            compliance_flags: Vec::new(),
        }
    }

    /// Set subject
    pub fn with_subject(mut self, subject: SecuritySubject) -> Self {
        self.subject = Some(subject);
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: SecurityResource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set action
    pub fn with_action(mut self, action: String) -> Self {
        self.action = Some(action);
        self
    }

    /// Mark as failed
    pub fn failed(mut self) -> Self {
        self.success = false;
        self
    }

    /// Add detail
    pub fn add_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Add compliance flag
    pub fn add_compliance_flag(mut self, flag: ComplianceFlag) -> Self {
        self.compliance_flags.push(flag);
        self
    }
}

/// Security event types for SOC 2 compliance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEventType {
    AccessGranted,
    AccessDenied,
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationSuccess,
    AuthorizationFailure,
    SessionCreated,
    SessionEnded,
    SessionTimeout,
    DataAccess,
    DataModification,
    SecurityViolation,
    ComplianceCheck,
}

/// Security subject (who performed the action)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySubject {
    pub id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Security resource (what was accessed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResource {
    pub resource_type: String,
    pub resource_id: String,
    pub attributes: HashMap<String, String>,
}

/// Compliance flags for SOC 2 requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceFlag {
    Soc2AccessControl,
    Soc2AuditTrail,
    GdprDataAccess,
    HipaaProtectedData,
}

/// Security alert for monitoring and alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub alert_type: SecurityAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub details: HashMap<String, String>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl SecurityAlert {
    /// Create a new security alert
    pub fn new(alert_type: SecurityAlertType, severity: AlertSeverity, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            alert_type,
            severity,
            message,
            details: HashMap::new(),
            resolved: false,
            resolved_at: None,
        }
    }

    /// Add detail
    pub fn add_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Mark as resolved
    pub fn resolve(mut self) -> Self {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
        self
    }
}

/// Types of security alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityAlertType {
    UnauthorizedAccess,
    SuspiciousActivity,
    ComplianceViolation,
    SecurityBreach,
    AuditFailure,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}
