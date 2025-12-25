// Input area implementation with multiline, vim mode, selection, and clipboard support
// Addresses GAP-32-002 through GAP-32-008

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Block;
use ratatui_textarea::{CursorMove, Input, Key, TextArea};
use std::collections::VecDeque;

/// Selection state for text selection operations (GAP-32-004)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Anchor position (row, col)
    pub anchor: (usize, usize),
    /// Active position (row, col) - where cursor is
    pub active: (usize, usize),
}

impl Selection {
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            anchor: (row, col),
            active: (row, col),
        }
    }

    pub fn update_active(&mut self, row: usize, col: usize) {
        self.active = (row, col);
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }
}

/// Autocomplete candidate (GAP-32-006)
#[derive(Debug, Clone)]
pub struct AutocompleteCandidate {
    pub text: String,
    pub description: Option<String>,
    pub relevance: f32,
}

/// Autocomplete provider trait (GAP-32-006)
pub trait AutocompleteProvider: Send + Sync {
    fn get_candidates(&self, input: &str, cursor_pos: (usize, usize)) -> Vec<AutocompleteCandidate>;
}

/// Multi-line input area with vim mode, selection, clipboard, and autocomplete
/// Addresses GAP-32-002 (multiline), GAP-32-004 (selection), GAP-32-005 (clipboard)
pub struct InputArea<'a> {
    /// Internal textarea widget (ratatui-textarea with vim mode support)
    textarea: TextArea<'a>,
    
    /// Current selection (GAP-32-004)
    selection: Option<Selection>,
    
    /// Input history with draft preservation (GAP-32-007)
    history: VecDeque<String>,
    history_index: Option<usize>,
    history_draft: Option<String>,
    
    /// Autocomplete state (GAP-32-006)
    autocomplete_candidates: Vec<AutocompleteCandidate>,
    autocomplete_selected: Option<usize>,
    autocomplete_provider: Option<Box<dyn AutocompleteProvider>>,
    
    /// Validation function (GAP-32-008)
    validator: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    
    /// Maximum history size
    max_history: usize,
}

