//! Agent execution with permission checking
//!
//! This module provides integration between the permissions system and agent execution.
//! It handles permission checking, prompting, and audit logging for tool execution.

use crate::audit::AuditLogger;
use crate::error::Result;
use crate::permission::{PermissionChecker, PermissionConfig, PermissionDecision};
use crate::prompt::{PermissionPrompt, PromptResult};
use std::sync::Arc;

/// Result of agent tool execution with permission decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentExecutionResult {
    /// Tool was allowed to execute
    Allowed,
    /// Tool execution was denied
    Denied,
    /// User was prompted and approved
    Approved,
    /// User was prompted and denied
    UserDenied,
}

impl std::fmt::Display for AgentExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentExecutionResult::Allowed => write!(f, "allowed"),
            AgentExecutionResult::Denied => write!(f, "denied"),
            AgentExecutionResult::Approved => write!(f, "approved"),
            AgentExecutionResult::UserDenied => write!(f, "user_denied"),
        }
    }
}

/// Agent execution context with permission checking
pub struct AgentExecutor {
    /// Permission configuration
    config: Arc<PermissionConfig>,
    /// Audit logger for recording decisions
    audit_logger: Arc<AuditLogger>,
    /// Optional agent name for per-agent overrides
    agent_name: Option<String>,
}

