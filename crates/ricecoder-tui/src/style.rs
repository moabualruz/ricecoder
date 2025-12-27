//! Styling and theming for the TUI

use std::env;

use serde::{Deserialize, Serialize};

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

// NOTE: Theme definitions have been moved to JSON files in config/themes/
// Use ricecoder_themes::Theme and ricecoder_themes::ThemeLoader instead.
// 
// The following utility functions are kept for color support detection:

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
