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

/// Selection state for text selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Anchor position (where selection started)
    pub anchor: (usize, usize),
    /// Active position (current cursor position)
    pub active: (usize, usize),
}

impl Selection {
    /// Create a new selection
    pub fn new(anchor: (usize, usize), active: (usize, usize)) -> Self {
        Self { anchor, active }
    }

    /// Get the range of the selection (normalized to start, end)
    pub fn range(&self) -> ((usize, usize), (usize, usize)) {
        if self.anchor <= self.active {
            (self.anchor, self.active)
        } else {
            (self.active, self.anchor)
        }
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }
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
    /// Current selection (if any)
    selection: Option<Selection>,
    /// History draft (saved before history navigation)
    history_draft: Option<String>,
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
            selection: None,
            history_draft: None,
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

    // ========================================================================
    // Selection API (GAP-32-004)
    // ========================================================================

    /// Get current selection
    pub fn selection(&self) -> Option<Selection> {
        self.selection
    }

    /// Set selection
    pub fn set_selection(&mut self, selection: Option<Selection>) {
        self.selection = selection;
    }

    /// Start selection from current cursor position
    pub fn start_selection(&mut self) {
        let pos = self.cursor_position();
        self.selection = Some(Selection::new(pos, pos));
    }

    /// Update selection to current cursor position
    pub fn update_selection(&mut self) {
        let pos = self.cursor_position();
        if let Some(sel) = &mut self.selection {
            sel.active = pos;
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.selection.and_then(|sel| {
            if sel.is_empty() {
                return None;
            }
            let ((start_row, start_col), (end_row, end_col)) = sel.range();
            let lines = self.textarea.lines();
            
            if start_row >= lines.len() {
                return None;
            }

            if start_row == end_row {
                // Single line selection
                lines.get(start_row).and_then(|line| {
                    let chars: Vec<char> = line.chars().collect();
                    if start_col < chars.len() && end_col <= chars.len() {
                        Some(chars[start_col..end_col].iter().collect())
                    } else {
                        None
                    }
                })
            } else {
                // Multi-line selection
                let mut result = String::new();
                for (i, line) in lines.iter().enumerate().skip(start_row).take(end_row - start_row + 1) {
                    if i == start_row {
                        let chars: Vec<char> = line.chars().collect();
                        if start_col < chars.len() {
                            result.push_str(&chars[start_col..].iter().collect::<String>());
                        }
                    } else if i == end_row {
                        let chars: Vec<char> = line.chars().collect();
                        if end_col <= chars.len() {
                            result.push_str(&chars[..end_col].iter().collect::<String>());
                        }
                    } else {
                        result.push_str(line);
                    }
                    if i < end_row {
                        result.push('\n');
                    }
                }
                Some(result)
            }
        })
    }

    /// Delete selected text
    pub fn delete_selection(&mut self) -> bool {
        if let Some(text) = self.selected_text() {
            // Note: This is a simplified implementation
            // A full implementation would use ratatui-textarea's selection API
            self.clear_selection();
            true
        } else {
            false
        }
    }

    // ========================================================================
    // Clipboard Integration (GAP-32-005)
    // ========================================================================

    /// Copy selected text to clipboard
    pub fn copy_selection(&self) -> Result<(), crate::clipboard::ClipboardError> {
        if let Some(text) = self.selected_text() {
            crate::clipboard::ClipboardManager::copy_text_static(&text)
        } else {
            Err(crate::clipboard::ClipboardError::CopyError("No selection".to_string()))
        }
    }

    /// Cut selected text to clipboard
    pub fn cut_selection(&mut self) -> Result<(), crate::clipboard::ClipboardError> {
        if let Some(text) = self.selected_text() {
            let result = crate::clipboard::ClipboardManager::copy_text_static(&text);
            if result.is_ok() {
                self.delete_selection();
            }
            result
        } else {
            Err(crate::clipboard::ClipboardError::CopyError("No selection".to_string()))
        }
    }

    /// Paste text from clipboard at cursor position
    pub fn paste_from_clipboard(&mut self) -> Result<(), crate::clipboard::ClipboardError> {
        let text = crate::clipboard::ClipboardManager::read_text()?;
        
        // Delete selection if exists
        if self.selection.is_some() {
            self.delete_selection();
        }

        // Insert text at cursor position
        for ch in text.chars() {
            if ch == '\n' {
                self.textarea.insert_newline();
            } else {
                self.textarea.insert_char(ch);
            }
        }

        Ok(())
    }

    // ========================================================================
    // History Draft Preservation (GAP-32-007)
    // ========================================================================

    /// Save current text as draft before history navigation
    pub fn save_draft(&mut self) {
        let text = self.text();
        if !text.is_empty() {
            self.history_draft = Some(text);
        }
    }

    /// Restore draft text
    pub fn restore_draft(&mut self) -> bool {
        if let Some(draft) = self.history_draft.take() {
            self.set_text(draft);
            true
        } else {
            false
        }
    }

    /// Get draft text without consuming it
    pub fn peek_draft(&self) -> Option<&str> {
        self.history_draft.as_deref()
    }

    /// Clear draft
    pub fn clear_draft(&mut self) {
        self.history_draft = None;
    }

    /// Check if draft exists
    pub fn has_draft(&self) -> bool {
        self.history_draft.is_some()
    }
}

impl Default for TextAreaWidget {
    fn default() -> Self {
        Self::new(false, 10)
    }
}
