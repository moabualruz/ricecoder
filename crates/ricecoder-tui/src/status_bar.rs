//! Status bar widget for displaying application status information
//!
//! This module provides a status bar widget that displays essential information
//! about the current state of the RiceCoder TUI, including provider status,
//! session information, project context, and mode indicators.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};
use ricecoder_sessions::{TokenUsage, TokenLimitStatus};
use std::path::PathBuf;

/// Status bar widget for displaying application status
#[derive(Debug, Clone)]
pub struct StatusBarWidget {
    /// Current provider name
    pub provider: String,
    /// Current model name
    pub model: String,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Current session name
    pub session_name: String,
    /// Message count in current session
    pub message_count: usize,
    /// Current project name (if any)
    pub project_name: Option<String>,
    /// Current working directory
    pub working_directory: PathBuf,
    /// Git branch (if in git repository)
    pub git_branch: Option<String>,
    /// VCS status summary (e.g., "1S 2M 1U")
    pub vcs_status: Option<String>,
    /// Ahead/behind counts relative to remote
    pub vcs_ahead_behind: Option<(usize, usize)>,
    /// Token usage information
    pub token_usage: Option<TokenUsage>,
    /// Current input mode
    pub input_mode: InputMode,
    /// Recording status
    pub recording_status: Option<String>,
    /// Search status
    pub search_status: Option<String>,
    /// Selection status
    pub selection_status: Option<String>,
    /// Whether to flash the status bar
    pub flash: bool,
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
    Connecting,
}

impl ConnectionStatus {
    pub fn display_text(&self) -> &'static str {
        match self {
            ConnectionStatus::Connected => "✓",
            ConnectionStatus::Disconnected => "✗",
            ConnectionStatus::Error => "⚠",
            ConnectionStatus::Connecting => "⟳",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ConnectionStatus::Connected => Color::Green,
            ConnectionStatus::Disconnected => Color::Yellow,
            ConnectionStatus::Error => Color::Red,
            ConnectionStatus::Connecting => Color::Blue,
        }
    }
}

// Token usage information is now imported from ricecoder-sessions

/// Input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Insert,
    Normal,
    Visual,
    Command,
}

impl InputMode {
    pub fn display_text(&self) -> &'static str {
        match self {
            InputMode::Insert => "INSERT",
            InputMode::Normal => "NORMAL",
            InputMode::Visual => "VISUAL",
            InputMode::Command => "COMMAND",
        }
    }
}

impl Default for StatusBarWidget {
    fn default() -> Self {
        Self {
            provider: "None".to_string(),
            model: "None".to_string(),
            connection_status: ConnectionStatus::Disconnected,
            session_name: "Untitled".to_string(),
            message_count: 0,
            project_name: None,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            git_branch: None,
            vcs_status: None,
            vcs_ahead_behind: None,
            token_usage: None,
            input_mode: InputMode::Insert,
            recording_status: None,
            search_status: None,
            selection_status: None,
            flash: false,
        }
    }
}

impl StatusBarWidget {
    /// Create a new status bar widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Set provider information
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    /// Set model information
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set connection status
    pub fn with_connection_status(mut self, status: ConnectionStatus) -> Self {
        self.connection_status = status;
        self
    }

    /// Set session name
    pub fn with_session_name(mut self, name: impl Into<String>) -> Self {
        self.session_name = name.into();
        self
    }

    /// Set message count
    pub fn with_message_count(mut self, count: usize) -> Self {
        self.message_count = count;
        self
    }

    /// Set project name
    pub fn with_project_name(mut self, name: Option<String>) -> Self {
        self.project_name = name;
        self
    }

