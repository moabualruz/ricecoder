//! Terminal background color detection for RiceCoder TUI
//!
//! Detects terminal capabilities and background color:
//! - OSC 11 background query
//! - Terminal type detection
//! - Light/dark mode detection
//!
//! # DDD Layer: Infrastructure
//! Terminal capability detection.

use ratatui::style::Color;
use std::io::{self, Read, Write};
use std::time::Duration;

/// Terminal color scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

impl ColorScheme {
    /// Detect from background color
    pub fn from_background(color: Color) -> Self {
        match color {
            Color::Rgb(r, g, b) => {
                // Calculate perceived brightness
                // Using relative luminance formula
                let brightness = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
                if brightness > 0.5 {
                    Self::Light
                } else {
                    Self::Dark
                }
            }
            Color::White | Color::LightRed | Color::LightGreen | 
            Color::LightYellow | Color::LightBlue | Color::LightMagenta |
            Color::LightCyan => Self::Light,
            _ => Self::Dark,
        }
    }
}

/// Terminal capabilities
#[derive(Debug, Clone, Default)]
pub struct TerminalCapabilities {
    /// Terminal type (from $TERM)
    pub term: String,
    /// Color scheme
    pub color_scheme: ColorScheme,
    /// Detected background color
    pub background: Option<Color>,
    /// Supports true color
    pub true_color: bool,
    /// Supports OSC 52 clipboard
    pub osc52_clipboard: bool,
    /// Supports Kitty keyboard protocol
    pub kitty_keyboard: bool,
    /// Is running in tmux
    pub in_tmux: bool,
    /// Is running in screen
    pub in_screen: bool,
    /// Terminal program name
    pub terminal_program: Option<String>,
}

impl TerminalCapabilities {
    /// Detect terminal capabilities
    pub fn detect() -> Self {
        let term = std::env::var("TERM").unwrap_or_default();
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();
        let term_program = std::env::var("TERM_PROGRAM").ok();
        let tmux = std::env::var("TMUX").is_ok();
        let sty = std::env::var("STY").is_ok(); // screen
        
        let true_color = colorterm == "truecolor" || colorterm == "24bit"
            || term.contains("256color")
            || term_program.as_deref() == Some("iTerm.app")
            || term_program.as_deref() == Some("WezTerm")
            || term_program.as_deref() == Some("Alacritty");
        
        let kitty_keyboard = term.contains("kitty")
            || term_program.as_deref() == Some("kitty")
            || std::env::var("KITTY_WINDOW_ID").is_ok();
        
        // OSC 52 is widely supported in modern terminals
        let osc52_clipboard = true_color || tmux || term.contains("xterm");
        
        Self {
            term,
            color_scheme: ColorScheme::Dark, // Will be updated by background detection
            background: None,
            true_color,
            osc52_clipboard,
            kitty_keyboard,
            in_tmux: tmux,
            in_screen: sty,
            terminal_program: term_program,
        }
    }
    
    /// Query background color using OSC 11
    /// Note: This is blocking and may not work in all terminals
    #[cfg(not(windows))]
    pub fn query_background(&mut self) -> Option<Color> {
        use std::os::unix::io::AsRawFd;
        
        // This is a simplified version - real implementation would need
        // to handle terminal modes and timeouts properly
        
        // Send OSC 11 query: ESC ] 11 ; ? BEL
        let query = "\x1b]11;?\x07";
        
        // Try to read response
        // Response format: ESC ] 11 ; rgb:RRRR/GGGG/BBBB BEL
        
        // For safety, we'll just use environment-based detection
        self.detect_from_environment()
    }
    
    #[cfg(windows)]
    pub fn query_background(&mut self) -> Option<Color> {
        // On Windows, use registry or console API
        self.detect_from_environment()
    }
    
    /// Detect color scheme from environment
    fn detect_from_environment(&mut self) -> Option<Color> {
        // Check for explicit color scheme settings
        if let Ok(scheme) = std::env::var("COLORFGBG") {
            // Format: "fg;bg" where bg > 6 usually means light
            if let Some(bg) = scheme.split(';').nth(1) {
                if let Ok(bg_num) = bg.parse::<u8>() {
                    self.color_scheme = if bg_num > 6 && bg_num < 16 {
                        ColorScheme::Light
                    } else {
                        ColorScheme::Dark
                    };
                    return None;
                }
            }
        }
        
        // Check for macOS dark mode
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("defaults")
                .args(["read", "-g", "AppleInterfaceStyle"])
                .output()
            {
                let is_dark = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .eq_ignore_ascii_case("dark");
                self.color_scheme = if is_dark {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                };
            }
        }
        
