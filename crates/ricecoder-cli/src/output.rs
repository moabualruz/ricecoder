// Output formatting and styling
// Adapted from automation/src/utils/colors.rs

use colored::Colorize;

/// Output styling configuration
pub struct OutputStyle {
    pub use_colors: bool,
}

impl Default for OutputStyle {
    fn default() -> Self {
        Self {
            use_colors: atty::is(atty::Stream::Stdout),
        }
    }
}

impl OutputStyle {
    /// Format success message
    pub fn success(&self, msg: &str) -> String {
        if self.use_colors {
            format!("{} {}", "✓".green().bold(), msg)
        } else {
            format!("✓ {}", msg)
        }
    }

    /// Format error message
    pub fn error(&self, msg: &str) -> String {
        if self.use_colors {
            format!("{} {}", "✗".red().bold(), msg)
        } else {
            format!("✗ {}", msg)
        }
    }

    /// Format warning message
    pub fn warning(&self, msg: &str) -> String {
        if self.use_colors {
            format!("{} {}", "⚠".yellow(), msg)
        } else {
            format!("⚠ {}", msg)
        }
    }

    /// Format info message
    pub fn info(&self, msg: &str) -> String {
        if self.use_colors {
            format!("{} {}", "ℹ".blue(), msg)
        } else {
            format!("ℹ {}", msg)
        }
    }

    /// Format code block
    pub fn code(&self, code: &str) -> String {
        if self.use_colors {
            code.cyan().to_string()
        } else {
            code.to_string()
        }
    }

    /// Format code block with language-specific syntax highlighting
    pub fn code_block(&self, code: &str, language: &str) -> String {
        // For now, just apply basic syntax highlighting
        // In a full implementation, this would use syntect for proper highlighting
        if self.use_colors {
            match language {
                "rust" | "rs" => code.cyan().to_string(),
                "python" | "py" => code.yellow().to_string(),
                "javascript" | "js" | "typescript" | "ts" => code.yellow().to_string(),
                "json" => code.cyan().to_string(),
                "yaml" | "yml" => code.cyan().to_string(),
                _ => code.to_string(),
            }
        } else {
            code.to_string()
        }
    }

    /// Format prompt
    pub fn prompt(&self, prompt: &str) -> String {
        if self.use_colors {
            format!("{} ", prompt.magenta().bold())
        } else {
            format!("{} ", prompt)
        }
    }

    /// Format header
    pub fn header(&self, title: &str) -> String {
        if self.use_colors {
            title.bold().to_string()
        } else {
            title.to_string()
        }
    }

    /// Format error with suggestions
    pub fn error_with_suggestion(&self, error: &str, suggestion: &str) -> String {
        let error_msg = self.error(error);
        let suggestion_msg = self.info(&format!("Suggestion: {}", suggestion));
        format!("{}\n{}", error_msg, suggestion_msg)
    }

    /// Format error with context
    pub fn error_with_context(&self, error: &str, context: &str) -> String {
        let error_msg = self.error(error);
        let context_msg = self.info(&format!("Context: {}", context));
        format!("{}\n{}", error_msg, context_msg)
    }

    /// Format verbose error with details
    pub fn error_verbose(&self, error: &str, details: &str) -> String {
        let error_msg = self.error(error);
        let details_msg = format!("\n{}", details);
        format!("{}{}", error_msg, details_msg)
    }

    /// Format error with multiple suggestions
    pub fn error_with_suggestions(&self, error: &str, suggestions: &[&str]) -> String {
        let mut output = self.error(error);
        if !suggestions.is_empty() {
            output.push_str("\n\n💡 Suggestions:");
            for (i, suggestion) in suggestions.iter().enumerate() {
                output.push_str(&format!("\n  {}. {}", i + 1, suggestion));
            }
        }
        output
    }

    /// Format error with documentation link
    pub fn error_with_docs(&self, error: &str, doc_url: &str) -> String {
        format!(
            "{}\n\n📖 Learn more: {}",
            self.error(error),
            doc_url
        )
    }

    /// Format a section header
    pub fn section(&self, title: &str) -> String {
        if self.use_colors {
            format!(
                "\n{}\n{}",
                title.bold().underline(),
                "─".repeat(title.len())
            )
        } else {
            format!("\n{}\n{}", title, "─".repeat(title.len()))
        }
    }

