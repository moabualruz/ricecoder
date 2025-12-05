//! Audit logging for security events
//!
//! This module provides audit logging functionality for tracking security-relevant events
//! such as API key access, authentication attempts, and permission decisions.

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::info;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    /// API key accessed
    ApiKeyAccessed,
    /// API key rotated
    ApiKeyRotated,
    /// Authentication attempt
    AuthenticationAttempt,
    /// Authorization decision
    AuthorizationDecision,
    /// Configuration loaded
    ConfigurationLoaded,
    /// File accessed
    FileAccessed,
    /// File modified
    FileModified,
    /// Permission denied
    PermissionDenied,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Security error
    SecurityError,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp (ISO 8601 format)
    pub timestamp: String,
    /// Event type
    pub event_type: AuditEventType,
    /// Provider or component name
    pub component: String,
    /// User or service performing the action
    pub actor: String,
    /// Resource being accessed
    pub resource: String,
    /// Action result (success/failure)
    pub result: String,
    /// Additional details
    pub details: String,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        component: &str,
        actor: &str,
        resource: &str,
        result: &str,
        details: &str,
    ) -> Self {
        let timestamp = chrono::Local::now().to_rfc3339();
        Self {
            timestamp,
            event_type,
            component: component.to_string(),
            actor: actor.to_string(),
            resource: resource.to_string(),
            result: result.to_string(),
            details: details.to_string(),
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Audit logger for recording security events
pub struct AuditLogger {
    /// Path to audit log file
    log_path: PathBuf,
    /// Lock for thread-safe file access
    lock: Mutex<()>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_path: PathBuf) -> Self {
        Self {
            log_path,
            lock: Mutex::new(()),
        }
    }

    /// Log an audit event
    pub fn log(&self, entry: &AuditLogEntry) -> Result<(), Box<dyn std::error::Error>> {
        let _guard = self.lock.lock().unwrap();

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        // Write JSON entry
        let json = entry.to_json()?;
        writeln!(file, "{}", json)?;

        // Also log to tracing
        info!(
            event_type = ?entry.event_type,
            component = %entry.component,
            actor = %entry.actor,
            resource = %entry.resource,
            result = %entry.result,
            "Audit event logged"
        );

        Ok(())
    }

    /// Log API key access
    pub fn log_api_key_access(
        &self,
        provider: &str,
        actor: &str,
        result: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = AuditLogEntry::new(
            AuditEventType::ApiKeyAccessed,
            "providers",
            actor,
            provider,
            result,
            "API key accessed",
        );
        self.log(&entry)
    }

    /// Log API key rotation
    pub fn log_api_key_rotation(
        &self,
        provider: &str,
        actor: &str,
        result: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = AuditLogEntry::new(
            AuditEventType::ApiKeyRotated,
            "providers",
            actor,
            provider,
            result,
            "API key rotated",
        );
        self.log(&entry)
    }

    /// Log authentication attempt
    pub fn log_authentication_attempt(
        &self,
        provider: &str,
        actor: &str,
        result: &str,
        details: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = AuditLogEntry::new(
            AuditEventType::AuthenticationAttempt,
            "providers",
            actor,
            provider,
            result,
            details,
        );
        self.log(&entry)
    }

    /// Log authorization decision
    pub fn log_authorization_decision(
        &self,
        resource: &str,
        actor: &str,
        allowed: bool,
        details: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let result = if allowed { "allowed" } else { "denied" };
        let entry = AuditLogEntry::new(
            AuditEventType::AuthorizationDecision,
            "permissions",
            actor,
            resource,
            result,
            details,
        );
        self.log(&entry)
    }

    /// Log rate limit exceeded
    pub fn log_rate_limit_exceeded(
        &self,
        provider: &str,
        actor: &str,
        details: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = AuditLogEntry::new(
            AuditEventType::RateLimitExceeded,
            "providers",
            actor,
            provider,
            "rate_limit_exceeded",
            details,
        );
        self.log(&entry)
    }

    /// Log security error
    pub fn log_security_error(
        &self,
        component: &str,
        actor: &str,
        resource: &str,
        error: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = AuditLogEntry::new(
            AuditEventType::SecurityError,
            component,
            actor,
            resource,
            "error",
            error,
        );
        self.log(&entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_log_entry_creation() {
        let entry = AuditLogEntry::new(
            AuditEventType::ApiKeyAccessed,
            "providers",
            "system",
            "openai",
            "success",
            "API key accessed",
        );

        assert_eq!(entry.event_type, AuditEventType::ApiKeyAccessed);
        assert_eq!(entry.component, "providers");
        assert_eq!(entry.actor, "system");
        assert_eq!(entry.resource, "openai");
        assert_eq!(entry.result, "success");
    }

    #[test]
    fn test_audit_log_entry_to_json() {
        let entry = AuditLogEntry::new(
            AuditEventType::ApiKeyAccessed,
            "providers",
            "system",
            "openai",
            "success",
            "API key accessed",
        );

        let json = entry.to_json().unwrap();
        assert!(json.contains("ApiKeyAccessed"));
        assert!(json.contains("providers"));
        assert!(json.contains("openai"));
    }

    #[test]
    fn test_audit_logger_log() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone());
        let entry = AuditLogEntry::new(
            AuditEventType::ApiKeyAccessed,
            "providers",
            "system",
            "openai",
            "success",
            "API key accessed",
        );

        let result = logger.log(&entry);
        assert!(result.is_ok());

        // Verify file was created and contains entry
        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("ApiKeyAccessed"));
    }

    #[test]
    fn test_audit_logger_log_api_key_access() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone());
        let result = logger.log_api_key_access("openai", "system", "success");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("ApiKeyAccessed"));
        assert!(content.contains("openai"));
    }

    #[test]
    fn test_audit_logger_log_authentication_attempt() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone());
        let result = logger.log_authentication_attempt("openai", "system", "success", "Valid API key");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("AuthenticationAttempt"));
    }

    #[test]
    fn test_audit_logger_log_authorization_decision() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone());
        let result = logger.log_authorization_decision("tool:read_file", "system", true, "Permission granted");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("AuthorizationDecision"));
        assert!(content.contains("allowed"));
    }

    #[test]
    fn test_audit_logger_log_rate_limit_exceeded() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone());
        let result = logger.log_rate_limit_exceeded("openai", "system", "Rate limit: 10 req/sec");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("RateLimitExceeded"));
    }

    #[test]
    fn test_audit_logger_multiple_entries() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone());

        logger.log_api_key_access("openai", "system", "success").unwrap();
        logger.log_api_key_access("anthropic", "system", "success").unwrap();
        logger.log_authentication_attempt("openai", "system", "success", "Valid key").unwrap();

        let content = std::fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }
}
