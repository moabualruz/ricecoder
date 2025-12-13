//! Code editor widget with syntax highlighting
//!
//! This module provides a code display widget with syntax highlighting support for
//! multiple programming languages.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Syntax highlighting theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTheme {
    /// Dark theme
    Dark,
    /// Light theme
    Light,
    /// Monokai theme
    Monokai,
}

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// Rust
    Rust,
    /// Python
    Python,
    /// JavaScript/TypeScript
    JavaScript,
    /// TypeScript
    TypeScript,
    /// Go
    Go,
    /// Java
    Java,
    /// Kotlin
    Kotlin,
    /// C/C++
    C,
    /// Shell/Bash
    Shell,
    /// YAML
    Yaml,
    /// JSON
    Json,
    /// Markdown
    Markdown,
    /// SQL
    Sql,
    /// HTML
    Html,
    /// CSS
    Css,
    /// PHP
    Php,
    /// Ruby
    Ruby,
    /// Swift
    Swift,
    /// Scala
    Scala,
    /// C++
    Cpp,
    /// Plain text
    PlainText,
    /// Unknown
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" => Language::JavaScript,
            "ts" => Language::TypeScript,
            "go" => Language::Go,
            "java" => Language::Java,
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" => Language::C,
            "sh" | "bash" => Language::Shell,
            "yaml" | "yml" => Language::Yaml,
            "json" => Language::Json,
            "md" | "markdown" => Language::Markdown,
            "sql" => Language::Sql,
            "html" | "htm" => Language::Html,
            "css" => Language::Css,
            _ => Language::PlainText,
        }
    }

    /// Get the language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::C => "C/C++",
            Language::Shell => "Shell",
            Language::Yaml => "YAML",
            Language::Json => "JSON",
            Language::Markdown => "Markdown",
            Language::Sql => "SQL",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Php => "PHP",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Scala => "Scala",
            Language::Cpp => "C++",
            Language::Unknown => "Unknown",
            Language::PlainText => "Plain Text",
        }
    }
}

/// Code line with optional syntax highlighting
#[derive(Debug, Clone)]
pub struct CodeLine {
    /// Line number
    pub line_number: usize,
    /// Line content
    pub content: String,
    /// Language for syntax highlighting
    pub language: Language,
    /// Whether line is highlighted
    pub highlighted: bool,
}

impl CodeLine {
    /// Create a new code line
    pub fn new(line_number: usize, content: impl Into<String>, language: Language) -> Self {
        Self {
            line_number,
            content: content.into(),
            language,
            highlighted: false,
        }
    }

    /// Highlight this line
    pub fn highlight(&mut self) {
        self.highlighted = true;
    }

    /// Unhighlight this line
    pub fn unhighlight(&mut self) {
        self.highlighted = false;
    }

    /// Get the display text with line number
    pub fn display_text(&self) -> String {
        format!("{:4} | {}", self.line_number, self.content)
    }
}

/// Code editor widget for displaying code with syntax highlighting
pub struct CodeEditorWidget {
    /// Code lines for display
    lines: Vec<CodeLine>,
    /// Current language
    language: Language,
    /// Syntax theme
    theme: SyntaxTheme,
    /// Current scroll offset
    scroll_offset: usize,
    /// Selected line
    selected_line: Option<usize>,
    /// Title for the widget
    title: String,
    /// Whether to show line numbers
    show_line_numbers: bool,
    /// Whether to show borders
    show_borders: bool,
    /// Tab width (for indentation)
    tab_width: usize,
}

impl CodeEditorWidget {
    /// Create a new code editor widget
    pub fn new(language: Language) -> Self {
        Self {
            lines: Vec::new(),
            language,
            theme: SyntaxTheme::Dark,
            scroll_offset: 0,
            selected_line: None,
            title: "Code".to_string(),
            show_line_numbers: true,
            show_borders: true,
            tab_width: 4,
        }
    }

    /// Set the code content
    pub fn set_code(&mut self, code: &str) {
        self.lines.clear();
        for (idx, line) in code.lines().enumerate() {
            self.lines.push(CodeLine::new(idx + 1, line, self.language));
        }
    }

    /// Add a line of code
    pub fn add_line(&mut self, content: impl Into<String>) {
        let line_number = self.lines.len() + 1;
        self.lines.push(CodeLine::new(line_number, content, self.language));
    }

    /// Clear all code
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0;
        self.selected_line = None;
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get a specific line
    pub fn get_line(&self, index: usize) -> Option<&CodeLine> {
        self.lines.get(index)
    }

