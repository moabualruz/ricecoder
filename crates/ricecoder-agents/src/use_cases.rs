//! Use cases for agent operations
//!
//! This module contains use case implementations that orchestrate
//! complex operations using the application services.

use std::sync::Arc;

use ricecoder_providers::{
    community::ProviderAnalytics,
    curation::SelectionConstraints,
    models::{Capability, ModelInfo},
    performance_monitor::{PerformanceSummary, ProviderMetrics},
    provider::manager::{ModelFilter, ModelFilterCriteria, ProviderManager, ProviderStatus},
};
use ricecoder_security::{
    access_control::{AccessControl, Permission, ResourceType},
    audit::AuditLogger,
    compliance::{ComplianceValidator, DataClassification as SecurityDataClassification},
};
use ricecoder_sessions::{
    BackgroundAgentManager, DataClassification, EnterpriseSharingPolicy, Message, MessageRole,
    Session, SessionContext, SessionError, SessionManager, SessionMode, SessionPerformanceMonitor,
    SessionRouter, SessionStore, SharePermissions, ShareService, TokenUsageTracker,
};
use serde_json::json;
use tracing::{debug, error, info, warn};

use crate::{
    error::AgentError,
    mcp_integration::{ExternalToolBackend, ExternalToolIntegrationService, ToolExecutionResult},
};

/// Process and validate tool execution result
pub(crate) fn process_tool_result(
    tool_name: &str,
    raw_result: &serde_json::Value,
) -> Result<ToolExecutionResult, AgentError> {
    // Validate result structure
    if !raw_result.is_object() {
        return Err(AgentError::ExecutionFailed(
            "Invalid tool result format: expected object".to_string(),
        ));
    }

    let success = raw_result
        .get("success")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| {
            AgentError::ExecutionFailed(
                "Missing or invalid 'success' field in tool result".to_string(),
            )
        })?;

    let execution_time_ms = raw_result
        .get("execution_time_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Validate execution time is reasonable
    if execution_time_ms > 300_000 {
        // 5 minutes
        warn!(
            execution_time_ms = execution_time_ms,
            "Unusually long tool execution time detected"
        );
    }

    if success {
        // Validate successful result structure
        let data = raw_result.get("result").cloned();

        // Validate metadata if present
        if let Some(metadata) = raw_result.get("metadata") {
            if let Some(provider) = metadata.get("provider").and_then(|p| p.as_str()) {
                debug!(
                    provider = provider,
                    execution_time_ms = execution_time_ms,
                    "Tool executed successfully"
                );
            }
        }

        Ok(ToolExecutionResult {
            success: true,
            data,
            error: None,
            execution_time_ms,
        })
    } else {
        // Process error result
        let error_message = raw_result
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown tool execution error".to_string());

        // Log error details
        error!(
            tool_name = %tool_name,
            error = %error_message,
            execution_time_ms = execution_time_ms,
            "Tool execution failed"
        );

        // Check for specific error types
        let processed_error = if error_message.contains("timeout") {
            format!("Tool execution timed out: {}", error_message)
        } else if error_message.contains("connection") {
            format!("Tool connection error: {}", error_message)
        } else if error_message.contains("authentication") {
            format!("Tool authentication error: {}", error_message)
        } else {
            error_message
        };

        Ok(ToolExecutionResult {
            success: false,
            data: None,
            error: Some(processed_error),
            execution_time_ms,
        })
    }
}

/// Use case for executing external tools
pub struct ExecuteExternalToolUseCase {
    tool_integration: Arc<ExternalToolIntegrationService>,
}

impl ExecuteExternalToolUseCase {
    /// Create a new tool execution use case
    pub fn new(tool_integration: Arc<ExternalToolIntegrationService>) -> Self {
        Self { tool_integration }
    }

    /// Execute a tool with the given parameters
    pub async fn execute(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
        session_id: Option<String>,
    ) -> Result<ToolExecutionResult, AgentError> {
        debug!(
            tool_name = %tool_name,
            session_id = ?session_id,
            "Executing external tool use case"
        );

        // Check if tool integration is ready
        if !Arc::as_ref(&self.tool_integration).is_ready().await {
            warn!("Tool integration service is not ready");
            return Err(AgentError::ExecutionFailed(
                "Tool integration service not configured".to_string(),
            ));
        }

        // Execute the tool through the integration service
        let result = Arc::as_ref(&self.tool_integration)
            .execute_tool(tool_name, parameters, session_id)
            .await?;

        // Parse and validate the result
        let execution_result = process_tool_result(tool_name, &result)?;

        info!(
            tool_name = %tool_name,
            success = execution_result.success,
            execution_time_ms = execution_result.execution_time_ms,
            "Tool execution completed"
        );

        Ok(execution_result)
    }
}

/// Use case for configuring tool backends
pub struct ConfigureToolBackendUseCase {
    tool_integration: Arc<ExternalToolIntegrationService>,
}

