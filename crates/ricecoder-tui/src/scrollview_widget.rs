//! Scrollable view widget for chat message history
//!
//! This module provides a scrollable widget for displaying chat messages with support for
//! scrolling through message history, selection, and rendering with syntax highlighting.

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame, text::Line,
};

/// Scrollable view widget for displaying chat messages
pub struct ScrollViewWidget {
    /// Messages to display
    messages: Vec<String>,
    /// Current scroll position
    scroll_offset: usize,
    /// Selected message index
    selected: Option<usize>,
    /// Title for the block
    title: String,
    /// Whether to show borders
    show_borders: bool,
}

impl ScrollViewWidget {
    /// Create a new scrollable view widget
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            selected: None,
            title: title.into(),
            show_borders: true,
        }
    }

    /// Add a message to the view
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
        self.selected = None;
    }

    /// Get the number of messages
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self, visible_height: usize) {
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self, visible_height: usize) {
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        self.scroll_offset = max_scroll;
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set the scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Select a message by index
    pub fn select(&mut self, index: usize) {
        if index < self.messages.len() {
            self.selected = Some(index);
        }
    }

    /// Deselect the current message
    pub fn deselect(&mut self) {
        self.selected = None;
    }

    /// Get the selected message index
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Get the selected message text
    pub fn selected_message(&self) -> Option<&str> {
        self.selected
            .and_then(|idx| self.messages.get(idx))
            .map(|s| s.as_str())
    }

    /// Select the next message
    pub fn select_next(&mut self) {
        match self.selected {
            None => self.selected = Some(0),
            Some(idx) if idx < self.messages.len() - 1 => self.selected = Some(idx + 1),
            _ => {}
        }
    }

    /// Select the previous message
    pub fn select_prev(&mut self) {
        match self.selected {
            None => {}
            Some(0) => self.selected = None,
            Some(idx) => self.selected = Some(idx - 1),
        }
    }

    /// Get visible messages based on scroll offset and height
    pub fn visible_messages(&self, height: usize) -> Vec<&str> {
        self.messages
            .iter()
            .skip(self.scroll_offset)
            .take(height)
            .map(|s| s.as_str())
            .collect()
    }

    /// Set whether to show borders
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Get the total height needed to display all messages
    pub fn total_height(&self) -> usize {
        self.messages.len()
    }

    /// Check if at the top
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Check if at the bottom
    pub fn is_at_bottom(&self, visible_height: usize) -> bool {
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        self.scroll_offset >= max_scroll
    }

    /// Get the scroll percentage (0-100)
    pub fn scroll_percentage(&self, visible_height: usize) -> u8 {
        if self.messages.is_empty() {
            return 100;
        }

        let max_scroll = self.messages.len().saturating_sub(visible_height);
        if max_scroll == 0 {
            return 100;
        }

        ((self.scroll_offset as f32 / max_scroll as f32) * 100.0) as u8
    }

    /// Render the widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = if self.show_borders {
            Block::default()
                .title(self.title.as_str())
                .borders(Borders::ALL)
        } else {
            Block::default()
        };

        let inner = if self.show_borders {
            block.inner(area)
        } else {
            area
        };

        // Get visible messages
        let visible = self.visible_messages(inner.height as usize);
        let lines: Vec<Line> = visible
            .iter()
            .enumerate()
            .map(|(idx, msg)| {
                let actual_idx = self.scroll_offset + idx;
                let is_selected = self.selected == Some(actual_idx);

                if is_selected {
                    Line::raw(format!("> {}", msg))
                } else {
                    Line::raw(format!("  {}", msg))
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        f.render_widget(block, area);
        f.render_widget(paragraph, inner);
    }
}

impl Default for ScrollViewWidget {
    fn default() -> Self {
        Self::new("Messages")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollview_creation() {
        let view = ScrollViewWidget::new("Test");
        assert_eq!(view.message_count(), 0);
        assert_eq!(view.scroll_offset(), 0);
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_add_message() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");

        assert_eq!(view.message_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.select(0);

        view.clear();
        assert_eq!(view.message_count(), 0);
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_scroll_up() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        view.set_scroll_offset(5);
        view.scroll_up();
        assert_eq!(view.scroll_offset(), 4);

        view.scroll_to_top();
        view.scroll_up();
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        view.scroll_down(5);
        assert_eq!(view.scroll_offset(), 1);

        view.scroll_to_bottom(5);
        assert_eq!(view.scroll_offset(), 5);
    }

    #[test]
    fn test_selection() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.add_message("Message 3");

        view.select(1);
        assert_eq!(view.selected(), Some(1));
        assert_eq!(view.selected_message(), Some("Message 2"));

        view.deselect();
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_select_next() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.add_message("Message 3");

        assert!(view.selected().is_none());

        view.select_next();
        assert_eq!(view.selected(), Some(0));

        view.select_next();
        assert_eq!(view.selected(), Some(1));

        view.select_next();
        assert_eq!(view.selected(), Some(2));

        view.select_next();
        assert_eq!(view.selected(), Some(2)); // Stay at last
    }

    #[test]
    fn test_select_prev() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.add_message("Message 3");

        view.select(2);
        view.select_prev();
        assert_eq!(view.selected(), Some(1));

        view.select_prev();
        assert_eq!(view.selected(), Some(0));

        view.select_prev();
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_visible_messages() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        let visible = view.visible_messages(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0], "Message 0");

        view.set_scroll_offset(5);
        let visible = view.visible_messages(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0], "Message 5");
    }

    #[test]
    fn test_scroll_percentage() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        assert_eq!(view.scroll_percentage(5), 0);

        view.set_scroll_offset(5);
        assert_eq!(view.scroll_percentage(5), 100);

        view.set_scroll_offset(2);
        let percentage = view.scroll_percentage(5);
        assert!(percentage > 0 && percentage < 100);
    }

    #[test]
    fn test_is_at_top_bottom() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        assert!(view.is_at_top());
        assert!(!view.is_at_bottom(5));

        view.scroll_to_bottom(5);
        assert!(!view.is_at_top());
        assert!(view.is_at_bottom(5));
    }

    #[test]
    fn test_total_height() {
        let mut view = ScrollViewWidget::new("Test");
        assert_eq!(view.total_height(), 0);

        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }
        assert_eq!(view.total_height(), 10);
    }

    #[test]
    fn test_set_title() {
        let mut view = ScrollViewWidget::new("Original");
        assert_eq!(view.title, "Original");

        view.set_title("New Title");
        assert_eq!(view.title, "New Title");
    }

    #[test]
    fn test_set_show_borders() {
        let mut view = ScrollViewWidget::new("Test");
        assert!(view.show_borders);

        view.set_show_borders(false);
        assert!(!view.show_borders);
    }
}
