//! Template engine for rendering templates with variable substitution
//!
//! Provides template rendering with support for:
//! - Variable substitution with case transformations
//! - Conditional blocks ({{#if}}...{{/if}})
//! - Loops ({{#each}}...{{/each}})
//! - Includes/partials ({{> partial}})

use std::collections::HashMap;

use crate::{
    models::{RenderResult, TemplateContext},
    templates::{
        error::TemplateError,
        parser::{TemplateElement, TemplateParser},
        resolver::{CaseTransform, PlaceholderResolver},
    },
};

/// Template engine for rendering templates with variable substitution
pub struct TemplateEngine {
    /// Placeholder resolver for case transformations
    resolver: PlaceholderResolver,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            resolver: PlaceholderResolver::new(),
        }
    }

    /// Create a new template engine with custom resolver
    pub fn with_resolver(resolver: PlaceholderResolver) -> Self {
        Self { resolver }
    }

    /// Add a value for placeholder substitution
    pub fn add_value(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.resolver.add_value(name, value);
    }

    /// Add multiple values at once
    pub fn add_values(&mut self, values: HashMap<String, String>) {
        self.resolver.add_values(values);
    }

    /// Mark a placeholder as required
    pub fn require(&mut self, name: impl Into<String>) {
        self.resolver.require(name);
    }

    /// Render a template with the provided context
    ///
    /// # Arguments
    /// * `template_content` - The template content to render
    /// * `context` - The rendering context with values and options
    ///
    /// # Returns
    /// Rendered content or error
    pub fn render(
        &self,
        template_content: &str,
        context: &TemplateContext,
    ) -> Result<RenderResult, TemplateError> {
        // Parse the template
        let parsed = TemplateParser::parse(template_content)?;

        // Validate all required placeholders are provided
        self.resolver.validate()?;

        // Render the template
        let rendered = self.render_elements(&parsed.elements, context)?;

        Ok(RenderResult {
            content: rendered,
            warnings: Vec::new(),
            placeholders_used: self.resolver.provided_names(),
        })
    }

    /// Render template elements recursively
    fn render_elements(
        &self,
        elements: &[TemplateElement],
        context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        let mut result = String::new();

        for element in elements {
            match element {
                TemplateElement::Text(text) => {
                    result.push_str(text);
                }
                TemplateElement::Placeholder(placeholder_name) => {
                    let rendered = self.render_placeholder(placeholder_name, context)?;
                    result.push_str(&rendered);
                }
                TemplateElement::Conditional { condition, content } => {
                    if self.evaluate_condition(condition, context)? {
                        let rendered = self.render_elements(content, context)?;
                        result.push_str(&rendered);
                    }
                }
                TemplateElement::Loop { variable, content } => {
                    let rendered = self.render_loop(variable, content, context)?;
                    result.push_str(&rendered);
                }
                TemplateElement::Include(partial_name) => {
                    // For now, includes are not supported in the basic implementation
                    // This would require a template loader
                    return Err(TemplateError::RenderError(format!(
                        "Includes not yet supported: {}",
                        partial_name
                    )));
                }
            }
        }

        Ok(result)
    }

    /// Render a placeholder with case transformation
    fn render_placeholder(
        &self,
        placeholder_name: &str,
        _context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        // Parse the placeholder syntax to extract name and case transform
        let (name, case_transform) = self.parse_placeholder_syntax(placeholder_name)?;

        // Resolve the placeholder with the case transformation
        self.resolver.resolve(&name, case_transform)
    }

    /// Parse placeholder syntax to extract name and case transform
    fn parse_placeholder_syntax(
        &self,
        content: &str,
    ) -> Result<(String, CaseTransform), TemplateError> {
        let content = content.trim();

        // Determine case transform based on suffix
        if content.ends_with("_snake") {
            let name = content.trim_end_matches("_snake").to_lowercase();
            Ok((name, CaseTransform::SnakeCase))
        } else if content.ends_with("-kebab") {
            let name = content.trim_end_matches("-kebab").to_lowercase();
            Ok((name, CaseTransform::KebabCase))
        } else if content.ends_with("Camel") {
            let name = content.trim_end_matches("Camel").to_lowercase();
            Ok((name, CaseTransform::CamelCase))
        } else if content.chars().all(|c| c.is_uppercase() || c == '_') && content.len() > 1 {
            // All uppercase = UPPERCASE transform
            let name = content.to_lowercase();
            Ok((name, CaseTransform::UpperCase))
        } else if content.chars().next().is_some_and(|c| c.is_uppercase()) {
            // Starts with uppercase = PascalCase
            let name = content.to_lowercase();
            Ok((name, CaseTransform::PascalCase))
        } else {
            // Default to lowercase
            Ok((content.to_string(), CaseTransform::LowerCase))
        }
    }

    /// Evaluate a condition expression
    fn evaluate_condition(
        &self,
        condition: &str,
        _context: &TemplateContext,
    ) -> Result<bool, TemplateError> {
        // Simple condition evaluation
        // For now, just check if the condition is a non-empty placeholder name
        let condition = condition.trim();

        // Check if placeholder exists and has a truthy value
        if self.resolver.has_value(condition) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Render a loop block
    fn render_loop(
        &self,
        variable: &str,
        _content: &[TemplateElement],
        _context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        // For now, loops are not fully supported
        // This would require iterating over array values
        Err(TemplateError::RenderError(format!(
            "Loops not yet fully supported: {}",
            variable
        )))
    }

    /// Render a template string directly with simple variable substitution
    ///
    /// This is a simpler alternative to the full render method that just does
    /// basic placeholder substitution without parsing complex template syntax.
    pub fn render_simple(&self, template_content: &str) -> Result<String, TemplateError> {
        let mut result = template_content.to_string();

        // Find and replace all placeholders
        while let Some(start) = result.find("{{") {
            if let Some(end_offset) = result[start..].find("}}") {
                let end = start + end_offset + 1; // +1 to include the second }
                let placeholder_content = &result[start + 2..start + end_offset];

                // Parse placeholder syntax
                let (name, case_transform) = self.parse_placeholder_syntax(placeholder_content)?;

                // Resolve the placeholder
                let resolved = self.resolver.resolve(&name, case_transform)?;

                // Replace the placeholder (including the {{ and }})
                result.replace_range(start..=end, &resolved);
            } else {
                break;
            }
        }

        Ok(result)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RenderOptions;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert_eq!(engine.resolver.provided_names().len(), 0);
    }

    #[test]
    fn test_add_value() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");
        assert_eq!(engine.resolver.provided_names().len(), 1);
    }

    #[test]
    fn test_render_simple_placeholder() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");
        let result = engine.render_simple("Hello {{name}}");
        assert_eq!(result.unwrap(), "Hello my_project");
    }

    #[test]
    fn test_render_simple_pascal_case() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");
        let result = engine.render_simple("struct {{Name}} {}");
        assert_eq!(result.unwrap(), "struct MyProject {}");
    }

    #[test]
    fn test_render_simple_snake_case() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "MyProject");
        let result = engine.render_simple("let {{name_snake}} = 42;");
        assert_eq!(result.unwrap(), "let my_project = 42;");
    }

    #[test]
    fn test_render_simple_kebab_case() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "MyProject");
        let result = engine.render_simple("package-name: {{name-kebab}}");
        assert_eq!(result.unwrap(), "package-name: my-project");
    }

    #[test]
    fn test_render_simple_uppercase() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");
        let result = engine.render_simple("const {{NAME}} = 1;");
        assert_eq!(result.unwrap(), "const MY_PROJECT = 1;");
    }

    #[test]
    fn test_render_simple_camel_case() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");
        let result = engine.render_simple("function {{nameCamel}}() {}");
        assert_eq!(result.unwrap(), "function myProject() {}");
    }

    #[test]
    fn test_render_simple_multiple_placeholders() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");
        engine.add_value("author", "john_doe");
        let result = engine.render_simple("Project: {{Name}}, Author: {{author}}");
        assert_eq!(result.unwrap(), "Project: MyProject, Author: john_doe");
    }

    #[test]
    fn test_render_simple_missing_placeholder() {
        let engine = TemplateEngine::new();
        let result = engine.render_simple("Hello {{name}}");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_placeholder_syntax_pascal_case() {
        let engine = TemplateEngine::new();
        let (name, transform) = engine.parse_placeholder_syntax("Name").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::PascalCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_snake_case() {
        let engine = TemplateEngine::new();
        let (name, transform) = engine.parse_placeholder_syntax("name_snake").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::SnakeCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_kebab_case() {
        let engine = TemplateEngine::new();
        let (name, transform) = engine.parse_placeholder_syntax("name-kebab").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::KebabCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_uppercase() {
        let engine = TemplateEngine::new();
        let (name, transform) = engine.parse_placeholder_syntax("NAME").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::UpperCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_camel_case() {
        let engine = TemplateEngine::new();
        let (name, transform) = engine.parse_placeholder_syntax("nameCamel").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::CamelCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_lowercase() {
        let engine = TemplateEngine::new();
        let (name, transform) = engine.parse_placeholder_syntax("name").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::LowerCase);
    }

    #[test]
    fn test_render_with_context() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");

        let context = TemplateContext {
            values: Default::default(),
            options: RenderOptions::default(),
        };

        let result = engine.render("Hello {{name}}", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello my_project");
    }

    #[test]
    fn test_render_result_includes_placeholders_used() {
        let mut engine = TemplateEngine::new();
        engine.add_value("name", "my_project");

        let context = TemplateContext {
            values: Default::default(),
            options: RenderOptions::default(),
        };

        let result = engine.render("Hello {{name}}", &context).unwrap();
        assert!(result.placeholders_used.contains(&"name".to_string()));
    }
}