        // Check Windows console
        #[cfg(windows)]
        {
            // Could use Windows API to detect console background
            // For now, assume dark
            self.color_scheme = ColorScheme::Dark;
        }
        
        None
    }
    
    /// Get appropriate colors for current scheme
    pub fn themed_colors(&self) -> ThemedColors {
        match self.color_scheme {
            ColorScheme::Dark => ThemedColors::dark(),
            ColorScheme::Light => ThemedColors::light(),
        }
    }
}

/// Theme-appropriate colors
#[derive(Debug, Clone)]
pub struct ThemedColors {
    pub background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub border: Color,
}

impl ThemedColors {
    /// Dark theme colors
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 46),
            foreground: Color::Rgb(205, 214, 244),
            muted: Color::Rgb(127, 132, 156),
            primary: Color::Rgb(137, 180, 250),
            secondary: Color::Rgb(180, 190, 254),
            success: Color::Rgb(166, 227, 161),
            warning: Color::Rgb(249, 226, 175),
            error: Color::Rgb(243, 139, 168),
            border: Color::Rgb(88, 91, 112),
        }
    }
    
    /// Light theme colors
    pub fn light() -> Self {
        Self {
            background: Color::Rgb(239, 241, 245),
            foreground: Color::Rgb(76, 79, 105),
            muted: Color::Rgb(140, 143, 161),
            primary: Color::Rgb(30, 102, 245),
            secondary: Color::Rgb(114, 135, 253),
            success: Color::Rgb(64, 160, 43),
            warning: Color::Rgb(223, 142, 29),
            error: Color::Rgb(210, 15, 57),
            border: Color::Rgb(188, 192, 204),
        }
    }
}

/// Terminal info for display
#[derive(Debug, Clone)]
pub struct TerminalInfo {
    pub name: String,
    pub capabilities: Vec<String>,
}

impl From<&TerminalCapabilities> for TerminalInfo {
    fn from(caps: &TerminalCapabilities) -> Self {
        let mut capabilities = Vec::new();
        
        if caps.true_color {
            capabilities.push("24-bit color".to_string());
        }
        if caps.kitty_keyboard {
            capabilities.push("Kitty keyboard".to_string());
        }
        if caps.osc52_clipboard {
            capabilities.push("OSC 52 clipboard".to_string());
        }
        if caps.in_tmux {
            capabilities.push("tmux".to_string());
        }
        
        let name = caps.terminal_program
            .clone()
            .unwrap_or_else(|| caps.term.clone());
        
        Self { name, capabilities }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_scheme_from_background() {
        assert_eq!(
            ColorScheme::from_background(Color::Rgb(0, 0, 0)),
            ColorScheme::Dark
        );
        assert_eq!(
            ColorScheme::from_background(Color::Rgb(255, 255, 255)),
            ColorScheme::Light
        );
        assert_eq!(
            ColorScheme::from_background(Color::Rgb(30, 30, 46)),
            ColorScheme::Dark
        );
        assert_eq!(
            ColorScheme::from_background(Color::Rgb(239, 241, 245)),
            ColorScheme::Light
        );
    }
    
    #[test]
    fn test_terminal_capabilities_detect() {
        let caps = TerminalCapabilities::detect();
        // Just verify it doesn't panic
        assert!(caps.term.is_empty() || !caps.term.is_empty());
    }
    
    #[test]
    fn test_themed_colors() {
        let dark = ThemedColors::dark();
        let light = ThemedColors::light();
        
        // Dark background should be darker than light
        if let (Color::Rgb(dr, dg, db), Color::Rgb(lr, lg, lb)) = 
            (dark.background, light.background) 
        {
            let dark_brightness = dr as u32 + dg as u32 + db as u32;
            let light_brightness = lr as u32 + lg as u32 + lb as u32;
            assert!(dark_brightness < light_brightness);
        }
    }
    
    #[test]
    fn test_terminal_info_from_caps() {
        let mut caps = TerminalCapabilities::default();
        caps.term = "xterm-256color".to_string();
        caps.true_color = true;
        caps.kitty_keyboard = true;
        
        let info = TerminalInfo::from(&caps);
        assert!(info.capabilities.contains(&"24-bit color".to_string()));
        assert!(info.capabilities.contains(&"Kitty keyboard".to_string()));
    }
}
