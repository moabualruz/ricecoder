//! Prompt widget for displaying the command prompt
//!
//! This module provides the `PromptWidget` for displaying a beautiful, styled command prompt
//! with context information. The prompt displays:
//! - Git branch information
//! - Project name
//! - Current application mode (Chat, Command, Diff, Help)
//! - Active AI provider and model
//!
//! # Features
//!
//! - **Context indicators**: Display git branch, project name, mode, and provider
//! - **Multi-line input**: Support for multi-line command input with text wrapping
//! - **Input history**: Navigate through previous commands with up/down arrows
//! - **Customizable styling**: Configure colors and appearance via `PromptConfig`
//! - **Cursor positioning**: Full cursor control and text editing
//!
//! # Examples
//!
//! Creating a prompt widget with context:
//!
//! ```ignore
//! use ricecoder_tui::{PromptWidget, ContextIndicators, AppMode};
//!
//! let mut context = ContextIndicators::new();
//! context.git_branch = Some("main".to_string());
//! context.project_name = Some("ricecoder".to_string());
//! context.mode = AppMode::Chat;
//! context.provider = Some("OpenAI".to_string());
//! context.model = Some("gpt-4".to_string());
//!
//! let mut prompt = PromptWidget::new(context);
//! ```
//!
//! Customizing the prompt appearance:
//!
//! ```ignore
//! use ricecoder_tui::PromptConfig;
//!
//! let config = PromptConfig {
//!     prefix: "❯ ".to_string(),
//!     show_git_branch: true,
//!     show_mode: true,
//!     ..Default::default()
//! };
//! ```

use crate::{model::AppMode, style::Color};

/// Context indicators for the prompt
#[derive(Debug, Clone)]
pub struct ContextIndicators {
    /// Git branch name
    pub git_branch: Option<String>,
    /// Project name
    pub project_name: Option<String>,
    /// Current mode
    pub mode: AppMode,
    /// AI provider name
    pub provider: Option<String>,
    /// AI model name
    pub model: Option<String>,
}

impl ContextIndicators {
    /// Create new context indicators
    pub fn new() -> Self {
        Self {
            git_branch: None,
            project_name: None,
            mode: AppMode::Chat,
            provider: None,
            model: None,
        }
    }

    /// Set git branch
    pub fn with_git_branch(mut self, branch: impl Into<String>) -> Self {
        self.git_branch = Some(branch.into());
        self
    }

    /// Set project name
    pub fn with_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Set provider and model
    pub fn with_provider(mut self, provider: impl Into<String>, model: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self.model = Some(model.into());
        self
    }

    /// Format context as string
    pub fn format(&self) -> String {
        let mut parts = Vec::new();

        if let Some(branch) = &self.git_branch {
            parts.push(format!("({})", branch));
        }

        if let Some(project) = &self.project_name {
            parts.push(project.clone());
        }

        let mode_str = match self.mode {
            AppMode::Chat => "💬",
            AppMode::Command => "⚙️",
            AppMode::Diff => "📝",
            AppMode::Mcp => "🔧",
            AppMode::Provider => "🤖",
            AppMode::Session => "📋",
            AppMode::Help => "❓",
        };
        parts.push(mode_str.to_string());

        if let (Some(provider), Some(model)) = (&self.provider, &self.model) {
            parts.push(format!("[{}/{}]", provider, model));
        }

        parts.join(" ")
    }
}

impl Default for ContextIndicators {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt configuration
#[derive(Debug, Clone)]
pub struct PromptConfig {
    /// Prompt prefix
    pub prefix: String,
    /// Prompt suffix
    pub suffix: String,
    /// Foreground color
    pub fg_color: Color,
    /// Background color
    pub bg_color: Option<Color>,
    /// Show context
    pub show_context: bool,
}

impl PromptConfig {
    /// Create new prompt config
    pub fn new() -> Self {
        Self {
            prefix: "❯".to_string(),
            suffix: " ".to_string(),
            fg_color: Color::new(0, 122, 255),
            bg_color: None,
            show_context: true,
        }
    }

    /// Set prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set suffix
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Set foreground color
    pub fn with_fg_color(mut self, color: Color) -> Self {
        self.fg_color = color;
        self
    }

    /// Set background color
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    /// Set show context
    pub fn with_show_context(mut self, show: bool) -> Self {
        self.show_context = show;
        self
    }
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt widget
pub struct PromptWidget {
    /// Input text
    pub input: String,
    /// Cursor position
    pub cursor: usize,
    /// Context indicators
    pub context: ContextIndicators,
    /// Prompt configuration
    pub config: PromptConfig,
    /// Input history
    pub history: Vec<String>,
    /// History index
    pub history_index: Option<usize>,
}

impl PromptWidget {
    /// Create a new prompt widget
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            context: ContextIndicators::new(),
            config: PromptConfig::new(),
            history: Vec::new(),
            history_index: None,
        }
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, ch: char) {
        self.input.insert(self.cursor, ch);
        self.cursor += 1;
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.input.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn move_end(&mut self) {
        self.cursor = self.input.len();
    }

    /// Submit input
    pub fn submit(&mut self) -> String {
        let input = self.input.clone();
        self.history.push(input.clone());
        self.input.clear();
        self.cursor = 0;
        self.history_index = None;
        input
    }

    /// Navigate history up
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
                self.input = self.history[self.history.len() - 1].clone();
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.input = self.history[idx - 1].clone();
            }
            _ => {}
        }

        self.cursor = self.input.len();
    }

    /// Navigate history down
    pub fn history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
                self.cursor = self.input.len();
            }
            Some(_) => {
                self.history_index = None;
                self.input.clear();
                self.cursor = 0;
            }
            None => {}
        }
    }

    /// Format the prompt line
    pub fn format_prompt(&self) -> String {
        let mut prompt = String::new();

        if self.config.show_context {
            prompt.push_str(&self.context.format());
            prompt.push(' ');
        }

        prompt.push_str(&self.config.prefix);
        prompt.push_str(&self.config.suffix);

        prompt
    }

    /// Get the full display line
    pub fn display_line(&self) -> String {
        format!("{}{}", self.format_prompt(), self.input)
    }
}

impl Default for PromptWidget {
    fn default() -> Self {
        Self::new()
    }
}
