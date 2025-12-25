//! RBAC (Role-Based Access Control) integration for MCP
//!
//! This module provides enterprise-grade RBAC controls for MCP server and tool access,
//! integrating with the ricecoder-security RBAC system.

use std::{collections::HashMap, sync::Arc};

use tracing::{debug, info, warn};

use crate::{
    error::{Error, Result},
    permissions::MCPPermissionManager,
};

/// MCP RBAC Manager for enterprise access control
pub struct MCRBACManager {
    access_control: Arc<ricecoder_security::access_control::AccessControl>,
    permission_manager: Arc<MCPPermissionManager>,
    role_mappings: HashMap<String, Vec<String>>, // MCP role -> Security roles
}

impl MCRBACManager {
    /// Create a new MCP RBAC manager
    pub fn new(
        access_control: Arc<ricecoder_security::access_control::AccessControl>,
        permission_manager: Arc<MCPPermissionManager>,
    ) -> Self {
        Self {
            access_control,
            permission_manager,
            role_mappings: HashMap::new(),
        }
    }

    /// Add role mapping for MCP roles to security roles
    pub fn add_role_mapping(&mut self, mcp_role: String, security_roles: Vec<String>) {
        self.role_mappings.insert(mcp_role, security_roles);
    }

    /// Check if a principal has access to an MCP server
    pub fn check_server_access(
        &self,
        principal: &ricecoder_security::access_control::Principal,
        server_id: &str,
    ) -> Result<bool> {
        // Expand principal roles using role mappings
        let mut expanded_roles = principal.roles.clone();
        for role_name in &principal.roles {
            if let Some(security_roles) = self.role_mappings.get(role_name) {
                for security_role in security_roles {
                    if !expanded_roles.contains(security_role) {
                        expanded_roles.push(security_role.clone());
                    }
                }
            }
        }

        // Create expanded principal with mapped roles
        let expanded_principal = ricecoder_security::access_control::Principal {
            id: principal.id.clone(),
            roles: expanded_roles,
            attributes: principal.attributes.clone(),
        };

        // Check if expanded principal has MCP server access permission
        let has_server_access = self.access_control.has_permission(
            &expanded_principal,
            &ricecoder_security::access_control::Permission::ApiKeyAccess,
        );

        if !has_server_access {
            debug!(
                "Principal {} denied access to MCP server {}",
                principal.id, server_id
            );
            return Ok(false);
        }

        debug!(
            "Principal {} granted access to MCP server {} via role mapping",
            principal.id, server_id
        );
        Ok(true)
    }

    /// Check if a principal has permission to execute a tool
    pub fn check_tool_execution_permission(
        &self,
        principal: &ricecoder_security::access_control::Principal,
        tool_id: &str,
        agent_id: Option<&str>,
    ) -> Result<bool> {
        // First check MCP permission manager
        let mcp_permission = self
            .permission_manager
            .check_permission(tool_id, agent_id)?;

        match mcp_permission {
            ricecoder_permissions::PermissionLevel::Allow => {
                // Check RBAC for additional restrictions
                let has_rbac_access = self.check_rbac_tool_access(principal, tool_id)?;
                if !has_rbac_access {
                    warn!(
                        "RBAC denied tool execution for principal {} on tool {}",
                        principal.id, tool_id
                    );
                    return Ok(false);
                }
                Ok(true)
            }
            ricecoder_permissions::PermissionLevel::Ask => {
                // For ask permissions, require explicit RBAC approval
                self.check_rbac_tool_access(principal, tool_id)
            }
            ricecoder_permissions::PermissionLevel::Deny => {
                debug!(
                    "MCP permission manager denied tool execution for {} on {}",
                    principal.id, tool_id
                );
                Ok(false)
            }
        }
    }

    /// Check RBAC-based tool access
    fn check_rbac_tool_access(
        &self,
        principal: &ricecoder_security::access_control::Principal,
        tool_id: &str,
    ) -> Result<bool> {
        // Map tool patterns to permissions
        let tool_permission = match tool_id {
            t if t.starts_with("file-") => ricecoder_security::access_control::Permission::Write,
            t if t.starts_with("read-") => ricecoder_security::access_control::Permission::Read,
            t if t.starts_with("execute-") => {
                ricecoder_security::access_control::Permission::Execute
            }
            t if t.starts_with("admin-") => ricecoder_security::access_control::Permission::Admin,
            _ => ricecoder_security::access_control::Permission::Execute, // Default to execute
        };

        let has_permission = self
            .access_control
            .has_permission(principal, &tool_permission);

        debug!(
            "RBAC check for principal {} on tool {} (permission: {:?}): {}",
            principal.id, tool_id, tool_permission, has_permission
        );

        Ok(has_permission)
    }

    /// Get all permissions for a principal across MCP resources
    pub fn get_principal_mcp_permissions(
        &self,
        principal: &ricecoder_security::access_control::Principal,
    ) -> Vec<String> {
        let mut permissions = Vec::new();

        // Add server access permissions
        if self.access_control.has_permission(
            principal,
            &ricecoder_security::access_control::Permission::ApiKeyAccess,
        ) {
            permissions.push("mcp:server:access".to_string());
        }

        // Add tool execution permissions based on RBAC
        let tool_permissions = [
            (
                "file",
                ricecoder_security::access_control::Permission::Write,
            ),
            ("read", ricecoder_security::access_control::Permission::Read),
            (
                "execute",
                ricecoder_security::access_control::Permission::Execute,
            ),
            (
                "admin",
                ricecoder_security::access_control::Permission::Admin,
            ),
        ];

        for (prefix, perm) in &tool_permissions {
            if self.access_control.has_permission(principal, perm) {
                permissions.push(format!("mcp:tool:{}:execute", prefix));
            }
        }

        permissions
    }

