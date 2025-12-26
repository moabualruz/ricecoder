//! Session footer component
//!
//! Displays keybind hints, mode indicator, and status messages.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Session footer state
#[derive(Debug, Clone, Default)]
pub struct SessionFooter {
    /// Current mode indicator
    pub mode: String,
    /// Status message (temporary)
    pub status_message: Option<String>,
    /// Keybind hints to display
    pub hints: Vec<KeybindHint>,
    /// Working directory
    pub working_dir: String,
}

/// A keybind hint to display in footer
#[derive(Debug, Clone)]
pub struct KeybindHint {
    /// Key combination (e.g., "Ctrl+C")
    pub key: String,
    /// Action description (e.g., "cancel")
    pub action: String,
}

impl KeybindHint {
    pub fn new(key: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
        }
    }
}

impl SessionFooter {
    /// Create a new session footer
    pub fn new() -> Self {
        Self {
            mode: "chat".to_string(),
            hints: vec![
                KeybindHint::new("?", "help"),
                KeybindHint::new("Ctrl+P", "command"),
                KeybindHint::new("Ctrl+C", "cancel"),
                KeybindHint::new("Esc", "back"),
            ],
            ..Default::default()
        }
    }

    /// Set the current mode
    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = mode.into();
        self
    }

    /// Set a status message
    pub fn with_status(mut self, message: impl Into<String>) -> Self {
        self.status_message = Some(message.into());
        self
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Set keybind hints
    pub fn with_hints(mut self, hints: Vec<KeybindHint>) -> Self {
        self.hints = hints;
        self
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = dir.into();
        self
    }

    /// Render the footer
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &SessionFooterTheme) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build footer line
        let mut spans = Vec::new();

        // Mode indicator
        spans.push(Span::styled(
            format!(" {} ", self.mode.to_uppercase()),
            Style::default()
                .fg(theme.mode_fg)
                .bg(theme.mode_bg)
                .bold(),
        ));
        spans.push(Span::styled(" ", Style::default()));

        // Working directory (truncated)
        if !self.working_dir.is_empty() {
            let display_dir = truncate_path(&self.working_dir, 30);
            spans.push(Span::styled(display_dir, Style::default().fg(theme.working_dir)));
            spans.push(Span::styled(" · ", Style::default().fg(theme.separator)));
        }

        // Status message OR keybind hints
        if let Some(ref msg) = self.status_message {
            spans.push(Span::styled(msg, Style::default().fg(theme.status)));
        } else {
            // Keybind hints
            for (i, hint) in self.hints.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled("  ", Style::default()));
                }
                spans.push(Span::styled(&hint.key, Style::default().fg(theme.key).bold()));
                spans.push(Span::styled(" ", Style::default()));
                spans.push(Span::styled(&hint.action, Style::default().fg(theme.action)));
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, inner);
    }
}

/// Truncate a path to fit within max_len characters
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    // Replace home with ~
    let home = dirs::home_dir()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut display = if !home.is_empty() && path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    };

    if display.len() > max_len {
        display = format!("...{}", &display[display.len() - max_len + 3..]);
    }

    display
}

/// Theme colors for session footer
#[derive(Debug, Clone)]
pub struct SessionFooterTheme {
    pub border: Color,
    pub mode_fg: Color,
    pub mode_bg: Color,
    pub working_dir: Color,
    pub separator: Color,
    pub key: Color,
    pub action: Color,
    pub status: Color,
}

impl Default for SessionFooterTheme {
    fn default() -> Self {
        Self {
            border: Color::DarkGray,
            mode_fg: Color::Black,
            mode_bg: Color::Cyan,
            working_dir: Color::Gray,
            separator: Color::DarkGray,
            key: Color::Yellow,
            action: Color::Gray,
            status: Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_creation() {
        let footer = SessionFooter::new()
            .with_mode("shell")
            .with_working_dir("/home/user/project");

        assert_eq!(footer.mode, "shell");
        assert_eq!(footer.working_dir, "/home/user/project");
    }

    #[test]
    fn test_status_message() {
        let mut footer = SessionFooter::new().with_status("Processing...");
        assert!(footer.status_message.is_some());

        footer.clear_status();
        assert!(footer.status_message.is_none());
    }

    #[test]
    fn test_truncate_path() {
        let short = "/home/user";
        assert_eq!(truncate_path(short, 20), short);

        let long = "/home/user/very/long/path/to/project";
        let truncated = truncate_path(long, 20);
        assert!(truncated.len() <= 20);
        assert!(truncated.starts_with("..."));
    }

    #[test]
    fn test_keybind_hint() {
        let hint = KeybindHint::new("Ctrl+C", "cancel");
        assert_eq!(hint.key, "Ctrl+C");
        assert_eq!(hint.action, "cancel");
    }
}
