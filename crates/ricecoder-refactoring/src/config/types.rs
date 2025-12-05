//! Configuration types for refactoring engine

use serde::{Deserialize, Serialize};
use crate::types::{RefactoringRule, RefactoringTransformation};

/// Configuration for a language's refactoring rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Language name
    pub language: String,
    /// File extensions for this language
    pub extensions: Vec<String>,
    /// Parser plugin (e.g., tree-sitter-rust)
    pub parser_plugin: Option<String>,
    /// Refactoring rules
    pub rules: Vec<RefactoringRule>,
    /// Refactoring transformations
    pub transformations: Vec<RefactoringTransformation>,
}

impl LanguageConfig {
    /// Validate the configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.language.is_empty() {
            return Err(crate::error::RefactoringError::InvalidConfiguration(
                "Language name cannot be empty".to_string(),
            ));
        }

        if self.extensions.is_empty() {
            return Err(crate::error::RefactoringError::InvalidConfiguration(
                "At least one file extension must be specified".to_string(),
            ));
        }

        // Validate rules
        for rule in &self.rules {
            if rule.name.is_empty() {
                return Err(crate::error::RefactoringError::InvalidConfiguration(
                    "Rule name cannot be empty".to_string(),
                ));
            }

            if rule.pattern.is_empty() {
                return Err(crate::error::RefactoringError::InvalidConfiguration(
                    format!("Rule '{}' has empty pattern", rule.name),
                ));
            }
        }

        // Validate transformations
        for transformation in &self.transformations {
            if transformation.name.is_empty() {
                return Err(crate::error::RefactoringError::InvalidConfiguration(
                    "Transformation name cannot be empty".to_string(),
                ));
            }

            if transformation.from_pattern.is_empty() {
                return Err(crate::error::RefactoringError::InvalidConfiguration(
                    format!("Transformation '{}' has empty from_pattern", transformation.name),
                ));
            }

            if transformation.to_pattern.is_empty() {
                return Err(crate::error::RefactoringError::InvalidConfiguration(
                    format!("Transformation '{}' has empty to_pattern", transformation.name),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RefactoringType;

    #[test]
    fn test_validate_valid_config() -> crate::error::Result<()> {
        let config = LanguageConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            parser_plugin: Some("tree-sitter-rust".to_string()),
            rules: vec![RefactoringRule {
                name: "unused_variable".to_string(),
                pattern: "let \\w+ = .*;".to_string(),
                refactoring_type: RefactoringType::RemoveUnused,
                enabled: true,
            }],
            transformations: vec![],
        };

        config.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_empty_language() {
        let config = LanguageConfig {
            language: "".to_string(),
            extensions: vec![".rs".to_string()],
            parser_plugin: None,
            rules: vec![],
            transformations: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_extensions() {
        let config = LanguageConfig {
            language: "rust".to_string(),
            extensions: vec![],
            parser_plugin: None,
            rules: vec![],
            transformations: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_rule_name() {
        let config = LanguageConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            parser_plugin: None,
            rules: vec![RefactoringRule {
                name: "".to_string(),
                pattern: "pattern".to_string(),
                refactoring_type: RefactoringType::Rename,
                enabled: true,
            }],
            transformations: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_rule_pattern() {
        let config = LanguageConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            parser_plugin: None,
            rules: vec![RefactoringRule {
                name: "test".to_string(),
                pattern: "".to_string(),
                refactoring_type: RefactoringType::Rename,
                enabled: true,
            }],
            transformations: vec![],
        };

        assert!(config.validate().is_err());
    }
}
