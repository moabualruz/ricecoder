//! Text area widget for message input
//!
//! This module provides a text input widget with vim-like keybindings.
//! It supports multi-line input, vim mode, and integration with the TUI event loop.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;

/// Vim mode states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Insert,
    Normal,
    Visual,
    VisualLine,
}

/// Text selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    pub start: usize,
    pub end: usize,
}

/// Undo/redo entry
#[derive(Debug, Clone)]
struct UndoEntry {
    text: String,
    cursor: (usize, usize),
    selection: Option<TextSelection>,
}

/// Text area widget for message input
pub struct TextAreaWidget {
    /// The input text
    text: String,
    /// Whether vim mode is enabled
    vim_mode: bool,
    /// Current vim mode (if vim_mode is enabled)
    vim_state: VimMode,
    /// Maximum height for the textarea
    max_height: u16,
    /// Cursor position (line, column)
    cursor: (usize, usize),
    /// Text selection (start, end byte positions)
    selection: Option<TextSelection>,
    /// Undo history
    undo_stack: VecDeque<UndoEntry>,
    /// Redo history
    redo_stack: VecDeque<UndoEntry>,
    /// Maximum undo history size
    max_undo_history: usize,
}

impl TextAreaWidget {
    /// Create a new text area widget
    pub fn new(vim_mode: bool, max_height: u16) -> Self {
        Self {
            text: String::new(),
            vim_mode,
            vim_state: if vim_mode { VimMode::Normal } else { VimMode::Insert },
            max_height,
            cursor: (0, 0),
            selection: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_history: 50,
        }
    }

