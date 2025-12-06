//! Error types for MCP integration

use std::fmt;
use thiserror::Error;

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during MCP operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("MCP server error: {0}")]
    ServerError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Timeout after {0}ms")]
    TimeoutError(u64),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Permission denied for tool: {0}")]
    PermissionDenied(String),

    #[error("Tool execution failed: {0}")]
    ExecutionError(String),

    #[error("Parameter validation failed: {0}")]
    ParameterValidationError(String),

    #[error("Output validation failed: {0}")]
    OutputValidationError(String),

    #[error("Naming conflict: {0}")]
    NamingConflict(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Server disconnected: {0}")]
    ServerDisconnected(String),

    #[error("Reconnection failed: {0}")]
    ReconnectionFailed(String),

    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(String),

    #[error("Tool execution interrupted")]
    ExecutionInterrupted,

    #[error("Invalid tool parameters: {0}")]
    InvalidToolParameters(String),

    #[error("Invalid tool output: {0}")]
    InvalidToolOutput(String),

    #[error("Configuration validation failed: {0}")]
    ConfigValidationError(String),

    #[error("Tool registration failed: {0}")]
    ToolRegistrationError(String),

    #[error("Multiple naming conflicts detected: {0}")]
    MultipleNamingConflicts(String),
}

impl Error {
    /// Creates a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Error::ServerError(msg) => format!("MCP server error: {}", msg),
            Error::ConfigError(msg) => format!("Configuration error: {}. Please check your configuration files.", msg),
            Error::ValidationError(msg) => format!("Validation error: {}", msg),
            Error::TimeoutError(ms) => format!("Operation timed out after {}ms. Please try again or increase the timeout.", ms),
            Error::ToolNotFound(tool_id) => format!("Tool '{}' not found. Please check the tool ID and try again.", tool_id),
            Error::PermissionDenied(tool_id) => format!("Permission denied for tool '{}'. Contact your administrator.", tool_id),
            Error::ExecutionError(msg) => format!("Tool execution failed: {}. Please check the tool parameters and try again.", msg),
            Error::ParameterValidationError(msg) => format!("Invalid tool parameters: {}. Please provide valid parameters.", msg),
            Error::OutputValidationError(msg) => format!("Tool returned invalid output: {}. Please contact the tool provider.", msg),
            Error::NamingConflict(msg) => format!("Naming conflict detected: {}. Please use a qualified tool name.", msg),
            Error::ConnectionError(msg) => format!("Connection error: {}. Please check your network connection.", msg),
            Error::SerializationError(msg) => format!("Serialization error: {}. Please check the data format.", msg),
            Error::StorageError(msg) => format!("Storage error: {}. Please check your storage configuration.", msg),
            Error::IoError(msg) => format!("IO error: {}. Please check file permissions.", msg),
            Error::InternalError(msg) => format!("Internal error: {}. Please contact support.", msg),
            Error::ServerDisconnected(server_id) => format!("Server '{}' disconnected. Attempting to reconnect...", server_id),
            Error::ReconnectionFailed(msg) => format!("Failed to reconnect to server: {}. Please check the server status.", msg),
            Error::MaxRetriesExceeded(msg) => format!("Maximum reconnection attempts exceeded: {}. Please check the server.", msg),
            Error::ExecutionInterrupted => "Tool execution was interrupted. Please try again.".to_string(),
            Error::InvalidToolParameters(msg) => format!("Invalid tool parameters: {}. Please provide valid parameters.", msg),
            Error::InvalidToolOutput(msg) => format!("Tool returned invalid output: {}. Please contact the tool provider.", msg),
            Error::ConfigValidationError(msg) => format!("Configuration validation failed: {}. Please fix your configuration.", msg),
            Error::ToolRegistrationError(msg) => format!("Tool registration failed: {}. Please check the tool definition.", msg),
            Error::MultipleNamingConflicts(msg) => format!("Multiple naming conflicts detected: {}. Please use qualified tool names.", msg),
        }
    }

    /// Gets the error type for logging
    pub fn error_type(&self) -> &'static str {
        match self {
            Error::ServerError(_) => "ServerError",
            Error::ConfigError(_) => "ConfigError",
            Error::ValidationError(_) => "ValidationError",
            Error::TimeoutError(_) => "TimeoutError",
            Error::ToolNotFound(_) => "ToolNotFound",
            Error::PermissionDenied(_) => "PermissionDenied",
            Error::ExecutionError(_) => "ExecutionError",
            Error::ParameterValidationError(_) => "ParameterValidationError",
            Error::OutputValidationError(_) => "OutputValidationError",
            Error::NamingConflict(_) => "NamingConflict",
            Error::ConnectionError(_) => "ConnectionError",
            Error::SerializationError(_) => "SerializationError",
            Error::StorageError(_) => "StorageError",
            Error::IoError(_) => "IoError",
            Error::InternalError(_) => "InternalError",
            Error::ServerDisconnected(_) => "ServerDisconnected",
            Error::ReconnectionFailed(_) => "ReconnectionFailed",
            Error::MaxRetriesExceeded(_) => "MaxRetriesExceeded",
            Error::ExecutionInterrupted => "ExecutionInterrupted",
            Error::InvalidToolParameters(_) => "InvalidToolParameters",
            Error::InvalidToolOutput(_) => "InvalidToolOutput",
            Error::ConfigValidationError(_) => "ConfigValidationError",
            Error::ToolRegistrationError(_) => "ToolRegistrationError",
            Error::MultipleNamingConflicts(_) => "MultipleNamingConflicts",
        }
    }

    /// Checks if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::TimeoutError(_)
                | Error::ConnectionError(_)
                | Error::ServerDisconnected(_)
                | Error::ExecutionInterrupted
        )
    }

    /// Checks if this is a permanent error
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            Error::ToolNotFound(_)
                | Error::PermissionDenied(_)
                | Error::NamingConflict(_)
                | Error::MultipleNamingConflicts(_)
        )
    }
}

