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
    /// Syntax highlighting theme
    pub syntax: SyntaxTheme,

    // === OpenCode-style UI Colors ===
    /// Muted text color (placeholders, hints, disabled)
    pub text_muted: ratatui::style::Color,
    /// Panel background (slightly raised surfaces)
    pub background_panel: ratatui::style::Color,
    /// Element background (buttons, inputs)
    pub background_element: ratatui::style::Color,
    /// Default border color
    pub border: ratatui::style::Color,
    /// Active/focused border color
    pub border_active: ratatui::style::Color,
    /// Agent colors by name
    pub agent_colors: AgentColors,
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

    /// Get a built-in theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "dark" => Some(Self::default()),
            "light" => Some(Self::light()),
            "monokai" => Some(Self::monokai()),
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "high-contrast" => Some(Self::high_contrast()),
            // New themes ported from OpenCode
            "tokyonight" | "tokyo-night" => Some(Self::tokyonight()),
            "catppuccin" => Some(Self::catppuccin()),
            "gruvbox" => Some(Self::gruvbox()),
            "one-dark" | "onedark" => Some(Self::one_dark()),
            "solarized" => Some(Self::solarized()),
            "opencode" => Some(Self::opencode()),
            _ => None,
        }
    }

    /// Get all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "dark",
            "light",
            "monokai",
            "dracula",
            "nord",
            "high-contrast",
            // New themes ported from OpenCode
            "tokyonight",
            "catppuccin",
            "gruvbox",
            "one-dark",
            "solarized",
            "opencode",
        ]
    }

    /// Create a default dark theme
    pub fn default() -> Self {
        use ratatui::style::Color;
        Self {
            name: "dark".to_string(),
            primary: Color::Rgb(255, 255, 255),
            secondary: Color::Rgb(204, 204, 204),
            background: Color::Rgb(0, 0, 0),
            foreground: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(255, 255, 0),
            error: Color::Rgb(255, 0, 0),
            warning: Color::Rgb(255, 255, 0),
            success: Color::Rgb(0, 255, 0),
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
            // OpenCode-style UI colors
            text_muted: Color::DarkGray,
            background_panel: Color::Rgb(30, 30, 30),
            background_element: Color::Rgb(45, 45, 45),
            border: Color::Rgb(60, 60, 60),
            border_active: Color::Cyan,
            agent_colors: AgentColors::default(),
        }
    }

    /// Create a light theme
    pub fn light() -> Self {
        use ratatui::style::Color;
        Self {
            name: "light".to_string(),
            primary: Color::Rgb(0, 0, 0),
            secondary: Color::Rgb(51, 51, 51),
            background: Color::Rgb(255, 255, 255),
            foreground: Color::Rgb(0, 0, 0),
            accent: Color::Rgb(0, 0, 255),
            error: Color::Rgb(255, 0, 0),
            warning: Color::Rgb(255, 102, 0),
            success: Color::Rgb(0, 170, 0),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(0, 0, 255),
                string: Color::Rgb(0, 128, 0),
                number: Color::Rgb(255, 102, 0),
                comment: Color::Rgb(119, 119, 119),
                function: Color::Rgb(128, 0, 128),
                variable: Color::Rgb(0, 0, 0),
                r#type: Color::Rgb(0, 128, 128),
                constant: Color::Rgb(0, 0, 255),
            },
            // OpenCode-style UI colors (light variant)
            text_muted: Color::Rgb(150, 150, 150),
            background_panel: Color::Rgb(245, 245, 245),
            background_element: Color::Rgb(235, 235, 235),
            border: Color::Rgb(200, 200, 200),
            border_active: Color::Rgb(0, 100, 200),
            agent_colors: AgentColors {
                build: Color::Rgb(0, 150, 150),
                plan: Color::Rgb(200, 130, 50),
                general: Color::Rgb(0, 150, 0),
                explore: Color::Rgb(0, 100, 200),
            },
        }
    }

    /// Create a monokai theme
    pub fn monokai() -> Self {
        use ratatui::style::Color;
        Self {
            name: "monokai".to_string(),
            primary: Color::Rgb(248, 248, 242),
            secondary: Color::Rgb(117, 113, 94),
            background: Color::Rgb(39, 40, 34),
            foreground: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(249, 38, 114),
            error: Color::Rgb(249, 38, 114),
            warning: Color::Rgb(253, 151, 31),
            success: Color::Rgb(166, 226, 46),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(249, 38, 114),
                string: Color::Rgb(230, 219, 116),
                number: Color::Rgb(174, 129, 255),
                comment: Color::Rgb(117, 113, 94),
                function: Color::Rgb(166, 226, 46),
                variable: Color::Rgb(248, 248, 242),
                r#type: Color::Rgb(102, 217, 239),
                constant: Color::Rgb(174, 129, 255),
            },
            text_muted: Color::Rgb(117, 113, 94),
            background_panel: Color::Rgb(49, 50, 44),
            background_element: Color::Rgb(59, 60, 54),
            border: Color::Rgb(70, 71, 65),
            border_active: Color::Rgb(102, 217, 239),
            agent_colors: AgentColors {
                build: Color::Rgb(102, 217, 239),
                plan: Color::Rgb(253, 151, 31),
                general: Color::Rgb(166, 226, 46),
                explore: Color::Rgb(174, 129, 255),
            },
        }
    }

    /// Create a dracula theme
    pub fn dracula() -> Self {
        use ratatui::style::Color;
        Self {
            name: "dracula".to_string(),
            primary: Color::Rgb(248, 248, 242),
            secondary: Color::Rgb(98, 114, 164),
            background: Color::Rgb(40, 42, 54),
            foreground: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(255, 121, 198),
            error: Color::Rgb(255, 85, 85),
            warning: Color::Rgb(241, 250, 140),
            success: Color::Rgb(80, 250, 123),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(189, 147, 249),
                string: Color::Rgb(241, 250, 140),
                number: Color::Rgb(189, 147, 249),
                comment: Color::Rgb(98, 114, 164),
                function: Color::Rgb(80, 250, 123),
                variable: Color::Rgb(248, 248, 242),
                r#type: Color::Rgb(139, 233, 253),
                constant: Color::Rgb(255, 121, 198),
            },
            text_muted: Color::Rgb(98, 114, 164),
            background_panel: Color::Rgb(50, 52, 64),
            background_element: Color::Rgb(60, 62, 74),
            border: Color::Rgb(68, 71, 90),
            border_active: Color::Rgb(139, 233, 253),
            agent_colors: AgentColors {
                build: Color::Rgb(139, 233, 253),
                plan: Color::Rgb(241, 250, 140),
                general: Color::Rgb(80, 250, 123),
                explore: Color::Rgb(189, 147, 249),
            },
        }
    }

    /// Create a nord theme
    pub fn nord() -> Self {
        use ratatui::style::Color;
        Self {
            name: "nord".to_string(),
            primary: Color::Rgb(216, 222, 233),
            secondary: Color::Rgb(136, 192, 208),
            background: Color::Rgb(46, 52, 64),
            foreground: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(163, 190, 140),
            error: Color::Rgb(191, 97, 106),
            warning: Color::Rgb(235, 203, 139),
            success: Color::Rgb(163, 190, 140),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(136, 192, 208),
                string: Color::Rgb(163, 190, 140),
                number: Color::Rgb(180, 142, 173),
                comment: Color::Rgb(76, 86, 106),
                function: Color::Rgb(136, 192, 208),
                variable: Color::Rgb(216, 222, 233),
                r#type: Color::Rgb(129, 161, 193),
                constant: Color::Rgb(180, 142, 173),
            },
            text_muted: Color::Rgb(76, 86, 106),
            background_panel: Color::Rgb(59, 66, 82),
            background_element: Color::Rgb(67, 76, 94),
            border: Color::Rgb(76, 86, 106),
            border_active: Color::Rgb(136, 192, 208),
            agent_colors: AgentColors {
                build: Color::Rgb(136, 192, 208),
                plan: Color::Rgb(235, 203, 139),
                general: Color::Rgb(163, 190, 140),
                explore: Color::Rgb(129, 161, 193),
            },
        }
    }

    /// Create a high contrast theme
    pub fn high_contrast() -> Self {
        use ratatui::style::Color;
        Self {
            name: "high-contrast".to_string(),
            primary: Color::Rgb(255, 255, 255),
            secondary: Color::Rgb(200, 200, 200),
            background: Color::Rgb(0, 0, 0),
            foreground: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(255, 255, 0),
            error: Color::Rgb(255, 0, 0),
            warning: Color::Rgb(255, 165, 0),
            success: Color::Rgb(0, 255, 0),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(255, 255, 0),
                string: Color::Rgb(0, 255, 0),
                number: Color::Rgb(0, 255, 255),
                comment: Color::Rgb(128, 128, 128),
                function: Color::Rgb(255, 0, 255),
                variable: Color::Rgb(255, 255, 255),
                r#type: Color::Rgb(0, 191, 255),
                constant: Color::Rgb(255, 20, 147),
            },
            text_muted: Color::Rgb(180, 180, 180),
            background_panel: Color::Rgb(20, 20, 20),
            background_element: Color::Rgb(40, 40, 40),
            border: Color::Rgb(255, 255, 255),
            border_active: Color::Rgb(255, 255, 0),
            agent_colors: AgentColors {
                build: Color::Rgb(0, 255, 255),
                plan: Color::Rgb(255, 165, 0),
                general: Color::Rgb(0, 255, 0),
                explore: Color::Rgb(0, 191, 255),
            },
        }
    }

    /// Create a Tokyo Night theme
    pub fn tokyonight() -> Self {
        use ratatui::style::Color;
        Self {
            name: "tokyonight".to_string(),
            primary: Color::Rgb(130, 170, 255),      // #82aaff
            secondary: Color::Rgb(192, 153, 255),    // #c099ff
            background: Color::Rgb(26, 27, 38),      // #1a1b26
            foreground: Color::Rgb(200, 211, 245),   // #c8d3f5
            accent: Color::Rgb(255, 150, 108),       // #ff966c
            error: Color::Rgb(255, 117, 127),        // #ff757f
            warning: Color::Rgb(255, 150, 108),      // #ff966c
            success: Color::Rgb(195, 232, 141),      // #c3e88d
            syntax: SyntaxTheme {
                keyword: Color::Rgb(192, 153, 255),  // #c099ff
                string: Color::Rgb(195, 232, 141),   // #c3e88d
                number: Color::Rgb(255, 150, 108),   // #ff966c
                comment: Color::Rgb(130, 139, 184),  // #828bb8
                function: Color::Rgb(130, 170, 255), // #82aaff
                variable: Color::Rgb(255, 117, 127), // #ff757f
                r#type: Color::Rgb(255, 199, 119),   // #ffc777
                constant: Color::Rgb(255, 150, 108), // #ff966c
            },
            text_muted: Color::Rgb(130, 139, 184),   // #828bb8
            background_panel: Color::Rgb(30, 32, 48),// #1e2030
            background_element: Color::Rgb(34, 36, 54), // #222436
            border: Color::Rgb(115, 122, 162),       // #737aa2
            border_active: Color::Rgb(144, 153, 178),// #9099b2
            agent_colors: AgentColors {
                build: Color::Rgb(134, 225, 252),    // #86e1fc
                plan: Color::Rgb(255, 199, 119),     // #ffc777
                general: Color::Rgb(195, 232, 141),  // #c3e88d
                explore: Color::Rgb(130, 170, 255),  // #82aaff
            },
        }
    }

    /// Create a Catppuccin theme (Mocha variant)
    pub fn catppuccin() -> Self {
        use ratatui::style::Color;
        Self {
            name: "catppuccin".to_string(),
            primary: Color::Rgb(137, 180, 250),      // #89b4fa
            secondary: Color::Rgb(203, 166, 247),    // #cba6f7
            background: Color::Rgb(30, 30, 46),      // #1e1e2e
            foreground: Color::Rgb(205, 214, 244),   // #cdd6f4
            accent: Color::Rgb(245, 194, 231),       // #f5c2e7
            error: Color::Rgb(243, 139, 168),        // #f38ba8
            warning: Color::Rgb(249, 226, 175),      // #f9e2af
            success: Color::Rgb(166, 227, 161),      // #a6e3a1
            syntax: SyntaxTheme {
                keyword: Color::Rgb(203, 166, 247),  // #cba6f7
                string: Color::Rgb(166, 227, 161),   // #a6e3a1
                number: Color::Rgb(250, 179, 135),   // #fab387
                comment: Color::Rgb(147, 153, 178),  // #9399b2
                function: Color::Rgb(137, 180, 250), // #89b4fa
                variable: Color::Rgb(243, 139, 168), // #f38ba8
                r#type: Color::Rgb(249, 226, 175),   // #f9e2af
                constant: Color::Rgb(250, 179, 135), // #fab387
            },
            text_muted: Color::Rgb(186, 194, 222),   // #bac2de
            background_panel: Color::Rgb(24, 24, 37),// #181825
            background_element: Color::Rgb(17, 17, 27), // #11111b
            border: Color::Rgb(49, 50, 68),          // #313244
            border_active: Color::Rgb(69, 71, 90),   // #45475a
            agent_colors: AgentColors {
                build: Color::Rgb(148, 226, 213),    // #94e2d5
                plan: Color::Rgb(249, 226, 175),     // #f9e2af
                general: Color::Rgb(166, 227, 161),  // #a6e3a1
                explore: Color::Rgb(137, 180, 250),  // #89b4fa
            },
        }
    }

    /// Create a Gruvbox theme
    pub fn gruvbox() -> Self {
        use ratatui::style::Color;
        Self {
            name: "gruvbox".to_string(),
            primary: Color::Rgb(131, 165, 152),      // #83a598
            secondary: Color::Rgb(211, 134, 155),    // #d3869b
            background: Color::Rgb(40, 40, 40),      // #282828
            foreground: Color::Rgb(235, 219, 178),   // #ebdbb2
            accent: Color::Rgb(142, 192, 124),       // #8ec07c
            error: Color::Rgb(251, 73, 52),          // #fb4934
            warning: Color::Rgb(254, 128, 25),       // #fe8019
            success: Color::Rgb(184, 187, 38),       // #b8bb26
            syntax: SyntaxTheme {
                keyword: Color::Rgb(251, 73, 52),    // #fb4934
                string: Color::Rgb(250, 189, 47),    // #fabd2f
                number: Color::Rgb(211, 134, 155),   // #d3869b
                comment: Color::Rgb(146, 131, 116),  // #928374
                function: Color::Rgb(184, 187, 38),  // #b8bb26
                variable: Color::Rgb(131, 165, 152), // #83a598
                r#type: Color::Rgb(142, 192, 124),   // #8ec07c
                constant: Color::Rgb(254, 128, 25),  // #fe8019
            },
            text_muted: Color::Rgb(146, 131, 116),   // #928374
            background_panel: Color::Rgb(60, 56, 54),// #3c3836
            background_element: Color::Rgb(80, 73, 69), // #504945
            border: Color::Rgb(102, 92, 84),         // #665c54
            border_active: Color::Rgb(235, 219, 178),// #ebdbb2
            agent_colors: AgentColors {
                build: Color::Rgb(131, 165, 152),    // #83a598
                plan: Color::Rgb(250, 189, 47),      // #fabd2f
                general: Color::Rgb(184, 187, 38),   // #b8bb26
                explore: Color::Rgb(69, 133, 136),   // #458588
            },
        }
    }

    /// Create a One Dark theme (Atom)
    pub fn one_dark() -> Self {
        use ratatui::style::Color;
        Self {
            name: "one-dark".to_string(),
            primary: Color::Rgb(97, 175, 239),       // #61afef
            secondary: Color::Rgb(198, 120, 221),    // #c678dd
            background: Color::Rgb(40, 44, 52),      // #282c34
            foreground: Color::Rgb(171, 178, 191),   // #abb2bf
            accent: Color::Rgb(86, 182, 194),        // #56b6c2
            error: Color::Rgb(224, 108, 117),        // #e06c75
            warning: Color::Rgb(229, 192, 123),      // #e5c07b
            success: Color::Rgb(152, 195, 121),      // #98c379
            syntax: SyntaxTheme {
                keyword: Color::Rgb(198, 120, 221),  // #c678dd
                string: Color::Rgb(152, 195, 121),   // #98c379
                number: Color::Rgb(209, 154, 102),   // #d19a66
                comment: Color::Rgb(92, 99, 112),    // #5c6370
                function: Color::Rgb(97, 175, 239),  // #61afef
                variable: Color::Rgb(224, 108, 117), // #e06c75
                r#type: Color::Rgb(229, 192, 123),   // #e5c07b
                constant: Color::Rgb(86, 182, 194),  // #56b6c2
            },
            text_muted: Color::Rgb(92, 99, 112),     // #5c6370
            background_panel: Color::Rgb(33, 37, 43),// #21252b
            background_element: Color::Rgb(53, 59, 69), // #353b45
            border: Color::Rgb(57, 63, 74),          // #393f4a
            border_active: Color::Rgb(97, 175, 239),// #61afef
            agent_colors: AgentColors {
                build: Color::Rgb(86, 182, 194),     // #56b6c2
                plan: Color::Rgb(229, 192, 123),     // #e5c07b
                general: Color::Rgb(152, 195, 121),  // #98c379
                explore: Color::Rgb(97, 175, 239),   // #61afef
            },
        }
    }

    /// Create a Solarized theme (dark variant)
    pub fn solarized() -> Self {
        use ratatui::style::Color;
        Self {
            name: "solarized".to_string(),
            primary: Color::Rgb(38, 139, 210),       // #268bd2
            secondary: Color::Rgb(108, 113, 196),    // #6c71c4
            background: Color::Rgb(0, 43, 54),       // #002b36
            foreground: Color::Rgb(131, 148, 150),   // #839496
            accent: Color::Rgb(42, 161, 152),        // #2aa198
            error: Color::Rgb(220, 50, 47),          // #dc322f
            warning: Color::Rgb(181, 137, 0),        // #b58900
            success: Color::Rgb(133, 153, 0),        // #859900
            syntax: SyntaxTheme {
                keyword: Color::Rgb(133, 153, 0),    // #859900
                string: Color::Rgb(42, 161, 152),    // #2aa198
                number: Color::Rgb(211, 54, 130),    // #d33682
                comment: Color::Rgb(88, 110, 117),   // #586e75
                function: Color::Rgb(38, 139, 210),  // #268bd2
                variable: Color::Rgb(42, 161, 152),  // #2aa198
                r#type: Color::Rgb(181, 137, 0),     // #b58900
                constant: Color::Rgb(133, 153, 0),   // #859900
            },
            text_muted: Color::Rgb(88, 110, 117),    // #586e75
            background_panel: Color::Rgb(7, 54, 66), // #073642
            background_element: Color::Rgb(7, 54, 66), // #073642
            border: Color::Rgb(7, 54, 66),           // #073642
            border_active: Color::Rgb(88, 110, 117), // #586e75
            agent_colors: AgentColors {
                build: Color::Rgb(42, 161, 152),     // #2aa198
                plan: Color::Rgb(181, 137, 0),       // #b58900
                general: Color::Rgb(133, 153, 0),    // #859900
                explore: Color::Rgb(38, 139, 210),   // #268bd2
            },
        }
    }

    /// Create an OpenCode theme (RiceCoder's reference theme)
    pub fn opencode() -> Self {
        use ratatui::style::Color;
        Self {
            name: "opencode".to_string(),
            primary: Color::Rgb(250, 178, 131),      // #fab283
            secondary: Color::Rgb(92, 156, 245),     // #5c9cf5
            background: Color::Rgb(10, 10, 10),      // #0a0a0a
            foreground: Color::Rgb(238, 238, 238),   // #eeeeee
            accent: Color::Rgb(157, 124, 216),       // #9d7cd8
            error: Color::Rgb(224, 108, 117),        // #e06c75
            warning: Color::Rgb(245, 167, 66),       // #f5a742
            success: Color::Rgb(127, 216, 143),      // #7fd88f
            syntax: SyntaxTheme {
                keyword: Color::Rgb(157, 124, 216),  // #9d7cd8
                string: Color::Rgb(127, 216, 143),   // #7fd88f
                number: Color::Rgb(245, 167, 66),    // #f5a742
                comment: Color::Rgb(128, 128, 128),  // #808080
                function: Color::Rgb(250, 178, 131), // #fab283
                variable: Color::Rgb(224, 108, 117), // #e06c75
                r#type: Color::Rgb(229, 192, 123),   // #e5c07b
                constant: Color::Rgb(86, 182, 194),  // #56b6c2
            },
            text_muted: Color::Rgb(128, 128, 128),   // #808080
            background_panel: Color::Rgb(20, 20, 20),// #141414
            background_element: Color::Rgb(30, 30, 30), // #1e1e1e
            border: Color::Rgb(72, 72, 72),          // #484848
            border_active: Color::Rgb(96, 96, 96),   // #606060
            agent_colors: AgentColors {
                build: Color::Rgb(86, 182, 194),     // #56b6c2
                plan: Color::Rgb(229, 192, 123),     // #e5c07b
                general: Color::Rgb(127, 216, 143),  // #7fd88f
                explore: Color::Rgb(92, 156, 245),   // #5c9cf5
            },
        }
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
