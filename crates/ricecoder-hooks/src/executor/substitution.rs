//! Variable substitution engine for hooks
//!
//! This module provides variable substitution functionality for hook actions.
//! It supports placeholder parsing, variable lookup in event context, and nested paths.
//!
//! # Examples
//!
//! Basic variable substitution:
//!
//! ```ignore
//! use ricecoder_hooks::executor::VariableSubstitutor;
//! use ricecoder_hooks::types::EventContext;
//! use serde_json::json;
//!
//! let context = EventContext {
//!     data: json!({
//!         "file_path": "/path/to/file.rs",
//!         "size": 1024,
//!     }),
//!     metadata: json!({}),
//! };
//!
//! let template = "Processing {{file_path}} ({{size}} bytes)";
//! let result = VariableSubstitutor::substitute(template, &context)?;
//! assert_eq!(result, "Processing /path/to/file.rs (1024 bytes)");
//! ```
//!
//! Nested path substitution:
//!
//! ```ignore
//! let context = EventContext {
//!     data: json!({
//!         "metadata": {
//!             "size": 2048,
//!             "hash": "abc123",
//!         }
//!     }),
//!     metadata: json!({}),
//! };
//!
//! let template = "File size: {{metadata.size}}, hash: {{metadata.hash}}";
//! let result = VariableSubstitutor::substitute(template, &context)?;
//! assert_eq!(result, "File size: 2048, hash: abc123");
//! ```

use crate::error::{HooksError, Result};
use crate::types::EventContext;
use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

/// Variable substitution engine
///
/// Provides methods for substituting variables in templates using event context.
/// Supports:
/// - Simple variable substitution: `{{variable_name}}`
/// - Nested path substitution: `{{metadata.size}}`
/// - Literal text with variables: `"path/{{file_path}}/subdir"`
/// - Error handling for missing variables
pub struct VariableSubstitutor;

impl VariableSubstitutor {
    /// Substitute variables in a template string
    ///
    /// Replaces all `{{variable_name}}` placeholders with values from the event context.
    /// Supports nested paths like `{{metadata.size}}`.
    ///
    /// # Arguments
    ///
    /// * `template` - Template string with placeholders
    /// * `context` - Event context containing variables
    ///
    /// # Returns
    ///
    /// Substituted string or error if variables are missing
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let context = EventContext {
    ///     data: json!({"file_path": "/path/to/file.rs"}),
    ///     metadata: json!({}),
    /// };
    /// let result = VariableSubstitutor::substitute("File: {{file_path}}", &context)?;
    /// assert_eq!(result, "File: /path/to/file.rs");
    /// ```
    pub fn substitute(template: &str, context: &EventContext) -> Result<String> {
        let placeholder_regex = get_placeholder_regex();

        let mut result = template.to_string();

        for cap in placeholder_regex.captures_iter(template) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str();

            let value = Self::lookup_variable(var_name, context)?;
            let value_str = Self::value_to_string(&value);

            result = result.replace(full_match, &value_str);
        }

        Ok(result)
    }

    /// Substitute variables in a JSON value
    ///
    /// If the value is a string, performs variable substitution.
    /// If the value is an object or array, recursively substitutes in all string values.
    /// Other types are returned unchanged.
    ///
    /// # Arguments
    ///
    /// * `value` - JSON value to substitute
    /// * `context` - Event context containing variables
    ///
    /// # Returns
    ///
    /// Substituted JSON value or error if variables are missing
    pub fn substitute_json(value: &Value, context: &EventContext) -> Result<Value> {
        match value {
            Value::String(s) => {
                let substituted = Self::substitute(s, context)?;
                Ok(Value::String(substituted))
            }
            Value::Object(map) => {
                let mut result = serde_json::Map::new();
                for (key, val) in map {
                    result.insert(key.clone(), Self::substitute_json(val, context)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
                let result: Result<Vec<Value>> = arr
                    .iter()
                    .map(|v| Self::substitute_json(v, context))
                    .collect();
                Ok(Value::Array(result?))
            }
            other => Ok(other.clone()),
        }
    }

    /// Look up a variable in the event context
    ///
    /// Supports nested paths using dot notation (e.g., `metadata.size`).
    /// First looks in `context.data`, then in `context.metadata`.
    ///
    /// # Arguments
    ///
    /// * `var_name` - Variable name (supports dot notation for nested paths)
    /// * `context` - Event context
    ///
    /// # Returns
    ///
    /// JSON value or error if variable not found
    fn lookup_variable(var_name: &str, context: &EventContext) -> Result<Value> {
        // Try to find in data first
        if let Some(value) = Self::lookup_in_value(var_name, &context.data) {
            return Ok(value);
        }

        // Try to find in metadata
        if let Some(value) = Self::lookup_in_value(var_name, &context.metadata) {
            return Ok(value);
        }

        // Variable not found
        Err(HooksError::ExecutionFailed(format!(
            "Variable not found in context: {}",
            var_name
        )))
    }

    /// Look up a variable in a JSON value using dot notation
    ///
    /// Supports nested paths like `metadata.size` or `user.profile.name`.
    ///
    /// # Arguments
    ///
    /// * `path` - Variable path (dot-separated)
    /// * `value` - JSON value to search in
    ///
    /// # Returns
    ///
    /// JSON value if found, None otherwise
    fn lookup_in_value(path: &str, value: &Value) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();

        let mut current = value;
        for part in parts {
            current = current.get(part)?;
        }

        Some(current.clone())
    }

    /// Convert a JSON value to a string
    ///
    /// Handles different JSON types appropriately:
    /// - Strings: returned as-is
    /// - Numbers: converted to string representation
    /// - Booleans: converted to "true" or "false"
    /// - Null: converted to "null"
    /// - Objects/Arrays: converted to JSON string
    fn value_to_string(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(_) | Value::Object(_) => value.to_string(),
        }
    }
}

