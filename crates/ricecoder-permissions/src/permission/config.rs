//! Permission configuration storage

use serde::{Deserialize, Serialize};
use crate::permission::models::{PermissionLevel, ToolPermission};

/// Configuration for storing permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    /// List of tool permissions
    pub permissions: Vec<ToolPermission>,
    /// Default permission level for tools not explicitly configured
    pub default_level: PermissionLevel,
}

impl PermissionConfig {
    /// Create a new permission configuration with default settings
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
            default_level: PermissionLevel::Ask,
        }
    }

    /// Create a new permission configuration with a specific default level
    pub fn with_default(default_level: PermissionLevel) -> Self {
        Self {
            permissions: Vec::new(),
            default_level,
        }
    }

    /// Add a permission to the configuration
    pub fn add_permission(&mut self, permission: ToolPermission) {
        self.permissions.push(permission);
    }

    /// Get all permissions
    pub fn get_permissions(&self) -> &[ToolPermission] {
        &self.permissions
    }

    /// Get the default permission level
    pub fn get_default_level(&self) -> PermissionLevel {
        self.default_level
    }

    /// Get permissions for a specific tool
    pub fn get_permissions_for_tool(&self, tool_name: &str) -> crate::error::Result<Vec<ToolPermission>> {
        // Use glob matcher to find permissions that apply to this tool
        let matcher = crate::glob_matcher::GlobMatcher::new();
        
        let matching_perms: Vec<ToolPermission> = self.permissions
            .iter()
            .filter(|p| {
                // Check if this permission's pattern matches the tool name
                matcher.match_pattern(&p.tool_pattern, tool_name)
            })
            .cloned()
            .collect();

        Ok(matching_perms)
    }

    /// Get the default permission level
    pub fn default_permission_level(&self) -> PermissionLevel {
        self.default_level
    }
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_config_creation() {
        let config = PermissionConfig::new();
        assert_eq!(config.permissions.len(), 0);
        assert_eq!(config.default_level, PermissionLevel::Ask);
    }

    #[test]
    fn test_permission_config_with_default() {
        let config = PermissionConfig::with_default(PermissionLevel::Allow);
        assert_eq!(config.permissions.len(), 0);
        assert_eq!(config.default_level, PermissionLevel::Allow);
    }

    #[test]
    fn test_permission_config_add_permission() {
        let mut config = PermissionConfig::new();
        let perm = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);
        config.add_permission(perm);

        assert_eq!(config.permissions.len(), 1);
        assert_eq!(config.permissions[0].tool_pattern, "test_*");
    }

    #[test]
    fn test_permission_config_get_permissions() {
        let mut config = PermissionConfig::new();
        let perm1 = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);
        let perm2 = ToolPermission::new("other_*".to_string(), PermissionLevel::Deny);

        config.add_permission(perm1);
        config.add_permission(perm2);

        let perms = config.get_permissions();
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn test_permission_config_serialization() {
        let mut config = PermissionConfig::new();
        config.add_permission(ToolPermission::new(
            "test_*".to_string(),
            PermissionLevel::Allow,
        ));

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PermissionConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.permissions.len(), 1);
        assert_eq!(deserialized.default_level, PermissionLevel::Ask);
    }

    #[test]
    fn test_permission_config_default() {
        let config = PermissionConfig::default();
        assert_eq!(config.permissions.len(), 0);
        assert_eq!(config.default_level, PermissionLevel::Ask);
    }
}
