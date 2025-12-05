//! JSON path expression parser
//!
//! Supports parsing and evaluating JSON path expressions like:
//! - `$.result.items` - nested field access
//! - `$.result.items[*].label` - array indexing with wildcard
//! - `$[0].range.start.line` - array indexing with specific index
//! - `$.result` - simple field access

use crate::error::{ExternalLspError, Result};
use serde_json::Value;
use std::str::FromStr;

/// A segment of a JSON path expression
#[derive(Debug, Clone, PartialEq, Eq)]
enum PathSegment {
    /// Root selector ($)
    Root,
    /// Field access (.field)
    Field(String),
    /// Array index ([0])
    Index(usize),
    /// Array wildcard ([*])
    Wildcard,
}

/// Parses and evaluates JSON path expressions
#[derive(Debug, Clone)]
pub struct JsonPathParser {
    segments: Vec<PathSegment>,
}

impl JsonPathParser {
    /// Create a new JSON path parser from an expression string
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let parser = JsonPathParser::parse("$.result.items[*].label")?;
    /// ```
    pub fn parse(expression: &str) -> Result<Self> {
        let segments = Self::parse_expression(expression)?;
        Ok(Self { segments })
    }

    /// Parse a JSON path expression into segments
    fn parse_expression(expr: &str) -> Result<Vec<PathSegment>> {
        let expr = expr.trim();

        if !expr.starts_with('$') {
            return Err(ExternalLspError::JsonPathError(
                "JSON path must start with $".to_string(),
            ));
        }

        let mut segments = vec![PathSegment::Root];
        let mut remaining = &expr[1..];

        while !remaining.is_empty() {
            if remaining.starts_with('.') {
                // Field access
                remaining = &remaining[1..];

                // Find the end of the field name
                let end = remaining
                    .find(['.', '['])
                    .unwrap_or(remaining.len());

                if end == 0 {
                    return Err(ExternalLspError::JsonPathError(
                        "Empty field name in JSON path".to_string(),
                    ));
                }

                let field = remaining[..end].to_string();
                segments.push(PathSegment::Field(field));
                remaining = &remaining[end..];
            } else if remaining.starts_with('[') {
                // Array access
                let end = remaining.find(']').ok_or_else(|| {
                    ExternalLspError::JsonPathError("Unclosed bracket in JSON path".to_string())
                })?;

                let index_str = &remaining[1..end];

                if index_str == "*" {
                    segments.push(PathSegment::Wildcard);
                } else {
                    let index: usize = index_str.parse().map_err(|_| {
                        ExternalLspError::JsonPathError(format!(
                            "Invalid array index: {}",
                            index_str
                        ))
                    })?;
                    segments.push(PathSegment::Index(index));
                }

                remaining = &remaining[end + 1..];
            } else {
                return Err(ExternalLspError::JsonPathError(format!(
                    "Unexpected character in JSON path: {}",
                    remaining.chars().next().unwrap_or('?')
                )));
            }
        }

        Ok(segments)
    }

    /// Extract values from a JSON object using this path
    ///
    /// Returns a vector of values. For paths with wildcards, may return multiple values.
    pub fn extract(&self, value: &Value) -> Result<Vec<Value>> {
        Self::extract_recursive(value, &self.segments, 0)
    }

    /// Recursively extract values following the path segments
    fn extract_recursive(value: &Value, segments: &[PathSegment], index: usize) -> Result<Vec<Value>> {
        if index >= segments.len() {
            return Ok(vec![value.clone()]);
        }

        match &segments[index] {
            PathSegment::Root => {
                // Root is always the current value
                Self::extract_recursive(value, segments, index + 1)
            }
            PathSegment::Field(field) => {
                let next_value = value
                    .get(field)
                    .ok_or_else(|| {
                        ExternalLspError::JsonPathError(format!(
                            "Field '{}' not found in JSON object",
                            field
                        ))
                    })?;

                Self::extract_recursive(next_value, segments, index + 1)
            }
            PathSegment::Index(idx) => {
                let array = value.as_array().ok_or_else(|| {
                    ExternalLspError::JsonPathError(format!(
                        "Expected array at index access, got {}",
                        Self::value_type_name(value)
                    ))
                })?;

                let next_value = array.get(*idx).ok_or_else(|| {
                    ExternalLspError::JsonPathError(format!(
                        "Array index {} out of bounds (length: {})",
                        idx,
                        array.len()
                    ))
                })?;

                Self::extract_recursive(next_value, segments, index + 1)
            }
            PathSegment::Wildcard => {
                let array = value.as_array().ok_or_else(|| {
                    ExternalLspError::JsonPathError(format!(
                        "Expected array for wildcard, got {}",
                        Self::value_type_name(value)
                    ))
                })?;

                let mut results = Vec::new();
                for item in array {
                    let mut item_results = Self::extract_recursive(item, segments, index + 1)?;
                    results.append(&mut item_results);
                }

                Ok(results)
            }
        }
    }

    /// Extract a single value from a JSON object, returning an error if not found
    pub fn extract_single(&self, value: &Value) -> Result<Value> {
        let results = self.extract(value)?;

        if results.is_empty() {
            return Err(ExternalLspError::JsonPathError(
                "JSON path returned no results".to_string(),
            ));
        }

        if results.len() > 1 {
            return Err(ExternalLspError::JsonPathError(
                "JSON path returned multiple results, expected single value".to_string(),
            ));
        }

        Ok(results[0].clone())
    }

    /// Get the type name of a JSON value
    fn value_type_name(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

impl FromStr for JsonPathParser {
    type Err = ExternalLspError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_field() {
        let parser = JsonPathParser::parse("$.result").unwrap();
        let json = json!({"result": "value"});
        let results = parser.extract(&json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!("value"));
    }

    #[test]
    fn test_parse_nested_field() {
        let parser = JsonPathParser::parse("$.result.items").unwrap();
        let json = json!({"result": {"items": [1, 2, 3]}});
        let results = parser.extract(&json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!([1, 2, 3]));
    }

    #[test]
    fn test_parse_array_index() {
        let parser = JsonPathParser::parse("$.items[0]").unwrap();
        let json = json!({"items": ["a", "b", "c"]});
        let results = parser.extract(&json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!("a"));
    }

    #[test]
    fn test_parse_wildcard() {
        let parser = JsonPathParser::parse("$.items[*].label").unwrap();
        let json = json!({
            "items": [
                {"label": "first"},
                {"label": "second"},
                {"label": "third"}
            ]
        });
        let results = parser.extract(&json).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], json!("first"));
        assert_eq!(results[1], json!("second"));
        assert_eq!(results[2], json!("third"));
    }

    #[test]
    fn test_parse_missing_field() {
        let parser = JsonPathParser::parse("$.missing").unwrap();
        let json = json!({"result": "value"});
        let result = parser.extract(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_expression() {
        let result = JsonPathParser::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_bracket() {
        let result = JsonPathParser::parse("$.items[0");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_single_success() {
        let parser = JsonPathParser::parse("$.result").unwrap();
        let json = json!({"result": "value"});
        let result = parser.extract_single(&json).unwrap();
        assert_eq!(result, json!("value"));
    }

    #[test]
    fn test_extract_single_multiple_results() {
        let parser = JsonPathParser::parse("$.items[*]").unwrap();
        let json = json!({"items": [1, 2, 3]});
        let result = parser.extract_single(&json);
        assert!(result.is_err());
    }
}
