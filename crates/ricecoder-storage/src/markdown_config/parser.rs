//! Markdown parser for extracting YAML frontmatter and content

use crate::markdown_config::error::{MarkdownConfigError, MarkdownConfigResult};
use crate::markdown_config::types::ParsedMarkdown;
use std::path::Path;

/// Parser for markdown files with YAML frontmatter
#[derive(Debug, Clone)]
pub struct MarkdownParser;

impl MarkdownParser {
    /// Create a new markdown parser
    pub fn new() -> Self {
        Self
    }

    /// Parse markdown content and extract frontmatter and body
    ///
    /// Expects frontmatter to be delimited by `---` at the start of the file.
    /// Format:
    /// ```
    /// ---
    /// yaml: frontmatter
    /// ---
    /// # Markdown content
    /// ```
    pub fn parse(&self, content: &str) -> MarkdownConfigResult<ParsedMarkdown> {
        self.parse_with_context(content, None)
    }

    /// Parse markdown content with file path context for better error messages
    pub fn parse_with_context(
        &self,
        content: &str,
        file_path: Option<&Path>,
    ) -> MarkdownConfigResult<ParsedMarkdown> {
        let trimmed = content.trim();

        // Check if content starts with frontmatter delimiter
        if !trimmed.starts_with("---") {
            // No frontmatter, entire content is body
            return Ok(ParsedMarkdown::new(None, content.to_string()));
        }

        // Find the closing delimiter
        let rest = &trimmed[3..]; // Skip opening "---"
        let closing_delimiter_pos = rest.find("---");

        match closing_delimiter_pos {
            Some(pos) => {
                // Extract frontmatter and body
                let frontmatter = rest[..pos].trim().to_string();
                let body_start = pos + 3; // Skip closing "---"
                let body = rest[body_start..].trim().to_string();

                // Validate that frontmatter is not empty
                if frontmatter.is_empty() {
                    let msg = match file_path {
                        Some(path) => format!(
                            "Frontmatter cannot be empty in {}",
                            path.display()
                        ),
                        None => "Frontmatter cannot be empty".to_string(),
                    };
                    return Err(MarkdownConfigError::parse_error(msg));
                }

                Ok(ParsedMarkdown::new(Some(frontmatter), body))
            }
            None => {
                // Opening delimiter found but no closing delimiter
                let msg = match file_path {
                    Some(path) => format!(
                        "Unclosed frontmatter in {}: found opening '---' but no closing '---'",
                        path.display()
                    ),
                    None => "Unclosed frontmatter: found opening '---' but no closing '---'"
                        .to_string(),
                };
                Err(MarkdownConfigError::parse_error(msg))
            }
        }
    }

}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_frontmatter() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test-agent
description: A test agent
---
# Test Content
This is the body"#;

        let result = parser.parse(content).unwrap();
        assert_eq!(
            result.frontmatter,
            Some("name: test-agent\ndescription: A test agent".to_string())
        );
        assert_eq!(result.content, "# Test Content\nThis is the body");
    }

    #[test]
    fn test_parse_without_frontmatter() {
        let parser = MarkdownParser::new();
        let content = "# Test Content\nThis is the body";

        let result = parser.parse(content).unwrap();
        assert_eq!(result.frontmatter, None);
        assert_eq!(result.content, "# Test Content\nThis is the body");
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let parser = MarkdownParser::new();
        let content = r#"---
---
# Test Content"#;

        let result = parser.parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_frontmatter() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test
# Test Content"#;

        let result = parser.parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() {
        let parser = MarkdownParser::new();
        let content = r#"  ---
name: test
  ---
  # Content"#;

        let result = parser.parse(content).unwrap();
        assert_eq!(result.frontmatter, Some("name: test".to_string()));
        assert_eq!(result.content, "# Content");
    }

    #[test]
    fn test_parse_multiline_frontmatter() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test-agent
description: A test agent
model: gpt-4
temperature: 0.7
---
# Test Content"#;

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert!(fm.contains("name: test-agent"));
        assert!(fm.contains("model: gpt-4"));
    }
}
