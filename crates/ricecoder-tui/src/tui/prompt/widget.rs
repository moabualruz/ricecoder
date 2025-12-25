//! Main Prompt widget for ratatui
//!
//! The central prompt component that combines:
//! - Text input with tui-textarea
//! - Extmark rendering (inline annotations)
//! - Mode indicator (normal/shell)
//! - Status bar with agent/model info
//! - Spinner for active sessions
//!
//! # DDD Layer: Presentation
//! The main UI widget for prompt input.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use super::state::{PromptMode, PromptState, PLACEHOLDERS};

/// Border style for the prompt
#[derive(Debug, Clone, Copy, Default)]
pub enum PromptBorderStyle {
    /// Standard border
    #[default]
    Normal,
    /// Shell mode border (highlighted)
    Shell,
    /// Leader key active
    Leader,
    /// Focused state
    Focused,
}

impl PromptBorderStyle {
    /// Get border color for style
    pub fn color(&self, agent_color: Color, primary: Color) -> Color {
        match self {
            Self::Normal => agent_color,
            Self::Shell => primary,
            Self::Leader => Color::Gray,
            Self::Focused => agent_color,
        }
    }
}

/// Configuration for the prompt widget
#[derive(Debug, Clone)]
pub struct PromptWidgetConfig {
    /// Agent name
    pub agent_name: String,
    /// Agent color
    pub agent_color: Color,
    /// Model name
    pub model_name: String,
    /// Provider name
    pub provider_name: String,
    /// Primary color
    pub primary_color: Color,
    /// Background color
    pub background_color: Color,
    /// Text color
    pub text_color: Color,
    /// Muted text color
    pub text_muted_color: Color,
    /// Error color
    pub error_color: Color,
    /// Whether leader key is active
    pub leader_active: bool,
    /// Session status
    pub session_status: SessionStatus,
    /// Placeholder index
    pub placeholder_index: usize,
}

impl Default for PromptWidgetConfig {
    fn default() -> Self {
        Self {
            agent_name: "build".to_string(),
            agent_color: Color::Cyan,
            model_name: "claude-3.5-sonnet".to_string(),
            provider_name: "anthropic".to_string(),
            primary_color: Color::Blue,
            background_color: Color::Rgb(30, 30, 46),
            text_color: Color::White,
            text_muted_color: Color::Gray,
            error_color: Color::Red,
            leader_active: false,
            session_status: SessionStatus::Idle,
            placeholder_index: 0,
        }
    }
}

/// Session status for the prompt
#[derive(Debug, Clone, Default)]
pub enum SessionStatus {
    /// No active session
    #[default]
    Idle,
    /// Session is running
    Running,
    /// Session is retrying with error
    Retry {
        message: String,
        attempt: u32,
        next_retry_secs: u32,
    },
}

impl SessionStatus {
    /// Check if idle
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

/// The prompt widget
pub struct PromptWidget<'a> {
    /// Prompt state
    state: &'a PromptState,
    /// Configuration
    config: PromptWidgetConfig,
}

impl<'a> PromptWidget<'a> {
    /// Create a new prompt widget
    pub fn new(state: &'a PromptState, config: PromptWidgetConfig) -> Self {
        Self { state, config }
    }

    /// Get border style based on state
    fn border_style(&self) -> PromptBorderStyle {
        if self.config.leader_active {
            PromptBorderStyle::Leader
        } else if self.state.mode == PromptMode::Shell {
            PromptBorderStyle::Shell
        } else if self.state.focused {
            PromptBorderStyle::Focused
        } else {
            PromptBorderStyle::Normal
        }
    }

    /// Get highlight color
    fn highlight_color(&self) -> Color {
        self.border_style()
            .color(self.config.agent_color, self.config.primary_color)
    }

