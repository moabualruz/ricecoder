//! Condition evaluation for conditional branching steps

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{ConditionStep, Workflow, WorkflowState};
use serde_json::Value;

/// Evaluates conditions and determines branching paths
///
/// Handles:
/// - Evaluating condition expressions based on previous step results
/// - Determining which branch (then or else) to execute
/// - Supporting nested conditions
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Evaluate a condition step and determine the execution path
    ///
    /// Returns the list of step IDs to execute based on the condition evaluation.
    /// If condition is true, returns then_steps; otherwise returns else_steps.
    pub fn evaluate_condition(
        workflow: &Workflow,
        state: &WorkflowState,
        condition_step: &ConditionStep,
    ) -> WorkflowResult<Vec<String>> {
        // Evaluate the condition expression
        let result = Self::evaluate_expression(&condition_step.condition, workflow, state)?;

        // Return appropriate branch based on result
        if result {
            Ok(condition_step.then_steps.clone())
        } else {
            Ok(condition_step.else_steps.clone())
        }
    }

    /// Evaluate a condition expression
    ///
    /// Supports simple expressions like:
    /// - "step_id.output.field == value"
    /// - "step_id.status == 'completed'"
    /// - "step_id.output.count > 5"
    ///
    /// Returns true if the condition is satisfied, false otherwise.
    pub fn evaluate_expression(
        expression: &str,
        workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<bool> {
        let expression = expression.trim();

        // Handle not equal (check before ==)
        if let Some(pos) = expression.find("!=") {
            let left = expression[..pos].trim();
            let right = expression[pos + 2..].trim();
            let equal = Self::evaluate_equality(left, right, workflow, state)?;
            return Ok(!equal);
        }

        // Handle simple equality comparisons
        if let Some(pos) = expression.find("==") {
            let left = expression[..pos].trim();
            let right = expression[pos + 2..].trim();
            return Self::evaluate_equality(left, right, workflow, state);
        }

        // Handle greater than or equal (check before >)
        if let Some(pos) = expression.find(">=") {
            let left = expression[..pos].trim();
            let right = expression[pos + 2..].trim();
            return Self::evaluate_greater_equal(left, right, workflow, state);
        }

        // Handle less than or equal (check before <)
        if let Some(pos) = expression.find("<=") {
            let left = expression[..pos].trim();
            let right = expression[pos + 2..].trim();
            return Self::evaluate_less_equal(left, right, workflow, state);
        }

        // Handle greater than comparisons
        if let Some(pos) = expression.find('>') {
            let left = expression[..pos].trim();
            let right = expression[pos + 1..].trim();
            return Self::evaluate_greater_than(left, right, workflow, state);
        }

        // Handle less than comparisons
        if let Some(pos) = expression.find('<') {
            let left = expression[..pos].trim();
            let right = expression[pos + 1..].trim();
            return Self::evaluate_less_than(left, right, workflow, state);
        }

        Err(WorkflowError::Invalid(format!(
            "Unsupported condition expression: {}",
            expression
        )))
    }

    /// Evaluate equality comparison
    pub fn evaluate_equality(
        left: &str,
        right: &str,
        workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<bool> {
        let left_value = Self::resolve_value(left, workflow, state)?;
        let right_value = Self::parse_value(right);

        Ok(left_value == right_value)
    }

    /// Evaluate greater than comparison
    pub fn evaluate_greater_than(
        left: &str,
        right: &str,
        workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<bool> {
        let left_value = Self::resolve_value(left, workflow, state)?;
        let right_value = Self::parse_value(right);

        match (left_value, right_value) {
            (Value::Number(l), Value::Number(r)) => {
                let l_f64 = l.as_f64().unwrap_or(0.0);
                let r_f64 = r.as_f64().unwrap_or(0.0);
                Ok(l_f64 > r_f64)
            }
            _ => Err(WorkflowError::Invalid(
                "Cannot compare non-numeric values with >".to_string(),
            )),
        }
    }

    /// Evaluate less than comparison
    fn evaluate_less_than(
        left: &str,
        right: &str,
        workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<bool> {
        let left_value = Self::resolve_value(left, workflow, state)?;
        let right_value = Self::parse_value(right);

        match (left_value, right_value) {
            (Value::Number(l), Value::Number(r)) => {
                let l_f64 = l.as_f64().unwrap_or(0.0);
                let r_f64 = r.as_f64().unwrap_or(0.0);
                Ok(l_f64 < r_f64)
            }
            _ => Err(WorkflowError::Invalid(
                "Cannot compare non-numeric values with <".to_string(),
            )),
        }
    }

    /// Evaluate greater than or equal comparison
    fn evaluate_greater_equal(
        left: &str,
        right: &str,
        workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<bool> {
        let left_value = Self::resolve_value(left, workflow, state)?;
        let right_value = Self::parse_value(right);

        match (left_value, right_value) {
            (Value::Number(l), Value::Number(r)) => {
                let l_f64 = l.as_f64().unwrap_or(0.0);
                let r_f64 = r.as_f64().unwrap_or(0.0);
                Ok(l_f64 >= r_f64)
            }
            _ => Err(WorkflowError::Invalid(
                "Cannot compare non-numeric values with >=".to_string(),
            )),
        }
    }

    /// Evaluate less than or equal comparison
    fn evaluate_less_equal(
        left: &str,
        right: &str,
        workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<bool> {
        let left_value = Self::resolve_value(left, workflow, state)?;
        let right_value = Self::parse_value(right);

        match (left_value, right_value) {
            (Value::Number(l), Value::Number(r)) => {
                let l_f64 = l.as_f64().unwrap_or(0.0);
                let r_f64 = r.as_f64().unwrap_or(0.0);
                Ok(l_f64 <= r_f64)
            }
            _ => Err(WorkflowError::Invalid(
                "Cannot compare non-numeric values with <=".to_string(),
            )),
        }
    }

    /// Resolve a value reference (e.g., "step_id.output.field")
    fn resolve_value(
        reference: &str,
        _workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<Value> {
        let parts: Vec<&str> = reference.split('.').collect();

        if parts.is_empty() {
            return Err(WorkflowError::Invalid(
                "Invalid value reference".to_string(),
            ));
        }

        let step_id = parts[0];

        // Get the step result
        let step_result = state.step_results.get(step_id).ok_or_else(|| {
            WorkflowError::StateError(format!("Step {} has not been executed", step_id))
        })?;

        // Start with null
        let mut value = Value::Null;
        let mut is_first = true;

        // Navigate through the path
        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                // Skip the step_id itself
                continue;
            }

            if part.is_empty() {
                continue;
            }

            // Handle special fields of the step result (only on first access after step_id)
            if is_first && *part == "output" {
                value = step_result.output.clone().unwrap_or(Value::Null);
                is_first = false;
            } else if is_first && *part == "status" {
                value = Value::String(format!("{:?}", step_result.status));
                is_first = false;
            } else if is_first && *part == "error" {
                value = step_result
                    .error
                    .as_ref()
                    .map(|e| Value::String(e.clone()))
                    .unwrap_or(Value::Null);
                is_first = false;
            } else if is_first && *part == "duration_ms" {
                value = Value::Number(serde_json::Number::from(step_result.duration_ms));
                is_first = false;
            } else {
                // Navigate through the JSON object
                is_first = false;
                // Handle array indexing like "field[0]"
                if let Some(bracket_pos) = part.find('[') {
                    let field_name = &part[..bracket_pos];
                    let index_str = &part[bracket_pos + 1..part.len() - 1];

                    if let Ok(index) = index_str.parse::<usize>() {
                        value = value[field_name][index].clone();
                    } else {
                        return Err(WorkflowError::Invalid(format!(
                            "Invalid array index: {}",
                            index_str
                        )));
                    }
                } else {
                    value = value[part].clone();
                }
            }
        }

        Ok(value)
    }

    /// Parse a literal value (e.g., "5", "'completed'", "true")
    pub fn parse_value(value_str: &str) -> Value {
        let trimmed = value_str.trim();

        // Handle string literals (quoted)
        if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('"') && trimmed.ends_with('"'))
        {
            let unquoted = &trimmed[1..trimmed.len() - 1];
            return Value::String(unquoted.to_string());
        }

        // Handle boolean
        if trimmed == "true" {
            return Value::Bool(true);
        }
        if trimmed == "false" {
            return Value::Bool(false);
        }

        // Handle null
        if trimmed == "null" {
            return Value::Null;
        }

        // Handle numbers
        if let Ok(int_val) = trimmed.parse::<i64>() {
            return Value::Number(serde_json::Number::from(int_val));
        }

        if let Ok(float_val) = trimmed.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(float_val) {
                return Value::Number(num);
            }
        }

        // Default to string
        Value::String(trimmed.to_string())
    }

    /// Get the next steps to execute after a condition
    ///
    /// Returns the list of step IDs that should be executed based on the condition result.
    pub fn get_next_steps(
        workflow: &Workflow,
        state: &WorkflowState,
        condition_step: &ConditionStep,
    ) -> WorkflowResult<Vec<String>> {
        Self::evaluate_condition(workflow, state, condition_step)
    }
}


