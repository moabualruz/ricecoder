//! Template validation engine
//!
//! Validates template syntax, checks placeholder references,
//! and validates boilerplate structure.

use crate::models::{Boilerplate, BoilerplateFile, Template};
use crate::templates::error::{BoilerplateError, TemplateError};
use crate::templates::parser::{ParsedTemplate, TemplateElement, TemplateParser};
use std::collections::HashSet;

/// Template validation engine
pub struct ValidationEngine;

impl ValidationEngine {
    /// Validate template syntax
    ///
    /// # Arguments
    /// * `content` - Template content to validate
    ///
    /// # Returns
    /// Ok if valid, Err with line number if invalid
    pub fn validate_template_syntax(content: &str) -> Result<(), TemplateError> {
        TemplateParser::parse(content)?;
        Ok(())
    }

    /// Validate that all required placeholders are provided
    ///
    /// # Arguments
    /// * `template` - Template to validate
    /// * `provided_placeholders` - Set of provided placeholder names
    ///
    /// # Returns
    /// Ok if all required placeholders provided, Err otherwise
    pub fn validate_placeholder_references(
        template: &Template,
        provided_placeholders: &HashSet<String>,
    ) -> Result<(), TemplateError> {
        for placeholder in &template.placeholders {
            if placeholder.required && !provided_placeholders.contains(&placeholder.name) {
                return Err(TemplateError::MissingPlaceholder(placeholder.name.clone()));
            }
        }
        Ok(())
    }

    /// Validate boilerplate structure
    ///
    /// # Arguments
    /// * `boilerplate` - Boilerplate to validate
    ///
    /// # Returns
    /// Ok if valid, Err if structure is invalid
    pub fn validate_boilerplate_structure(boilerplate: &Boilerplate) -> Result<(), BoilerplateError> {
        // Check required fields
        if boilerplate.id.is_empty() {
            return Err(BoilerplateError::InvalidStructure(
                "Boilerplate ID cannot be empty".to_string(),
            ));
        }

        if boilerplate.name.is_empty() {
            return Err(BoilerplateError::InvalidStructure(
                "Boilerplate name cannot be empty".to_string(),
            ));
        }

        if boilerplate.language.is_empty() {
            return Err(BoilerplateError::InvalidStructure(
                "Boilerplate language cannot be empty".to_string(),
            ));
        }

        // Check files
        if boilerplate.files.is_empty() {
            return Err(BoilerplateError::InvalidStructure(
                "Boilerplate must have at least one file".to_string(),
            ));
        }

        // Validate each file
        for file in &boilerplate.files {
            Self::validate_boilerplate_file(file)?;
        }

        Ok(())
    }

    /// Validate a single boilerplate file
    fn validate_boilerplate_file(file: &BoilerplateFile) -> Result<(), BoilerplateError> {
        if file.path.is_empty() {
            return Err(BoilerplateError::InvalidStructure(
                "Boilerplate file path cannot be empty".to_string(),
            ));
        }

        if file.template.is_empty() {
            return Err(BoilerplateError::InvalidStructure(
                "Boilerplate file template cannot be empty".to_string(),
            ));
        }

        // Validate template syntax if it looks like a template
        if file.template.contains("{{") {
            ValidationEngine::validate_template_syntax(&file.template)
                .map_err(|e| BoilerplateError::InvalidStructure(format!("Invalid template in file {}: {}", file.path, e)))?;
        }

        Ok(())
    }

    /// Validate placeholder consistency in template
    ///
    /// # Arguments
    /// * `parsed_template` - Parsed template to validate
    ///
    /// # Returns
    /// Ok if consistent, Err if issues found
    pub fn validate_placeholder_consistency(parsed_template: &ParsedTemplate) -> Result<(), TemplateError> {
        // Check for duplicate placeholder definitions
        let mut seen = HashSet::new();
        for placeholder in &parsed_template.placeholders {
            if !seen.insert(&placeholder.name) {
                return Err(TemplateError::ValidationFailed(format!(
                    "Duplicate placeholder definition: {}",
                    placeholder.name
                )));
            }
        }

        Ok(())
    }

    /// Validate template block nesting
    ///
    /// # Arguments
    /// * `elements` - Template elements to validate
    ///
    /// # Returns
    /// Ok if nesting is valid, Err if issues found
    pub fn validate_block_nesting(elements: &[TemplateElement]) -> Result<(), TemplateError> {
        Self::validate_nesting_recursive(elements, 0)
    }

