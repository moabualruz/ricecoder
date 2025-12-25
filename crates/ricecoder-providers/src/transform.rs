//! Provider-specific message transforms
//!
//! This module implements provider-specific transformations for:
//! - Claude: toolCallId sanitization
//! - Mistral: toolCallId normalization
//! - Gemini: schema sanitization

use crate::models::Message;
use regex::Regex;
use serde_json::Value;

/// Transform messages for Claude provider
pub fn transform_for_claude(messages: Vec<Message>) -> Vec<Message> {
    messages
        .into_iter()
        .map(|mut msg| {
            // Sanitize toolCallId if present
            if let Ok(mut content) = serde_json::from_str::<Value>(&msg.content) {
                if let Some(tool_calls) = content.get_mut("tool_calls").and_then(|v| v.as_array_mut()) {
                    for call in tool_calls {
                        if let Some(id) = call.get_mut("id").and_then(|v| v.as_str()) {
                            let sanitized = sanitize_claude_tool_id(id);
                            *call.get_mut("id").unwrap() = Value::String(sanitized);
                        }
                    }
                    msg.content = content.to_string();
                }
            }
            msg
        })
        .collect()
}

/// Sanitize Claude toolCallId: replace [^a-zA-Z0-9_-] with _
fn sanitize_claude_tool_id(id: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_-]").unwrap();
    re.replace_all(id, "_").to_string()
}

/// Transform messages for Mistral provider
pub fn transform_for_mistral(messages: Vec<Message>) -> Vec<Message> {
    let mut result = Vec::new();
    let mut last_role: Option<String> = None;

    for mut msg in messages {
        // Insert assistant message between tool → user transitions
        if last_role.as_deref() == Some("tool") {
            if msg.role == "user" {
                result.push(Message {
                    role: "assistant".to_string(),
                    content: "Done.".to_string(),
                });
            }
        }

        // Normalize toolCallId if present
        if let Ok(mut content) = serde_json::from_str::<Value>(&msg.content) {
            if let Some(tool_calls) = content.get_mut("tool_calls").and_then(|v| v.as_array_mut()) {
                for call in tool_calls {
                    if let Some(id) = call.get_mut("id").and_then(|v| v.as_str()) {
                        let normalized = normalize_mistral_tool_id(id);
                        *call.get_mut("id").unwrap() = Value::String(normalized);
                    }
                }
                msg.content = content.to_string();
            }
        }

        let role_clone = msg.role.clone();
        result.push(msg);
        last_role = Some(role_clone);
    }

    result
}

/// Normalize Mistral toolCallId: remove non-alphanumeric, truncate to 9, pad with 0
fn normalize_mistral_tool_id(id: &str) -> String {
    let clean: String = id.chars().filter(|c| c.is_alphanumeric()).collect();
    let truncated: String = clean.chars().take(9).collect();
    format!("{:0<9}", truncated) // Pad with zeros if < 9
}

/// Transform schema for Gemini provider
pub fn transform_schema_for_gemini(schema: Value) -> Value {
    transform_schema_recursive(schema)
}

fn transform_schema_recursive(mut value: Value) -> Value {
    match &mut value {
        Value::Object(map) => {
            // Convert enum values to strings
            if let Some(enum_values) = map.get_mut("enum") {
                if let Some(arr) = enum_values.as_array_mut() {
                    *arr = arr
                        .iter()
                        .map(|v| Value::String(v.to_string().trim_matches('"').to_string()))
                        .collect();
                }
            }

            // Change type to string if enum present and type is integer/number
            if map.contains_key("enum") {
                if let Some(type_val) = map.get("type") {
                    if type_val == "integer" || type_val == "number" {
                        map.insert("type".to_string(), Value::String("string".to_string()));
                    }
                }
            }

            // Filter required fields
            if let Some(required) = map.get("required").and_then(|v| v.as_array()) {
                if let Some(properties) = map.get("properties").and_then(|v| v.as_object()) {
                    let valid_required: Vec<Value> = required
                        .iter()
                        .filter(|field| {
                            field
                                .as_str()
                                .map(|s| properties.contains_key(s))
                                .unwrap_or(false)
                        })
                        .cloned()
                        .collect();
                    map.insert("required".to_string(), Value::Array(valid_required));
                }
            }

            // Ensure arrays have items
            if map.get("type") == Some(&Value::String("array".to_string())) {
                if !map.contains_key("items") {
                    map.insert("items".to_string(), Value::Object(Default::default()));
                }
            }

            // Recursively transform nested objects
            for (_, v) in map.iter_mut() {
                *v = transform_schema_recursive(v.clone());
            }

            Value::Object(map.clone())
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| transform_schema_recursive(v.clone())).collect())
        }
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_claude_tool_id() {
        assert_eq!(sanitize_claude_tool_id("tool::call::123"), "tool__call__123");
        assert_eq!(sanitize_claude_tool_id("call@#$%456"), "call____456");
    }

    #[test]
    fn test_normalize_mistral_tool_id() {
        assert_eq!(normalize_mistral_tool_id("call_123"), "call12300");
        assert_eq!(normalize_mistral_tool_id("verylongid123"), "verylong1");
    }

    #[test]
    fn test_mistral_message_sequencing() {
        let messages = vec![
            Message {
                role: "tool".to_string(),
                content: "result".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "question".to_string(),
            },
        ];

        let transformed = transform_for_mistral(messages);
        assert_eq!(transformed.len(), 3);
        assert_eq!(transformed[1].role, "assistant");
        assert_eq!(transformed[1].content, "Done.");
    }

    #[test]
    fn test_gemini_schema_transform() {
        let schema = serde_json::json!({
            "type": "integer",
            "enum": [1, 2, 3]
        });

        let transformed = transform_schema_for_gemini(schema);
        assert_eq!(transformed["type"], "string");
    }
}
