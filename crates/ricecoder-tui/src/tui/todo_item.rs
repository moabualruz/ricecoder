//! Todo item widget for displaying task status
//!
//! This module provides a widget for rendering todo items with status indicators
//! (completed, in progress, pending) and styled text.
//!
//! # Examples
//!
//! ```ignore
//! use ricecoder_tui::tui::todo_item::{TodoItem, TodoStatus};
//! use ratatui::Frame;
//!
//! let item = TodoItem::new(TodoStatus::InProgress, "Implement feature");
//! frame.render_widget(item, area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

/// Todo item status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatus {
    /// Task is completed
    Completed,
    /// Task is in progress
    InProgress,
    /// Task is pending
    Pending,
}

impl TodoStatus {
    /// Get the status character
    pub const fn char(&self) -> &'static str {
        match self {
            TodoStatus::Completed => "✓",
            TodoStatus::InProgress => "•",
            TodoStatus::Pending => " ",
        }
    }

    /// Get the status color
    pub const fn color(&self) -> Color {
        match self {
            TodoStatus::Completed => Color::Green,
            TodoStatus::InProgress => Color::Yellow,
            TodoStatus::Pending => Color::DarkGray,
        }
    }
}

/// Todo item widget
#[derive(Debug, Clone)]
pub struct TodoItem {
    status: TodoStatus,
    content: String,
    muted_color: Color,
}

impl TodoItem {
    /// Create a new todo item
    pub fn new(status: TodoStatus, content: impl Into<String>) -> Self {
        Self {
            status,
            content: content.into(),
            muted_color: Color::DarkGray,
        }
    }

    /// Set the muted color for pending items
    pub fn muted_color(mut self, color: Color) -> Self {
        self.muted_color = color;
        self
    }

    /// Get the status indicator as a styled span
    fn status_span(&self) -> Span {
        Span::styled(
            format!("[{}] ", self.status.char()),
            Style::default().fg(self.status.color()),
        )
    }

    /// Get the content as a styled span
    fn content_span(&self) -> Span {
        let color = if self.status == TodoStatus::Pending {
            self.muted_color
        } else {
            self.status.color()
        };

        Span::styled(&self.content, Style::default().fg(color))
    }
}

impl Widget for TodoItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(vec![self.status_span(), self.content_span()]);

        let paragraph = Paragraph::new(line).wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_status_char() {
        assert_eq!(TodoStatus::Completed.char(), "✓");
        assert_eq!(TodoStatus::InProgress.char(), "•");
        assert_eq!(TodoStatus::Pending.char(), " ");
    }

    #[test]
    fn test_todo_status_color() {
        assert_eq!(TodoStatus::Completed.color(), Color::Green);
        assert_eq!(TodoStatus::InProgress.color(), Color::Yellow);
        assert_eq!(TodoStatus::Pending.color(), Color::DarkGray);
    }

    #[test]
    fn test_todo_item_creation() {
        let item = TodoItem::new(TodoStatus::InProgress, "Test task");
        assert_eq!(item.status, TodoStatus::InProgress);
        assert_eq!(item.content, "Test task");
    }

    #[test]
    fn test_todo_item_muted_color() {
        let item = TodoItem::new(TodoStatus::Pending, "Test").muted_color(Color::Gray);
        assert_eq!(item.muted_color, Color::Gray);
    }
}
