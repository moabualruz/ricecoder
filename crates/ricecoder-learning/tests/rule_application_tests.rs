/// Integration tests for rule application functionality
use ricecoder_learning::{
    GenerationContext, LearningManager, Rule, RuleApplicationEngine, RuleScope, RuleSource,
};

#[test]
fn test_rule_matching_logic_simple_pattern() {
    let rule = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    assert!(RuleApplicationEngine::matches_pattern(&rule, &context));
}

#[test]
fn test_rule_matching_logic_language_pattern() {
    let rule = Rule::new(
        RuleScope::Session,
        "rust".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    assert!(RuleApplicationEngine::matches_pattern(&rule, &context));
}

#[test]
fn test_rule_matching_logic_no_match() {
    let rule = Rule::new(
        RuleScope::Session,
        "class".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    assert!(!RuleApplicationEngine::matches_pattern(&rule, &context));
}

#[test]
fn test_rule_matching_logic_json_pattern() {
    let rule = Rule::new(
        RuleScope::Session,
        r#"{"generation_type": "function", "language": "rust"}"#.to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    assert!(RuleApplicationEngine::matches_pattern(&rule, &context));
}

#[test]
fn test_rule_matching_logic_json_pattern_no_match() {
    let rule = Rule::new(
        RuleScope::Session,
        r#"{"generation_type": "class", "language": "python"}"#.to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    assert!(!RuleApplicationEngine::matches_pattern(&rule, &context));
}

#[test]
fn test_rule_application_to_generation_context_single_rule() {
    let rule = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = RuleApplicationEngine::apply_rule(&rule, &context);
    assert!(result.matched);
    assert_eq!(result.action, Some("add_documentation".to_string()));
    assert_eq!(result.rule.id, rule.id);
}

#[test]
fn test_rule_application_to_generation_context_multiple_rules() {
    let rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "rust".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    let rule3 = Rule::new(
        RuleScope::Session,
        "class".to_string(),
        "add_inheritance".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let results = RuleApplicationEngine::apply_rules(&[rule1, rule2, rule3], &context);
    assert_eq!(results.len(), 3);
    assert!(results[0].matched); // function rule matches
    assert!(results[1].matched); // rust rule matches
    assert!(!results[2].matched); // class rule doesn't match
}

#[test]
fn test_rule_chaining() {
    let rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let results = RuleApplicationEngine::chain_rules(&[rule1, rule2], &context);
    assert!(results.is_ok());
    let results = results.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results[0].matched);
    assert!(results[1].matched);
}

#[test]
fn test_rule_composition() {
    let rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = RuleApplicationEngine::compose_rules(&[rule1, rule2], &context);
    assert!(result.is_ok());
    let composed = result.unwrap();
    assert!(composed.is_some());
    let composed_str = composed.unwrap();
    assert!(composed_str.contains("add_documentation"));
    assert!(composed_str.contains("add_error_handling"));
}

#[tokio::test]
async fn test_learning_manager_apply_rule_to_context() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = manager.apply_rule_to_context(&rule, &context).await;
    assert!(result.matched);
    assert_eq!(result.action, Some("add_documentation".to_string()));
}

#[tokio::test]
async fn test_learning_manager_apply_rules_to_context() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "rust".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let results = manager.apply_rules_to_context(&[rule1, rule2], &context).await;
    assert_eq!(results.len(), 2);
    assert!(results[0].matched);
    assert!(results[1].matched);
}

#[tokio::test]
async fn test_learning_manager_apply_rules_with_precedence() {
    let manager = LearningManager::new(RuleScope::Session);

    let mut rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );
    rule1.confidence = 0.5;

    let mut rule2 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );
    rule2.confidence = 0.9;

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = manager.apply_rules_with_precedence(&[rule1, rule2], &context).await;
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.action, Some("add_error_handling".to_string()));
    assert_eq!(result.confidence, 0.9);
}

#[tokio::test]
async fn test_learning_manager_chain_rules() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let results = manager.chain_rules(&[rule1, rule2], &context).await;
    assert!(results.is_ok());
    let results = results.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_learning_manager_compose_rules() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = manager.compose_rules(&[rule1, rule2], &context).await;
    assert!(result.is_ok());
    let composed = result.unwrap();
    assert!(composed.is_some());
}

