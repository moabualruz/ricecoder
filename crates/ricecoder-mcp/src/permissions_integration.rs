//! Integration with ricecoder-permissions framework
//!
//! This module provides integration between the MCP tool system and the ricecoder-permissions
//! framework, enabling permission checking and enforcement for tool execution within agent workflows.

use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Permission level for tool execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPermissionLevel {
    /// Tool execution is allowed without prompting
    Allow,
    /// User is prompted before tool execution
    Ask,
    /// Tool execution is denied
    Deny,
}

impl ToolPermissionLevel {
    /// Convert from ricecoder-permissions PermissionLevel
    pub fn from_permission_level(level: &str) -> Option<Self> {
        match level {
            "allow" => Some(ToolPermissionLevel::Allow),
            "ask" => Some(ToolPermissionLevel::Ask),
            "deny" => Some(ToolPermissionLevel::Deny),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolPermissionLevel::Allow => "allow",
            ToolPermissionLevel::Ask => "ask",
            ToolPermissionLevel::Deny => "deny",
        }
    }
}

/// Tool permission decision
#[derive(Debug, Clone)]
pub struct ToolPermissionDecision {
    /// The tool ID being checked
    pub tool_id: String,
    /// The permission level
    pub level: ToolPermissionLevel,
    /// The agent ID (if any)
    pub agent_id: Option<String>,
    /// Whether this is a per-agent override
    pub is_override: bool,
    /// Reason for the decision
    pub reason: String,
}

impl ToolPermissionDecision {
    /// Creates a new permission decision
    pub fn new(tool_id: String, level: ToolPermissionLevel, reason: String) -> Self {
        Self {
            tool_id,
            level,
            agent_id: None,
            is_override: false,
            reason,
        }
    }

    /// Sets the agent ID
    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Marks this as an override
    pub fn as_override(mut self) -> Self {
        self.is_override = true;
        self
    }
}

/// Tool permission checker for agents
///
/// This trait allows agents to check permissions before executing tools.
pub trait ToolPermissionChecker: Send + Sync {
    /// Check permission for a tool execution
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to check
    /// * `agent_id` - The ID of the agent executing the tool (optional)
    ///
    /// # Returns
    ///
    /// The permission decision
    fn check_permission(&self, tool_id: &str, agent_id: Option<&str>) -> Result<ToolPermissionDecision>;

    /// Check permission and return whether execution is allowed
    fn is_allowed(&self, tool_id: &str, agent_id: Option<&str>) -> Result<bool> {
        let decision = self.check_permission(tool_id, agent_id)?;
        Ok(decision.level == ToolPermissionLevel::Allow)
    }

    /// Check permission and return whether user should be prompted
    fn requires_prompt(&self, tool_id: &str, agent_id: Option<&str>) -> Result<bool> {
        let decision = self.check_permission(tool_id, agent_id)?;
        Ok(decision.level == ToolPermissionLevel::Ask)
    }

    /// Check permission and return whether execution is denied
    fn is_denied(&self, tool_id: &str, agent_id: Option<&str>) -> Result<bool> {
        let decision = self.check_permission(tool_id, agent_id)?;
        Ok(decision.level == ToolPermissionLevel::Deny)
    }
}

/// Tool permission prompt for user interaction
#[derive(Debug, Clone)]
pub struct ToolPermissionPrompt {
    /// The tool ID being requested
    pub tool_id: String,
    /// The tool name
    pub tool_name: String,
    /// The tool description
    pub tool_description: String,
    /// The parameters being passed to the tool
    pub parameters: HashMap<String, Value>,
    /// The agent ID requesting the tool
    pub agent_id: Option<String>,
}

impl ToolPermissionPrompt {
    /// Creates a new permission prompt
    pub fn new(
        tool_id: String,
        tool_name: String,
        tool_description: String,
        parameters: HashMap<String, Value>,
    ) -> Self {
        Self {
            tool_id,
            tool_name,
            tool_description,
            parameters,
            agent_id: None,
        }
    }

    /// Sets the agent ID
    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Gets a formatted prompt message
    pub fn format_message(&self) -> String {
        let mut msg = format!(
            "Tool Execution Request\n\
             =====================\n\
             Tool: {} ({})\n\
             Description: {}\n",
            self.tool_name, self.tool_id, self.tool_description
        );

        if let Some(agent_id) = &self.agent_id {
            msg.push_str(&format!("Requested by: {}\n", agent_id));
        }

        if !self.parameters.is_empty() {
            msg.push_str("\nParameters:\n");
            for (key, value) in &self.parameters {
                msg.push_str(&format!("  {}: {}\n", key, value));
            }
        }

        msg.push_str("\nAllow execution? (yes/no)");
        msg
    }
}

/// User decision for permission prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserPermissionDecision {
    /// User approved the tool execution
    Approved,
    /// User denied the tool execution
    Denied,
    /// User cancelled the operation
    Cancelled,
}

