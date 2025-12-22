//! Placeholder resolution and case transformation

use std::collections::{HashMap, HashSet};

use crate::templates::error::TemplateError;

/// Represents a case transformation for placeholder values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseTransform {
    /// PascalCase (e.g., MyProject)
    PascalCase,
    /// camelCase (e.g., myProject)
    CamelCase,
    /// snake_case (e.g., my_project)
    SnakeCase,
    /// kebab-case (e.g., my-project)
    KebabCase,
    /// UPPERCASE (e.g., MY_PROJECT)
    UpperCase,
    /// lowercase (e.g., myproject)
    LowerCase,
}

impl CaseTransform {
    /// Apply case transformation to a string
    pub fn apply(&self, input: &str) -> String {
        use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};

        match self {
            CaseTransform::PascalCase => input.to_pascal_case(),
            CaseTransform::CamelCase => input.to_lower_camel_case(),
            CaseTransform::SnakeCase => input.to_snake_case(),
            CaseTransform::KebabCase => input.to_kebab_case(),
            CaseTransform::UpperCase => input.to_uppercase(),
            CaseTransform::LowerCase => input.to_lowercase(),
        }
    }
}

/// Represents a placeholder in a template
#[derive(Debug, Clone)]
pub struct Placeholder {
    /// The base name of the placeholder (without case suffix)
    pub name: String,
    /// The case transformation to apply
    pub case_transform: CaseTransform,
    /// Whether this placeholder is required
    pub required: bool,
    /// Default value if not provided
    pub default: Option<String>,
}

impl Placeholder {
    /// Create a new placeholder with the given name and case transform
    pub fn new(name: String, case_transform: CaseTransform) -> Self {
        Self {
            name,
            case_transform,
            required: true,
            default: None,
        }
    }

    /// Set whether this placeholder is required
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set a default value for this placeholder
    pub fn with_default(mut self, default: String) -> Self {
        self.default = Some(default);
        self.required = false;
        self
    }
}

/// Resolves placeholders in templates with case transformations
pub struct PlaceholderResolver {
    /// Mapping of placeholder names to their values
    values: HashMap<String, String>,
    /// Set of required placeholders
    required: HashSet<String>,
    /// Maximum depth for nested placeholder resolution
    max_depth: usize,
}

