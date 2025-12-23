//! MCP Protocol Mappers
//!
//! Maps between MCP protocol types and application layer types.
//! This module serves as the bridge between the external MCP interface
//! and our internal domain-driven application layer.
//!
//! # Architecture
//! ```text
//! MCP Input → Mapper → Application Request → Use Case → Application Response → Mapper → MCP Output
//! ```

use anyhow::{Context, Result};
use ricegrep::application::{
    AppError, IoOperation,
    use_cases::{EditFileRequest, WriteFileRequest},
};

// ============================================================================
// Request Mappers: MCP Input → Application Request
// ============================================================================

/// Map EditToolInput to EditFileRequest
pub fn map_edit_request(
    file_path: &str,
    old_string: &str,
    new_string: &str,
    replace_all: bool,
) -> EditFileRequest {
    EditFileRequest {
        file_path: file_path.to_string(),
        pattern: old_string.to_string(),
        replacement: new_string.to_string(),
        is_regex: false, // MCP edit uses literal matching
        dry_run: false,  // MCP edit always executes
    }
}

/// Map WriteToolInput to WriteFileRequest
pub fn map_write_request(file_path: &str, content: &str) -> WriteFileRequest {
    WriteFileRequest::new(file_path, content)
}

// ============================================================================
// Error Mappers: Application Error → User-Friendly Message
// ============================================================================

/// Map AppError to user-friendly error message for MCP consumers
pub fn map_error_to_message(error: &AppError) -> String {
    match error {
        AppError::Validation { message } => {
            format!("Validation error: {}", message)
        }
        AppError::Domain(domain_err) => {
            format!("Domain error: {}", domain_err)
        }
        AppError::Io { operation, path, source } => {
            map_io_error_message(operation, path, source)
        }
        AppError::Index { operation, message } => {
            format!("Index {} failed: {}", operation, message)
        }
        AppError::Search { query, message } => {
            format!("Search for '{}' failed: {}", query, message)
        }
        AppError::Config(msg) => {
            format!("Configuration error: {}", msg)
        }
    }
}

/// Map I/O errors to detailed user-friendly messages
fn map_io_error_message(operation: &IoOperation, path: &str, source: &std::io::Error) -> String {
    match (operation, source.kind()) {
        (IoOperation::Read, std::io::ErrorKind::NotFound) => {
            format!("File not found: {}. Check the path and try again.", path)
        }
        (IoOperation::Read, std::io::ErrorKind::PermissionDenied) => {
            format!("Permission denied: {}. Check file permissions.", path)
        }
        (IoOperation::Read, std::io::ErrorKind::InvalidData) => {
            format!("File contains invalid UTF-8: {}. Check file encoding.", path)
        }
        (IoOperation::Write, std::io::ErrorKind::NotFound) => {
            format!("Cannot write to path: {} - parent directory does not exist.", path)
        }
        (IoOperation::Write, std::io::ErrorKind::PermissionDenied) => {
            format!("Permission denied writing to: {}. Check directory permissions.", path)
        }
        (IoOperation::Delete, std::io::ErrorKind::PermissionDenied) => {
            format!("Permission denied deleting: {}. Check file permissions.", path)
        }
        (IoOperation::Create, std::io::ErrorKind::PermissionDenied) => {
            format!("Permission denied creating: {}. Check directory permissions.", path)
        }
        _ => {
            format!(
                "Failed to {} '{}': {}. Check if file is locked or disk is full.",
                operation, path, source
            )
        }
    }
}

// ============================================================================
// Response Formatters: Application Response → MCP Output String
// ============================================================================

/// Format edit response for MCP output
pub fn format_edit_response(
    file_path: &str,
    old_string: &str,
    new_string: &str,
    matches_replaced: usize,
    _replace_all: bool,
) -> String {
    // Consistent format: always use "occurrence(s)" for backward compatibility
    format!(
        "Edited {}: replaced {} occurrence(s) of '{}' with '{}'",
        file_path, matches_replaced, old_string, new_string
    )
}

/// Format write response for MCP output
pub fn format_write_response(file_path: &str, bytes_written: usize, content: &str) -> String {
    let line_count = content.lines().count();
    format!(
        "Wrote {} ({} bytes, {} lines)",
        file_path, bytes_written, line_count
    )
}

// ============================================================================
// Async Wrappers: Run sync use cases in async context
// ============================================================================

use tokio::task;

/// Execute a sync operation in a blocking context
/// This allows our sync application layer to work with async MCP handlers
pub async fn run_blocking<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    task::spawn_blocking(f)
        .await
        .context("Task panicked")?
        .map_err(|e| anyhow::anyhow!(map_error_to_message(&e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_edit_request() {
        let request = map_edit_request("test.rs", "old", "new", false);
        
        assert_eq!(request.file_path, "test.rs");
        assert_eq!(request.pattern, "old");
        assert_eq!(request.replacement, "new");
        assert!(!request.is_regex);
        assert!(!request.dry_run);
    }

    #[test]
    fn test_map_write_request() {
        let request = map_write_request("output.txt", "hello world");
        
        assert_eq!(request.file_path, "output.txt");
        assert_eq!(request.content, "hello world");
    }

    #[test]
    fn test_format_edit_response() {
        let response = format_edit_response("test.rs", "foo", "bar", 3, true);
        assert!(response.contains("3 occurrence(s)"));
        assert!(response.contains("foo"));
        assert!(response.contains("bar"));
    }

    #[test]
    fn test_format_edit_response_single() {
        let response = format_edit_response("test.rs", "foo", "bar", 1, false);
        assert!(response.contains("1 occurrence(s)"));
    }

    #[test]
    fn test_format_write_response() {
        let response = format_write_response("test.txt", 100, "line1\nline2\nline3");
        assert!(response.contains("100 bytes"));
        assert!(response.contains("3 lines"));
    }

    #[test]
    fn test_map_error_validation() {
        let error = AppError::Validation { 
            message: "empty path".to_string() 
        };
        let message = map_error_to_message(&error);
        assert!(message.contains("Validation"));
        assert!(message.contains("empty path"));
    }

    #[test]
    fn test_map_io_error_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let error = AppError::Io {
            operation: IoOperation::Read,
            path: "missing.rs".to_string(),
            source: io_err,
        };
        let message = map_error_to_message(&error);
        assert!(message.contains("File not found"));
        assert!(message.contains("missing.rs"));
    }

    #[test]
    fn test_map_io_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let error = AppError::Io {
            operation: IoOperation::Write,
            path: "protected.rs".to_string(),
            source: io_err,
        };
        let message = map_error_to_message(&error);
        assert!(message.contains("Permission denied"));
        assert!(message.contains("protected.rs"));
    }
}
