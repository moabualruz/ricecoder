//! Permission Manager integration for MCP tools

use crate::error::{Error, Result};
use ricecoder_permissions::{GlobMatcher, PermissionLevel};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Permission rule for tool access control
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionRule {
    pub pattern: String,
    pub level: PermissionLevelConfig,
    pub agent_id: Option<String>,
}

/// Permission level configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevelConfig {
    Allow,
    Ask,
    Deny,
}

impl From<PermissionLevelConfig> for PermissionLevel {
    fn from(level: PermissionLevelConfig) -> Self {
        match level {
            PermissionLevelConfig::Allow => PermissionLevel::Allow,
            PermissionLevelConfig::Ask => PermissionLevel::Ask,
            PermissionLevelConfig::Deny => PermissionLevel::Deny,
        }
    }
}

/// MCP Permission Manager for controlling tool access
#[derive(Debug, Clone)]
pub struct MCPPermissionManager {
    global_rules: Vec<PermissionRule>,
    agent_rules: HashMap<String, Vec<PermissionRule>>,
    glob_matcher: GlobMatcher,
}

impl MCPPermissionManager {
    /// Creates a new MCP Permission Manager
    pub fn new() -> Self {
        Self {
            global_rules: Vec::new(),
            agent_rules: HashMap::new(),
            glob_matcher: GlobMatcher::new(),
        }
    }

    /// Adds a global permission rule
    pub fn add_global_rule(&mut self, rule: PermissionRule) -> Result<()> {
        // Validate the pattern
        self.glob_matcher
            .validate_pattern(&rule.pattern)
            .map_err(|e| Error::ValidationError(format!("Invalid pattern: {}", e)))?;

        self.global_rules.push(rule);
        Ok(())
    }

    /// Adds a per-agent permission rule
    pub fn add_agent_rule(&mut self, agent_id: String, rule: PermissionRule) -> Result<()> {
        // Validate the pattern
        self.glob_matcher
            .validate_pattern(&rule.pattern)
            .map_err(|e| Error::ValidationError(format!("Invalid pattern: {}", e)))?;

        self.agent_rules
            .entry(agent_id)
            .or_insert_with(Vec::new)
            .push(rule);
        Ok(())
    }

    /// Checks permission for a tool execution
    pub fn check_permission(
        &self,
        tool_id: &str,
        agent_id: Option<&str>,
    ) -> Result<PermissionLevel> {
        // Check per-agent rules first (higher priority)
        if let Some(agent_id) = agent_id {
            if let Some(rules) = self.agent_rules.get(agent_id) {
                if let Some(level) = self.match_rules(tool_id, rules)? {
                    debug!(
                        "Tool '{}' permission for agent '{}': {:?}",
                        tool_id, agent_id, level
                    );
                    return Ok(level);
                }
            }
        }

        // Check global rules
        if let Some(level) = self.match_rules(tool_id, &self.global_rules)? {
            debug!("Tool '{}' global permission: {:?}", tool_id, level);
            return Ok(level);
        }

        // Default to deny if no rule matches
        warn!(
            "No permission rule found for tool '{}', defaulting to deny",
            tool_id
        );
        Ok(PermissionLevel::Deny)
    }

    /// Matches a tool ID against a set of rules
    fn match_rules(
        &self,
        tool_id: &str,
        rules: &[PermissionRule],
    ) -> Result<Option<PermissionLevel>> {
        for rule in rules {
            if self.glob_matcher.match_pattern(&rule.pattern, tool_id) {
                let level = PermissionLevelConfig::clone(&rule.level).into();
                return Ok(Some(level));
            }
        }
        Ok(None)
    }

    /// Gets all global rules
    pub fn get_global_rules(&self) -> &[PermissionRule] {
        &self.global_rules
    }

    /// Gets all agent rules for a specific agent
    pub fn get_agent_rules(&self, agent_id: &str) -> Option<&[PermissionRule]> {
        self.agent_rules.get(agent_id).map(|v| v.as_slice())
    }

