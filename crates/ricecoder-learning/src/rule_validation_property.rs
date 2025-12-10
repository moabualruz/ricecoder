/// Property-based tests for rule validation accuracy
///
/// **Feature: ricecoder-learning, Property 4: Rule Validation Accuracy**
/// **Validates: Requirements 2.3**
///
/// Tests that the rule validator correctly accepts valid rules and rejects invalid rules
/// across a wide range of randomly generated scenarios.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use crate::{Rule, RuleScope, RuleSource, RuleValidator};

    /// Strategy for generating valid rule patterns
    fn valid_pattern_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9_\-\.]+"
            .prop_map(|s| s.to_string())
            .prop_filter("pattern must not be empty", |s| !s.is_empty())
    }

    /// Strategy for generating valid rule actions
    fn valid_action_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9_\-\.\s]+"
            .prop_map(|s| s.to_string())
            .prop_filter("action must not be empty", |s| !s.is_empty())
    }

    /// Strategy for generating valid confidence scores
    fn valid_confidence_strategy() -> impl Strategy<Value = f32> {
        0.0f32..=1.0f32
    }

    /// Strategy for generating valid success rates
    fn valid_success_rate_strategy() -> impl Strategy<Value = f32> {
        0.0f32..=1.0f32
    }

    /// Strategy for generating valid rules
    fn valid_rule_strategy() -> impl Strategy<Value = Rule> {
        (
            valid_pattern_strategy(),
            valid_action_strategy(),
            valid_confidence_strategy(),
            valid_success_rate_strategy(),
        )
            .prop_map(|(pattern, action, confidence, success_rate)| {
                let mut rule = Rule::new(
                    RuleScope::Global,
                    pattern,
                    action,
                    RuleSource::Learned,
                );
                rule.confidence = confidence;
                rule.success_rate = success_rate;
                rule
            })
    }

    /// Strategy for generating invalid confidence scores
    fn invalid_confidence_strategy() -> impl Strategy<Value = f32> {
        prop_oneof![
            Just(1.5f32),
            Just(-0.5f32),
            Just(2.0f32),
            Just(-1.0f32),
            Just(f32::NAN),
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
        ]
    }

    /// Strategy for generating invalid success rates
    fn invalid_success_rate_strategy() -> impl Strategy<Value = f32> {
        prop_oneof![
            Just(1.5f32),
            Just(-0.5f32),
            Just(2.0f32),
            Just(-1.0f32),
            Just(f32::NAN),
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
        ]
    }

    /// Property 4: Valid rules are accepted
    ///
    /// For any valid rule, the validator SHALL accept it without errors.
    #[allow(unused_doc_comments)]
    proptest! {
        #[test]
        fn prop_valid_rules_accepted(rule in valid_rule_strategy()) {
            let validator = RuleValidator::new();
            assert!(
                validator.validate(&rule).is_ok(),
                "Valid rule should be accepted: {:?}",
                rule
            );
        }

        /// Property 4: Invalid confidence scores are rejected
        ///
        /// For any rule with an invalid confidence score, the validator SHALL reject it.
        #[test]
        fn prop_invalid_confidence_rejected(confidence in invalid_confidence_strategy()) {
            let validator = RuleValidator::new();
            let mut rule = Rule::new(
                RuleScope::Global,
                "pattern".to_string(),
                "action".to_string(),
                RuleSource::Learned,
            );
            rule.confidence = confidence;

            assert!(
                validator.validate(&rule).is_err(),
                "Rule with invalid confidence {} should be rejected",
                confidence
            );
        }

        /// Property 4: Invalid success rates are rejected
        ///
        /// For any rule with an invalid success rate, the validator SHALL reject it.
        #[test]
        fn prop_invalid_success_rate_rejected(success_rate in invalid_success_rate_strategy()) {
            let validator = RuleValidator::new();
            let mut rule = Rule::new(
                RuleScope::Global,
                "pattern".to_string(),
                "action".to_string(),
                RuleSource::Learned,
            );
            rule.success_rate = success_rate;

            assert!(
                validator.validate(&rule).is_err(),
                "Rule with invalid success rate {} should be rejected",
                success_rate
            );
        }
    }

    /// Property 4: Empty patterns are rejected
    ///
    /// For any rule with an empty pattern, the validator SHALL reject it.
    #[test]
    fn prop_empty_pattern_rejected() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.pattern = String::new();

        assert!(
            validator.validate(&rule).is_err(),
            "Rule with empty pattern should be rejected"
        );
    }

    /// Property 4: Empty actions are rejected
    ///
    /// For any rule with an empty action, the validator SHALL reject it.
    #[test]
    fn prop_empty_action_rejected() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.action = String::new();

        assert!(
            validator.validate(&rule).is_err(),
            "Rule with empty action should be rejected"
        );
    }

    /// Property 4: Empty IDs are rejected
    ///
    /// For any rule with an empty ID, the validator SHALL reject it.
    #[test]
    fn prop_empty_id_rejected() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.id = String::new();

        assert!(
            validator.validate(&rule).is_err(),
            "Rule with empty ID should be rejected"
        );
    }

    /// Property 4: Zero version is rejected
    ///
    /// For any rule with version 0, the validator SHALL reject it.
    #[test]
    fn prop_zero_version_rejected() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.version = 0;

        assert!(
            validator.validate(&rule).is_err(),
            "Rule with version 0 should be rejected"
        );
    }

    proptest! {
        /// Property 4: Conflict detection works correctly
        ///
        /// For any two rules with the same pattern and scope, the validator SHALL detect a conflict.
        #[test]
        fn prop_conflict_detection(pattern in valid_pattern_strategy()) {
            let validator = RuleValidator::new();

            let rule1 = Rule::new(
                RuleScope::Global,
                pattern.clone(),
                "action1".to_string(),
                RuleSource::Learned,
            );

            let rule2 = Rule::new(
                RuleScope::Global,
                pattern,
                "action2".to_string(),
                RuleSource::Learned,
            );

            assert!(
                validator.check_conflicts(&rule2, &[rule1]).is_err(),
                "Conflicting rules should be detected"
            );
        }

        /// Property 4: No conflict for different scopes
        ///
        /// For any two rules with the same pattern but different scopes, the validator SHALL NOT detect a conflict.
        #[test]
        fn prop_no_conflict_different_scope(pattern in valid_pattern_strategy()) {
            let validator = RuleValidator::new();

            let rule1 = Rule::new(
                RuleScope::Global,
                pattern.clone(),
                "action1".to_string(),
                RuleSource::Learned,
            );

            let rule2 = Rule::new(
                RuleScope::Project,
                pattern,
                "action2".to_string(),
                RuleSource::Learned,
            );

            assert!(
                validator.check_conflicts(&rule2, &[rule1]).is_ok(),
                "Rules with different scopes should not conflict"
            );
        }

        /// Property 4: No conflict for different patterns
        ///
        /// For any two rules with different patterns but the same scope, the validator SHALL NOT detect a conflict.
        #[test]
        fn prop_no_conflict_different_pattern(
            pattern1 in valid_pattern_strategy(),
            pattern2 in valid_pattern_strategy(),
        ) {
            prop_assume!(pattern1 != pattern2);

            let validator = RuleValidator::new();

            let rule1 = Rule::new(
                RuleScope::Global,
                pattern1,
                "action1".to_string(),
                RuleSource::Learned,
            );

            let rule2 = Rule::new(
                RuleScope::Global,
                pattern2,
                "action2".to_string(),
                RuleSource::Learned,
            );

            assert!(
                validator.check_conflicts(&rule2, &[rule1]).is_ok(),
                "Rules with different patterns should not conflict"
            );
        }
    }

    /// Property 4: Validation report accuracy
    ///
    /// For any invalid rule, the validation report SHALL contain at least one error.
    #[test]
    fn prop_validation_report_accuracy() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.confidence = 1.5; // Invalid confidence

        let report = validator.validate_with_report(&rule);
        assert!(
            report.has_errors(),
            "Validation report should contain errors for invalid rule"
        );
    }

    proptest! {
        /// Property 4: Valid rules have no errors in report
        ///
        /// For any valid rule, the validation report SHALL contain no errors.
        #[test]
        fn prop_valid_rules_no_errors(rule in valid_rule_strategy()) {
            let validator = RuleValidator::new();
            let report = validator.validate_with_report(&rule);

            assert!(
                !report.has_errors(),
                "Valid rule should have no errors in report: {:?}",
                rule
            );
        }
    }

    /// Property 4: JSON action validation
    ///
    /// For any rule with a valid JSON action, the validator SHALL accept it.
    #[test]
    fn prop_json_action_accepted() {
        let validator = RuleValidator::new();
        let rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            r#"{"key": "value", "nested": {"inner": 42}}"#.to_string(),
            RuleSource::Learned,
        );

        assert!(
            validator.validate(&rule).is_ok(),
            "Valid JSON action should be accepted"
        );
    }

    /// Property 4: Invalid JSON action rejected
    ///
    /// For any rule with an invalid JSON action, the validator SHALL reject it.
    #[test]
    fn prop_invalid_json_action_rejected() {
        let validator = RuleValidator::new();
        let rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            r#"{"key": invalid, "nested": {inner: 42}}"#.to_string(),
            RuleSource::Learned,
        );

        assert!(
            validator.validate(&rule).is_err(),
            "Invalid JSON action should be rejected"
        );
    }

    /// Property 4: Metadata must be object
    ///
    /// For any rule with non-object metadata, the validator SHALL reject it.
    #[test]
    fn prop_non_object_metadata_rejected() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.metadata = serde_json::json!([1, 2, 3]); // Array instead of object

        assert!(
            validator.validate(&rule).is_err(),
            "Non-object metadata should be rejected"
        );
    }

    proptest! {
        /// Property 4: Multiple rules validation
        ///
        /// For any set of valid rules, the validator SHALL accept all of them.
        #[test]
        fn prop_multiple_valid_rules_accepted(rules in prop::collection::vec(valid_rule_strategy(), 1..10)) {
            let validator = RuleValidator::new();

            for rule in rules {
                assert!(
                    validator.validate(&rule).is_ok(),
                    "All valid rules should be accepted"
                );
            }
        }

        /// Property 4: Boundary confidence values
        ///
        /// For any rule with confidence exactly 0.0 or 1.0, the validator SHALL accept it.
        #[test]
        fn prop_boundary_confidence_accepted(confidence in prop_oneof![Just(0.0f32), Just(1.0f32)]) {
            let validator = RuleValidator::new();
            let mut rule = Rule::new(
                RuleScope::Global,
                "pattern".to_string(),
                "action".to_string(),
                RuleSource::Learned,
            );
            rule.confidence = confidence;

            assert!(
                validator.validate(&rule).is_ok(),
                "Boundary confidence values should be accepted"
            );
        }

        /// Property 4: Boundary success rate values
        ///
        /// For any rule with success rate exactly 0.0 or 1.0, the validator SHALL accept it.
        #[test]
        fn prop_boundary_success_rate_accepted(success_rate in prop_oneof![Just(0.0f32), Just(1.0f32)]) {
            let validator = RuleValidator::new();
            let mut rule = Rule::new(
                RuleScope::Global,
                "pattern".to_string(),
                "action".to_string(),
                RuleSource::Learned,
            );
            rule.success_rate = success_rate;

            assert!(
                validator.validate(&rule).is_ok(),
                "Boundary success rate values should be accepted"
            );
        }
    }
}
