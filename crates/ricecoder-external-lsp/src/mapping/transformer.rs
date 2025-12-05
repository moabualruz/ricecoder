//! Output transformation engine
//!
//! Transforms LSP server responses to ricecoder models using configuration-driven rules.
//! Supports JSON path expressions for field extraction and custom transformation functions.

use crate::error::{ExternalLspError, Result};
use crate::types::{CompletionMappingRules, DiagnosticsMappingRules, HoverMappingRules};
use serde_json::{json, Value};
use std::collections::HashMap;

use super::json_path::JsonPathParser;

/// Transforms LSP server output to ricecoder models
#[derive(Debug, Clone)]
pub struct OutputTransformer {
    /// Custom transformation functions (by name)
    custom_transforms: HashMap<String, String>,
}

impl OutputTransformer {
    /// Create a new output transformer
    pub fn new() -> Self {
        Self {
            custom_transforms: HashMap::new(),
        }
    }

    /// Create a transformer with custom transformation functions
    pub fn with_transforms(custom_transforms: HashMap<String, String>) -> Self {
        Self { custom_transforms }
    }

    /// Register a custom transformation function
    pub fn register_transform(&mut self, name: String, function: String) {
        self.custom_transforms.insert(name, function);
    }

    /// Transform a completion response using the provided rules
    pub fn transform_completion(
        &self,
        response: &Value,
        rules: &CompletionMappingRules,
    ) -> Result<Vec<Value>> {
        // Extract items array using JSON path
        let items_parser = JsonPathParser::parse(&rules.items_path)?;
        let items = items_parser.extract(response)?;

        if items.is_empty() {
            return Ok(Vec::new());
        }

        // If we got multiple items (from wildcard), use them; otherwise expect an array
        let items_array = if items.len() == 1 && items[0].is_array() {
            items[0].as_array().unwrap().clone()
        } else if items.len() == 1 && items[0].is_object() {
            // Single object, wrap in array
            vec![items[0].clone()]
        } else {
            // Multiple items from wildcard
            items
        };

        // Transform each item
        let mut results = Vec::new();
        for item in items_array {
            let transformed = self.apply_field_mappings(&item, &rules.field_mappings)?;

            // Apply custom transformation if specified
            let final_item = if let Some(transform_name) = &rules.transform {
                self.apply_custom_transform(&transformed, transform_name)?
            } else {
                transformed
            };

            results.push(final_item);
        }

        Ok(results)
    }

    /// Transform a diagnostics response using the provided rules
    pub fn transform_diagnostics(
        &self,
        response: &Value,
        rules: &DiagnosticsMappingRules,
    ) -> Result<Vec<Value>> {
        // Extract items array using JSON path
        let items_parser = JsonPathParser::parse(&rules.items_path)?;
        let items = items_parser.extract(response)?;

        if items.is_empty() {
            return Ok(Vec::new());
        }

        // If we got multiple items (from wildcard), use them; otherwise expect an array
        let items_array = if items.len() == 1 && items[0].is_array() {
            items[0].as_array().unwrap().clone()
        } else if items.len() == 1 && items[0].is_object() {
            // Single object, wrap in array
            vec![items[0].clone()]
        } else {
            // Multiple items from wildcard
            items
        };

        // Transform each item
        let mut results = Vec::new();
        for item in items_array {
            let transformed = self.apply_field_mappings(&item, &rules.field_mappings)?;

            // Apply custom transformation if specified
            let final_item = if let Some(transform_name) = &rules.transform {
                self.apply_custom_transform(&transformed, transform_name)?
            } else {
                transformed
            };

            results.push(final_item);
        }

        Ok(results)
    }

    /// Transform a hover response using the provided rules
    pub fn transform_hover(
        &self,
        response: &Value,
        rules: &HoverMappingRules,
    ) -> Result<Value> {
        // Extract content using JSON path
        let content_parser = JsonPathParser::parse(&rules.content_path)?;
        let content = content_parser.extract_single(response)?;

        // Apply field mappings
        let transformed = self.apply_field_mappings(&content, &rules.field_mappings)?;

        // Apply custom transformation if specified
        let final_value = if let Some(transform_name) = &rules.transform {
            self.apply_custom_transform(&transformed, transform_name)?
        } else {
            transformed
        };

        Ok(final_value)
    }

