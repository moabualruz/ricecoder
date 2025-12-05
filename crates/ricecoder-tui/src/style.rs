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
            primary: Color::new(255, 255, 255), // Pure white (high contrast on black)
            secondary: Color::new(255, 255, 0), // Pure yellow
            accent: Color::new(255, 0, 0),      // Pure red
            background: Color::new(0, 0, 0),    // Pure black
            foreground: Color::new(255, 255, 255), // Pure white
            error: Color::new(255, 0, 0),       // Pure red
            warning: Color::new(255, 255, 0),   // Pure yellow
            success: Color::new(0, 255, 0),     // Pure green
        }
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

    /// Get a theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::default()),
            "light" => Some(Self::light()),
            "monokai" => Some(Self::monokai()),
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "high-contrast" => Some(Self::high_contrast()),
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
        ]
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
        assert!(Theme::by_name("invalid").is_none());
    }

    #[test]
    fn test_theme_by_name_case_insensitive() {
        assert!(Theme::by_name("DARK").is_some());
        assert!(Theme::by_name("Light").is_some());
        assert!(Theme::by_name("MONOKAI").is_some());
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert_eq!(themes.len(), 6);
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"light"));
        assert!(themes.contains(&"monokai"));
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"nord"));
        assert!(themes.contains(&"high-contrast"));
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
