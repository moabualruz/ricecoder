//! Text input wrapper for prompt
//!
//! Wraps tui-textarea with keybinding support and cursor management.
//!
//! # DDD Layer: Infrastructure
//! Provides the text input component for the prompt widget.

use tui_textarea::{TextArea, CursorMove, Input, Key};
use ratatui::style::{Color, Style};
use std::collections::HashMap;

/// Text input actions matching OpenCode's TEXTAREA_ACTIONS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    Submit,
    Newline,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
    LineHome,
    LineEnd,
    SelectLineHome,
    SelectLineEnd,
    VisualLineHome,
    VisualLineEnd,
    SelectVisualLineHome,
    SelectVisualLineEnd,
    BufferHome,
    BufferEnd,
    SelectBufferHome,
    SelectBufferEnd,
    DeleteLine,
    DeleteToLineEnd,
    DeleteToLineStart,
    Backspace,
    Delete,
    Undo,
    Redo,
    WordForward,
    WordBackward,
    SelectWordForward,
    SelectWordBackward,
    DeleteWordForward,
    DeleteWordBackward,
}

impl InputAction {
    /// Get all available actions
    pub fn all() -> &'static [InputAction] {
        use InputAction::*;
        &[
            Submit, Newline, MoveLeft, MoveRight, MoveUp, MoveDown,
            SelectLeft, SelectRight, SelectUp, SelectDown,
            LineHome, LineEnd, SelectLineHome, SelectLineEnd,
            VisualLineHome, VisualLineEnd, SelectVisualLineHome, SelectVisualLineEnd,
            BufferHome, BufferEnd, SelectBufferHome, SelectBufferEnd,
            DeleteLine, DeleteToLineEnd, DeleteToLineStart,
            Backspace, Delete, Undo, Redo,
            WordForward, WordBackward, SelectWordForward, SelectWordBackward,
            DeleteWordForward, DeleteWordBackward,
        ]
    }
    
    /// Convert to config key name
    pub fn config_key(&self) -> &'static str {
        match self {
            Self::Submit => "input_submit",
            Self::Newline => "input_newline",
            Self::MoveLeft => "input_move_left",
            Self::MoveRight => "input_move_right",
            Self::MoveUp => "input_move_up",
            Self::MoveDown => "input_move_down",
            Self::SelectLeft => "input_select_left",
            Self::SelectRight => "input_select_right",
            Self::SelectUp => "input_select_up",
            Self::SelectDown => "input_select_down",
            Self::LineHome => "input_line_home",
            Self::LineEnd => "input_line_end",
            Self::SelectLineHome => "input_select_line_home",
            Self::SelectLineEnd => "input_select_line_end",
            Self::VisualLineHome => "input_visual_line_home",
            Self::VisualLineEnd => "input_visual_line_end",
            Self::SelectVisualLineHome => "input_select_visual_line_home",
            Self::SelectVisualLineEnd => "input_select_visual_line_end",
            Self::BufferHome => "input_buffer_home",
            Self::BufferEnd => "input_buffer_end",
            Self::SelectBufferHome => "input_select_buffer_home",
            Self::SelectBufferEnd => "input_select_buffer_end",
            Self::DeleteLine => "input_delete_line",
            Self::DeleteToLineEnd => "input_delete_to_line_end",
            Self::DeleteToLineStart => "input_delete_to_line_start",
            Self::Backspace => "input_backspace",
            Self::Delete => "input_delete",
            Self::Undo => "input_undo",
            Self::Redo => "input_redo",
            Self::WordForward => "input_word_forward",
            Self::WordBackward => "input_word_backward",
            Self::SelectWordForward => "input_select_word_forward",
            Self::SelectWordBackward => "input_select_word_backward",
            Self::DeleteWordForward => "input_delete_word_forward",
            Self::DeleteWordBackward => "input_delete_word_backward",
        }
    }
}