    /// Apply field mappings to extract and rename fields
    fn apply_field_mappings(
        &self,
        source: &Value,
        field_mappings: &HashMap<String, String>,
    ) -> Result<Value> {
        let mut result = json!({});

        for (target_field, source_path) in field_mappings {
            let parser = JsonPathParser::parse(source_path)?;

            match parser.extract_single(source) {
                Ok(value) => {
                    result[target_field] = value;
                }
                Err(_) => {
                    // Field not found, skip it (optional field)
                    continue;
                }
            }
        }

        Ok(result)
    }

    /// Apply a custom transformation function
    fn apply_custom_transform(&self, value: &Value, transform_name: &str) -> Result<Value> {
        // For now, we support built-in transformations
        // Custom transformation functions would be evaluated here
        match transform_name {
            "identity" => Ok(value.clone()),
            "stringify" => Ok(Value::String(value.to_string())),
            _ => {
                // Check if it's a registered custom transform
                if self.custom_transforms.contains_key(transform_name) {
                    // In a real implementation, we would evaluate the custom function
                    // For now, just return the value as-is
                    Ok(value.clone())
                } else {
                    Err(ExternalLspError::TransformationError(format!(
                        "Unknown transformation function: {}",
                        transform_name
                    )))
                }
            }
        }
    }
}

impl Default for OutputTransformer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_completion_simple() {
        let transformer = OutputTransformer::new();
        let response = json!({
            "result": {
                "items": [
                    {"label": "foo", "detail": "function"},
                    {"label": "bar", "detail": "variable"}
                ]
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("detail".to_string(), "$.detail".to_string());

        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let results = transformer.transform_completion(&response, &rules).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["label"], "foo");
        assert_eq!(results[1]["label"], "bar");
    }

    #[test]
    fn test_transform_completion_with_wildcard() {
        let transformer = OutputTransformer::new();
        let response = json!({
            "completions": [
                {"name": "foo", "type": "function"},
                {"name": "bar", "type": "variable"}
            ]
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.name".to_string());
        field_mappings.insert("kind".to_string(), "$.type".to_string());

        let rules = CompletionMappingRules {
            items_path: "$.completions[*]".to_string(),
            field_mappings,
            transform: None,
        };

        let results = transformer.transform_completion(&response, &rules).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["label"], "foo");
        assert_eq!(results[0]["kind"], "function");
    }

    #[test]
    fn test_transform_diagnostics() {
        let transformer = OutputTransformer::new();
        let response = json!({
            "issues": [
                {"message": "error", "line": 1},
                {"message": "warning", "line": 2}
            ]
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("message".to_string(), "$.message".to_string());
        field_mappings.insert("line".to_string(), "$.line".to_string());

        let rules = DiagnosticsMappingRules {
            items_path: "$.issues".to_string(),
            field_mappings,
            transform: None,
        };

        let results = transformer.transform_diagnostics(&response, &rules).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["message"], "error");
    }

    #[test]
    fn test_transform_hover() {
        let transformer = OutputTransformer::new();
        let response = json!({
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

        let result = transformer.transform_hover(&response, &rules).unwrap();
        assert_eq!(result["language"], "rust");
        assert_eq!(result["value"], "fn foo() -> i32");
    }

    #[test]
    fn test_missing_field_in_mapping() {
        let transformer = OutputTransformer::new();
        let response = json!({
            "result": {
                "items": [
                    {"label": "foo"}
                ]
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("detail".to_string(), "$.detail".to_string()); // Missing

        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let results = transformer.transform_completion(&response, &rules).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["label"], "foo");
        assert!(!results[0].get("detail").is_some() || results[0]["detail"].is_null());
    }
}
