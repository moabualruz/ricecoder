//! Template syntax parser
//!
//! Parses template syntax, validates structure, extracts placeholders,
//! and detects conditionals, loops, and includes.

use crate::models::Placeholder;
use crate::templates::error::TemplateError;
use std::collections::HashSet;

/// Represents a parsed template element
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateElement {
    /// Plain text content
    Text(String),
    /// Placeholder: {{name}} or {{Name}} or {{name_snake}} etc.
    Placeholder(String),
    /// Conditional block: {{#if condition}}...{{/if}}
    Conditional {
        /// Condition expression
        condition: String,
        /// Content inside the conditional
        content: Vec<TemplateElement>,
    },
    /// Loop block: {{#each items}}...{{/each}}
    Loop {
        /// Variable name to iterate over
        variable: String,
        /// Content inside the loop
        content: Vec<TemplateElement>,
    },
    /// Include/partial: {{> partial_name}}
    Include(String),
}

/// Parsed template structure
#[derive(Debug, Clone)]
pub struct ParsedTemplate {
    /// Template elements
    pub elements: Vec<TemplateElement>,
    /// Extracted placeholders
    pub placeholders: Vec<Placeholder>,
    /// All placeholder names found
    pub placeholder_names: HashSet<String>,
}

/// Template parser
pub struct TemplateParser;

impl TemplateParser {
    /// Parse template content and extract structure
    ///
    /// # Arguments
    /// * `content` - Template content to parse
    ///
    /// # Returns
    /// Parsed template structure or error with line number
    pub fn parse(content: &str) -> Result<ParsedTemplate, TemplateError> {
        let mut parser = Parser::new(content);
        parser.parse()
    }

    /// Extract all placeholders from template content
    ///
    /// # Arguments
    /// * `content` - Template content
    ///
    /// # Returns
    /// Vector of placeholder names found
    pub fn extract_placeholders(content: &str) -> Result<Vec<String>, TemplateError> {
        let parsed = Self::parse(content)?;
        Ok(parsed.placeholder_names.into_iter().collect())
    }

    /// Detect if template has conditionals
    pub fn has_conditionals(content: &str) -> Result<bool, TemplateError> {
        Ok(content.contains("{{#if") && content.contains("{{/if}}"))
    }

    /// Detect if template has loops
    pub fn has_loops(content: &str) -> Result<bool, TemplateError> {
        Ok(content.contains("{{#each") && content.contains("{{/each}}"))
    }

    /// Detect if template has includes
    pub fn has_includes(content: &str) -> Result<bool, TemplateError> {
        Ok(content.contains("{{>"))
    }
}

/// Internal parser state machine
struct Parser {
    content: String,
    position: usize,
    line: usize,
    placeholder_names: HashSet<String>,
}