#[tokio::test]
async fn test_learning_manager_validate_rule_application() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = manager.validate_rule_application(&rule, &context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_learning_manager_get_matching_rules() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "class".to_string(),
        "add_inheritance".to_string(),
        RuleSource::Learned,
    );

    manager.store_rule(rule1).await.unwrap();
    manager.store_rule(rule2).await.unwrap();

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let matching = manager.get_matching_rules(&context).await.unwrap();
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].action, "add_documentation");
}

#[tokio::test]
async fn test_learning_manager_get_matching_rules_sorted() {
    let manager = LearningManager::new(RuleScope::Session);

    let mut rule1 = Rule::new(
        RuleScope::Session,
        "function_doc".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );
    rule1.confidence = 0.5;

    let mut rule2 = Rule::new(
        RuleScope::Session,
        "function_error".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );
    rule2.confidence = 0.9;

    manager.store_rule(rule1).await.unwrap();
    manager.store_rule(rule2).await.unwrap();

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let matching = manager.get_matching_rules_sorted(&context).await.unwrap();
    assert_eq!(matching.len(), 2);
    assert_eq!(matching[0].confidence, 0.9);
    assert_eq!(matching[1].confidence, 0.5);
}

#[tokio::test]
async fn test_learning_manager_apply_learned_rules_to_generation() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    manager.store_rule(rule).await.unwrap();

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = manager.apply_learned_rules_to_generation(&context).await;
    assert!(result.is_ok());
    let action = result.unwrap();
    assert_eq!(action, Some("add_documentation".to_string()));
}

#[tokio::test]
async fn test_learning_manager_apply_learned_rules_get_all() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule1 = Rule::new(
        RuleScope::Session,
        "function_doc".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let rule2 = Rule::new(
        RuleScope::Session,
        "function_error".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );

    manager.store_rule(rule1).await.unwrap();
    manager.store_rule(rule2).await.unwrap();

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let actions = manager.apply_learned_rules_get_all(&context).await;
    assert!(actions.is_ok());
    let actions = actions.unwrap();
    assert_eq!(actions.len(), 2);
    assert!(actions.contains(&"add_documentation".to_string()));
    assert!(actions.contains(&"add_error_handling".to_string()));
}

#[tokio::test]
async fn test_learning_manager_apply_learned_rules_with_context() {
    let manager = LearningManager::new(RuleScope::Session);

    let rule = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    manager.store_rule(rule).await.unwrap();

    let result = manager
        .apply_learned_rules_with_context(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        )
        .await;

    assert!(result.is_ok());
    let action = result.unwrap();
    assert_eq!(action, Some("add_documentation".to_string()));
}

#[test]
fn test_rule_application_no_matching_rules() {
    let rule = Rule::new(
        RuleScope::Session,
        "class".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = RuleApplicationEngine::apply_rules_with_precedence(&[rule], &context);
    assert!(result.is_none());
}

#[test]
fn test_rule_application_empty_rules() {
    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = RuleApplicationEngine::apply_rules_with_precedence(&[], &context);
    assert!(result.is_none());
}

#[test]
fn test_rule_application_multiple_matching_rules_precedence() {
    let mut rule1 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_documentation".to_string(),
        RuleSource::Learned,
    );
    rule1.confidence = 0.6;

    let mut rule2 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_error_handling".to_string(),
        RuleSource::Learned,
    );
    rule2.confidence = 0.8;

    let mut rule3 = Rule::new(
        RuleScope::Session,
        "function".to_string(),
        "add_logging".to_string(),
        RuleSource::Learned,
    );
    rule3.confidence = 0.7;

    let context = GenerationContext::new(
        "function".to_string(),
        "rust".to_string(),
        "fn test() {}".to_string(),
    );

    let result = RuleApplicationEngine::apply_rules_with_precedence(&[rule1, rule2, rule3], &context);
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.action, Some("add_error_handling".to_string()));
    assert_eq!(result.confidence, 0.8);
}