impl PlaceholderResolver {
    /// Create a new placeholder resolver
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            required: HashSet::new(),
            max_depth: 10,
        }
    }

    /// Set the maximum depth for nested placeholder resolution
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Add a value for a placeholder
    pub fn add_value(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.values.insert(name.into(), value.into());
    }

    /// Add multiple values at once
    pub fn add_values(&mut self, values: HashMap<String, String>) {
        self.values.extend(values);
    }

    /// Mark a placeholder as required
    pub fn require(&mut self, name: impl Into<String>) {
        self.required.insert(name.into());
    }

    /// Resolve a placeholder with the given case transformation
    pub fn resolve(
        &self,
        name: &str,
        case_transform: CaseTransform,
    ) -> Result<String, TemplateError> {
        let value = self
            .values
            .get(name)
            .ok_or_else(|| TemplateError::MissingPlaceholder(name.to_string()))?;

        Ok(case_transform.apply(value))
    }

    /// Resolve a placeholder with optional default value
    pub fn resolve_with_default(
        &self,
        name: &str,
        case_transform: CaseTransform,
        default: Option<&str>,
    ) -> Result<String, TemplateError> {
        match self.values.get(name) {
            Some(value) => Ok(case_transform.apply(value)),
            None => {
                if let Some(default_val) = default {
                    Ok(case_transform.apply(default_val))
                } else {
                    Err(TemplateError::MissingPlaceholder(name.to_string()))
                }
            }
        }
    }

    /// Validate that all required placeholders are provided
    pub fn validate(&self) -> Result<(), TemplateError> {
        for required_name in &self.required {
            if !self.values.contains_key(required_name) {
                return Err(TemplateError::MissingPlaceholder(required_name.clone()));
            }
        }
        Ok(())
    }

    /// Get all placeholder names that have been provided
    pub fn provided_names(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    /// Check if a placeholder value is provided
    pub fn has_value(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// Extract placeholder names from a template string
    /// Returns a vector of placeholder names found in the template
    pub fn extract_placeholder_names(&self, template: &str) -> Vec<String> {
        let mut names = Vec::new();
        let mut start = 0;

        while let Some(pos) = template[start..].find("{{") {
            let start_pos = start + pos;
            if let Some(end_pos) = template[start_pos..].find("}}") {
                let end_pos = start_pos + end_pos;
                let content = &template[start_pos + 2..end_pos];

                // Parse the placeholder to extract the name
                if let Ok((name, _)) = self.parse_placeholder_syntax(content) {
                    names.push(name);
                }

                start = end_pos + 2;
            } else {
                break;
            }
        }

        names
    }

    /// Resolve a placeholder with nested resolution support
    /// Supports values that reference other placeholders (e.g., "{{other_name}}")
    pub fn resolve_nested(
        &self,
        name: &str,
        case_transform: CaseTransform,
    ) -> Result<String, TemplateError> {
        self.resolve_nested_internal(name, case_transform, 0, &mut HashSet::new())
    }

    /// Internal recursive implementation for nested resolution with circular reference detection
    fn resolve_nested_internal(
        &self,
        name: &str,
        case_transform: CaseTransform,
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> Result<String, TemplateError> {
        // Check depth limit first
        if depth >= self.max_depth {
            return Err(TemplateError::RenderError(format!(
                "Maximum nesting depth ({}) exceeded for placeholder: {}",
                self.max_depth, name
            )));
        }

        // Check for circular references
        if visited.contains(name) {
            return Err(TemplateError::RenderError(format!(
                "Circular reference detected for placeholder: {}",
                name
            )));
        }

        let value = self
            .values
            .get(name)
            .ok_or_else(|| TemplateError::MissingPlaceholder(name.to_string()))?;

        // Check if value contains nested placeholders
        if value.contains("{{") && value.contains("}}") {
            visited.insert(name.to_string());
            let resolved = self.resolve_nested_value(value, depth + 1, visited)?;
            visited.remove(name);
            Ok(case_transform.apply(&resolved))
        } else {
            Ok(case_transform.apply(value))
        }
    }

    /// Resolve nested placeholders within a value string
    fn resolve_nested_value(
        &self,
        value: &str,
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> Result<String, TemplateError> {
        let mut result = value.to_string();
        let mut changed = true;

        // Keep resolving until no more placeholders are found
        while changed && depth <= self.max_depth {
            changed = false;

            // Find and replace placeholders
            if let Some(start) = result.find("{{") {
                if let Some(end) = result[start..].find("}}") {
                    let end = start + end;
                    let placeholder_content = &result[start + 2..end];

                    // Extract placeholder name and case transform
                    let (placeholder_name, case_transform) =
                        self.parse_placeholder_syntax(placeholder_content)?;

                    // Resolve the nested placeholder
                    let resolved = self.resolve_nested_internal(
                        &placeholder_name,
                        case_transform,
                        depth + 1,
                        visited,
                    )?;

                    // Replace the placeholder with its resolved value
                    result.replace_range(start..=end, &resolved);
                    changed = true;
                }
            }
        }

        Ok(result)
    }

    /// Parse placeholder syntax to extract name and case transform
    /// Supports formats like: "name", "Name", "NAME", "name_snake", "name-kebab", "nameCamel"
    fn parse_placeholder_syntax(
        &self,
        content: &str,
    ) -> Result<(String, CaseTransform), TemplateError> {
        let content = content.trim();

        // Determine case transform based on suffix
        if content.ends_with("_snake") {
            let name = content.trim_end_matches("_snake").to_string();
            Ok((name, CaseTransform::SnakeCase))
        } else if content.ends_with("-kebab") {
            let name = content.trim_end_matches("-kebab").to_string();
            Ok((name, CaseTransform::KebabCase))
        } else if content.ends_with("Camel") {
            let name = content.trim_end_matches("Camel").to_string();
            Ok((name, CaseTransform::CamelCase))
        } else if content.chars().all(|c| c.is_uppercase() || c == '_') && content.len() > 1 {
            // All uppercase = UPPERCASE transform
            Ok((content.to_string(), CaseTransform::UpperCase))
        } else if content.chars().next().is_some_and(|c| c.is_uppercase()) {
            // Starts with uppercase = PascalCase
            Ok((content.to_string(), CaseTransform::PascalCase))
        } else {
            // Default to lowercase
            Ok((content.to_string(), CaseTransform::LowerCase))
        }
    }
}

impl Default for PlaceholderResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_transform_pascal_case() {
        let result = CaseTransform::PascalCase.apply("my_project");
        assert_eq!(result, "MyProject");
    }

    #[test]
    fn test_case_transform_camel_case() {
        let result = CaseTransform::CamelCase.apply("my_project");
        assert_eq!(result, "myProject");
    }

    #[test]
    fn test_case_transform_snake_case() {
        let result = CaseTransform::SnakeCase.apply("MyProject");
        assert_eq!(result, "my_project");
    }

    #[test]
    fn test_case_transform_kebab_case() {
        let result = CaseTransform::KebabCase.apply("MyProject");
        assert_eq!(result, "my-project");
    }

    #[test]
    fn test_case_transform_upper_case() {
        let result = CaseTransform::UpperCase.apply("my_project");
        assert_eq!(result, "MY_PROJECT");
    }

    #[test]
    fn test_case_transform_lower_case() {
        let result = CaseTransform::LowerCase.apply("MyProject");
        assert_eq!(result, "myproject");
    }

    #[test]
    fn test_placeholder_resolver_add_value() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("name", "my_project");
        assert!(resolver.has_value("name"));
    }

    #[test]
    fn test_placeholder_resolver_resolve() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("name", "my_project");
        let result = resolver.resolve("name", CaseTransform::PascalCase);
        assert_eq!(result.unwrap(), "MyProject");
    }

    #[test]
    fn test_placeholder_resolver_missing_placeholder() {
        let resolver = PlaceholderResolver::new();
        let result = resolver.resolve("name", CaseTransform::PascalCase);
        assert!(result.is_err());
    }

    #[test]
    fn test_placeholder_resolver_validate_required() {
        let mut resolver = PlaceholderResolver::new();
        resolver.require("name");
        let result = resolver.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_placeholder_resolver_validate_success() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("name", "my_project");
        resolver.require("name");
        let result = resolver.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_placeholder_resolver_with_default() {
        let resolver = PlaceholderResolver::new();
        let result =
            resolver.resolve_with_default("name", CaseTransform::PascalCase, Some("default_value"));
        assert_eq!(result.unwrap(), "DefaultValue");
    }

    #[test]
    fn test_placeholder_resolver_add_multiple_values() {
        let mut resolver = PlaceholderResolver::new();
        let mut values = HashMap::new();
        values.insert("name".to_string(), "my_project".to_string());
        values.insert("author".to_string(), "john_doe".to_string());
        resolver.add_values(values);
        assert!(resolver.has_value("name"));
        assert!(resolver.has_value("author"));
    }

    #[test]
    fn test_case_transform_with_numbers() {
        let result = CaseTransform::PascalCase.apply("my_project_2");
        assert_eq!(result, "MyProject2");
    }

    #[test]
    fn test_case_transform_with_acronyms() {
        let result = CaseTransform::SnakeCase.apply("HTTPServer");
        assert_eq!(result, "http_server");
    }

    #[test]
    fn test_case_transform_single_word() {
        let result = CaseTransform::PascalCase.apply("project");
        assert_eq!(result, "Project");
    }

    #[test]
    fn test_case_transform_already_pascal() {
        let result = CaseTransform::PascalCase.apply("MyProject");
        assert_eq!(result, "MyProject");
    }

    #[test]
    fn test_case_transform_kebab_to_pascal() {
        let result = CaseTransform::PascalCase.apply("my-project");
        assert_eq!(result, "MyProject");
    }

    #[test]
    fn test_case_transform_empty_string() {
        let result = CaseTransform::PascalCase.apply("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_nested_placeholder_resolution() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("base", "my_project");
        resolver.add_value("full_name", "{{base}}_extended");
        let result = resolver.resolve_nested("full_name", CaseTransform::SnakeCase);
        assert_eq!(result.unwrap(), "my_project_extended");
    }

    #[test]
    fn test_nested_placeholder_with_case_transform() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("base", "my_project");
        resolver.add_value("full_name", "{{base}}_extended");
        let result = resolver.resolve_nested("full_name", CaseTransform::PascalCase);
        assert_eq!(result.unwrap(), "MyProjectExtended");
    }

    #[test]
    fn test_circular_reference_detection() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("a", "{{b}}");
        resolver.add_value("b", "{{a}}");
        let result = resolver.resolve_nested("a", CaseTransform::LowerCase);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_reference_detection() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("a", "{{a}}");
        let result = resolver.resolve_nested("a", CaseTransform::LowerCase);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_depth_exceeded() {
        let mut resolver = PlaceholderResolver::new().with_max_depth(2);
        resolver.add_value("a", "{{b}}");
        resolver.add_value("b", "{{c}}");
        resolver.add_value("c", "{{d}}");
        resolver.add_value("d", "value");
        let result = resolver.resolve_nested("a", CaseTransform::LowerCase);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_nested_placeholders() {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value("first", "hello");
        resolver.add_value("second", "world");
        resolver.add_value("combined", "{{first}}_{{second}}");
        let result = resolver.resolve_nested("combined", CaseTransform::SnakeCase);
        assert_eq!(result.unwrap(), "hello_world");
    }

    #[test]
    fn test_parse_placeholder_syntax_pascal_case() {
        let resolver = PlaceholderResolver::new();
        let (name, transform) = resolver.parse_placeholder_syntax("Name").unwrap();
        assert_eq!(name, "Name");
        assert_eq!(transform, CaseTransform::PascalCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_snake_case() {
        let resolver = PlaceholderResolver::new();
        let (name, transform) = resolver.parse_placeholder_syntax("name_snake").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::SnakeCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_kebab_case() {
        let resolver = PlaceholderResolver::new();
        let (name, transform) = resolver.parse_placeholder_syntax("name-kebab").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::KebabCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_camel_case() {
        let resolver = PlaceholderResolver::new();
        let (name, transform) = resolver.parse_placeholder_syntax("nameCamel").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::CamelCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_uppercase() {
        let resolver = PlaceholderResolver::new();
        let (name, transform) = resolver.parse_placeholder_syntax("NAME").unwrap();
        assert_eq!(name, "NAME");
        assert_eq!(transform, CaseTransform::UpperCase);
    }

    #[test]
    fn test_parse_placeholder_syntax_lowercase() {
        let resolver = PlaceholderResolver::new();
        let (name, transform) = resolver.parse_placeholder_syntax("name").unwrap();
        assert_eq!(name, "name");
        assert_eq!(transform, CaseTransform::LowerCase);
    }
}
