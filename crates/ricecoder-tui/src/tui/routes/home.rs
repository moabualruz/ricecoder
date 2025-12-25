//! Home route view for RiceCoder TUI
//!
//! This module provides the main home screen view, displaying:
//! - Logo/banner
//! - Prompt input
//! - MCP server status
//! - Tips for returning users
//! - Directory and version info

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// MCP server status information
#[derive(Debug, Clone, Default)]
pub struct McpStatus {
    /// Total number of MCP servers
    pub total: usize,
    /// Number of connected servers
    pub connected: usize,
    /// Whether any server has errors
    pub has_errors: bool,
}

impl McpStatus {
    /// Check if any MCP servers are configured
    pub fn has_servers(&self) -> bool {
        self.total > 0
    }

    /// Get the connected server count display text
    pub fn display_text(&self) -> String {
        if self.total == 0 {
            String::new()
        } else if self.connected == 1 {
            "1 mcp server".to_string()
        } else {
            format!("{} mcp servers", self.connected)
        }
    }
}

/// Home view state
#[derive(Debug, Clone)]
pub struct HomeState {
    /// Current working directory
    pub directory: String,
    /// Application version
    pub version: String,
    /// MCP server status
    pub mcp_status: McpStatus,
    /// Whether user is first-time (no sessions)
    pub is_first_time_user: bool,
    /// Whether tips are hidden
    pub tips_hidden: bool,
    /// Current tip text (if showing)
    pub current_tip: Option<String>,
    /// Initial prompt text (from args or route)
    pub initial_prompt: Option<String>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            directory: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "~".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            mcp_status: McpStatus::default(),
            is_first_time_user: true,
            tips_hidden: false,
            current_tip: None,
            initial_prompt: None,
        }
    }
}

impl HomeState {
    /// Create new home state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the working directory
    pub fn with_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = dir.into();
        self
    }

    /// Set MCP status
    pub fn with_mcp_status(mut self, status: McpStatus) -> Self {
        self.mcp_status = status;
        self
    }

    /// Set first-time user status
    pub fn with_first_time_user(mut self, is_first: bool) -> Self {
        self.is_first_time_user = is_first;
        self
    }

    /// Set tips visibility
    pub fn with_tips_hidden(mut self, hidden: bool) -> Self {
        self.tips_hidden = hidden;
        self
    }

    /// Set current tip
    pub fn with_tip(mut self, tip: impl Into<String>) -> Self {
        self.current_tip = Some(tip.into());
        self
    }

    /// Check if tips should be shown
    pub fn should_show_tips(&self) -> bool {
        !self.is_first_time_user && !self.tips_hidden
    }
}

/// Theme colors for home view
#[derive(Debug, Clone)]
pub struct HomeTheme {
    pub text: Color,
    pub text_muted: Color,
    pub success: Color,
    pub error: Color,
    pub accent: Color,
    pub background: Color,
}

impl Default for HomeTheme {
    fn default() -> Self {
        Self {
            text: Color::White,
            text_muted: Color::Gray,
            success: Color::Green,
            error: Color::Red,
            accent: Color::Cyan,
            background: Color::Reset,
        }
    }
}

/// Home view widget
pub struct HomeView<'a> {
    state: &'a HomeState,
    theme: HomeTheme,
    /// ASCII logo lines (compact version)
    logo_lines: Vec<&'static str>,
}

impl<'a> HomeView<'a> {
    /// Create new home view
    pub fn new(state: &'a HomeState) -> Self {
        Self {
            state,
            theme: HomeTheme::default(),
            logo_lines: Self::default_logo(),
        }
    }

