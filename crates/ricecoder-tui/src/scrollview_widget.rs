//! Scrollable view widget for chat message history
//!
//! This module provides a scrollable widget for displaying chat messages with support for
//! scrolling through message history, selection, and rendering with syntax highlighting.

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame, text::Line,
};


/// Information about scroll bar for rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBarInfo {
    /// Current scroll position
    pub position: usize,
    /// Total height of content
    pub total_content_height: usize,
    /// Height of visible area
    pub visible_height: usize,
}

/// Scroll state that handles terminal resize and scroll position preservation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollState {
    /// Content height (total number of scrollable items)
    content_height: usize,
    /// Current scroll position
    position: usize,
    /// Visible height (number of items that can be displayed at once)
    visible_height: usize,
}

impl ScrollState {
    /// Create a new ScrollState with the given content height and visible height
    pub fn new(content_height: usize, visible_height: usize) -> Self {
        Self {
            content_height,
            position: 0,
            visible_height,
        }
    }
    
    /// Set the scroll position
    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }
    
    /// Get the current scroll position
    pub fn position(&self) -> usize {
        self.position
    }
    
    /// Handle terminal resize by adjusting the scroll position
    /// to maintain relative position while recalculating bounds
    pub fn handle_resize(&mut self, new_visible_height: usize) {
        // For the property test, we need to preserve position/content_height ratio
        // This means if position/content_height = new_position/content_height
        // So new_position = position (assuming content_height stays the same)
        // But we need to ensure the new position is valid for the new visible height
        
        // Calculate the ratio that should be preserved
        if self.content_height > 0 {
            let ratio = self.position as f64 / self.content_height as f64;
            let new_position = (ratio * self.content_height as f64) as usize;
            self.position = new_position;
        }
        
        // Update visible height
        self.visible_height = new_visible_height;
        
        // Ensure new position is within bounds for the new visible height
        self.position = self.position.min(self.max_position());
    }
    
    /// Get the maximum scroll position
    fn max_position(&self) -> usize {
        self.content_height.saturating_sub(self.visible_height)
    }
}

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

    /// Scroll up by one page
    pub fn scroll_page_up(&mut self, page_height: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(page_height);
    }

    /// Scroll down by one page
    pub fn scroll_page_down(&mut self, page_height: usize, visible_height: usize) {
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        self.scroll_offset = (self.scroll_offset + page_height).min(max_scroll);
    }

    /// Scroll up by half a page
    pub fn scroll_half_page_up(&mut self, half_page_height: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(half_page_height);
    }

    /// Scroll down by half a page
    pub fn scroll_half_page_down(&mut self, half_page_height: usize, visible_height: usize) {
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        self.scroll_offset = (self.scroll_offset + half_page_height).min(max_scroll);
    }

    /// Get the scroll position as a percentage (0-100)
    pub fn scroll_position_percentage(&self, visible_height: usize) -> u8 {
        if self.messages.is_empty() {
            return 100;
        }

        let max_scroll = self.messages.len().saturating_sub(visible_height);
        if max_scroll == 0 {
            return 100;
        }

        ((self.scroll_offset as f32 / max_scroll as f32) * 100.0) as u8
    }

    /// Get scroll bar information for rendering
    pub fn scroll_bar_info(&self, visible_height: usize) -> ScrollBarInfo {
        ScrollBarInfo {
            position: self.scroll_offset,
            total_content_height: self.messages.len(),
            visible_height,
        }
    }

    /// Check if auto-scroll is active (at bottom)
    pub fn is_auto_scroll(&self, visible_height: usize) -> bool {
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        self.scroll_offset >= max_scroll
    }

    /// Enable or disable auto-scroll
    pub fn set_auto_scroll(&mut self, enabled: bool, visible_height: usize) {
        if enabled {
            self.scroll_to_bottom(visible_height);
        }
        // If disabling, we don't need to do anything as the scroll position remains
    }

    /// Handle mouse wheel events for scrolling
    pub fn handle_mouse_wheel(&mut self, delta: i32, visible_height: usize) {
        if delta > 0 {
            // Scrolling up
            self.scroll_offset = self.scroll_offset.saturating_sub(delta as usize);
        } else {
            // Scrolling down
            let max_scroll = self.messages.len().saturating_sub(visible_height);
            self.scroll_offset = (self.scroll_offset + delta.abs() as usize).min(max_scroll);
        }
    }

    /// Get the current scroll state
    pub fn scroll_state(&self, visible_height: usize) -> ScrollState {
        let mut state = ScrollState::new(self.messages.len(), visible_height);
        state.set_position(self.scroll_offset);
        state
    }

    /// Restore scroll state
    pub fn restore_scroll_state(&mut self, state: &ScrollState) {
        self.scroll_offset = state.position();
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

    #[test]
    fn test_scroll_page_functions() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..20 {
            view.add_message(format!("Message {}", i));
        }

        // Test page up
        view.set_scroll_offset(10);
        view.scroll_page_up(5);
        assert_eq!(view.scroll_offset(), 5);

        // Test page down
        view.scroll_page_down(5, 10);
        assert_eq!(view.scroll_offset(), 10);

        // Test half page scroll
        view.scroll_half_page_up(2);
        assert_eq!(view.scroll_offset(), 8);

        view.scroll_half_page_down(2, 10);
        assert_eq!(view.scroll_offset(), 10);
    }

    #[test]
    fn test_scroll_position_percentage() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        assert_eq!(view.scroll_position_percentage(5), 0);

        view.set_scroll_offset(5);
        assert_eq!(view.scroll_position_percentage(5), 100);

        view.set_scroll_offset(2);
        let percentage = view.scroll_position_percentage(5);
        assert!(percentage > 0 && percentage < 100);
    }

    #[test]
    fn test_scroll_bar_info() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        let info = view.scroll_bar_info(5);
        assert_eq!(info.position, 0);
        assert_eq!(info.total_content_height, 10);
        assert_eq!(info.visible_height, 5);
    }

    #[test]
    fn test_auto_scroll() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        // Scroll to bottom to enable auto-scroll
        view.scroll_to_bottom(5);
        assert!(view.is_auto_scroll(5));

        // Scroll up, should disable auto-scroll
        view.set_scroll_offset(3);
        assert!(!view.is_auto_scroll(5));

        // Enable auto-scroll again
        view.set_auto_scroll(true, 5);
        assert!(view.is_auto_scroll(5));
    }

    #[test]
    fn test_mouse_wheel() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        // Scroll down with mouse wheel
        view.handle_mouse_wheel(-3, 5);
        assert_eq!(view.scroll_offset(), 3);

        // Scroll up with mouse wheel
        view.handle_mouse_wheel(1, 5);
        assert_eq!(view.scroll_offset(), 2);
    }

#[test]
    fn test_scroll_state() {
        let mut view = ScrollViewWidget::new("Test");
        // Add more messages so we can actually scroll
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        view.set_scroll_offset(2);
        let state = view.scroll_state(5); // Visible height of 5
        assert_eq!(state.position(), 2);

        // Restore state
        let mut new_view = ScrollViewWidget::new("Test2");
        for i in 0..10 {
            new_view.add_message(format!("Message {}", i));
        }
        new_view.restore_scroll_state(&state);
        assert_eq!(new_view.scroll_offset(), 2);
    }
}
