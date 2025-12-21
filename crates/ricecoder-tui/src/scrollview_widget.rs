//! Scrollable view widget for chat message history
//!
//! This module provides a scrollable widget for displaying chat messages with support for
//! scrolling through message history, selection, and rendering with syntax highlighting.

use ratatui::{
    layout::{Rect, Size},
    prelude::StatefulWidget,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_scrollview::{ScrollView, ScrollViewState};

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
    /// Scroll view state from tui-scrollview
    scroll_state: ScrollViewState,
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
            scroll_state: ScrollViewState::default(),
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
        self.scroll_state = ScrollViewState::default();
        self.selected = None;
    }

    /// Get the number of messages
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Scroll to the bottom of the content
    pub fn scroll_to_bottom(&mut self, visible_height: usize) {
        // Calculate the maximum scroll position
        let content_height = self.messages.len();
        if content_height > visible_height {
            let max_scroll = content_height - visible_height;
            // Scroll down to the bottom
            for _ in 0..max_scroll {
                self.scroll_state.scroll_down();
            }
        }
    }

    /// Enable or disable auto-scroll
    pub fn set_auto_scroll(&mut self, enabled: bool, visible_height: usize) {
        if enabled {
            self.scroll_to_bottom(visible_height);
        }
        // If disabling, we don't need to do anything as the scroll position remains
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
        // TODO: Implement with ScrollViewState offset
        self.messages
            .iter()
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
        // TODO: Check ScrollViewState offset
        true
    }

    /// Check if at the bottom
    pub fn is_at_bottom(&self, _visible_height: usize) -> bool {
        // TODO: Check ScrollViewState offset
        false
    }

    /// Render the widget
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
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

        // Calculate content size (assuming each message is 1 line)
        let content_height = self.messages.len() as u16;
        let content_width = 100; // Estimate, could be calculated
        let content_size = Size::new(content_width, content_height);

        // Create scroll view
        let mut scroll_view = ScrollView::new(content_size);

        // Render messages into the scroll view
        for (idx, message) in self.messages.iter().enumerate() {
            let is_selected = self.selected == Some(idx);
            let line = if is_selected {
                Line::raw(format!("> {}", message))
            } else {
                Line::raw(format!("  {}", message))
            };

            let paragraph = Paragraph::new(line);
            let y_pos = idx as u16;
            scroll_view.render_widget(paragraph, Rect::new(0, y_pos, content_width, 1));
        }

        // Render the block
        f.render_widget(block, area);

        // Render the scroll view with state
        StatefulWidget::render(scroll_view, inner, f.buffer_mut(), &mut self.scroll_state);
    }
}

impl Default for ScrollViewWidget {
    fn default() -> Self {
        Self::new("Messages")
    }
}
