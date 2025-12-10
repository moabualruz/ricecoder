/// Property-based tests for scope configuration isolation
/// **Feature: ricecoder-learning, Property 8: Scope Configuration Isolation**
/// **Validates: Requirements 1.5, 1.6, 1.7**

#[cfg(test)]
mod tests {
    use crate::models::{Rule, RuleScope, RuleSource};
    use crate::scope_config::ScopeFilter;
    use proptest::prelude::*;

    /// Strategy for generating random rule scopes
    fn arb_rule_scope() -> impl Strategy<Value = RuleScope> {
        prop_oneof![
            Just(RuleScope::Global),
            Just(RuleScope::Project),
            Just(RuleScope::Session),
        ]
    }

    /// Strategy for generating random rules
    fn arb_rule(scope: RuleScope) -> impl Strategy<Value = Rule> {
        (
            "[a-z]{1,10}",
            "[a-z]{1,10}",
        )
            .prop_map(move |(pattern, action)| {
                Rule::new(
                    scope,
                    pattern,
                    action,
                    RuleSource::Learned,
                )
            })
    }

    /// Strategy for generating random rule collections
    fn arb_rules_for_scope(scope: RuleScope) -> impl Strategy<Value = Vec<Rule>> {
        prop::collection::vec(arb_rule(scope), 0..20)
    }

