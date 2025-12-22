//! Key redaction and safety utilities
//!
//! This module provides utilities for redacting sensitive information (API keys, credentials)
//! from logs, error messages, and debug output to prevent accidental credential leakage.

use std::sync::OnceLock;

use regex::Regex;

/// Redaction filter for removing sensitive information from strings
pub struct RedactionFilter {
    /// Patterns to redact (regex patterns for API keys, tokens, etc.)
    patterns: Vec<RedactionPattern>,
}

/// A pattern to redact with its replacement
struct RedactionPattern {
    /// Regex pattern to match
    regex: Regex,
    /// Replacement string (e.g., "[REDACTED]")
    replacement: String,
}

impl RedactionFilter {
    /// Create a new redaction filter with default patterns
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // OpenAI API keys (sk-*)
                RedactionPattern {
                    regex: Regex::new(r"sk-[A-Za-z0-9]{20,}").unwrap(),
                    replacement: "[REDACTED_OPENAI_KEY]".to_string(),
                },
                // Anthropic API keys (sk-ant-*)
                RedactionPattern {
                    regex: Regex::new(r"sk-ant-[A-Za-z0-9]{20,}").unwrap(),
                    replacement: "[REDACTED_ANTHROPIC_KEY]".to_string(),
                },
                // Generic API keys (api_key=*, apiKey=*, api-key=*)
                RedactionPattern {
                    regex: Regex::new(r"(?i)(api[_-]?key|token|secret|password)\s*=\s*[^\s,;]+")
                        .unwrap(),
                    replacement: "$1=[REDACTED]".to_string(),
                },
                // Bearer tokens
                RedactionPattern {
                    regex: Regex::new(r"(?i)bearer\s+[A-Za-z0-9._\-/+=]+").unwrap(),
                    replacement: "Bearer [REDACTED]".to_string(),
                },
                // Authorization headers
                RedactionPattern {
                    regex: Regex::new(r"(?i)authorization:\s*[^\s,;]+").unwrap(),
                    replacement: "Authorization: [REDACTED]".to_string(),
                },
                // Environment variable patterns
                RedactionPattern {
                    regex: Regex::new(
                        r"(?i)(OPENAI|ANTHROPIC|GOOGLE|GROQ|MISTRAL)_API_KEY\s*=\s*[^\s,;]+",
                    )
                    .unwrap(),
                    replacement: "$1_API_KEY=[REDACTED]".to_string(),
                },
            ],
        }
    }

    /// Add a custom redaction pattern
    pub fn add_pattern(&mut self, pattern: &str, replacement: &str) -> Result<(), String> {
        let regex = Regex::new(pattern).map_err(|e| e.to_string())?;
        self.patterns.push(RedactionPattern {
            regex,
            replacement: replacement.to_string(),
        });
        Ok(())
    }

    /// Redact sensitive information from a string
    pub fn redact(&self, input: &str) -> String {
        let mut result = input.to_string();
        for pattern in &self.patterns {
            result = pattern
                .regex
                .replace_all(&result, &pattern.replacement)
                .to_string();
        }
        result
    }

    /// Check if a string contains any sensitive information
    pub fn contains_sensitive_info(&self, input: &str) -> bool {
        self.patterns.iter().any(|p| p.regex.is_match(input))
    }
}

impl Default for RedactionFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the global redaction filter (singleton)
pub fn get_redaction_filter() -> &'static RedactionFilter {
    static FILTER: OnceLock<RedactionFilter> = OnceLock::new();
    FILTER.get_or_init(RedactionFilter::new)
}

/// Redact sensitive information from a string using the global filter
pub fn redact(input: &str) -> String {
    get_redaction_filter().redact(input)
}

/// Check if a string contains sensitive information
pub fn contains_sensitive_info(input: &str) -> bool {
    get_redaction_filter().contains_sensitive_info(input)
}

/// A wrapper type that automatically redacts its Debug output
pub struct Redacted<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> std::fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", redact(self.0.as_ref()))
    }
}

impl<T: AsRef<str>> std::fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", redact(self.0.as_ref()))
    }
}
