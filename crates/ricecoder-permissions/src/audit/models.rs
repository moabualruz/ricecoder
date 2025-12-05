//! Audit log data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Action recorded in audit log
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditAction {
    /// Tool execution was allowed
    Allowed,
    /// Tool execution was denied
    Denied,
    /// User was prompted for permission
    Prompted,
    /// User approved tool execution
    Approved,
    /// User rejected tool execution
    Rejected,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Allowed => write!(f, "allowed"),
            AuditAction::Denied => write!(f, "denied"),
            AuditAction::Prompted => write!(f, "prompted"),
            AuditAction::Approved => write!(f, "approved"),
            AuditAction::Rejected => write!(f, "rejected"),
        }
    }
}

/// Result of a permission check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditResult {
    /// Tool execution succeeded
    Success,
    /// Tool execution was blocked
    Blocked,
    /// User cancelled the operation
    Cancelled,
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditResult::Success => write!(f, "success"),
            AuditResult::Blocked => write!(f, "blocked"),
            AuditResult::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Entry in the audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique identifier for this log entry
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Name of the tool being accessed
    pub tool: String,
    /// Action that was taken
    pub action: AuditAction,
    /// Result of the action
    pub result: AuditResult,
    /// Optional agent identifier
    pub agent: Option<String>,
    /// Optional additional context
    pub context: Option<String>,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(tool: String, action: AuditAction, result: AuditResult) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            tool,
            action,
            result,
            agent: None,
            context: None,
        }
    }

    /// Create a new audit log entry with agent
    pub fn with_agent(
        tool: String,
        action: AuditAction,
        result: AuditResult,
        agent: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            tool,
            action,
            result,
            agent: Some(agent),
            context: None,
        }
    }

    /// Add context to the log entry
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_action_display() {
        assert_eq!(AuditAction::Allowed.to_string(), "allowed");
        assert_eq!(AuditAction::Denied.to_string(), "denied");
        assert_eq!(AuditAction::Prompted.to_string(), "prompted");
        assert_eq!(AuditAction::Approved.to_string(), "approved");
        assert_eq!(AuditAction::Rejected.to_string(), "rejected");
    }

    #[test]
    fn test_audit_result_display() {
        assert_eq!(AuditResult::Success.to_string(), "success");
        assert_eq!(AuditResult::Blocked.to_string(), "blocked");
        assert_eq!(AuditResult::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_audit_log_entry_creation() {
        let entry = AuditLogEntry::new(
            "test_tool".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        );

        assert_eq!(entry.tool, "test_tool");
        assert_eq!(entry.action, AuditAction::Allowed);
        assert_eq!(entry.result, AuditResult::Success);
        assert_eq!(entry.agent, None);
        assert_eq!(entry.context, None);
        assert!(!entry.id.is_empty());
    }

    #[test]
    fn test_audit_log_entry_with_agent() {
        let entry = AuditLogEntry::with_agent(
            "test_tool".to_string(),
            AuditAction::Denied,
            AuditResult::Blocked,
            "agent1".to_string(),
        );

        assert_eq!(entry.tool, "test_tool");
        assert_eq!(entry.action, AuditAction::Denied);
        assert_eq!(entry.result, AuditResult::Blocked);
        assert_eq!(entry.agent, Some("agent1".to_string()));
    }

    #[test]
    fn test_audit_log_entry_with_context() {
        let entry = AuditLogEntry::new(
            "test_tool".to_string(),
            AuditAction::Prompted,
            AuditResult::Success,
        )
        .with_context("User approved after 5 seconds".to_string());

        assert_eq!(
            entry.context,
            Some("User approved after 5 seconds".to_string())
        );
    }

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry::new(
            "test_tool".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        );

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditLogEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tool, entry.tool);
        assert_eq!(deserialized.action, entry.action);
        assert_eq!(deserialized.result, entry.result);
    }

    #[test]
    fn test_audit_log_entry_timestamp() {
        let before = Utc::now();
        let entry = AuditLogEntry::new(
            "test_tool".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        );
        let after = Utc::now();

        assert!(entry.timestamp >= before);
        assert!(entry.timestamp <= after);
    }
}