/// Tool permission enforcement for agent workflows
///
/// This struct provides permission checking and enforcement for tool execution
/// within agent workflows.
pub struct ToolPermissionEnforcer {
    checker: Arc<dyn ToolPermissionChecker>,
}

impl ToolPermissionEnforcer {
    /// Creates a new tool permission enforcer
    pub fn new(checker: Arc<dyn ToolPermissionChecker>) -> Self {
        Self { checker }
    }

    /// Checks if a tool can be executed
    pub fn can_execute(&self, tool_id: &str, agent_id: Option<&str>) -> Result<bool> {
        self.checker.is_allowed(tool_id, agent_id)
    }

    /// Checks if a tool execution requires user prompt
    pub fn requires_user_prompt(&self, tool_id: &str, agent_id: Option<&str>) -> Result<bool> {
        self.checker.requires_prompt(tool_id, agent_id)
    }

    /// Checks if a tool execution is denied
    pub fn is_execution_denied(&self, tool_id: &str, agent_id: Option<&str>) -> Result<bool> {
        self.checker.is_denied(tool_id, agent_id)
    }

    /// Gets the permission decision for a tool
    pub fn get_decision(&self, tool_id: &str, agent_id: Option<&str>) -> Result<ToolPermissionDecision> {
        self.checker.check_permission(tool_id, agent_id)
    }

    /// Logs a permission decision
    pub fn log_decision(&self, decision: &ToolPermissionDecision) {
        let agent_info = decision
            .agent_id
            .as_ref()
            .map(|id| format!(" (agent: {})", id))
            .unwrap_or_default();

        let override_info = if decision.is_override {
            " [OVERRIDE]"
        } else {
            ""
        };

        tracing::info!(
            "Tool permission: {} - {} for tool {}{}{}",
            decision.level.as_str(),
            decision.reason,
            decision.tool_id,
            agent_info,
            override_info
        );
    }

    /// Logs a permission denial
    pub fn log_denial(&self, tool_id: &str, agent_id: Option<&str>, reason: &str) {
        let agent_info = agent_id
            .map(|id| format!(" (agent: {})", id))
            .unwrap_or_default();

        tracing::warn!(
            "Tool execution denied: {} for tool {}{}",
            reason,
            tool_id,
            agent_info
        );
    }
}

/// Permission-aware tool execution wrapper
///
/// This struct wraps tool execution with permission checking.
pub struct PermissionAwareToolExecution {
    enforcer: Arc<ToolPermissionEnforcer>,
}

impl PermissionAwareToolExecution {
    /// Creates a new permission-aware tool execution wrapper
    pub fn new(enforcer: Arc<ToolPermissionEnforcer>) -> Self {
        Self { enforcer }
    }