/// Get or create the placeholder regex
///
/// Uses OnceLock to compile the regex only once for performance.
fn get_placeholder_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        // Matches {{variable_name}} or {{nested.path}}
        Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_\.]*)\}\}").expect("Invalid regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_context() -> EventContext {
        EventContext {
            data: json!({
                "file_path": "/path/to/file.rs",
                "size": 1024,
                "hash": "abc123def456",
                "metadata": {
                    "created": "2024-01-01",
                    "modified": "2024-01-02",
                    "nested": {
                        "deep": "value"
                    }
                }
            }),
            metadata: json!({
                "user": "alice",
                "project": "my-project"
            }),
        }
    }

    #[test]
    fn test_substitute_simple_variable() {
        let context = create_test_context();
        let template = "File: {{file_path}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "File: /path/to/file.rs");
    }

    #[test]
    fn test_substitute_multiple_variables() {
        let context = create_test_context();
        let template = "File {{file_path}} is {{size}} bytes";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "File /path/to/file.rs is 1024 bytes");
    }

    #[test]
    fn test_substitute_nested_path() {
        let context = create_test_context();
        let template = "Created: {{metadata.created}}, Modified: {{metadata.modified}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "Created: 2024-01-01, Modified: 2024-01-02");
    }

    #[test]
    fn test_substitute_deeply_nested_path() {
        let context = create_test_context();
        let template = "Deep value: {{metadata.nested.deep}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "Deep value: value");
    }

    #[test]
    fn test_substitute_from_metadata() {
        let context = create_test_context();
        let template = "User: {{user}}, Project: {{project}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "User: alice, Project: my-project");
    }

    #[test]
    fn test_substitute_missing_variable() {
        let context = create_test_context();
        let template = "File: {{missing_var}}";
        let result = VariableSubstitutor::substitute(template, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_substitute_missing_nested_path() {
        let context = create_test_context();
        let template = "Value: {{metadata.missing.path}}";
        let result = VariableSubstitutor::substitute(template, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_substitute_no_variables() {
        let context = create_test_context();
        let template = "No variables here";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "No variables here");
    }

    #[test]
    fn test_substitute_empty_template() {
        let context = create_test_context();
        let template = "";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_substitute_number_variable() {
        let context = create_test_context();
        let template = "Size: {{size}} bytes";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "Size: 1024 bytes");
    }

    #[test]
    fn test_substitute_json_string() {
        let context = create_test_context();
        let value = json!("File: {{file_path}}");
        let result = VariableSubstitutor::substitute_json(&value, &context).unwrap();
        assert_eq!(result, json!("File: /path/to/file.rs"));
    }

    #[test]
    fn test_substitute_json_object() {
        let context = create_test_context();
        let value = json!({
            "path": "{{file_path}}",
            "size": "{{size}}"
        });
        let result = VariableSubstitutor::substitute_json(&value, &context).unwrap();
        assert_eq!(
            result,
            json!({
                "path": "/path/to/file.rs",
                "size": "1024"
            })
        );
    }

    #[test]
    fn test_substitute_json_array() {
        let context = create_test_context();
        let value = json!(["{{file_path}}", "{{size}}"]);
        let result = VariableSubstitutor::substitute_json(&value, &context).unwrap();
        assert_eq!(result, json!(["/path/to/file.rs", "1024"]));
    }

    #[test]
    fn test_substitute_json_number_unchanged() {
        let context = create_test_context();
        let value = json!(42);
        let result = VariableSubstitutor::substitute_json(&value, &context).unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_substitute_json_nested_object() {
        let context = create_test_context();
        let value = json!({
            "file": {
                "path": "{{file_path}}",
                "size": "{{size}}"
            }
        });
        let result = VariableSubstitutor::substitute_json(&value, &context).unwrap();
        assert_eq!(
            result,
            json!({
                "file": {
                    "path": "/path/to/file.rs",
                    "size": "1024"
                }
            })
        );
    }

    #[test]
    fn test_substitute_same_variable_multiple_times() {
        let context = create_test_context();
        let template = "{{file_path}} and {{file_path}} again";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "/path/to/file.rs and /path/to/file.rs again");
    }

    #[test]
    fn test_substitute_variable_at_start() {
        let context = create_test_context();
        let template = "{{file_path}} is a file";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "/path/to/file.rs is a file");
    }

    #[test]
    fn test_substitute_variable_at_end() {
        let context = create_test_context();
        let template = "The file is {{file_path}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "The file is /path/to/file.rs");
    }

    #[test]
    fn test_substitute_variable_only() {
        let context = create_test_context();
        let template = "{{file_path}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "/path/to/file.rs");
    }

    #[test]
    fn test_substitute_with_special_characters() {
        let mut context = create_test_context();
        context.data = json!({
            "path": "/path/with/special-chars_123.rs"
        });
        let template = "File: {{path}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "File: /path/with/special-chars_123.rs");
    }

    #[test]
    fn test_substitute_boolean_variable() {
        let mut context = create_test_context();
        context.data = json!({
            "enabled": true
        });
        let template = "Enabled: {{enabled}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "Enabled: true");
    }

    #[test]
    fn test_substitute_null_variable() {
        let mut context = create_test_context();
        context.data = json!({
            "value": null
        });
        let template = "Value: {{value}}";
        let result = VariableSubstitutor::substitute(template, &context).unwrap();
        assert_eq!(result, "Value: null");
    }
}
