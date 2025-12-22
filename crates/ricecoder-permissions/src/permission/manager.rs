//! Permission manager for storing and retrieving permissions

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::error::Result;
#[allow(unused_imports)]
use crate::permission::models::{PermissionLevel, ToolPermission};

/// Manages permission storage and retrieval
pub struct PermissionManager {
    /// Permissions stored by tool pattern
    /// Key: tool pattern, Value: list of permissions for that pattern
    permissions: Arc<RwLock<HashMap<String, Vec<ToolPermission>>>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store or update a permission
    pub fn store_permission(&self, permission: ToolPermission) -> Result<()> {
        let mut perms = self.permissions.write().map_err(|e| {
            crate::error::Error::Internal(format!("Failed to acquire write lock: {}", e))
        })?;

        let tool_pattern = permission.tool_pattern.clone();
        perms
            .entry(tool_pattern)
            .or_insert_with(Vec::new)
            .push(permission);

        Ok(())
    }

    /// Get permissions for a specific tool
    pub fn get_permission(&self, tool_name: &str) -> Result<Vec<ToolPermission>> {
        let perms = self.permissions.read().map_err(|e| {
            crate::error::Error::Internal(format!("Failed to acquire read lock: {}", e))
        })?;

        // Collect all permissions that match this tool
        let mut matching_perms = Vec::new();

        for (pattern, permissions) in perms.iter() {
            // Check if pattern matches tool name
            if self.pattern_matches(pattern, tool_name) {
                matching_perms.extend(permissions.clone());
            }
        }

        Ok(matching_perms)
    }

    /// Get all permissions
    pub fn get_all_permissions(&self) -> Result<Vec<ToolPermission>> {
        let perms = self.permissions.read().map_err(|e| {
            crate::error::Error::Internal(format!("Failed to acquire read lock: {}", e))
        })?;

        let all_perms: Vec<ToolPermission> =
            perms.values().flat_map(|v| v.iter().cloned()).collect();

        Ok(all_perms)
    }

    /// Remove a permission by tool pattern
    pub fn remove_permission(&self, tool_pattern: &str) -> Result<()> {
        let mut perms = self.permissions.write().map_err(|e| {
            crate::error::Error::Internal(format!("Failed to acquire write lock: {}", e))
        })?;

        perms.remove(tool_pattern);
        Ok(())
    }

    /// Update an existing permission
    pub fn update_permission(
        &self,
        old_pattern: &str,
        new_permission: ToolPermission,
    ) -> Result<()> {
        let mut perms = self.permissions.write().map_err(|e| {
            crate::error::Error::Internal(format!("Failed to acquire write lock: {}", e))
        })?;

        // Remove old permission
        perms.remove(old_pattern);

        // Add new permission
        let tool_pattern = new_permission.tool_pattern.clone();
        perms
            .entry(tool_pattern)
            .or_insert_with(Vec::new)
            .push(new_permission);

        Ok(())
    }

    /// Reload permissions from a list (replaces all existing permissions)
    pub fn reload_permissions(&self, new_permissions: Vec<ToolPermission>) -> Result<()> {
        let mut perms = self.permissions.write().map_err(|e| {
            crate::error::Error::Internal(format!("Failed to acquire write lock: {}", e))
        })?;

        // Clear existing permissions
        perms.clear();

        // Add new permissions
        for permission in new_permissions {
            let tool_pattern = permission.tool_pattern.clone();
            perms
                .entry(tool_pattern)
                .or_insert_with(Vec::new)
                .push(permission);
        }

        Ok(())
    }

