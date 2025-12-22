//! Condition evaluation for hook execution

use tracing::{debug, warn};

use crate::{
    error::{HooksError, Result},
    types::{Condition, EventContext},
};

/// Evaluates conditions against event context
///
/// Conditions allow hooks to be executed conditionally based on event context values.
/// For now, this is a placeholder implementation that always returns true.
/// Future implementations can support more complex condition expressions.
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Evaluate a condition against event context
    ///
    /// # Arguments
    ///
    /// * `condition` - The condition to evaluate
    /// * `context` - The event context to evaluate against
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Condition is met, hook should execute
    /// * `Ok(false)` - Condition is not met, hook should be skipped
    /// * `Err` - Error evaluating condition
    pub fn evaluate(condition: &Condition, context: &EventContext) -> Result<bool> {
        debug!(
            expression = %condition.expression,
            context_keys = ?condition.context_keys,
            "Evaluating condition"
        );

        // Verify that all required context keys are present
        for key in &condition.context_keys {
            if context.data.get(key).is_none() && context.metadata.get(key).is_none() {
                warn!(
                    key = %key,
                    "Required context key not found for condition evaluation"
                );
                return Err(HooksError::InvalidConfiguration(format!(
                    "Required context key '{}' not found",
                    key
                )));
            }
        }

        // For now, always return true (conditions are evaluated but always pass)
        // Future implementations can support:
        // - Simple comparisons: file_path.ends_with('.rs')
        // - Pattern matching: file_path matches '*.rs'
        // - Logical operators: AND, OR, NOT
        // - Nested conditions
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn create_test_context() -> EventContext {
        EventContext {
            data: json!({
                "file_path": "/path/to/file.rs",
                "size": 1024,
            }),
            metadata: json!({
                "user": "alice",
                "project": "my-project",
            }),
        }
    }

    #[test]
    fn test_evaluate_condition_with_valid_context() {
        let condition = Condition {
            expression: "file_path.ends_with('.rs')".to_string(),
            context_keys: vec!["file_path".to_string()],
        };
        let context = create_test_context();

        let result = ConditionEvaluator::evaluate(&condition, &context).unwrap();

        assert!(result);
    }

    #[test]
    fn test_evaluate_condition_with_missing_context_key() {
        let condition = Condition {
            expression: "missing_key == 'value'".to_string(),
            context_keys: vec!["missing_key".to_string()],
        };
        let context = create_test_context();

        let result = ConditionEvaluator::evaluate(&condition, &context);

        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_condition_with_metadata_key() {
        let condition = Condition {
            expression: "user == 'alice'".to_string(),
            context_keys: vec!["user".to_string()],
        };
        let context = create_test_context();

        let result = ConditionEvaluator::evaluate(&condition, &context).unwrap();

        assert!(result);
    }

    #[test]
    fn test_evaluate_condition_with_multiple_keys() {
        let condition = Condition {
            expression: "file_path.ends_with('.rs') && user == 'alice'".to_string(),
            context_keys: vec!["file_path".to_string(), "user".to_string()],
        };
        let context = create_test_context();

        let result = ConditionEvaluator::evaluate(&condition, &context).unwrap();

        assert!(result);
    }
}
