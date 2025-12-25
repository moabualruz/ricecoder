//! Simple code editor widget for the TUI
//!
//! This provides basic code editing functionality without tree-sitter dependencies.
//! For advanced syntax highlighting and language features, use the LSP integration.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_textarea::{TextArea, Input, Key};

/// Simple code editor widget
pub struct CodeEditor<'a> {
    textarea: TextArea<'a>,
    language: Option<String>,
    filename: Option<String>,
}

impl<'a> CodeEditor<'a> {
    /// Create a new code editor
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Code Editor"));

        Self {
            textarea,
            language: None,
            filename: None,
        }
    }

    /// Set the language for syntax highlighting hint
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the filename for display
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Get the current content as lines
    pub fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    /// Set the content from lines
    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.textarea = TextArea::new(lines);
        self.update_block();
    }

    /// Handle input
    pub fn input(&mut self, input: impl Into<Input>) -> bool {
        self.textarea.input(input)
    }

    /// Get the current cursor position
    pub fn cursor(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.textarea.set_cursor_row(row);
        self.textarea.set_cursor_col(col);
    }

    /// Insert text at cursor
    pub fn insert(&mut self, text: &str) {
        // Simple insertion - in a real implementation, this would handle syntax
        let (row, col) = self.cursor();
        if let Some(line) = self.textarea.lines().get_mut(row) {
            line.insert_str(col, text);
            self.textarea.set_cursor_col(col + text.len());
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        let (row, col) = self.cursor();
        if col > 0 {
            if let Some(line) = self.textarea.lines().get_mut(row) {
                line.remove(col - 1);
                self.textarea.set_cursor_col(col - 1);
            }
        } else if row > 0 {
            // Join with previous line
            let current_line = self.textarea.lines()[row].clone();
            let prev_line_len = self.textarea.lines()[row - 1].len();

            // Remove current line
            self.textarea.lines_mut().remove(row);

            // Append to previous line
            if let Some(prev_line) = self.textarea.lines_mut().get_mut(row - 1) {
                prev_line.push_str(&current_line);
            }

            self.textarea.set_cursor_row(row - 1);
            self.textarea.set_cursor_col(prev_line_len);
        }
    }

    /// Get the widget for rendering
    pub fn widget(&'a self) -> impl Widget + 'a {
        self.textarea.widget()
    }

    fn update_block(&mut self) {
        let mut block = Block::default().borders(Borders::ALL);

        let title = match (&self.filename, &self.language) {
            (Some(filename), Some(language)) => format!("Code Editor - {} ({})", filename, language),
            (Some(filename), None) => format!("Code Editor - {}", filename),
            (None, Some(language)) => format!("Code Editor ({})", language),
            (None, None) => "Code Editor".to_string(),
        };

        block = block.title(title);
        self.textarea.set_block(block);
    }
}

impl<'a> Default for CodeEditor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