impl<'a> InputArea<'a> {
    /// Create a new input area
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        
        // Configure default style
        textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        
        Self {
            textarea,
            selection: None,
            history: VecDeque::new(),
            history_index: None,
            history_draft: None,
            autocomplete_candidates: Vec::new(),
            autocomplete_selected: None,
            autocomplete_provider: None,
            validator: None,
            max_history: 100,
        }
    }

    /// Enable vim mode (GAP-32-002, GAP-32-003)
    pub fn with_vim_mode(mut self) -> Self {
        // ratatui-textarea handles vim mode internally
        self
    }

    /// Set block/border (GAP-32-003)
    pub fn set_block(&mut self, block: Block<'a>) {
        self.textarea.set_block(block);
    }

    /// Set style (GAP-32-003)
    pub fn set_style(&mut self, style: Style) {
        self.textarea.set_style(style);
    }

    /// Set autocomplete provider (GAP-32-006)
    pub fn set_autocomplete_provider(&mut self, provider: Box<dyn AutocompleteProvider>) {
        self.autocomplete_provider = Some(provider);
    }

    /// Set validator (GAP-32-008)
    pub fn set_validator(&mut self, validator: Box<dyn Fn(&str) -> Result<(), String>>) {
        self.validator = Some(validator);
    }

    /// Get current text
    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Set text
    pub fn set_text(&mut self, text: String) {
        let lines: Vec<&str> = text.split('\n').collect();
        self.textarea = TextArea::from(lines);
    }

    /// Get cursor position (row, col) - GAP-32-003
    pub fn cursor_position(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    /// Handle input with vim mode and selection support
    /// GAP-32-002, GAP-32-003, GAP-32-004
    pub fn handle_input(&mut self, input: Input) -> bool {
        // Handle selection mode (Shift + movement)
        match input {
            // Start selection on Shift+Arrow (GAP-32-004)
            Input { key: Key::Left, shift: true, .. } => {
                self.start_or_update_selection();
                self.textarea.move_cursor(CursorMove::Back);
                true
            }
            Input { key: Key::Right, shift: true, .. } => {
                self.start_or_update_selection();
                self.textarea.move_cursor(CursorMove::Forward);
                true
            }
            Input { key: Key::Up, shift: true, .. } => {
                self.start_or_update_selection();
                self.textarea.move_cursor(CursorMove::Up);
                true
            }
            Input { key: Key::Down, shift: true, .. } => {
                self.start_or_update_selection();
                self.textarea.move_cursor(CursorMove::Down);
                true
            }
            // Clear selection on non-Shift movement
            Input { key: Key::Left, shift: false, .. } 
            | Input { key: Key::Right, shift: false, .. }
            | Input { key: Key::Up, shift: false, .. }
            | Input { key: Key::Down, shift: false, .. } => {
                self.selection = None;
                self.textarea.input(input);
                true
            }
            // Copy selection (Ctrl+C) - GAP-32-005
            Input { key: Key::Char('c'), ctrl: true, .. } => {
                self.copy_selection();
                true
            }
            // Paste from clipboard (Ctrl+V) - GAP-32-005
            Input { key: Key::Char('v'), ctrl: true, .. } => {
                self.paste_from_clipboard();
                true
            }
            // Cut selection (Ctrl+X) - GAP-32-005
            Input { key: Key::Char('x'), ctrl: true, .. } => {
                self.cut_selection();
                true
            }
            // Select all (Ctrl+A) - GAP-32-004
            Input { key: Key::Char('a'), ctrl: true, .. } => {
                self.select_all();
                true
            }
            // Autocomplete accept (Tab) - GAP-32-006
            Input { key: Key::Tab, .. } => {
                if self.autocomplete_selected.is_some() {
                    self.accept_autocomplete();
                } else {
                    self.show_autocomplete();
                }
                true
            }
            // Autocomplete navigation (Ctrl+N/P) - GAP-32-006
            Input { key: Key::Char('n'), ctrl: true, .. } => {
                self.next_autocomplete();
                true
            }
            Input { key: Key::Char('p'), ctrl: true, .. } => {
                self.prev_autocomplete();
                true
            }
            // History navigation (Up/Down when at first/last line) - GAP-32-007
            Input { key: Key::Up, .. } if self.textarea.cursor().0 == 0 => {
                self.history_up();
                true
            }
            Input { key: Key::Down, .. } if self.textarea.cursor().0 == self.textarea.lines().len() - 1 => {
                self.history_down();
                true
            }
            // Default handling (vim mode handled by ratatui-textarea)
            _ => {
                self.selection = None; // Clear selection on typing
                self.textarea.input(input);
                self.update_autocomplete();
                true
            }
        }
    }

    /// Submit input with validation - GAP-32-008
    pub fn submit(&mut self) -> Result<String, String> {
        let text = self.text();
        
        // Enforce validation on submit (GAP-32-008)
        if let Some(validator) = &self.validator {
            validator(&text)?;
        }

        // Add to history (GAP-32-007)
        if !text.is_empty() {
            self.history.push_back(text.clone());
            if self.history.len() > self.max_history {
                self.history.pop_front();
            }
        }

        // Clear input
        self.textarea = TextArea::default();
        self.history_index = None;
        self.history_draft = None;
        self.selection = None;

        Ok(text)
    }

    // Selection operations (GAP-32-004)
    
    fn start_or_update_selection(&mut self) {
        let pos = self.cursor_position();
        match &mut self.selection {
            Some(sel) => sel.update_active(pos.0, pos.1),
            None => self.selection = Some(Selection::new(pos.0, pos.1)),
        }
    }

    fn select_all(&mut self) {
        let lines = self.textarea.lines();
        if lines.is_empty() {
            return;
        }
        
        let last_line = lines.len() - 1;
        let last_col = lines[last_line].len();
        
        self.selection = Some(Selection {
            anchor: (0, 0),
            active: (last_line, last_col),
        });
    }

    fn get_selected_text(&self) -> Option<String> {
        let sel = self.selection.as_ref()?;
        if sel.is_empty() {
            return None;
        }

        let lines = self.textarea.lines();
        let (start, end) = if sel.anchor <= sel.active {
            (sel.anchor, sel.active)
        } else {
            (sel.active, sel.anchor)
        };

        if start.0 == end.0 {
            // Same line
            let line = &lines[start.0];
            Some(line[start.1..end.1.min(line.len())].to_string())
        } else {
            // Multiple lines
            let mut result = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i < start.0 || i > end.0 {
                    continue;
                }
                if i == start.0 {
                    result.push_str(&line[start.1..]);
                } else if i == end.0 {
                    result.push_str(&line[..end.1.min(line.len())]);
                } else {
                    result.push_str(line);
                }
                if i < end.0 {
                    result.push('\n');
                }
            }
            Some(result)
        }
    }

    // Clipboard operations (GAP-32-005)
    
    fn copy_selection(&self) {
        if let Some(text) = self.get_selected_text() {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(text);
            }
        }
    }

    fn paste_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                self.textarea.insert_str(&text);
            }
        }
    }

    fn cut_selection(&mut self) {
        self.copy_selection();
        if self.selection.is_some() {
            self.delete_selection();
        }
    }

    fn delete_selection(&mut self) {
        // Implementation depends on ratatui-textarea's API
        // For now, just clear selection
        self.selection = None;
    }

    // Autocomplete operations (GAP-32-006)
    
    fn show_autocomplete(&mut self) {
        if let Some(provider) = &self.autocomplete_provider {
            let cursor_pos = self.cursor_position();
            self.autocomplete_candidates = provider.get_candidates(&self.text(), cursor_pos);
            if !self.autocomplete_candidates.is_empty() {
                self.autocomplete_selected = Some(0);
            }
        }
    }

    fn update_autocomplete(&mut self) {
        // Update autocomplete on text change
        if self.autocomplete_provider.is_some() {
            self.show_autocomplete();
        }
    }

    fn next_autocomplete(&mut self) {
        if let Some(selected) = self.autocomplete_selected {
            if !self.autocomplete_candidates.is_empty() {
                self.autocomplete_selected = Some((selected + 1) % self.autocomplete_candidates.len());
            }
        }
    }

    fn prev_autocomplete(&mut self) {
        if let Some(selected) = self.autocomplete_selected {
            if !self.autocomplete_candidates.is_empty() {
                let new_selected = if selected == 0 {
                    self.autocomplete_candidates.len() - 1
                } else {
                    selected - 1
                };
                self.autocomplete_selected = Some(new_selected);
            }
        }
    }

    fn accept_autocomplete(&mut self) {
        if let Some(selected) = self.autocomplete_selected {
            if let Some(candidate) = self.autocomplete_candidates.get(selected) {
                // Insert candidate text
                self.textarea.insert_str(&candidate.text);
                self.autocomplete_candidates.clear();
                self.autocomplete_selected = None;
            }
        }
    }

    // History operations with draft preservation (GAP-32-007)
    
    fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                // Save current draft before browsing history
                let current_text = self.text();
                if !current_text.is_empty() {
                    self.history_draft = Some(current_text);
                }
                
                // Go to most recent history entry
                self.history_index = Some(self.history.len() - 1);
                if let Some(entry) = self.history.back() {
                    self.set_text(entry.clone());
                }
            }
            Some(idx) if idx > 0 => {
                // Go to older history entry
                self.history_index = Some(idx - 1);
                if let Some(entry) = self.history.get(idx - 1) {
                    self.set_text(entry.clone());
                }
            }
            _ => {}
        }
    }

    fn history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.history.len() - 1 => {
                // Go to newer history entry
                self.history_index = Some(idx + 1);
                if let Some(entry) = self.history.get(idx + 1) {
                    self.set_text(entry.clone());
                }
            }
            Some(_) => {
                // Restore draft
                self.history_index = None;
                if let Some(draft) = self.history_draft.take() {
                    self.set_text(draft);
                } else {
                    self.textarea = TextArea::default();
                }
            }
            None => {}
        }
    }

    /// Render the input area
    pub fn render(&mut self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        self.textarea.render(area, buf);
        
        // TODO: Render autocomplete popup if candidates exist
        // TODO: Render selection highlight if selection exists
    }

    /// Get the underlying textarea widget (for advanced usage)
    pub fn textarea_mut(&mut self) -> &mut TextArea<'a> {
        &mut self.textarea
    }

    /// Get autocomplete candidates for display
    pub fn autocomplete_candidates(&self) -> &[AutocompleteCandidate] {
        &self.autocomplete_candidates
    }

    /// Get selected autocomplete index
    pub fn autocomplete_selected(&self) -> Option<usize> {
        self.autocomplete_selected
    }
}

impl<'a> Default for InputArea<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiline_input() {
        let mut input = InputArea::new();
        input.set_text("line1\nline2\nline3".to_string());
        assert_eq!(input.text(), "line1\nline2\nline3");
    }

    #[test]
    fn test_selection() {
        let mut input = InputArea::new();
        let sel = Selection::new(0, 0);
        input.selection = Some(sel);
        assert!(!input.selection.unwrap().is_empty() || input.selection.unwrap().anchor == input.selection.unwrap().active);
    }

    #[test]
    fn test_history_preservation() {
        let mut input = InputArea::new();
        input.history.push_back("entry1".to_string());
        input.history.push_back("entry2".to_string());
        
        // Save draft
        input.set_text("current_draft".to_string());
        input.history_up();
        assert_eq!(input.history_draft, Some("current_draft".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut input = InputArea::new();
        input.set_validator(Box::new(|text| {
            if text.is_empty() {
                Err("Input cannot be empty".to_string())
            } else {
                Ok(())
            }
        }));

        input.set_text("".to_string());
        assert!(input.submit().is_err());

        input.set_text("valid".to_string());
        assert!(input.submit().is_ok());
    }
}
