//! Audit logger implementation

use super::models::{AuditAction, AuditLogEntry, AuditResult};
use std::sync::{Arc, RwLock};

/// Audit logger for recording permission checks and denials
#[derive(Clone)]
pub struct AuditLogger {
    entries: Arc<RwLock<Vec<AuditLogEntry>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log a tool execution
    pub fn log_execution(
        &self,
        tool: String,
        agent: Option<String>,
        context: Option<String>,
    ) -> Result<(), String> {
        let mut entry = AuditLogEntry::new(tool, AuditAction::Allowed, AuditResult::Success);

        if let Some(agent_name) = agent {
            entry.agent = Some(agent_name);
        }

        if let Some(ctx) = context {
            entry.context = Some(ctx);
        }

        let mut entries = self
            .entries
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        entries.push(entry);

        Ok(())
    }

    /// Log a tool denial
    pub fn log_denial(
        &self,
        tool: String,
        agent: Option<String>,
        context: Option<String>,
    ) -> Result<(), String> {
        let mut entry = AuditLogEntry::new(tool, AuditAction::Denied, AuditResult::Blocked);

        if let Some(agent_name) = agent {
            entry.agent = Some(agent_name);
        }

        if let Some(ctx) = context {
            entry.context = Some(ctx);
        }

        let mut entries = self
            .entries
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        entries.push(entry);

        Ok(())
    }

    /// Log a permission prompt
    pub fn log_prompt(
        &self,
        tool: String,
        agent: Option<String>,
        context: Option<String>,
    ) -> Result<(), String> {
        let mut entry = AuditLogEntry::new(tool, AuditAction::Prompted, AuditResult::Success);

        if let Some(agent_name) = agent {
            entry.agent = Some(agent_name);
        }

        if let Some(ctx) = context {
            entry.context = Some(ctx);
        }

        let mut entries = self
            .entries
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        entries.push(entry);

        Ok(())
    }

    /// Get all entries
    pub fn entries(&self) -> Result<Vec<AuditLogEntry>, String> {
        let entries = self
            .entries
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;
        Ok(entries.clone())
    }

    /// Get the number of entries
    pub fn len(&self) -> Result<usize, String> {
        let entries = self
            .entries
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;
        Ok(entries.len())
    }

    /// Check if the logger is empty
    pub fn is_empty(&self) -> Result<bool, String> {
        let entries = self
            .entries
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;
        Ok(entries.is_empty())
    }

    /// Clear all entries
    pub fn clear(&self) -> Result<(), String> {
        let mut entries = self
            .entries
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        entries.clear();
        Ok(())
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        assert!(logger.is_empty().unwrap());
        assert_eq!(logger.len().unwrap(), 0);
    }

    #[test]
    fn test_log_execution() {
        let logger = AuditLogger::new();
        let result = logger.log_execution("test_tool".to_string(), None, None);

        assert!(result.is_ok());
        assert_eq!(logger.len().unwrap(), 1);

        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].tool, "test_tool");
        assert_eq!(entries[0].action, AuditAction::Allowed);
        assert_eq!(entries[0].result, AuditResult::Success);
    }

    #[test]
    fn test_log_execution_with_agent() {
        let logger = AuditLogger::new();
        let result =
            logger.log_execution("test_tool".to_string(), Some("agent1".to_string()), None);

        assert!(result.is_ok());
        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].agent, Some("agent1".to_string()));
    }

    #[test]
    fn test_log_execution_with_context() {
        let logger = AuditLogger::new();
        let result = logger.log_execution(
            "test_tool".to_string(),
            None,
            Some("User context".to_string()),
        );

        assert!(result.is_ok());
        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].context, Some("User context".to_string()));
    }

    #[test]
    fn test_log_denial() {
        let logger = AuditLogger::new();
        let result = logger.log_denial("test_tool".to_string(), None, None);

        assert!(result.is_ok());
        assert_eq!(logger.len().unwrap(), 1);

        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].tool, "test_tool");
        assert_eq!(entries[0].action, AuditAction::Denied);
        assert_eq!(entries[0].result, AuditResult::Blocked);
    }

    #[test]
    fn test_log_denial_with_agent() {
        let logger = AuditLogger::new();
        let result = logger.log_denial("test_tool".to_string(), Some("agent1".to_string()), None);

        assert!(result.is_ok());
        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].agent, Some("agent1".to_string()));
    }

    #[test]
    fn test_log_prompt() {
        let logger = AuditLogger::new();
        let result = logger.log_prompt("test_tool".to_string(), None, None);

        assert!(result.is_ok());
        assert_eq!(logger.len().unwrap(), 1);

        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].tool, "test_tool");
        assert_eq!(entries[0].action, AuditAction::Prompted);
        assert_eq!(entries[0].result, AuditResult::Success);
    }

    #[test]
    fn test_log_prompt_with_agent() {
        let logger = AuditLogger::new();
        let result = logger.log_prompt("test_tool".to_string(), Some("agent1".to_string()), None);

        assert!(result.is_ok());
        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].agent, Some("agent1".to_string()));
    }

    #[test]
    fn test_multiple_logs() {
        let logger = AuditLogger::new();

        logger
            .log_execution("tool1".to_string(), None, None)
            .unwrap();
        logger.log_denial("tool2".to_string(), None, None).unwrap();
        logger.log_prompt("tool3".to_string(), None, None).unwrap();

        assert_eq!(logger.len().unwrap(), 3);

        let entries = logger.entries().unwrap();
        assert_eq!(entries[0].tool, "tool1");
        assert_eq!(entries[0].action, AuditAction::Allowed);
        assert_eq!(entries[1].tool, "tool2");
        assert_eq!(entries[1].action, AuditAction::Denied);
        assert_eq!(entries[2].tool, "tool3");
        assert_eq!(entries[2].action, AuditAction::Prompted);
    }

    #[test]
    fn test_clear_entries() {
        let logger = AuditLogger::new();

        logger
            .log_execution("tool1".to_string(), None, None)
            .unwrap();
        logger
            .log_execution("tool2".to_string(), None, None)
            .unwrap();

        assert_eq!(logger.len().unwrap(), 2);

        logger.clear().unwrap();
        assert_eq!(logger.len().unwrap(), 0);
        assert!(logger.is_empty().unwrap());
    }

    #[test]
    fn test_default_creation() {
        let logger = AuditLogger::default();
        assert!(logger.is_empty().unwrap());
    }

    #[test]
    fn test_clone() {
        let logger1 = AuditLogger::new();
        logger1
            .log_execution("tool1".to_string(), None, None)
            .unwrap();

        let logger2 = logger1.clone();
        assert_eq!(logger2.len().unwrap(), 1);

        logger2
            .log_execution("tool2".to_string(), None, None)
            .unwrap();
        assert_eq!(logger1.len().unwrap(), 2);
    }
}
