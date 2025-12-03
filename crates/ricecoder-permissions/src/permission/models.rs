//! Permission data models

use serde::{Deserialize, Serialize};

/// Permission level for tool access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// Tool execution is allowed without prompting
    Allow,
    /// User is prompted before tool execution
    Ask,
    /// Tool execution is denied
    Deny,
}

impl PermissionLevel {
    /// Check if this permission level is more restrictive than another
    pub fn is_more_restrictive_than(&self, other: PermissionLevel) -> bool {
        matches!((self, other), (PermissionLevel::Deny, _) | (PermissionLevel::Ask, PermissionLevel::Allow))
    }
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionLevel::Allow => write!(f, "allow"),
            PermissionLevel::Ask => write!(f, "ask"),
            PermissionLevel::Deny => write!(f, "deny"),
        }
    }
}

/// Permission for a specific tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    /// Glob pattern for matching tool names
    pub tool_pattern: String,
    /// Permission level
    pub level: PermissionLevel,
    /// Optional agent-specific override (None means global)
    pub agent: Option<String>,
}

impl ToolPermission {
    /// Create a new tool permission
    pub fn new(tool_pattern: String, level: PermissionLevel) -> Self {
        Self {
            tool_pattern,
            level,
            agent: None,
        }
    }

    /// Create a new tool permission with agent override
    pub fn with_agent(tool_pattern: String, level: PermissionLevel, agent: String) -> Self {
        Self {
            tool_pattern,
            level,
            agent: Some(agent),
        }
    }

    /// Check if this permission applies to a specific agent
    pub fn applies_to_agent(&self, agent: Option<&str>) -> bool {
        match (&self.agent, agent) {
            (None, _) => true,                    // Global permission applies to all
            (Some(perm_agent), Some(check_agent)) => perm_agent == check_agent,
            (Some(_), None) => false,             // Agent-specific doesn't apply to global
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_level_display() {
        assert_eq!(PermissionLevel::Allow.to_string(), "allow");
        assert_eq!(PermissionLevel::Ask.to_string(), "ask");
        assert_eq!(PermissionLevel::Deny.to_string(), "deny");
    }

    #[test]
    fn test_permission_level_restrictiveness() {
        assert!(PermissionLevel::Deny.is_more_restrictive_than(PermissionLevel::Allow));
        assert!(PermissionLevel::Deny.is_more_restrictive_than(PermissionLevel::Ask));
        assert!(PermissionLevel::Ask.is_more_restrictive_than(PermissionLevel::Allow));
        assert!(!PermissionLevel::Allow.is_more_restrictive_than(PermissionLevel::Ask));
    }

    #[test]
    fn test_tool_permission_creation() {
        let perm = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);
        assert_eq!(perm.tool_pattern, "test_*");
        assert_eq!(perm.level, PermissionLevel::Allow);
        assert_eq!(perm.agent, None);
    }

    #[test]
    fn test_tool_permission_with_agent() {
        let perm = ToolPermission::with_agent(
            "test_*".to_string(),
            PermissionLevel::Deny,
            "agent1".to_string(),
        );
        assert_eq!(perm.tool_pattern, "test_*");
        assert_eq!(perm.level, PermissionLevel::Deny);
        assert_eq!(perm.agent, Some("agent1".to_string()));
    }

    #[test]
    fn test_tool_permission_applies_to_agent() {
        let global_perm = ToolPermission::new("*".to_string(), PermissionLevel::Allow);
        let agent_perm = ToolPermission::with_agent(
            "*".to_string(),
            PermissionLevel::Deny,
            "agent1".to_string(),
        );

        // Global permission applies to all
        assert!(global_perm.applies_to_agent(None));
        assert!(global_perm.applies_to_agent(Some("agent1")));
        assert!(global_perm.applies_to_agent(Some("agent2")));

        // Agent-specific permission only applies to that agent
        assert!(!agent_perm.applies_to_agent(None));
        assert!(agent_perm.applies_to_agent(Some("agent1")));
        assert!(!agent_perm.applies_to_agent(Some("agent2")));
    }

    #[test]
    fn test_permission_level_serialization() {
        let level = PermissionLevel::Allow;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"allow\"");

        let deserialized: PermissionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, PermissionLevel::Allow);
    }

    #[test]
    fn test_tool_permission_serialization() {
        let perm = ToolPermission::new("test_*".to_string(), PermissionLevel::Ask);
        let json = serde_json::to_string(&perm).unwrap();
        let deserialized: ToolPermission = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tool_pattern, perm.tool_pattern);
        assert_eq!(deserialized.level, perm.level);
        assert_eq!(deserialized.agent, perm.agent);
    }
}
