//! YAML parser for frontmatter validation and deserialization

use crate::markdown_config::error::{MarkdownConfigError, MarkdownConfigResult};
use serde::de::DeserializeOwned;

/// Parser for YAML frontmatter
#[derive(Debug, Clone)]
pub struct YamlParser;

impl YamlParser {
    /// Create a new YAML parser
    pub fn new() -> Self {
        Self
    }

    /// Parse YAML string into a typed structure
    pub fn parse<T: DeserializeOwned>(&self, yaml: &str) -> MarkdownConfigResult<T> {
        serde_yaml::from_str(yaml).map_err(|e| {
            MarkdownConfigError::yaml_error(format!("Failed to parse YAML: {}", e))
        })
    }

    /// Validate YAML structure without deserializing to a specific type
    pub fn validate_structure(&self, yaml: &str) -> MarkdownConfigResult<()> {
        // Try to parse as a generic YAML value to validate structure
        serde_yaml::from_str::<serde_yaml::Value>(yaml).map_err(|e| {
            MarkdownConfigError::yaml_error(format!("Invalid YAML structure: {}", e))
        })?;
        Ok(())
    }

    /// Check if required fields are present in YAML
    pub fn has_required_fields(&self, yaml: &str, required_fields: &[&str]) -> MarkdownConfigResult<()> {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).map_err(|e| {
            MarkdownConfigError::yaml_error(format!("Failed to parse YAML: {}", e))
        })?;

        let mapping = value.as_mapping().ok_or_else(|| {
            MarkdownConfigError::validation_error("YAML must be a mapping (object)")
        })?;

        for field in required_fields {
            let key = serde_yaml::Value::String(field.to_string());
            if !mapping.contains_key(&key) {
                return Err(MarkdownConfigError::missing_field(*field));
            }
        }

        Ok(())
    }

    /// Get a field value from YAML
    pub fn get_field(&self, yaml: &str, field: &str) -> MarkdownConfigResult<Option<String>> {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).map_err(|e| {
            MarkdownConfigError::yaml_error(format!("Failed to parse YAML: {}", e))
        })?;

        let mapping = value.as_mapping().ok_or_else(|| {
            MarkdownConfigError::validation_error("YAML must be a mapping (object)")
        })?;

        let key = serde_yaml::Value::String(field.to_string());
        Ok(mapping.get(&key).and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    /// Validate YAML against a schema (checks for required fields and types)
    pub fn validate_schema(
        &self,
        yaml: &str,
        required_fields: &[&str],
    ) -> MarkdownConfigResult<()> {
        // First validate structure
        self.validate_structure(yaml)?;

        // Then check required fields
        self.has_required_fields(yaml, required_fields)?;

        Ok(())
    }

    /// Get all validation errors from YAML
    pub fn get_all_validation_errors(
        &self,
        yaml: &str,
        required_fields: &[&str],
    ) -> Vec<MarkdownConfigError> {
        let mut errors = Vec::new();

        // Check structure
        if let Err(e) = self.validate_structure(yaml) {
            errors.push(e);
            return errors; // Can't check fields if structure is invalid
        }

        // Check required fields
        for field in required_fields {
            if let Err(e) = self.has_required_fields(yaml, &[field]) {
                errors.push(e);
            }
        }

        errors
    }
}

impl Default for YamlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    #[test]
    fn test_parse_valid_yaml() {
        let parser = YamlParser::new();
        let yaml = "name: test\nvalue: 42";

        let result: TestConfig = parser.parse(yaml).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let parser = YamlParser::new();
        let yaml = "name: test\n  invalid: [unclosed";

        let result: Result<TestConfig, _> = parser.parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_structure_valid() {
        let parser = YamlParser::new();
        let yaml = "name: test\nvalue: 42";

        let result = parser.validate_structure(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_structure_invalid() {
        let parser = YamlParser::new();
        let yaml = "name: test\n  invalid: [unclosed";

        let result = parser.validate_structure(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_has_required_fields_present() {
        let parser = YamlParser::new();
        let yaml = "name: test\nvalue: 42\ndescription: optional";

        let result = parser.has_required_fields(yaml, &["name", "value"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_required_fields_missing() {
        let parser = YamlParser::new();
        let yaml = "name: test";

        let result = parser.has_required_fields(yaml, &["name", "value"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_field_exists() {
        let parser = YamlParser::new();
        let yaml = "name: test-value\nother: data";

        let result = parser.get_field(yaml, "name").unwrap();
        assert_eq!(result, Some("test-value".to_string()));
    }

    #[test]
    fn test_get_field_missing() {
        let parser = YamlParser::new();
        let yaml = "name: test-value";

        let result = parser.get_field(yaml, "missing").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_field_non_string() {
        let parser = YamlParser::new();
        let yaml = "name: test\nvalue: 42";

        let result = parser.get_field(yaml, "value").unwrap();
        assert_eq!(result, None); // Non-string values return None
    }
}
