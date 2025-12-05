//! Pattern validation

use crate::error::{RefactoringError, Result};
use super::RefactoringPattern;

/// Validates refactoring patterns
pub struct PatternValidator;

impl PatternValidator {
    /// Validate a pattern
    pub fn validate(pattern: &RefactoringPattern) -> Result<()> {
        // Check pattern name
        if pattern.name.is_empty() {
            return Err(RefactoringError::InvalidConfiguration(
                "Pattern name cannot be empty".to_string(),
            ));
        }

        // Check pattern template
        if pattern.template.is_empty() {
            return Err(RefactoringError::InvalidConfiguration(
                "Pattern template cannot be empty".to_string(),
            ));
        }

        // Check that all placeholders in template have corresponding parameters
        let template_placeholders = Self::extract_placeholders(&pattern.template);
        let parameter_placeholders: std::collections::HashSet<_> =
            pattern.parameters.iter().map(|p| p.placeholder.clone()).collect();

        for placeholder in template_placeholders {
            if !parameter_placeholders.contains(&placeholder) {
                return Err(RefactoringError::InvalidConfiguration(format!(
                    "Template uses placeholder {} but no parameter defined",
                    placeholder
                )));
            }
        }

        // Check that all parameters are used in template
        for param in &pattern.parameters {
            if !pattern.template.contains(&param.placeholder) {
                return Err(RefactoringError::InvalidConfiguration(format!(
                    "Parameter {} is defined but not used in template",
                    param.name
                )));
            }
        }

        Ok(())
    }

    /// Extract placeholders from a template
    fn extract_placeholders(template: &str) -> Vec<String> {
        let mut placeholders = vec![];
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second {
                let mut placeholder = String::from("{{");

                while let Some(ch) = chars.next() {
                    placeholder.push(ch);
                    if ch == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second }
                        placeholder.push('}');
                        placeholders.push(placeholder);
                        break;
                    }
                }
            }
        }

        placeholders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::{RefactoringPattern, PatternParameter, PatternScope};

    #[test]
    fn test_validate_valid_pattern() -> Result<()> {
        let pattern = RefactoringPattern {
            name: "test".to_string(),
            description: "Test pattern".to_string(),
            template: "fn {{old_name}}() -> fn {{new_name}}()".to_string(),
            parameters: vec![
                PatternParameter {
                    name: "old_name".to_string(),
                    placeholder: "{{old_name}}".to_string(),
                    description: "Old name".to_string(),
                },
                PatternParameter {
                    name: "new_name".to_string(),
                    placeholder: "{{new_name}}".to_string(),
                    description: "New name".to_string(),
                },
            ],
            scope: PatternScope::Global,
        };

        PatternValidator::validate(&pattern)?;
        Ok(())
    }

    #[test]
    fn test_validate_empty_name() {
        let pattern = RefactoringPattern {
            name: "".to_string(),
            description: "Test pattern".to_string(),
            template: "template".to_string(),
            parameters: vec![],
            scope: PatternScope::Global,
        };

        assert!(PatternValidator::validate(&pattern).is_err());
    }

    #[test]
    fn test_validate_empty_template() {
        let pattern = RefactoringPattern {
            name: "test".to_string(),
            description: "Test pattern".to_string(),
            template: "".to_string(),
            parameters: vec![],
            scope: PatternScope::Global,
        };

        assert!(PatternValidator::validate(&pattern).is_err());
    }

    #[test]
    fn test_validate_unused_placeholder() {
        let pattern = RefactoringPattern {
            name: "test".to_string(),
            description: "Test pattern".to_string(),
            template: "fn {{old_name}}()".to_string(),
            parameters: vec![
                PatternParameter {
                    name: "old_name".to_string(),
                    placeholder: "{{old_name}}".to_string(),
                    description: "Old name".to_string(),
                },
                PatternParameter {
                    name: "new_name".to_string(),
                    placeholder: "{{new_name}}".to_string(),
                    description: "New name".to_string(),
                },
            ],
            scope: PatternScope::Global,
        };

        assert!(PatternValidator::validate(&pattern).is_err());
    }

    #[test]
    fn test_extract_placeholders() {
        let template = "fn {{old_name}}() -> fn {{new_name}}()";
        let placeholders = PatternValidator::extract_placeholders(template);

        assert_eq!(placeholders.len(), 2);
        assert!(placeholders.contains(&"{{old_name}}".to_string()));
        assert!(placeholders.contains(&"{{new_name}}".to_string()));
    }
}