    /// Set custom theme
    pub fn with_theme(mut self, theme: HomeTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set custom logo lines
    pub fn with_logo(mut self, lines: Vec<&'static str>) -> Self {
        self.logo_lines = lines;
        self
    }

    /// Default compact logo (r[)
    fn default_logo() -> Vec<&'static str> {
        vec![
            "                        ▓▓▓▓▓▓▓▓▓▓▓▓▓▓",
            "                        ▓▓▓▓▓▓▓▓▓▓▓▓▓▓",
            "                        ▓▓▓▓▓▓        ",
            "                        ▓▓▓▓▓▓        ",
            "                        ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓▓▓▓▓▓             ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓▓▓▓               ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓▓                 ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓        ",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓",
            "▓▓▓▓▓▓                  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓",
        ]
    }

    /// Render the logo
    fn render_logo(&self, area: Rect, buf: &mut Buffer) {
        let logo_height = self.logo_lines.len() as u16;
        
        // Center the logo vertically in the area
        let start_y = if area.height > logo_height {
            area.y + (area.height - logo_height) / 2
        } else {
            area.y
        };

        for (i, line) in self.logo_lines.iter().enumerate() {
            if start_y + i as u16 >= area.y + area.height {
                break;
            }

            // Center horizontally
            let line_width = line.chars().count() as u16;
            let start_x = if area.width > line_width {
                area.x + (area.width - line_width) / 2
            } else {
                area.x
            };

            let y = start_y + i as u16;
            for (j, ch) in line.chars().enumerate() {
                let x = start_x + j as u16;
                if x < area.x + area.width {
                    buf[(x, y)]
                        .set_char(ch)
                        .set_style(Style::default().fg(self.theme.accent));
                }
            }
        }
    }

    /// Render MCP status hint
    fn render_mcp_hint(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.mcp_status.has_servers() {
            return;
        }

        let (indicator, indicator_color) = if self.state.mcp_status.has_errors {
            ("•", self.theme.error)
        } else {
            ("•", self.theme.success)
        };

        let text = if self.state.mcp_status.has_errors {
            format!("{} mcp errors ctrl+x s", indicator)
        } else {
            format!("{} {}", indicator, self.state.mcp_status.display_text())
        };

        let spans = vec![
            Span::styled(indicator, Style::default().fg(indicator_color)),
            Span::styled(
                text.trim_start_matches(indicator),
                Style::default().fg(self.theme.text),
            ),
        ];

        let line = Line::from(spans);
        let para = Paragraph::new(line).alignment(Alignment::Center);
        para.render(area, buf);
    }

    /// Render the footer with directory, MCP status, and version
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        // Split footer into left (directory), center (MCP), right (version)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(area);

        // Directory (left)
        let dir_text = Paragraph::new(self.state.directory.as_str())
            .style(Style::default().fg(self.theme.text_muted));
        dir_text.render(chunks[0], buf);

        // MCP status (center)
        if self.state.mcp_status.has_servers() {
            let (indicator, color) = if self.state.mcp_status.has_errors {
                ("⊙ ", self.theme.error)
            } else {
                ("⊙ ", self.theme.success)
            };

            let mcp_spans = vec![
                Span::styled(indicator, Style::default().fg(color)),
                Span::styled(
                    format!("{} MCP", self.state.mcp_status.connected),
                    Style::default().fg(self.theme.text),
                ),
                Span::styled(" /status", Style::default().fg(self.theme.text_muted)),
            ];

            let mcp_line = Line::from(mcp_spans);
            let mcp_para = Paragraph::new(mcp_line).alignment(Alignment::Center);
            mcp_para.render(chunks[1], buf);
        }

        // Version (right)
        let version_text = Paragraph::new(format!("v{}", self.state.version))
            .style(Style::default().fg(self.theme.text_muted))
            .alignment(Alignment::Right);
        version_text.render(chunks[2], buf);
    }

    /// Render tips section
    fn render_tips(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.should_show_tips() {
            return;
        }

        if let Some(tip) = &self.state.current_tip {
            let tip_block = Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(self.theme.text_muted));

            let tip_para = Paragraph::new(tip.as_str())
                .block(tip_block)
                .style(Style::default().fg(self.theme.text_muted))
                .alignment(Alignment::Center);

            tip_para.render(area, buf);
        }
    }
}

