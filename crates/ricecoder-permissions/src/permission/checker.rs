//! Permission checking logic

use crate::{
    error::Result,
    permission::models::{PermissionLevel, ToolPermission},
};

/// Decision result from permission checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Tool execution is allowed
    Allow,
    /// User should be prompted before execution
    Ask,
    /// Tool execution is denied
    Deny,
}

impl std::fmt::Display for PermissionDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionDecision::Allow => write!(f, "allow"),
            PermissionDecision::Ask => write!(f, "ask"),
            PermissionDecision::Deny => write!(f, "deny"),
        }
    }
}

/// Permission checker for evaluating tool access
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check permission for a tool
    ///
    /// # Arguments
    /// * `permissions` - List of applicable permissions
    /// * `agent` - Optional agent name for per-agent overrides
    /// * `default_level` - Default permission level if no match found
    ///
    /// # Returns
    /// The most restrictive permission that applies, with agent-specific permissions overriding global
    pub fn check_permission(
        permissions: &[ToolPermission],
        agent: Option<&str>,
        default_level: PermissionLevel,
    ) -> Result<PermissionDecision> {
        // Separate agent-specific and global permissions
        let agent_specific_perms: Vec<&ToolPermission> = permissions
            .iter()
            .filter(|p| p.agent.is_some() && p.applies_to_agent(agent))
            .collect();

        let global_perms: Vec<&ToolPermission> =
            permissions.iter().filter(|p| p.agent.is_none()).collect();

        // Determine which permissions to use
        // If there are agent-specific permissions, use only those (they override global)
        // Otherwise, use global permissions
        let applicable_perms = if !agent_specific_perms.is_empty() {
            agent_specific_perms
        } else {
            global_perms
        };

        if applicable_perms.is_empty() {
            // No specific permission found, use default
            return Ok(Self::level_to_decision(default_level));
        }

        // Find the most restrictive permission among applicable ones
        let most_restrictive = applicable_perms
            .iter()
            .fold(applicable_perms[0], |acc, perm| {
                if perm.level.is_more_restrictive_than(acc.level) {
                    perm
                } else {
                    acc
                }
            });

        Ok(Self::level_to_decision(most_restrictive.level))
    }

    /// Convert a permission level to a decision
    fn level_to_decision(level: PermissionLevel) -> PermissionDecision {
        match level {
            PermissionLevel::Allow => PermissionDecision::Allow,
            PermissionLevel::Ask => PermissionDecision::Ask,
            PermissionLevel::Deny => PermissionDecision::Deny,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_decision_display() {
        assert_eq!(PermissionDecision::Allow.to_string(), "allow");
        assert_eq!(PermissionDecision::Ask.to_string(), "ask");
        assert_eq!(PermissionDecision::Deny.to_string(), "deny");
    }

    #[test]
    fn test_check_permission_allow() {
        let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Allow);
        let decision =
            PermissionChecker::check_permission(&[perm], None, PermissionLevel::Ask).unwrap();

        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_check_permission_ask() {
        let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Ask);
        let decision =
            PermissionChecker::check_permission(&[perm], None, PermissionLevel::Allow).unwrap();

        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_check_permission_deny() {
        let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Deny);
        let decision =
            PermissionChecker::check_permission(&[perm], None, PermissionLevel::Allow).unwrap();

        assert_eq!(decision, PermissionDecision::Deny);
    }

    #[test]
    fn test_check_permission_no_match_uses_default() {
        let decision =
            PermissionChecker::check_permission(&[], None, PermissionLevel::Ask).unwrap();

        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_check_permission_most_restrictive() {
        let perm1 = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);
        let perm2 = ToolPermission::new("test_tool".to_string(), PermissionLevel::Deny);

        let decision =
            PermissionChecker::check_permission(&[perm1, perm2], None, PermissionLevel::Ask)
                .unwrap();

        // Should use the most restrictive (Deny)
        assert_eq!(decision, PermissionDecision::Deny);
    }

    #[test]
    fn test_check_permission_per_agent_override() {
        let global_perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Allow);
        let agent_perm = ToolPermission::with_agent(
            "test_tool".to_string(),
            PermissionLevel::Deny,
            "agent1".to_string(),
        );

        // For agent1, should use agent-specific permission (Deny)
        let decision = PermissionChecker::check_permission(
            &[global_perm.clone(), agent_perm.clone()],
            Some("agent1"),
            PermissionLevel::Ask,
        )
        .unwrap();

        assert_eq!(decision, PermissionDecision::Deny);

        // For agent2, should use global permission (Allow)
        let decision = PermissionChecker::check_permission(
            &[global_perm],
            Some("agent2"),
            PermissionLevel::Ask,
        )
        .unwrap();

        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_check_permission_agent_override_most_restrictive() {
        let global_allow = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);
        let global_deny = ToolPermission::new("test_tool".to_string(), PermissionLevel::Deny);
        let agent_ask = ToolPermission::with_agent(
            "test_tool".to_string(),
            PermissionLevel::Ask,
            "agent1".to_string(),
        );

        // For agent1, agent-specific permissions override global
        // Agent-specific: agent_ask (Ask)
        // Result: Ask (agent-specific overrides global)
        let decision = PermissionChecker::check_permission(
            &[global_allow, global_deny, agent_ask.clone()],
            Some("agent1"),
            PermissionLevel::Allow,
        )
        .unwrap();

        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_check_permission_multiple_global_most_restrictive() {
        let perm1 = ToolPermission::new("test_*".to_string(), PermissionLevel::Allow);
        let perm2 = ToolPermission::new("test_*".to_string(), PermissionLevel::Ask);
        let perm3 = ToolPermission::new("test_tool".to_string(), PermissionLevel::Deny);

        let decision = PermissionChecker::check_permission(
            &[perm1, perm2, perm3],
            None,
            PermissionLevel::Allow,
        )
        .unwrap();

        // Should use the most restrictive (Deny)
        assert_eq!(decision, PermissionDecision::Deny);
    }

    #[test]
    fn test_check_permission_agent_specific_only() {
        let agent_perm = ToolPermission::with_agent(
            "test_tool".to_string(),
            PermissionLevel::Deny,
            "agent1".to_string(),
        );

        // For agent1, should use agent-specific permission
        let decision = PermissionChecker::check_permission(
            &[agent_perm.clone()],
            Some("agent1"),
            PermissionLevel::Allow,
        )
        .unwrap();

        assert_eq!(decision, PermissionDecision::Deny);

        // For agent2, should use default (no applicable permissions)
        let decision = PermissionChecker::check_permission(
            &[agent_perm],
            Some("agent2"),
            PermissionLevel::Allow,
        )
        .unwrap();

        assert_eq!(decision, PermissionDecision::Allow);
    }
}
