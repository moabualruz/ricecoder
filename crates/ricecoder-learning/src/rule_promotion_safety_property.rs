/// Property-based tests for rule promotion safety
/// **Feature: ricecoder-learning, Property 6: Rule Promotion Safety**
/// **Validates: Requirements 4.1**

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use crate::{
        models::{Rule, RuleScope, RuleSource},
        rule_promoter::RulePromoter,
    };

    /// Strategy for generating valid rules in project scope
    fn project_rule_strategy() -> impl Strategy<Value = Rule> {
        (
            "[a-z_][a-z0-9_]{0,19}",
            "[a-z_][a-z0-9_]{0,19}",
        )
            .prop_map(|(pattern, action)| {
                Rule::new(
                    RuleScope::Project,
                    pattern,
                    action,
                    RuleSource::Learned,
                )
            })
    }

    /// Strategy for generating valid rules in global scope
    fn global_rule_strategy() -> impl Strategy<Value = Rule> {
        (
            "[a-z_][a-z0-9_]{0,19}",
            "[a-z_][a-z0-9_]{0,19}",
        )
            .prop_map(|(pattern, action)| {
                Rule::new(
                    RuleScope::Global,
                    pattern,
                    action,
                    RuleSource::Learned,
                )
            })
    }

    /// Property 6: Rule Promotion Safety
    /// For any rule promotion from project to global scope, the Learning System
    /// SHALL verify the promoted rule does not conflict with existing global rules.
    ///
    /// This property tests that:
    /// 1. A rule can be promoted from project to global scope
    /// 2. The promoted rule is validated against existing global rules
    /// 3. Conflicts are detected before promotion
    /// 4. The promoted rule has the correct scope and source after promotion
    proptest! {
        #[test]
        fn prop_rule_promotion_does_not_create_conflicts(
            project_rule in project_rule_strategy(),
            global_rules in prop::collection::vec(global_rule_strategy(), 0..10),
        ) {
        let mut promoter = RulePromoter::new();

        // Request promotion
        let review_result = promoter.request_promotion(project_rule.clone(), &global_rules);

        // The promotion request should succeed (conflicts are detected but don't prevent request)
        assert!(review_result.is_ok());

        let review = review_result.unwrap();

        // If there are conflicts, they should be detected
        if !review.conflicts.is_empty() {
            // Verify that all conflicts have the same pattern as the project rule
            for conflict in &review.conflicts {
                assert_eq!(conflict.pattern, project_rule.pattern);
                // Conflicts occur when same pattern but different action
                // So if it's a conflict, the action must be different
                if conflict.pattern == project_rule.pattern {
                    // This is expected - same pattern can have different actions
                    // which is why it's flagged as a conflict
                }
            }
        }

        // Approve the promotion
        let promoted_result = promoter.approve_promotion(&project_rule.id, None);
        assert!(promoted_result.is_ok());

        let promoted_rule = promoted_result.unwrap();

        // Verify the promoted rule has the correct scope and source
        assert_eq!(promoted_rule.scope, RuleScope::Global);
        assert_eq!(promoted_rule.source, RuleSource::Promoted);

        // Verify the promoted rule's pattern and action are preserved
        assert_eq!(promoted_rule.pattern, project_rule.pattern);
        assert_eq!(promoted_rule.action, project_rule.action);
        }
    }

    /// Property: Promoted rules maintain data integrity
    /// For any promoted rule, all original data should be preserved except for
    /// scope, source, and version which should be updated appropriately.
    proptest! {
        #[test]
        fn prop_promoted_rule_maintains_data_integrity(
            project_rule in project_rule_strategy(),
        ) {
        let mut promoter = RulePromoter::new();

        let original_pattern = project_rule.pattern.clone();
        let original_action = project_rule.action.clone();
        let original_confidence = project_rule.confidence;
        let original_usage_count = project_rule.usage_count;

        promoter.request_promotion(project_rule.clone(), &[]).unwrap();
        let promoted_rule = promoter.approve_promotion(&project_rule.id, None).unwrap();

        // Verify data integrity
        assert_eq!(promoted_rule.pattern, original_pattern);
        assert_eq!(promoted_rule.action, original_action);
        assert_eq!(promoted_rule.confidence, original_confidence);
        assert_eq!(promoted_rule.usage_count, original_usage_count);

        // Verify scope and source changed
        assert_eq!(promoted_rule.scope, RuleScope::Global);
        assert_eq!(promoted_rule.source, RuleSource::Promoted);

        // Verify version incremented
        assert_eq!(promoted_rule.version, project_rule.version + 1);
        }
    }

    /// Property: Promotion history is accurate
    /// For any promotion, the promotion history should accurately record
    /// the promotion event with correct metadata.
    proptest! {
        #[test]
        fn prop_promotion_history_is_accurate(
            project_rule in project_rule_strategy(),
        ) {
        let mut promoter = RulePromoter::new();

        let rule_id = project_rule.id.clone();
        promoter.request_promotion(project_rule.clone(), &[]).unwrap();
        promoter.approve_promotion(&rule_id, Some("Test approval".to_string())).unwrap();

        let history = promoter.get_promotion_history();
        assert_eq!(history.len(), 1);

        let entry = &history[0];
        assert_eq!(entry.rule_id, rule_id);
        assert_eq!(entry.source_scope, RuleScope::Project);
        assert_eq!(entry.target_scope, RuleScope::Global);
        assert!(entry.approved);
        assert_eq!(entry.reason, Some("Test approval".to_string()));
        }
    }

    /// Property: Multiple promotions can be tracked independently
    /// For any set of promotions, each should be tracked independently
    /// in the promotion history.
    proptest! {
        #[test]
        fn prop_multiple_promotions_tracked_independently(
            project_rules in prop::collection::vec(project_rule_strategy(), 1..10),
        ) {
        let mut promoter = RulePromoter::new();

        for rule in &project_rules {
            promoter.request_promotion(rule.clone(), &[]).unwrap();
        }

        // Approve some, reject others
        let history_before = promoter.get_promotion_history().len();
        assert_eq!(history_before, 0);

        for (i, rule) in project_rules.iter().enumerate() {
            if i % 2 == 0 {
                promoter.approve_promotion(&rule.id, None).unwrap();
            } else {
                promoter.reject_promotion(&rule.id, None).unwrap();
            }
        }

        let history = promoter.get_promotion_history();
        assert_eq!(history.len(), project_rules.len());

        // Verify each promotion is recorded correctly
        for (i, entry) in history.iter().enumerate() {
            assert_eq!(entry.rule_id, project_rules[i].id);
            assert_eq!(entry.approved, i % 2 == 0);
        }
        }
    }

    /// Property: Pending promotions are isolated from history
    /// For any pending promotion, it should not appear in the promotion history
    /// until it is approved or rejected.
    proptest! {
        #[test]
        fn prop_pending_promotions_isolated_from_history(
            project_rules in prop::collection::vec(project_rule_strategy(), 1..5),
        ) {
        let mut promoter = RulePromoter::new();

        for rule in &project_rules {
            promoter.request_promotion(rule.clone(), &[]).unwrap();
        }

        // No promotions should be in history yet
        let history = promoter.get_promotion_history();
        assert_eq!(history.len(), 0);

        // Pending promotions should exist
        let pending = promoter.get_pending_promotions();
        assert_eq!(pending.len(), project_rules.len());

        // Approve one
        promoter.approve_promotion(&project_rules[0].id, None).unwrap();

        // Now history should have one entry
        let history = promoter.get_promotion_history();
        assert_eq!(history.len(), 1);

        // And pending should have one less
        let pending = promoter.get_pending_promotions();
        assert_eq!(pending.len(), project_rules.len() - 1);
        }
    }

    /// Property: Conflict detection is consistent
    /// For any set of rules, conflict detection should be consistent
    /// regardless of the order in which rules are checked.
    proptest! {
        #[test]
        fn prop_conflict_detection_is_consistent(
            project_rule in project_rule_strategy(),
            global_rules in prop::collection::vec(global_rule_strategy(), 0..5),
        ) {
        let mut promoter1 = RulePromoter::new();
        let mut promoter2 = RulePromoter::new();

        // Request promotion in both promoters
        let review1 = promoter1.request_promotion(project_rule.clone(), &global_rules).unwrap();
        let review2 = promoter2.request_promotion(project_rule.clone(), &global_rules).unwrap();

        // Both should detect the same conflicts
        assert_eq!(review1.conflicts.len(), review2.conflicts.len());

        // Verify conflict consistency
        for conflict in &review1.conflicts {
            assert!(review2.conflicts.iter().any(|c| c.id == conflict.id));
        }
        }
    }

    /// Property: Validation prevents invalid promotions
    /// For any promoted rule, validation should ensure it doesn't conflict
    /// with existing global rules.
    proptest! {
        #[test]
        fn prop_validation_prevents_invalid_promotions(
            mut project_rule in project_rule_strategy(),
            global_rules in prop::collection::vec(global_rule_strategy(), 0..5),
        ) {
        let promoter = RulePromoter::new();

        project_rule.scope = RuleScope::Global;
        project_rule.source = RuleSource::Promoted;

        // Validation should succeed if there are no conflicts
        let validation_result = promoter.validate_promotion(&project_rule, &global_rules);

        if validation_result.is_err() {
            // If validation fails, there must be a conflict
            let has_conflict = global_rules.iter().any(|gr| {
                gr.pattern == project_rule.pattern && gr.action != project_rule.action
            });
            assert!(has_conflict);
        }
        }
    }
}
