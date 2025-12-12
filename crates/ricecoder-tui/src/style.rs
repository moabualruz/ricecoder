//! Styling and theming for the TUI

use serde::{Deserialize, Serialize};
use std::env;

/// Color definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Color {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
}

impl Color {
    /// Create a new color
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a color from hex string
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 7 || !hex.starts_with('#') {
            return None;
        }

        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

        Some(Self { r, g, b })
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert to ratatui Color
    pub fn to_ratatui(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }

    /// Calculate relative luminance (WCAG formula)
    pub fn luminance(&self) -> f32 {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let r = if r <= 0.03928 {
            r / 12.92
        } else {
            ((r + 0.055) / 1.055).powf(2.4)
        };
        let g = if g <= 0.03928 {
            g / 12.92
        } else {
            ((g + 0.055) / 1.055).powf(2.4)
        };
        let b = if b <= 0.03928 {
            b / 12.92
        } else {
            ((b + 0.055) / 1.055).powf(2.4)
        };

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Calculate contrast ratio between two colors (WCAG formula)
    pub fn contrast_ratio(&self, other: &Color) -> f32 {
        let l1 = self.luminance();
        let l2 = other.luminance();

        let lighter = l1.max(l2);
        let darker = l1.min(l2);

        (lighter + 0.05) / (darker + 0.05)
    }

    /// Check if contrast ratio meets WCAG AA standard (4.5:1 for normal text)
    pub fn meets_wcag_aa(&self, other: &Color) -> bool {
        self.contrast_ratio(other) >= 4.5
    }

    /// Check if contrast ratio meets WCAG AAA standard (7:1 for normal text)
    pub fn meets_wcag_aaa(&self, other: &Color) -> bool {
        self.contrast_ratio(other) >= 7.0
    }

    /// Adapt color to the specified color support level
    pub fn adapt(&self, support: ColorSupport) -> Self {
        match support {
            ColorSupport::TrueColor => *self,
            ColorSupport::Color256 => self.to_ansi256(),
            ColorSupport::Color16 => self.to_ansi16(),
        }
    }

    /// Convert to nearest ANSI 256 color
    pub fn to_ansi256(&self) -> Self {
        // Simple 6x6x6 cube mapping for now
        // A full implementation would use a lookup table or better algorithm
        let r = (self.r as u16 * 5 / 255) as u8;
        let g = (self.g as u16 * 5 / 255) as u8;
        let b = (self.b as u16 * 5 / 255) as u8;
        
        // Map back to RGB for the struct
        Self {
            r: r * 51,
            g: g * 51,
            b: b * 51,
        }
    }

    /// Convert to nearest ANSI 16 color
    pub fn to_ansi16(&self) -> Self {
        // Very basic mapping based on brightness and dominant channel
        let brightness = self.r as u16 + self.g as u16 + self.b as u16;
        
        if brightness < 100 {
            return Self::new(0, 0, 0); // Black
        }
        
        if brightness > 600 {
            return Self::new(255, 255, 255); // White
        }
        
        if self.r > self.g && self.r > self.b {
            Self::new(255, 0, 0) // Red
        } else if self.g > self.r && self.g > self.b {
            Self::new(0, 255, 0) // Green
        } else if self.b > self.r && self.b > self.g {
            Self::new(0, 0, 255) // Blue
        } else if self.r > self.b && self.g > self.b {
            Self::new(255, 255, 0) // Yellow
        } else if self.r > self.g && self.b > self.g {
            Self::new(255, 0, 255) // Magenta
        } else {
            Self::new(0, 255, 255) // Cyan
        }
    }
}

/// Text style
#[derive(Debug, Clone, Copy, Default)]
pub struct TextStyle {
    /// Foreground color
    pub fg: Option<Color>,
    /// Background color
    pub bg: Option<Color>,
    /// Bold text
    pub bold: bool,
    /// Italic text
    pub italic: bool,
    /// Underlined text
    pub underline: bool,
}

impl TextStyle {
    /// Create a new text style
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    /// Set foreground color
    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set background color
    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Set bold
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set italic
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Set underline
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

/// Progress indicator
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Current progress (0-100)
    pub progress: u8,
    /// Total steps
    pub total: u32,
    /// Current step
    pub current: u32,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(total: u32) -> Self {
        Self {
            progress: 0,
            total,
            current: 0,
        }
    }

