//! Text area widget for message input
//!
//! This module provides a text input widget using ratatui-textarea.
//! It supports multi-line input, vim mode, and integration with the TUI event loop.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::TextArea;

/// Vim mode states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Insert,
    Normal,
    Visual,
    VisualLine,
}

/// Text area widget for message input
pub struct TextAreaWidget {
    /// The underlying ratatui-textarea widget
    textarea: TextArea<'static>,
    /// Whether vim mode is enabled
    vim_mode: bool,
    /// Current vim mode (if vim_mode is enabled)
    vim_state: VimMode,
    /// Maximum height for the textarea
    max_height: u16,
}

impl TextAreaWidget {
    /// Create a new text area widget
    pub fn new(vim_mode: bool, max_height: u16) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_max_histories(50); // Enable undo/redo

        Self {
            textarea,
            vim_mode,
            vim_state: if vim_mode {
                VimMode::Normal
            } else {
                VimMode::Insert
            },
            max_height,
        }
    }

    /// Get the current input text
    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Set the input text
    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        self.textarea = TextArea::new(lines);
        self.textarea.set_max_histories(50);
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.textarea = TextArea::default();
        self.textarea.set_max_histories(50);
    }

    /// Check if input is empty
    pub fn is_empty(&self) -> bool {
        self.textarea.is_empty()
    }

    /// Get the current vim mode
    pub fn vim_mode(&self) -> VimMode {
        self.vim_state
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.textarea
            .lines()
            .iter()
            .map(|line| line.chars().count())
            .sum::<usize>()
            + self.textarea.lines().len().saturating_sub(1) // Add newlines between lines
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.textarea
            .lines()
            .iter()
            .flat_map(|line| line.split_whitespace())
            .filter(|word| !word.is_empty())
            .count()
    }

    /// Get cursor position as (line, column) in character positions
    pub fn cursor_position(&self) -> (usize, usize) {
        let (row, col) = self.textarea.cursor();
        (row, col)
    }

    /// Set vim mode
    pub fn set_vim_mode(&mut self, mode: VimMode) {
        self.vim_state = mode;
    }

    /// Render the textarea as a ratatui widget
    pub fn widget(&self) -> &TextArea<'static> {
        &self.textarea
    }

    /// Render the textarea as a mutable ratatui widget
    pub fn widget_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.textarea
    }

    /// Undo last operation
    pub fn undo(&mut self) -> bool {
        self.textarea.undo()
    }

    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        self.textarea.redo()
    }
}

impl Default for TextAreaWidget {
    fn default() -> Self {
        Self::new(false, 10)
    }
}
