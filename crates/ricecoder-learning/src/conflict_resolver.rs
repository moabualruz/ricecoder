/// Conflict detection and resolution for rules
use crate::error::{LearningError, Result};
use crate::models::{Rule, RuleScope};
use std::collections::HashMap;

/// Detects and resolves conflicts between rules
pub struct ConflictResolver;

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self
    }

    /// Detect if two rules conflict
    pub fn detect_conflict(rule1: &Rule, rule2: &Rule) -> bool {
        // Rules conflict if they have the same pattern but different actions
        rule1.pattern == rule2.pattern && rule1.action != rule2.action
    }

    /// Find all conflicts in a set of rules
    pub fn find_conflicts(rules: &[Rule]) -> Vec<(Rule, Rule)> {
        let mut conflicts = Vec::new();

        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                if Self::detect_conflict(&rules[i], &rules[j]) {
                    conflicts.push((rules[i].clone(), rules[j].clone()));
                }
            }
        }

        conflicts
    }

    /// Check if a rule conflicts with existing rules
    pub fn check_conflicts(rule: &Rule, existing_rules: &[Rule]) -> Result<()> {
        for existing_rule in existing_rules {
            if Self::detect_conflict(rule, existing_rule) {
                return Err(LearningError::ConflictResolutionFailed(
                    format!(
                        "Rule '{}' conflicts with existing rule '{}': both match pattern '{}' but have different actions",
                        rule.id, existing_rule.id, rule.pattern
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Apply scope precedence to select the appropriate rule
    /// Project rules override global rules when both exist
    pub fn apply_precedence(rules: &[Rule]) -> Option<Rule> {
        if rules.is_empty() {
            return None;
        }

        // Sort by scope precedence: Project > Global > Session
        let mut sorted_rules = rules.to_vec();
        sorted_rules.sort_by_key(|r| match r.scope {
            RuleScope::Project => 0,
            RuleScope::Global => 1,
            RuleScope::Session => 2,
        });

        Some(sorted_rules[0].clone())
    }

    /// Get rules by pattern, applying scope precedence
    pub fn get_rules_by_pattern_with_precedence(rules: &[Rule], pattern: &str) -> Vec<Rule> {
        let matching_rules: Vec<Rule> = rules
            .iter()
            .filter(|r| r.pattern == pattern)
            .cloned()
            .collect();

        if matching_rules.is_empty() {
            return Vec::new();
        }

        // Group by pattern and apply precedence
        let mut result = Vec::new();
        let mut seen_patterns = std::collections::HashSet::new();

        for rule in &matching_rules {
            if !seen_patterns.contains(&rule.pattern) {
                if let Some(precedent_rule) = Self::apply_precedence(
                    &matching_rules
                        .iter()
                        .filter(|r| r.pattern == rule.pattern)
                        .cloned()
                        .collect::<Vec<_>>(),
                ) {
                    result.push(precedent_rule);
                    seen_patterns.insert(rule.pattern.clone());
                }
            }
        }

        result
    }

    /// Resolve conflicts by applying scope precedence
    pub fn resolve_conflicts(rules: &[Rule]) -> Result<Vec<Rule>> {
        // Group rules by pattern
        let mut pattern_groups: HashMap<String, Vec<Rule>> = HashMap::new();

        for rule in rules {
            pattern_groups
                .entry(rule.pattern.clone())
                .or_default()
                .push(rule.clone());
        }

        // For each pattern group, apply precedence
        let mut resolved_rules = Vec::new();

        for (_, group) in pattern_groups {
            if let Some(rule) = Self::apply_precedence(&group) {
                resolved_rules.push(rule);
            }
        }

        Ok(resolved_rules)
    }

    /// Log conflict resolution decision
    pub fn log_conflict_resolution(selected_rule: &Rule, conflicting_rules: &[Rule]) -> String {
        let conflicting_ids: Vec<String> = conflicting_rules.iter().map(|r| r.id.clone()).collect();

        format!(
            "Conflict resolution: Selected rule '{}' (scope: {}) over conflicting rules: {}",
            selected_rule.id,
            selected_rule.scope,
            conflicting_ids.join(", ")
        )
    }

    /// Get the highest priority rule for a pattern across all scopes
    pub fn get_highest_priority_rule(rules: &[Rule], pattern: &str) -> Option<Rule> {
        let matching_rules: Vec<Rule> = rules
            .iter()
            .filter(|r| r.pattern == pattern)
            .cloned()
            .collect();

        Self::apply_precedence(&matching_rules)
    }

    /// Check if rules in different scopes conflict
    pub fn check_cross_scope_conflicts(
        project_rules: &[Rule],
        global_rules: &[Rule],
    ) -> Vec<(Rule, Rule)> {
        let mut conflicts = Vec::new();

        for project_rule in project_rules {
            for global_rule in global_rules {
                if Self::detect_conflict(project_rule, global_rule) {
                    conflicts.push((project_rule.clone(), global_rule.clone()));
                }
            }
        }

        conflicts
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RuleSource;

    fn create_test_rule(id: &str, scope: RuleScope, pattern: &str, action: &str) -> Rule {
        let mut rule = Rule::new(
            scope,
            pattern.to_string(),
            action.to_string(),
            RuleSource::Learned,
        );
        rule.id = id.to_string();
        rule
    }

    #[test]
    fn test_detect_conflict_same_pattern_different_action() {
        let rule1 = create_test_rule("rule1", RuleScope::Global, "pattern1", "action1");
        let rule2 = create_test_rule("rule2", RuleScope::Global, "pattern1", "action2");

        assert!(ConflictResolver::detect_conflict(&rule1, &rule2));
    }

    #[test]
    fn test_detect_conflict_same_pattern_same_action() {
        let rule1 = create_test_rule("rule1", RuleScope::Global, "pattern1", "action1");
        let rule2 = create_test_rule("rule2", RuleScope::Global, "pattern1", "action1");

        assert!(!ConflictResolver::detect_conflict(&rule1, &rule2));
    }

    #[test]
    fn test_detect_conflict_different_pattern() {
        let rule1 = create_test_rule("rule1", RuleScope::Global, "pattern1", "action1");
        let rule2 = create_test_rule("rule2", RuleScope::Global, "pattern2", "action1");

        assert!(!ConflictResolver::detect_conflict(&rule1, &rule2));
    }

    #[test]
    fn test_find_conflicts() {
        let rules = vec![
            create_test_rule("rule1", RuleScope::Global, "pattern1", "action1"),
            create_test_rule("rule2", RuleScope::Global, "pattern1", "action2"),
            create_test_rule("rule3", RuleScope::Global, "pattern2", "action1"),
        ];

        let conflicts = ConflictResolver::find_conflicts(&rules);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].0.id, "rule1");
        assert_eq!(conflicts[0].1.id, "rule2");
    }

    #[test]
    fn test_check_conflicts_no_conflict() {
        let rule = create_test_rule("rule1", RuleScope::Global, "pattern1", "action1");
        let existing_rules = vec![
            create_test_rule("rule2", RuleScope::Global, "pattern2", "action1"),
            create_test_rule("rule3", RuleScope::Global, "pattern3", "action2"),
        ];

        let result = ConflictResolver::check_conflicts(&rule, &existing_rules);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_conflicts_with_conflict() {
        let rule = create_test_rule("rule1", RuleScope::Global, "pattern1", "action1");
        let existing_rules = vec![create_test_rule(
            "rule2",
            RuleScope::Global,
            "pattern1",
            "action2",
        )];

        let result = ConflictResolver::check_conflicts(&rule, &existing_rules);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_precedence_project_over_global() {
        let rules = vec![
            create_test_rule("rule1", RuleScope::Global, "pattern1", "action1"),
            create_test_rule("rule2", RuleScope::Project, "pattern1", "action2"),
        ];

        let selected = ConflictResolver::apply_precedence(&rules);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "rule2");
    }

    #[test]
    fn test_apply_precedence_global_over_session() {
        let rules = vec![
            create_test_rule("rule1", RuleScope::Session, "pattern1", "action1"),
            create_test_rule("rule2", RuleScope::Global, "pattern1", "action2"),
        ];

        let selected = ConflictResolver::apply_precedence(&rules);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "rule2");
    }

    #[test]
    fn test_apply_precedence_project_over_all() {
        let rules = vec![
            create_test_rule("rule1", RuleScope::Session, "pattern1", "action1"),
            create_test_rule("rule2", RuleScope::Global, "pattern1", "action2"),
            create_test_rule("rule3", RuleScope::Project, "pattern1", "action3"),
        ];

        let selected = ConflictResolver::apply_precedence(&rules);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "rule3");
    }

    #[test]
    fn test_apply_precedence_empty() {
        let rules = vec![];
        let selected = ConflictResolver::apply_precedence(&rules);
        assert!(selected.is_none());
    }

    #[test]
    fn test_resolve_conflicts_multiple_patterns() {
        let rules = vec![
            create_test_rule("rule1", RuleScope::Global, "pattern1", "action1"),
            create_test_rule("rule2", RuleScope::Project, "pattern1", "action2"),
            create_test_rule("rule3", RuleScope::Global, "pattern2", "action1"),
            create_test_rule("rule4", RuleScope::Session, "pattern2", "action2"),
        ];

        let resolved = ConflictResolver::resolve_conflicts(&rules).unwrap();
        assert_eq!(resolved.len(), 2);

        // Check that project rule is selected for pattern1
        let pattern1_rule = resolved.iter().find(|r| r.pattern == "pattern1").unwrap();
        assert_eq!(pattern1_rule.id, "rule2");

        // Check that global rule is selected for pattern2
        let pattern2_rule = resolved.iter().find(|r| r.pattern == "pattern2").unwrap();
        assert_eq!(pattern2_rule.id, "rule3");
    }

    #[test]
    fn test_log_conflict_resolution() {
        let selected = create_test_rule("rule1", RuleScope::Project, "pattern1", "action1");
        let conflicting = vec![create_test_rule(
            "rule2",
            RuleScope::Global,
            "pattern1",
            "action2",
        )];

        let log = ConflictResolver::log_conflict_resolution(&selected, &conflicting);
        assert!(log.contains("rule1"));
        assert!(log.contains("project"));
        assert!(log.contains("rule2"));
    }

    #[test]
    fn test_get_highest_priority_rule() {
        let rules = vec![
            create_test_rule("rule1", RuleScope::Global, "pattern1", "action1"),
            create_test_rule("rule2", RuleScope::Project, "pattern1", "action2"),
            create_test_rule("rule3", RuleScope::Session, "pattern1", "action3"),
        ];

        let highest = ConflictResolver::get_highest_priority_rule(&rules, "pattern1");
        assert!(highest.is_some());
        assert_eq!(highest.unwrap().id, "rule2");
    }

    #[test]
    fn test_get_highest_priority_rule_not_found() {
        let rules = vec![create_test_rule(
            "rule1",
            RuleScope::Global,
            "pattern1",
            "action1",
        )];

        let highest = ConflictResolver::get_highest_priority_rule(&rules, "pattern2");
        assert!(highest.is_none());
    }

    #[test]
    fn test_check_cross_scope_conflicts() {
        let project_rules = vec![create_test_rule(
            "rule1",
            RuleScope::Project,
            "pattern1",
            "action1",
        )];

        let global_rules = vec![create_test_rule(
            "rule2",
            RuleScope::Global,
            "pattern1",
            "action2",
        )];

        let conflicts =
            ConflictResolver::check_cross_scope_conflicts(&project_rules, &global_rules);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_check_cross_scope_no_conflicts() {
        let project_rules = vec![create_test_rule(
            "rule1",
            RuleScope::Project,
            "pattern1",
            "action1",
        )];

        let global_rules = vec![create_test_rule(
            "rule2",
            RuleScope::Global,
            "pattern2",
            "action1",
        )];

        let conflicts =
            ConflictResolver::check_cross_scope_conflicts(&project_rules, &global_rules);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_get_rules_by_pattern_with_precedence() {
        let rules = vec![
            create_test_rule("rule1", RuleScope::Global, "pattern1", "action1"),
            create_test_rule("rule2", RuleScope::Project, "pattern1", "action2"),
            create_test_rule("rule3", RuleScope::Session, "pattern1", "action3"),
            create_test_rule("rule4", RuleScope::Global, "pattern2", "action1"),
        ];

        let pattern1_rules =
            ConflictResolver::get_rules_by_pattern_with_precedence(&rules, "pattern1");
        assert_eq!(pattern1_rules.len(), 1);
        assert_eq!(pattern1_rules[0].id, "rule2");

        let pattern2_rules =
            ConflictResolver::get_rules_by_pattern_with_precedence(&rules, "pattern2");
        assert_eq!(pattern2_rules.len(), 1);
        assert_eq!(pattern2_rules[0].id, "rule4");
    }
}