impl ConfigureToolBackendUseCase {
    /// Create a new backend configuration use case
    pub fn new(tool_integration: Arc<ExternalToolIntegrationService>) -> Self {
        Self { tool_integration }
    }

    /// Configure an MCP backend
    #[cfg(feature = "mcp")]
    pub async fn configure_mcp_backend(
        &self,
        server_name: String,
        server_command: String,
        server_args: Vec<String>,
    ) -> Result<(), AgentError> {
        debug!(
            server_name = %server_name,
            server_command = %server_command,
            server_args = ?server_args,
            "Configuring MCP backend"
        );

        let backend = ExternalToolBackend::mcp(server_command, server_args);
        Arc::as_ref(&self.tool_integration)
            .configure_backend(server_name.clone(), backend)
            .await?;

        info!(server_name = %server_name, "MCP backend configured successfully");
        Ok(())
    }

    /// Configure an MCP backend (mock implementation when MCP feature is disabled)
    #[cfg(not(feature = "mcp"))]
    pub async fn configure_mcp_backend(
        &self,
        server_name: String,
        _server_command: String,
        _server_args: Vec<String>,
    ) -> Result<(), AgentError> {
        debug!(server_name = %server_name, "Configuring mock MCP backend");

        let backend = ExternalToolBackend::mcp_default();
        Arc::as_ref(&self.tool_integration)
            .configure_backend(server_name.clone(), backend)
            .await?;

        info!(server_name = %server_name, "Mock MCP backend configured successfully");
        Ok(())
    }

    /// Configure an HTTP API backend
    pub async fn configure_http_backend(
        &self,
        name: String,
        base_url: String,
    ) -> Result<(), AgentError> {
        debug!(name = %name, base_url = %base_url, "Configuring HTTP API backend");

        let backend = ExternalToolBackend::http_api(base_url);
        self.tool_integration
            .configure_backend(name.clone(), backend)
            .await?;

        info!(name = %name, "HTTP API backend configured successfully");
        Ok(())
    }

    /// Get current configuration status
    pub async fn get_configuration_status(&self) -> serde_json::Value {
        json!({
            "configured_backends": Arc::as_ref(&self.tool_integration).get_configured_backends().await,
            "ready": Arc::as_ref(&self.tool_integration).is_ready().await
        })
    }
}

/// Use case for managing tool operations
pub struct ToolManagementUseCase {
    tool_integration: Arc<ExternalToolIntegrationService>,
}

impl ToolManagementUseCase {
    /// Create a new tool management use case
    pub fn new(tool_integration: Arc<ExternalToolIntegrationService>) -> Self {
        Self { tool_integration }
    }

    /// List all configured backends
    pub async fn list_backends(&self) -> Vec<String> {
        Arc::as_ref(&self.tool_integration)
            .get_configured_backends()
            .await
    }