    /// Audit access control decision
    pub async fn audit_access_decision(
        &self,
        audit_logger: &ricecoder_security::audit::AuditLogger,
        principal: &ricecoder_security::access_control::Principal,
        resource: &str,
        action: &str,
        allowed: bool,
        reason: &str,
    ) -> Result<()> {
        let event_type = if allowed {
            ricecoder_security::audit::AuditEventType::Authorization
        } else {
            ricecoder_security::audit::AuditEventType::SecurityViolation
        };

        audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type,
                user_id: Some(principal.id.clone()),
                session_id: None,
                action: action.to_string(),
                resource: resource.to_string(),
                metadata: serde_json::json!({
                    "allowed": allowed,
                    "reason": reason,
                    "principal_roles": principal.roles,
                    "rbac_check": true
                }),
            })
            .await?;

        info!(
            "Audited RBAC decision: {} {} access to {} for principal {}: {}",
            if allowed { "granted" } else { "denied" },
            action,
            resource,
            principal.id,
            reason
        );

        Ok(())
    }
}

/// MCP Server Authorization middleware
pub struct MCPAuthorizationMiddleware {
    rbac_manager: Arc<MCRBACManager>,
    audit_logger: Arc<ricecoder_security::audit::AuditLogger>,
}

impl MCPAuthorizationMiddleware {
    /// Create new authorization middleware
    pub fn new(
        rbac_manager: Arc<MCRBACManager>,
        audit_logger: Arc<ricecoder_security::audit::AuditLogger>,
    ) -> Self {
        Self {
            rbac_manager,
            audit_logger,
        }
    }

    /// Authorize server access
    pub async fn authorize_server_access(
        &self,
        principal: &ricecoder_security::access_control::Principal,
        server_id: &str,
    ) -> Result<()> {
        let allowed = self
            .rbac_manager
            .check_server_access(principal, server_id)?;

        self.rbac_manager
            .audit_access_decision(
                &self.audit_logger,
                principal,
                &format!("mcp:server:{}", server_id),
                "access",
                allowed,
                if allowed {
                    "RBAC check passed"
                } else {
                    "RBAC check failed"
                },
            )
            .await?;

        if !allowed {
            return Err(Error::AuthorizationError(format!(
                "Principal {} not authorized to access MCP server {}",
                principal.id, server_id
            )));
        }

        Ok(())
    }

    /// Authorize tool execution
    pub async fn authorize_tool_execution(
        &self,
        principal: &ricecoder_security::access_control::Principal,
        tool_id: &str,
        agent_id: Option<&str>,
    ) -> Result<()> {
        let allowed = self
            .rbac_manager
            .check_tool_execution_permission(principal, tool_id, agent_id)?;

        self.rbac_manager
            .audit_access_decision(
                &self.audit_logger,
                principal,
                &format!("mcp:tool:{}", tool_id),
                "execute",
                allowed,
                if allowed {
                    "RBAC and permission check passed"
                } else {
                    "RBAC or permission check failed"
                },
            )
            .await?;

        if !allowed {
            return Err(Error::AuthorizationError(format!(
                "Principal {} not authorized to execute tool {}",
                principal.id, tool_id
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ricecoder_security::access_control::{AccessControl, Permission, Principal, Role};

    use super::*;

    #[test]
    fn test_rbac_manager_creation() {
        let access_control = Arc::new(AccessControl::new());
        let permission_manager = Arc::new(MCPPermissionManager::new());
        let rbac_manager = MCRBACManager::new(access_control, permission_manager);

        assert!(rbac_manager.role_mappings.is_empty());
    }

    #[test]
    fn test_role_mapping() {
        let access_control = Arc::new(AccessControl::new());
        let permission_manager = Arc::new(MCPPermissionManager::new());
        let mut rbac_manager = MCRBACManager::new(access_control, permission_manager);

        rbac_manager.add_role_mapping(
            "mcp-admin".to_string(),
            vec!["admin".to_string(), "user".to_string()],
        );

        assert_eq!(rbac_manager.role_mappings.len(), 1);
        assert_eq!(
            rbac_manager.role_mappings["mcp-admin"],
            vec!["admin", "user"]
        );
    }

    #[test]
    fn test_server_access_check() {
        let mut access_control = AccessControl::new();

        // Add admin role
        access_control.add_custom_role(
            "admin".to_string(),
            vec![Permission::ApiKeyAccess, Permission::Admin],
        );

        let access_control = Arc::new(access_control);
        let permission_manager = Arc::new(MCPPermissionManager::new());
        let mut rbac_manager = MCRBACManager::new(access_control, permission_manager);

        // Add role mapping
        rbac_manager.add_role_mapping("mcp-admin".to_string(), vec!["admin".to_string()]);

        // Test principal with admin role
        let admin_principal = Principal {
            id: "admin-user".to_string(),
            roles: vec!["mcp-admin".to_string()],
            attributes: HashMap::new(),
        };

        let result = rbac_manager.check_server_access(&admin_principal, "test-server");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test principal without role
        let user_principal = Principal {
            id: "regular-user".to_string(),
            roles: vec![],
            attributes: HashMap::new(),
        };

        let result = rbac_manager.check_server_access(&user_principal, "test-server");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