    /// Set working directory
    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = dir;
        self
    }

    /// Set git branch
    pub fn with_git_branch(mut self, branch: Option<String>) -> Self {
        self.git_branch = branch;
        self
    }

    /// Set VCS status summary
    pub fn with_vcs_status(mut self, status: Option<String>) -> Self {
        self.vcs_status = status;
        self
    }

    /// Set VCS ahead/behind counts
    pub fn with_vcs_ahead_behind(mut self, ahead_behind: Option<(usize, usize)>) -> Self {
        self.vcs_ahead_behind = ahead_behind;
        self
    }

    /// Set token usage
    pub fn with_token_usage(mut self, usage: Option<TokenUsage>) -> Self {
        self.token_usage = usage;
        self
    }

    /// Set input mode
    pub fn with_input_mode(mut self, mode: InputMode) -> Self {
        self.input_mode = mode;
        self
    }

    /// Set recording status
    pub fn with_recording_status(mut self, status: Option<String>) -> Self {
        self.recording_status = status;
        self
    }

    /// Set search status
    pub fn with_search_status(mut self, status: Option<String>) -> Self {
        self.search_status = status;
        self
    }

    /// Set selection status
    pub fn with_selection_status(mut self, status: Option<String>) -> Self {
        self.selection_status = status;
        self
    }

    /// Set flash state
    pub fn with_flash(mut self, flash: bool) -> Self {
        self.flash = flash;
        self
    }

    /// Get the display text for the left side (essential info)
    fn left_section(&self) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Provider and model
        spans.push(Span::styled(
            format!("{}@{}", self.provider, self.model),
            Style::default().fg(Color::Cyan),
        ));
        spans.push(Span::raw(" "));

        // Connection status
        spans.push(Span::styled(
            self.connection_status.display_text(),
            Style::default().fg(self.connection_status.color()),
        ));
        spans.push(Span::raw(" "));

        // Session name and message count
        spans.push(Span::styled(
            format!("{} ({})", self.session_name, self.message_count),
            Style::default().fg(Color::White),
        ));

        spans
    }

    /// Get the display text for the center (context info)
    fn center_section(&self) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Project name
        if let Some(project) = &self.project_name {
            spans.push(Span::styled(
                format!("[{}]", project),
                Style::default().fg(Color::Yellow),
            ));
            spans.push(Span::raw(" "));
        }

        // Working directory
        let dir_display = self.working_directory.display().to_string();
        let dir_short = if dir_display.len() > 30 {
            format!("...{}", &dir_display[dir_display.len().saturating_sub(27)..])
        } else {
            dir_display
        };
        spans.push(Span::styled(dir_short, Style::default().fg(Color::Gray)));
        spans.push(Span::raw(" "));

        // Git branch and VCS status
        if let Some(branch) = &self.git_branch {
            let mut branch_text = format!("({}", branch);

            // Add VCS status if available
            if let Some(status) = &self.vcs_status {
                branch_text.push_str(&format!(" {})", status));
            } else {
                branch_text.push(')');
            }

            let color = if self.vcs_status.is_some() {
                Color::Yellow // Yellow if there are changes
            } else {
                Color::Green // Green if clean
            };

            spans.push(Span::styled(branch_text, Style::default().fg(color)));
            spans.push(Span::raw(" "));
        }

        // VCS ahead/behind info
        if let Some((ahead, behind)) = self.vcs_ahead_behind {
            if ahead > 0 || behind > 0 {
                let ahead_text = if ahead > 0 { format!("↑{}", ahead) } else { String::new() };
                let behind_text = if behind > 0 { format!("↓{}", behind) } else { String::new() };
                spans.push(Span::styled(
                    format!("{}{}", ahead_text, behind_text),
                    Style::default().fg(Color::Cyan),
                ));
                spans.push(Span::raw(" "));
            }
        }

        // Token usage with warning indicators
        if let Some(usage) = &self.token_usage {
            let percentage = if usage.token_limit > 0 {
                (usage.total_tokens as f64 / usage.token_limit as f64) * 100.0
            } else {
                0.0
            };

            let status = if percentage >= 90.0 {
                TokenLimitStatus::Critical
            } else if percentage >= 75.0 {
                TokenLimitStatus::Warning
            } else {
                TokenLimitStatus::Normal
            };

            let usage_text = format!("{} {}/{} ({:.1}%)",
                status.symbol(),
                usage.total_tokens,
                usage.token_limit,
                percentage
            );

            let color = match status {
                TokenLimitStatus::Normal => Color::Green,
                TokenLimitStatus::Warning => Color::Yellow,
                TokenLimitStatus::Critical => Color::Red,
                TokenLimitStatus::Unknown => Color::Gray,
            };

            spans.push(Span::styled(
                format!("Tokens: {}", usage_text),
                Style::default().fg(color),
            ));
            spans.push(Span::raw(" "));
        }

        spans
    }

    /// Get the display text for the right side (mode info)
    fn right_section(&self) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Input mode
        spans.push(Span::styled(
            self.input_mode.display_text(),
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        ));

        // Recording status
        if let Some(recording) = &self.recording_status {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("REC:{}", recording),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
        }

        // Search status
        if let Some(search) = &self.search_status {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("SEARCH:{}", search),
                Style::default().fg(Color::Yellow),
            ));
        }

        // Selection status
        if let Some(selection) = &self.selection_status {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("SEL:{}", selection),
                Style::default().fg(Color::Green),
            ));
        }

        spans
    }
}

impl Widget for StatusBarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the status bar block
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(if self.flash {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            });

        // Get the inner area
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Create the status line
        let mut spans = Vec::new();

        // Left section
        spans.extend(self.left_section());
        spans.push(Span::raw(" │ "));

        // Center section
        spans.extend(self.center_section());

        // Right section (right-aligned)
        let right_spans = self.right_section();
        if !right_spans.is_empty() {
            // Calculate space needed for right section
            let right_width: usize = right_spans.iter().map(|s| s.content.len()).sum();
            let total_width = inner_area.width as usize;
            let left_width: usize = spans.iter().map(|s| s.content.len()).sum();

            if left_width + right_width + 3 < total_width {
                // Add padding to push right section to the right
                let padding = total_width - left_width - right_width - 3;
                spans.push(Span::raw(" ".repeat(padding)));
                spans.push(Span::raw("│ "));
                spans.extend(right_spans);
            } else {
                // Not enough space, just add right section after separator
                spans.push(Span::raw("│ "));
                spans.extend(right_spans);
            }
        }

        // Create the line and render it
        let line = Line::from(spans);
        line.render(inner_area, buf);
    }
}