//! Core theme types and data structures

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A complete theme definition
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Primary colors
    pub primary: ratatui::style::Color,
    /// Secondary colors
    pub secondary: ratatui::style::Color,
    /// Background colors
    pub background: ratatui::style::Color,
    /// Text colors
    pub foreground: ratatui::style::Color,
    /// Accent colors
    pub accent: ratatui::style::Color,
    /// Error colors
    pub error: ratatui::style::Color,
    /// Warning colors
    pub warning: ratatui::style::Color,
    /// Success colors
    pub success: ratatui::style::Color,
    /// Info colors (OpenCode parity)
    pub info: ratatui::style::Color,
    /// Syntax highlighting theme
    pub syntax: SyntaxTheme,

    // === OpenCode-style UI Colors ===
    /// Muted text color (placeholders, hints, disabled)
    pub text_muted: ratatui::style::Color,
    /// Selected list item text color (OpenCode parity)
    pub selected_list_item_text: ratatui::style::Color,
    /// Panel background (slightly raised surfaces)
    pub background_panel: ratatui::style::Color,
    /// Element background (buttons, inputs)
    pub background_element: ratatui::style::Color,
    /// Menu background (dropdowns, popups) (OpenCode parity)
    pub background_menu: ratatui::style::Color,
    /// Default border color
    pub border: ratatui::style::Color,
    /// Active/focused border color
    pub border_active: ratatui::style::Color,
    /// Subtle border color (OpenCode parity)
    pub border_subtle: ratatui::style::Color,
    /// Agent colors by name
    pub agent_colors: AgentColors,
    /// Diff colors for code diffs (OpenCode parity)
    pub diff: DiffColors,
}

impl Theme {
    /// Validate the theme data
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }
        // Add more validation as needed
        Ok(())
    }

    /// Fallback theme used when no JSON themes can be loaded.
    /// This is a minimal dark theme that ensures the app always works.
    pub fn fallback() -> Self {
        use ratatui::style::Color;
        Self {
            name: "fallback".to_string(),
            primary: Color::Rgb(255, 255, 255),
            secondary: Color::Rgb(204, 204, 204),
            background: Color::Rgb(0, 0, 0),
            foreground: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(255, 255, 0),
            error: Color::Rgb(255, 0, 0),
            warning: Color::Rgb(255, 255, 0),
            success: Color::Rgb(0, 255, 0),
            info: Color::Rgb(100, 180, 255),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(255, 102, 0),
                string: Color::Rgb(0, 255, 0),
                number: Color::Rgb(255, 255, 0),
                comment: Color::Rgb(136, 136, 136),
                function: Color::Rgb(255, 0, 255),
                variable: Color::Rgb(255, 255, 255),
                r#type: Color::Rgb(0, 255, 255),
                constant: Color::Rgb(255, 102, 0),
            },
            text_muted: Color::DarkGray,
            selected_list_item_text: Color::Rgb(255, 255, 255),
            background_panel: Color::Rgb(30, 30, 30),
            background_element: Color::Rgb(45, 45, 45),
            background_menu: Color::Rgb(35, 35, 35),
            border: Color::Rgb(60, 60, 60),
            border_active: Color::Cyan,
            border_subtle: Color::Rgb(40, 40, 40),
            agent_colors: AgentColors::default(),
            diff: DiffColors::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::fallback()
    }
}

/// Theme manager trait for loading and managing themes
pub trait ThemeManager {
    /// Load a theme by name
    fn load_theme(&mut self, name: &str) -> Result<(), ThemeError>;
    /// Get a theme by name
    fn get_theme(&self, name: &str) -> Option<Theme>;
    /// List all available themes
    fn list_themes(&self) -> Vec<String>;
}

/// Theme error type
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("Invalid theme format: {0}")]
    InvalidFormat(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeMetadata {
    /// Theme name
    pub name: String,
    /// Theme author
    pub author: String,
    /// Theme description
    pub description: String,
    /// Theme version
    pub version: String,
    /// Compatible RiceCoder versions
    pub ricecoder_version: String,
}

/// Color palette for the theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    /// Primary colors
    pub primary: ColorDefinition,
    /// Secondary colors
    pub secondary: ColorDefinition,
    /// Background colors
    pub background: ColorDefinition,
    /// Text colors
    pub text: ColorDefinition,
    /// Accent colors
    pub accent: ColorDefinition,
    /// Error colors
    pub error: ColorDefinition,
    /// Warning colors
    pub warning: ColorDefinition,
    /// Success colors
    pub success: ColorDefinition,
    /// Info colors
    pub info: ColorDefinition,
    /// Muted colors
    pub muted: ColorDefinition,
    /// Border colors
    pub border: ColorDefinition,
}

