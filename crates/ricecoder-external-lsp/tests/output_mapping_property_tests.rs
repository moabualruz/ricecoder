//! Property-based tests for output mapping
//!
//! **Feature: ricecoder-external-lsp, Property 5: Output Mapping Correctness**
//! **Validates: Requirements ELSP-2.5**

use proptest::prelude::*;
use ricecoder_external_lsp::mapping::{
    CompletionMapper, DiagnosticsMapper, HoverMapper, JsonPathParser,
};
use ricecoder_external_lsp::types::{
    CompletionMappingRules, DiagnosticsMappingRules, HoverMappingRules,
};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Strategy for generating valid JSON path expressions
fn arb_json_path() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("$.result".to_string()),
        Just("$.result.items".to_string()),
        Just("$.result.items[0]".to_string()),
        Just("$.result.items[*].label".to_string()),
        Just("$.data".to_string()),
        Just("$.data[*]".to_string()),
    ]
}

/// Strategy for generating valid field mappings
fn arb_field_mappings() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map("label|detail|kind|message|severity", "\\$\\.\\w+", 1..5).prop_map(
        |map| {
            map.into_iter()
                .map(|(k, _)| {
                    let v = match k.as_str() {
                        "label" => "$.label",
                        "detail" => "$.detail",
                        "kind" => "$.kind",
                        "message" => "$.message",
                        "severity" => "$.severity",
                        _ => "$.value",
                    };
                    (k, v.to_string())
                })
                .collect()
        },
    )
}

/// Strategy for generating valid JSON responses with completion items
fn arb_completion_response() -> impl Strategy<Value = Value> {
    prop::collection::vec(("[a-z]+", "[a-z]+", 0u32..20u32), 1..10).prop_map(|items| {
        let items_json: Vec<Value> = items
            .into_iter()
            .map(|(label, detail, kind)| {
                json!({
                    "label": label,
                    "detail": detail,
                    "kind": kind
                })
            })
            .collect();

        json!({
            "result": {
                "items": items_json
            }
        })
    })
}

/// Strategy for generating valid JSON responses with diagnostics
fn arb_diagnostics_response() -> impl Strategy<Value = Value> {
    prop::collection::vec(("[a-z ]+", 1u32..3u32), 0..10).prop_map(|items| {
        let items_json: Vec<Value> = items
            .into_iter()
            .map(|(message, severity)| {
                json!({
                    "message": message,
                    "severity": severity,
                    "range": {
                        "start": {"line": 0, "character": 0},
                        "end": {"line": 0, "character": 5}
                    }
                })
            })
            .collect();

        json!({
            "result": items_json
        })
    })
}

/// Strategy for generating valid JSON responses with hover info
fn arb_hover_response() -> impl Strategy<Value = Value> {
    ("[a-z]+", "[a-z ]+").prop_map(|(language, value)| {
        json!({
            "result": {
                "contents": {
                    "language": language,
                    "value": value
                }
            }
        })
    })
}