    /// Check if a glob pattern matches a tool name
    fn pattern_matches(&self, pattern: &str, tool_name: &str) -> bool {
        // Simple pattern matching: * matches any sequence of characters
        if pattern == "*" {
            return true;
        }

        // For now, support simple glob patterns
        // This is a basic implementation; more complex patterns can be added later
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut pos = 0;

            for (i, part) in parts.iter().enumerate() {
                if i == 0 {
                    // First part must match at the beginning
                    if !tool_name.starts_with(part) {
                        return false;
                    }
                    pos += part.len();
                } else if i == parts.len() - 1 {
                    // Last part must match at the end
                    if !tool_name.ends_with(part) {
                        return false;
                    }
                } else {
                    // Middle parts must be found in order
                    if let Some(found_pos) = tool_name[pos..].find(part) {
                        pos += found_pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        } else {
            pattern == tool_name
        }
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_manager_creation() {
        let manager = PermissionManager::new();
        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 0);
    }

    #[test]
    fn test_store_permission() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();

        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].tool_pattern, "test_tool");
        assert_eq!(perms[0].level, PermissionLevel::Allow);
    }

    #[test]
    fn test_get_permission_by_tool_name() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();

        let perms = manager.get_permission("test_tool").unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].tool_pattern, "test_tool");
    }

    #[test]
    fn test_get_all_permissions() {
        let manager = PermissionManager::new();
        let perm1 = ToolPermission::new("tool1".to_string(), PermissionLevel::Allow);
        let perm2 = ToolPermission::new("tool2".to_string(), PermissionLevel::Deny);

        manager.store_permission(perm1).unwrap();
        manager.store_permission(perm2).unwrap();

        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn test_remove_permission() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();
        assert_eq!(manager.get_all_permissions().unwrap().len(), 1);

        manager.remove_permission("test_tool").unwrap();
        assert_eq!(manager.get_all_permissions().unwrap().len(), 0);
    }

    #[test]
    fn test_store_multiple_permissions_same_pattern() {
        let manager = PermissionManager::new();
        let perm1 = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);
        let perm2 = ToolPermission::with_agent(
            "test_*".to_string(),
            PermissionLevel::Deny,
            "agent1".to_string(),
        );

        manager.store_permission(perm1).unwrap();
        manager.store_permission(perm2).unwrap();

        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn test_pattern_matching_wildcard() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();

        let perms = manager.get_permission("test_tool").unwrap();
        assert_eq!(perms.len(), 1);

        let perms = manager.get_permission("test_another").unwrap();
        assert_eq!(perms.len(), 1);
    }

    #[test]
    fn test_pattern_matching_exact() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("exact_tool".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();

        let perms = manager.get_permission("exact_tool").unwrap();
        assert_eq!(perms.len(), 1);

        let perms = manager.get_permission("other_tool").unwrap();
        assert_eq!(perms.len(), 0);
    }

    #[test]
    fn test_pattern_matching_global_wildcard() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("*".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();

        let perms = manager.get_permission("any_tool").unwrap();
        assert_eq!(perms.len(), 1);

        let perms = manager.get_permission("another_tool").unwrap();
        assert_eq!(perms.len(), 1);
    }

    #[test]
    fn test_update_permission() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();
        assert_eq!(manager.get_all_permissions().unwrap().len(), 1);

        let updated_perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Deny);
        manager
            .update_permission("test_tool", updated_perm)
            .unwrap();

        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].level, PermissionLevel::Deny);
    }

    #[test]
    fn test_reload_permissions() {
        let manager = PermissionManager::new();
        let perm1 = ToolPermission::new("tool1".to_string(), PermissionLevel::Allow);
        let perm2 = ToolPermission::new("tool2".to_string(), PermissionLevel::Deny);

        manager.store_permission(perm1).unwrap();
        manager.store_permission(perm2).unwrap();
        assert_eq!(manager.get_all_permissions().unwrap().len(), 2);

        let new_perms = vec![
            ToolPermission::new("new_tool1".to_string(), PermissionLevel::Ask),
            ToolPermission::new("new_tool2".to_string(), PermissionLevel::Allow),
            ToolPermission::new("new_tool3".to_string(), PermissionLevel::Deny),
        ];

        manager.reload_permissions(new_perms).unwrap();

        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 3);
        assert!(perms.iter().any(|p| p.tool_pattern == "new_tool1"));
        assert!(perms.iter().any(|p| p.tool_pattern == "new_tool2"));
        assert!(perms.iter().any(|p| p.tool_pattern == "new_tool3"));
    }

    #[test]
    fn test_reload_permissions_clears_old() {
        let manager = PermissionManager::new();
        let perm1 = ToolPermission::new("old_tool".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm1).unwrap();
        assert_eq!(manager.get_all_permissions().unwrap().len(), 1);

        let new_perms = vec![ToolPermission::new(
            "new_tool".to_string(),
            PermissionLevel::Ask,
        )];

        manager.reload_permissions(new_perms).unwrap();

        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].tool_pattern, "new_tool");
    }

    #[test]
    fn test_update_permission_changes_pattern() {
        let manager = PermissionManager::new();
        let perm = ToolPermission::new("old_pattern".to_string(), PermissionLevel::Allow);

        manager.store_permission(perm).unwrap();

        let updated_perm = ToolPermission::new("new_pattern".to_string(), PermissionLevel::Deny);
        manager
            .update_permission("old_pattern", updated_perm)
            .unwrap();

        let perms = manager.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].tool_pattern, "new_pattern");
        assert_eq!(perms[0].level, PermissionLevel::Deny);
    }
}
