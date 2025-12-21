//! Access control and permission management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Result, SecurityError};

/// Permission types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Create,
    Read,
    Write,
    Execute,
    Delete,
    Share,
    Admin,
    ApiKeyAccess,
    SessionCreate,
    SessionShare,
    AuditRead,
    UpdateAuto,
}

/// Resource types for access control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Session,
    SessionShare,
    ApiKey,
    AuditLog,
    User,
    System,
}

/// Role definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    User,
    Admin,
    Service,
    Auditor,
}

impl Role {
    /// Check if this role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }

    /// Get permissions for this role
    pub fn permissions(&self) -> &[Permission] {
        match self {
            Role::User => &[
                Permission::Read,
                Permission::Write,
                Permission::Execute,
                Permission::SessionCreate,
            ],
            Role::Admin => &[
                Permission::Read,
                Permission::Write,
                Permission::Execute,
                Permission::Admin,
                Permission::ApiKeyAccess,
                Permission::SessionCreate,
                Permission::SessionShare,
                Permission::AuditRead,
                Permission::UpdateAuto,
            ],
            Role::Service => &[
                Permission::Read,
                Permission::ApiKeyAccess,
                Permission::SessionCreate,
            ],
            Role::Auditor => &[Permission::Read, Permission::AuditRead],
        }
    }

    /// Get role name
    pub fn name(&self) -> &str {
        match self {
            Role::User => "user",
            Role::Admin => "admin",
            Role::Service => "service",
            Role::Auditor => "auditor",
        }
    }

    /// Get role description
    pub fn description(&self) -> &str {
        match self {
            Role::User => "Basic user with read/write/execute permissions",
            Role::Admin => "Administrator with full access",
            Role::Service => "Service account for automated operations",
            Role::Auditor => "Audit and compliance monitoring",
        }
    }
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
    custom_roles: HashMap<String, Vec<Permission>>,
}

impl AccessControl {
    /// Create a new access control system
    pub fn new() -> Self {
        Self {
            custom_roles: HashMap::new(),
        }
    }

    /// Add a custom role
    pub fn add_custom_role(&mut self, name: String, permissions: Vec<Permission>) {
        self.custom_roles.insert(name, permissions);
    }

    /// Check if a principal has a specific permission
    pub fn has_permission(&self, principal: &Principal, permission: &Permission) -> bool {
        for role_name in &principal.roles {
            // Check built-in roles
            if let Some(role) = Self::role_from_name(role_name) {
                if role.has_permission(permission) {
                    return true;
                }
            }
            // Check custom roles
            if let Some(permissions) = self.custom_roles.get(role_name) {
                if permissions.contains(permission) {
                    return true;
                }
            }
        }
        false
    }

    /// Get built-in role from name
    fn role_from_name(name: &str) -> Option<Role> {
        match name {
            "user" => Some(Role::User),
            "admin" => Some(Role::Admin),
            "service" => Some(Role::Service),
            "auditor" => Some(Role::Auditor),
            _ => None,
        }
    }