proptest! {
    /// Property 5: Output Mapping Correctness
    ///
    /// For any LSP server response and configured output mapping rules, the transformed
    /// result SHALL contain all required fields mapped correctly, and invalid mappings
    /// SHALL be rejected with clear error messages.
    ///
    /// This property tests that:
    /// 1. Valid JSON paths are parsed successfully
    /// 2. Field mappings extract the correct values
    /// 3. Transformed results contain all mapped fields
    /// 4. Invalid paths are rejected with clear errors
    #[test]
    fn prop_json_path_parsing_valid(path in arb_json_path()) {
        // Valid paths should parse successfully
        let result = JsonPathParser::parse(&path);
        prop_assert!(result.is_ok(), "Valid path should parse: {}", path);
    }

    /// Property: Invalid JSON paths are rejected
    ///
    /// JSON paths that don't start with $ or have invalid syntax should be rejected
    /// with clear error messages.
    #[test]
    fn prop_json_path_parsing_invalid(invalid_path in "[a-z]+") {
        // Paths not starting with $ should fail
        if !invalid_path.starts_with('$') {
            let result = JsonPathParser::parse(&invalid_path);
            prop_assert!(result.is_err(), "Invalid path should fail: {}", invalid_path);
        }
    }

    /// Property: Completion mapping preserves all fields
    ///
    /// When mapping completion items, all fields specified in the mapping rules
    /// should be present in the output (or skipped if the source field doesn't exist).
    #[test]
    fn prop_completion_mapping_preserves_fields(
        response in arb_completion_response(),
    ) {
        let mapper = CompletionMapper::new();

        // Use fixed field mappings that match the response structure
        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("detail".to_string(), "$.detail".to_string());
        field_mappings.insert("kind".to_string(), "$.kind".to_string());

        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings: field_mappings.clone(),
            transform: None,
        };

        let result = mapper.map(&response, &rules);
        prop_assert!(result.is_ok(), "Mapping should succeed");

        let items = result.unwrap();

        // For each mapped item, verify all mapped fields are present
        for item in items {
            for field_name in field_mappings.keys() {
                // Field should be present (even if null)
                prop_assert!(
                    item.get(field_name).is_some(),
                    "Field '{}' should be present in mapped item",
                    field_name
                );
            }
        }
    }

    /// Property: Diagnostics mapping preserves all fields
    ///
    /// When mapping diagnostics, all fields specified in the mapping rules
    /// should be present in the output (or skipped if the source field doesn't exist).
    #[test]
    fn prop_diagnostics_mapping_preserves_fields(
        response in arb_diagnostics_response(),
    ) {
        let mapper = DiagnosticsMapper::new();

        // Use fixed field mappings that match the response structure
        let mut field_mappings = HashMap::new();
        field_mappings.insert("message".to_string(), "$.message".to_string());
        field_mappings.insert("severity".to_string(), "$.severity".to_string());
        field_mappings.insert("range".to_string(), "$.range".to_string());

        let rules = DiagnosticsMappingRules {
            items_path: "$.result".to_string(),
            field_mappings: field_mappings.clone(),
            transform: None,
        };

        let result = mapper.map(&response, &rules);
        prop_assert!(result.is_ok(), "Mapping should succeed");

        let items = result.unwrap();

        // For each mapped item, verify all mapped fields are present
        for item in items {
            for field_name in field_mappings.keys() {
                // Field should be present (even if null)
                prop_assert!(
                    item.get(field_name).is_some(),
                    "Field '{}' should be present in mapped item",
                    field_name
                );
            }
        }
    }

    /// Property: Hover mapping produces valid output
    ///
    /// When mapping hover information, the output should be a valid JSON object
    /// with all mapped fields present.
    #[test]
    fn prop_hover_mapping_produces_valid_output(
        response in arb_hover_response(),
    ) {
        let mapper = HoverMapper::new();

        // Use fixed field mappings that match the response structure
        let mut field_mappings = HashMap::new();
        field_mappings.insert("language".to_string(), "$.language".to_string());
        field_mappings.insert("value".to_string(), "$.value".to_string());

        let rules = HoverMappingRules {
            content_path: "$.result.contents".to_string(),
            field_mappings: field_mappings.clone(),
            transform: None,
        };

        let result = mapper.map(&response, &rules);
        prop_assert!(result.is_ok(), "Mapping should succeed");

        let item = result.unwrap();

        // Result should be an object
        prop_assert!(item.is_object(), "Result should be a JSON object");

        // All mapped fields should be present
        for field_name in field_mappings.keys() {
            prop_assert!(
                item.get(field_name).is_some(),
                "Field '{}' should be present in mapped item",
                field_name
            );
        }
    }

    /// Property: Mapping is deterministic
    ///
    /// Mapping the same response with the same rules should always produce
    /// identical results.
    #[test]
    fn prop_mapping_is_deterministic(
        response in arb_completion_response(),
        field_mappings in arb_field_mappings(),
    ) {
        let mapper = CompletionMapper::new();
        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let result1 = mapper.map(&response, &rules);
        let result2 = mapper.map(&response, &rules);

        // Both should succeed or both should fail
        prop_assert_eq!(result1.is_ok(), result2.is_ok(), "Mapping should be deterministic");

        // If both succeeded, they should be equal
        if let (Ok(items1), Ok(items2)) = (result1, result2) {
            prop_assert_eq!(items1, items2, "Mapping results should be identical");
        }
    }

    /// Property: Empty responses are handled gracefully
    ///
    /// Responses with empty arrays should return empty results without errors.
    #[test]
    fn prop_empty_response_handling(field_mappings in arb_field_mappings()) {
        let mapper = CompletionMapper::new();
        let empty_response = json!({
            "result": {
                "items": []
            }
        });

        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&empty_response, &rules);
        prop_assert!(result.is_ok(), "Empty response should be handled gracefully");
        prop_assert_eq!(result.unwrap().len(), 0, "Empty response should return empty results");
    }

    /// Property: Field extraction is accurate
    ///
    /// When extracting fields using JSON paths, the extracted values should
    /// match the source values exactly.
    #[test]
    fn prop_field_extraction_accuracy(
        label in "[a-z]+",
        detail in "[a-z]+",
    ) {
        let response = json!({
            "result": {
                "items": [
                    {
                        "label": label.clone(),
                        "detail": detail.clone()
                    }
                ]
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("detail".to_string(), "$.detail".to_string());

        let mapper = CompletionMapper::new();
        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&response, &rules).unwrap();
        prop_assert_eq!(result.len(), 1);
        prop_assert_eq!(result[0]["label"].as_str().unwrap(), label);
        prop_assert_eq!(result[0]["detail"].as_str().unwrap(), detail);
    }

    /// Property: Mapping handles missing optional fields
    ///
    /// When a field mapping references a path that doesn't exist, the field
    /// should be omitted from the result (not cause an error).
    #[test]
    fn prop_missing_optional_fields_handled(
        label in "[a-z]+",
    ) {
        let response = json!({
            "result": {
                "items": [
                    {
                        "label": label.clone()
                        // "detail" is missing
                    }
                ]
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("detail".to_string(), "$.detail".to_string()); // Missing

        let mapper = CompletionMapper::new();
        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&response, &rules);
        prop_assert!(result.is_ok(), "Mapping should succeed even with missing optional fields");

        let items = result.unwrap();
        prop_assert_eq!(items.len(), 1);
        prop_assert_eq!(items[0]["label"].as_str().unwrap(), label);
        // detail should not be present or be null
        prop_assert!(!items[0].get("detail").is_some() || items[0]["detail"].is_null());
    }

    /// Property: Multiple items are mapped independently
    ///
    /// When mapping multiple items, each item should be mapped independently
    /// without affecting other items.
    #[test]
    fn prop_multiple_items_independent(
        items_data in prop::collection::vec(("[a-z]+", "[a-z]+"), 2..5),
    ) {
        let items_json: Vec<Value> = items_data
            .iter()
            .map(|(label, detail)| {
                json!({
                    "label": label,
                    "detail": detail
                })
            })
            .collect();

        let response = json!({
            "result": {
                "items": items_json
            }
        });

        let mut field_mappings = HashMap::new();
        field_mappings.insert("label".to_string(), "$.label".to_string());
        field_mappings.insert("detail".to_string(), "$.detail".to_string());

        let mapper = CompletionMapper::new();
        let rules = CompletionMappingRules {
            items_path: "$.result.items".to_string(),
            field_mappings,
            transform: None,
        };

        let result = mapper.map(&response, &rules).unwrap();
        prop_assert_eq!(result.len(), items_data.len());

        // Verify each item is mapped correctly
        for (i, (expected_label, expected_detail)) in items_data.iter().enumerate() {
            prop_assert_eq!(result[i]["label"].as_str().unwrap(), expected_label.as_str());
            prop_assert_eq!(result[i]["detail"].as_str().unwrap(), expected_detail.as_str());
        }
    }
}