/// Agent-specific colors for the TUI
#[derive(Debug, Clone, PartialEq)]
pub struct AgentColors {
    /// Build agent color (default: cyan)
    pub build: ratatui::style::Color,
    /// Plan agent color (default: orange)
    pub plan: ratatui::style::Color,
    /// General agent color (default: green)
    pub general: ratatui::style::Color,
    /// Explore agent color (default: blue)
    pub explore: ratatui::style::Color,
}

/// Diff colors for code diff rendering (OpenCode parity)
#[derive(Debug, Clone, PartialEq)]
pub struct DiffColors {
    /// Added line background
    pub added: ratatui::style::Color,
    /// Removed line background
    pub removed: ratatui::style::Color,
    /// Context line background (unchanged)
    pub context: ratatui::style::Color,
    /// Hunk header color (@@ -1,2 +1,3 @@)
    pub hunk_header: ratatui::style::Color,
    /// Highlight for added text within a line
    pub highlight_added: ratatui::style::Color,
    /// Highlight for removed text within a line
    pub highlight_removed: ratatui::style::Color,
    /// Line number for added lines
    pub line_number_added: ratatui::style::Color,
    /// Line number for removed lines
    pub line_number_removed: ratatui::style::Color,
}

impl Default for DiffColors {
    fn default() -> Self {
        use ratatui::style::Color;
        Self {
            added: Color::Rgb(30, 60, 30),           // Dark green bg
            removed: Color::Rgb(60, 30, 30),         // Dark red bg
            context: Color::Reset,                    // No highlight
            hunk_header: Color::Rgb(100, 100, 180),  // Muted blue
            highlight_added: Color::Rgb(50, 120, 50), // Brighter green
            highlight_removed: Color::Rgb(120, 50, 50), // Brighter red
            line_number_added: Color::Rgb(80, 180, 80), // Green text
            line_number_removed: Color::Rgb(180, 80, 80), // Red text
        }
    }
}

impl Default for AgentColors {
    fn default() -> Self {
        use ratatui::style::Color;
        Self {
            build: Color::Cyan,
            plan: Color::Rgb(255, 200, 100),
            general: Color::Green,
            explore: Color::Blue,
        }
    }
}

impl AgentColors {
    /// Get color for an agent by name
    pub fn get(&self, name: &str) -> ratatui::style::Color {
        match name.to_lowercase().as_str() {
            "build" => self.build,
            "plan" => self.plan,
            "general" => self.general,
            "explore" => self.explore,
            _ => self.build, // Default to build color
        }
    }
}

/// Syntax highlighting theme
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTheme {
    /// Keywords
    pub keyword: ratatui::style::Color,
    /// Strings
    pub string: ratatui::style::Color,
    /// Numbers
    pub number: ratatui::style::Color,
    /// Comments
    pub comment: ratatui::style::Color,
    /// Functions
    pub function: ratatui::style::Color,
    /// Variables
    pub variable: ratatui::style::Color,
    /// Types
    pub r#type: ratatui::style::Color,
    /// Constants
    pub constant: ratatui::style::Color,
}

/// Color definition with foreground, background, and modifiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorDefinition {
    /// Foreground color (hex or named)
    pub fg: Option<String>,
    /// Background color (hex or named)
    pub bg: Option<String>,
    /// Text modifiers
    pub modifiers: Vec<String>,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    /// Current theme name
    pub current_theme: String,
    /// Custom theme overrides
    pub overrides: BTreeMap<String, serde_json::Value>,
    /// Theme settings
    pub settings: ThemeSettings,
}

/// Theme settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeSettings {
    /// Enable high contrast mode
    pub high_contrast: bool,
    /// Enable accessibility improvements
    pub accessibility: bool,
    /// Animation settings
    pub animations: bool,
}
