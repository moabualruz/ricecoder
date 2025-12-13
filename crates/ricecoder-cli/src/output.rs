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


