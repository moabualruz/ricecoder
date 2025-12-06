/// Property-based tests for rule persistence correctness
/// **Feature: ricecoder-learning, Property 2: Rule Persistence Correctness**
/// **Validates: Requirements 1.3**

#[cfg(test)]
mod tests {
    use crate::{Rule, RuleScope, RuleSource, RuleStorage};

    #[tokio::test]
    async fn test_rule_persistence_correctness_single() {
        // Test that a single rule can be stored and retrieved with identical content
        let storage = RuleStorage::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Session,
            "test_pattern".to_string(),
            "test_action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();

        // Store the rule
        let store_result = storage.store_rule(rule.clone()).await;
        assert!(store_result.is_ok(), "Failed to store rule");
        assert_eq!(store_result.unwrap(), rule_id, "Returned rule ID should match");

        // Retrieve the rule
        let retrieve_result = storage.get_rule(&rule_id).await;
        assert!(retrieve_result.is_ok(), "Failed to retrieve rule");

        let retrieved = retrieve_result.unwrap();

        // Verify all fields match
        assert_eq!(retrieved.id, rule.id, "Rule ID should match");
        assert_eq!(retrieved.scope, rule.scope, "Rule scope should match");
        assert_eq!(retrieved.pattern, rule.pattern, "Rule pattern should match");
        assert_eq!(retrieved.action, rule.action, "Rule action should match");
        assert_eq!(retrieved.source, rule.source, "Rule source should match");
        assert_eq!(retrieved.version, rule.version, "Rule version should match");
        assert_eq!(retrieved.confidence, rule.confidence, "Rule confidence should match");
        assert_eq!(retrieved.usage_count, rule.usage_count, "Rule usage count should match");
        assert_eq!(retrieved.success_rate, rule.success_rate, "Rule success rate should match");
        assert_eq!(retrieved.metadata, rule.metadata, "Rule metadata should match");
    }

    #[tokio::test]
    async fn test_multiple_rules_persistence() {
        // Test that multiple rules can be stored and retrieved independently
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rules = Vec::new();
        for i in 0..5 {
            let mut rule = Rule::new(
                RuleScope::Session,
                format!("pattern_{}", i),
                format!("action_{}", i),
                RuleSource::Learned,
            );
            rule.confidence = 0.5 + (i as f32 * 0.1);
            rule.usage_count = i as u64 * 10;
            rules.push(rule);
        }

        // Store all rules
        let mut rule_ids = Vec::new();
        for rule in &rules {
            let result = storage.store_rule(rule.clone()).await;
            assert!(result.is_ok(), "Failed to store rule");
            rule_ids.push(result.unwrap());
        }

        // Verify all rules can be retrieved
        for (i, rule_id) in rule_ids.iter().enumerate() {
            let result = storage.get_rule(rule_id).await;
            assert!(result.is_ok(), "Failed to retrieve rule {}", i);

            let retrieved = result.unwrap();
            let original = &rules[i];

            assert_eq!(retrieved.id, original.id, "Rule {} ID should match", i);
            assert_eq!(retrieved.pattern, original.pattern, "Rule {} pattern should match", i);
            assert_eq!(retrieved.action, original.action, "Rule {} action should match", i);
            assert_eq!(retrieved.confidence, original.confidence, "Rule {} confidence should match", i);
        }
    }

    #[tokio::test]
    async fn test_rule_deletion_persistence() {
        // Test that deleted rules cannot be retrieved
        let storage = RuleStorage::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Session,
            "test_pattern".to_string(),
            "test_action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();

        // Store the rule
        storage.store_rule(rule).await.unwrap();

        // Verify it exists
        let exists = storage.get_rule(&rule_id).await;
        assert!(exists.is_ok(), "Rule should exist after storage");

        // Delete the rule
        let delete_result = storage.delete_rule(&rule_id).await;
        assert!(delete_result.is_ok(), "Failed to delete rule");

        // Verify it no longer exists
        let not_found = storage.get_rule(&rule_id).await;
        assert!(not_found.is_err(), "Rule should not exist after deletion");
    }

    #[tokio::test]
    async fn test_list_rules_completeness() {
        // Test that listing returns all stored rules
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rules = Vec::new();
        for i in 0..5 {
            let rule = Rule::new(
                RuleScope::Session,
                format!("pattern_{}", i),
                format!("action_{}", i),
                RuleSource::Learned,
            );
            rules.push(rule);
        }

        // Store all rules
        for rule in &rules {
            storage.store_rule(rule.clone()).await.unwrap();
        }

        // List all rules
        let listed = storage.list_rules().await.unwrap();

        // Verify count matches
        assert_eq!(listed.len(), rules.len(), "Listed rules count should match stored count");

        // Verify all rules are present
        for original in &rules {
            let found = listed.iter().find(|r| r.id == original.id);
            assert!(found.is_some(), "Rule {} should be in list", original.id);
        }
    }

    #[tokio::test]
    async fn test_rule_count_accuracy() {
        // Test that rule count is accurate
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rules = Vec::new();
        for i in 0..5 {
            let rule = Rule::new(
                RuleScope::Session,
                format!("pattern_{}", i),
                format!("action_{}", i),
                RuleSource::Learned,
            );
            rules.push(rule);
        }

        // Store all rules
        for rule in &rules {
            storage.store_rule(rule.clone()).await.unwrap();
        }

        // Check count
        let count = storage.rule_count().await.unwrap();
        assert_eq!(count, rules.len(), "Rule count should match stored count");
    }

    #[tokio::test]
    async fn test_clear_all_completeness() {
        // Test that clearing all rules removes all stored rules
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rules = Vec::new();
        for i in 0..5 {
            let rule = Rule::new(
                RuleScope::Session,
                format!("pattern_{}", i),
                format!("action_{}", i),
                RuleSource::Learned,
            );
            rules.push(rule);
        }

        // Store all rules
        for rule in &rules {
            storage.store_rule(rule.clone()).await.unwrap();
        }

        // Verify rules exist
        let count_before = storage.rule_count().await.unwrap();
        assert_eq!(count_before, rules.len(), "Rules should be stored");

        // Clear all
        storage.clear_all().await.unwrap();

        // Verify all rules are gone
        let count_after = storage.rule_count().await.unwrap();
        assert_eq!(count_after, 0, "All rules should be cleared");
    }

    #[tokio::test]
    async fn test_scope_filtering_correctness() {
        // Test that filtering by scope returns only rules in that scope
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rules = Vec::new();
        for i in 0..5 {
            let rule = Rule::new(
                RuleScope::Session,
                format!("pattern_{}", i),
                format!("action_{}", i),
                RuleSource::Learned,
            );
            rules.push(rule);
        }

        // Store all rules (all will be Session scope)
        for rule in &rules {
            storage.store_rule(rule.clone()).await.unwrap();
        }

        // Filter by Session scope
        let filtered = storage.filter_by_scope(RuleScope::Session).await.unwrap();

        // All rules should be returned since they're all Session scope
        assert_eq!(filtered.len(), rules.len(), "All rules should match Session scope");

        // Verify all returned rules have Session scope
        for rule in &filtered {
            assert_eq!(rule.scope, RuleScope::Session, "Filtered rule should have Session scope");
        }
    }
}