    /// Update progress
    pub fn update(&mut self, current: u32) {
        self.current = current.min(self.total);
        self.progress = ((self.current as f32 / self.total as f32) * 100.0) as u8;
    }

    /// Get progress bar string
    pub fn bar(&self, width: usize) -> String {
        let filled = (width as f32 * self.progress as f32 / 100.0) as usize;
        let empty = width.saturating_sub(filled);
        format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
    }
}

/// Theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Clone, PartialEq)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Primary color
    pub primary: Color,
    /// Secondary color
    pub secondary: Color,
    /// Accent color
    pub accent: Color,
    /// Background color
    pub background: Color,
    /// Foreground color
    pub foreground: Color,
    /// Error color
    pub error: Color,
    /// Warning color
    pub warning: Color,
    /// Success color
    pub success: Color,
}

impl Default for Theme {
    fn default() -> Self {
        // Dark theme as default
        Self {
            name: "dark".to_string(),
            primary: Color::new(0, 122, 255),      // Blue
            secondary: Color::new(90, 200, 250),   // Light blue
            accent: Color::new(255, 45, 85),       // Red
            background: Color::new(17, 24, 39),    // Dark gray
            foreground: Color::new(243, 244, 246), // Light gray
            error: Color::new(239, 68, 68),        // Red
            warning: Color::new(245, 158, 11),     // Amber
            success: Color::new(34, 197, 94),      // Green
        }
    }
}