/// Error context for detailed error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub tool_id: Option<String>,
    pub parameters: Option<String>,
    pub server_id: Option<String>,
    pub stack_trace: Option<String>,
}

impl ErrorContext {
    /// Creates a new error context
    pub fn new() -> Self {
        Self {
            tool_id: None,
            parameters: None,
            server_id: None,
            stack_trace: None,
        }
    }

    /// Sets the tool ID
    pub fn with_tool_id(mut self, tool_id: String) -> Self {
        self.tool_id = Some(tool_id);
        self
    }

    /// Sets the parameters
    pub fn with_parameters(mut self, parameters: String) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Sets the server ID
    pub fn with_server_id(mut self, server_id: String) -> Self {
        self.server_id = Some(server_id);
        self
    }

    /// Sets the stack trace
    pub fn with_stack_trace(mut self, stack_trace: String) -> Self {
        self.stack_trace = Some(stack_trace);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrorContext {{")?;
        if let Some(tool_id) = &self.tool_id {
            write!(f, " tool_id: {}", tool_id)?;
        }
        if let Some(parameters) = &self.parameters {
            write!(f, " parameters: {}", parameters)?;
        }
        if let Some(server_id) = &self.server_id {
            write!(f, " server_id: {}", server_id)?;
        }
        if let Some(stack_trace) = &self.stack_trace {
            write!(f, " stack_trace: {}", stack_trace)?;
        }
        write!(f, " }}")
    }
}

/// Protocol message types for MCP communication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub tool_id: String,
    pub parameters: serde_json::Value,
}

/// Result of a tool execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Error response from tool execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolError {
    pub tool_id: String,
    pub error: String,
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl ToolError {
    /// Creates a new tool error
    pub fn new(tool_id: String, error: String, error_type: String) -> Self {
        Self {
            tool_id,
            error,
            error_type,
            context: None,
        }
    }

    /// Sets the error context
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

/// Structured error log entry for comprehensive error logging
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorLogEntry {
    pub timestamp: String,
    pub error_type: String,
    pub message: String,
    pub tool_id: Option<String>,
    pub server_id: Option<String>,
    pub parameters: Option<String>,
    pub stack_trace: Option<String>,
    pub is_recoverable: bool,
    pub retry_count: Option<u32>,
}

impl ErrorLogEntry {
    /// Creates a new error log entry
    pub fn new(error_type: String, message: String) -> Self {
        Self {
            timestamp: chrono::Local::now().to_rfc3339(),
            error_type,
            message,
            tool_id: None,
            server_id: None,
            parameters: None,
            stack_trace: None,
            is_recoverable: false,
            retry_count: None,
        }
    }

    /// Sets the tool ID
    pub fn with_tool_id(mut self, tool_id: String) -> Self {
        self.tool_id = Some(tool_id);
        self
    }

    /// Sets the server ID
    pub fn with_server_id(mut self, server_id: String) -> Self {
        self.server_id = Some(server_id);
        self
    }

    /// Sets the parameters
    pub fn with_parameters(mut self, parameters: String) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Sets the stack trace
    pub fn with_stack_trace(mut self, stack_trace: String) -> Self {
        self.stack_trace = Some(stack_trace);
        self
    }

    /// Sets whether the error is recoverable
    pub fn with_recoverable(mut self, recoverable: bool) -> Self {
        self.is_recoverable = recoverable;
        self
    }

    /// Sets the retry count
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = Some(count);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_message() {
        let error = Error::ToolNotFound("test-tool".to_string());
        let msg = error.user_message();
        assert!(msg.contains("test-tool"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_error_type() {
        let error = Error::TimeoutError(5000);
        assert_eq!(error.error_type(), "TimeoutError");
    }

    #[test]
    fn test_error_is_recoverable() {
        assert!(Error::TimeoutError(5000).is_recoverable());
        assert!(Error::ConnectionError("test".to_string()).is_recoverable());
        assert!(!Error::ToolNotFound("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_error_is_permanent() {
        assert!(Error::ToolNotFound("test".to_string()).is_permanent());
        assert!(Error::PermissionDenied("test".to_string()).is_permanent());
        assert!(!Error::TimeoutError(5000).is_permanent());
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new()
            .with_tool_id("test-tool".to_string())
            .with_parameters("param1=value1".to_string())
            .with_server_id("server1".to_string());

        assert_eq!(context.tool_id, Some("test-tool".to_string()));
        assert_eq!(context.parameters, Some("param1=value1".to_string()));
        assert_eq!(context.server_id, Some("server1".to_string()));
    }

    #[test]
    fn test_tool_error_with_context() {
        let error = ToolError::new(
            "test-tool".to_string(),
            "Execution failed".to_string(),
            "ExecutionError".to_string(),
        )
        .with_context("Additional context".to_string());

        assert_eq!(error.tool_id, "test-tool");
        assert_eq!(error.context, Some("Additional context".to_string()));
    }

    #[test]
    fn test_error_log_entry() {
        let entry = ErrorLogEntry::new(
            "ExecutionError".to_string(),
            "Tool execution failed".to_string(),
        )
        .with_tool_id("test-tool".to_string())
        .with_recoverable(true)
        .with_retry_count(3);

        assert_eq!(entry.error_type, "ExecutionError");
        assert_eq!(entry.tool_id, Some("test-tool".to_string()));
        assert!(entry.is_recoverable);
        assert_eq!(entry.retry_count, Some(3));
    }
}
