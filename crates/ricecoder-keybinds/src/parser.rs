//! Keybind configuration parsers for JSON and Markdown formats

use std::{collections::HashMap, sync::Arc};

use crate::{error::ParseError, models::Keybind};

/// Trait for parsing keybind configurations
pub trait KeybindParser: Send + Sync {
    /// Parse keybind configuration from content
    fn parse(&self, content: &str) -> Result<Vec<Keybind>, ParseError>;
}

/// Registry for keybind parsers supporting multiple formats
pub struct ParserRegistry {
    parsers: HashMap<String, Arc<dyn KeybindParser>>,
}

impl ParserRegistry {
    /// Create a new parser registry with default parsers
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        parsers.insert(
            "json".to_string(),
            Arc::new(JsonKeybindParser) as Arc<dyn KeybindParser>,
        );
        parsers.insert(
            "markdown".to_string(),
            Arc::new(MarkdownKeybindParser) as Arc<dyn KeybindParser>,
        );
        parsers.insert(
            "md".to_string(),
            Arc::new(MarkdownKeybindParser) as Arc<dyn KeybindParser>,
        );

        ParserRegistry { parsers }
    }

    /// Register a custom parser for a format
    pub fn register(&mut self, format: impl Into<String>, parser: Arc<dyn KeybindParser>) {
        self.parsers.insert(format.into(), parser);
    }

    /// Get a parser for a specific format
    pub fn get_parser(&self, format: &str) -> Option<Arc<dyn KeybindParser>> {
        self.parsers.get(format).cloned()
    }

    /// Auto-detect format and parse content
    pub fn parse_auto(&self, content: &str) -> Result<Vec<Keybind>, ParseError> {
        // Try JSON first
        if let Ok(keybinds) = self
            .get_parser("json")
            .ok_or_else(|| ParseError::InvalidJson("No JSON parser available".to_string()))?
            .parse(content)
        {
            return Ok(keybinds);
        }

        // Fall back to Markdown
        self.get_parser("markdown")
            .ok_or_else(|| ParseError::InvalidMarkdown("No Markdown parser available".to_string()))?
            .parse(content)
    }

    /// Parse content with explicit format
    pub fn parse(&self, content: &str, format: &str) -> Result<Vec<Keybind>, ParseError> {
        let parser = self
            .get_parser(format)
            .ok_or_else(|| ParseError::InvalidJson(format!("Unknown format: {}", format)))?;
        parser.parse(content)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON keybind parser
pub struct JsonKeybindParser;

impl KeybindParser for JsonKeybindParser {
    fn parse(&self, content: &str) -> Result<Vec<Keybind>, ParseError> {
        let value: serde_json::Value =
            serde_json::from_str(content).map_err(|e| ParseError::InvalidJson(e.to_string()))?;

        let keybinds_array = value
            .get("keybinds")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ParseError::MissingField("keybinds".to_string()))?;

        let mut keybinds = Vec::new();
        for (idx, item) in keybinds_array.iter().enumerate() {
            let keybind: Keybind =
                serde_json::from_value(item.clone()).map_err(|e| ParseError::LineError {
                    line: idx + 1,
                    message: e.to_string(),
                })?;

            // Validate required fields
            if keybind.action_id.is_empty() {
                return Err(ParseError::LineError {
                    line: idx + 1,
                    message: "Missing action_id".to_string(),
                });
            }
            if keybind.key.is_empty() {
                return Err(ParseError::LineError {
                    line: idx + 1,
                    message: "Missing key".to_string(),
                });
            }

            keybinds.push(keybind);
        }

        Ok(keybinds)
    }
}

/// Markdown keybind parser
pub struct MarkdownKeybindParser;

impl KeybindParser for MarkdownKeybindParser {
    fn parse(&self, content: &str) -> Result<Vec<Keybind>, ParseError> {
        let mut keybinds = Vec::new();
        let mut current_category = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and code blocks
            if trimmed.is_empty() || trimmed.starts_with("```") {
                continue;
            }

            // Extract category from headers
            if let Some(category) = trimmed.strip_prefix("## ") {
                current_category = category.trim().to_string();
                continue;
            }

            // Parse keybind entries: `action_id`: key - description
            if trimmed.starts_with("- `") {
                let keybind = parse_markdown_entry(trimmed, &current_category, line_num + 1)?;
                keybinds.push(keybind);
            }
        }

        Ok(keybinds)
    }
}

/// Parse a single markdown keybind entry
/// Format: - `action_id`: key - description
fn parse_markdown_entry(
    line: &str,
    category: &str,
    line_num: usize,
) -> Result<Keybind, ParseError> {
    // Remove leading "- `"
    let content = line
        .strip_prefix("- `")
        .ok_or_else(|| ParseError::LineError {
            line: line_num,
            message: "Invalid markdown format".to_string(),
        })?;

    // Find the closing backtick
    let backtick_pos = content.find('`').ok_or_else(|| ParseError::LineError {
        line: line_num,
        message: "Missing closing backtick".to_string(),
    })?;

    let action_id = content[..backtick_pos].to_string();

    // Find the colon
    let rest = &content[backtick_pos + 1..];
    let colon_pos = rest.find(':').ok_or_else(|| ParseError::LineError {
        line: line_num,
        message: "Missing colon after action_id".to_string(),
    })?;

    // Find the dash separator
    let key_part = &rest[colon_pos + 1..];
    let dash_pos = key_part.find(" - ").ok_or_else(|| ParseError::LineError {
        line: line_num,
        message: "Missing ' - ' separator".to_string(),
    })?;

    let key = key_part[..dash_pos].trim().to_string();
    let description = key_part[dash_pos + 3..].trim().to_string();

    Ok(Keybind {
        action_id,
        key,
        category: category.to_string(),
        description,
        is_default: false,
        contexts: Vec::new(),
    })
}