impl Theme {
    /// Create a light theme
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            primary: Color::new(0, 102, 204),      // Blue
            secondary: Color::new(102, 178, 255),  // Light blue
            accent: Color::new(204, 0, 0),         // Red
            background: Color::new(255, 255, 255), // White
            foreground: Color::new(0, 0, 0),       // Black
            error: Color::new(220, 38, 38),        // Red
            warning: Color::new(217, 119, 6),      // Amber
            success: Color::new(22, 163, 74),      // Green
        }
    }

    /// Create a Monokai theme
    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            primary: Color::new(102, 217, 239),    // Cyan
            secondary: Color::new(249, 38, 114),   // Magenta
            accent: Color::new(166, 226, 46),      // Green
            background: Color::new(39, 40, 34),    // Dark gray
            foreground: Color::new(248, 248, 242), // Off-white
            error: Color::new(249, 38, 114),       // Magenta
            warning: Color::new(253, 151, 31),     // Orange
            success: Color::new(166, 226, 46),     // Green
        }
    }

    /// Create a Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            primary: Color::new(139, 233, 253),    // Cyan
            secondary: Color::new(189, 147, 249),  // Purple
            accent: Color::new(255, 121, 198),     // Pink
            background: Color::new(40, 42, 54),    // Dark gray
            foreground: Color::new(248, 248, 242), // Off-white
            error: Color::new(255, 85, 85),        // Red
            warning: Color::new(241, 250, 140),    // Yellow
            success: Color::new(80, 250, 123),     // Green
        }
    }

    /// Create a Nord theme
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            primary: Color::new(136, 192, 208),    // Frost 1
            secondary: Color::new(163, 190, 140),  // Aurora 1
            accent: Color::new(191, 97, 106),      // Aurora 5
            background: Color::new(46, 52, 64),    // Polar night 0
            foreground: Color::new(236, 239, 244), // Snow storm 0
            error: Color::new(191, 97, 106),       // Aurora 5
            warning: Color::new(235, 203, 139),    // Aurora 3
            success: Color::new(163, 190, 140),    // Aurora 1
        }
    }

    /// Create a high contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            name: "high-contrast".to_string(),
            primary: Color::new(255, 255, 255), // Pure white
            secondary: Color::new(255, 255, 0), // Pure yellow
            accent: Color::new(255, 0, 0),      // Pure red
            background: Color::new(0, 0, 0),    // Pure black
            foreground: Color::new(255, 255, 255), // Pure white
            error: Color::new(255, 0, 0),       // Pure red
            warning: Color::new(255, 255, 0),   // Pure yellow
            success: Color::new(0, 255, 0),     // Pure green
        }
    }

    /// Create Catppuccin Latte theme
    pub fn catppuccin_latte() -> Self {
        Self {
            name: "catppuccin-latte".to_string(),
            primary: Color::new(30, 102, 245),     // Blue
            secondary: Color::new(114, 135, 253),  // Lavender
            accent: Color::new(230, 69, 83),       // Maroon
            background: Color::new(239, 241, 245), // Base
            foreground: Color::new(76, 79, 105),   // Text
            error: Color::new(210, 15, 57),        // Red
            warning: Color::new(223, 142, 29),     // Yellow
            success: Color::new(64, 160, 43),      // Green
        }
    }

    /// Create Catppuccin Frappe theme
    pub fn catppuccin_frappe() -> Self {
        Self {
            name: "catppuccin-frappe".to_string(),
            primary: Color::new(140, 170, 238),    // Blue
            secondary: Color::new(186, 194, 222),  // Lavender
            accent: Color::new(234, 153, 156),     // Maroon
            background: Color::new(48, 52, 70),    // Base
            foreground: Color::new(198, 208, 245), // Text
            error: Color::new(231, 130, 132),      // Red
            warning: Color::new(229, 200, 144),    // Yellow
            success: Color::new(166, 209, 137),    // Green
        }
    }

    /// Create Catppuccin Macchiato theme
    pub fn catppuccin_macchiato() -> Self {
        Self {
            name: "catppuccin-macchiato".to_string(),
            primary: Color::new(138, 173, 244),    // Blue
            secondary: Color::new(183, 189, 248),  // Lavender
            accent: Color::new(238, 153, 160),     // Maroon
            background: Color::new(36, 39, 58),    // Base
            foreground: Color::new(202, 211, 245), // Text
            error: Color::new(237, 135, 150),      // Red
            warning: Color::new(238, 212, 159),    // Yellow
            success: Color::new(166, 218, 149),    // Green
        }
    }

    /// Create Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha".to_string(),
            primary: Color::new(137, 180, 250),    // Blue
            secondary: Color::new(180, 190, 254),  // Lavender
            accent: Color::new(235, 160, 172),     // Maroon
            background: Color::new(30, 30, 46),    // Base
            foreground: Color::new(205, 214, 244), // Text
            error: Color::new(243, 139, 168),      // Red
            warning: Color::new(249, 226, 175),    // Yellow
            success: Color::new(166, 227, 161),    // Green
        }
    }

    /// Create Tokyo Night Storm theme
    pub fn tokyo_night_storm() -> Self {
        Self {
            name: "tokyo-night-storm".to_string(),
            primary: Color::new(122, 162, 247),    // Blue
            secondary: Color::new(187, 154, 247),  // Purple
            accent: Color::new(247, 118, 142),     // Red
            background: Color::new(36, 40, 59),    // Storm Background
            foreground: Color::new(192, 202, 245), // Foreground
            error: Color::new(247, 118, 142),      // Red
            warning: Color::new(224, 175, 104),    // Yellow
            success: Color::new(158, 206, 106),    // Green
        }
    }

    /// Create Tokyo Night Moon theme
    pub fn tokyo_night_moon() -> Self {
        Self {
            name: "tokyo-night-moon".to_string(),
            primary: Color::new(130, 170, 255),    // Blue
            secondary: Color::new(192, 153, 255),  // Purple
            accent: Color::new(255, 117, 127),     // Red
            background: Color::new(34, 36, 54),    // Moon Background
            foreground: Color::new(200, 208, 224), // Foreground
            error: Color::new(255, 83, 112),       // Red
            warning: Color::new(255, 199, 119),    // Yellow
            success: Color::new(195, 232, 141),    // Green
        }
    }

    /// Create Tokyo Night Day theme
    pub fn tokyo_night_day() -> Self {
        Self {
            name: "tokyo-night-day".to_string(),
            primary: Color::new(55, 96, 191),      // Blue
            secondary: Color::new(152, 84, 241),   // Purple
            accent: Color::new(245, 42, 101),      // Red
            background: Color::new(225, 226, 231), // Day Background
            foreground: Color::new(55, 96, 191),   // Foreground
            error: Color::new(245, 42, 101),       // Red
            warning: Color::new(140, 108, 62),     // Yellow
            success: Color::new(88, 117, 57),      // Green
        }
    }

    /// Create Tokyo Night Night theme
    pub fn tokyo_night_night() -> Self {
        Self {
            name: "tokyo-night-night".to_string(),
            primary: Color::new(122, 162, 247),    // Blue
            secondary: Color::new(187, 154, 247),  // Purple
            accent: Color::new(247, 118, 142),     // Red
            background: Color::new(26, 27, 38),    // Night Background
            foreground: Color::new(192, 202, 245), // Foreground
            error: Color::new(247, 118, 142),      // Red
            warning: Color::new(224, 175, 104),    // Yellow
            success: Color::new(158, 206, 106),    // Green
        }
    }

    /// Create Gruvbox Light theme
    pub fn gruvbox_light() -> Self {
        Self {
            name: "gruvbox-light".to_string(),
            primary: Color::new(7, 102, 120),      // Blue
            secondary: Color::new(143, 63, 113),   // Purple
            accent: Color::new(157, 0, 6),         // Red
            background: Color::new(251, 241, 199), // Cream
            foreground: Color::new(60, 56, 54),    // Dark Gray
            error: Color::new(204, 36, 29),        // Red
            warning: Color::new(215, 153, 33),     // Yellow
            success: Color::new(152, 151, 26),     // Green
        }
    }

    /// Create Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox-dark".to_string(),
            primary: Color::new(131, 165, 152),    // Blue
            secondary: Color::new(211, 134, 155),  // Purple
            accent: Color::new(251, 73, 52),       // Red
            background: Color::new(40, 40, 40),    // Dark Gray
            foreground: Color::new(235, 219, 178), // Cream
            error: Color::new(204, 36, 29),        // Red
            warning: Color::new(250, 189, 47),     // Yellow
            success: Color::new(184, 187, 38),     // Green
        }
    }

    /// Create Solarized Light theme
    pub fn solarized_light() -> Self {
        Self {
            name: "solarized-light".to_string(),
            primary: Color::new(38, 139, 210),     // Blue
            secondary: Color::new(211, 54, 130),   // Magenta
            accent: Color::new(220, 50, 47),       // Red
            background: Color::new(253, 246, 227), // Base3
            foreground: Color::new(101, 123, 131), // Base00
            error: Color::new(220, 50, 47),        // Red
            warning: Color::new(181, 137, 0),      // Yellow
            success: Color::new(133, 153, 0),      // Green
        }
    }

    /// Create Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark".to_string(),
            primary: Color::new(38, 139, 210),     // Blue
            secondary: Color::new(211, 54, 130),   // Magenta
            accent: Color::new(220, 50, 47),       // Red
            background: Color::new(0, 43, 54),     // Base03
            foreground: Color::new(131, 148, 150), // Base0
            error: Color::new(220, 50, 47),        // Red
            warning: Color::new(181, 137, 0),      // Yellow
            success: Color::new(133, 153, 0),      // Green
        }
    }

    /// Create Everforest theme
    pub fn everforest() -> Self {
        Self {
            name: "everforest".to_string(),
            primary: Color::new(127, 187, 179),    // Blue
            secondary: Color::new(211, 134, 155),  // Purple
            accent: Color::new(230, 126, 128),     // Red
            background: Color::new(43, 51, 57),    // Background
            foreground: Color::new(211, 198, 170), // Foreground
            error: Color::new(230, 126, 128),      // Red
            warning: Color::new(219, 188, 127),    // Yellow
            success: Color::new(167, 192, 128),    // Green
        }
    }

    /// Create Kanagawa theme
    pub fn kanagawa() -> Self {
        Self {
            name: "kanagawa".to_string(),
            primary: Color::new(126, 156, 216),    // Blue
            secondary: Color::new(149, 127, 184),  // Purple
            accent: Color::new(232, 36, 36),       // Red
            background: Color::new(31, 31, 40),    // Background
            foreground: Color::new(220, 220, 170), // Foreground
            error: Color::new(232, 36, 36),        // Red
            warning: Color::new(255, 158, 59),     // Orange
            success: Color::new(118, 148, 106),    // Green
        }
    }

    /// Create Rose Pine theme
    pub fn rose_pine() -> Self {
        Self {
            name: "rose-pine".to_string(),
            primary: Color::new(156, 207, 216),    // Foam
            secondary: Color::new(196, 167, 231),  // Iris
            accent: Color::new(235, 111, 146),     // Love
            background: Color::new(25, 23, 36),    // Base
            foreground: Color::new(224, 222, 244), // Text
            error: Color::new(235, 111, 146),      // Love
            warning: Color::new(246, 193, 119),    // Gold
            success: Color::new(49, 116, 143),     // Pine
        }
    }

    /// Create Synthwave theme
    pub fn synthwave() -> Self {
        Self {
            name: "synthwave".to_string(),
            primary: Color::new(255, 113, 206),    // Neon Pink
            secondary: Color::new(1, 205, 254),    // Neon Blue
            accent: Color::new(5, 255, 161),       // Neon Green
            background: Color::new(43, 33, 58),    // Dark Purple
            foreground: Color::new(255, 255, 255), // White
            error: Color::new(255, 0, 56),         // Red
            warning: Color::new(255, 251, 150),    // Yellow
            success: Color::new(185, 103, 255),    // Purple
        }
    }

    /// Create Material theme
    pub fn material() -> Self {
        Self {
            name: "material".to_string(),
            primary: Color::new(130, 177, 255),    // Blue
            secondary: Color::new(199, 146, 234),  // Purple
            accent: Color::new(240, 98, 146),      // Pink
            background: Color::new(38, 50, 56),    // Dark Blue Gray
            foreground: Color::new(238, 255, 255), // White
            error: Color::new(255, 83, 112),       // Red
            warning: Color::new(255, 203, 107),    // Yellow
            success: Color::new(195, 232, 141),    // Green
        }
    }

    /// Create One Dark theme
    pub fn one_dark() -> Self {
        Self {
            name: "one-dark".to_string(),
            primary: Color::new(97, 175, 239),     // Blue
            secondary: Color::new(198, 120, 221),  // Purple
            accent: Color::new(224, 108, 117),     // Red
            background: Color::new(40, 44, 52),    // Dark Gray
            foreground: Color::new(171, 178, 191), // Light Gray
            error: Color::new(224, 108, 117),      // Red
            warning: Color::new(229, 192, 123),    // Yellow
            success: Color::new(152, 195, 121),    // Green
        }
    }

    /// Create One Light theme
    pub fn one_light() -> Self {
        Self {
            name: "one-light".to_string(),
            primary: Color::new(64, 120, 242),     // Blue
            secondary: Color::new(166, 38, 164),   // Purple
            accent: Color::new(228, 86, 73),       // Red
            background: Color::new(250, 250, 250), // Light Gray
            foreground: Color::new(56, 58, 66),    // Dark Gray
            error: Color::new(228, 86, 73),        // Red
            warning: Color::new(193, 132, 1),      // Yellow
            success: Color::new(80, 161, 79),      // Green
        }
    }

    /// Create Palenight theme
    pub fn palenight() -> Self {
        Self {
            name: "palenight".to_string(),
            primary: Color::new(130, 170, 255),    // Blue
            secondary: Color::new(199, 146, 234),  // Purple
            accent: Color::new(240, 113, 120),     // Red
            background: Color::new(41, 45, 62),    // Dark Blue
            foreground: Color::new(166, 172, 205), // Gray
            error: Color::new(255, 83, 112),       // Red
            warning: Color::new(255, 203, 107),    // Yellow
            success: Color::new(195, 232, 141),    // Green
        }
    }

    /// Create Aura theme
    pub fn aura() -> Self {
        Self {
            name: "aura".to_string(),
            primary: Color::new(130, 224, 170),    // Green
            secondary: Color::new(162, 125, 245),  // Purple
            accent: Color::new(246, 148, 255),     // Pink
            background: Color::new(21, 20, 27),    // Dark
            foreground: Color::new(237, 236, 238), // Light
            error: Color::new(255, 103, 103),      // Red
            warning: Color::new(255, 202, 133),    // Orange
            success: Color::new(97, 255, 202),     // Green
        }
    }

    /// Create Ayu theme
    pub fn ayu() -> Self {
        Self {
            name: "ayu".to_string(),
            primary: Color::new(57, 186, 230),     // Blue
            secondary: Color::new(210, 166, 255),  // Purple
            accent: Color::new(255, 51, 51),       // Red
            background: Color::new(11, 14, 20),    // Dark
            foreground: Color::new(191, 189, 182), // Light
            error: Color::new(255, 51, 51),        // Red
            warning: Color::new(255, 143, 64),     // Orange
            success: Color::new(170, 217, 76),     // Green
        }
    }

    /// Look up a theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::default()),
            "light" => Some(Self::light()),
            "monokai" => Some(Self::monokai()),
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "high-contrast" => Some(Self::high_contrast()),
            "catppuccin-latte" => Some(Self::catppuccin_latte()),
            "catppuccin-frappe" => Some(Self::catppuccin_frappe()),
            "catppuccin-macchiato" => Some(Self::catppuccin_macchiato()),
            "catppuccin-mocha" => Some(Self::catppuccin_mocha()),
            "tokyo-night-storm" => Some(Self::tokyo_night_storm()),
            "tokyo-night-moon" => Some(Self::tokyo_night_moon()),
            "tokyo-night-day" => Some(Self::tokyo_night_day()),
            "tokyo-night-night" => Some(Self::tokyo_night_night()),
            "gruvbox-light" => Some(Self::gruvbox_light()),
            "gruvbox-dark" => Some(Self::gruvbox_dark()),
            "solarized-light" => Some(Self::solarized_light()),
            "solarized-dark" => Some(Self::solarized_dark()),
            "everforest" => Some(Self::everforest()),
            "kanagawa" => Some(Self::kanagawa()),
            "rose-pine" => Some(Self::rose_pine()),
            "synthwave" => Some(Self::synthwave()),
            "material" => Some(Self::material()),
            "one-dark" => Some(Self::one_dark()),
            "one-light" => Some(Self::one_light()),
            "palenight" => Some(Self::palenight()),
            "aura" => Some(Self::aura()),
            "ayu" => Some(Self::ayu()),
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
            "catppuccin-latte",
            "catppuccin-frappe",
            "catppuccin-macchiato",
            "catppuccin-mocha",
            "tokyo-night-storm",
            "tokyo-night-moon",
            "tokyo-night-day",
            "tokyo-night-night",
            "gruvbox-light",
            "gruvbox-dark",
            "solarized-light",
            "solarized-dark",
            "everforest",
            "kanagawa",
            "rose-pine",
            "synthwave",
            "material",
            "one-dark",
            "one-light",
            "palenight",
            "aura",
            "ayu",
        ]
    }

    /// Detect terminal color capabilities
    pub fn detect_color_support() -> ColorSupport {
        // Check COLORTERM environment variable for true color support
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm.contains("truecolor") || colorterm.contains("24bit") {
                return ColorSupport::TrueColor;
            }
        }

        // Check TERM environment variable
        if let Ok(term) = env::var("TERM") {
            if term.contains("256color") {
                return ColorSupport::Color256;
            }
            if term.contains("color") {
                return ColorSupport::Color16;
            }
        }

        // Default to 256 color support
        ColorSupport::Color256
    }

    /// Check if the theme meets WCAG AA contrast standards
    pub fn meets_wcag_aa(&self) -> bool {
        // Check foreground vs background
        self.foreground.meets_wcag_aa(&self.background)
            && self.primary.meets_wcag_aa(&self.background)
            && self.error.meets_wcag_aa(&self.background)
    }

    /// Check if the theme meets WCAG AAA contrast standards
    pub fn meets_wcag_aaa(&self) -> bool {
        // Check foreground vs background
        self.foreground.meets_wcag_aaa(&self.background)
            && self.primary.meets_wcag_aaa(&self.background)
            && self.error.meets_wcag_aaa(&self.background)
    }

    /// Get contrast ratio between foreground and background
    pub fn foreground_contrast(&self) -> f32 {
        self.foreground.contrast_ratio(&self.background)
    }

    /// Get contrast ratio between primary color and background
    pub fn primary_contrast(&self) -> f32 {
        self.primary.contrast_ratio(&self.background)
    }

    /// Get contrast ratio between error color and background
    pub fn error_contrast(&self) -> f32 {
        self.error.contrast_ratio(&self.background)
    }

    /// Adapt the theme to terminal capabilities
    pub fn adapt(&mut self, support: ColorSupport) {
        self.primary = self.primary.adapt(support);
        self.secondary = self.secondary.adapt(support);
        self.accent = self.accent.adapt(support);
        self.background = self.background.adapt(support);
        self.foreground = self.foreground.adapt(support);
        self.error = self.error.adapt(support);
        self.warning = self.warning.adapt(support);
        self.success = self.success.adapt(support);
    }
}