    /// Check permission for a user with resource context (async for future extensibility)
    pub async fn check_permission(
        &self,
        user_id: Option<&str>,
        permission: &Permission,
        resource_type: &ResourceType,
        resource_id: Option<&str>,
    ) -> Result<bool> {
        // For now, implement basic permission checking
        // In a real implementation, this would check user roles from a database
        // and apply resource-specific access control rules

        // If no user_id provided, deny access
        let user_id = match user_id {
            Some(id) => id,
            None => return Ok(false),
        };

        // Create a mock principal for demonstration
        // In production, this would be loaded from a user store
        let principal = Principal {
            id: user_id.to_string(),
            roles: vec!["user".to_string()], // Default role
            attributes: HashMap::new(),
        };

        // Apply resource-specific logic
        match resource_type {
            ResourceType::Session => {
                // Sessions can be accessed by their owners or shared appropriately
                match permission {
                    Permission::Create => Ok(true), // Anyone can create sessions
                    Permission::Read | Permission::Write | Permission::Delete => {
                        // Check if user owns the resource
                        // For now, allow all - in production, check ownership
                        Ok(self.has_permission(&principal, permission))
                    }
                    Permission::Share => {
                        Ok(self.has_permission(&principal, &Permission::SessionShare))
                    }
                    _ => Ok(self.has_permission(&principal, permission)),
                }
            }
            ResourceType::SessionShare => {
                // Share management permissions
                match permission {
                    Permission::Read | Permission::Delete => {
                        // Check if user owns the share
                        Ok(self.has_permission(&principal, permission))
                    }
                    _ => Ok(false),
                }
            }
            ResourceType::ApiKey => match permission {
                Permission::Read | Permission::Write => {
                    Ok(self.has_permission(&principal, &Permission::ApiKeyAccess))
                }
                _ => Ok(false),
            },
            ResourceType::AuditLog => match permission {
                Permission::Read => Ok(self.has_permission(&principal, &Permission::AuditRead)),
                _ => Ok(false),
            },
            ResourceType::User => {
                // Users can only access their own data
                if resource_id == Some(user_id) {
                    Ok(self.has_permission(&principal, permission))
                } else {
                    Ok(self.has_permission(&principal, &Permission::Admin))
                }
            }
            ResourceType::System => Ok(self.has_permission(&principal, &Permission::Admin)),
        }
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
            // Check built-in roles
            if let Some(role) = Self::role_from_name(role_name) {
                for perm in role.permissions() {
                    if !all_permissions.contains(perm) {
                        all_permissions.push(perm.clone());
                    }
                }
            }
            // Check custom roles
            if let Some(permissions) = self.custom_roles.get(role_name) {
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
                    principal.id,
                    self.permission_name(required_permission),
                    resource
                ),
            });
        }
        Ok(())
    }

    /// Get built-in role by name
    pub fn get_builtin_role(&self, name: &str) -> Option<Role> {
        Self::role_from_name(name)
    }

    /// List all built-in roles
    pub fn list_builtin_roles(&self) -> Vec<Role> {
        vec![Role::User, Role::Admin, Role::Service, Role::Auditor]
    }

    /// Get human-readable permission name
    fn permission_name(&self, permission: &Permission) -> &str {
        match permission {
            Permission::Create => "create",
            Permission::Read => "read",
            Permission::Write => "write",
            Permission::Execute => "execute",
            Permission::Delete => "delete",
            Permission::Share => "share",
            Permission::Admin => "admin",
            Permission::ApiKeyAccess => "api_key_access",
            Permission::SessionCreate => "session_create",
            Permission::SessionShare => "session_share",
            Permission::AuditRead => "audit_read",
            Permission::UpdateAuto => "update_auto",
        }
    }
}

/// ABAC policy for attribute-based access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacPolicy {
    pub name: String,
    pub description: String,
    pub rules: Vec<AbacRule>,
}

/// ABAC rule defining access conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacRule {
    pub subject_attributes: HashMap<String, AttributeCondition>,
    pub resource_attributes: HashMap<String, AttributeCondition>,
    pub action: String,
    pub effect: AbacEffect,
}

/// Attribute condition for ABAC rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeCondition {
    Equals(String),
    NotEquals(String),
    Contains(String),
    NotContains(String),
    Regex(String),
    In(Vec<String>),
    NotIn(Vec<String>),
}

/// ABAC effect (allow or deny)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbacEffect {
    Allow,
    Deny,
}

/// Attribute-Based Access Control system
pub struct AttributeBasedAccessControl {
    policies: Vec<AbacPolicy>,
}

impl AttributeBasedAccessControl {
    /// Create a new ABAC system
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Add an ABAC policy
    pub fn add_policy(&mut self, policy: AbacPolicy) {
        self.policies.push(policy);
    }

    /// Evaluate access request using ABAC
    pub fn evaluate_access(
        &self,
        subject_attrs: &HashMap<String, String>,
        resource_attrs: &HashMap<String, String>,
        action: &str,
    ) -> AbacEffect {
        // Default deny
        let mut final_effect = AbacEffect::Deny;

        for policy in &self.policies {
            for rule in &policy.rules {
                if rule.action != action {
                    continue;
                }

                if self.matches_conditions(subject_attrs, &rule.subject_attributes)
                    && self.matches_conditions(resource_attrs, &rule.resource_attributes)
                {
                    final_effect = rule.effect.clone();
                    // First matching rule wins (in order)
                    break;
                }
            }
        }

        final_effect
    }

    /// Check if attributes match conditions
    fn matches_conditions(
        &self,
        attributes: &HashMap<String, String>,
        conditions: &HashMap<String, AttributeCondition>,
    ) -> bool {
        for (attr_name, condition) in conditions {
            let attr_value = match attributes.get(attr_name) {
                Some(value) => value,
                None => return false, // Missing required attribute
            };

            if !self.matches_condition(attr_value, condition) {
                return false;
            }
        }
        true
    }

    /// Check if a single attribute value matches a condition
    fn matches_condition(&self, value: &str, condition: &AttributeCondition) -> bool {
        match condition {
            AttributeCondition::Equals(expected) => value == expected,
            AttributeCondition::NotEquals(expected) => value != expected,
            AttributeCondition::Contains(substring) => value.contains(substring),
            AttributeCondition::NotContains(substring) => !value.contains(substring),
            AttributeCondition::Regex(pattern) => {
                regex::Regex::new(pattern).map_or(false, |re| re.is_match(value))
            }
            AttributeCondition::In(values) => values.contains(&value.to_string()),
            AttributeCondition::NotIn(values) => !values.contains(&value.to_string()),
        }
    }

