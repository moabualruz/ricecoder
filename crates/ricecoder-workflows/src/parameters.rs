//! Parameter parsing, validation, and substitution for workflows

use crate::error::{WorkflowError, WorkflowResult};
use serde_json::Value;
use std::collections::HashMap;

/// Parameter definition with optional default value
#[derive(Debug, Clone)]
pub struct ParameterDef {
    /// Parameter name
    pub name: String,
    /// Parameter type (string, number, boolean, object, array)
    pub param_type: ParameterType,
    /// Default value if not provided
    pub default: Option<Value>,
    /// Whether the parameter is required
    pub required: bool,
}

/// Parameter type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    /// String parameter
    String,
    /// Number parameter
    Number,
    /// Boolean parameter
    Boolean,
    /// Object parameter
    Object,
    /// Array parameter
    Array,
}

impl ParameterType {
    /// Check if a value matches this parameter type
    pub fn matches(&self, value: &Value) -> bool {
        match self {
            ParameterType::String => value.is_string(),
            ParameterType::Number => value.is_number(),
            ParameterType::Boolean => value.is_boolean(),
            ParameterType::Object => value.is_object(),
            ParameterType::Array => value.is_array(),
        }
    }
}

/// Parameter validator and parser
pub struct ParameterValidator;

impl ParameterValidator {
    /// Validate parameter definitions
    ///
    /// Ensures:
    /// - Parameter names are unique
    /// - Default values match parameter types
    /// - Required parameters don't have defaults
    pub fn validate_definitions(params: &[ParameterDef]) -> WorkflowResult<()> {
        let mut seen_names = std::collections::HashSet::new();

        for param in params {
            // Check for duplicate names
            if !seen_names.insert(&param.name) {
                return Err(WorkflowError::Invalid(format!(
                    "Duplicate parameter name: {}",
                    param.name
                )));
            }

            // Validate parameter name
            if param.name.is_empty() {
                return Err(WorkflowError::Invalid(
                    "Parameter name cannot be empty".to_string(),
                ));
            }

            // Check that parameter name is valid (alphanumeric, underscore, hyphen)
            if !param
                .name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(WorkflowError::Invalid(format!(
                    "Invalid parameter name: {}. Must contain only alphanumeric characters, underscores, and hyphens",
                    param.name
                )));
            }

            // Validate default value matches type
            if let Some(default) = &param.default {
                if !param.param_type.matches(default) {
                    return Err(WorkflowError::Invalid(format!(
                        "Default value for parameter '{}' does not match type {:?}",
                        param.name, param.param_type
                    )));
                }
            }

            // Required parameters should not have defaults
            if param.required && param.default.is_some() {
                return Err(WorkflowError::Invalid(format!(
                    "Required parameter '{}' cannot have a default value",
                    param.name
                )));
            }
        }

