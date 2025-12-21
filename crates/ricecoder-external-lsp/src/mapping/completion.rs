//! Completion output mapping
//!
//! Maps LSP completion responses to ricecoder CompletionItem models.
//! Supports custom field mappings via configuration and transformation functions.

use crate::error::Result;
use crate::types::CompletionMappingRules;
use serde_json::Value;

use super::transformer::OutputTransformer;

/// Maps LSP completion responses to ricecoder models
#[derive(Debug, Clone)]
pub struct CompletionMapper {
    transformer: OutputTransformer,
}

impl CompletionMapper {
    /// Create a new completion mapper
    pub fn new() -> Self {
        Self {
            transformer: OutputTransformer::new(),
        }
    }

    /// Create a mapper with custom transformations
    pub fn with_transformer(transformer: OutputTransformer) -> Self {
        Self { transformer }
    }

    /// Map an LSP completion response to ricecoder models
    ///
    /// # Arguments
    ///
    /// * `response` - The LSP server response (typically from textDocument/completion)
    /// * `rules` - The mapping rules from configuration
    ///
    /// # Returns
    ///
    /// A vector of mapped completion items
    pub fn map(&self, response: &Value, rules: &CompletionMappingRules) -> Result<Vec<Value>> {
        self.transformer.transform_completion(response, rules)
    }

    /// Map a single completion item
    ///
    /// This is useful for mapping individual items when the response structure
    /// doesn't match the expected array format.
    pub fn map_item(&self, item: &Value, rules: &CompletionMappingRules) -> Result<Value> {
        // Create a wrapper response with the item
        let wrapped = serde_json::json!({
            "result": {
                "items": [item]
            }
        });

        // Use default rules that expect this structure
        let default_rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings: rules.field_mappings.clone(),
            transform: rules.transform.clone(),
        };

        let results = self
            .transformer
            .transform_completion(&wrapped, &default_rules)?;

        if results.is_empty() {
            return Err(crate::error::ExternalLspError::TransformationError(
                "Failed to map completion item".to_string(),
            ));
        }

        Ok(results[0].clone())
    }
}

impl Default for CompletionMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_map_completion_response() {
        let mapper = CompletionMapper::new();
        let response = serde_json::json!({
            "result": {
                "items": [
                    {
                        "label": "foo",
                        "kind": 12,
                        "detail": "function",
                        "documentation": "A foo function"
                    },
                    {
                        "label": "bar",
                        "kind": 13,
                        "detail": "variable",
                        "documentation": "A bar variable"
                    }
                ]
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("kind".to_string(), "$.kind".to_string());
        field_mappings.insert("detail".to_string(), "$.detail".to_string());

        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let results = mapper.map(&response, &rules).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["label"], "foo");
        assert_eq!(results[1]["label"], "bar");
    }

    #[test]
    fn test_map_completion_with_custom_structure() {
        let mapper = CompletionMapper::new();
        let response = serde_json::json!({
            "completions": [
                {"name": "foo", "type": "function"},
                {"name": "bar", "type": "variable"}
            ]
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.name".to_string());
        field_mappings.insert("kind".to_string(), "$.type".to_string());

        let rules = CompletionMappingRules {
            items_path: "$.completions".to_string(),
            field_mappings,
            transform: None,
        };

        let results = mapper.map(&response, &rules).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["label"], "foo");
        assert_eq!(results[0]["kind"], "function");
    }

    #[test]
    fn test_map_single_item() {
        let mapper = CompletionMapper::new();
        let item = serde_json::json!({
            "label": "test",
            "kind": 12,
            "detail": "test function"
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("kind".to_string(), "$.kind".to_string());

        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map_item(&item, &rules).unwrap();
        assert_eq!(result["label"], "test");
        assert_eq!(result["kind"], 12);
    }

    #[test]
    fn test_map_empty_response() {
        let mapper = CompletionMapper::new();
        let response = serde_json::json!({
            "result": {
                "items": []
            }
        });

        let field_mappings = HashMap::new();
        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let results = mapper.map(&response, &rules).unwrap();
        assert_eq!(results.len(), 0);
    }
}
