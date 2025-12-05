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
    /// ```text
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

    #[test]
    fn test_parse_empty_body() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test
---"#;

        let result = parser.parse(content).unwrap();
        assert_eq!(result.frontmatter, Some("name: test".to_string()));
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_parse_complex_yaml_frontmatter() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: complex-agent
description: Complex agent
model: gpt-4
temperature: 0.7
max_tokens: 2000
tools:
  - tool1
  - tool2
---
# Complex Content
With multiple lines
And formatting"#;

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert!(fm.contains("tools:"));
        assert!(fm.contains("- tool1"));
    }

    #[test]
    fn test_parse_frontmatter_with_special_characters() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test-agent
description: "Agent with special chars: @#$%^&*()"
---
Content"#;

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.is_some());
        assert!(result.frontmatter.unwrap().contains("@#$%^&*()"));
    }

    #[test]
    fn test_parse_frontmatter_with_quotes() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: "test-agent"
description: 'Single quoted'
---
Content"#;

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.is_some());
    }

    #[test]
    fn test_parse_body_with_code_blocks() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test
---
# Content

```rust
fn main() {
    println!("Hello");
}
```

More content"#;

        let result = parser.parse(content).unwrap();
        assert!(result.content.contains("```rust"));
        assert!(result.content.contains("fn main()"));
    }

    #[test]
    fn test_parse_body_with_frontmatter_like_content() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test
---
# Content

This mentions --- but it's in the body
So it should be fine"#;

        let result = parser.parse(content).unwrap();
        assert!(result.content.contains("---"));
    }

    #[test]
    fn test_parse_with_context_error_message() {
        let parser = MarkdownParser::new();
        let content = r#"---
---
Content"#;
        let path = Path::new("test.agent.md");

        let result = parser.parse_with_context(content, Some(path));
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("test.agent.md"));
    }

    #[test]
    fn test_parse_consistency() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test-agent
description: Test
---
Body content"#;

        let result1 = parser.parse(content).unwrap();
        let result2 = parser.parse(content).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_parse_only_frontmatter_delimiter() {
        let parser = MarkdownParser::new();
        let content = "---";

        // Single "---" is treated as opening delimiter with no closing
        let result = parser.parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_delimiters_in_body() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: test
---
First section
---
Second section
---
Third section"#;

        let result = parser.parse(content).unwrap();
        assert_eq!(result.frontmatter, Some("name: test".to_string()));
        assert!(result.content.contains("First section"));
        assert!(result.content.contains("Second section"));
        assert!(result.content.contains("Third section"));
    }

    #[test]
    fn test_parse_very_long_frontmatter() {
        let parser = MarkdownParser::new();
        let mut frontmatter = String::from("---\n");
        for i in 0..100 {
            frontmatter.push_str(&format!("field{}: value{}\n", i, i));
        }
        frontmatter.push_str("---\nBody");

        let result = parser.parse(&frontmatter).unwrap();
        assert!(result.frontmatter.is_some());
        assert!(result.frontmatter.unwrap().contains("field99"));
    }

    #[test]
    fn test_parse_very_long_body() {
        let parser = MarkdownParser::new();
        let mut body = String::from("# Content\n");
        for i in 0..1000 {
            body.push_str(&format!("Line {}\n", i));
        }
        let content = format!("---\nname: test\n---\n{}", body);

        let result = parser.parse(&content).unwrap();
        assert!(result.content.contains("Line 999"));
    }

    #[test]
    fn test_parse_unicode_content() {
        let parser = MarkdownParser::new();
        let content = r#"---
name: 测试代理
description: 日本語のテスト
---
# 内容
Ελληνικά
العربية"#;

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.unwrap().contains("测试代理"));
        assert!(result.content.contains("Ελληνικά"));
    }

    #[test]
    fn test_parse_windows_line_endings() {
        let parser = MarkdownParser::new();
        let content = "---\r\nname: test\r\n---\r\nBody";

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.is_some());
    }

    #[test]
    fn test_parse_mixed_line_endings() {
        let parser = MarkdownParser::new();
        let content = "---\nname: test\r\n---\nBody";

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.is_some());
    }

    #[test]
    fn test_parse_tabs_in_frontmatter() {
        let parser = MarkdownParser::new();
        let content = "---\nname:\ttest\n---\nBody";

        let result = parser.parse(content).unwrap();
        assert!(result.frontmatter.is_some());
    }

    #[test]
    fn test_parse_empty_content() {
        let parser = MarkdownParser::new();
        let content = "";

        let result = parser.parse(content).unwrap();
        assert_eq!(result.frontmatter, None);
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_parse_only_whitespace() {
        let parser = MarkdownParser::new();
        let content = "   \n  \n   ";

        let result = parser.parse(content).unwrap();
        assert_eq!(result.frontmatter, None);
    }
}
