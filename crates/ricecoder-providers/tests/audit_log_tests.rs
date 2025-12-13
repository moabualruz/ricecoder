use ricecoder_providers::*;

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