    /// Get all policies
    pub fn get_policies(&self) -> &[AbacPolicy] {
        &self.policies
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
        self.access_control
            .has_permission(self.principal, permission)
    }

    /// Check if principal has any of the permissions
    pub fn has_any(&self, permissions: &[Permission]) -> bool {
        self.access_control
            .has_any_permission(self.principal, permissions)
    }

    /// Check if principal has all permissions
    pub fn has_all(&self, permissions: &[Permission]) -> bool {
        self.access_control
            .has_all_permissions(self.principal, permissions)
    }

    /// Validate resource access
    pub fn validate_resource(&self, resource: &str, permission: &Permission) -> Result<()> {
        self.access_control
            .validate_resource_access(self.principal, resource, permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_roles() {
        let ac = AccessControl::new();

        // Check that default roles exist
        assert!(ac.get_builtin_role("user").is_some());
        assert!(ac.get_builtin_role("admin").is_some());
        assert!(ac.get_builtin_role("service").is_some());
        assert!(ac.get_builtin_role("auditor").is_some());
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
        assert!(ac
            .validate_resource_access(&user, "session:user123", &Permission::Read)
            .is_ok());

        // User should not be able to access admin resources
        assert!(ac
            .validate_resource_access(&user, "admin:config", &Permission::Admin)
            .is_err());
    }

    #[test]
    fn test_custom_role() {
        let mut ac = AccessControl::new();

        ac.add_custom_role(
            "developer".to_string(),
            vec![
                Permission::Read,
                Permission::Write,
                Permission::ApiKeyAccess,
            ],
        );

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

    #[test]
    fn test_abac_basic() {
        let mut abac = AttributeBasedAccessControl::new();

        let policy = AbacPolicy {
            name: "developer_policy".to_string(),
            description: "Allow developers to access dev resources".to_string(),
            rules: vec![AbacRule {
                subject_attributes: HashMap::from([
                    (
                        "department".to_string(),
                        AttributeCondition::Equals("engineering".to_string()),
                    ),
                    (
                        "clearance".to_string(),
                        AttributeCondition::In(vec![
                            "secret".to_string(),
                            "top_secret".to_string(),
                        ]),
                    ),
                ]),
                resource_attributes: HashMap::from([(
                    "environment".to_string(),
                    AttributeCondition::Equals("development".to_string()),
                )]),
                action: "read".to_string(),
                effect: AbacEffect::Allow,
            }],
        };

        abac.add_policy(policy);

        // Test matching attributes
        let subject_attrs = HashMap::from([
            ("department".to_string(), "engineering".to_string()),
            ("clearance".to_string(), "secret".to_string()),
        ]);
        let resource_attrs =
            HashMap::from([("environment".to_string(), "development".to_string())]);

        let effect = abac.evaluate_access(&subject_attrs, &resource_attrs, "read");
        assert!(matches!(effect, AbacEffect::Allow));

        // Test non-matching attributes
        let bad_subject_attrs = HashMap::from([
            ("department".to_string(), "marketing".to_string()),
            ("clearance".to_string(), "secret".to_string()),
        ]);

        let effect = abac.evaluate_access(&bad_subject_attrs, &resource_attrs, "read");
        assert!(matches!(effect, AbacEffect::Deny));
    }

    #[test]
    fn test_abac_regex_condition() {
        let mut abac = AttributeBasedAccessControl::new();

        let policy = AbacPolicy {
            name: "regex_policy".to_string(),
            description: "Allow access based on regex pattern".to_string(),
            rules: vec![AbacRule {
                subject_attributes: HashMap::from([(
                    "email".to_string(),
                    AttributeCondition::Regex(r".*@company\.com$".to_string()),
                )]),
                resource_attributes: HashMap::new(),
                action: "access".to_string(),
                effect: AbacEffect::Allow,
            }],
        };

        abac.add_policy(policy);

        let valid_subject = HashMap::from([("email".to_string(), "user@company.com".to_string())]);
        let invalid_subject = HashMap::from([("email".to_string(), "user@gmail.com".to_string())]);

        let resource_attrs = HashMap::new();

        assert!(matches!(
            abac.evaluate_access(&valid_subject, &resource_attrs, "access"),
            AbacEffect::Allow
        ));
        assert!(matches!(
            abac.evaluate_access(&invalid_subject, &resource_attrs, "access"),
            AbacEffect::Deny
        ));
    }
}
