//! MCP Protocol Validation and Error Handling
//!
//! Provides validation for MCP protocol messages, error handling,
//! and protocol compliance checking.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::error::{Error, Result};
use crate::transport::{MCPMessage, MCPRequest, MCPResponse, MCPNotification, MCPError, MCPErrorData};

/// MCP protocol validator
pub struct MCPProtocolValidator {
    method_name_pattern: Regex,
    id_pattern: Regex,
}

impl MCPProtocolValidator {
    /// Create a new protocol validator
    pub fn new() -> Result<Self> {
        Ok(Self {
            method_name_pattern: Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]*$")
                .map_err(|e| Error::ValidationError(format!("Invalid regex: {}", e)))?,
            id_pattern: Regex::new(r"^[a-zA-Z0-9._-]+$")
                .map_err(|e| Error::ValidationError(format!("Invalid regex: {}", e)))?,
        })
    }

    /// Validate an MCP message
    pub fn validate_message(&self, message: &MCPMessage) -> Result<()> {
        match message {
            MCPMessage::Request(req) => self.validate_request(req),
            MCPMessage::Response(resp) => self.validate_response(resp),
            MCPMessage::Notification(notif) => self.validate_notification(notif),
            MCPMessage::Error(err) => self.validate_error(err),
        }
    }

    /// Validate an MCP request
    pub fn validate_request(&self, request: &MCPRequest) -> Result<()> {
        // Validate request ID
        if request.id.is_empty() {
            return Err(Error::ValidationError("Request ID cannot be empty".to_string()));
        }

        if request.id.len() > 256 {
            return Err(Error::ValidationError("Request ID too long".to_string()));
        }

        if !self.id_pattern.is_match(&request.id) {
            return Err(Error::ValidationError("Invalid request ID format".to_string()));
        }

        // Validate method name
        if request.method.is_empty() {
            return Err(Error::ValidationError("Method name cannot be empty".to_string()));
        }

        if request.method.len() > 256 {
            return Err(Error::ValidationError("Method name too long".to_string()));
        }

        if !self.method_name_pattern.is_match(&request.method) {
            return Err(Error::ValidationError("Invalid method name format".to_string()));
        }

        // Validate parameters (basic JSON validation is handled by serde)
        self.validate_json_value(&request.params, "request parameters")?;

        debug!("Validated MCP request: {} -> {}", request.id, request.method);
        Ok(())
    }

    /// Validate an MCP response
    pub fn validate_response(&self, response: &MCPResponse) -> Result<()> {
        // Validate response ID
        if response.id.is_empty() {
            return Err(Error::ValidationError("Response ID cannot be empty".to_string()));
        }

        if response.id.len() > 256 {
            return Err(Error::ValidationError("Response ID too long".to_string()));
        }

        if !self.id_pattern.is_match(&response.id) {
            return Err(Error::ValidationError("Invalid response ID format".to_string()));
        }

        // Validate result
        self.validate_json_value(&response.result, "response result")?;

        debug!("Validated MCP response: {}", response.id);
        Ok(())
    }

    /// Validate an MCP notification
    pub fn validate_notification(&self, notification: &MCPNotification) -> Result<()> {
        // Validate method name
        if notification.method.is_empty() {
            return Err(Error::ValidationError("Notification method cannot be empty".to_string()));
        }

        if notification.method.len() > 256 {
            return Err(Error::ValidationError("Notification method too long".to_string()));
        }

        if !self.method_name_pattern.is_match(&notification.method) {
            return Err(Error::ValidationError("Invalid notification method format".to_string()));
        }

        // Validate parameters
        self.validate_json_value(&notification.params, "notification parameters")?;

        debug!("Validated MCP notification: {}", notification.method);
        Ok(())
    }

    /// Validate an MCP error
    pub fn validate_error(&self, error: &MCPError) -> Result<()> {
        // Validate error ID if present
        if let Some(ref id) = error.id {
            if id.is_empty() {
                return Err(Error::ValidationError("Error ID cannot be empty".to_string()));
            }

            if id.len() > 256 {
                return Err(Error::ValidationError("Error ID too long".to_string()));
            }

            if !self.id_pattern.is_match(id) {
                return Err(Error::ValidationError("Invalid error ID format".to_string()));
            }
        }

        // Validate error code (should be integer)
        // Error codes are validated by serde deserialization

        // Validate error message
        if error.error.message.is_empty() {
            return Err(Error::ValidationError("Error message cannot be empty".to_string()));
        }

        if error.error.message.len() > 1024 {
            return Err(Error::ValidationError("Error message too long".to_string()));
        }

        // Validate error data if present
        if let Some(ref data) = error.error.data {
            self.validate_json_value(data, "error data")?;
        }

        debug!("Validated MCP error: {} - {}", error.error.code, error.error.message);
        Ok(())
    }

    /// Validate JSON value for protocol compliance
    fn validate_json_value(&self, value: &serde_json::Value, context: &str) -> Result<()> {
        // Check for reasonable size limits
        let json_string = serde_json::to_string(value)
            .map_err(|e| Error::ValidationError(format!("JSON serialization failed: {}", e)))?;

        if json_string.len() > 10 * 1024 * 1024 { // 10MB limit
            return Err(Error::ValidationError(format!(
                "{} too large ({} bytes, max 10MB)",
                context,
                json_string.len()
            )));
        }

        // Check for deeply nested structures (potential DoS)
        self.validate_json_depth(value, 0, context)?;

        Ok(())
    }

    /// Validate JSON nesting depth
    fn validate_json_depth(&self, value: &serde_json::Value, current_depth: usize, context: &str) -> Result<()> {
        const MAX_DEPTH: usize = 32;

        if current_depth > MAX_DEPTH {
            return Err(Error::ValidationError(format!(
                "{} has too much nesting (max depth: {})",
                context, MAX_DEPTH
            )));
        }

        match value {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.validate_json_depth(item, current_depth + 1, context)?;
                }
            }
            serde_json::Value::Object(obj) => {
                for (_, value) in obj {
                    self.validate_json_depth(value, current_depth + 1, context)?;
                }
            }
            _ => {} // Other types don't increase depth
        }

        Ok(())
    }
}

