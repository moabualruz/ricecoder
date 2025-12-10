//! Text area widget for message input
//!
//! This module provides a text input widget with vim-like keybindings.
//! It supports multi-line input, vim mode, and integration with the TUI event loop.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Text area widget for message input
pub struct TextAreaWidget {
    /// The input text
    text: String,
    /// Whether vim mode is enabled
    vim_mode: bool,
    /// Maximum height for the textarea
    max_height: u16,
    /// Cursor position (line, column)
    cursor: (usize, usize),
}

impl TextAreaWidget {
    /// Create a new text area widget
    pub fn new(vim_mode: bool, max_height: u16) -> Self {
        Self {
            text: String::new(),
            vim_mode,
            max_height,
            cursor: (0, 0),
        }
    }

    /// Get the current input text
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// Set the input text
    pub fn set_text(&mut self, text: impl Into<String>) {
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
        // Handle special keys
        match key.code {
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+Enter: new line
                    self.text.push('\n');
                    self.cursor.0 += 1;
                    self.cursor.1 = 0;
                    return true;
                } else {
                    // Enter: submit (handled by caller)
                    return false;
                }
            }
            KeyCode::Tab => {
                self.text.push('\t');
                self.cursor.1 += 1;
                return true;
            }
            KeyCode::Backspace => {
                if self.cursor.1 > 0 {
                    self.text.pop();
                    self.cursor.1 -= 1;
                }
                return true;
            }
            KeyCode::Delete => {
                // Delete next character
                if self.cursor.1 < self.text.len() {
                    self.text.remove(self.cursor.1);
                }
                return true;
            }
            KeyCode::Up => {
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                }
                return true;
            }
            KeyCode::Down => {
                if self.cursor.0 < self.line_count() - 1 {
                    self.cursor.0 += 1;
                }
                return true;
            }
            KeyCode::Left => {
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
                return true;
            }
            KeyCode::Right => {
                if self.cursor.1 < self.text.len() {
                    self.cursor.1 += 1;
                }
                return true;
            }
            KeyCode::Home => {
                self.cursor.1 = 0;
                return true;
            }
            KeyCode::End => {
                self.cursor.1 = self.text.len();
                return true;
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Handle Ctrl+key combinations
                    match c {
                        'a' => {
                            // Ctrl+A: move to start
                            self.cursor = (0, 0);
                            return true;
                        }
                        'u' => {
                            // Ctrl+U: clear line
                            self.text.clear();
                            self.cursor = (0, 0);
                            return true;
                        }
                        'k' => {
                            // Ctrl+K: delete to end of line
                            self.text.truncate(self.cursor.1);
                            return true;
                        }
                        _ => {}
                    }
                } else if self.vim_mode && key.modifiers.is_empty() {
                    // Handle vim-like keybindings
                    match c {
                        'h' => {
                            if self.cursor.1 > 0 {
                                self.cursor.1 -= 1;
                            }
                            return true;
                        }
                        'j' => {
                            if self.cursor.0 < self.line_count() - 1 {
                                self.cursor.0 += 1;
                            }
                            return true;
                        }
                        'k' => {
                            if self.cursor.0 > 0 {
                                self.cursor.0 -= 1;
                            }
                            return true;
                        }
                        'l' => {
                            if self.cursor.1 < self.text.len() {
                                self.cursor.1 += 1;
                            }
                            return true;
                        }
                        '0' => {
                            self.cursor.1 = 0;
                            return true;
                        }
                        '$' => {
                            self.cursor.1 = self.text.len();
                            return true;
                        }
                        _ => {
                            // Regular character
                            self.text.push(c);
                            self.cursor.1 += 1;
                            return true;
                        }
                    }
                } else {
                    // Regular character input
                    self.text.push(c);
                    self.cursor.1 += 1;
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    /// Get the height needed to display all content
    pub fn required_height(&self) -> u16 {
        let line_count = self.line_count() as u16;
        std::cmp::min(line_count, self.max_height)
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.cursor = (0, 0);
    }

    /// Copy selected text (caller should handle clipboard)
    pub fn copy_text(&self) -> String {
        self.text.clone()
    }

    /// Paste text
    pub fn paste_text(&mut self, text: &str) {
        self.text.push_str(text);
        self.cursor.1 += text.len();
    }
}

impl Default for TextAreaWidget {
    fn default() -> Self {
        Self::new(false, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textarea_creation() {
        let textarea = TextAreaWidget::new(false, 10);
        assert!(textarea.is_empty());
        assert_eq!(textarea.line_count(), 1);
    }

    #[test]
    fn test_textarea_text_operations() {
        let mut textarea = TextAreaWidget::new(false, 10);
        
        textarea.set_text("Hello, world!");
        assert_eq!(textarea.text(), "Hello, world!");
        assert!(!textarea.is_empty());

        textarea.clear();
        assert!(textarea.is_empty());
    }

    #[test]
    fn test_textarea_line_count() {
        let mut textarea = TextAreaWidget::new(false, 10);
        
        textarea.set_text("Line 1\nLine 2\nLine 3");
        assert_eq!(textarea.line_count(), 3);
    }

    #[test]
    fn test_textarea_cursor_position() {
        let textarea = TextAreaWidget::new(false, 10);
        let (row, col) = textarea.cursor();
        assert_eq!(row, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_textarea_copy_text() {
        let mut textarea = TextAreaWidget::new(false, 10);
        textarea.set_text("Copy this text");
        
        let copied = textarea.copy_text();
        assert_eq!(copied, "Copy this text");
    }

    #[test]
    fn test_textarea_paste_text() {
        let mut textarea = TextAreaWidget::new(false, 10);
        textarea.paste_text("Pasted text");
        
        assert_eq!(textarea.text(), "Pasted text");
    }

    #[test]
    fn test_textarea_required_height() {
        let mut textarea = TextAreaWidget::new(false, 5);
        
        textarea.set_text("Line 1\nLine 2\nLine 3");
        let height = textarea.required_height();
        assert_eq!(height, 3);

        textarea.set_text("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6");
        let height = textarea.required_height();
        assert_eq!(height, 5); // Capped at max_height
    }

    #[test]
    fn test_textarea_vim_mode() {
        let textarea = TextAreaWidget::new(true, 10);
        assert!(textarea.vim_mode);

        let textarea = TextAreaWidget::new(false, 10);
        assert!(!textarea.vim_mode);
    }
}