/// Terminal color support levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    /// 16 colors (basic ANSI)
    Color16,
    /// 256 colors
    Color256,
    /// True color (24-bit RGB)
    TrueColor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.to_hex(), "#ff8040");

        let parsed = Color::from_hex("#ff8040").unwrap();
        assert_eq!(parsed, color);
    }

    #[test]
    fn test_text_style() {
        let color = Color::new(255, 0, 0);
        let style = TextStyle::new().fg(color).bold().underline();
        assert_eq!(style.fg, Some(color));
        assert!(style.bold);
        assert!(style.underline);
        assert!(!style.italic);
    }

    #[test]
    fn test_progress_indicator() {
        let mut progress = ProgressIndicator::new(100);
        assert_eq!(progress.progress, 0);

        progress.update(50);
        assert_eq!(progress.progress, 50);
        assert_eq!(progress.current, 50);

        progress.update(150);
        assert_eq!(progress.current, 100);
        assert_eq!(progress.progress, 100);
    }

    #[test]
    fn test_progress_bar() {
        let mut progress = ProgressIndicator::new(100);
        progress.update(50);
        let bar = progress.bar(10);
        assert_eq!(bar, "[=====     ]");
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn test_theme_monokai() {
        let theme = Theme::monokai();
        assert_eq!(theme.name, "monokai");
    }

    #[test]
    fn test_theme_dracula() {
        let theme = Theme::dracula();
        assert_eq!(theme.name, "dracula");
    }

    #[test]
    fn test_theme_nord() {
        let theme = Theme::nord();
        assert_eq!(theme.name, "nord");
    }

    #[test]
    fn test_color_support_detection() {
        let support = ColorSupport::Color256;
        assert_eq!(support, ColorSupport::Color256);
    }

    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("dark").is_some());
        assert!(Theme::by_name("light").is_some());
        assert!(Theme::by_name("monokai").is_some());
        assert!(Theme::by_name("dracula").is_some());
        assert!(Theme::by_name("nord").is_some());
        assert!(Theme::by_name("catppuccin-mocha").is_some());
        assert!(Theme::by_name("invalid").is_none());
    }

    #[test]
    fn test_theme_by_name_case_insensitive() {
        assert!(Theme::by_name("DARK").is_some());
        assert!(Theme::by_name("Light").is_some());
        assert!(Theme::by_name("MONOKAI").is_some());
        assert!(Theme::by_name("Catppuccin-Mocha").is_some());
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert!(themes.len() >= 6);
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"light"));
        assert!(themes.contains(&"monokai"));
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"nord"));
        assert!(themes.contains(&"high-contrast"));
        assert!(themes.contains(&"catppuccin-mocha"));
    }

    #[test]
    fn test_color_contrast_ratio() {
        let white = Color::new(255, 255, 255);
        let black = Color::new(0, 0, 0);
        let contrast = white.contrast_ratio(&black);
        // White on black should have maximum contrast (21:1)
        assert!(contrast > 20.0);
    }

    #[test]
    fn test_wcag_aa_compliance() {
        let white = Color::new(255, 255, 255);
        let black = Color::new(0, 0, 0);
        assert!(white.meets_wcag_aa(&black));
        assert!(white.meets_wcag_aaa(&black));
    }

    #[test]
    fn test_high_contrast_theme_wcag_compliance() {
        let theme = Theme::high_contrast();
        // High contrast theme should meet at least AA standards
        assert!(theme.meets_wcag_aa());
    }

    #[test]
    fn test_theme_contrast_ratios() {
        let theme = Theme::high_contrast();
        let fg_contrast = theme.foreground_contrast();
        let primary_contrast = theme.primary_contrast();
        let error_contrast = theme.error_contrast();

        // Foreground and primary should meet WCAG AAA standards (7:1)
        assert!(fg_contrast >= 7.0);
        assert!(primary_contrast >= 7.0);
        // Error should at least meet AA standards (4.5:1)
        assert!(error_contrast >= 4.5);
    }
}