impl Default for MCPProtocolValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create default MCP protocol validator")
    }
}

/// MCP protocol error handler
pub struct MCPErrorHandler {
    error_codes: HashMap<i32, String>,
}

impl MCPErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        let mut error_codes = HashMap::new();

        // Standard MCP error codes
        error_codes.insert(-32700, "Parse error".to_string());
        error_codes.insert(-32600, "Invalid request".to_string());
        error_codes.insert(-32601, "Method not found".to_string());
        error_codes.insert(-32602, "Invalid params".to_string());
        error_codes.insert(-32603, "Internal error".to_string());
        error_codes.insert(-32000, "Server error".to_string());

        // MCP-specific error codes
        error_codes.insert(-32001, "Tool not found".to_string());
        error_codes.insert(-32002, "Tool execution failed".to_string());
        error_codes.insert(-32003, "Permission denied".to_string());
        error_codes.insert(-32004, "Invalid tool parameters".to_string());

        Self { error_codes }
    }

    /// Handle an MCP error and convert to appropriate Error type
    pub fn handle_error(&self, mcp_error: &MCPError) -> Error {
        let code = mcp_error.error.code;
        let message = &mcp_error.error.message;

        match code {
            -32700 => Error::ValidationError(format!("Parse error: {}", message)),
            -32600 => Error::ValidationError(format!("Invalid request: {}", message)),
            -32601 => Error::ToolNotFound(message.clone()),
            -32602 => Error::ParameterValidationError(format!("Invalid params: {}", message)),
            -32603 => Error::ExecutionError(format!("Internal error: {}", message)),
            -32000 => Error::ServerError(format!("Server error: {}", message)),
            -32001 => Error::ToolNotFound(message.clone()),
            -32002 => Error::ExecutionError(format!("Tool execution failed: {}", message)),
            -32003 => Error::PermissionDenied(message.clone()),
            -32004 => Error::ParameterValidationError(format!("Invalid tool parameters: {}", message)),
            _ => {
                warn!("Unknown MCP error code: {} - {}", code, message);
                Error::ServerError(format!("MCP error {}: {}", code, message))
            }
        }
    }

    /// Create an MCP error response
    pub fn create_error_response(&self, request_id: Option<String>, error: &Error) -> MCPMessage {
        let (code, message) = match error {
            Error::ValidationError(msg) => (-32600, format!("Invalid request: {}", msg)),
            Error::ToolNotFound(name) => (-32001, format!("Tool not found: {}", name)),
            Error::ParameterValidationError(msg) => (-32004, format!("Invalid parameters: {}", msg)),
            Error::PermissionDenied(tool) => (-32003, format!("Permission denied for tool: {}", tool)),
            Error::ExecutionError(msg) => (-32002, format!("Execution failed: {}", msg)),
            Error::TimeoutError(ms) => (-32000, format!("Timeout after {}ms", ms)),
            Error::ServerError(msg) => (-32000, msg.clone()),
            Error::ConnectionError(msg) => (-32000, format!("Connection error: {}", msg)),
            _ => (-32603, format!("Internal error: {:?}", error)),
        };

        MCPMessage::Error(MCPError {
            id: request_id,
            error: MCPErrorData {
                code,
                message,
                data: None,
            },
        })
    }

    /// Get description for an error code
    pub fn get_error_description(&self, code: i32) -> Option<&str> {
        self.error_codes.get(&code).map(|s| s.as_str())
    }

    /// Check if an error code represents a client error (4xx)
    pub fn is_client_error(&self, code: i32) -> bool {
        code >= -32600 && code <= -32000
    }

    /// Check if an error code represents a server error (5xx)
    pub fn is_server_error(&self, code: i32) -> bool {
        code <= -32000
    }
}