    /// Get a mutable line
    pub fn get_line_mut(&mut self, index: usize) -> Option<&mut CodeLine> {
        self.lines.get_mut(index)
    }

    /// Highlight a line
    pub fn highlight_line(&mut self, index: usize) {
        if let Some(line) = self.lines.get_mut(index) {
            line.highlight();
        }
    }

    /// Unhighlight a line
    pub fn unhighlight_line(&mut self, index: usize) {
        if let Some(line) = self.lines.get_mut(index) {
            line.unhighlight();
        }
    }

    /// Clear all highlights
    pub fn clear_highlights(&mut self) {
        for line in &mut self.lines {
            line.unhighlight();
        }
    }

    /// Select a line
    pub fn select_line(&mut self, index: usize) {
        if index < self.lines.len() {
            self.selected_line = Some(index);
        }
    }

    /// Deselect the current line
    pub fn deselect_line(&mut self) {
        self.selected_line = None;
    }

    /// Get the selected line index
    pub fn selected_line(&self) -> Option<usize> {
        self.selected_line
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self, visible_height: usize) {
        let max_scroll = self.lines.len().saturating_sub(visible_height);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self, visible_height: usize) {
        let max_scroll = self.lines.len().saturating_sub(visible_height);
        self.scroll_offset = max_scroll;
    }

    /// Get visible lines
    pub fn visible_lines(&self, height: usize) -> Vec<&CodeLine> {
        self.lines
            .iter()
            .skip(self.scroll_offset)
            .take(height)
            .collect()
    }

    /// Set the language
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        for line in &mut self.lines {
            line.language = language;
        }
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: SyntaxTheme) {
        self.theme = theme;
    }

    /// Set whether to show line numbers
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Set whether to show borders
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Set the tab width
    pub fn set_tab_width(&mut self, width: usize) {
        self.tab_width = width;
    }

    /// Get the current language
    pub fn language(&self) -> Language {
        self.language
    }

    /// Get the current theme
    pub fn theme(&self) -> SyntaxTheme {
        self.theme
    }

    /// Get the scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Check if line numbers are shown
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Check if borders are shown
    pub fn show_borders(&self) -> bool {
        self.show_borders
    }

    /// Get the tab width
    pub fn tab_width(&self) -> usize {
        self.tab_width
    }

    /// Get the scroll percentage
    pub fn scroll_percentage(&self, visible_height: usize) -> u8 {
        if self.lines.is_empty() {
            return 100;
        }

        let max_scroll = self.lines.len().saturating_sub(visible_height);
        if max_scroll == 0 {
            return 100;
        }

        ((self.scroll_offset as f32 / max_scroll as f32) * 100.0) as u8
    }

    /// Check if at the top
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Check if at the bottom
    pub fn is_at_bottom(&self, visible_height: usize) -> bool {
        let max_scroll = self.lines.len().saturating_sub(visible_height);
        self.scroll_offset >= max_scroll
    }

    /// Get all lines
    pub fn lines(&self) -> &[CodeLine] {
        &self.lines
    }
}

impl Default for CodeEditorWidget {
    fn default() -> Self {
        Self::new(Language::PlainText)
    }
}

impl Widget for &CodeEditorWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::default().title(self.title.as_str());

        if self.show_borders {
            block = block.borders(Borders::ALL);
        }

        let inner_area = block.inner(area);
        block.render(area, buf);

        // For now, render as simple text. In a full implementation, this would
        // include syntax highlighting using syntect or similar
        let visible_lines = self.visible_lines(inner_area.height as usize);

        for (i, line) in visible_lines.iter().enumerate() {
            if i >= inner_area.height as usize {
                break;
            }

            let mut style = Style::default();
            if Some(line.line_number - 1) == self.selected_line {
                style = style.bg(Color::Blue).fg(Color::White);
            } else if line.highlighted {
                style = style.bg(Color::Yellow).fg(Color::Black);
            }

            let content = if self.show_line_numbers {
                format!("{:4} | {}", line.line_number, line.content)
            } else {
                line.content.clone()
            };

            let span = Span::styled(content, style);
            let line_widget = Line::from(span);
            let paragraph = Paragraph::new(line_widget).style(style);

            let line_area = Rect {
                x: inner_area.x,
                y: inner_area.y + i as u16,
                width: inner_area.width,
                height: 1,
            };

            paragraph.render(line_area, buf);
        }
    }
}


