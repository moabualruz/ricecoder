//! MCP Read Tool Helpers
//!
//! File reading utilities for the MCP read tool, including binary detection
//! and content formatting.

use std::path::Path;

/// Binary file extensions that should not be read as text
const BINARY_EXTENSIONS: &[&str] = &[
    "exe", "dll", "so", "dylib", "bin", "o", "a", "zip", "tar", "gz", "bz2", "xz",
    "7z", "jpg", "jpeg", "png", "gif", "bmp", "ico", "mp3", "mp4", "avi", "mov", "pdf",
    "doc", "docx",
];

/// Check if a file is binary based on extension and content heuristics
pub fn is_binary_file(path: &Path, content: &[u8]) -> bool {
    // Check extension first
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if BINARY_EXTENSIONS.contains(&ext_str.as_ref()) {
            return true;
        }
    }

    // Check content heuristic (null bytes in first 8KB)
    let check_len = content.len().min(8192);
    content[..check_len].contains(&0)
}

/// Format file content for MCP output with line numbers and pagination
pub fn format_file_content_for_mcp(
    _path: &str,
    content: &str,
    offset: usize,
    limit: usize,
) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = offset;
    let end = (start + limit).min(total_lines);

    let mut output = String::new();
    output.push_str("<file>\n");

    for (idx, line) in lines.iter().enumerate().skip(start).take(end - start) {
        let line_num = idx + 1;
        // Truncate lines longer than 2000 characters
        let formatted_line = if line.len() > 2000 {
            format!("{}...(line truncated)", &line[..2000])
        } else {
            line.to_string()
        };
        output.push_str(&format!("{:05}| {}\n", line_num, formatted_line));
    }

    if end < total_lines {
        output.push_str(&format!(
            "(File has more lines - total {} lines)\n",
            total_lines
        ));
    } else {
        output.push_str(&format!("(End of file - total {} lines)\n", total_lines));
    }

    output.push_str("</file>");
    output
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