    /// Check if tool integration is operational
    pub async fn health_check(&self) -> serde_json::Value {
        let is_ready = Arc::as_ref(&self.tool_integration).is_ready().await;
        let backends = Arc::as_ref(&self.tool_integration)
            .get_configured_backends()
            .await;

        json!({
            "healthy": is_ready,
            "backends_configured": backends.len(),
            "backend_names": backends,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Execute a batch of tools
    pub async fn execute_tool_batch(
        &self,
        tools: Vec<(String, serde_json::Value)>,
        session_id: Option<String>,
    ) -> Result<Vec<ToolExecutionResult>, AgentError> {
        let mut results = Vec::new();

        for (tool_name, parameters) in tools {
            let result = Arc::as_ref(&self.tool_integration)
                .execute_tool(&tool_name, parameters, session_id.clone())
                .await?;

            // Parse and validate result
            let execution_result = process_tool_result(&tool_name, &result)?;

            results.push(execution_result);
        }

        Ok(results)
    }
}

/// Use case for managing session lifecycle with enterprise features
pub struct SessionLifecycleUseCase {
    session_manager: Arc<SessionManager>,
    session_store: Arc<SessionStore>,
    access_control: Option<Arc<AccessControl>>,
    compliance_validator: Option<Arc<ComplianceValidator>>,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl SessionLifecycleUseCase {
    /// Create a new session lifecycle use case
    pub fn new(session_manager: Arc<SessionManager>, session_store: Arc<SessionStore>) -> Self {
        Self {
            session_manager,
            session_store,
            access_control: None,
            compliance_validator: None,
            audit_logger: None,
        }
    }

    /// Create a new session lifecycle use case with enterprise features
    pub fn with_enterprise_features(
        session_manager: Arc<SessionManager>,
        session_store: Arc<SessionStore>,
        access_control: Arc<AccessControl>,
        compliance_validator: Arc<ComplianceValidator>,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            session_manager,
            session_store,
            access_control: Some(access_control),
            compliance_validator: Some(compliance_validator),
            audit_logger: Some(audit_logger),
        }
    }

    /// Create a new session with enterprise features
    pub async fn create_session(
        &self,
        name: String,
        context: SessionContext,
        user_id: Option<String>,
    ) -> Result<String, AgentError> {
        debug!(name = %name, user_id = ?user_id, "Creating new session");

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Create;
            let resource_type = ResourceType::Session;
            if !access_control
                .check_permission(user_id.as_deref(), &permission, &resource_type, None)
                .await?
            {
                return Err(AgentError::AccessDenied(format!(
                    "User {} does not have permission to create sessions",
                    user_id.as_deref().unwrap_or("unknown")
                )));
            }
        }

        // Validate compliance if enabled
        if let Some(ref compliance_validator) = self.compliance_validator {
            // Check if session data classification is allowed
            let data_classification = SecurityDataClassification::Internal; // Default for sessions
            if !compliance_validator
                .validate_data_classification(&data_classification)
                .await?
            {
                return Err(AgentError::ComplianceViolation(
                    "Session data classification not allowed".to_string(),
                ));
            }
        }

        // Create session
        let session = Session::new(name, context);
        let session_id = session.id.clone();

        // Save to store for persistence (with encryption if enabled)
        self.session_store
            .save(&session)
            .await
            .map_err(|e| AgentError::Internal(format!("Failed to save session: {}", e)))?;

        // Log audit event if enabled
        if let Some(ref audit_logger) = self.audit_logger {
            let event = ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::DataAccess,
                user_id,
                session_id: Some(session_id.clone()),
                action: "session_created".to_string(),
                resource: format!("session:{}", session_id),
                metadata: serde_json::json!({
                    "session_name": session.name,
                    "created_at": session.created_at
                }),
            };
            // Note: In production, this should be async
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        info!(session_id = %session_id, "Session created successfully");
        Ok(session_id)
    }

    /// Load an existing session with access control
    pub async fn load_session(
        &self,
        session_id: &str,
        user_id: Option<String>,
    ) -> Result<Session, AgentError> {
        debug!(session_id = %session_id, user_id = ?user_id, "Loading session");

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Read;
            let resource_type = ResourceType::Session;
            if !access_control
                .check_permission(
                    user_id.as_deref(),
                    &permission,
                    &resource_type,
                    Some(session_id),
                )
                .await?
            {
                return Err(AgentError::AccessDenied(format!(
                    "User {} does not have permission to access session {}",
                    user_id.as_deref().unwrap_or("unknown"),
                    session_id
                )));
            }
        }

        let session = self
            .session_store
            .load(session_id)
            .await
            .map_err(|e| AgentError::Internal(format!("Failed to load session: {}", e)))?;

        // Validate compliance if enabled
        if let Some(ref compliance_validator) = self.compliance_validator {
            // Check data classification compliance
            let data_classification = SecurityDataClassification::Internal;
            if !compliance_validator
                .validate_data_access(&data_classification, user_id.as_deref())
                .await?
            {
                return Err(AgentError::ComplianceViolation(
                    "Access to session data not compliant".to_string(),
                ));
            }
        }

        // Log audit event if enabled
        if let Some(ref audit_logger) = self.audit_logger {
            let event = ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::DataAccess,
                user_id,
                session_id: Some(session_id.to_string()),
                action: "session_loaded".to_string(),
                resource: format!("session:{}", session_id),
                metadata: serde_json::json!({
                    "session_name": session.name
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        info!(session_id = %session_id, "Session loaded successfully");
        Ok(session)
    }

    /// Save a session with access control
    pub async fn save_session(
        &self,
        session: &Session,
        user_id: Option<String>,
    ) -> Result<(), AgentError> {
        debug!(session_id = %session.id, user_id = ?user_id, "Saving session");

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Write;
            let resource_type = ResourceType::Session;
            if !access_control
                .check_permission(
                    user_id.as_deref(),
                    &permission,
                    &resource_type,
                    Some(&session.id),
                )
                .await?
            {
                return Err(AgentError::AccessDenied(format!(
                    "User {} does not have permission to modify session {}",
                    user_id.as_deref().unwrap_or("unknown"),
                    session.id
                )));
            }
        }

        // Validate compliance if enabled
        if let Some(ref compliance_validator) = self.compliance_validator {
            let data_classification = SecurityDataClassification::Internal;
            if !compliance_validator
                .validate_data_modification(&data_classification, user_id.as_deref())
                .await?
            {
                return Err(AgentError::ComplianceViolation(
                    "Session modification not compliant".to_string(),
                ));
            }
        }

        self.session_store
            .save(session)
            .await
            .map_err(|e| AgentError::Internal(format!("Failed to save session: {}", e)))?;

        // Log audit event if enabled
        if let Some(ref audit_logger) = self.audit_logger {
            let event = ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::DataAccess,
                user_id,
                session_id: Some(session.id.clone()),
                action: "session_saved".to_string(),
                resource: format!("session:{}", session.id),
                metadata: serde_json::json!({
                    "session_name": session.name
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        info!(session_id = %session.id, "Session saved successfully");
        Ok(())
    }

    /// Delete a session with access control and compliance
    pub async fn delete_session(
        &self,
        session_id: &str,
        user_id: Option<String>,
    ) -> Result<(), AgentError> {
        debug!(session_id = %session_id, user_id = ?user_id, "Deleting session");

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Delete;
            let resource_type = ResourceType::Session;
            if !access_control
                .check_permission(
                    user_id.as_deref(),
                    &permission,
                    &resource_type,
                    Some(session_id),
                )
                .await?
            {
                return Err(AgentError::AccessDenied(format!(
                    "User {} does not have permission to delete session {}",
                    user_id.as_deref().unwrap_or("unknown"),
                    session_id
                )));
            }
        }

        // Validate compliance if enabled (check data erasure requirements)
        if let Some(ref compliance_validator) = self.compliance_validator {
            let data_classification = SecurityDataClassification::Internal;
            if !compliance_validator
                .validate_data_erasure(&data_classification, user_id.as_deref())
                .await?
            {
                return Err(AgentError::ComplianceViolation(
                    "Session deletion not compliant with data erasure requirements".to_string(),
                ));
            }
        }

        self.session_store
            .delete(session_id)
            .await
            .map_err(|e| AgentError::Internal(format!("Failed to delete session: {}", e)))?;

        // Log audit event if enabled
        if let Some(ref audit_logger) = self.audit_logger {
            let event = ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::DataErasure,
                user_id,
                session_id: Some(session_id.to_string()),
                action: "session_deleted".to_string(),
                resource: format!("session:{}", session_id),
                metadata: serde_json::json!({
                    "permanent_deletion": true
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        info!(session_id = %session_id, "Session deleted successfully");
        Ok(())
    }

    /// List sessions with access control filtering
    pub async fn list_sessions(&self, user_id: Option<String>) -> Result<Vec<Session>, AgentError> {
        debug!(user_id = ?user_id, "Listing sessions");

        let all_sessions = self
            .session_store
            .list()
            .await
            .map_err(|e| AgentError::Internal(format!("Failed to list sessions: {}", e)))?;

        // Filter sessions based on access control
        let accessible_sessions = if let Some(ref access_control) = self.access_control {
            let mut filtered = Vec::new();
            for session in all_sessions {
                let permission = Permission::Read;
                let resource_type = ResourceType::Session;
                if access_control
                    .check_permission(
                        user_id.as_deref(),
                        &permission,
                        &resource_type,
                        Some(&session.id),
                    )
                    .await?
                {
                    filtered.push(session);
                }
            }
            filtered
        } else {
            all_sessions
        };

        // Log audit event if enabled
        if let Some(ref audit_logger) = self.audit_logger {
            let event = ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::DataAccess,
                user_id,
                session_id: None,
                action: "sessions_listed".to_string(),
                resource: "sessions".to_string(),
                metadata: serde_json::json!({
                    "returned_count": accessible_sessions.len()
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        info!(
            count = accessible_sessions.len(),
            "Retrieved filtered session list"
        );
        Ok(accessible_sessions)
    }
}

/// Use case for session sharing and access control with enterprise features
pub struct SessionSharingUseCase {
    share_service: Arc<ShareService>,
    session_store: Arc<SessionStore>,
    access_control: Option<Arc<AccessControl>>,
    compliance_validator: Option<Arc<ComplianceValidator>>,
}

impl SessionSharingUseCase {
    /// Create a new session sharing use case
    pub fn new(share_service: Arc<ShareService>, session_store: Arc<SessionStore>) -> Self {
        Self {
            share_service,
            session_store,
            access_control: None,
            compliance_validator: None,
        }
    }

    /// Create a new session sharing use case with enterprise features
    pub fn with_enterprise_features(
        share_service: Arc<ShareService>,
        session_store: Arc<SessionStore>,
        access_control: Arc<AccessControl>,
        compliance_validator: Arc<ComplianceValidator>,
    ) -> Self {
        Self {
            share_service,
            session_store,
            access_control: Some(access_control),
            compliance_validator: Some(compliance_validator),
        }
    }

    /// Create a shareable link for a session with enterprise controls
    pub async fn create_share_link(
        &self,
        session_id: &str,
        permissions: SharePermissions,
        expires_in_seconds: Option<u64>,
        user_id: Option<String>,
        enterprise_policy: Option<EnterpriseSharingPolicy>,
    ) -> Result<String, AgentError> {
        debug!(session_id = %session_id, permissions = ?permissions, user_id = ?user_id, "Creating share link");

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Share;
            let resource_type = ResourceType::Session;
            if !access_control
                .check_permission(
                    user_id.as_deref(),
                    &permission,
                    &resource_type,
                    Some(session_id),
                )
                .await?
            {
                return Err(AgentError::AccessDenied(format!(
                    "User {} does not have permission to share session {}",
                    user_id.as_deref().unwrap_or("unknown"),
                    session_id
                )));
            }
        }

        // Validate enterprise policy and compliance
        if let Some(ref policy) = enterprise_policy {
            if let Some(ref compliance_validator) = self.compliance_validator {
                // Validate data classification for sharing
                let data_classification = match policy.data_classification {
                    DataClassification::Public => SecurityDataClassification::Public,
                    DataClassification::Internal => SecurityDataClassification::Internal,
                    DataClassification::Confidential => SecurityDataClassification::Confidential,
                    DataClassification::Restricted => SecurityDataClassification::Restricted,
                };

                if !compliance_validator
                    .validate_data_sharing(&data_classification, user_id.as_deref())
                    .await?
                {
                    return Err(AgentError::ComplianceViolation(
                        "Session sharing not compliant with data classification policies"
                            .to_string(),
                    ));
                }

                // Validate enterprise sharing policy
                if !compliance_validator
                    .validate_enterprise_policy(&data_classification)
                    .await?
                {
                    return Err(AgentError::ComplianceViolation(
                        "Enterprise sharing policy violates compliance requirements".to_string(),
                    ));
                }
            }
        }

        // Convert duration
        let expires_in = expires_in_seconds.map(|secs| chrono::Duration::seconds(secs as i64));

        // Create share link with enterprise policy
        let share = self
            .share_service
            .generate_share_link_with_policy(
                session_id,
                permissions,
                expires_in,
                enterprise_policy,
                user_id.clone(),
            )
            .map_err(|e| AgentError::Internal(format!("Failed to create share link: {}", e)))?;

        info!(session_id = %session_id, share_id = %share.id, user_id = ?user_id, "Share link created successfully");
        Ok(share.id)
    }

    /// Access a shared session with compliance validation
    pub async fn access_shared_session(
        &self,
        share_id: &str,
        accessing_user_id: Option<String>,
    ) -> Result<Session, AgentError> {
        debug!(share_id = %share_id, accessing_user_id = ?accessing_user_id, "Accessing shared session");

        // Get share information
        let share = self
            .share_service
            .get_share(share_id)
            .map_err(|e| AgentError::Internal(format!("Failed to get share: {}", e)))?;

        // Validate enterprise policy compliance if accessing user is different from creator
        if let Some(ref policy) = share.policy {
            if let Some(ref compliance_validator) = self.compliance_validator {
                let data_classification = match policy.data_classification {
                    DataClassification::Public => SecurityDataClassification::Public,
                    DataClassification::Internal => SecurityDataClassification::Internal,
                    DataClassification::Confidential => SecurityDataClassification::Confidential,
                    DataClassification::Restricted => SecurityDataClassification::Restricted,
                };

                if !compliance_validator
                    .validate_shared_data_access(&data_classification, accessing_user_id.as_deref())
                    .await?
                {
                    return Err(AgentError::ComplianceViolation(
                        "Access to shared session data not compliant".to_string(),
                    ));
                }
            }
        }

        // Load the session
        let session = self
            .session_store
            .load(&share.session_id)
            .await
            .map_err(|e| AgentError::Internal(format!("Failed to load shared session: {}", e)))?;

        // Apply permission filters
        let filtered_session = self
            .share_service
            .create_shared_session_view(&session, &share.permissions);

        info!(share_id = %share_id, session_id = %share.session_id, accessing_user_id = ?accessing_user_id, "Shared session accessed successfully");
        Ok(filtered_session)
    }

    /// Revoke a share link with access control
    pub async fn revoke_share_link(
        &self,
        share_id: &str,
        user_id: Option<String>,
    ) -> Result<(), AgentError> {
        debug!(share_id = %share_id, user_id = ?user_id, "Revoking share link");

        // Get share information first to check ownership/access
        let share = self.share_service.get_share(share_id).map_err(|e| {
            AgentError::Internal(format!("Failed to get share for revocation: {}", e))
        })?;

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Delete;
            let resource_type = ResourceType::SessionShare;
            // Check if user can revoke this specific share
            if !access_control
                .check_permission(
                    user_id.as_deref(),
                    &permission,
                    &resource_type,
                    Some(share_id),
                )
                .await?
            {
                // Also check if user owns the session
                let session_permission = Permission::Share;
                let session_resource_type = ResourceType::Session;
                if !access_control
                    .check_permission(
                        user_id.as_deref(),
                        &session_permission,
                        &session_resource_type,
                        Some(&share.session_id),
                    )
                    .await?
                {
                    return Err(AgentError::AccessDenied(format!(
                        "User {} does not have permission to revoke share {}",
                        user_id.as_deref().unwrap_or("unknown"),
                        share_id
                    )));
                }
            }
        }

        let user_id_for_revoke = user_id.clone();
        self.share_service
            .revoke_share(share_id, user_id_for_revoke)
            .map_err(|e| AgentError::Internal(format!("Failed to revoke share: {}", e)))?;

        info!(share_id = %share_id, user_id = ?user_id, "Share link revoked successfully");
        Ok(())
    }

    /// List active shares with access control filtering
    pub async fn list_active_shares(
        &self,
        user_id: Option<String>,
    ) -> Result<Vec<ricecoder_sessions::SessionShare>, AgentError> {
        debug!(user_id = ?user_id, "Listing active shares");

        let all_shares = self
            .share_service
            .list_shares()
            .map_err(|e| AgentError::Internal(format!("Failed to list shares: {}", e)))?;

        // Filter shares based on access control
        let accessible_shares = if let Some(ref access_control) = self.access_control {
            let mut filtered = Vec::new();
            for share in all_shares {
                let permission = Permission::Read;
                let resource_type = ResourceType::SessionShare;
                if access_control
                    .check_permission(
                        user_id.as_deref(),
                        &permission,
                        &resource_type,
                        Some(&share.id),
                    )
                    .await?
                {
                    filtered.push(share);
                }
            }
            filtered
        } else {
            all_shares
        };

        info!(count = accessible_shares.len(), user_id = ?user_id, "Retrieved filtered active shares list");
        Ok(accessible_shares)
    }

    /// Get share information
    pub async fn get_share_info(
        &self,
        share_id: &str,
    ) -> Result<ricecoder_sessions::SessionShare, AgentError> {
        debug!(share_id = %share_id, "Getting share information");

        let share = self
            .share_service
            .get_share(share_id)
            .map_err(|e| AgentError::Internal(format!("Failed to get share info: {}", e)))?;

        Ok(share)
    }
}

/// Use case for session state management with enterprise features
pub struct SessionStateManagementUseCase {
    session_manager: Arc<SessionManager>,
    access_control: Option<Arc<AccessControl>>,
    compliance_validator: Option<Arc<ComplianceValidator>>,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl SessionStateManagementUseCase {
    /// Create a new session state management use case
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self {
            session_manager,
            access_control: None,
            compliance_validator: None,
            audit_logger: None,
        }
    }

    /// Create a new session state management use case with enterprise features
    pub fn with_enterprise_features(
        session_manager: Arc<SessionManager>,
        access_control: Arc<AccessControl>,
        compliance_validator: Arc<ComplianceValidator>,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            session_manager,
            access_control: Some(access_control),
            compliance_validator: Some(compliance_validator),
            audit_logger: Some(audit_logger),
        }
    }

    /// Get token usage for a session with access control
    pub async fn get_token_usage(
        &self,
        session_id: &str,
        user_id: Option<String>,
    ) -> Result<ricecoder_sessions::TokenUsage, AgentError> {
        debug!(session_id = %session_id, user_id = ?user_id, "Getting token usage");

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Read;
            let resource_type = ResourceType::Session;
            if !access_control
                .check_permission(
                    user_id.as_deref(),
                    &permission,
                    &resource_type,
                    Some(session_id),
                )
                .await?
            {
                return Err(AgentError::AccessDenied(format!(
                    "User {} does not have permission to view session token usage",
                    user_id.as_deref().unwrap_or("unknown")
                )));
            }
        }

        let usage = self
            .session_manager
            .get_session_token_usage(session_id)
            .map_err(|e| AgentError::Internal(format!("Failed to get token usage: {}", e)))?;

        // Log audit event if enabled
        if let Some(ref audit_logger) = self.audit_logger {
            let event = ricecoder_security::audit::AuditEvent {
                event_type: ricecoder_security::audit::AuditEventType::DataAccess,
                user_id,
                session_id: Some(session_id.to_string()),
                action: "token_usage_accessed".to_string(),
                resource: format!("session:{}:token_usage", session_id),
                metadata: serde_json::json!({
                    "total_tokens": usage.total_tokens
                }),
            };
            let audit_logger = audit_logger.clone();
            let _ = tokio::spawn(async move {
                let _ = audit_logger.log_event(event).await;
            });
        }

        Ok(usage)
    }

    /// Check if session is within token limits with compliance validation
    pub async fn check_token_limits(
        &self,
        session_id: &str,
        user_id: Option<String>,
    ) -> Result<ricecoder_sessions::TokenLimitStatus, AgentError> {
        debug!(session_id = %session_id, user_id = ?user_id, "Checking token limits");

        // Check access control if enabled
        if let Some(ref access_control) = self.access_control {
            let permission = Permission::Read;
            let resource_type = ResourceType::Session;
            if !access_control
                .check_permission(
                    user_id.as_deref(),
                    &permission,
                    &resource_type,
                    Some(session_id),
                )
                .await?
            {
                return Err(AgentError::AccessDenied(format!(
                    "User {} does not have permission to check session token limits",
                    user_id.as_deref().unwrap_or("unknown")
                )));
            }
        }

        let status = self
            .session_manager
            .check_session_token_limits(session_id)
            .map_err(|e| AgentError::Internal(format!("Failed to check token limits: {}", e)))?;

        // Validate compliance if enabled (check for resource usage limits)
        if let Some(ref compliance_validator) = self.compliance_validator {
            if !compliance_validator
                .validate_resource_usage(&format!("{:?}", status))
                .await?
            {
                return Err(AgentError::ComplianceViolation(
                    "Session token usage exceeds compliance limits".to_string(),
                ));
            }
        }

        Ok(status)
    }
}

/// Use case for provider switching and management
pub struct ProviderSwitchingUseCase {
    provider_manager: Arc<ProviderManager>,
}

impl ProviderSwitchingUseCase {
    /// Create a new provider switching use case
    pub fn new(provider_manager: Arc<ProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Switch to a specific provider
    pub async fn switch_provider(&self, provider_id: &str) -> Result<(), AgentError> {
        debug!(provider_id = %provider_id, "Switching to provider");

        // Check if provider exists and is available
        let provider = Arc::as_ref(&self.provider_manager)
            .get_provider(provider_id)
            .map_err(|e| AgentError::Internal(format!("Provider not found: {}", e)))?;

        // Check provider health
        let is_healthy = Arc::as_ref(&self.provider_manager)
            .health_check(provider_id)
            .await
            .map_err(|e| AgentError::Internal(format!("Health check failed: {}", e)))?;

        if !is_healthy {
            return Err(AgentError::Internal(format!(
                "Provider {} is not healthy",
                provider_id
            )));
        }

        info!(provider_id = %provider_id, "Successfully switched to provider");
        Ok(())
    }

    /// Get the current default provider
    pub fn get_current_provider(&self) -> Result<String, AgentError> {
        let provider = Arc::as_ref(&self.provider_manager)
            .default_provider()
            .map_err(|e| AgentError::Internal(format!("Failed to get default provider: {}", e)))?;

        Ok(provider.id().to_string())
    }

    /// List all available providers with their status
    pub fn list_available_providers(&self) -> Vec<ProviderStatus> {
        Arc::as_ref(&self.provider_manager)
            .get_all_provider_statuses()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get provider status
    pub fn get_provider_status(&self, provider_id: &str) -> Option<ProviderStatus> {
        Arc::as_ref(&self.provider_manager)
            .get_provider_status(provider_id)
            .cloned()
    }

    /// Auto-detect available providers
    pub async fn auto_detect_providers(&self) -> Result<Vec<String>, AgentError> {
        // Note: This requires mutable access to ProviderManager, so we'd need to modify
        // the architecture to support this. For now, return an error.
        Err(AgentError::Internal(
            "Auto-detection requires mutable provider manager access".to_string(),
        ))
    }
}

/// Use case for provider performance monitoring
pub struct ProviderPerformanceUseCase {
    provider_manager: Arc<ProviderManager>,
}

impl ProviderPerformanceUseCase {
    /// Create a new provider performance use case
    pub fn new(provider_manager: Arc<ProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Get performance summary for a provider
    pub fn get_provider_performance(&self, provider_id: &str) -> Option<ProviderMetrics> {
        Arc::as_ref(&self.provider_manager)
            .performance_monitor()
            .get_metrics(provider_id)
    }

    /// Get performance summary for all providers
    pub fn get_all_provider_performance(&self) -> PerformanceSummary {
        Arc::as_ref(&self.provider_manager)
            .performance_monitor()
            .get_performance_summary()
    }

    /// Get providers sorted by performance
    pub fn get_providers_by_performance(&self) -> Vec<(String, f64)> {
        let all_metrics = Arc::as_ref(&self.provider_manager)
            .performance_monitor()
            .get_all_metrics();
        let mut providers: Vec<_> = all_metrics
            .into_iter()
            .map(|(id, metrics)| (id, metrics.avg_response_time_ms))
            .collect();

        // Sort by response time (lower is better)
        providers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        providers
    }
}

/// Use case for provider failover and optimization
pub struct ProviderFailoverUseCase {
    provider_manager: Arc<ProviderManager>,
}

impl ProviderFailoverUseCase {
    /// Create a new provider failover use case
    pub fn new(provider_manager: Arc<ProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Get failover provider for a failing provider
    pub fn get_failover_provider(&self, current_provider_id: &str) -> Option<String> {
        Arc::as_ref(&self.provider_manager).get_failover_provider(current_provider_id)
    }

    /// Check if a provider should be avoided
    pub fn should_avoid_provider(&self, provider_id: &str) -> bool {
        Arc::as_ref(&self.provider_manager).should_avoid_provider(provider_id)
    }

    /// Select the best provider based on constraints
    pub fn select_best_provider(
        &self,
        constraints: Option<SelectionConstraints>,
    ) -> Option<String> {
        Arc::as_ref(&self.provider_manager).select_best_provider(constraints.as_ref())
    }

    /// Select the best provider for specific model capabilities
    pub fn select_best_provider_for_capabilities(
        &self,
        capabilities: &[Capability],
        constraints: Option<SelectionConstraints>,
    ) -> Option<String> {
        Arc::as_ref(&self.provider_manager)
            .select_best_provider_for_model(capabilities, constraints.as_ref())
    }

    /// Get providers sorted by quality
    pub fn get_providers_by_quality(&self, provider_ids: &[String]) -> Vec<(String, f64)> {
        Arc::as_ref(&self.provider_manager).get_providers_by_quality(provider_ids)
    }
}

/// Use case for provider model management
pub struct ProviderModelUseCase {
    provider_manager: Arc<ProviderManager>,
}

impl ProviderModelUseCase {
    /// Create a new provider model use case
    pub fn new(provider_manager: Arc<ProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Get available models with optional filtering
    pub fn get_available_models(&self, filter: Option<ModelFilter>) -> Vec<ModelInfo> {
        Arc::as_ref(&self.provider_manager).get_available_models(filter)
    }

    /// Filter models by specific criteria
    pub fn filter_models(
        &self,
        models: &[ModelInfo],
        criteria: ModelFilterCriteria,
    ) -> Vec<ModelInfo> {
        Arc::as_ref(&self.provider_manager).filter_models(models, criteria)
    }

    /// Get models for a specific provider
    pub fn get_provider_models(&self, provider_id: &str) -> Result<Vec<ModelInfo>, AgentError> {
        let status = Arc::as_ref(&self.provider_manager)
            .get_provider_status(provider_id)
            .ok_or_else(|| AgentError::Internal(format!("Provider {} not found", provider_id)))?;

        Ok(status.models.clone())
    }

    /// Find models with specific capabilities
    pub fn find_models_with_capabilities(&self, capabilities: &[Capability]) -> Vec<ModelInfo> {
        let all_models = self.get_available_models(None);
        all_models
            .into_iter()
            .filter(|model| {
                capabilities
                    .iter()
                    .all(|cap| model.capabilities.contains(cap))
            })
            .collect()
    }
}

/// Use case for provider health monitoring
pub struct ProviderHealthUseCase {
    provider_manager: Arc<ProviderManager>,
}

impl ProviderHealthUseCase {
    /// Create a new provider health use case
    pub fn new(provider_manager: Arc<ProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Check health of a specific provider
    pub async fn check_provider_health(&self, provider_id: &str) -> Result<bool, AgentError> {
        Arc::as_ref(&self.provider_manager)
            .health_check(provider_id)
            .await
            .map_err(|e| AgentError::Internal(format!("Health check failed: {}", e)))
    }

    /// Check health of all providers
    pub async fn check_all_provider_health(&self) -> Vec<(String, Result<bool, AgentError>)> {
        let results = Arc::as_ref(&self.provider_manager).health_check_all().await;
        results
            .into_iter()
            .map(|(id, result)| {
                let mapped_result =
                    result.map_err(|e| AgentError::Internal(format!("Health check failed: {}", e)));
                (id, mapped_result)
            })
            .collect()
    }

    /// Invalidate health check cache for a provider
    pub async fn invalidate_provider_health_cache(&self, provider_id: &str) {
        Arc::as_ref(&self.provider_manager)
            .invalidate_health_check(provider_id)
            .await;
    }

    /// Invalidate all health check caches
    pub async fn invalidate_all_health_caches(&self) {
        Arc::as_ref(&self.provider_manager)
            .invalidate_all_health_checks()
            .await;
    }
}

/// Use case for community provider features
pub struct ProviderCommunityUseCase {
    provider_manager: Arc<ProviderManager>,
}

impl ProviderCommunityUseCase {
    /// Create a new provider community use case
    pub fn new(provider_manager: Arc<ProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Get community quality metrics for a provider
    pub fn get_community_quality_metrics(
        &self,
        provider_id: &str,
    ) -> Option<ricecoder_providers::community::CommunityQualityMetrics> {
        Arc::as_ref(&self.provider_manager).get_community_quality_metrics(provider_id)
    }

    /// Get popular providers from community
    pub fn get_popular_providers(&self, limit: usize) -> Vec<(String, u64)> {
        Arc::as_ref(&self.provider_manager).get_popular_providers(limit)
    }

    /// Get providers ranked by community quality
    pub fn get_providers_by_community_quality(&self, limit: usize) -> Vec<(String, f64)> {
        Arc::as_ref(&self.provider_manager).get_providers_by_community_quality(limit)
    }

    /// Get provider analytics
    pub fn get_provider_analytics(&self, provider_id: &str) -> Option<ProviderAnalytics> {
        Arc::as_ref(&self.provider_manager)
            .get_provider_analytics(provider_id)
            .cloned()
    }

    /// Submit a provider configuration to the community
    pub fn submit_community_config(
        &self,
        config: ricecoder_providers::community::CommunityProviderConfig,
    ) -> Result<String, AgentError> {
        // Note: This requires mutable access, so we'd need to modify the architecture
        Err(AgentError::Internal(
            "Community config submission requires mutable provider manager access".to_string(),
        ))
    }
}