/// Cursor position in the input
#[derive(Debug, Clone, Copy, Default)]
pub struct CursorPosition {
    /// Row (line number, 0-indexed)
    pub row: usize,
    /// Column (character offset, 0-indexed)
    pub col: usize,
    /// Visual row (for wrapped lines)
    pub visual_row: usize,
    /// Byte offset from start
    pub offset: usize,
}

/// Prompt text input with tui-textarea
pub struct PromptInput<'a> {
    /// The underlying textarea
    textarea: TextArea<'a>,
    /// Cursor color
    cursor_color: Color,
    /// Whether the input is focused
    focused: bool,
    /// Maximum height (in lines)
    max_height: u16,
    /// Minimum height
    min_height: u16,
}

impl<'a> Default for PromptInput<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PromptInput<'a> {
    /// Create a new prompt input
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        
        Self {
            textarea,
            cursor_color: Color::White,
            focused: true,
            max_height: 6,
            min_height: 1,
        }
    }
    
    /// Set placeholder text
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.textarea.set_placeholder_text(placeholder);
    }
    
    /// Set cursor color
    pub fn set_cursor_color(&mut self, color: Color) {
        self.cursor_color = color;
        self.textarea.set_cursor_style(Style::default().fg(color));
    }
    
    /// Set max height
    pub fn set_max_height(&mut self, height: u16) {
        self.max_height = height;
    }
    
    /// Get the plain text content
    pub fn plain_text(&self) -> String {
        self.textarea.lines().join("\n")
    }
    
    /// Set text content
    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        let lines: Vec<String> = text.lines().map(String::from).collect();
        self.textarea = TextArea::new(lines);
    }
    
    /// Clear the input
    pub fn clear(&mut self) {
        self.textarea = TextArea::default();
    }
    
    /// Insert text at cursor
    pub fn insert_text(&mut self, text: &str) {
        for c in text.chars() {
            self.textarea.insert_char(c);
        }
    }
    
    /// Get cursor position
    pub fn cursor(&self) -> CursorPosition {
        let (row, col) = self.textarea.cursor();
        CursorPosition {
            row,
            col,
            visual_row: row, // tui-textarea doesn't expose visual row
            offset: self.calculate_offset(row, col),
        }
    }
    
    /// Calculate byte offset from row/col
    fn calculate_offset(&self, row: usize, col: usize) -> usize {
        let mut offset = 0;
        for (i, line) in self.textarea.lines().iter().enumerate() {
            if i < row {
                offset += line.len() + 1; // +1 for newline
            } else {
                offset += col.min(line.len());
                break;
            }
        }
        offset
    }
    
    /// Set cursor offset
    pub fn set_cursor_offset(&mut self, offset: usize) {
        let text = self.plain_text();
        let mut current = 0;
        for (row, line) in text.lines().enumerate() {
            let line_len = line.len() + 1;
            if current + line_len > offset {
                let col = offset - current;
                self.textarea.move_cursor(CursorMove::Jump(row as u16, col as u16));
                return;
            }
            current += line_len;
        }
        // Move to end if offset exceeds content
        self.goto_buffer_end();
    }
    
    /// Move cursor to buffer start
    pub fn goto_buffer_start(&mut self) {
        self.textarea.move_cursor(CursorMove::Top);
        self.textarea.move_cursor(CursorMove::Head);
    }
    
    /// Move cursor to buffer end
    pub fn goto_buffer_end(&mut self) {
        self.textarea.move_cursor(CursorMove::Bottom);
        self.textarea.move_cursor(CursorMove::End);
    }
    
    /// Get current height in lines
    pub fn height(&self) -> u16 {
        let lines = self.textarea.lines().len() as u16;
        lines.clamp(self.min_height, self.max_height)
    }
    
    /// Focus the input
    pub fn focus(&mut self) {
        self.focused = true;
    }
    
    /// Blur the input
    pub fn blur(&mut self) {
        self.focused = false;
    }
    
    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }
    
    /// Handle input event
    pub fn handle_input(&mut self, input: Input) -> bool {
        self.textarea.input(input)
    }
    
    /// Get the textarea widget
    pub fn widget(&'a self) -> &TextArea<'a> {
        &self.textarea
    }
    
    /// Get mutable textarea
    pub fn widget_mut(&mut self) -> &mut TextArea<'a> {
        &mut self.textarea
    }
    
    /// Execute an input action
    pub fn execute_action(&mut self, action: InputAction) -> bool {
        match action {
            InputAction::Submit => false, // Handled externally
            InputAction::Newline => {
                self.textarea.insert_newline();
                true
            }
            InputAction::MoveLeft => {
                self.textarea.move_cursor(CursorMove::Back);
                true
            }
            InputAction::MoveRight => {
                self.textarea.move_cursor(CursorMove::Forward);
                true
            }
            InputAction::MoveUp => {
                self.textarea.move_cursor(CursorMove::Up);
                true
            }
            InputAction::MoveDown => {
                self.textarea.move_cursor(CursorMove::Down);
                true
            }
            InputAction::LineHome => {
                self.textarea.move_cursor(CursorMove::Head);
                true
            }
            InputAction::LineEnd => {
                self.textarea.move_cursor(CursorMove::End);
                true
            }
            InputAction::BufferHome => {
                self.goto_buffer_start();
                true
            }
            InputAction::BufferEnd => {
                self.goto_buffer_end();
                true
            }
            InputAction::Backspace => {
                self.textarea.delete_char();
                true
            }
            InputAction::Delete => {
                self.textarea.delete_next_char();
                true
            }
            InputAction::DeleteLine => {
                self.textarea.move_cursor(CursorMove::Head);
                self.textarea.delete_line_by_end();
                true
            }
            InputAction::DeleteToLineEnd => {
                self.textarea.delete_line_by_end();
                true
            }
            InputAction::DeleteToLineStart => {
                self.textarea.delete_line_by_head();
                true
            }
            InputAction::Undo => {
                self.textarea.undo();
                true
            }
            InputAction::Redo => {
                self.textarea.redo();
                true
            }
            InputAction::WordForward => {
                self.textarea.move_cursor(CursorMove::WordForward);
                true
            }
            InputAction::WordBackward => {
                self.textarea.move_cursor(CursorMove::WordBack);
                true
            }
            InputAction::DeleteWordForward => {
                self.textarea.delete_next_word();
                true
            }
            InputAction::DeleteWordBackward => {
                self.textarea.delete_word();
                true
            }
            // Selection actions - tui-textarea doesn't support selection directly
            // These would need custom implementation
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_action_all() {
        let all = InputAction::all();
        assert!(all.len() >= 30);
    }
    
    #[test]
    fn test_prompt_input_new() {
        let input = PromptInput::new();
        assert!(input.plain_text().is_empty());
        assert!(input.is_focused());
    }
    
    #[test]
    fn test_set_text() {
        let mut input = PromptInput::new();
        input.set_text("hello\nworld");
        assert_eq!(input.plain_text(), "hello\nworld");
    }
    
    #[test]
    fn test_insert_text() {
        let mut input = PromptInput::new();
        input.insert_text("hello");
        assert_eq!(input.plain_text(), "hello");
    }
    
    #[test]
    fn test_clear() {
        let mut input = PromptInput::new();
        input.set_text("hello");
        input.clear();
        assert!(input.plain_text().is_empty());
    }
    
    #[test]
    fn test_height() {
        let mut input = PromptInput::new();
        assert_eq!(input.height(), 1);
        
        input.set_text("line1\nline2\nline3");
        assert_eq!(input.height(), 3);
    }
    
    #[test]
    fn test_focus_blur() {
        let mut input = PromptInput::new();
        assert!(input.is_focused());
        
        input.blur();
        assert!(!input.is_focused());
        
        input.focus();
        assert!(input.is_focused());
    }
}
