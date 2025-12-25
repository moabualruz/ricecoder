//! Mouse selection handling for RiceCoder TUI
//!
//! Manages text selection in the prompt:
//! - Mouse drag selection
//! - Double-click word selection
//! - Triple-click line selection
//! - Selection highlighting
//! - Copy on select (optional)
//!
//! # DDD Layer: Infrastructure
//! Mouse input handling for text selection.

use ratatui::layout::Position;
use std::ops::Range;

/// Selection state
#[derive(Debug, Clone, Default)]
pub struct Selection {
    /// Start position (anchor)
    pub start: Option<SelectionPoint>,
    /// End position (cursor)
    pub end: Option<SelectionPoint>,
    /// Whether selection is active (mouse down)
    pub active: bool,
    /// Selection mode
    pub mode: SelectionMode,
}

/// A point in the selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPoint {
    /// Row (line number)
    pub row: usize,
    /// Column (character index)
    pub col: usize,
    /// Byte offset in text
    pub offset: usize,
}

impl SelectionPoint {
    pub fn new(row: usize, col: usize, offset: usize) -> Self {
        Self { row, col, offset }
    }
    
    /// Create from screen position and text
    pub fn from_position(pos: Position, text: &str, line_offsets: &[usize]) -> Self {
        let row = pos.y as usize;
        let col = pos.x as usize;
        
        let offset = if row < line_offsets.len() {
            let line_start = line_offsets[row];
            let line = text.get(line_start..).and_then(|s| s.lines().next()).unwrap_or("");
            line_start + col.min(line.len())
        } else {
            text.len()
        };
        
        Self { row, col, offset }
    }
}

impl PartialOrd for SelectionPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SelectionPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.offset.cmp(&other.offset)
    }
}

/// Selection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    #[default]
    Character,
    Word,
    Line,
}

impl Selection {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Start a new selection
    pub fn start(&mut self, point: SelectionPoint, mode: SelectionMode) {
        self.start = Some(point);
        self.end = Some(point);
        self.active = true;
        self.mode = mode;
    }
    
    /// Update selection end point
    pub fn update(&mut self, point: SelectionPoint) {
        if self.active {
            self.end = Some(point);
        }
    }
    
    /// Finish selection
    pub fn finish(&mut self) {
        self.active = false;
    }
    
    /// Clear selection
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
        self.active = false;
        self.mode = SelectionMode::Character;
    }
    
    /// Check if there's a valid selection
    pub fn has_selection(&self) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => start.offset != end.offset,
            _ => false,
        }
    }
    
    /// Get selection range (normalized: start < end)
    pub fn range(&self) -> Option<Range<usize>> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => {
                let (min, max) = if start.offset <= end.offset {
                    (start.offset, end.offset)
                } else {
                    (end.offset, start.offset)
                };
                if min != max {
                    Some(min..max)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Get selected text
    pub fn selected_text<'a>(&self, text: &'a str) -> Option<&'a str> {
        self.range().and_then(|r| text.get(r))
    }
    
    /// Expand selection to word boundaries
    pub fn expand_to_word(&mut self, text: &str) {
        if let Some(range) = self.range() {
            let start = find_word_start(text, range.start);
            let end = find_word_end(text, range.end);
            
            if let (Some(s), Some(e)) = (self.start.as_mut(), self.end.as_mut()) {
                if s.offset <= e.offset {
                    s.offset = start;
                    e.offset = end;
                } else {
                    e.offset = start;
                    s.offset = end;
                }
            }
        }
    }
    
    /// Expand selection to line boundaries
    pub fn expand_to_line(&mut self, text: &str) {
        if let Some(range) = self.range() {
            let start = find_line_start(text, range.start);
            let end = find_line_end(text, range.end);
            
            if let (Some(s), Some(e)) = (self.start.as_mut(), self.end.as_mut()) {
                if s.offset <= e.offset {
                    s.offset = start;
                    e.offset = end;
                } else {
                    e.offset = start;
                    s.offset = end;
                }
            }
        }
    }
    
    /// Check if a position is within selection
    pub fn contains(&self, offset: usize) -> bool {
        self.range().map(|r| r.contains(&offset)).unwrap_or(false)
    }
}

/// Find word start boundary
fn find_word_start(text: &str, pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = pos;
    while i > 0 && is_word_char(bytes.get(i - 1).copied().unwrap_or(0)) {
        i -= 1;
    }
    i
}

/// Find word end boundary
fn find_word_end(text: &str, pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = pos;
    while i < bytes.len() && is_word_char(bytes[i]) {
        i += 1;
    }
    i
}