    fn validate_nesting_recursive(elements: &[TemplateElement], depth: usize) -> Result<(), TemplateError> {
        // Prevent excessive nesting (max 10 levels)
        if depth > 10 {
            return Err(TemplateError::ValidationFailed(
                "Template nesting too deep (max 10 levels)".to_string(),
            ));
        }

        for element in elements {
            match element {
                TemplateElement::Conditional { content, .. } | TemplateElement::Loop { content, .. } => {
                    Self::validate_nesting_recursive(content, depth + 1)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Validate that all referenced partials exist
    ///
    /// # Arguments
    /// * `elements` - Template elements
    /// * `available_partials` - Set of available partial names
    ///
    /// # Returns
    /// Ok if all partials exist, Err if missing
    pub fn validate_partial_references(
        elements: &[TemplateElement],
        available_partials: &HashSet<String>,
    ) -> Result<(), TemplateError> {
        Self::validate_partials_recursive(elements, available_partials)
    }

    fn validate_partials_recursive(
        elements: &[TemplateElement],
        available_partials: &HashSet<String>,
    ) -> Result<(), TemplateError> {
        for element in elements {
            match element {
                TemplateElement::Include(partial_name) => {
                    if !available_partials.contains(partial_name) {
                        return Err(TemplateError::ValidationFailed(format!(
                            "Referenced partial not found: {}",
                            partial_name
                        )));
                    }
                }
                TemplateElement::Conditional { content, .. } | TemplateElement::Loop { content, .. } => {
                    Self::validate_partials_recursive(content, available_partials)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Comprehensive template validation
    ///
    /// # Arguments
    /// * `template` - Template to validate
    /// * `provided_placeholders` - Set of provided placeholder names
    /// * `available_partials` - Set of available partial names
    ///
    /// # Returns
    /// Ok if all validations pass, Err if any fail
    pub fn validate_template_comprehensive(
        template: &Template,
        provided_placeholders: &HashSet<String>,
        available_partials: &HashSet<String>,
    ) -> Result<(), TemplateError> {
        // Validate syntax
        Self::validate_template_syntax(&template.content)?;

        // Parse template
        let parsed = TemplateParser::parse(&template.content)?;

        // Validate placeholder consistency
        Self::validate_placeholder_consistency(&parsed)?;

        // Validate block nesting
        Self::validate_block_nesting(&parsed.elements)?;

        // Validate partial references
        Self::validate_partial_references(&parsed.elements, available_partials)?;

        // Validate placeholder references
        Self::validate_placeholder_references(template, provided_placeholders)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Placeholder;

    #[test]
    fn test_validate_valid_template_syntax() {
        let content = "Hello {{name}}";
        assert!(ValidationEngine::validate_template_syntax(content).is_ok());
    }

    #[test]
    fn test_validate_invalid_template_syntax() {
        let content = "Hello {{name";
        assert!(ValidationEngine::validate_template_syntax(content).is_err());
    }

    #[test]
    fn test_validate_placeholder_references_all_provided() {
        let template = Template {
            id: "test".to_string(),
            name: "test".to_string(),
            language: "rust".to_string(),
            content: "{{name}}".to_string(),
            placeholders: vec![Placeholder {
                name: "name".to_string(),
                description: "Name".to_string(),
                default: None,
                required: true,
            }],
            metadata: Default::default(),
        };

        let mut provided = HashSet::new();
        provided.insert("name".to_string());

        assert!(ValidationEngine::validate_placeholder_references(&template, &provided).is_ok());
    }

    #[test]
    fn test_validate_placeholder_references_missing() {
        let template = Template {
            id: "test".to_string(),
            name: "test".to_string(),
            language: "rust".to_string(),
            content: "{{name}}".to_string(),
            placeholders: vec![Placeholder {
                name: "name".to_string(),
                description: "Name".to_string(),
                default: None,
                required: true,
            }],
            metadata: Default::default(),
        };

        let provided = HashSet::new();
        assert!(ValidationEngine::validate_placeholder_references(&template, &provided).is_err());
    }

    #[test]
    fn test_validate_boilerplate_structure_valid() {
        let boilerplate = Boilerplate {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {}".to_string(),
                condition: None,
            }],
            dependencies: vec![],
            scripts: vec![],
        };

        assert!(ValidationEngine::validate_boilerplate_structure(&boilerplate).is_ok());
    }

    #[test]
    fn test_validate_boilerplate_structure_empty_id() {
        let boilerplate = Boilerplate {
            id: "".to_string(),
            name: "Test".to_string(),
            description: "Test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {}".to_string(),
                condition: None,
            }],
            dependencies: vec![],
            scripts: vec![],
        };

        assert!(ValidationEngine::validate_boilerplate_structure(&boilerplate).is_err());
    }

    #[test]
    fn test_validate_boilerplate_structure_no_files() {
        let boilerplate = Boilerplate {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![],
            dependencies: vec![],
            scripts: vec![],
        };

        assert!(ValidationEngine::validate_boilerplate_structure(&boilerplate).is_err());
    }

    #[test]
    fn test_validate_block_nesting_valid() {
        let elements = vec![TemplateElement::Conditional {
            condition: "test".to_string(),
            content: vec![TemplateElement::Text("content".to_string())],
        }];

        assert!(ValidationEngine::validate_block_nesting(&elements).is_ok());
    }

    #[test]
    fn test_validate_partial_references_valid() {
        let elements = vec![TemplateElement::Include("header".to_string())];
        let mut available = HashSet::new();
        available.insert("header".to_string());

        assert!(ValidationEngine::validate_partial_references(&elements, &available).is_ok());
    }

    #[test]
    fn test_validate_partial_references_missing() {
        let elements = vec![TemplateElement::Include("missing".to_string())];
        let available = HashSet::new();

        assert!(ValidationEngine::validate_partial_references(&elements, &available).is_err());
    }
}
