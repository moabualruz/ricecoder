//! Access control and permission management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{SecurityError, Result};

/// Permission types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
    ApiKeyAccess,
    SessionCreate,
    SessionShare,
    AuditRead,
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
    pub description: String,
}

/// User or service principal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub id: String,
    pub roles: Vec<String>, // Role names
    pub attributes: HashMap<String, String>,
}

/// Access control system
pub struct AccessControl {
    roles: HashMap<String, Role>,
    role_permissions: HashMap<String, Vec<Permission>>,
}

impl AccessControl {
    /// Create a new access control system
    pub fn new() -> Self {
        let mut ac = Self {
            roles: HashMap::new(),
            role_permissions: HashMap::new(),
        };

        // Initialize default roles
        ac.add_default_roles();
        ac
    }

    /// Add a role
    pub fn add_role(&mut self, role: Role) {
        let role_name = role.name.clone();
        let permissions = role.permissions.clone();
        self.roles.insert(role_name.clone(), role);
        self.role_permissions.insert(role_name, permissions);
    }

    /// Check if a principal has a specific permission
    pub fn has_permission(&self, principal: &Principal, permission: &Permission) -> bool {
        for role_name in &principal.roles {
            if let Some(permissions) = self.role_permissions.get(role_name) {
                if permissions.contains(permission) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a principal has any of the specified permissions
    pub fn has_any_permission(&self, principal: &Principal, permissions: &[Permission]) -> bool {
        permissions
            .iter()
            .any(|perm| self.has_permission(principal, perm))
    }

    /// Check if a principal has all of the specified permissions
    pub fn has_all_permissions(&self, principal: &Principal, permissions: &[Permission]) -> bool {
        permissions
            .iter()
            .all(|perm| self.has_permission(principal, perm))
    }

    /// Get all permissions for a principal
    pub fn get_principal_permissions(&self, principal: &Principal) -> Vec<Permission> {
        let mut all_permissions = Vec::new();

        for role_name in &principal.roles {
            if let Some(permissions) = self.role_permissions.get(role_name) {
                for perm in permissions {
                    if !all_permissions.contains(perm) {
                        all_permissions.push(perm.clone());
                    }
                }
            }
        }

        all_permissions
    }

    /// Validate resource access for a principal
    pub fn validate_resource_access(
        &self,
        principal: &Principal,
        resource: &str,
        required_permission: &Permission,
    ) -> Result<()> {
        if !self.has_permission(principal, required_permission) {
            return Err(SecurityError::AccessDenied {
                message: format!(
                    "Principal '{}' does not have permission '{}' for resource '{}'",
                    principal.id, self.permission_name(required_permission), resource
                ),
            });
        }
        Ok(())
    }

    /// Get role by name
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// List all roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    /// Add default roles
    fn add_default_roles(&mut self) {
        // User role - basic permissions
        self.add_role(Role {
            name: "user".to_string(),
            permissions: vec![
                Permission::Read,
                Permission::Write,
                Permission::Execute,
                Permission::SessionCreate,
            ],
            description: "Basic user with read/write/execute permissions".to_string(),
        });

        // Admin role - all permissions
        self.add_role(Role {
            name: "admin".to_string(),
            permissions: vec![
                Permission::Read,
                Permission::Write,
                Permission::Execute,
                Permission::Admin,
                Permission::ApiKeyAccess,
                Permission::SessionCreate,
                Permission::SessionShare,
                Permission::AuditRead,
            ],
            description: "Administrator with full access".to_string(),
        });

        // Service role - for automated systems
        self.add_role(Role {
            name: "service".to_string(),
            permissions: vec![
                Permission::Read,
                Permission::ApiKeyAccess,
                Permission::SessionCreate,
            ],
            description: "Service account for automated operations".to_string(),
        });

        // Auditor role - read-only access to audit logs
        self.add_role(Role {
            name: "auditor".to_string(),
            permissions: vec![
                Permission::Read,
                Permission::AuditRead,
            ],
            description: "Audit and compliance monitoring".to_string(),
        });
    }

    /// Get human-readable permission name
    fn permission_name(&self, permission: &Permission) -> &str {
        match permission {
            Permission::Read => "read",
            Permission::Write => "write",
            Permission::Execute => "execute",
            Permission::Admin => "admin",
            Permission::ApiKeyAccess => "api_key_access",
            Permission::SessionCreate => "session_create",
            Permission::SessionShare => "session_share",
            Permission::AuditRead => "audit_read",
        }
    }
}

/// Permission check helper
pub struct PermissionCheck<'a> {
    access_control: &'a AccessControl,
    principal: &'a Principal,
}

impl<'a> PermissionCheck<'a> {
    /// Create a new permission check
    pub fn new(access_control: &'a AccessControl, principal: &'a Principal) -> Self {
        Self {
            access_control,
            principal,
        }
    }

    /// Check if principal has permission
    pub fn has(&self, permission: &Permission) -> bool {
        self.access_control.has_permission(self.principal, permission)
    }

    /// Check if principal has any of the permissions
    pub fn has_any(&self, permissions: &[Permission]) -> bool {
        self.access_control.has_any_permission(self.principal, permissions)
    }

    /// Check if principal has all permissions
    pub fn has_all(&self, permissions: &[Permission]) -> bool {
        self.access_control.has_all_permissions(self.principal, permissions)
    }

    /// Validate resource access
    pub fn validate_resource(&self, resource: &str, permission: &Permission) -> Result<()> {
        self.access_control.validate_resource_access(self.principal, resource, permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_roles() {
        let ac = AccessControl::new();

        // Check that default roles exist
        assert!(ac.get_role("user").is_some());
        assert!(ac.get_role("admin").is_some());
        assert!(ac.get_role("service").is_some());
        assert!(ac.get_role("auditor").is_some());
    }

    #[test]
    fn test_user_permissions() {
        let ac = AccessControl::new();
        let user = Principal {
            id: "user123".to_string(),
            roles: vec!["user".to_string()],
            attributes: HashMap::new(),
        };

        assert!(ac.has_permission(&user, &Permission::Read));
        assert!(ac.has_permission(&user, &Permission::Write));
        assert!(ac.has_permission(&user, &Permission::Execute));
        assert!(ac.has_permission(&user, &Permission::SessionCreate));

        // Should not have admin permissions
        assert!(!ac.has_permission(&user, &Permission::Admin));
        assert!(!ac.has_permission(&user, &Permission::AuditRead));
    }

    #[test]
    fn test_admin_permissions() {
        let ac = AccessControl::new();
        let admin = Principal {
            id: "admin123".to_string(),
            roles: vec!["admin".to_string()],
            attributes: HashMap::new(),
        };

        // Admin should have all permissions
        assert!(ac.has_permission(&admin, &Permission::Read));
        assert!(ac.has_permission(&admin, &Permission::Write));
        assert!(ac.has_permission(&admin, &Permission::Execute));
        assert!(ac.has_permission(&admin, &Permission::Admin));
        assert!(ac.has_permission(&admin, &Permission::ApiKeyAccess));
        assert!(ac.has_permission(&admin, &Permission::SessionCreate));
        assert!(ac.has_permission(&admin, &Permission::SessionShare));
        assert!(ac.has_permission(&admin, &Permission::AuditRead));
    }

    #[test]
    fn test_permission_check_helper() {
        let ac = AccessControl::new();
        let user = Principal {
            id: "user123".to_string(),
            roles: vec!["user".to_string()],
            attributes: HashMap::new(),
        };

        let checker = PermissionCheck::new(&ac, &user);

        assert!(checker.has(&Permission::Read));
        assert!(!checker.has(&Permission::Admin));

        assert!(checker.has_any(&[Permission::Read, Permission::Admin]));
        assert!(!checker.has_all(&[Permission::Read, Permission::Admin]));
    }

    #[test]
    fn test_resource_access_validation() {
        let ac = AccessControl::new();
        let user = Principal {
            id: "user123".to_string(),
            roles: vec!["user".to_string()],
            attributes: HashMap::new(),
        };

        // User should be able to access their own session
        assert!(ac.validate_resource_access(&user, "session:user123", &Permission::Read).is_ok());

        // User should not be able to access admin resources
        assert!(ac.validate_resource_access(&user, "admin:config", &Permission::Admin).is_err());
    }

    #[test]
    fn test_custom_role() {
        let mut ac = AccessControl::new();

        let custom_role = Role {
            name: "developer".to_string(),
            permissions: vec![Permission::Read, Permission::Write, Permission::ApiKeyAccess],
            description: "Developer role".to_string(),
        };

        ac.add_role(custom_role);

        let developer = Principal {
            id: "dev123".to_string(),
            roles: vec!["developer".to_string()],
            attributes: HashMap::new(),
        };

        assert!(ac.has_permission(&developer, &Permission::Read));
        assert!(ac.has_permission(&developer, &Permission::Write));
        assert!(ac.has_permission(&developer, &Permission::ApiKeyAccess));
        assert!(!ac.has_permission(&developer, &Permission::Admin));
    }
}