    /// Checks if a tool can be executed and returns the decision
    pub async fn check_and_execute<F, T>(
        &self,
        tool_id: &str,
        agent_id: Option<&str>,
        execute_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Check permission
        let decision = self.enforcer.get_decision(tool_id, agent_id)?;
        self.enforcer.log_decision(&decision);

        match decision.level {
            ToolPermissionLevel::Allow => {
                // Execute the tool
                execute_fn()
            }
            ToolPermissionLevel::Ask => {
                // In a real implementation, this would prompt the user
                // For now, we'll return an error indicating user prompt is needed
                Err(crate::error::Error::PermissionDenied(format!(
                    "Tool execution requires user approval: {}",
                    tool_id
                )))
            }
            ToolPermissionLevel::Deny => {
                self.enforcer.log_denial(tool_id, agent_id, "Permission denied");
                Err(crate::error::Error::PermissionDenied(format!(
                    "Tool execution denied: {}",
                    tool_id
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPermissionChecker;

    impl ToolPermissionChecker for MockPermissionChecker {
        fn check_permission(
            &self,
            tool_id: &str,
            agent_id: Option<&str>,
        ) -> Result<ToolPermissionDecision> {
            let level = if tool_id.contains("allowed") {
                ToolPermissionLevel::Allow
            } else if tool_id.contains("ask") {
                ToolPermissionLevel::Ask
            } else {
                ToolPermissionLevel::Deny
            };

            Ok(ToolPermissionDecision::new(
                tool_id.to_string(),
                level,
                "Mock decision".to_string(),
            )
            .with_agent(agent_id.unwrap_or("unknown").to_string()))
        }
    }

    #[test]
    fn test_tool_permission_level_conversion() {
        assert_eq!(
            ToolPermissionLevel::from_permission_level("allow"),
            Some(ToolPermissionLevel::Allow)
        );
        assert_eq!(
            ToolPermissionLevel::from_permission_level("ask"),
            Some(ToolPermissionLevel::Ask)
        );
        assert_eq!(
            ToolPermissionLevel::from_permission_level("deny"),
            Some(ToolPermissionLevel::Deny)
        );
        assert_eq!(ToolPermissionLevel::from_permission_level("invalid"), None);
    }

    #[test]
    fn test_tool_permission_level_as_str() {
        assert_eq!(ToolPermissionLevel::Allow.as_str(), "allow");
        assert_eq!(ToolPermissionLevel::Ask.as_str(), "ask");
        assert_eq!(ToolPermissionLevel::Deny.as_str(), "deny");
    }

    #[test]
    fn test_tool_permission_decision_creation() {
        let decision = ToolPermissionDecision::new(
            "tool-1".to_string(),
            ToolPermissionLevel::Allow,
            "Test reason".to_string(),
        );

        assert_eq!(decision.tool_id, "tool-1");
        assert_eq!(decision.level, ToolPermissionLevel::Allow);
        assert_eq!(decision.reason, "Test reason");
        assert!(decision.agent_id.is_none());
        assert!(!decision.is_override);
    }

    #[test]
    fn test_tool_permission_decision_with_agent() {
        let decision = ToolPermissionDecision::new(
            "tool-1".to_string(),
            ToolPermissionLevel::Allow,
            "Test reason".to_string(),
        )
        .with_agent("agent-1".to_string())
        .as_override();

        assert_eq!(decision.agent_id, Some("agent-1".to_string()));
        assert!(decision.is_override);
    }

    #[test]
    fn test_tool_permission_prompt_creation() {
        let mut params = HashMap::new();
        params.insert("param1".to_string(), serde_json::json!("value1"));

        let prompt = ToolPermissionPrompt::new(
            "tool-1".to_string(),
            "Tool 1".to_string(),
            "A test tool".to_string(),
            params,
        );

        assert_eq!(prompt.tool_id, "tool-1");
        assert_eq!(prompt.tool_name, "Tool 1");
        assert_eq!(prompt.tool_description, "A test tool");
        assert!(prompt.agent_id.is_none());
    }

    #[test]
    fn test_tool_permission_prompt_format_message() {
        let mut params = HashMap::new();
        params.insert("param1".to_string(), serde_json::json!("value1"));

        let prompt = ToolPermissionPrompt::new(
            "tool-1".to_string(),
            "Tool 1".to_string(),
            "A test tool".to_string(),
            params,
        )
        .with_agent("agent-1".to_string());

        let message = prompt.format_message();
        assert!(message.contains("Tool 1"));
        assert!(message.contains("tool-1"));
        assert!(message.contains("A test tool"));
        assert!(message.contains("agent-1"));
        assert!(message.contains("param1"));
    }

    #[test]
    fn test_tool_permission_enforcer_creation() {
        let checker: Arc<dyn ToolPermissionChecker> = Arc::new(MockPermissionChecker);
        let enforcer = ToolPermissionEnforcer::new(checker);

        assert!(enforcer.can_execute("allowed-tool", None).is_ok());
    }

    #[test]
    fn test_tool_permission_enforcer_can_execute() {
        let checker: Arc<dyn ToolPermissionChecker> = Arc::new(MockPermissionChecker);
        let enforcer = ToolPermissionEnforcer::new(checker);

        assert!(enforcer.can_execute("allowed-tool", None).unwrap());
        assert!(!enforcer.can_execute("denied-tool", None).unwrap());
    }

    #[test]
    fn test_tool_permission_enforcer_requires_prompt() {
        let checker: Arc<dyn ToolPermissionChecker> = Arc::new(MockPermissionChecker);
        let enforcer = ToolPermissionEnforcer::new(checker);

        assert!(enforcer.requires_user_prompt("ask-tool", None).unwrap());
        assert!(!enforcer.requires_user_prompt("allowed-tool", None).unwrap());
    }

    #[test]
    fn test_tool_permission_enforcer_is_denied() {
        let checker: Arc<dyn ToolPermissionChecker> = Arc::new(MockPermissionChecker);
        let enforcer = ToolPermissionEnforcer::new(checker);

        assert!(enforcer.is_execution_denied("denied-tool", None).unwrap());
        assert!(!enforcer.is_execution_denied("allowed-tool", None).unwrap());
    }

    #[tokio::test]
    async fn test_permission_aware_tool_execution_allowed() {
        let checker: Arc<dyn ToolPermissionChecker> = Arc::new(MockPermissionChecker);
        let enforcer = Arc::new(ToolPermissionEnforcer::new(checker));
        let execution = PermissionAwareToolExecution::new(enforcer);

        let result = execution
            .check_and_execute("allowed-tool", None, || Ok(42))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_permission_aware_tool_execution_denied() {
        let checker: Arc<dyn ToolPermissionChecker> = Arc::new(MockPermissionChecker);
        let enforcer = Arc::new(ToolPermissionEnforcer::new(checker));
        let execution = PermissionAwareToolExecution::new(enforcer);

        let result = execution
            .check_and_execute("denied-tool", None, || Ok(42))
            .await;

        assert!(result.is_err());
    }
}