        Ok(())
    }

    /// Validate provided parameter values
    ///
    /// Ensures:
    /// - All required parameters are provided
    /// - Provided values match parameter types
    /// - No unknown parameters are provided
    pub fn validate_values(
        definitions: &[ParameterDef],
        values: &HashMap<String, Value>,
    ) -> WorkflowResult<()> {
        // Check for unknown parameters
        let known_names: std::collections::HashSet<_> =
            definitions.iter().map(|p| &p.name).collect();

        for provided_name in values.keys() {
            if !known_names.contains(provided_name) {
                return Err(WorkflowError::Invalid(format!(
                    "Unknown parameter: {}",
                    provided_name
                )));
            }
        }

        // Check required parameters and type matching
        for param_def in definitions {
            match values.get(&param_def.name) {
                Some(value) => {
                    // Validate type
                    if !param_def.param_type.matches(value) {
                        let type_name = match value {
                            Value::String(_) => "string",
                            Value::Number(_) => "number",
                            Value::Bool(_) => "boolean",
                            Value::Array(_) => "array",
                            Value::Object(_) => "object",
                            Value::Null => "null",
                        };
                        return Err(WorkflowError::Invalid(format!(
                            "Parameter '{}' has incorrect type. Expected {:?}, got {}",
                            param_def.name, param_def.param_type, type_name
                        )));
                    }
                }
                None => {
                    // Check if required
                    if param_def.required && param_def.default.is_none() {
                        return Err(WorkflowError::Invalid(format!(
                            "Required parameter '{}' not provided",
                            param_def.name
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Build final parameter values with defaults
    ///
    /// Merges provided values with defaults, returning a complete map
    pub fn build_final_values(
        definitions: &[ParameterDef],
        provided: &HashMap<String, Value>,
    ) -> WorkflowResult<HashMap<String, Value>> {
        Self::validate_values(definitions, provided)?;

        let mut final_values = HashMap::new();

        for param_def in definitions {
            if let Some(value) = provided.get(&param_def.name) {
                final_values.insert(param_def.name.clone(), value.clone());
            } else if let Some(default) = &param_def.default {
                final_values.insert(param_def.name.clone(), default.clone());
            }
        }

        Ok(final_values)
    }
}

/// Parameter substitution engine
pub struct ParameterSubstitutor;

impl ParameterSubstitutor {
    /// Substitute parameters in a JSON value
    ///
    /// Replaces all parameter references (${param_name}) with their values.
    /// Handles nested parameter references and validates all parameters are provided.
    pub fn substitute(value: &Value, parameters: &HashMap<String, Value>) -> WorkflowResult<Value> {
        match value {
            Value::String(s) => Self::substitute_string(s, parameters),
            Value::Object(obj) => {
                let mut result = serde_json::Map::new();
                for (key, val) in obj {
                    result.insert(key.clone(), Self::substitute(val, parameters)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
                let result: WorkflowResult<Vec<_>> = arr
                    .iter()
                    .map(|v| Self::substitute(v, parameters))
                    .collect();
                Ok(Value::Array(result?))
            }
            other => Ok(other.clone()),
        }
    }

    /// Substitute parameters in a string
    ///
    /// Replaces ${param_name} references with parameter values.
    /// Handles nested references and validates all parameters exist.
    fn substitute_string(s: &str, parameters: &HashMap<String, Value>) -> WorkflowResult<Value> {
        let mut result = s.to_string();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10; // Prevent infinite loops

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(WorkflowError::Invalid(
                    "Parameter substitution exceeded maximum iterations (possible circular reference)"
                        .to_string(),
                ));
            }

            // Find all parameter references
            let mut found_any = false;
            let mut new_result = result.clone();

            // Find ${...} patterns
            let mut start = 0;
            while let Some(pos) = new_result[start..].find("${") {
                let actual_pos = start + pos;
                if let Some(end_pos) = new_result[actual_pos + 2..].find('}') {
                    let actual_end = actual_pos + 2 + end_pos;
                    let param_name = &new_result[actual_pos + 2..actual_end];

                    // Validate parameter name
                    if !param_name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                    {
                        return Err(WorkflowError::Invalid(format!(
                            "Invalid parameter reference: ${{{}}}",
                            param_name
                        )));
                    }

                    // Get parameter value
                    match parameters.get(param_name) {
                        Some(value) => {
                            let replacement = match value {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                Value::Bool(b) => b.to_string(),
                                Value::Null => "null".to_string(),
                                _ => {
                                    return Err(WorkflowError::Invalid(format!(
                                        "Cannot substitute complex type for parameter '{}'",
                                        param_name
                                    )))
                                }
                            };

                            new_result.replace_range(actual_pos..=actual_end, &replacement);
                            found_any = true;
                            start = actual_pos + replacement.len();
                        }
                        None => {
                            return Err(WorkflowError::Invalid(format!(
                                "Parameter '{}' not provided",
                                param_name
                            )));
                        }
                    }
                } else {
                    start = actual_pos + 2;
                }
            }

            result = new_result;

            if !found_any {
                break;
            }
        }

        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_parameter_definitions_valid() {
        let params = vec![
            ParameterDef {
                name: "name".to_string(),
                param_type: ParameterType::String,
                default: Some(json!("default-name")),
                required: false,
            },
            ParameterDef {
                name: "count".to_string(),
                param_type: ParameterType::Number,
                default: None,
                required: true,
            },
        ];

        assert!(ParameterValidator::validate_definitions(&params).is_ok());
    }

    #[test]
    fn test_validate_parameter_definitions_duplicate_names() {
        let params = vec![
            ParameterDef {
                name: "name".to_string(),
                param_type: ParameterType::String,
                default: None,
                required: false,
            },
            ParameterDef {
                name: "name".to_string(),
                param_type: ParameterType::String,
                default: None,
                required: false,
            },
        ];

        assert!(ParameterValidator::validate_definitions(&params).is_err());
    }

    #[test]
    fn test_validate_parameter_definitions_type_mismatch() {
        let params = vec![ParameterDef {
            name: "count".to_string(),
            param_type: ParameterType::Number,
            default: Some(json!("not-a-number")),
            required: false,
        }];

        assert!(ParameterValidator::validate_definitions(&params).is_err());
    }

    #[test]
    fn test_validate_parameter_values_missing_required() {
        let definitions = vec![ParameterDef {
            name: "required_param".to_string(),
            param_type: ParameterType::String,
            default: None,
            required: true,
        }];

        let values = HashMap::new();

        assert!(ParameterValidator::validate_values(&definitions, &values).is_err());
    }

    #[test]
    fn test_validate_parameter_values_unknown_parameter() {
        let definitions = vec![ParameterDef {
            name: "known".to_string(),
            param_type: ParameterType::String,
            default: None,
            required: false,
        }];

        let mut values = HashMap::new();
        values.insert("unknown".to_string(), json!("value"));

        assert!(ParameterValidator::validate_values(&definitions, &values).is_err());
    }

    #[test]
    fn test_validate_parameter_values_type_mismatch() {
        let definitions = vec![ParameterDef {
            name: "count".to_string(),
            param_type: ParameterType::Number,
            default: None,
            required: true,
        }];

        let mut values = HashMap::new();
        values.insert("count".to_string(), json!("not-a-number"));

        assert!(ParameterValidator::validate_values(&definitions, &values).is_err());
    }

    #[test]
    fn test_build_final_values_with_defaults() {
        let definitions = vec![
            ParameterDef {
                name: "name".to_string(),
                param_type: ParameterType::String,
                default: Some(json!("default-name")),
                required: false,
            },
            ParameterDef {
                name: "count".to_string(),
                param_type: ParameterType::Number,
                default: None,
                required: true,
            },
        ];

        let mut provided = HashMap::new();
        provided.insert("count".to_string(), json!(42));

        let result = ParameterValidator::build_final_values(&definitions, &provided);
        assert!(result.is_ok());

        let final_values = result.unwrap();
        assert_eq!(final_values.get("name"), Some(&json!("default-name")));
        assert_eq!(final_values.get("count"), Some(&json!(42)));
    }

    #[test]
    fn test_substitute_simple_string() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));

        let result = ParameterSubstitutor::substitute(&json!("Hello ${name}"), &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("Hello Alice"));
    }

    #[test]
    fn test_substitute_multiple_references() {
        let mut params = HashMap::new();
        params.insert("first".to_string(), json!("Alice"));
        params.insert("last".to_string(), json!("Smith"));

        let result = ParameterSubstitutor::substitute(&json!("${first} ${last}"), &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("Alice Smith"));
    }

    #[test]
    fn test_substitute_in_object() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));

        let input = json!({
            "greeting": "Hello ${name}",
            "nested": {
                "message": "Welcome ${name}"
            }
        });

        let result = ParameterSubstitutor::substitute(&input, &params);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(
            result.get("greeting").and_then(|v| v.as_str()),
            Some("Hello Alice")
        );
        assert_eq!(
            result
                .get("nested")
                .and_then(|v| v.get("message"))
                .and_then(|v| v.as_str()),
            Some("Welcome Alice")
        );
    }

    #[test]
    fn test_substitute_in_array() {
        let mut params = HashMap::new();
        params.insert("item".to_string(), json!("apple"));

        let input = json!(["${item}", "banana", "${item}"]);

        let result = ParameterSubstitutor::substitute(&input, &params);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.as_array().map(|a| a.len()), Some(3));
    }

    #[test]
    fn test_substitute_missing_parameter() {
        let params = HashMap::new();

        let result = ParameterSubstitutor::substitute(&json!("Hello ${name}"), &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_substitute_number_parameter() {
        let mut params = HashMap::new();
        params.insert("count".to_string(), json!(42));

        let result = ParameterSubstitutor::substitute(&json!("Count: ${count}"), &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("Count: 42"));
    }

    #[test]
    fn test_substitute_boolean_parameter() {
        let mut params = HashMap::new();
        params.insert("enabled".to_string(), json!(true));

        let result = ParameterSubstitutor::substitute(&json!("Enabled: ${enabled}"), &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("Enabled: true"));
    }

    #[test]
    fn test_substitute_nested_references() {
        let mut params = HashMap::new();
        params.insert("greeting".to_string(), json!("Hello"));
        params.insert("name".to_string(), json!("${greeting} Alice"));

        // Nested references are recursively substituted
        let result = ParameterSubstitutor::substitute(&json!("${name}"), &params);
        assert!(result.is_ok());
        // The result should have both references substituted
        assert_eq!(result.unwrap(), json!("Hello Alice"));
    }

    #[test]
    fn test_substitute_invalid_parameter_name() {
        let params = HashMap::new();

        let result = ParameterSubstitutor::substitute(&json!("Hello ${na@me}"), &params);
        assert!(result.is_err());
    }
}
