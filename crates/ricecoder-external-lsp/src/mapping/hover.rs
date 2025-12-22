//! Hover output mapping
//!
//! Maps LSP hover responses to ricecoder HoverInfo models.
//! Supports custom field mappings via configuration and transformation functions.

use serde_json::Value;

use super::transformer::OutputTransformer;
use crate::{error::Result, types::HoverMappingRules};

/// Maps LSP hover responses to ricecoder models
#[derive(Debug, Clone)]
pub struct HoverMapper {
    transformer: OutputTransformer,
}

impl HoverMapper {
    /// Create a new hover mapper
    pub fn new() -> Self {
        Self {
            transformer: OutputTransformer::new(),
        }
    }

    /// Create a mapper with custom transformations
    pub fn with_transformer(transformer: OutputTransformer) -> Self {
        Self { transformer }
    }

    /// Map an LSP hover response to ricecoder models
    ///
    /// # Arguments
    ///
    /// * `response` - The LSP server response (typically from textDocument/hover)
    /// * `rules` - The mapping rules from configuration
    ///
    /// # Returns
    ///
    /// The mapped hover information
    pub fn map(&self, response: &Value, rules: &HoverMappingRules) -> Result<Value> {
        self.transformer.transform_hover(response, rules)
    }

    /// Map hover content directly
    ///
    /// This is useful when you already have the hover content extracted
    /// and just need to apply field mappings.
    pub fn map_content(&self, content: &Value, rules: &HoverMappingRules) -> Result<Value> {
        // Create a wrapper response with the content
        let wrapped = serde_json::json!({
            "result": {
                "contents": content
            }
        });

        // Use default rules that expect this structure
        let default_rules = HoverMappingRules {
            content_path: "$.result.contents".to_string(),
            field_mappings: rules.field_mappings.clone(),
            transform: rules.transform.clone(),
        };

        self.transformer.transform_hover(&wrapped, &default_rules)
    }
}

impl Default for HoverMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_map_hover_response() {
        let mapper = HoverMapper::new();
        let response = serde_json::json!({
            "result": {
                "contents": {
                    "language": "rust",
                    "value": "fn foo() -> i32"
                }
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("language".to_string(), "$.language".to_string());
        field_mappings.insert("value".to_string(), "$.value".to_string());

        let rules = HoverMappingRules {
            content_path: "$.result.contents".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&response, &rules).unwrap();
        assert_eq!(result["language"], "rust");
        assert_eq!(result["value"], "fn foo() -> i32");
    }

    #[test]
    fn test_map_hover_with_custom_structure() {
        let mapper = HoverMapper::new();
        let response = serde_json::json!({
            "hover_info": {
                "doc": "A function that returns an integer",
                "signature": "fn foo() -> i32"
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("documentation".to_string(), "$.doc".to_string());
        field_mappings.insert("signature".to_string(), "$.signature".to_string());

        let rules = HoverMappingRules {
            content_path: "$.hover_info".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&response, &rules).unwrap();
        assert_eq!(
            result["documentation"],
            "A function that returns an integer"
        );
        assert_eq!(result["signature"], "fn foo() -> i32");
    }

    #[test]
    fn test_map_hover_content() {
        let mapper = HoverMapper::new();
        let content = serde_json::json!({
            "language": "python",
            "value": "def bar(): pass"
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("language".to_string(), "$.language".to_string());
        field_mappings.insert("value".to_string(), "$.value".to_string());

        let rules = HoverMappingRules {
            content_path: "$.result.contents".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map_content(&content, &rules).unwrap();
        assert_eq!(result["language"], "python");
        assert_eq!(result["value"], "def bar(): pass");
    }

    #[test]
    fn test_map_hover_with_markdown() {
        let mapper = HoverMapper::new();
        let response = serde_json::json!({
            "result": {
                "contents": {
                    "kind": "markdown",
                    "value": "# Function\n\nThis is a function"
                }
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("kind".to_string(), "$.kind".to_string());
        field_mappings.insert("value".to_string(), "$.value".to_string());

        let rules = HoverMappingRules {
            content_path: "$.result.contents".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&response, &rules).unwrap();
        assert_eq!(result["kind"], "markdown");
        assert!(result["value"].as_str().unwrap().contains("Function"));
    }

    #[test]
    fn test_map_hover_missing_field() {
        let mapper = HoverMapper::new();
        let response = serde_json::json!({
            "result": {
                "contents": {
                    "value": "fn foo() -> i32"
                }
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("language".to_string(), "$.language".to_string()); // Missing
        field_mappings.insert("value".to_string(), "$.value".to_string());

        let rules = HoverMappingRules {
            content_path: "$.result.contents".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&response, &rules).unwrap();
        assert_eq!(result["value"], "fn foo() -> i32");
        // Missing field should not be in result
        assert!(!result.get("language").is_some() || result["language"].is_null());
    }
}
