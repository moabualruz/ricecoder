//! Session header component
//!
//! Displays session title, model info, and status indicators.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Session header state
#[derive(Debug, Clone, Default)]
pub struct SessionHeader {
    /// Session title
    pub title: String,
    /// Model name (e.g., "claude-3-sonnet")
    pub model: String,
    /// Provider name (e.g., "anthropic")
    pub provider: String,
    /// Token usage
    pub tokens_used: u64,
    /// Token limit
    pub token_limit: u64,
    /// Estimated cost in USD
    pub estimated_cost: f64,
    /// Is session currently processing
    pub is_processing: bool,
    /// Session ID
    pub session_id: String,
}

impl SessionHeader {
    /// Create a new session header
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            title: "New Session".to_string(),
            ..Default::default()
        }
    }

    /// Set the session title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the model info
    pub fn with_model(mut self, provider: impl Into<String>, model: impl Into<String>) -> Self {
        self.provider = provider.into();
        self.model = model.into();
        self
    }

    /// Set token usage
    pub fn with_tokens(mut self, used: u64, limit: u64) -> Self {
        self.tokens_used = used;
        self.token_limit = limit;
        self
    }

    /// Set estimated cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost = cost;
        self
    }

    /// Set processing state
    pub fn with_processing(mut self, processing: bool) -> Self {
        self.is_processing = processing;
        self
    }

    /// Render the header
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &SessionHeaderTheme) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build header line
        let mut spans = vec![
            Span::styled(&self.title, Style::default().fg(theme.title).bold()),
        ];

        // Add model info
        if !self.model.is_empty() {
            spans.push(Span::styled(" · ", Style::default().fg(theme.separator)));
            spans.push(Span::styled(&self.model, Style::default().fg(theme.model)));
        }

        // Add provider
        if !self.provider.is_empty() {
            spans.push(Span::styled(" (", Style::default().fg(theme.separator)));
            spans.push(Span::styled(&self.provider, Style::default().fg(theme.provider)));
            spans.push(Span::styled(")", Style::default().fg(theme.separator)));
        }

        // Add token usage
        if self.token_limit > 0 {
            spans.push(Span::styled(" · ", Style::default().fg(theme.separator)));
            let usage_pct = (self.tokens_used as f64 / self.token_limit as f64 * 100.0) as u8;
            let usage_color = if usage_pct > 90 {
                theme.usage_critical
            } else if usage_pct > 70 {
                theme.usage_warning
            } else {
                theme.usage_normal
            };
            spans.push(Span::styled(
                format!("{}/{} tokens", self.tokens_used, self.token_limit),
                Style::default().fg(usage_color),
            ));
        }

        // Add cost
        if self.estimated_cost > 0.0 {
            spans.push(Span::styled(" · ", Style::default().fg(theme.separator)));
            spans.push(Span::styled(
                format!("${:.4}", self.estimated_cost),
                Style::default().fg(theme.cost),
            ));
        }

        // Add processing indicator
        if self.is_processing {
            spans.push(Span::styled(" ", Style::default()));
            spans.push(Span::styled(
                "●",
                Style::default().fg(theme.processing),
            ));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, inner);
    }
}

/// Theme colors for session header
#[derive(Debug, Clone)]
pub struct SessionHeaderTheme {
    pub title: Color,
    pub model: Color,
    pub provider: Color,
    pub separator: Color,
    pub border: Color,
    pub usage_normal: Color,
    pub usage_warning: Color,
    pub usage_critical: Color,
    pub cost: Color,
    pub processing: Color,
}

impl Default for SessionHeaderTheme {
    fn default() -> Self {
        Self {
            title: Color::White,
            model: Color::Cyan,
            provider: Color::Gray,
            separator: Color::DarkGray,
            border: Color::DarkGray,
            usage_normal: Color::Green,
            usage_warning: Color::Yellow,
            usage_critical: Color::Red,
            cost: Color::Yellow,
            processing: Color::Cyan,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = SessionHeader::new("session-123")
            .with_title("My Session")
            .with_model("anthropic", "claude-3-sonnet")
            .with_tokens(1000, 100000)
            .with_cost(0.0025);

        assert_eq!(header.title, "My Session");
        assert_eq!(header.model, "claude-3-sonnet");
        assert_eq!(header.provider, "anthropic");
        assert_eq!(header.tokens_used, 1000);
        assert_eq!(header.token_limit, 100000);
        assert!((header.estimated_cost - 0.0025).abs() < 0.0001);
    }

    #[test]
    fn test_processing_state() {
        let header = SessionHeader::new("test").with_processing(true);
        assert!(header.is_processing);
    }
}