    /// Format a list item
    pub fn list_item(&self, item: &str) -> String {
        format!("  • {}", item)
    }

    /// Format a numbered list item
    pub fn numbered_item(&self, number: usize, item: &str) -> String {
        format!("  {}. {}", number, item)
    }

    /// Format a key-value pair
    pub fn key_value(&self, key: &str, value: &str) -> String {
        if self.use_colors {
            format!("  {}: {}", key.bold(), value)
        } else {
            format!("  {}: {}", key, value)
        }
    }

    /// Format a tip/hint
    pub fn tip(&self, tip: &str) -> String {
        if self.use_colors {
            format!("{} {}", "💡".yellow(), tip)
        } else {
            format!("💡 {}", tip)
        }
    }

    /// Format a link
    pub fn link(&self, text: &str, url: &str) -> String {
        if self.use_colors {
            format!("{} ({})", text.cyan(), url.cyan())
        } else {
            format!("{} ({})", text, url)
        }
    }
}

/// Print formatted output
pub fn print_success(msg: &str) {
    let style = OutputStyle::default();
    println!("{}", style.success(msg));
}

pub fn print_error(msg: &str) {
    let style = OutputStyle::default();
    eprintln!("{}", style.error(msg));
}

pub fn print_warning(msg: &str) {
    let style = OutputStyle::default();
    println!("{}", style.warning(msg));
}

pub fn print_info(msg: &str) {
    let style = OutputStyle::default();
    println!("{}", style.info(msg));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_style_without_colors() {
        let style = OutputStyle { use_colors: false };
        assert_eq!(style.success("test"), "✓ test");
        assert_eq!(style.error("test"), "✗ test");
        assert_eq!(style.warning("test"), "⚠ test");
        assert_eq!(style.info("test"), "ℹ test");
    }

    #[test]
    fn test_output_formatting_idempotence() {
        let style = OutputStyle { use_colors: false };
        let msg = "test message";
        let formatted1 = style.success(msg);
        let formatted2 = style.success(msg);
        assert_eq!(formatted1, formatted2);
    }

    #[test]
    fn test_error_with_suggestion() {
        let style = OutputStyle { use_colors: false };
        let result = style.error_with_suggestion("File not found", "Check the file path");
        assert!(result.contains("✗ File not found"));
        assert!(result.contains("Suggestion: Check the file path"));
    }

    #[test]
    fn test_error_with_context() {
        let style = OutputStyle { use_colors: false };
        let result = style.error_with_context("Invalid config", "in ~/.ricecoder/config.toml");
        assert!(result.contains("✗ Invalid config"));
        assert!(result.contains("Context: in ~/.ricecoder/config.toml"));
    }

    #[test]
    fn test_section_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.section("Configuration");
        assert!(result.contains("Configuration"));
        assert!(result.contains("─"));
    }

    #[test]
    fn test_list_item_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.list_item("First item");
        assert!(result.contains("•"));
        assert!(result.contains("First item"));
    }

    #[test]
    fn test_key_value_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.key_value("key", "value");
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_error_with_suggestions() {
        let style = OutputStyle { use_colors: false };
        let suggestions = vec!["Try this", "Or that"];
        let result = style.error_with_suggestions("Something failed", &suggestions);
        assert!(result.contains("✗ Something failed"));
        assert!(result.contains("Suggestions:"));
        assert!(result.contains("1. Try this"));
        assert!(result.contains("2. Or that"));
    }

    #[test]
    fn test_error_with_docs() {
        let style = OutputStyle { use_colors: false };
        let result = style.error_with_docs("File not found", "https://docs.example.com");
        assert!(result.contains("✗ File not found"));
        assert!(result.contains("https://docs.example.com"));
    }

    #[test]
    fn test_numbered_item_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.numbered_item(1, "First item");
        assert!(result.contains("1. First item"));
    }

    #[test]
    fn test_tip_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.tip("This is a helpful tip");
        assert!(result.contains("💡"));
        assert!(result.contains("This is a helpful tip"));
    }

    #[test]
    fn test_link_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.link("Documentation", "https://docs.example.com");
        assert!(result.contains("Documentation"));
        assert!(result.contains("https://docs.example.com"));
    }
}
