//! Skill permission system (Gaps G-17-03, G-17-04, G-17-12)
//!
//! Implements OpenCode-compatible wildcard permission matching with allow/deny/ask semantics.
//! Supports per-session approved caching to avoid re-prompting.

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Permission action (OpenCode Config.Permission)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillPermissionAction {
    /// Ask user for permission each time
    Ask,
    /// Allow without prompting
    Allow,
    /// Deny access
    Deny,
}

/// Skill permission rules with wildcard pattern support (Gap G-17-03)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPermission {
    /// Permission rules: pattern -> action
    /// Example: { "*": "allow", "dangerous-*": "deny", "review-*": "ask" }
    pub rules: HashMap<String, SkillPermissionAction>,
}

impl Default for SkillPermission {
    fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert("*".to_string(), SkillPermissionAction::Allow);
        Self { rules }
    }
}

impl SkillPermission {
    /// Create a new permission set with custom rules
    pub fn new(rules: HashMap<String, SkillPermissionAction>) -> Self {
        Self { rules }
    }

    /// Check permission for a skill name using wildcard matching (OpenCode Wildcard.all logic)
    /// Returns the action from the most specific matching pattern
    pub fn check(&self, skill_name: &str) -> SkillPermissionAction {
        // Sort by pattern length (ascending) then alphabetically
        // This ensures longer (more specific) patterns override shorter ones
        let mut sorted_patterns: Vec<_> = self.rules.iter().collect();
        sorted_patterns.sort_by(|(a, _), (b, _)| {
            a.len().cmp(&b.len()).then_with(|| a.cmp(b))
        });

        // Return the last matching action (most specific wins)
        let mut result = SkillPermissionAction::Deny; // Default if no match
        for (pattern, action) in sorted_patterns {
            if wildcard_match(skill_name, pattern) {
                result = *action;
            }
        }
        result
    }
}

/// Wildcard pattern matching (OpenCode Wildcard.match logic)
/// Supports `*` (any sequence) and `?` (single char)
fn wildcard_match(text: &str, pattern: &str) -> bool {
    // Convert glob pattern to regex-like matching
    let regex_pattern = pattern
        .replace('.', "\\.")
        .replace('*', ".*")
        .replace('?', ".");
    
    regex::Regex::new(&format!("^{}$", regex_pattern))
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

/// Per-session approved skill cache (Gap G-17-04)
/// Implements OpenCode Permission.ask caching with "always" support
#[derive(Debug, Clone)]
pub struct SkillPermissionChecker {
    /// Session-specific approved patterns
    /// Tracks patterns that user approved with "always" response
    approved_cache: Arc<RwLock<HashSet<String>>>,
}

impl Default for SkillPermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillPermissionChecker {
    /// Create a new permission checker
    pub fn new() -> Self {
        Self {
            approved_cache: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check if a skill is already approved in this session
    pub fn is_approved(&self, skill_name: &str) -> bool {
        self.approved_cache
            .read()
            .map(|cache| cache.contains(skill_name))
            .unwrap_or(false)
    }

    /// Add a skill to approved cache (user selected "always")
    pub fn approve(&self, skill_name: &str) {
        if let Ok(mut cache) = self.approved_cache.write() {
            cache.insert(skill_name.to_string());
        }
    }

    /// Clear all approved skills (e.g., on session end)
    pub fn clear(&self) {
        if let Ok(mut cache) = self.approved_cache.write() {
            cache.clear();
        }
    }

    /// Check permission with caching (combines rules + cache)
    /// Returns final action after considering cache
    pub fn check_with_cache(
        &self,
        skill_name: &str,
        rules: &SkillPermission,
    ) -> SkillPermissionAction {
        // Check cache first
        if self.is_approved(skill_name) {
            return SkillPermissionAction::Allow;
        }

        // Fall back to rule-based check
        rules.check(skill_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_match() {
        assert!(wildcard_match("hello", "hello"));
        assert!(wildcard_match("hello-world", "hello-*"));
        assert!(wildcard_match("test", "*"));
        assert!(wildcard_match("a", "?"));
        assert!(!wildcard_match("hello", "world"));
    }

    #[test]
    fn test_permission_check_precedence() {
        let mut rules = HashMap::new();
        rules.insert("*".to_string(), SkillPermissionAction::Allow);
        rules.insert("dangerous-*".to_string(), SkillPermissionAction::Deny);
        rules.insert("review-*".to_string(), SkillPermissionAction::Ask);
        
        let perms = SkillPermission::new(rules);
        
        assert_eq!(perms.check("anything"), SkillPermissionAction::Allow);
        assert_eq!(perms.check("dangerous-op"), SkillPermissionAction::Deny);
        assert_eq!(perms.check("review-code"), SkillPermissionAction::Ask);
    }

    #[test]
    fn test_permission_cache() {
        let checker = SkillPermissionChecker::new();
        
        assert!(!checker.is_approved("test-skill"));
        checker.approve("test-skill");
        assert!(checker.is_approved("test-skill"));
        
        checker.clear();
        assert!(!checker.is_approved("test-skill"));
    }

    #[test]
    fn test_check_with_cache() {
        let checker = SkillPermissionChecker::new();
        let mut rules = HashMap::new();
        rules.insert("*".to_string(), SkillPermissionAction::Ask);
        let perms = SkillPermission::new(rules);

        // First check: should be Ask
        assert_eq!(
            checker.check_with_cache("my-skill", &perms),
            SkillPermissionAction::Ask
        );

        // Approve it
        checker.approve("my-skill");

        // Second check: should be Allow (cached)
        assert_eq!(
            checker.check_with_cache("my-skill", &perms),
            SkillPermissionAction::Allow
        );
    }
}
