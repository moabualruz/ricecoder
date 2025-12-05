//! Storage integration for permissions system
//!
//! This module provides persistence for permissions configuration and audit logs.

use crate::audit::AuditLogEntry;
use crate::error::Result;
use crate::permission::PermissionConfig;
use std::path::Path;

/// Repository trait for storing and retrieving permissions
pub trait PermissionRepository: Send + Sync {
    /// Load permission configuration from storage
    fn load_config(&self) -> Result<PermissionConfig>;

    /// Save permission configuration to storage
    fn save_config(&self, config: &PermissionConfig) -> Result<()>;

    /// Load audit logs from storage
    fn load_audit_logs(&self) -> Result<Vec<AuditLogEntry>>;

    /// Save audit logs to storage
    fn save_audit_logs(&self, logs: &[AuditLogEntry]) -> Result<()>;

    /// Append a single audit log entry to storage
    fn append_audit_log(&self, entry: &AuditLogEntry) -> Result<()>;
}

/// File-based permission repository
pub struct FilePermissionRepository {
    /// Path to permissions configuration file
    config_path: std::path::PathBuf,
    /// Path to audit logs file
    audit_path: std::path::PathBuf,
}

impl FilePermissionRepository {
    /// Create a new file-based permission repository
    pub fn new<P: AsRef<Path>>(config_path: P, audit_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            audit_path: audit_path.as_ref().to_path_buf(),
        }
    }

    /// Create a new file-based permission repository with default paths
    pub fn with_defaults<P: AsRef<Path>>(base_path: P) -> Self {
        let base = base_path.as_ref();
        Self {
            config_path: base.join("permissions.json"),
            audit_path: base.join("audit_logs.json"),
        }
    }
}

impl PermissionRepository for FilePermissionRepository {
    fn load_config(&self) -> Result<PermissionConfig> {
        if !self.config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(PermissionConfig::new());
        }

        let content = std::fs::read_to_string(&self.config_path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn save_config(&self, config: &PermissionConfig) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn load_audit_logs(&self) -> Result<Vec<AuditLogEntry>> {
        if !self.audit_path.exists() {
            // Return empty logs if file doesn't exist
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.audit_path)?;
        let logs = serde_json::from_str(&content)?;
        Ok(logs)
    }

    fn save_audit_logs(&self, logs: &[AuditLogEntry]) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.audit_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(logs)?;
        std::fs::write(&self.audit_path, content)?;
        Ok(())
    }

    fn append_audit_log(&self, entry: &AuditLogEntry) -> Result<()> {
        // Load existing logs
        let mut logs = self.load_audit_logs()?;

        // Append new entry
        logs.push(entry.clone());

        // Save all logs
        self.save_audit_logs(&logs)?;
        Ok(())
    }
}

/// In-memory permission repository (for testing)
pub struct InMemoryPermissionRepository {
    config: std::sync::Arc<std::sync::RwLock<PermissionConfig>>,
    logs: std::sync::Arc<std::sync::RwLock<Vec<AuditLogEntry>>>,
}

impl InMemoryPermissionRepository {
    /// Create a new in-memory permission repository
    pub fn new() -> Self {
        Self {
            config: std::sync::Arc::new(std::sync::RwLock::new(PermissionConfig::new())),
            logs: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryPermissionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionRepository for InMemoryPermissionRepository {
    fn load_config(&self) -> Result<PermissionConfig> {
        let config = self
            .config
            .read()
            .map_err(|e| crate::error::Error::Internal(format!("Failed to read config: {}", e)))?;
        Ok(config.clone())
    }

    fn save_config(&self, config: &PermissionConfig) -> Result<()> {
        let mut stored_config = self
            .config
            .write()
            .map_err(|e| crate::error::Error::Internal(format!("Failed to write config: {}", e)))?;
        *stored_config = config.clone();
        Ok(())
    }

    fn load_audit_logs(&self) -> Result<Vec<AuditLogEntry>> {
        let logs = self
            .logs
            .read()
            .map_err(|e| crate::error::Error::Internal(format!("Failed to read logs: {}", e)))?;
        Ok(logs.clone())
    }

    fn save_audit_logs(&self, logs: &[AuditLogEntry]) -> Result<()> {
        let mut stored_logs = self
            .logs
            .write()
            .map_err(|e| crate::error::Error::Internal(format!("Failed to write logs: {}", e)))?;
        *stored_logs = logs.to_vec();
        Ok(())
    }

    fn append_audit_log(&self, entry: &AuditLogEntry) -> Result<()> {
        let mut logs = self
            .logs
            .write()
            .map_err(|e| crate::error::Error::Internal(format!("Failed to write logs: {}", e)))?;
        logs.push(entry.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditAction;
    use crate::audit::AuditResult;
    use crate::permission::PermissionLevel;
    use crate::permission::ToolPermission;

    #[test]
    fn test_in_memory_repository_save_and_load_config() {
        let repo = InMemoryPermissionRepository::new();

        let mut config = PermissionConfig::new();
        config.add_permission(ToolPermission::new(
            "test_tool".to_string(),
            PermissionLevel::Allow,
        ));

        repo.save_config(&config).unwrap();

        let loaded = repo.load_config().unwrap();
        assert_eq!(loaded.get_permissions().len(), 1);
        assert_eq!(loaded.get_permissions()[0].tool_pattern, "test_tool");
    }

    #[test]
    fn test_in_memory_repository_save_and_load_logs() {
        let repo = InMemoryPermissionRepository::new();

        let entry = AuditLogEntry::new(
            "test_tool".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        );

        repo.append_audit_log(&entry).unwrap();

        let logs = repo.load_audit_logs().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].tool, "test_tool");
    }

    #[test]
    fn test_in_memory_repository_append_multiple_logs() {
        let repo = InMemoryPermissionRepository::new();

        let entry1 = AuditLogEntry::new(
            "tool1".to_string(),
            AuditAction::Allowed,
            AuditResult::Success,
        );
        let entry2 = AuditLogEntry::new(
            "tool2".to_string(),
            AuditAction::Denied,
            AuditResult::Blocked,
        );

        repo.append_audit_log(&entry1).unwrap();
        repo.append_audit_log(&entry2).unwrap();

        let logs = repo.load_audit_logs().unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].tool, "tool1");
        assert_eq!(logs[1].tool, "tool2");
    }

    #[test]
    fn test_in_memory_repository_default() {
        let repo = InMemoryPermissionRepository::default();
        let config = repo.load_config().unwrap();
        assert_eq!(config.get_permissions().len(), 0);
    }
}