    /// Get placeholder text
    fn placeholder(&self) -> &'static str {
        PLACEHOLDERS
            .get(self.config.placeholder_index)
            .unwrap_or(&PLACEHOLDERS[0])
    }

    /// Render the mode indicator
    fn render_mode_indicator(&self) -> Line<'a> {
        let mode_text = match self.state.mode {
            PromptMode::Normal => capitalize(&self.config.agent_name),
            PromptMode::Shell => "Shell".to_string(),
        };

        Line::from(vec![
            Span::styled(
                format!("{} ", mode_text),
                Style::default().fg(self.highlight_color()),
            ),
        ])
    }

    /// Render the model info
    fn render_model_info(&self) -> Line<'a> {
        if self.state.mode == PromptMode::Shell {
            return Line::default();
        }

        Line::from(vec![
            Span::styled(
                self.config.model_name.clone(),
                Style::default().fg(if self.config.leader_active {
                    self.config.text_muted_color
                } else {
                    self.config.text_color
                }),
            ),
            Span::raw(" "),
            Span::styled(
                self.config.provider_name.clone(),
                Style::default().fg(self.config.text_muted_color),
            ),
        ])
    }

    /// Render status bar
    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        match &self.config.session_status {
            SessionStatus::Idle => {
                self.render_idle_status(area, buf);
            }
            SessionStatus::Running => {
                self.render_running_status(area, buf);
            }
            SessionStatus::Retry {
                message,
                attempt,
                next_retry_secs,
            } => {
                self.render_retry_status(area, buf, message, *attempt, *next_retry_secs);
            }
        }
    }

    /// Render idle status hints
    fn render_idle_status(&self, area: Rect, buf: &mut Buffer) {
        let hints = match self.state.mode {
            PromptMode::Normal => vec![
                ("Tab", "switch agent"),
                ("Ctrl+K", "commands"),
            ],
            PromptMode::Shell => vec![
                ("Esc", "exit shell mode"),
            ],
        };

        let hint_spans: Vec<Span> = hints
            .into_iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(key, Style::default().fg(self.config.text_color)),
                    Span::raw(" "),
                    Span::styled(desc, Style::default().fg(self.config.text_muted_color)),
                    Span::raw("  "),
                ]
            })
            .collect();

        let line = Line::from(hint_spans);
        let para = Paragraph::new(line);
        para.render(area, buf);
    }

    /// Render running status with interrupt hint
    fn render_running_status(&self, area: Rect, buf: &mut Buffer) {
        let interrupt_style = if self.state.interrupt_count > 0 {
            Style::default().fg(self.config.primary_color)
        } else {
            Style::default().fg(self.config.text_color)
        };

        let hint_text = if self.state.interrupt_count > 0 {
            "again to interrupt"
        } else {
            "interrupt"
        };

        let line = Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green)),
            Span::styled("Running ", Style::default().fg(self.config.text_muted_color)),
            Span::raw("  "),
            Span::styled("esc ", interrupt_style),
            Span::styled(hint_text, Style::default().fg(self.config.text_muted_color)),
        ]);

        let para = Paragraph::new(line);
        para.render(area, buf);
    }

    /// Render retry status with error message
    fn render_retry_status(&self, area: Rect, buf: &mut Buffer, message: &str, attempt: u32, secs: u32) {
        let truncated = if message.len() > 80 {
            format!("{}...", &message[..77])
        } else {
            message.to_string()
        };

        let retry_text = if secs > 0 {
            format!("[retrying in {}s attempt #{}]", secs, attempt)
        } else {
            format!("[retrying attempt #{}]", attempt)
        };

        let line = Line::from(vec![
            Span::styled("● ", Style::default().fg(self.config.error_color)),
            Span::styled(truncated, Style::default().fg(self.config.error_color)),
            Span::raw(" "),
            Span::styled(retry_text, Style::default().fg(self.config.error_color)),
        ]);

        let para = Paragraph::new(line).wrap(Wrap { trim: true });
        para.render(area, buf);
    }
}

impl<'a> Widget for PromptWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout: input area + status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),     // Input area
                Constraint::Length(1),  // Mode/model line
                Constraint::Length(1),  // Status bar
            ])
            .split(area);

        let input_area = chunks[0];
        let info_area = chunks[1];
        let status_area = chunks[2];

        // Render input area with left border
        let border_color = self.highlight_color();
        let input_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(self.config.background_color));

        let inner = input_block.inner(input_area);
        input_block.render(input_area, buf);

        // Render input text or placeholder
        let input_text = if self.state.prompt.input.is_empty() && !self.state.disabled {
            format!("Ask anything... \"{}\"", self.placeholder())
        } else {
            self.state.prompt.input.clone()
        };

        let text_style = if self.state.prompt.input.is_empty() {
            Style::default().fg(self.config.text_muted_color)
        } else if self.config.leader_active {
            Style::default().fg(self.config.text_muted_color)
        } else {
            Style::default().fg(self.config.text_color)
        };

        let para = Paragraph::new(input_text)
            .style(text_style)
            .wrap(Wrap { trim: false });
        para.render(inner, buf);

        // Render mode + model info
        let info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1),  // Border spacing
                Constraint::Min(0),     // Content
            ])
            .split(info_area);

        let mode_model_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),     // Mode
                Constraint::Min(0),     // Model
            ])
            .split(info_layout[1]);

        let mode_para = Paragraph::new(self.render_mode_indicator());
        mode_para.render(mode_model_layout[0], buf);

        let model_para = Paragraph::new(self.render_model_info());
        model_para.render(mode_model_layout[1], buf);

        // Render status bar
        let status_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(status_area);

        self.render_status_bar(status_layout[1], buf);
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_widget_config_default() {
        let config = PromptWidgetConfig::default();
        assert_eq!(config.agent_name, "build");
        assert!(!config.leader_active);
    }

    #[test]
    fn test_session_status() {
        assert!(SessionStatus::Idle.is_idle());
        assert!(!SessionStatus::Idle.is_running());
        assert!(SessionStatus::Running.is_running());
    }

    #[test]
    fn test_border_style_color() {
        let style = PromptBorderStyle::Normal;
        let color = style.color(Color::Cyan, Color::Blue);
        assert_eq!(color, Color::Cyan);

        let style = PromptBorderStyle::Shell;
        let color = style.color(Color::Cyan, Color::Blue);
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("build"), "Build");
        assert_eq!(capitalize("plan"), "Plan");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("a"), "A");
    }

    #[test]
    fn test_prompt_widget_creation() {
        let state = PromptState::new();
        let config = PromptWidgetConfig::default();
        let widget = PromptWidget::new(&state, config);
        
        assert_eq!(widget.highlight_color(), Color::Cyan);
    }

    #[test]
    fn test_prompt_widget_shell_mode() {
        let mut state = PromptState::new();
        state.mode = PromptMode::Shell;
        let config = PromptWidgetConfig::default();
        let widget = PromptWidget::new(&state, config);
        
        assert_eq!(widget.highlight_color(), Color::Blue);
    }
}