/// Find line start
fn find_line_start(text: &str, pos: usize) -> usize {
    text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

/// Find line end
fn find_line_end(text: &str, pos: usize) -> usize {
    text[pos..].find('\n').map(|i| pos + i).unwrap_or(text.len())
}

/// Check if byte is a word character
fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Calculate line offsets for text
pub fn calculate_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Mouse event handler for selection
pub struct SelectionHandler {
    selection: Selection,
    /// Last click time for double/triple click detection
    last_click_time: std::time::Instant,
    /// Click count for multi-click detection
    click_count: u8,
    /// Copy on select enabled
    copy_on_select: bool,
}

impl SelectionHandler {
    pub fn new() -> Self {
        Self {
            selection: Selection::new(),
            last_click_time: std::time::Instant::now(),
            click_count: 0,
            copy_on_select: false,
        }
    }
    
    pub fn with_copy_on_select(mut self, enabled: bool) -> Self {
        self.copy_on_select = enabled;
        self
    }
    
    pub fn selection(&self) -> &Selection {
        &self.selection
    }
    
    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }
    
    /// Handle mouse down
    pub fn on_mouse_down(&mut self, pos: Position, text: &str, line_offsets: &[usize]) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_click_time);
        
        // Multi-click detection (within 500ms)
        if elapsed.as_millis() < 500 {
            self.click_count = (self.click_count + 1).min(3);
        } else {
            self.click_count = 1;
        }
        self.last_click_time = now;
        
        let point = SelectionPoint::from_position(pos, text, line_offsets);
        
        let mode = match self.click_count {
            2 => SelectionMode::Word,
            3 => SelectionMode::Line,
            _ => SelectionMode::Character,
        };
        
        self.selection.start(point, mode);
        
        // Expand immediately for word/line selection
        match mode {
            SelectionMode::Word => self.selection.expand_to_word(text),
            SelectionMode::Line => self.selection.expand_to_line(text),
            _ => {}
        }
    }
    
    /// Handle mouse drag
    pub fn on_mouse_drag(&mut self, pos: Position, text: &str, line_offsets: &[usize]) {
        let point = SelectionPoint::from_position(pos, text, line_offsets);
        self.selection.update(point);
        
        // Maintain word/line boundaries during drag
        match self.selection.mode {
            SelectionMode::Word => self.selection.expand_to_word(text),
            SelectionMode::Line => self.selection.expand_to_line(text),
            _ => {}
        }
    }
    
    /// Handle mouse up
    pub fn on_mouse_up(&mut self, text: &str) -> Option<String> {
        self.selection.finish();
        
        if self.copy_on_select && self.selection.has_selection() {
            self.selection.selected_text(text).map(|s| s.to_string())
        } else {
            None
        }
    }
    
    /// Clear selection
    pub fn clear(&mut self) {
        self.selection.clear();
    }
}

impl Default for SelectionHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_selection_point_ordering() {
        let p1 = SelectionPoint::new(0, 0, 5);
        let p2 = SelectionPoint::new(0, 5, 10);
        assert!(p1 < p2);
    }
    
    #[test]
    fn test_selection_range() {
        let mut sel = Selection::new();
        sel.start = Some(SelectionPoint::new(0, 0, 5));
        sel.end = Some(SelectionPoint::new(0, 10, 15));
        
        assert_eq!(sel.range(), Some(5..15));
        
        // Reversed
        sel.start = Some(SelectionPoint::new(0, 10, 15));
        sel.end = Some(SelectionPoint::new(0, 0, 5));
        assert_eq!(sel.range(), Some(5..15));
    }
    
    #[test]
    fn test_selected_text() {
        let text = "hello world";
        let mut sel = Selection::new();
        sel.start = Some(SelectionPoint::new(0, 0, 0));
        sel.end = Some(SelectionPoint::new(0, 5, 5));
        
        assert_eq!(sel.selected_text(text), Some("hello"));
    }
    
    #[test]
    fn test_word_boundaries() {
        let text = "hello world";
        assert_eq!(find_word_start(text, 3), 0);
        assert_eq!(find_word_end(text, 3), 5);
        assert_eq!(find_word_start(text, 8), 6);
        assert_eq!(find_word_end(text, 8), 11);
    }
    
    #[test]
    fn test_line_boundaries() {
        let text = "line1\nline2\nline3";
        assert_eq!(find_line_start(text, 8), 6);
        assert_eq!(find_line_end(text, 8), 11);
    }
    
    #[test]
    fn test_calculate_line_offsets() {
        let text = "line1\nline2\nline3";
        let offsets = calculate_line_offsets(text);
        assert_eq!(offsets, vec![0, 6, 12]);
    }
    
    #[test]
    fn test_selection_handler() {
        let mut handler = SelectionHandler::new();
        let text = "hello world";
        let offsets = calculate_line_offsets(text);
        
        handler.on_mouse_down(Position::new(0, 0), text, &offsets);
        assert!(handler.selection().active);
        
        handler.on_mouse_drag(Position::new(5, 0), text, &offsets);
        handler.on_mouse_up(text);
        
        assert!(!handler.selection().active);
    }
}
