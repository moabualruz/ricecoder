//! Formatting utilities for file content and tool responses
//!
//! This module provides reusable formatting utilities for:
//! - MCP file content formatting (line numbers, pagination, truncation)
//! - Tool response formatting (edit/write/read responses)
//!
//! For binary file detection and MIME types, see the `filetype` module.
//!
//! These utilities are designed to be used across different tools and contexts:
//! - MCP servers (ricegrep, ricecoder-mcp)
//! - TUI displays
//! - CLI output

use std::path::Path;

// Re-export binary detection from filetype for backward compatibility
pub use crate::filetype::{is_binary_content, is_binary_extension, is_binary_file};

// ============================================================================
// MCP File Content Formatting
// ============================================================================

/// Options for MCP file content formatting.
#[derive(Debug, Clone)]
pub struct McpFormatOptions {
    /// Starting line offset (0-based)
    pub offset: usize,
    /// Number of lines to include
    pub limit: usize,
    /// Maximum characters per line before truncation
    pub max_line_length: usize,
    /// Whether to wrap content in `<file>...</file>` tags
    pub wrap_in_tags: bool,
    /// Line number format width (default: 5 for "00001")
    pub line_number_width: usize,
}

impl Default for McpFormatOptions {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 2000,
            max_line_length: 2000,
            wrap_in_tags: true,
            line_number_width: 5,
        }
    }
}

/// Format file content for MCP output with line numbers and pagination.
///
/// Output format (with default options):
/// ```text
/// <file>
/// 00001| first line
/// 00002| second line
/// ...
/// (End of file - total N lines)
/// </file>
/// ```
pub fn format_file_content_for_mcp(
    _path: &str,
    content: &str,
    offset: usize,
    limit: usize,
) -> String {
    format_file_content_with_options(content, &McpFormatOptions {
        offset,
        limit,
        ..Default::default()
    })
}

/// Format file content with full options control.
pub fn format_file_content_with_options(content: &str, options: &McpFormatOptions) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = options.offset;
    let end = (start + options.limit).min(total_lines);

    let mut output = String::new();
    
    if options.wrap_in_tags {
        output.push_str("<file>\n");
    }

    for (idx, line) in lines.iter().enumerate().skip(start).take(end.saturating_sub(start)) {
        let line_num = idx + 1;
        
        // Truncate lines longer than max_line_length
        let formatted_line = if line.len() > options.max_line_length {
            format!("{}...(line truncated)", &line[..options.max_line_length])
        } else {
            line.to_string()
        };
        
        // Format with line number (e.g., "00001| ")
        output.push_str(&format!(
            "{:0width$}| {}\n",
            line_num,
            formatted_line,
            width = options.line_number_width
        ));
    }

    // Add footer
    if end < total_lines {
        output.push_str(&format!("(File has more lines - total {} lines)\n", total_lines));
    } else {
        output.push_str(&format!("(End of file - total {} lines)\n", total_lines));
    }

    if options.wrap_in_tags {
        output.push_str("</file>");
    }

    output
}

// ============================================================================
// Tool Response Formatting
// ============================================================================

/// Format edit operation response.
///
/// Returns a human-readable message like:
/// "Edited path/to/file.rs: replaced 3 occurrence(s) of 'old' with 'new'"
pub fn format_edit_response(
    file_path: &str,
    old_string: &str,
    new_string: &str,
    occurrences_replaced: usize,
    _replace_all: bool,
) -> String {
    format!(
        "Edited {}: replaced {} occurrence(s) of '{}' with '{}'",
        file_path, occurrences_replaced, old_string, new_string
    )
}

/// Format write operation response.
///
/// Returns a human-readable message like:
/// "Wrote path/to/file.rs (1234 bytes, 56 lines)"
pub fn format_write_response(file_path: &str, bytes_written: usize, content: &str) -> String {
    let line_count = content.lines().count();
    format!(
        "Wrote {} ({} bytes, {} lines)",
        file_path, bytes_written, line_count
    )
}

/// Format read operation response for errors.
///
/// Returns a user-friendly error message based on error kind.
pub fn format_read_error(file_path: &str, error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            format!("File not found: {}. Check the path and try again.", file_path)
        }
        std::io::ErrorKind::PermissionDenied => {
            format!("Permission denied: {}. Check file permissions.", file_path)
        }
        std::io::ErrorKind::InvalidData => {
            format!("File contains invalid UTF-8: {}. Check file encoding.", file_path)
        }
        _ => {
            format!(
                "File is locked or inaccessible: {}. Close other applications and retry. (Error: {})",
                file_path, error
            )
        }
    }
}

/// Format write operation error.
pub fn format_write_error(file_path: &str, error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            format!("Cannot write to path: {} - parent directory does not exist.", file_path)
        }
        std::io::ErrorKind::PermissionDenied => {
            format!("Permission denied writing to: {}. Check directory permissions.", file_path)
        }
        std::io::ErrorKind::InvalidInput => {
            format!("Invalid file path: {}", file_path)
        }
        _ => {
            format!(
                "Cannot write file: {} - disk may be full or file locked. (Error: {})",
                file_path, error
            )
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Binary detection tests are in filetype module.
    // This module tests formatting functionality only.

    #[test]
    fn test_format_file_content_basic() {
        let content = "line 1\nline 2\nline 3";
        let output = format_file_content_for_mcp("test.txt", content, 0, 10);

        assert!(output.contains("<file>"));
        assert!(output.contains("</file>"));
        assert!(output.contains("00001| line 1"));
        assert!(output.contains("00002| line 2"));
        assert!(output.contains("00003| line 3"));
        assert!(output.contains("(End of file - total 3 lines)"));
    }

    #[test]
    fn test_format_file_content_with_offset() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        let output = format_file_content_for_mcp("test.txt", content, 2, 2);

        assert!(output.contains("00003| line 3"));
        assert!(output.contains("00004| line 4"));
        assert!(!output.contains("00001| line 1"));
        assert!(!output.contains("00005| line 5"));
        assert!(output.contains("(File has more lines - total 5 lines)"));
    }

    #[test]
    fn test_format_file_content_truncates_long_lines() {
        let long_line = "x".repeat(2500);
        let output = format_file_content_for_mcp("test.txt", &long_line, 0, 10);

        assert!(output.contains("...(line truncated)"));
    }

    #[test]
    fn test_format_file_content_no_tags() {
        let content = "line 1\nline 2";
        let options = McpFormatOptions {
            wrap_in_tags: false,
            ..Default::default()
        };
        let output = format_file_content_with_options(content, &options);

        assert!(!output.contains("<file>"));
        assert!(!output.contains("</file>"));
        assert!(output.contains("00001| line 1"));
    }

    #[test]
    fn test_format_edit_response() {
        let response = format_edit_response("test.rs", "foo", "bar", 3, true);
        assert_eq!(response, "Edited test.rs: replaced 3 occurrence(s) of 'foo' with 'bar'");

        let response = format_edit_response("test.rs", "old", "new", 1, false);
        assert_eq!(response, "Edited test.rs: replaced 1 occurrence(s) of 'old' with 'new'");
    }

    #[test]
    fn test_format_write_response() {
        let content = "line1\nline2\nline3";
        let response = format_write_response("test.txt", 17, content);
        assert_eq!(response, "Wrote test.txt (17 bytes, 3 lines)");
    }

    #[test]
    fn test_format_read_error() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let msg = format_read_error("missing.txt", &err);
        assert!(msg.contains("File not found"));
        assert!(msg.contains("missing.txt"));

        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let msg = format_read_error("protected.txt", &err);
        assert!(msg.contains("Permission denied"));
    }
}