impl AgentExecutor {
    /// Create a new agent executor
    pub fn new(config: Arc<PermissionConfig>, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            config,
            audit_logger,
            agent_name: None,
        }
    }

    /// Create a new agent executor with a specific agent name
    pub fn with_agent(
        config: Arc<PermissionConfig>,
        audit_logger: Arc<AuditLogger>,
        agent_name: String,
    ) -> Self {
        Self {
            config,
            audit_logger,
            agent_name: Some(agent_name),
        }
    }

    /// Check permission and execute tool with permission handling
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool to execute
    /// * `tool_description` - Optional description of the tool
    /// * `action_description` - Optional description of what the tool will do
    /// * `execute_fn` - Closure that executes the tool
    ///
    /// # Returns
    /// Result containing the execution result and tool output
    pub fn execute_with_permission<F, T>(
        &self,
        tool_name: &str,
        tool_description: Option<&str>,
        action_description: Option<&str>,
        execute_fn: F,
    ) -> Result<(AgentExecutionResult, Option<T>)>
    where
        F: FnOnce() -> Result<T>,
    {
        // Check permission for this tool
        let decision = self.check_permission(tool_name)?;

        match decision {
            PermissionDecision::Allow => {
                // Log the execution
                self.audit_logger
                    .log_execution(tool_name.to_string(), self.agent_name.clone(), None)
                    .map_err(crate::error::Error::Internal)?;

                // Execute the tool
                let result = execute_fn()?;
                Ok((AgentExecutionResult::Allowed, Some(result)))
            }
            PermissionDecision::Ask => {
                // Show permission prompt
                let prompt = PermissionPrompt::new(tool_name.to_string());
                let prompt = if let Some(desc) = tool_description {
                    prompt.with_description(desc.to_string())
                } else {
                    prompt
                };
                let prompt = if let Some(action) = action_description {
                    prompt.with_action(action.to_string())
                } else {
                    prompt
                };

                // Log the prompt
                self.audit_logger
                    .log_prompt(tool_name.to_string(), self.agent_name.clone(), None)
                    .map_err(crate::error::Error::Internal)?;

                // Collect user decision
                let prompt_result = prompt
                    .execute()
                    .map_err(|e| crate::error::Error::PromptError(e.to_string()))?;

                match prompt_result {
                    PromptResult::Approved => {
                        // Log the execution
                        self.audit_logger
                            .log_execution(
                                tool_name.to_string(),
                                self.agent_name.clone(),
                                Some("User approved via prompt".to_string()),
                            )
                            .map_err(crate::error::Error::Internal)?;

                        // Execute the tool
                        let result = execute_fn()?;
                        Ok((AgentExecutionResult::Approved, Some(result)))
                    }
                    PromptResult::Denied => {
                        // Log the denial
                        self.audit_logger
                            .log_denial(
                                tool_name.to_string(),
                                self.agent_name.clone(),
                                Some("User denied via prompt".to_string()),
                            )
                            .map_err(crate::error::Error::Internal)?;

                        Ok((AgentExecutionResult::UserDenied, None))
                    }
                }
            }
            PermissionDecision::Deny => {
                // Log the denial
                self.audit_logger
                    .log_denial(
                        tool_name.to_string(),
                        self.agent_name.clone(),
                        Some("Permission denied".to_string()),
                    )
                    .map_err(crate::error::Error::Internal)?;

                Ok((AgentExecutionResult::Denied, None))
            }
        }
    }

    /// Check permission for a tool without executing it
    pub fn check_permission(&self, tool_name: &str) -> Result<PermissionDecision> {
        let permissions = self.config.get_permissions_for_tool(tool_name)?;
        let default_level = self.config.default_permission_level();

        let decision = PermissionChecker::check_permission(
            &permissions,
            self.agent_name.as_deref(),
            default_level,
        )?;

        Ok(decision)
    }

    /// Get the agent name
    pub fn agent_name(&self) -> Option<&str> {
        self.agent_name.as_deref()
    }

    /// Get the audit logger
    pub fn audit_logger(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.audit_logger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::{PermissionLevel, ToolPermission};

    #[test]
    fn test_agent_executor_creation() {
        let config = Arc::new(PermissionConfig::new());
        let logger = Arc::new(AuditLogger::new());
        let executor = AgentExecutor::new(config, logger);

        assert_eq!(executor.agent_name(), None);
    }

    #[test]
    fn test_agent_executor_with_agent_name() {
        let config = Arc::new(PermissionConfig::new());
        let logger = Arc::new(AuditLogger::new());
        let executor = AgentExecutor::with_agent(config, logger, "agent1".to_string());

        assert_eq!(executor.agent_name(), Some("agent1"));
    }

    #[test]
    fn test_check_permission_allow() {
        let mut config = PermissionConfig::new();
        config.add_permission(ToolPermission::new(
            "test_tool".to_string(),
            PermissionLevel::Allow,
        ));

        let config = Arc::new(config);
        let logger = Arc::new(AuditLogger::new());
        let executor = AgentExecutor::new(config, logger);

        let decision = executor.check_permission("test_tool").unwrap();
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_check_permission_deny() {
        let mut config = PermissionConfig::new();
        config.add_permission(ToolPermission::new(
            "test_tool".to_string(),
            PermissionLevel::Deny,
        ));

        let config = Arc::new(config);
        let logger = Arc::new(AuditLogger::new());
        let executor = AgentExecutor::new(config, logger);

        let decision = executor.check_permission("test_tool").unwrap();
        assert_eq!(decision, PermissionDecision::Deny);
    }

    #[test]
    fn test_execute_with_permission_allowed() {
        let mut config = PermissionConfig::new();
        config.add_permission(ToolPermission::new(
            "test_tool".to_string(),
            PermissionLevel::Allow,
        ));

        let config = Arc::new(config);
        let logger = Arc::new(AuditLogger::new());
        let executor = AgentExecutor::new(config, logger.clone());

        let (result, output) = executor
            .execute_with_permission("test_tool", None, None, || Ok(42))
            .unwrap();

        assert_eq!(result, AgentExecutionResult::Allowed);
        assert_eq!(output, Some(42));

        // Check that execution was logged
        let entries = logger.entries().unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_execute_with_permission_denied() {
        let mut config = PermissionConfig::new();
        config.add_permission(ToolPermission::new(
            "test_tool".to_string(),
            PermissionLevel::Deny,
        ));

        let config = Arc::new(config);
        let logger = Arc::new(AuditLogger::new());
        let executor = AgentExecutor::new(config, logger.clone());

        let (result, output) = executor
            .execute_with_permission("test_tool", None, None, || Ok(42))
            .unwrap();

        assert_eq!(result, AgentExecutionResult::Denied);
        assert_eq!(output, None);

        // Check that denial was logged
        let entries = logger.entries().unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_execute_with_permission_per_agent_override() {
        let mut config = PermissionConfig::new();
        config.add_permission(ToolPermission::new(
            "test_tool".to_string(),
            PermissionLevel::Allow,
        ));
        config.add_permission(ToolPermission::with_agent(
            "test_tool".to_string(),
            PermissionLevel::Deny,
            "agent1".to_string(),
        ));

        let config = Arc::new(config);
        let logger = Arc::new(AuditLogger::new());

        // For agent1, should be denied
        let executor =
            AgentExecutor::with_agent(config.clone(), logger.clone(), "agent1".to_string());
        let (result, _) = executor
            .execute_with_permission("test_tool", None, None, || Ok(42))
            .unwrap();
        assert_eq!(result, AgentExecutionResult::Denied);

        // For agent2, should be allowed
        let executor = AgentExecutor::with_agent(config, logger, "agent2".to_string());
        let (result, _) = executor
            .execute_with_permission("test_tool", None, None, || Ok(42))
            .unwrap();
        assert_eq!(result, AgentExecutionResult::Allowed);
    }
}