    /// Clears all rules
    pub fn clear_rules(&mut self) {
        self.global_rules.clear();
        self.agent_rules.clear();
    }
}

impl Default for MCPPermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_permission_manager() {
        let manager = MCPPermissionManager::new();
        assert!(manager.get_global_rules().is_empty());
    }

    #[test]
    fn test_add_global_rule() {
        let mut manager = MCPPermissionManager::new();
        let rule = PermissionRule {
            pattern: "database-*".to_string(),
            level: PermissionLevelConfig::Allow,
            agent_id: None,
        };

        let result = manager.add_global_rule(rule);
        assert!(result.is_ok());
        assert_eq!(manager.get_global_rules().len(), 1);
    }

    #[test]
    fn test_add_agent_rule() {
        let mut manager = MCPPermissionManager::new();
        let rule = PermissionRule {
            pattern: "code-*".to_string(),
            level: PermissionLevelConfig::Allow,
            agent_id: Some("code-analyzer".to_string()),
        };

        let result = manager.add_agent_rule("code-analyzer".to_string(), rule);
        assert!(result.is_ok());
        assert!(manager.get_agent_rules("code-analyzer").is_some());
    }

    #[test]
    fn test_check_permission_allow() {
        let mut manager = MCPPermissionManager::new();
        let rule = PermissionRule {
            pattern: "database-*".to_string(),
            level: PermissionLevelConfig::Allow,
            agent_id: None,
        };

        manager.add_global_rule(rule).unwrap();

        let result = manager.check_permission("database-query", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PermissionLevel::Allow);
    }

    #[test]
    fn test_check_permission_deny() {
        let mut manager = MCPPermissionManager::new();
        let rule = PermissionRule {
            pattern: "dangerous-*".to_string(),
            level: PermissionLevelConfig::Deny,
            agent_id: None,
        };

        manager.add_global_rule(rule).unwrap();

        let result = manager.check_permission("dangerous-operation", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PermissionLevel::Deny);
    }

    #[test]
    fn test_check_permission_ask() {
        let mut manager = MCPPermissionManager::new();
        let rule = PermissionRule {
            pattern: "api-*".to_string(),
            level: PermissionLevelConfig::Ask,
            agent_id: None,
        };

        manager.add_global_rule(rule).unwrap();

        let result = manager.check_permission("api-call", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PermissionLevel::Ask);
    }

    #[test]
    fn test_per_agent_override() {
        let mut manager = MCPPermissionManager::new();

        // Global rule: deny
        let global_rule = PermissionRule {
            pattern: "file-*".to_string(),
            level: PermissionLevelConfig::Deny,
            agent_id: None,
        };
        manager.add_global_rule(global_rule).unwrap();

        // Per-agent rule: allow
        let agent_rule = PermissionRule {
            pattern: "file-*".to_string(),
            level: PermissionLevelConfig::Allow,
            agent_id: Some("file-manager".to_string()),
        };
        manager
            .add_agent_rule("file-manager".to_string(), agent_rule)
            .unwrap();

        // Check permission for agent - should use per-agent rule
        let result = manager.check_permission("file-read", Some("file-manager"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PermissionLevel::Allow);

        // Check permission for other agent - should use global rule
        let result = manager.check_permission("file-read", Some("other-agent"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PermissionLevel::Deny);
    }

    #[test]
    fn test_default_deny() {
        let manager = MCPPermissionManager::new();

        // No rules defined - should default to deny
        let result = manager.check_permission("unknown-tool", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PermissionLevel::Deny);
    }

    #[test]
    fn test_clear_rules() {
        let mut manager = MCPPermissionManager::new();
        let rule = PermissionRule {
            pattern: "test-*".to_string(),
            level: PermissionLevelConfig::Allow,
            agent_id: None,
        };

        manager.add_global_rule(rule).unwrap();
        assert_eq!(manager.get_global_rules().len(), 1);

        manager.clear_rules();
        assert!(manager.get_global_rules().is_empty());
    }
}
