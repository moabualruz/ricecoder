//! Diagnostics output mapping
//!
//! Maps LSP diagnostic responses to ricecoder Diagnostic models.
//! Supports custom field mappings via configuration and transformation functions.

use crate::error::Result;
use crate::types::DiagnosticsMappingRules;
use serde_json::Value;

use super::transformer::OutputTransformer;

/// Maps LSP diagnostic responses to ricecoder models
#[derive(Debug, Clone)]
pub struct DiagnosticsMapper {
    transformer: OutputTransformer,
}

impl DiagnosticsMapper {
    /// Create a new diagnostics mapper
    pub fn new() -> Self {
        Self {
            transformer: OutputTransformer::new(),
        }
    }

    /// Create a mapper with custom transformations
    pub fn with_transformer(transformer: OutputTransformer) -> Self {
        Self { transformer }
    }

    /// Map an LSP diagnostics response to ricecoder models
    ///
    /// # Arguments
    ///
    /// * `response` - The LSP server response (typically from textDocument/publishDiagnostics)
    /// * `rules` - The mapping rules from configuration
    ///
    /// # Returns
    ///
    /// A vector of mapped diagnostic items
    pub fn map(&self, response: &Value, rules: &DiagnosticsMappingRules) -> Result<Vec<Value>> {
        self.transformer.transform_diagnostics(response, rules)
    }

    /// Map a single diagnostic item
    ///
    /// This is useful for mapping individual items when the response structure
    /// doesn't match the expected array format.
    pub fn map_item(&self, item: &Value, rules: &DiagnosticsMappingRules) -> Result<Value> {
        // Create a wrapper response with the item
        let wrapped = serde_json::json!({
            "result": {
                "items": [item]
            }
        });

        // Use default rules that expect this structure
        let default_rules = DiagnosticsMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings: rules.field_mappings.clone(),
            transform: rules.transform.clone(),
        };

        let results = self.transformer.transform_diagnostics(&wrapped, &default_rules)?;

        if results.is_empty() {
            return Err(crate::error::ExternalLspError::TransformationError(
                "Failed to map diagnostic item".to_string(),
            ));
        }

        Ok(results[0].clone())
    }
}

impl Default for DiagnosticsMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_map_diagnostics_response() {
        let mapper = DiagnosticsMapper::new();
        let response = serde_json::json!({
            "result": [
                {
                    "message": "error: undefined variable",
                    "range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 5}},
                    "severity": 1
                },
                {
                    "message": "warning: unused variable",
                    "range": {"start": {"line": 2, "character": 0}, "end": {"line": 2, "character": 3}},
                    "severity": 2
                }
            ]
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("message".to_string(), "$.message".to_string());
        field_mappings.insert("range".to_string(), "$.range".to_string());
        field_mappings.insert("severity".to_string(), "$.severity".to_string());

        let rules = DiagnosticsMappingRules {
            items_path: "$.result".to_string(),
            field_mappings,
            transform: None,
        };

        let results = mapper.map(&response, &rules).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["message"], "error: undefined variable");
        assert_eq!(results[1]["message"], "warning: unused variable");
    }

    #[test]
    fn test_map_diagnostics_with_custom_structure() {
        let mapper = DiagnosticsMapper::new();
        let response = serde_json::json!({
            "issues": [
                {"error_message": "error", "error_line": 1},
                {"error_message": "warning", "error_line": 2}
            ]
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("message".to_string(), "$.error_message".to_string());
        field_mappings.insert("line".to_string(), "$.error_line".to_string());

        let rules = DiagnosticsMappingRules {
            items_path: "$.issues".to_string(),
            field_mappings,
            transform: None,
        };

        let results = mapper.map(&response, &rules).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["message"], "error");
        assert_eq!(results[0]["line"], 1);
    }

    #[test]
    fn test_map_single_diagnostic() {
        let mapper = DiagnosticsMapper::new();
        let item = serde_json::json!({
            "message": "test error",
            "severity": 1,
            "range": {"start": {"line": 0, "character": 0}}
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("message".to_string(), "$.message".to_string());
        field_mappings.insert("severity".to_string(), "$.severity".to_string());

        let rules = DiagnosticsMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map_item(&item, &rules).unwrap();
        assert_eq!(result["message"], "test error");
        assert_eq!(result["severity"], 1);
    }

    #[test]
    fn test_map_empty_diagnostics() {
        let mapper = DiagnosticsMapper::new();
        let response = serde_json::json!({
            "result": []
        });

        let field_mappings = HashMap::new();
        let rules = DiagnosticsMappingRules {
            items_path: "$.result".to_string(),
            field_mappings,
            transform: None,
        };

        let results = mapper.map(&response, &rules).unwrap();
        assert_eq!(results.len(), 0);
    }
}
