/// Rule application engine for guiding code generation
use crate::error::{LearningError, Result};
use crate::models::Rule;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Context for code generation that rules can match against
#[derive(Debug, Clone)]
pub struct GenerationContext {
    /// Type of generation (e.g., "function", "class", "module")
    pub generation_type: String,
    /// Language being generated
    pub language: String,
    /// Current code or prompt
    pub input: String,
    /// Additional context metadata
    pub metadata: HashMap<String, Value>,
}

impl GenerationContext {
    /// Create a new generation context
    pub fn new(generation_type: String, language: String, input: String) -> Self {
        Self {
            generation_type,
            language,
            input,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Convert context to JSON for pattern matching
    pub fn to_json(&self) -> Value {
        json!({
            "generation_type": self.generation_type,
            "language": self.language,
            "input_length": self.input.len(),
            "metadata": self.metadata,
        })
    }
}

/// Result of applying a rule to a generation context
#[derive(Debug, Clone)]
pub struct RuleApplicationResult {
    /// The rule that was applied
    pub rule: Rule,
    /// Whether the rule matched the context
    pub matched: bool,
    /// The action to apply (if matched)
    pub action: Option<String>,
    /// Confidence in the match
    pub confidence: f32,
    /// Additional details about the application
    pub details: HashMap<String, Value>,
}

impl RuleApplicationResult {
    /// Create a new rule application result
    pub fn new(rule: Rule, matched: bool) -> Self {
        Self {
            confidence: rule.confidence,
            action: if matched { Some(rule.action.clone()) } else { None },
            rule,
            matched,
            details: HashMap::new(),
        }
    }

    /// Add a detail to the result
    pub fn with_detail(mut self, key: String, value: Value) -> Self {
        self.details.insert(key, value);
        self
    }
}

/// Rule application engine for matching and applying rules
pub struct RuleApplicationEngine;

impl RuleApplicationEngine {
    /// Check if a rule pattern matches a generation context
    pub fn matches_pattern(rule: &Rule, context: &GenerationContext) -> bool {
        // Parse the pattern as a simple JSON pattern
        if let Ok(pattern_value) = serde_json::from_str::<Value>(&rule.pattern) {
            Self::pattern_matches(&pattern_value, context)
        } else {
            // Fallback to simple string matching
            rule.pattern.contains(&context.generation_type)
                || rule.pattern.contains(&context.language)
        }
    }

    /// Check if a pattern value matches the generation context
    fn pattern_matches(pattern: &Value, context: &GenerationContext) -> bool {
        match pattern {
            Value::Object(obj) => {
                // Check each field in the pattern
                for (key, value) in obj {
                    match key.as_str() {
                        "generation_type" => {
                            if let Value::String(expected) = value {
                                if context.generation_type != *expected {
                                    return false;
                                }
                            }
                        }
                        "language" => {
                            if let Value::String(expected) = value {
                                if context.language != *expected {
                                    return false;
                                }
                            }
                        }
                        "metadata" => {
                            if let Value::Object(expected_meta) = value {
                                for (meta_key, meta_value) in expected_meta {
                                    if let Some(actual_value) = context.metadata.get(meta_key) {
                                        if actual_value != meta_value {
                                            return false;
                                        }
                                    } else {
                                        return false;
                                    }
                                }
                            }
                        }
                        _ => {
                            // Unknown pattern field, skip
                        }
                    }
                }
                true
            }
            Value::String(pattern_str) => {
                // Simple string pattern matching
                pattern_str.contains(&context.generation_type)
                    || pattern_str.contains(&context.language)
            }
            _ => false,
        }
    }

    /// Apply a single rule to a generation context
    pub fn apply_rule(rule: &Rule, context: &GenerationContext) -> RuleApplicationResult {
        let matched = Self::matches_pattern(rule, context);
        let mut result = RuleApplicationResult::new(rule.clone(), matched);

        if matched {
            result = result.with_detail(
                "applied_at".to_string(),
                json!(chrono::Utc::now().to_rfc3339()),
            );
        }

        result
    }

    /// Apply multiple rules to a generation context
    pub fn apply_rules(
        rules: &[Rule],
        context: &GenerationContext,
    ) -> Vec<RuleApplicationResult> {
        rules
            .iter()
            .map(|rule| Self::apply_rule(rule, context))
            .collect()
    }

    /// Apply rules and get the highest confidence matched rule
    pub fn apply_rules_with_precedence(
        rules: &[Rule],
        context: &GenerationContext,
    ) -> Option<RuleApplicationResult> {
        let results = Self::apply_rules(rules, context);
        results
            .into_iter()
            .filter(|r| r.matched)
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Chain multiple rules together
    pub fn chain_rules(
        rules: &[Rule],
        context: &GenerationContext,
    ) -> Result<Vec<RuleApplicationResult>> {
        let mut results = Vec::new();
        let mut current_context = context.clone();

        for rule in rules {
            let result = Self::apply_rule(rule, &current_context);

            if result.matched {
                // Update context with the action for the next rule
                if let Some(action) = &result.action {
                    current_context = current_context.with_metadata(
                        "last_action".to_string(),
                        json!(action),
                    );
                }
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Compose multiple rules into a single action
    pub fn compose_rules(
        rules: &[Rule],
        context: &GenerationContext,
    ) -> Result<Option<String>> {
        let results = Self::apply_rules(rules, context);
        let matched_actions: Vec<String> = results
            .iter()
            .filter(|r| r.matched)
            .filter_map(|r| r.action.clone())
            .collect();

        if matched_actions.is_empty() {
            Ok(None)
        } else {
            // Compose actions by joining them
            Ok(Some(matched_actions.join("\n")))
        }
    }

    /// Validate that a rule can be applied to a context
    pub fn validate_rule_application(rule: &Rule, context: &GenerationContext) -> Result<()> {
        // Validate that the pattern is valid JSON or a simple string
        if let Err(_) = serde_json::from_str::<Value>(&rule.pattern) {
            // If not JSON, check if it's a simple string pattern
            if rule.pattern.is_empty() {
                return Err(LearningError::RuleApplicationFailed(
                    "Rule pattern cannot be empty".to_string(),
                ));
            }
        }

        // Validate that the action is not empty
        if rule.action.is_empty() {
            return Err(LearningError::RuleApplicationFailed(
                "Rule action cannot be empty".to_string(),
            ));
        }

        // Validate that the context is valid
        if context.generation_type.is_empty() {
            return Err(LearningError::RuleApplicationFailed(
                "Generation context type cannot be empty".to_string(),
            ));
        }

        if context.language.is_empty() {
            return Err(LearningError::RuleApplicationFailed(
                "Generation context language cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Get all matching rules for a context
    pub fn get_matching_rules(rules: &[Rule], context: &GenerationContext) -> Vec<Rule> {
        rules
            .iter()
            .filter(|rule| Self::matches_pattern(rule, context))
            .cloned()
            .collect()
    }

    /// Get matching rules sorted by confidence
    pub fn get_matching_rules_sorted(rules: &[Rule], context: &GenerationContext) -> Vec<Rule> {
        let mut matching = Self::get_matching_rules(rules, context);
        matching.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matching
    }

    /// Get matching rules sorted by usage
    pub fn get_matching_rules_by_usage(rules: &[Rule], context: &GenerationContext) -> Vec<Rule> {
        let mut matching = Self::get_matching_rules(rules, context);
        matching.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        matching
    }

    /// Get matching rules sorted by success rate
    pub fn get_matching_rules_by_success(rules: &[Rule], context: &GenerationContext) -> Vec<Rule> {
        let mut matching = Self::get_matching_rules(rules, context);
        matching.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_context_creation() {
        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        assert_eq!(context.generation_type, "function");
        assert_eq!(context.language, "rust");
        assert_eq!(context.input, "fn test() {}");
    }

    #[test]
    fn test_generation_context_with_metadata() {
        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        )
        .with_metadata("style".to_string(), json!("async"));

        assert_eq!(context.metadata.get("style").unwrap(), &json!("async"));
    }

    #[test]
    fn test_rule_application_result() {
        let rule = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let result = RuleApplicationResult::new(rule.clone(), true);
        assert!(result.matched);
        assert_eq!(result.action, Some("add_documentation".to_string()));
    }

    #[test]
    fn test_simple_pattern_matching() {
        let rule = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        assert!(RuleApplicationEngine::matches_pattern(&rule, &context));
    }

    #[test]
    fn test_pattern_not_matching() {
        let rule = Rule::new(
            crate::models::RuleScope::Session,
            "class".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        assert!(!RuleApplicationEngine::matches_pattern(&rule, &context));
    }

    #[test]
    fn test_apply_single_rule() {
        let rule = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        let result = RuleApplicationEngine::apply_rule(&rule, &context);
        assert!(result.matched);
        assert_eq!(result.action, Some("add_documentation".to_string()));
    }

    #[test]
    fn test_apply_multiple_rules() {
        let rule1 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule2 = Rule::new(
            crate::models::RuleScope::Session,
            "rust".to_string(),
            "add_error_handling".to_string(),
            crate::models::RuleSource::Learned,
        );

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        let results = RuleApplicationEngine::apply_rules(&[rule1, rule2], &context);
        assert_eq!(results.len(), 2);
        assert!(results[0].matched);
        assert!(results[1].matched);
    }

    #[test]
    fn test_apply_rules_with_precedence() {
        let mut rule1 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );
        rule1.confidence = 0.7;

        let mut rule2 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_error_handling".to_string(),
            crate::models::RuleSource::Learned,
        );
        rule2.confidence = 0.9;

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        let result = RuleApplicationEngine::apply_rules_with_precedence(&[rule1, rule2], &context);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().action,
            Some("add_error_handling".to_string())
        );
    }

    #[test]
    fn test_chain_rules() {
        let rule1 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule2 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_error_handling".to_string(),
            crate::models::RuleSource::Learned,
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
    }

    #[test]
    fn test_compose_rules() {
        let rule1 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule2 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_error_handling".to_string(),
            crate::models::RuleSource::Learned,
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

    #[test]
    fn test_validate_rule_application() {
        let rule = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        let result = RuleApplicationEngine::validate_rule_application(&rule, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rule_application_empty_pattern() {
        let mut rule = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );
        rule.pattern = String::new();

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        let result = RuleApplicationEngine::validate_rule_application(&rule, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_matching_rules() {
        let rule1 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule2 = Rule::new(
            crate::models::RuleScope::Session,
            "class".to_string(),
            "add_error_handling".to_string(),
            crate::models::RuleSource::Learned,
        );

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        let matching = RuleApplicationEngine::get_matching_rules(&[rule1, rule2], &context);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].action, "add_documentation");
    }

    #[test]
    fn test_get_matching_rules_sorted_by_confidence() {
        let mut rule1 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_documentation".to_string(),
            crate::models::RuleSource::Learned,
        );
        rule1.confidence = 0.5;

        let mut rule2 = Rule::new(
            crate::models::RuleScope::Session,
            "function".to_string(),
            "add_error_handling".to_string(),
            crate::models::RuleSource::Learned,
        );
        rule2.confidence = 0.9;

        let context = GenerationContext::new(
            "function".to_string(),
            "rust".to_string(),
            "fn test() {}".to_string(),
        );

        let matching = RuleApplicationEngine::get_matching_rules_sorted(&[rule1, rule2], &context);
        assert_eq!(matching.len(), 2);
        assert_eq!(matching[0].confidence, 0.9);
        assert_eq!(matching[1].confidence, 0.5);
    }
}
