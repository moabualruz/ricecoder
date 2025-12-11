//! Markdown rendering for the TUI

use lazy_static::lazy_static;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style as SyntectStyle};
use syntect::util::LinesWithEndings;
use ratatui::prelude::*;
use ratatui::style::Color;

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

/// Markdown element types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownElement {
    /// Plain text
    Text(String),
    /// Header (level, content)
    Header(u8, String),
    /// Bold text
    Bold(String),
    /// Italic text
    Italic(String),
    /// Code inline
    Code(String),
    /// Code block (language, content)
    CodeBlock(Option<String>, String),
    /// List item
    ListItem(String),
    /// Link (text, url)
    Link(String, String),
    /// Newline
    Newline,
}

impl MarkdownElement {
    /// Check if element is a block element
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            MarkdownElement::Header(_, _)
                | MarkdownElement::CodeBlock(_, _)
                | MarkdownElement::ListItem(_)
        )
    }
}

/// Markdown parser
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse markdown text into elements
    pub fn parse(text: &str) -> Vec<MarkdownElement> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check for code block
            if let Some(after_backticks) = line.strip_prefix("```") {
                let lang = after_backticks.trim().to_string();
                let lang = if lang.is_empty() { None } else { Some(lang) };
                let mut code = String::new();
                i += 1;

                while i < lines.len() && !lines[i].starts_with("```") {
                    if !code.is_empty() {
                        code.push('\n');
                    }
                    code.push_str(lines[i]);
                    i += 1;
                }

                elements.push(MarkdownElement::CodeBlock(lang, code));
                i += 1;
                continue;
            }

            // Check for headers
            if line.starts_with('#') {
                let level = line.chars().take_while(|c| *c == '#').count() as u8;
                let content = line[level as usize..].trim().to_string();
                elements.push(MarkdownElement::Header(level, content));
                i += 1;
                continue;
            }

            // Check for list items
            if line.starts_with("- ") || line.starts_with("* ") {
                let content = line[2..].trim().to_string();
                elements.push(MarkdownElement::ListItem(content));
                i += 1;
                continue;
            }

            // Parse inline elements
            let parsed = Self::parse_inline(line);
            elements.extend(parsed);
            elements.push(MarkdownElement::Newline);
            i += 1;
        }

        elements
    }

    /// Parse inline markdown elements
    #[allow(clippy::while_let_on_iterator)]
    fn parse_inline(text: &str) -> Vec<MarkdownElement> {
        let mut elements = Vec::new();
        let mut current = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '*' | '_' => {
                    if !current.is_empty() {
                        elements.push(MarkdownElement::Text(current.clone()));
                        current.clear();
                    }

                    // Check for bold or italic
                    if chars.peek() == Some(&ch) {
                        chars.next(); // consume second marker
                        let mut content = String::new();
                        let mut found = false;

                        while let Some(c) = chars.next() {
                            if c == ch && chars.peek() == Some(&ch) {
                                chars.next();
                                found = true;
                                break;
                            }
                            content.push(c);
                        }

                        if found {
                            elements.push(MarkdownElement::Bold(content));
                        }
                    } else {
                        // Italic
                        let mut content = String::new();
                        let mut found = false;

                        while let Some(c) = chars.next() {
                            if c == ch {
                                found = true;
                                break;
                            }
                            content.push(c);
                        }

                        if found {
                            elements.push(MarkdownElement::Italic(content));
                        }
                    }
                }
                '`' => {
                    if !current.is_empty() {
                        elements.push(MarkdownElement::Text(current.clone()));
                        current.clear();
                    }

                    let mut content = String::new();
                    while let Some(c) = chars.next() {
                        if c == '`' {
                            break;
                        }
                        content.push(c);
                    }

                    elements.push(MarkdownElement::Code(content));
                }
                '[' => {
                    if !current.is_empty() {
                        elements.push(MarkdownElement::Text(current.clone()));
                        current.clear();
                    }

                    let mut link_text = String::new();
                    while let Some(c) = chars.next() {
                        if c == ']' {
                            break;
                        }
                        link_text.push(c);
                    }

                    if chars.peek() == Some(&'(') {
                        chars.next();
                        let mut url = String::new();
                        while let Some(c) = chars.next() {
                            if c == ')' {
                                break;
                            }
                            url.push(c);
                        }

                        elements.push(MarkdownElement::Link(link_text, url));
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            elements.push(MarkdownElement::Text(current));
        }

        elements
    }

    /// Render markdown elements to plain text
    pub fn render_plain(elements: &[MarkdownElement]) -> String {
        let mut output = String::new();

        for element in elements {
            match element {
                MarkdownElement::Text(text) => output.push_str(text),
                MarkdownElement::Header(level, content) => {
                    output.push_str(&"#".repeat(*level as usize));
                    output.push(' ');
                    output.push_str(content);
                    output.push('\n');
                }
                MarkdownElement::Bold(text) => {
                    output.push_str("**");
                    output.push_str(text);
                    output.push_str("**");
                }
                MarkdownElement::Italic(text) => {
                    output.push('*');
                    output.push_str(text);
                    output.push('*');
                }
                MarkdownElement::Code(text) => {
                    output.push('`');
                    output.push_str(text);
                    output.push('`');
                }
                MarkdownElement::CodeBlock(lang, code) => {
                    output.push_str("```");
                    if let Some(l) = lang {
                        output.push_str(l);
                    }
                    output.push('\n');
                    output.push_str(code);
                    output.push_str("\n```\n");
                }
                MarkdownElement::ListItem(text) => {
                    output.push_str("- ");
                    output.push_str(text);
                    output.push('\n');
                }
                MarkdownElement::Link(text, url) => {
                    output.push('[');
                    output.push_str(text);
                    output.push_str("](");
                    output.push_str(url);
                    output.push(')');
                }
                MarkdownElement::Newline => output.push('\n'),
            }
        }

        output
    }

    /// Highlight code block using syntect
    pub fn highlight(code: &str, lang: Option<&str>) -> Vec<Line<'static>> {
        let ps = &SYNTAX_SET;
        let ts = &THEME_SET;
        let syntax = lang
            .and_then(|l| ps.find_syntax_by_token(l))
            .unwrap_or_else(|| ps.find_syntax_plain_text());
        
        // Use base16-ocean.dark as a safe default that usually exists
        // In a real app, map app theme to syntect theme
        let theme = &ts.themes["base16-ocean.dark"]; 
        
        let mut h = HighlightLines::new(syntax, theme);
        let mut lines = Vec::new();
        
        for line in LinesWithEndings::from(code) {
            let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line, ps).unwrap();
            let spans: Vec<Span> = ranges.into_iter().map(|(style, text)| {
                let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                Span::styled(text.to_string(), Style::default().fg(fg))
            }).collect();
            lines.push(Line::from(spans));
        }
        
        lines
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headers() {
        let text = "# Header 1\n## Header 2\n### Header 3";
        let elements = MarkdownParser::parse(text);

        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], MarkdownElement::Header(1, _)));
        assert!(matches!(elements[1], MarkdownElement::Header(2, _)));
        assert!(matches!(elements[2], MarkdownElement::Header(3, _)));
    }

    #[test]
    fn test_parse_code_block() {
        let text = "```rust\nfn main() {}\n```";
        let elements = MarkdownParser::parse(text);

        assert_eq!(elements.len(), 1);
        assert!(matches!(
            elements[0],
            MarkdownElement::CodeBlock(Some(_), _)
        ));
    }

    #[test]
    fn test_parse_list() {
        let text = "- Item 1\n- Item 2\n- Item 3";
        let elements = MarkdownParser::parse(text);

        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], MarkdownElement::ListItem(_)));
    }

    #[test]
    fn test_parse_inline_bold() {
        let text = "This is **bold** text";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Bold(_))));
    }

    #[test]
    fn test_parse_inline_italic() {
        let text = "This is *italic* text";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Italic(_))));
    }

    #[test]
    fn test_parse_inline_code() {
        let text = "Use `let x = 5;` for variables";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Code(_))));
    }

    #[test]
    fn test_parse_link() {
        let text = "Visit [example](https://example.com)";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Link(_, _))));
    }

    #[test]
    fn test_render_plain() {
        let elements = vec![
            MarkdownElement::Header(1, "Title".to_string()),
            MarkdownElement::Text("Some text".to_string()),
        ];

        let output = MarkdownParser::render_plain(&elements);
        assert!(output.contains("# Title"));
        assert!(output.contains("Some text"));
    }
}