impl Parser {
    fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            position: 0,
            line: 1,
            placeholder_names: HashSet::new(),
        }
    }

    fn parse(&mut self) -> Result<ParsedTemplate, TemplateError> {
        let elements = self.parse_elements()?;
        let placeholders = self.extract_placeholder_definitions();

        Ok(ParsedTemplate {
            elements,
            placeholders,
            placeholder_names: self.placeholder_names.clone(),
        })
    }

    fn parse_elements(&mut self) -> Result<Vec<TemplateElement>, TemplateError> {
        let mut elements = Vec::new();

        while self.position < self.content.len() {
            if self.peek_char() == Some('{') && self.peek_ahead(1) == Some('{') {
                // Handle template syntax
                let element = self.parse_template_syntax()?;
                elements.push(element);
            } else {
                // Handle plain text
                let text = self.parse_text();
                if !text.is_empty() {
                    elements.push(TemplateElement::Text(text));
                }
            }
        }

        Ok(elements)
    }

    fn parse_template_syntax(&mut self) -> Result<TemplateElement, TemplateError> {
        self.consume_char()?; // {
        self.consume_char()?; // {

        match self.peek_char() {
            Some('#') => self.parse_block(),
            Some('>') => self.parse_include(),
            Some(_) => self.parse_placeholder(),
            None => Err(TemplateError::InvalidSyntax {
                line: self.line,
                message: "Unexpected end of template".to_string(),
            }),
        }
    }

    fn parse_block(&mut self) -> Result<TemplateElement, TemplateError> {
        self.consume_char()?; // #

        let block_type = self.read_until_whitespace_or_char('}')?;

        match block_type.as_str() {
            "if" => self.parse_conditional(),
            "each" => self.parse_loop(),
            _ => Err(TemplateError::InvalidSyntax {
                line: self.line,
                message: format!("Unknown block type: {}", block_type),
            }),
        }
    }

    fn parse_conditional(&mut self) -> Result<TemplateElement, TemplateError> {
        self.skip_whitespace();
        let condition = self.read_until_string("}}")?;
        self.consume_string("}}")?;

        let content = self.parse_until_end_block("if")?;

        Ok(TemplateElement::Conditional { condition, content })
    }

    fn parse_loop(&mut self) -> Result<TemplateElement, TemplateError> {
        self.skip_whitespace();
        let variable = self.read_until_whitespace_or_char('}')?;
        self.skip_whitespace();
        self.consume_string("}}")?;

        let content = self.parse_until_end_block("each")?;

        Ok(TemplateElement::Loop { variable, content })
    }

    fn parse_include(&mut self) -> Result<TemplateElement, TemplateError> {
        self.consume_char()?; // >
        self.skip_whitespace();

        let partial_name = self.read_until_string("}}")?;
        self.consume_string("}}")?;

        Ok(TemplateElement::Include(partial_name))
    }

    fn parse_placeholder(&mut self) -> Result<TemplateElement, TemplateError> {
        let placeholder_name = self.read_until_string("}}")?;
        self.consume_string("}}")?;

        // Extract base name (without case suffix)
        let base_name = self.extract_base_name(&placeholder_name);
        self.placeholder_names.insert(base_name);

        Ok(TemplateElement::Placeholder(placeholder_name))
    }

    fn parse_text(&mut self) -> String {
        let mut text = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == '{' && self.peek_ahead(1) == Some('{') {
                break;
            }

            if ch == '\n' {
                self.line += 1;
            }

            text.push(ch);
            self.position += 1;
        }

        text
    }

    fn parse_until_end_block(
        &mut self,
        block_type: &str,
    ) -> Result<Vec<TemplateElement>, TemplateError> {
        let mut elements = Vec::new();
        let end_marker = format!("{{{{/{}}}}}", block_type);

        while self.position < self.content.len() {
            if self.content[self.position..].starts_with(&end_marker) {
                self.position += end_marker.len();
                return Ok(elements);
            }

            if self.peek_char() == Some('{') && self.peek_ahead(1) == Some('{') {
                let element = self.parse_template_syntax()?;
                elements.push(element);
            } else {
                let text = self.parse_text();
                if !text.is_empty() {
                    elements.push(TemplateElement::Text(text));
                }
            }
        }

        Err(TemplateError::InvalidSyntax {
            line: self.line,
            message: format!("Unclosed {{{{#{}}}}}", block_type),
        })
    }

    fn extract_base_name(&self, placeholder: &str) -> String {
        // Remove case suffixes to get base name
        // {{Name}} -> name
        // {{name_snake}} -> name
        // {{name-kebab}} -> name
        // {{nameCamel}} -> name
        // {{NAME}} -> name

        let placeholder = placeholder.trim();

        // Handle snake_case - take everything before first underscore
        if let Some(pos) = placeholder.find('_') {
            return placeholder[..pos].to_lowercase();
        }

        // Handle kebab-case - take everything before first hyphen
        if let Some(pos) = placeholder.find('-') {
            return placeholder[..pos].to_lowercase();
        }

        // For all-uppercase (like NAME), just lowercase it
        if placeholder
            .chars()
            .all(|c| c.is_uppercase() || !c.is_alphabetic())
        {
            return placeholder.to_lowercase();
        }

        // Handle PascalCase and camelCase
        // Extract the base word (everything before the first uppercase letter after the first char)
        let mut base = String::new();
        let mut chars = placeholder.chars().peekable();

        // First character
        if let Some(first) = chars.next() {
            base.push(first.to_lowercase().next().unwrap_or(first));
        }

        // Remaining characters until we hit an uppercase letter
        while let Some(&ch) = chars.peek() {
            if ch.is_uppercase() {
                break;
            }
            base.push(ch);
            chars.next();
        }

        if base.is_empty() {
            placeholder.to_lowercase()
        } else {
            base
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.content.chars().nth(self.position)
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.content.chars().nth(self.position + offset)
    }

    fn consume_char(&mut self) -> Result<char, TemplateError> {
        match self.peek_char() {
            Some(ch) => {
                self.position += 1;
                if ch == '\n' {
                    self.line += 1;
                }
                Ok(ch)
            }
            None => Err(TemplateError::InvalidSyntax {
                line: self.line,
                message: "Unexpected end of template".to_string(),
            }),
        }
    }

    fn consume_string(&mut self, s: &str) -> Result<(), TemplateError> {
        for ch in s.chars() {
            if self.consume_char()? != ch {
                return Err(TemplateError::InvalidSyntax {
                    line: self.line,
                    message: format!("Expected '{}'", s),
                });
            }
        }
        Ok(())
    }

    fn read_until_string(&mut self, delimiter: &str) -> Result<String, TemplateError> {
        let mut result = String::new();

        while self.position < self.content.len() {
            if self.content[self.position..].starts_with(delimiter) {
                return Ok(result);
            }

            result.push(self.consume_char()?);
        }

        Err(TemplateError::InvalidSyntax {
            line: self.line,
            message: format!("Unterminated template element, expected '{}'", delimiter),
        })
    }

    fn read_until_whitespace_or_char(&mut self, ch: char) -> Result<String, TemplateError> {
        let mut result = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_whitespace() || c == ch {
                return Ok(result);
            }
            result.push(self.consume_char()?);
        }

        Ok(result)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                }
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn extract_placeholder_definitions(&self) -> Vec<Placeholder> {
        self.placeholder_names
            .iter()
            .map(|name| Placeholder {
                name: name.clone(),
                description: format!("Placeholder: {}", name),
                default: None,
                required: true,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_placeholder() {
        let content = "Hello {{name}}";
        let result = TemplateParser::parse(content).unwrap();
        assert_eq!(result.placeholder_names.len(), 1);
        assert!(result.placeholder_names.contains("name"));
    }

    #[test]
    fn test_parse_multiple_placeholders() {
        let content = "{{Name}} is a {{type}}";
        let result = TemplateParser::parse(content).unwrap();
        assert_eq!(result.placeholder_names.len(), 2);
        assert!(result.placeholder_names.contains("name"));
        assert!(result.placeholder_names.contains("type"));
    }

    #[test]
    fn test_parse_case_variations() {
        let content = "{{Name}} {{name}} {{NAME}} {{name_snake}} {{name-kebab}} {{nameCamel}}";
        let result = TemplateParser::parse(content).unwrap();
        assert_eq!(result.placeholder_names.len(), 1);
        assert!(result.placeholder_names.contains("name"));
    }

    #[test]
    fn test_parse_conditional() {
        let content = "{{#if condition}}content{{/if}}";
        let result = TemplateParser::parse(content).unwrap();
        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_parse_loop() {
        let content = "{{#each items}}{{name}}{{/each}}";
        let result = TemplateParser::parse(content).unwrap();
        assert!(result.placeholder_names.contains("name"));
    }

    #[test]
    fn test_extract_placeholders() {
        let content = "{{Name}} and {{description}}";
        let placeholders = TemplateParser::extract_placeholders(content).unwrap();
        assert_eq!(placeholders.len(), 2);
    }

    #[test]
    fn test_has_conditionals() {
        let content = "{{#if test}}yes{{/if}}";
        assert!(TemplateParser::has_conditionals(content).unwrap());
    }

    #[test]
    fn test_has_loops() {
        let content = "{{#each items}}{{name}}{{/each}}";
        assert!(TemplateParser::has_loops(content).unwrap());
    }

    #[test]
    fn test_has_includes() {
        let content = "{{> partial}}";
        assert!(TemplateParser::has_includes(content).unwrap());
    }

    #[test]
    fn test_unclosed_placeholder_error() {
        let content = "Hello {{name";
        let result = TemplateParser::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_conditional_error() {
        let content = "{{#if test}}content";
        let result = TemplateParser::parse(content);
        assert!(result.is_err());
    }
}
