//! Formatting utilities with ANSI color support
//!
//! This module provides file content formatting with optional ANSI colors
//! for terminal output. Colors are automatically disabled when output is
//! not a TTY (e.g., when piped to another command or to MCP consumers).
//!
//! # Color Scheme
//! - Line numbers: Cyan (dim)
//! - File paths: Green (bold)
//! - Search matches: Red (bold)
//! - Warnings: Yellow
//! - Errors: Red
//! - Tags (`<file>`, `</file>`): Blue (dim)

use colored::{ColoredString, Colorize};

/// Configuration for output formatting
#[derive(Debug, Clone, Copy)]
pub struct FormatConfig {
    /// Whether to use ANSI colors
    pub use_colors: bool,
    /// Maximum line length before truncation (0 = no truncation)
    pub max_line_length: usize,
    /// Width for line number padding
    pub line_num_width: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            use_colors: false,
            max_line_length: 2000,
            line_num_width: 5,
        }
    }
}

impl FormatConfig {
    /// Create config with colors auto-detected based on stdout TTY
    pub fn auto() -> Self {
        Self {
            use_colors: atty::is(atty::Stream::Stdout),
            ..Default::default()
        }
    }

    /// Create config with colors explicitly enabled
    pub fn with_colors() -> Self {
        Self {
            use_colors: true,
            ..Default::default()
        }
    }

    /// Create config with colors explicitly disabled (for MCP, pipes, etc.)
    pub fn without_colors() -> Self {
        Self {
            use_colors: false,
            ..Default::default()
        }
    }
}

/// Color helper that respects config
struct Painter {
    use_colors: bool,
}

