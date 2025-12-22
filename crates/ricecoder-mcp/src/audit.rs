//! MCP Audit Logging Integration
//!
//! Provides audit logging for MCP operations including tool execution,
//! server connections, and security events.

use std::sync::Arc;

use tracing::info;

use crate::{
    error::Result, metadata::ToolMetadata, server_management::ServerRegistration,
    tool_execution::ToolExecutionResult,
};

/// MCP audit logger for recording security and operational events
pub struct MCPAuditLogger {
    audit_logger: Arc<ricecoder_security::audit::AuditLogger>,
}

impl MCPAuditLogger {
    /// Create a new MCP audit logger
    pub fn new(audit_logger: Arc<ricecoder_security::audit::AuditLogger>) -> Self {
        Self { audit_logger }
    }

    /// Log server registration
    pub async fn log_server_registration(
        &self,
        server_config: &crate::server_management::ServerConfig,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::SystemAccess,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: "server_register".to_string(),
                resource: format!("mcp_server:{}", server_config.id),
                metadata: serde_json::json!({
                    "server_name": server_config.name,
                    "transport_type": format!("{:?}", server_config.transport_config.transport_type),
                    "auto_start": server_config.auto_start,
                    "enabled_tools_count": server_config.enabled_tools.len()
                }),
            })
            .await?;

        info!(
            "Audited MCP server registration: {} by user {:?}",
            server_config.id, user_id
        );
        Ok(())
    }

    /// Log server unregistration
    pub async fn log_server_unregistration(
        &self,
        server_id: &str,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::SystemAccess,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: "server_unregister".to_string(),
                resource: format!("mcp_server:{}", server_id),
                metadata: serde_json::json!({
                    "server_id": server_id
                }),
            })
            .await?;

        info!(
            "Audited MCP server unregistration: {} by user {:?}",
            server_id, user_id
        );
        Ok(())
    }

    /// Log server connection
    pub async fn log_server_connection(
        &self,
        server_id: &str,
        success: bool,
        error_message: Option<String>,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        let event_type = if success {
            ricecoder_security::audit::AuditEventType::SystemAccess
        } else {
            ricecoder_security::audit::AuditEventType::SecurityViolation
        };

        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: if success {
                    "server_connect"
                } else {
                    "server_connect_failed"
                }
                .to_string(),
                resource: format!("mcp_server:{}", server_id),
                metadata: serde_json::json!({
                    "server_id": server_id,
                    "success": success,
                    "error_message": error_message
                }),
            })
            .await?;

        if success {
            info!(
                "Audited successful MCP server connection: {} by user {:?}",
                server_id, user_id
            );
        } else {
            info!(
                "Audited failed MCP server connection: {} by user {:?}: {:?}",
                server_id, user_id, error_message
            );
        }
        Ok(())
    }

    /// Log server disconnection
    pub async fn log_server_disconnection(
        &self,
        server_id: &str,
        reason: &str,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::SystemAccess,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: "server_disconnect".to_string(),
                resource: format!("mcp_server:{}", server_id),
                metadata: serde_json::json!({
                    "server_id": server_id,
                    "reason": reason
                }),
            })
            .await?;

        info!(
            "Audited MCP server disconnection: {} by user {:?}: {}",
            server_id, user_id, reason
        );
        Ok(())
    }

    /// Log tool enablement
    pub async fn log_tool_enablement(
        &self,
        server_id: &str,
        tool_name: &str,
        enabled: bool,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::Authorization,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: if enabled {
                    "tool_enable"
                } else {
                    "tool_disable"
                }
                .to_string(),
                resource: format!("mcp_tool:{}.{}", server_id, tool_name),
                metadata: serde_json::json!({
                    "server_id": server_id,
                    "tool_name": tool_name,
                    "enabled": enabled
                }),
            })
            .await?;

        info!(
            "Audited MCP tool {}: {}.{} by user {:?}",
            if enabled { "enablement" } else { "disablement" },
            server_id,
            tool_name,
            user_id
        );
        Ok(())
    }

    /// Log tool execution
    pub async fn log_tool_execution(
        &self,
        server_id: &str,
        tool_name: &str,
        execution_result: &ToolExecutionResult,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        let event_type = if execution_result.success {
            ricecoder_security::audit::AuditEventType::DataAccess
        } else {
            ricecoder_security::audit::AuditEventType::SecurityViolation
        };

        let execution_time_ms = execution_result.execution_time_ms;
        let error_message = execution_result.error.as_ref().map(|e| e.error.clone());
        let has_result = execution_result.result.is_some();

        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: "tool_execute".to_string(),
                resource: format!("mcp_tool:{}.{}", server_id, tool_name),
                metadata: serde_json::json!({
                    "server_id": server_id,
                    "tool_name": tool_name,
                    "success": execution_result.success,
                    "execution_time_ms": execution_time_ms,
                    "error_message": error_message,
                    "has_result": has_result
                }),
            })
            .await?;

        if execution_result.success {
            info!(
                "Audited successful MCP tool execution: {}.{} by user {:?} ({}ms)",
                server_id, tool_name, user_id, execution_time_ms
            );
        } else {
            info!(
                "Audited failed MCP tool execution: {}.{} by user {:?}: {:?}",
                server_id, tool_name, user_id, error_message
            );
        }
        Ok(())
    }

    /// Log tool permission check
    pub async fn log_tool_permission_check(
        &self,
        server_id: &str,
        tool_name: &str,
        allowed: bool,
        reason: &str,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        let event_type = if allowed {
            ricecoder_security::audit::AuditEventType::Authorization
        } else {
            ricecoder_security::audit::AuditEventType::SecurityViolation
        };

        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: if allowed {
                    "tool_access_granted"
                } else {
                    "tool_access_denied"
                }
                .to_string(),
                resource: format!("mcp_tool:{}.{}", server_id, tool_name),
                metadata: serde_json::json!({
                    "server_id": server_id,
                    "tool_name": tool_name,
                    "allowed": allowed,
                    "reason": reason
                }),
            })
            .await?;

        info!(
            "Audited MCP tool permission check: {}.{} {} for user {:?}: {}",
            server_id,
            tool_name,
            if allowed { "granted" } else { "denied" },
            user_id,
            reason
        );
        Ok(())
    }

    /// Log connection pool event
    pub async fn log_connection_pool_event(
        &self,
        server_id: &str,
        event_type: &str,
        details: serde_json::Value,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::SystemAccess,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: format!("connection_pool_{}", event_type),
                resource: format!("mcp_connection_pool:{}", server_id),
                metadata: details,
            })
            .await?;

        info!(
            "Audited MCP connection pool event: {} for server {}",
            event_type, server_id
        );
        Ok(())
    }

    /// Log health check event
    pub async fn log_health_check(
        &self,
        server_id: &str,
        healthy: bool,
        details: serde_json::Value,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        let event_type = if healthy {
            ricecoder_security::audit::AuditEventType::SystemAccess
        } else {
            ricecoder_security::audit::AuditEventType::SecurityViolation
        };

        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: if healthy {
                    "health_check_passed"
                } else {
                    "health_check_failed"
                }
                .to_string(),
                resource: format!("mcp_server:{}", server_id),
                metadata: serde_json::json!({
                    "server_id": server_id,
                    "healthy": healthy,
                    "details": details
                }),
            })
            .await?;

        info!(
            "Audited MCP health check: {} {} for server {}",
            if healthy { "passed" } else { "failed" },
            if healthy { "" } else { "with issues" },
            server_id
        );
        Ok(())
    }

    /// Log protocol validation event
    pub async fn log_protocol_validation(
        &self,
        message_type: &str,
        valid: bool,
        error_details: Option<String>,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<()> {
        let event_type = if valid {
            ricecoder_security::audit::AuditEventType::DataAccess
        } else {
            ricecoder_security::audit::AuditEventType::SecurityViolation
        };

        self.audit_logger
            .log_event(ricecoder_security::audit::AuditEvent {
                event_type,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
                action: if valid {
                    "protocol_validation_passed"
                } else {
                    "protocol_validation_failed"
                }
                .to_string(),
                resource: "mcp_protocol".to_string(),
                metadata: serde_json::json!({
                    "message_type": message_type,
                    "valid": valid,
                    "error_details": error_details
                }),
            })
            .await?;

        if !valid {
            info!(
                "Audited MCP protocol validation failure: {} by user {:?}: {:?}",
                message_type, user_id, error_details
            );
        }
        Ok(())
    }
}