impl Widget for HomeView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Main layout: logo area (center), tips (if showing), footer
        let has_tips = self.state.should_show_tips() && self.state.current_tip.is_some();
        
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_tips {
                vec![
                    Constraint::Min(20),        // Logo + prompt area
                    Constraint::Length(3),      // Tips
                    Constraint::Length(1),      // Footer
                ]
            } else {
                vec![
                    Constraint::Min(20),        // Logo + prompt area
                    Constraint::Length(1),      // Footer
                ]
            })
            .margin(1)
            .split(area);

        // Render logo in center area
        let logo_area = main_chunks[0];
        self.render_logo(logo_area, buf);

        // Render MCP hint below logo if connected
        if self.state.mcp_status.has_servers() {
            let hint_area = Rect {
                x: logo_area.x,
                y: logo_area.y + logo_area.height.saturating_sub(2),
                width: logo_area.width,
                height: 1,
            };
            self.render_mcp_hint(hint_area, buf);
        }

        // Render tips if showing
        if has_tips {
            self.render_tips(main_chunks[1], buf);
        }

        // Render footer
        let footer_idx = if has_tips { 2 } else { 1 };
        if footer_idx < main_chunks.len() {
            self.render_footer(main_chunks[footer_idx], buf);
        }
    }
}

/// Home component with state management
pub struct Home {
    state: HomeState,
    theme: HomeTheme,
}

impl Home {
    /// Create new home component
    pub fn new() -> Self {
        Self {
            state: HomeState::default(),
            theme: HomeTheme::default(),
        }
    }

    /// Create with custom state
    pub fn with_state(state: HomeState) -> Self {
        Self {
            state,
            theme: HomeTheme::default(),
        }
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: HomeTheme) {
        self.theme = theme;
    }

    /// Update MCP status
    pub fn update_mcp_status(&mut self, status: McpStatus) {
        self.state.mcp_status = status;
    }

    /// Update directory
    pub fn update_directory(&mut self, dir: String) {
        self.state.directory = dir;
    }

    /// Toggle tips visibility
    pub fn toggle_tips(&mut self) {
        self.state.tips_hidden = !self.state.tips_hidden;
    }

    /// Set current tip
    pub fn set_tip(&mut self, tip: Option<String>) {
        self.state.current_tip = tip;
    }

    /// Set first-time user status
    pub fn set_first_time_user(&mut self, is_first: bool) {
        self.state.is_first_time_user = is_first;
    }

    /// Get state reference
    pub fn state(&self) -> &HomeState {
        &self.state
    }

    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut HomeState {
        &mut self.state
    }

    /// Create widget for rendering
    pub fn widget(&self) -> HomeView<'_> {
        HomeView::new(&self.state).with_theme(self.theme.clone())
    }
}

impl Default for Home {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_state_default() {
        let state = HomeState::default();
        assert!(state.is_first_time_user);
        assert!(!state.tips_hidden);
        assert!(state.current_tip.is_none());
    }

    #[test]
    fn test_mcp_status_display() {
        let status = McpStatus {
            total: 3,
            connected: 2,
            has_errors: false,
        };
        assert_eq!(status.display_text(), "2 mcp servers");

        let single = McpStatus {
            total: 1,
            connected: 1,
            has_errors: false,
        };
        assert_eq!(single.display_text(), "1 mcp server");
    }

    #[test]
    fn test_should_show_tips() {
        let mut state = HomeState::default();
        
        // First time user - no tips
        assert!(!state.should_show_tips());

        // Returning user - show tips
        state.is_first_time_user = false;
        assert!(state.should_show_tips());

        // Tips hidden
        state.tips_hidden = true;
        assert!(!state.should_show_tips());
    }

    #[test]
    fn test_home_toggle_tips() {
        let mut home = Home::new();
        assert!(!home.state().tips_hidden);
        
        home.toggle_tips();
        assert!(home.state().tips_hidden);
        
        home.toggle_tips();
        assert!(!home.state().tips_hidden);
    }
}