impl Painter {
    fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }

    /// Format line number in cyan (dim)
    fn line_num(&self, num: usize, width: usize) -> String {
        let formatted = format!("{:0width$}", num, width = width);
        if self.use_colors {
            formatted.cyan().dimmed().to_string()
        } else {
            formatted
        }
    }

    /// Format separator (|) in dim
    fn separator(&self) -> String {
        if self.use_colors {
            "|".dimmed().to_string()
        } else {
            "|".to_string()
        }
    }

    /// Format file tag (<file>, </file>) in blue dim
    fn tag(&self, text: &str) -> String {
        if self.use_colors {
            text.blue().dimmed().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format status message (e.g., "End of file") in dim
    fn status(&self, text: &str) -> String {
        if self.use_colors {
            text.dimmed().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format truncation marker in yellow
    fn truncation(&self, text: &str) -> String {
        if self.use_colors {
            text.yellow().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format file path in green bold
    pub fn file_path(&self, path: &str) -> String {
        if self.use_colors {
            path.green().bold().to_string()
        } else {
            path.to_string()
        }
    }

    /// Format search match in red bold
    pub fn match_highlight(&self, text: &str) -> String {
        if self.use_colors {
            text.red().bold().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format error message in red
    pub fn error(&self, text: &str) -> String {
        if self.use_colors {
            text.red().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format warning message in yellow
    pub fn warning(&self, text: &str) -> String {
        if self.use_colors {
            text.yellow().to_string()
        } else {
            text.to_string()
        }
    }
}

/// Format file content with line numbers and optional colors
///
/// # Arguments
/// * `path` - File path for display (not used in output, available for future use)
/// * `content` - The file content to format
/// * `offset` - Starting line index (0-based)
/// * `limit` - Maximum number of lines to include
/// * `config` - Formatting configuration
///
/// # Returns
/// Formatted string with line numbers, wrapped in `<file>` tags
pub fn format_file_content(
    _path: &str,
    content: &str,
    offset: usize,
    limit: usize,
    config: &FormatConfig,
) -> String {
    let painter = Painter::new(config.use_colors);
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = offset;
    let end = (start + limit).min(total_lines);

    let mut output = String::new();
    output.push_str(&painter.tag("<file>"));
    output.push('\n');

    for (idx, line) in lines.iter().enumerate().skip(start).take(end - start) {
        let line_num = idx + 1;

        // Truncate lines longer than max_line_length
        let formatted_line = if config.max_line_length > 0 && line.len() > config.max_line_length {
            let truncated = &line[..config.max_line_length];
            let marker = painter.truncation("...(line truncated)");
            format!("{}{}", truncated, marker)
        } else {
            line.to_string()
        };

        // Format: "00001| content"
        output.push_str(&painter.line_num(line_num, config.line_num_width));
        output.push_str(&painter.separator());
        output.push(' ');
        output.push_str(&formatted_line);
        output.push('\n');
    }

    // Status line
    let status = if end < total_lines {
        format!("(File has more lines - total {} lines)", total_lines)
    } else {
        format!("(End of file - total {} lines)", total_lines)
    };
    output.push_str(&painter.status(&status));
    output.push('\n');

    output.push_str(&painter.tag("</file>"));
    output
}

/// Format a search result line with optional match highlighting
///
/// # Arguments
/// * `path` - File path
/// * `line_num` - Line number
/// * `line_content` - The line content
/// * `match_start` - Start position of match (None if no highlighting)
/// * `match_end` - End position of match (None if no highlighting)
/// * `config` - Formatting configuration
pub fn format_search_result(
    path: &str,
    line_num: usize,
    line_content: &str,
    match_range: Option<(usize, usize)>,
    config: &FormatConfig,
) -> String {
    let painter = Painter::new(config.use_colors);

    let highlighted_content = if let Some((start, end)) = match_range {
        if start < line_content.len() && end <= line_content.len() && start < end {
            let before = &line_content[..start];
            let matched = &line_content[start..end];
            let after = &line_content[end..];
            format!("{}{}{}", before, painter.match_highlight(matched), after)
        } else {
            line_content.to_string()
        }
    } else {
        line_content.to_string()
    };

    format!(
        "{}:{}:{}",
        painter.file_path(path),
        painter.line_num(line_num, 0),
        highlighted_content
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_config_default() {
        let config = FormatConfig::default();
        assert!(!config.use_colors);
        assert_eq!(config.max_line_length, 2000);
        assert_eq!(config.line_num_width, 5);
    }

    #[test]
    fn test_format_config_with_colors() {
        let config = FormatConfig::with_colors();
        assert!(config.use_colors);
    }

    #[test]
    fn test_format_config_without_colors() {
        let config = FormatConfig::without_colors();
        assert!(!config.use_colors);
    }

    #[test]
    fn test_format_file_content_basic() {
        let content = "line 1\nline 2\nline 3";
        let config = FormatConfig::without_colors();
        let result = format_file_content("test.rs", content, 0, 10, &config);

        assert!(result.contains("<file>"));
        assert!(result.contains("</file>"));
        assert!(result.contains("00001| line 1"));
        assert!(result.contains("00002| line 2"));
        assert!(result.contains("00003| line 3"));
        assert!(result.contains("(End of file - total 3 lines)"));
    }

    #[test]
    fn test_format_file_content_with_offset() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        let config = FormatConfig::without_colors();
        let result = format_file_content("test.rs", content, 2, 2, &config);

        assert!(!result.contains("00001|")); // line 1 skipped
        assert!(!result.contains("00002|")); // line 2 skipped
        assert!(result.contains("00003| line 3"));
        assert!(result.contains("00004| line 4"));
        assert!(!result.contains("00005|")); // line 5 exceeds limit
        assert!(result.contains("(File has more lines - total 5 lines)"));
    }

    #[test]
    fn test_format_file_content_truncation() {
        let long_line = "x".repeat(3000);
        let content = format!("short\n{}\nend", long_line);
        let config = FormatConfig::without_colors();
        let result = format_file_content("test.rs", &content, 0, 10, &config);

        assert!(result.contains("...(line truncated)"));
        // Should contain first 2000 chars of the long line
        assert!(result.contains(&"x".repeat(2000)));
        // Should NOT contain 3000 x's (truncated)
        assert!(!result.contains(&"x".repeat(3000)));
    }

    #[test]
    fn test_format_search_result_no_highlight() {
        let config = FormatConfig::without_colors();
        let result = format_search_result("src/main.rs", 42, "fn main() {", None, &config);

        assert_eq!(result, "src/main.rs:42:fn main() {");
    }

    #[test]
    fn test_format_search_result_with_highlight_no_color() {
        let config = FormatConfig::without_colors();
        let result =
            format_search_result("src/main.rs", 42, "fn main() {", Some((3, 7)), &config);

        // Without colors, should just be the same (no ANSI codes)
        assert_eq!(result, "src/main.rs:42:fn main() {");
    }

    #[test]
    fn test_format_search_result_with_highlight_colors() {
        // Force colors for this test (colored crate disables in non-TTY)
        colored::control::set_override(true);
        
        let config = FormatConfig::with_colors();
        let result =
            format_search_result("src/main.rs", 42, "fn main() {", Some((3, 7)), &config);

        // With colors forced, should contain ANSI codes (check for escape sequence)
        assert!(result.contains("\x1b["), "Expected ANSI codes in: {}", result);
        // Should still contain the text
        assert!(result.contains("main"));
        
        colored::control::unset_override();
    }

    #[test]
    fn test_painter_line_num_no_color() {
        let painter = Painter::new(false);
        assert_eq!(painter.line_num(42, 5), "00042");
    }

    #[test]
    fn test_painter_line_num_with_color() {
        // Force colors for this test
        colored::control::set_override(true);
        
        let painter = Painter::new(true);
        let result = painter.line_num(42, 5);
        // Should contain ANSI escape codes
        assert!(result.contains("\x1b["), "Expected ANSI codes in: {}", result);
        // Should still contain the number
        assert!(result.contains("00042"));
        
        colored::control::unset_override();
    }

    #[test]
    fn test_painter_file_path_with_color() {
        // Force colors for this test
        colored::control::set_override(true);
        
        let painter = Painter::new(true);
        let result = painter.file_path("src/main.rs");
        // Should contain ANSI escape codes
        assert!(result.contains("\x1b["), "Expected ANSI codes in: {}", result);
        // Should still contain the path
        assert!(result.contains("src/main.rs"));
        
        colored::control::unset_override();
    }

    #[test]
    fn test_painter_match_highlight() {
        // Force colors for this test
        colored::control::set_override(true);
        
        let painter = Painter::new(true);
        let result = painter.match_highlight("TODO");
        // Should contain ANSI escape codes
        assert!(result.contains("\x1b["), "Expected ANSI codes in: {}", result);
        // Should still contain the text
        assert!(result.contains("TODO"));
        
        colored::control::unset_override();
    }
}
