use crate::error::{CommandError, Result};
use regex::Regex;
use std::collections::HashMap;

/// Template processor for command substitution
pub struct TemplateProcessor;

impl TemplateProcessor {
    /// Process a template string with variable substitution
    ///
    /// Supports the following placeholder formats:
    /// - {{variable}} - simple substitution
    /// - {{variable:default}} - substitution with default value
    pub fn process(template: &str, variables: &HashMap<String, String>) -> Result<String> {
        let mut result = template.to_string();

        // Pattern for {{variable}} or {{variable:default}}
        let pattern = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*(?::[^}]*)?)\}\}")?;

        for cap in pattern.captures_iter(template) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_spec = cap.get(1).unwrap().as_str();

            // Parse variable name and default value
            let (var_name, default_value) = if let Some(colon_pos) = var_spec.find(':') {
                let (name, default) = var_spec.split_at(colon_pos);
                (name, Some(&default[1..]))
            } else {
                (var_spec, None)
            };

            // Get the value from variables or use default
            let value = variables
                .get(var_name)
                .map(|s| s.as_str())
                .or(default_value)
                .ok_or_else(|| {
                    CommandError::TemplateError(format!("Missing variable: {}", var_name))
                })?;

            result = result.replace(full_match, value);
        }

        Ok(result)
    }

    /// Extract all variable names from a template
    pub fn extract_variables(template: &str) -> Result<Vec<String>> {
        let pattern = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*(?::[^}]*)?)\}\}")?;
        let mut variables = Vec::new();

        for cap in pattern.captures_iter(template) {
            let var_spec = cap.get(1).unwrap().as_str();
            let var_name = if let Some(colon_pos) = var_spec.find(':') {
                &var_spec[..colon_pos]
            } else {
                var_spec
            };
            if !variables.contains(&var_name.to_string()) {
                variables.push(var_name.to_string());
            }
        }

        Ok(variables)
    }

    /// Check if a template has all required variables
    pub fn validate_variables(template: &str, provided: &HashMap<String, String>) -> Result<()> {
        let required = Self::extract_variables(template)?;

        for var in required {
            if !provided.contains_key(&var) {
                // Check if variable has a default value in template
                let pattern = Regex::new(&format!(r"\{{\{{{var}:([^}}]*)\}}\}}"))?;
                if !pattern.is_match(template) {
                    return Err(CommandError::TemplateError(format!(
                        "Missing required variable: {}",
                        var
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_substitution() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());

        let template = "Hello {{name}}, you are {{age}} years old";
        let result = TemplateProcessor::process(template, &vars).unwrap();
        assert_eq!(result, "Hello Alice, you are 30 years old");
    }

    #[test]
    fn test_substitution_with_default() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());

        let template = "Hello {{name}}, you are {{age:unknown}} years old";
        let result = TemplateProcessor::process(template, &vars).unwrap();
        assert_eq!(result, "Hello Bob, you are unknown years old");
    }

    #[test]
    fn test_missing_variable_error() {
        let vars = HashMap::new();
        let template = "Hello {{name}}";
        let result = TemplateProcessor::process(template, &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_variables() {
        let template = "Hello {{name}}, you are {{age:unknown}} years old";
        let vars = TemplateProcessor::extract_variables(template).unwrap();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"age".to_string()));
    }

    #[test]
    fn test_validate_variables_success() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Charlie".to_string());

        let template = "Hello {{name}}";
        let result = TemplateProcessor::validate_variables(template, &vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_variables_with_default() {
        let vars = HashMap::new();
        let template = "Hello {{name:Guest}}";
        let result = TemplateProcessor::validate_variables(template, &vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_variables_missing() {
        let vars = HashMap::new();
        let template = "Hello {{name}}";
        let result = TemplateProcessor::validate_variables(template, &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_variables() {
        let vars = HashMap::new();
        let template = "Hello world";
        let result = TemplateProcessor::process(template, &vars).unwrap();
        assert_eq!(result, "Hello world");
    }
}
