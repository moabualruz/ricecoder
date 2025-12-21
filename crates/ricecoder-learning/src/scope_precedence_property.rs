/// Property-based tests for scope precedence enforcement
/// **Feature: ricecoder-learning, Property 3: Scope Precedence Enforcement**
/// **Validates: Requirements 2.1**

#[cfg(test)]
mod tests {
    use crate::{ConflictResolver, Rule, RuleScope, RuleSource};
    use proptest::prelude::*;

    proptest! {
        /// Property: Project rules override global rules when both exist
        /// For any conflicting rules in different scopes, the Learning System SHALL apply
        /// project rules over global rules when both exist.
        #[test]
        fn prop_project_rules_override_global(
            project_pattern in "[a-z0-9]{1,20}",
            project_action in "[a-z0-9]{1,20}",
            global_action in "[a-z0-9]{1,20}",
        ) {
            // Create project and global rules with the same pattern
            let mut project_rule = Rule::new(
                RuleScope::Project,
                project_pattern.clone(),
                project_action,
                RuleSource::Learned,
            );
            project_rule.id = "project_rule".to_string();

            let mut global_rule = Rule::new(
                RuleScope::Global,
                project_pattern.clone(),
                global_action,
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let rules = vec![global_rule.clone(), project_rule.clone()];

            // Apply precedence
            let selected = ConflictResolver::apply_precedence(&rules);

            // Project rule should be selected
            prop_assert!(selected.is_some());
            prop_assert_eq!(selected.unwrap().id, "project_rule");
        }

        /// Property: Global rules override session rules when both exist
        /// For any conflicting rules in different scopes, the Learning System SHALL apply
        /// global rules over session rules when both exist.
        #[test]
        fn prop_global_rules_override_session(
            pattern in "[a-z0-9]{1,20}",
            global_action in "[a-z0-9]{1,20}",
            session_action in "[a-z0-9]{1,20}",
        ) {
            let mut global_rule = Rule::new(
                RuleScope::Global,
                pattern.clone(),
                global_action,
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let mut session_rule = Rule::new(
                RuleScope::Session,
                pattern,
                session_action,
                RuleSource::Learned,
            );
            session_rule.id = "session_rule".to_string();

            let rules = vec![session_rule.clone(), global_rule.clone()];

            // Apply precedence
            let selected = ConflictResolver::apply_precedence(&rules);

            // Global rule should be selected
            prop_assert!(selected.is_some());
            prop_assert_eq!(selected.unwrap().id, "global_rule");
        }

        /// Property: Project rules override session rules when both exist
        /// For any conflicting rules in different scopes, the Learning System SHALL apply
        /// project rules over session rules when both exist.
        #[test]
        fn prop_project_rules_override_session(
            pattern in "[a-z0-9]{1,20}",
            project_action in "[a-z0-9]{1,20}",
            session_action in "[a-z0-9]{1,20}",
        ) {
            let mut project_rule = Rule::new(
                RuleScope::Project,
                pattern.clone(),
                project_action,
                RuleSource::Learned,
            );
            project_rule.id = "project_rule".to_string();

            let mut session_rule = Rule::new(
                RuleScope::Session,
                pattern,
                session_action,
                RuleSource::Learned,
            );
            session_rule.id = "session_rule".to_string();

            let rules = vec![session_rule.clone(), project_rule.clone()];

            // Apply precedence
            let selected = ConflictResolver::apply_precedence(&rules);

            // Project rule should be selected
            prop_assert!(selected.is_some());
            prop_assert_eq!(selected.unwrap().id, "project_rule");
        }

        /// Property: Project > Global > Session precedence is always maintained
        /// For any set of rules with all three scopes, the precedence order
        /// Project > Global > Session must be maintained.
        #[test]
        fn prop_full_precedence_order(
            pattern in "[a-z0-9]{1,20}",
            project_action in "[a-z0-9]{1,20}",
            global_action in "[a-z0-9]{1,20}",
            session_action in "[a-z0-9]{1,20}",
        ) {
            let mut project_rule = Rule::new(
                RuleScope::Project,
                pattern.clone(),
                project_action,
                RuleSource::Learned,
            );
            project_rule.id = "project_rule".to_string();

            let mut global_rule = Rule::new(
                RuleScope::Global,
                pattern.clone(),
                global_action,
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let mut session_rule = Rule::new(
                RuleScope::Session,
                pattern,
                session_action,
                RuleSource::Learned,
            );
            session_rule.id = "session_rule".to_string();

            // Test all permutations of rule order
            let permutations = vec![
                vec![project_rule.clone(), global_rule.clone(), session_rule.clone()],
                vec![project_rule.clone(), session_rule.clone(), global_rule.clone()],
                vec![global_rule.clone(), project_rule.clone(), session_rule.clone()],
                vec![global_rule.clone(), session_rule.clone(), project_rule.clone()],
                vec![session_rule.clone(), project_rule.clone(), global_rule.clone()],
                vec![session_rule.clone(), global_rule.clone(), project_rule.clone()],
            ];

            for rules in permutations {
                let selected = ConflictResolver::apply_precedence(&rules);
                prop_assert!(selected.is_some());
                // Project rule should always be selected regardless of order
                prop_assert_eq!(selected.unwrap().id, "project_rule");
            }
        }

        /// Property: Resolve conflicts maintains precedence for multiple patterns
        /// When resolving conflicts across multiple patterns, each pattern group
        /// should have the highest precedence rule selected.
        #[test]
        fn prop_resolve_conflicts_maintains_precedence(
            pattern1 in "[a-z0-9]{1,20}",
            pattern2 in "[a-z0-9]{1,20}",
        ) {
            prop_assume!(pattern1 != pattern2);

            let mut project_rule1 = Rule::new(
                RuleScope::Project,
                pattern1.clone(),
                "project_action1".to_string(),
                RuleSource::Learned,
            );
            project_rule1.id = "project_rule1".to_string();

            let mut global_rule1 = Rule::new(
                RuleScope::Global,
                pattern1.clone(),
                "global_action1".to_string(),
                RuleSource::Learned,
            );
            global_rule1.id = "global_rule1".to_string();

            let mut global_rule2 = Rule::new(
                RuleScope::Global,
                pattern2.clone(),
                "global_action2".to_string(),
                RuleSource::Learned,
            );
            global_rule2.id = "global_rule2".to_string();

            let mut session_rule2 = Rule::new(
                RuleScope::Session,
                pattern2.clone(),
                "session_action2".to_string(),
                RuleSource::Learned,
            );
            session_rule2.id = "session_rule2".to_string();

            let rules = vec![
                project_rule1.clone(),
                global_rule1.clone(),
                global_rule2.clone(),
                session_rule2.clone(),
            ];

            let resolved = ConflictResolver::resolve_conflicts(&rules).unwrap();

            // Should have 2 rules (one per pattern)
            prop_assert_eq!(resolved.len(), 2);

            // Pattern 1 should have project rule
            let pattern1_rule = resolved.iter().find(|r| r.pattern == pattern1);
            prop_assert!(pattern1_rule.is_some());
            prop_assert_eq!(pattern1_rule.unwrap().id.as_str(), "project_rule1");

            // Pattern 2 should have global rule (higher precedence than session)
            let pattern2_rule = resolved.iter().find(|r| r.pattern == pattern2);
            prop_assert!(pattern2_rule.is_some());
            prop_assert_eq!(pattern2_rule.unwrap().id.as_str(), "global_rule2");
        }

        /// Property: Get highest priority rule returns project rule when available
        /// For any pattern with rules in multiple scopes, get_highest_priority_rule
        /// should return the project rule if available.
        #[test]
        fn prop_get_highest_priority_returns_project(
            pattern in "[a-z0-9]{1,20}",
        ) {
            let mut project_rule = Rule::new(
                RuleScope::Project,
                pattern.clone(),
                "project_action".to_string(),
                RuleSource::Learned,
            );
            project_rule.id = "project_rule".to_string();

            let mut global_rule = Rule::new(
                RuleScope::Global,
                pattern.clone(),
                "global_action".to_string(),
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let mut session_rule = Rule::new(
                RuleScope::Session,
                pattern.clone(),
                "session_action".to_string(),
                RuleSource::Learned,
            );
            session_rule.id = "session_rule".to_string();

            let rules = vec![global_rule, session_rule, project_rule.clone()];

            let highest = ConflictResolver::get_highest_priority_rule(&rules, &pattern);

            prop_assert!(highest.is_some());
            prop_assert_eq!(highest.unwrap().id, "project_rule");
        }

        /// Property: Get highest priority rule returns global when project unavailable
        /// For any pattern with rules in global and session scopes (no project),
        /// get_highest_priority_rule should return the global rule.
        #[test]
        fn prop_get_highest_priority_returns_global_when_no_project(
            pattern in "[a-z0-9]{1,20}",
        ) {
            let mut global_rule = Rule::new(
                RuleScope::Global,
                pattern.clone(),
                "global_action".to_string(),
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let mut session_rule = Rule::new(
                RuleScope::Session,
                pattern.clone(),
                "session_action".to_string(),
                RuleSource::Learned,
            );
            session_rule.id = "session_rule".to_string();

            let rules = vec![session_rule, global_rule.clone()];

            let highest = ConflictResolver::get_highest_priority_rule(&rules, &pattern);

            prop_assert!(highest.is_some());
            prop_assert_eq!(highest.unwrap().id, "global_rule");
        }

        /// Property: Scope precedence is independent of rule order
        /// The precedence order should be maintained regardless of the order
        /// in which rules are provided.
        #[test]
        fn prop_precedence_independent_of_order(
            pattern in "[a-z0-9]{1,20}",
        ) {
            let mut project_rule = Rule::new(
                RuleScope::Project,
                pattern.clone(),
                "project_action".to_string(),
                RuleSource::Learned,
            );
            project_rule.id = "project_rule".to_string();

            let mut global_rule = Rule::new(
                RuleScope::Global,
                pattern.clone(),
                "global_action".to_string(),
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let mut session_rule = Rule::new(
                RuleScope::Session,
                pattern.clone(),
                "session_action".to_string(),
                RuleSource::Learned,
            );
            session_rule.id = "session_rule".to_string();

            // Test multiple orderings
            let orderings = vec![
                vec![project_rule.clone(), global_rule.clone(), session_rule.clone()],
                vec![session_rule.clone(), project_rule.clone(), global_rule.clone()],
                vec![global_rule.clone(), session_rule.clone(), project_rule.clone()],
            ];

            let mut results = Vec::new();
            for rules in orderings {
                let selected = ConflictResolver::apply_precedence(&rules);
                results.push(selected.map(|r| r.id));
            }

            // All results should be the same (project_rule)
            prop_assert!(results.iter().all(|r| r == &Some("project_rule".to_string())));
        }

        /// Property: Cross-scope conflict detection works correctly
        /// When checking for conflicts between project and global rules,
        /// conflicts should be detected when patterns match but actions differ.
        #[test]
        fn prop_cross_scope_conflict_detection(
            pattern in "[a-z0-9]{1,20}",
            project_action in "[a-z0-9]{1,20}",
            global_action in "[a-z0-9]{1,20}",
        ) {
            prop_assume!(project_action != global_action);

            let mut project_rule = Rule::new(
                RuleScope::Project,
                pattern.clone(),
                project_action,
                RuleSource::Learned,
            );
            project_rule.id = "project_rule".to_string();

            let mut global_rule = Rule::new(
                RuleScope::Global,
                pattern,
                global_action,
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let conflicts = ConflictResolver::check_cross_scope_conflicts(
                &[project_rule.clone()],
                &[global_rule.clone()],
            );

            // Should detect the conflict
            prop_assert_eq!(conflicts.len(), 1);
            prop_assert_eq!(conflicts[0].0.id.as_str(), "project_rule");
            prop_assert_eq!(conflicts[0].1.id.as_str(), "global_rule");
        }

        /// Property: No conflicts when patterns differ
        /// When rules have different patterns, no conflicts should be detected
        /// even if they have different actions.
        #[test]
        fn prop_no_conflict_different_patterns(
            pattern1 in "[a-z0-9]{1,20}",
            pattern2 in "[a-z0-9]{1,20}",
        ) {
            prop_assume!(pattern1 != pattern2);

            let mut project_rule = Rule::new(
                RuleScope::Project,
                pattern1,
                "action1".to_string(),
                RuleSource::Learned,
            );
            project_rule.id = "project_rule".to_string();

            let mut global_rule = Rule::new(
                RuleScope::Global,
                pattern2,
                "action2".to_string(),
                RuleSource::Learned,
            );
            global_rule.id = "global_rule".to_string();

            let conflicts = ConflictResolver::check_cross_scope_conflicts(
                &[project_rule],
                &[global_rule],
            );

            // Should not detect any conflicts
            prop_assert_eq!(conflicts.len(), 0);
        }
    }
}