    /// Get the current input text
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// Set the input text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.save_undo_state();
        self.text = text.into();
        self.cursor = (0, 0);
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = (0, 0);
    }

    /// Check if input is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get the current vim mode
    pub fn vim_mode(&self) -> VimMode {
        self.vim_state
    }

    /// Get the current text selection
    pub fn selection(&self) -> Option<TextSelection> {
        self.selection
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    /// Get cursor position as (line, column) in character positions
    pub fn cursor_position(&self) -> (usize, usize) {
        self.cursor
    }

    /// Set vim mode
    pub fn set_vim_mode(&mut self, mode: VimMode) {
        self.vim_state = mode;
    }

    /// Start text selection
    pub fn start_selection(&mut self) {
        if let Some(sel) = self.selection {
            self.selection = Some(TextSelection {
                start: sel.start,
                end: self.byte_position(),
            });
        } else {
            let pos = self.byte_position();
            self.selection = Some(TextSelection {
                start: pos,
                end: pos,
            });
        }
    }

    /// Clear text selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.selection.map(|sel| {
            let start = sel.start.min(sel.end);
            let end = sel.start.max(sel.end);
            self.text.chars().skip(start).take(end - start).collect()
        })
    }

    /// Cut selected text
    pub fn cut_selection(&mut self) -> Option<String> {
        if let Some(sel) = self.selection {
            self.save_undo_state();
            let start = sel.start.min(sel.end);
            let end = sel.start.max(sel.end);
            let selected = self.text.chars().skip(start).take(end - start).collect::<String>();
            self.text = self.text.chars().take(start).chain(self.text.chars().skip(end)).collect();
            self.cursor = self.byte_to_line_col(start);
            self.clear_selection();
            Some(selected)
        } else {
            None
        }
    }

    /// Copy selected text
    pub fn copy_selection(&self) -> Option<String> {
        self.selected_text()
    }

    /// Delete selected text
    pub fn delete_selection(&mut self) {
        if let Some(_) = self.selection {
            self.save_undo_state();
            let start = self.selection.unwrap().start.min(self.selection.unwrap().end);
            let end = self.selection.unwrap().start.max(self.selection.unwrap().end);
            self.text = self.text.chars().take(start).chain(self.text.chars().skip(end)).collect();
            self.cursor = self.byte_to_line_col(start);
            self.clear_selection();
        }
    }

    /// Undo last operation
    pub fn undo(&mut self) -> bool {
        if let Some(entry) = self.undo_stack.pop_back() {
            self.save_redo_state();
            self.text = entry.text;
            self.cursor = entry.cursor;
            self.selection = entry.selection;
            true
        } else {
            false
        }
    }

    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        if let Some(entry) = self.redo_stack.pop_back() {
            self.save_undo_state();
            self.text = entry.text;
            self.cursor = entry.cursor;
            self.selection = entry.selection;
            true
        } else {
            false
        }
    }

    /// Save current state for undo
    fn save_undo_state(&mut self) {
        let entry = UndoEntry {
            text: self.text.clone(),
            cursor: self.cursor,
            selection: self.selection,
        };
        self.undo_stack.push_back(entry);
        if self.undo_stack.len() > self.max_undo_history {
            self.undo_stack.pop_front();
        }
        // Clear redo stack on new operation
        self.redo_stack.clear();
    }

    /// Save current state for redo
    fn save_redo_state(&mut self) {
        let entry = UndoEntry {
            text: self.text.clone(),
            cursor: self.cursor,
            selection: self.selection,
        };
        self.redo_stack.push_back(entry);
    }

    /// Convert byte position to (line, column) in characters
    fn byte_to_line_col(&self, byte_pos: usize) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;
        let mut char_count = 0;

        for (i, c) in self.text.char_indices() {
            if i >= byte_pos {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            char_count += 1;
        }

        (line, char_count - self.line_start_char_pos(line))
    }

    /// Get byte position of cursor
    fn byte_position(&self) -> usize {
        let mut pos = 0;
        let mut current_line = 0;
        let mut current_col = 0;

        for (i, c) in self.text.char_indices() {
            if current_line == self.cursor.0 && current_col == self.cursor.1 {
                return i;
            }
            if c == '\n' {
                current_line += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
            pos = i;
        }

        // If cursor is at end
        if current_line == self.cursor.0 {
            self.text.len()
        } else {
            pos + 1
        }
    }

    /// Get character position at start of line
    fn line_start_char_pos(&self, line: usize) -> usize {
        let mut current_line = 0;
        let mut pos = 0;

        for (i, c) in self.text.char_indices() {
            if current_line == line {
                return pos;
            }
            if c == '\n' {
                current_line += 1;
                pos = i + 1;
            }
        }

        pos
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.text.lines().count().max(1)
    }

    /// Get the current cursor position
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Handle a key event
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        // Save state for undo before making changes
        let will_modify = matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete | KeyCode::Enter | KeyCode::Tab);
        if will_modify {
            self.save_undo_state();
        }
        // Handle vim mode first if enabled
        if self.vim_mode {
            return self.handle_vim_key_event(key);
        }

        // Handle standard editing keys
        match key.code {
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+Enter: new line
                    self.insert_char('\n');
                    self.cursor.0 += 1;
                    self.cursor.1 = 0;
                    return true;
                } else {
                    // Enter: submit (handled by caller)
                    return false;
                }
            }
            KeyCode::Tab => {
                self.insert_char('\t');
                return true;
            }
            KeyCode::Backspace => {
                self.delete_char_before_cursor();
                return true;
            }
            KeyCode::Delete => {
                self.delete_char_at_cursor();
                return true;
            }
            KeyCode::Up => {
                self.move_cursor_up();
                return true;
            }
            KeyCode::Down => {
                self.move_cursor_down();
                return true;
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.start_selection();
                }
                self.move_cursor_left();
                return true;
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.start_selection();
                }
                self.move_cursor_right();
                return true;
            }
            KeyCode::Home => {
                self.move_cursor_line_start();
                return true;
            }
            KeyCode::End => {
                self.move_cursor_line_end();
                return true;
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Handle Ctrl+key combinations
                    match c {
                        'a' => {
                            // Ctrl+A: select all
                            self.select_all();
                            return true;
                        }
                        'c' => {
                            // Ctrl+C: copy
                            // Copy is handled by caller
                            return false;
                        }
                        'x' => {
                            // Ctrl+X: cut
                            self.cut_selection();
                            return true;
                        }
                        'v' => {
                            // Ctrl+V: paste
                            // Paste is handled by caller
                            return false;
                        }
                        'z' => {
                            // Ctrl+Z: undo
                            self.undo();
                            return true;
                        }
                        'y' => {
                            // Ctrl+Y: redo
                            self.redo();
                            return true;
                        }
                        'u' => {
                            // Ctrl+U: clear line
                            self.clear_current_line();
                            return true;
                        }
                        'k' => {
                            // Ctrl+K: delete to end of line
                            self.delete_to_line_end();
                            return true;
                        }
                        _ => {}
                    }
                } else {
                    // Regular character input
                    self.insert_char(c);
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    /// Handle vim mode key events
    fn handle_vim_key_event(&mut self, key: KeyEvent) -> bool {
        match self.vim_state {
            VimMode::Insert => self.handle_vim_insert_mode(key),
            VimMode::Normal => self.handle_vim_normal_mode(key),
            VimMode::Visual => self.handle_vim_visual_mode(key),
            VimMode::VisualLine => self.handle_vim_visual_line_mode(key),
        }
    }

    /// Handle vim insert mode
    fn handle_vim_insert_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                // Escape to normal mode
                self.vim_state = VimMode::Normal;
                self.move_cursor_left(); // Move cursor back one position
                true
            }
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.insert_char('\n');
                    self.cursor.0 += 1;
                    self.cursor.1 = 0;
                    true
                } else {
                    false // Submit
                }
            }
            KeyCode::Tab => {
                self.insert_char('\t');
                true
            }
            KeyCode::Backspace => {
                self.delete_char_before_cursor();
                true
            }
            KeyCode::Delete => {
                self.delete_char_at_cursor();
                true
            }
            KeyCode::Char(c) => {
                self.insert_char(c);
                true
            }
            _ => false
        }
    }

    /// Handle vim normal mode
    fn handle_vim_normal_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                match c {
                    'i' => {
                        // Enter insert mode
                        self.vim_state = VimMode::Insert;
                        true
                    }
                    'a' => {
                        // Enter insert mode after cursor
                        self.move_cursor_right();
                        self.vim_state = VimMode::Insert;
                        true
                    }
                    'A' => {
                        // Enter insert mode at end of line
                        self.move_cursor_line_end();
                        self.vim_state = VimMode::Insert;
                        true
                    }
                    'o' => {
                        // Open new line below
                        self.move_cursor_line_end();
                        self.insert_char('\n');
                        self.cursor.0 += 1;
                        self.cursor.1 = 0;
                        self.vim_state = VimMode::Insert;
                        true
                    }
                    'O' => {
                        // Open new line above
                        self.move_cursor_line_start();
                        self.insert_char('\n');
                        self.cursor.1 = 0;
                        self.vim_state = VimMode::Insert;
                        true
                    }
                    'v' => {
                        // Enter visual mode
                        self.vim_state = VimMode::Visual;
                        self.start_selection();
                        true
                    }
                    'V' => {
                        // Enter visual line mode
                        self.vim_state = VimMode::VisualLine;
                        self.start_selection();
                        true
                    }
                    'h' => {
                        self.move_cursor_left();
                        true
                    }
                    'j' => {
                        self.move_cursor_down();
                        true
                    }
                    'k' => {
                        self.move_cursor_up();
                        true
                    }
                    'l' => {
                        self.move_cursor_right();
                        true
                    }
                    '0' => {
                        self.move_cursor_line_start();
                        true
                    }
                    '$' => {
                        self.move_cursor_line_end();
                        true
                    }
                    'x' => {
                        self.delete_char_at_cursor();
                        true
                    }
                    'd' => {
                        // Delete commands - wait for next character
                        // For now, just dd (delete line)
                        if let Some(next_key) = self.wait_for_next_key() {
                            if next_key.code == KeyCode::Char('d') {
                                self.delete_current_line();
                            }
                        }
                        true
                    }
                    'y' => {
                        // Yank commands
                        if let Some(next_key) = self.wait_for_next_key() {
                            if next_key.code == KeyCode::Char('y') {
                                // yy yank line - handled by caller
                                return false;
                            }
                        }
                        true
                    }
                    'p' => {
                        // Paste - handled by caller
                        false
                    }
                    '/' => {
                        // Search - handled by caller
                        false
                    }
                    'u' => {
                        self.undo();
                        true
                    }
                    'r' => {
                        self.redo();
                        true
                    }
                    _ => false
                }
            }
            _ => false
        }
    }

    /// Handle vim visual mode
    fn handle_vim_visual_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.vim_state = VimMode::Normal;
                self.clear_selection();
                true
            }
            KeyCode::Char(c) => {
                match c {
                    'd' => {
                        self.cut_selection();
                        self.vim_state = VimMode::Normal;
                        true
                    }
                    'y' => {
                        // Yank selection - handled by caller
                        self.vim_state = VimMode::Normal;
                        false
                    }
                    'h' => {
                        self.move_cursor_left();
                        self.start_selection();
                        true
                    }
                    'j' => {
                        self.move_cursor_down();
                        self.start_selection();
                        true
                    }
                    'k' => {
                        self.move_cursor_up();
                        self.start_selection();
                        true
                    }
                    'l' => {
                        self.move_cursor_right();
                        self.start_selection();
                        true
                    }
                    _ => false
                }
            }
            _ => false
        }
    }

    /// Handle vim visual line mode
    fn handle_vim_visual_line_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.vim_state = VimMode::Normal;
                self.clear_selection();
                true
            }
            KeyCode::Char(c) => {
                match c {
                    'd' => {
                        self.cut_selection();
                        self.vim_state = VimMode::Normal;
                        true
                    }
                    'y' => {
                        // Yank selection - handled by caller
                        self.vim_state = VimMode::Normal;
                        false
                    }
                    'j' => {
                        self.move_cursor_down();
                        self.start_selection();
                        true
                    }
                    'k' => {
                        self.move_cursor_up();
                        self.start_selection();
                        true
                    }
                    _ => false
                }
            }
            _ => false
        }
    }

    /// Wait for next key (simplified - in real implementation would need async)
    fn wait_for_next_key(&self) -> Option<KeyEvent> {
        None // Simplified for now
    }

    /// Insert character at cursor
    fn insert_char(&mut self, c: char) {
        let pos = self.byte_position();
        self.text.insert(pos, c);
        if c == '\n' {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        } else {
            self.cursor.1 += 1;
        }
    }

    /// Delete character before cursor
    fn delete_char_before_cursor(&mut self) {
        if self.cursor.1 > 0 {
            let pos = self.byte_position();
            self.text.remove(pos - 1);
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            // Delete newline
            let pos = self.byte_position();
            self.text.remove(pos - 1);
            self.cursor.0 -= 1;
            // Move to end of previous line
            let line_start = self.line_start_char_pos(self.cursor.0);
            self.cursor.1 = self.text.chars().skip(line_start).count();
        }
    }

    /// Delete character at cursor
    fn delete_char_at_cursor(&mut self) {
        let pos = self.byte_position();
        if pos < self.text.len() {
            self.text.remove(pos);
        }
    }

    /// Move cursor up
    fn move_cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            // Adjust column to not exceed line length
            let line_len = self.line_length(self.cursor.0);
            if self.cursor.1 > line_len {
                self.cursor.1 = line_len;
            }
        }
    }

    /// Move cursor down
    fn move_cursor_down(&mut self) {
        if self.cursor.0 < self.line_count() - 1 {
            self.cursor.0 += 1;
            // Adjust column to not exceed line length
            let line_len = self.line_length(self.cursor.0);
            if self.cursor.1 > line_len {
                self.cursor.1 = line_len;
            }
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.line_length(self.cursor.0);
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        let line_len = self.line_length(self.cursor.0);
        if self.cursor.1 < line_len {
            self.cursor.1 += 1;
        } else if self.cursor.0 < self.line_count() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    /// Move cursor to line start
    fn move_cursor_line_start(&mut self) {
        self.cursor.1 = 0;
    }

    /// Move cursor to line end
    fn move_cursor_line_end(&mut self) {
        self.cursor.1 = self.line_length(self.cursor.0);
    }

    /// Get length of line
    fn line_length(&self, line: usize) -> usize {
        self.text.lines().nth(line).map(|l| l.chars().count()).unwrap_or(0)
    }

    /// Clear current line
    fn clear_current_line(&mut self) {
        let line_start = self.line_start_char_pos(self.cursor.0);
        let line_end = if self.cursor.0 < self.line_count() - 1 {
            self.line_start_char_pos(self.cursor.0 + 1) - 1
        } else {
            self.text.len()
        };
        self.text.replace_range(line_start..line_end, "");
        self.cursor.1 = 0;
    }

    /// Delete to end of line
    fn delete_to_line_end(&mut self) {
        let pos = self.byte_position();
        let line_end = if self.cursor.0 < self.line_count() - 1 {
            self.line_start_char_pos(self.cursor.0 + 1) - 1
        } else {
            self.text.len()
        };
        self.text.replace_range(pos..line_end, "");
    }

    /// Delete current line
    fn delete_current_line(&mut self) {
        let line_start = self.line_start_char_pos(self.cursor.0);
        let line_end = if self.cursor.0 < self.line_count() - 1 {
            self.line_start_char_pos(self.cursor.0 + 1)
        } else {
            self.text.len()
        };
        self.text.replace_range(line_start..line_end, "");
        // Move cursor to start of line or previous line
        if self.cursor.0 >= self.line_count() && self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
        self.cursor.1 = 0;
    }

    /// Get the height needed to display all content
    pub fn required_height(&self) -> u16 {
        let line_count = self.line_count() as u16;
        std::cmp::min(line_count, self.max_height)
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection = Some(TextSelection {
            start: 0,
            end: self.text.chars().count(),
        });
    }

    /// Copy selected text (caller should handle clipboard)
    pub fn copy_text(&self) -> String {
        self.selected_text().unwrap_or_else(|| self.text.clone())
    }

    /// Paste text
    pub fn paste_text(&mut self, text: &str) {
        self.save_undo_state();
        let pos = self.byte_position();
        self.text.insert_str(pos, text);
        // Update cursor position - move cursor to end of pasted text
        let char_count = text.chars().count();
        let lines_added = text.chars().filter(|&c| c == '\n').count();

        if lines_added > 0 {
            // Multi-line paste
            let lines: Vec<&str> = text.lines().collect();
            if let Some(last_line) = lines.last() {
                self.cursor.0 += lines_added;
                self.cursor.1 = last_line.chars().count();
            }
        } else {
            self.cursor.1 += char_count;
        }
        self.clear_selection();
    }
}

impl Default for TextAreaWidget {
    fn default() -> Self {
        Self::new(false, 10)
    }
}
