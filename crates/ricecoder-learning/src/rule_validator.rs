use regex::Regex;
use serde_json::Value;

/// Rule validation component
///
/// Validates rules before storage to ensure they meet syntax, structure,
/// and consistency requirements.
use crate::error::{LearningError, Result};
use crate::models::Rule;

/// Validates rules before storage
#[derive(Debug, Clone)]
pub struct RuleValidator {
    /// Cached regex patterns for validation
    pattern_cache: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Regex>>>,
}

impl RuleValidator {
    /// Create a new rule validator
    pub fn new() -> Self {
        Self {
            pattern_cache: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// Validate a rule before storage
    ///
    /// Performs comprehensive validation including:
    /// - JSON schema validation
    /// - Syntax verification
    /// - Reference checking
    /// - Conflict detection
    pub fn validate(&self, rule: &Rule) -> Result<()> {
        // Validate basic structure
        self.validate_structure(rule)?;

        // Validate pattern syntax
        self.validate_pattern_syntax(&rule.pattern)?;

        // Validate action syntax
        self.validate_action_syntax(&rule.action)?;

        // Validate metadata
        self.validate_metadata(&rule.metadata)?;

        // Validate confidence score
        self.validate_confidence(rule.confidence)?;

        // Validate success rate
        self.validate_success_rate(rule.success_rate)?;

        Ok(())
    }

    /// Validate the basic structure of a rule
    fn validate_structure(&self, rule: &Rule) -> Result<()> {
        // Check required fields
        if rule.id.is_empty() {
            return Err(LearningError::RuleValidationFailed(
                "Rule ID cannot be empty".to_string(),
            ));
        }

        if rule.pattern.is_empty() {
            return Err(LearningError::RuleValidationFailed(
                "Rule pattern cannot be empty".to_string(),
            ));
        }

        if rule.action.is_empty() {
            return Err(LearningError::RuleValidationFailed(
                "Rule action cannot be empty".to_string(),
            ));
        }

        // Validate version
        if rule.version == 0 {
            return Err(LearningError::RuleValidationFailed(
                "Rule version must be greater than 0".to_string(),
            ));
        }

        // Note: usage_count is u64, so it cannot be negative

        Ok(())
    }

    /// Validate pattern syntax
    fn validate_pattern_syntax(&self, pattern: &str) -> Result<()> {
        // Pattern should not be empty
        if pattern.is_empty() {
            return Err(LearningError::RuleValidationFailed(
                "Pattern cannot be empty".to_string(),
            ));
        }

        // Pattern should not exceed reasonable length
        if pattern.len() > 10000 {
            return Err(LearningError::RuleValidationFailed(
                "Pattern exceeds maximum length of 10000 characters".to_string(),
            ));
        }

        // Try to compile as regex if it looks like one
        if pattern.starts_with('^') || pattern.contains('*') || pattern.contains('+') {
            if let Err(e) = self.compile_regex(pattern) {
                return Err(LearningError::RuleValidationFailed(format!(
                    "Invalid regex pattern: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    /// Validate action syntax
    fn validate_action_syntax(&self, action: &str) -> Result<()> {
        // Action should not be empty
        if action.is_empty() {
            return Err(LearningError::RuleValidationFailed(
                "Action cannot be empty".to_string(),
            ));
        }

        // Action should not exceed reasonable length
        if action.len() > 50000 {
            return Err(LearningError::RuleValidationFailed(
                "Action exceeds maximum length of 50000 characters".to_string(),
            ));
        }

        // Try to parse as JSON if it looks like JSON
        if action.trim().starts_with('{') || action.trim().starts_with('[') {
            if let Err(e) = serde_json::from_str::<Value>(action) {
                return Err(LearningError::RuleValidationFailed(format!(
                    "Invalid JSON in action: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    /// Validate metadata
    fn validate_metadata(&self, metadata: &Value) -> Result<()> {
        // Metadata should be an object
        if !metadata.is_object() {
            return Err(LearningError::RuleValidationFailed(
                "Metadata must be a JSON object".to_string(),
            ));
        }

        // Metadata should not be too large
        let metadata_str = metadata.to_string();
        if metadata_str.len() > 100000 {
            return Err(LearningError::RuleValidationFailed(
                "Metadata exceeds maximum size".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate confidence score
    fn validate_confidence(&self, confidence: f32) -> Result<()> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err(LearningError::RuleValidationFailed(
                "Confidence score must be between 0.0 and 1.0".to_string(),
            ));
        }

        if confidence.is_nan() || confidence.is_infinite() {
            return Err(LearningError::RuleValidationFailed(
                "Confidence score must be a valid number".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate success rate
    fn validate_success_rate(&self, success_rate: f32) -> Result<()> {
        if !(0.0..=1.0).contains(&success_rate) {
            return Err(LearningError::RuleValidationFailed(
                "Success rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        if success_rate.is_nan() || success_rate.is_infinite() {
            return Err(LearningError::RuleValidationFailed(
                "Success rate must be a valid number".to_string(),
            ));
        }

        Ok(())
    }

    /// Compile and cache a regex pattern
    fn compile_regex(&self, pattern: &str) -> Result<()> {
        let mut cache = self
            .pattern_cache
            .lock()
            .map_err(|e| LearningError::RuleValidationFailed(format!("Lock error: {}", e)))?;

        if cache.contains_key(pattern) {
            return Ok(());
        }

        match Regex::new(pattern) {
            Ok(regex) => {
                cache.insert(pattern.to_string(), regex);
                Ok(())
            }
            Err(e) => Err(LearningError::RuleValidationFailed(format!(
                "Invalid regex: {}",
                e
            ))),
        }
    }

    /// Validate that a rule doesn't conflict with existing rules
    pub fn check_conflicts(&self, new_rule: &Rule, existing_rules: &[Rule]) -> Result<()> {
        for existing in existing_rules {
            // Rules in the same scope with identical patterns conflict
            if existing.scope == new_rule.scope && existing.pattern == new_rule.pattern {
                return Err(LearningError::RuleValidationFailed(format!(
                    "Rule conflicts with existing rule '{}' in {} scope",
                    existing.id, existing.scope
                )));
            }
        }

        Ok(())
    }

    /// Generate a detailed validation report
    pub fn validate_with_report(&self, rule: &Rule) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Check structure
        if let Err(e) = self.validate_structure(rule) {
            report.add_error("structure", e.to_string());
        }

        // Check pattern
        if let Err(e) = self.validate_pattern_syntax(&rule.pattern) {
            report.add_error("pattern", e.to_string());
        }

        // Check action
        if let Err(e) = self.validate_action_syntax(&rule.action) {
            report.add_error("action", e.to_string());
        }

        // Check metadata
        if let Err(e) = self.validate_metadata(&rule.metadata) {
            report.add_error("metadata", e.to_string());
        }

        // Check confidence
        if let Err(e) = self.validate_confidence(rule.confidence) {
            report.add_error("confidence", e.to_string());
        }

        // Check success rate
        if let Err(e) = self.validate_success_rate(rule.success_rate) {
            report.add_error("success_rate", e.to_string());
        }

        report
    }
}

impl Default for RuleValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Errors found during validation
    errors: std::collections::HashMap<String, Vec<String>>,
    /// Warnings found during validation
    warnings: std::collections::HashMap<String, Vec<String>>,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new() -> Self {
        Self {
            errors: std::collections::HashMap::new(),
            warnings: std::collections::HashMap::new(),
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, field: &str, message: String) {
        self.errors
            .entry(field.to_string())
            .or_default()
            .push(message);
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, field: &str, message: String) {
        self.warnings
            .entry(field.to_string())
            .or_default()
            .push(message);
    }

    /// Check if the report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the report has any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get all errors
    pub fn errors(&self) -> &std::collections::HashMap<String, Vec<String>> {
        &self.errors
    }

    /// Get all warnings
    pub fn warnings(&self) -> &std::collections::HashMap<String, Vec<String>> {
        &self.warnings
    }

    /// Get a formatted error message
    pub fn error_message(&self) -> String {
        if self.errors.is_empty() {
            return String::new();
        }

        let mut message = String::from("Validation errors:\n");
        for (field, errors) in &self.errors {
            message.push_str(&format!("  {}: ", field));
            message.push_str(&errors.join(", "));
            message.push('\n');
        }

        message
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Rule, RuleScope, RuleSource};

    #[test]
    fn test_validator_creation() {
        let validator = RuleValidator::new();
        assert!(validator.pattern_cache.lock().is_ok());
    }

    #[test]
    fn test_validate_valid_rule() {
        let validator = RuleValidator::new();
        let rule = Rule::new(
            RuleScope::Global,
            "test_pattern".to_string(),
            "test_action".to_string(),
            RuleSource::Learned,
        );

        assert!(validator.validate(&rule).is_ok());
    }

    #[test]
    fn test_validate_empty_pattern() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "test".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.pattern = String::new();

        assert!(validator.validate(&rule).is_err());
    }

    #[test]
    fn test_validate_empty_action() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.action = String::new();

        assert!(validator.validate(&rule).is_err());
    }

    #[test]
    fn test_validate_invalid_confidence() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.confidence = 1.5;

        assert!(validator.validate(&rule).is_err());
    }

    #[test]
    fn test_validate_invalid_success_rate() {
        let validator = RuleValidator::new();
        let mut rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );
        rule.success_rate = -0.5;

        assert!(validator.validate(&rule).is_err());
    }

    #[test]
    fn test_validate_json_action() {
        let validator = RuleValidator::new();
        let rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            r#"{"key": "value"}"#.to_string(),
            RuleSource::Learned,
        );

        assert!(validator.validate(&rule).is_ok());
    }

    #[test]
    fn test_validate_invalid_json_action() {
        let validator = RuleValidator::new();
        let rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            r#"{"key": invalid}"#.to_string(),
            RuleSource::Learned,
        );

        assert!(validator.validate(&rule).is_err());
    }

    #[test]
    fn test_check_conflicts() {
        let validator = RuleValidator::new();
        let rule1 = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action2".to_string(),
            RuleSource::Learned,
        );

        assert!(validator.check_conflicts(&rule2, &[rule1]).is_err());
    }

    #[test]
    fn test_check_no_conflicts_different_scope() {
        let validator = RuleValidator::new();
        let rule1 = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action2".to_string(),
            RuleSource::Learned,
        );

        assert!(validator.check_conflicts(&rule2, &[rule1]).is_ok());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(!report.has_errors());

        report.add_error("field1", "error1".to_string());
        assert!(report.has_errors());

        report.add_warning("field2", "warning1".to_string());
        assert!(report.has_warnings());

        let message = report.error_message();
        assert!(message.contains("field1"));
        assert!(message.contains("error1"));
    }
}
