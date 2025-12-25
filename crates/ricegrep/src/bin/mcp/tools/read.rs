//! MCP Read Tool Helpers
//!
//! File reading utilities for the MCP read tool.
//! Delegates to ricecoder-tools for formatting and file type utilities.

use std::path::Path;

/// Check if a file is binary based on extension and content heuristics.
///
/// Delegates to `ricecoder_tools::filetype::is_binary_file`.
pub fn is_binary_file(path: &Path, content: &[u8]) -> bool {
    ricecoder_tools::filetype::is_binary_file(path, content)
}

/// Format file content for MCP output with line numbers and pagination.
///
/// Delegates to `ricecoder_tools::format::format_file_content_for_mcp`.
pub fn format_file_content_for_mcp(
    path: &str,
    content: &str,
    offset: usize,
    limit: usize,
) -> String {
    ricecoder_tools::format::format_file_content_for_mcp(path, content, offset, limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_binary_file_by_extension() {
        let path = PathBuf::from("test.exe");
        assert!(is_binary_file(&path, b"anything"));

        let path = PathBuf::from("test.png");
        assert!(is_binary_file(&path, b"anything"));

        let path = PathBuf::from("test.rs");
        assert!(!is_binary_file(&path, b"fn main() {}"));
    }

    #[test]
    fn test_is_binary_file_by_content() {
        let path = PathBuf::from("test.txt");
        
        // Text content
        assert!(!is_binary_file(&path, b"Hello, World!"));
        
        // Binary content (contains null bytes)
        assert!(is_binary_file(&path, b"Hello\x00World"));
    }

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
        let content = format!("{}", long_line);
        let output = format_file_content_for_mcp("test.txt", &content, 0, 10);
        
        assert!(output.contains("...(line truncated)"));
    }
}
