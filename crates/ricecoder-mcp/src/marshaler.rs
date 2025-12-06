//! Tool Marshaler component for converting between formats

use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Marshals tool inputs and outputs between agent and MCP formats
#[derive(Debug, Clone)]
pub struct ToolMarshaler;

impl ToolMarshaler {
    /// Creates a new ToolMarshaler
    pub fn new() -> Self {
        Self
    }

    /// Marshals tool inputs from agent format to MCP format
    ///
    /// # Arguments
    /// * `input` - Input parameters in agent format
    ///
    /// # Returns
    /// Marshaled input in MCP format
    ///
    /// # Errors
    /// Returns error if input validation fails
    pub fn marshal_input(&self, input: &Value) -> Result<Value> {
        // Validate input before marshaling
        self.validate_input(input)?;

        // Convert agent format to MCP format
        // For now, pass through with minimal transformation
        Ok(input.clone())
    }

    /// Unmarshals tool outputs from MCP format to agent format
    ///
    /// # Arguments
    /// * `output` - Output in MCP format
    ///
    /// # Returns
    /// Unmarshaled output in agent format
    ///
    /// # Errors
    /// Returns error if output validation fails
    pub fn unmarshal_output(&self, output: &Value) -> Result<Value> {
        // Validate output after unmarshaling
        self.validate_output(output)?;

        // Convert MCP format to agent format
        // For now, pass through with minimal transformation
        Ok(output.clone())
    }

    /// Validates tool parameters before marshaling
    ///
    /// # Arguments
    /// * `input` - Input parameters to validate
    ///
    /// # Errors
    /// Returns error if validation fails
    fn validate_input(&self, input: &Value) -> Result<()> {
        // Check that input is an object or null
        match input {
            Value::Object(_) | Value::Null => Ok(()),
            _ => Err(Error::ParameterValidationError(
                "Input must be an object or null".to_string(),
            )),
        }
    }

    /// Validates tool output after unmarshaling
    ///
    /// # Arguments
    /// * `output` - Output to validate
    ///
    /// # Errors
    /// Returns error if validation fails
    fn validate_output(&self, output: &Value) -> Result<()> {
        // Output can be any valid JSON value
        // Basic validation: ensure it's not undefined
        match output {
            Value::Null => Ok(()),
            _ => Ok(()),
        }
    }

    /// Converts a JSON value to a specific type
    ///
    /// # Arguments
    /// * `value` - Value to convert
    /// * `target_type` - Target type name
    ///
    /// # Returns
    /// Converted value
    ///
    /// # Errors
    /// Returns error if conversion fails
    pub fn convert_type(&self, value: &Value, target_type: &str) -> Result<Value> {
        match target_type {
            "string" => match value {
                Value::String(s) => Ok(Value::String(s.clone())),
                Value::Number(n) => Ok(Value::String(n.to_string())),
                Value::Bool(b) => Ok(Value::String(b.to_string())),
                Value::Null => Ok(Value::String("null".to_string())),
                _ => Err(Error::ValidationError(format!(
                    "Cannot convert {} to string",
                    value.get_type()
                ))),
            },
            "number" => match value {
                Value::Number(n) => Ok(Value::Number(n.clone())),
                Value::String(s) => s
                    .parse::<f64>()
                    .map(|n| json!(n))
                    .map_err(|_| {
                        Error::ValidationError(format!("Cannot convert '{}' to number", s))
                    }),
                _ => Err(Error::ValidationError(format!(
                    "Cannot convert {} to number",
                    value.get_type()
                ))),
            },
            "boolean" => match value {
                Value::Bool(b) => Ok(Value::Bool(*b)),
                Value::String(s) => match s.to_lowercase().as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    _ => Err(Error::ValidationError(format!(
                        "Cannot convert '{}' to boolean",
                        s
                    ))),
                },
                _ => Err(Error::ValidationError(format!(
                    "Cannot convert {} to boolean",
                    value.get_type()
                ))),
            },
            "array" => match value {
                Value::Array(_) => Ok(value.clone()),
                _ => Err(Error::ValidationError(format!(
                    "Cannot convert {} to array",
                    value.get_type()
                ))),
            },
            "object" => match value {
                Value::Object(_) => Ok(value.clone()),
                _ => Err(Error::ValidationError(format!(
                    "Cannot convert {} to object",
                    value.get_type()
                ))),
            },
            _ => Err(Error::ValidationError(format!(
                "Unknown target type: {}",
                target_type
            ))),
        }
    }
}

/// Extension trait for getting JSON type name
trait JsonType {
    fn get_type(&self) -> &'static str;
}

impl JsonType for Value {
    fn get_type(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

impl Default for ToolMarshaler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marshal_input_valid_object() {
        let marshaler = ToolMarshaler::new();
        let input = json!({"param1": "value1", "param2": 42});
        let result = marshaler.marshal_input(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), input);
    }

    #[test]
    fn test_marshal_input_null() {
        let marshaler = ToolMarshaler::new();
        let input = Value::Null;
        let result = marshaler.marshal_input(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_marshal_input_invalid_array() {
        let marshaler = ToolMarshaler::new();
        let input = json!([1, 2, 3]);
        let result = marshaler.marshal_input(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_unmarshal_output_valid() {
        let marshaler = ToolMarshaler::new();
        let output = json!({"result": "success"});
        let result = marshaler.unmarshal_output(&output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_type_string_to_number() {
        let marshaler = ToolMarshaler::new();
        let value = Value::String("42".to_string());
        let result = marshaler.convert_type(&value, "number");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(42.0));
    }

    #[test]
    fn test_convert_type_number_to_string() {
        let marshaler = ToolMarshaler::new();
        let value = json!(42);
        let result = marshaler.convert_type(&value, "string");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("42".to_string()));
    }

    #[test]
    fn test_convert_type_string_to_boolean() {
        let marshaler = ToolMarshaler::new();
        let value = Value::String("true".to_string());
        let result = marshaler.convert_type(&value, "boolean");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_convert_type_invalid() {
        let marshaler = ToolMarshaler::new();
        let value = json!({"key": "value"});
        let result = marshaler.convert_type(&value, "number");
        assert!(result.is_err());
    }
}