    /// Property 8: Scope Configuration Isolation
    /// Test that rules in one scope don't affect other scopes
    /// For any set of rules in different scopes, filtering by scope should return
    /// only rules from that scope, and rules from one scope should not interfere
    /// with rules from another scope unless they have the same pattern.
    #[allow(unused_doc_comments)]
    proptest! {
        #[test]
        fn prop_scope_isolation_no_cross_scope_interference(
            global_rules in arb_rules_for_scope(RuleScope::Global),
            project_rules in arb_rules_for_scope(RuleScope::Project),
            session_rules in arb_rules_for_scope(RuleScope::Session),
        ) {
            // Combine all rules
            let mut all_rules = global_rules.clone();
            all_rules.extend(project_rules.clone());
            all_rules.extend(session_rules.clone());

            // Filter by each scope
            let filtered_global = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Global);
            let filtered_project = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Project);
            let filtered_session = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Session);

            // Verify that filtered rules match the original rules for each scope
            prop_assert_eq!(filtered_global.len(), global_rules.len());
            prop_assert_eq!(filtered_project.len(), project_rules.len());
            prop_assert_eq!(filtered_session.len(), session_rules.len());

            // Verify that all filtered rules have the correct scope
            for rule in &filtered_global {
                prop_assert_eq!(rule.scope, RuleScope::Global);
            }
            for rule in &filtered_project {
                prop_assert_eq!(rule.scope, RuleScope::Project);
            }
            for rule in &filtered_session {
                prop_assert_eq!(rule.scope, RuleScope::Session);
            }

            // Verify that there's no overlap between scopes
            for global_rule in &filtered_global {
                for project_rule in &filtered_project {
                    prop_assert_ne!(&global_rule.id, &project_rule.id);
                }
                for session_rule in &filtered_session {
                    prop_assert_ne!(&global_rule.id, &session_rule.id);
                }
            }
            for project_rule in &filtered_project {
                for session_rule in &filtered_session {
                    prop_assert_ne!(&project_rule.id, &session_rule.id);
                }
            }
        }

        #[test]
        fn prop_scope_isolation_filtering_is_idempotent(
            global_rules in arb_rules_for_scope(RuleScope::Global),
            project_rules in arb_rules_for_scope(RuleScope::Project),
        ) {
            // Combine rules
            let mut all_rules = global_rules.clone();
            all_rules.extend(project_rules.clone());

            // Filter twice
            let filtered_once = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Global);
            let filtered_twice = ScopeFilter::filter_by_scope(&filtered_once, RuleScope::Global);

            // Filtering twice should produce the same result as filtering once
            prop_assert_eq!(filtered_once.len(), filtered_twice.len());
            for (rule1, rule2) in filtered_once.iter().zip(filtered_twice.iter()) {
                prop_assert_eq!(&rule1.id, &rule2.id);
            }
        }

        #[test]
        fn prop_scope_isolation_union_covers_all_scopes(
            global_rules in arb_rules_for_scope(RuleScope::Global),
            project_rules in arb_rules_for_scope(RuleScope::Project),
            session_rules in arb_rules_for_scope(RuleScope::Session),
        ) {
            // Combine all rules
            let mut all_rules = global_rules.clone();
            all_rules.extend(project_rules.clone());
            all_rules.extend(session_rules.clone());

            // Filter by each scope
            let filtered_global = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Global);
            let filtered_project = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Project);
            let filtered_session = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Session);

            // Union of filtered rules should equal all rules
            let mut union = filtered_global.clone();
            union.extend(filtered_project.clone());
            union.extend(filtered_session.clone());

            prop_assert_eq!(union.len(), all_rules.len());
        }

        #[test]
        fn prop_scope_isolation_no_interference_different_patterns(
            global_rules in arb_rules_for_scope(RuleScope::Global),
            project_rules in arb_rules_for_scope(RuleScope::Project),
        ) {
            // Check interference between global and project rules
            let interference = ScopeFilter::check_scope_interference(&global_rules, &project_rules);

            // Interference should only occur if there are rules with the same pattern
            // but different actions
            let mut expected_interference = false;
            for global_rule in &global_rules {
                for project_rule in &project_rules {
                    if global_rule.pattern == project_rule.pattern && global_rule.action != project_rule.action {
                        expected_interference = true;
                        break;
                    }
                }
                if expected_interference {
                    break;
                }
            }

            prop_assert_eq!(interference, expected_interference);
        }

        #[test]
        fn prop_scope_isolation_filter_by_multiple_scopes(
            global_rules in arb_rules_for_scope(RuleScope::Global),
            project_rules in arb_rules_for_scope(RuleScope::Project),
            session_rules in arb_rules_for_scope(RuleScope::Session),
        ) {
            // Combine all rules
            let mut all_rules = global_rules.clone();
            all_rules.extend(project_rules.clone());
            all_rules.extend(session_rules.clone());

            // Filter by multiple scopes
            let filtered = ScopeFilter::filter_by_scopes(
                &all_rules,
                &[RuleScope::Project, RuleScope::Session],
            );

            // Should include project and session rules but not global
            let expected_count = project_rules.len() + session_rules.len();
            prop_assert_eq!(filtered.len(), expected_count);

            // All filtered rules should be from project or session scope
            for rule in &filtered {
                prop_assert!(
                    rule.scope == RuleScope::Project || rule.scope == RuleScope::Session,
                    "Rule scope should be Project or Session, got {:?}",
                    rule.scope
                );
            }
        }

        #[test]
        fn prop_scope_isolation_precedence_respects_scopes(
            global_rules in arb_rules_for_scope(RuleScope::Global),
            project_rules in arb_rules_for_scope(RuleScope::Project),
            session_rules in arb_rules_for_scope(RuleScope::Session),
        ) {
            // Combine all rules
            let mut all_rules = global_rules.clone();
            all_rules.extend(project_rules.clone());
            all_rules.extend(session_rules.clone());

            // Get rules with precedence for project scope
            let project_precedence = ScopeFilter::get_rules_with_precedence(&all_rules, RuleScope::Project);

            // Should include project and session rules but not global
            for rule in &project_precedence {
                prop_assert!(
                    rule.scope == RuleScope::Project || rule.scope == RuleScope::Session,
                    "Project precedence should only include Project and Session rules"
                );
            }

            // Get rules with precedence for global scope
            let global_precedence = ScopeFilter::get_rules_with_precedence(&all_rules, RuleScope::Global);

            // Should include only global rules
            for rule in &global_precedence {
                prop_assert_eq!(
                    rule.scope,
                    RuleScope::Global,
                    "Global precedence should only include Global rules"
                );
            }
        }

        #[test]
        fn prop_scope_isolation_empty_scopes(
            global_rules in arb_rules_for_scope(RuleScope::Global),
        ) {
            // Test with empty project and session rules
            let project_rules: Vec<Rule> = Vec::new();
            let session_rules: Vec<Rule> = Vec::new();

            let mut all_rules = global_rules.clone();
            all_rules.extend(project_rules.clone());
            all_rules.extend(session_rules.clone());

            // Filter by each scope
            let filtered_global = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Global);
            let filtered_project = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Project);
            let filtered_session = ScopeFilter::filter_by_scope(&all_rules, RuleScope::Session);

            // Verify counts
            prop_assert_eq!(filtered_global.len(), global_rules.len());
            prop_assert_eq!(filtered_project.len(), 0);
            prop_assert_eq!(filtered_session.len(), 0);
        }
    }
}