impl Default for MCPErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP protocol compliance checker
pub struct MCPComplianceChecker {
    validator: MCPProtocolValidator,
    error_handler: MCPErrorHandler,
}

impl MCPComplianceChecker {
    /// Create a new compliance checker
    pub fn new() -> Self {
        Self {
            validator: MCPProtocolValidator::new().expect("Failed to create validator"),
            error_handler: MCPErrorHandler::new(),
        }
    }

    /// Check if a message is protocol compliant
    pub fn check_compliance(&self, message: &MCPMessage) -> Result<()> {
        // First validate the message structure
        self.validator.validate_message(message)?;

        // Additional compliance checks
        self.check_message_size(message)?;
        self.check_reserved_methods(message)?;

        Ok(())
    }

    /// Check message size limits
    fn check_message_size(&self, message: &MCPMessage) -> Result<()> {
        let json_size = serde_json::to_string(message)
            .map_err(|e| Error::ValidationError(format!("JSON serialization failed: {}", e)))?
            .len();

        // 1MB limit for individual messages
        if json_size > 1024 * 1024 {
            return Err(Error::ValidationError(format!(
                "Message too large: {} bytes (max 1MB)",
                json_size
            )));
        }

        Ok(())
    }

    /// Check for reserved method names
    fn check_reserved_methods(&self, message: &MCPMessage) -> Result<()> {
        let method = match message {
            MCPMessage::Request(req) => &req.method,
            MCPMessage::Notification(notif) => &notif.method,
            _ => return Ok(()), // Responses and errors don't have methods
        };

        // Reserved MCP method prefixes
        let reserved_prefixes = ["mcp.", "rpc."];

        for prefix in &reserved_prefixes {
            if method.starts_with(prefix) {
                return Err(Error::ValidationError(format!(
                    "Method name '{}' uses reserved prefix '{}'",
                    method, prefix
                )));
            }
        }

        Ok(())
    }

    /// Validate and handle an incoming message
    pub fn validate_and_handle(&self, message: &MCPMessage) -> Result<()> {
        match self.check_compliance(message) {
            Ok(_) => {
                debug!("Message passed compliance check");
                Ok(())
            }
            Err(e) => {
                warn!("Message failed compliance check: {:?}", e);
                Err(e)
            }
        }
    }
}

impl Default for MCPComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_validator_request() {
        let validator = MCPProtocolValidator::new().unwrap();

        let valid_request = MCPRequest {
            id: "req_123".to_string(),
            method: "tools.execute".to_string(),
            params: serde_json::json!({"tool": "grep", "args": ["pattern"]}),
        };

        assert!(validator.validate_request(&valid_request).is_ok());

        // Test invalid request ID
        let invalid_request = MCPRequest {
            id: "".to_string(),
            method: "tools.execute".to_string(),
            params: serde_json::json!({}),
        };

        assert!(validator.validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_protocol_validator_notification() {
        let validator = MCPProtocolValidator::new().unwrap();

        let valid_notification = MCPNotification {
            method: "tools.available".to_string(),
            params: serde_json::json!({"count": 5}),
        };

        assert!(validator.validate_notification(&valid_notification).is_ok());

        // Test invalid method name
        let invalid_notification = MCPNotification {
            method: "invalid method!".to_string(),
            params: serde_json::json!({}),
        };

        assert!(validator.validate_notification(&invalid_notification).is_err());
    }

    #[test]
    fn test_error_handler() {
        let handler = MCPErrorHandler::new();

        // Test known error codes
        assert_eq!(handler.get_error_description(-32601), Some("Method not found"));
        assert_eq!(handler.get_error_description(-32001), Some("Tool not found"));

        // Test error classification
        assert!(handler.is_client_error(-32600)); // Invalid request
        assert!(handler.is_server_error(-32000)); // Server error
    }

    #[test]
    fn test_compliance_checker() {
        let checker = MCPComplianceChecker::new();

        let valid_request = MCPMessage::Request(MCPRequest {
            id: "req_123".to_string(),
            method: "tools.execute".to_string(),
            params: serde_json::json!({"tool": "grep"}),
        });

        assert!(checker.check_compliance(&valid_request).is_ok());

        // Test reserved method prefix
        let invalid_request = MCPMessage::Request(MCPRequest {
            id: "req_123".to_string(),
            method: "mcp.internal".to_string(),
            params: serde_json::json!({}),
        });

        assert!(checker.check_compliance(&invalid_request).is_err());
    }

    #[test]
    fn test_json_depth_validation() {
        let validator = MCPProtocolValidator::new().unwrap();

        // Create deeply nested JSON
        let mut deep_value = serde_json::json!({"level": 0});
        let mut current = &mut deep_value;

        for i in 1..35 { // Exceed max depth of 32
            *current = serde_json::json!({"level": i, "nested": {}});
            if let Some(obj) = current.as_object_mut() {
                if let Some(nested) = obj.get_mut("nested") {
                    current = nested;
                }
            }
        }

        assert!(validator.validate_json_value(&deep_value, "test").is_err());
    }
}