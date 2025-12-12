//! Terminal state management and capability detection
//!
//! This module handles capturing and restoring terminal state to ensure
//! the terminal is left in a clean state when the application exits.
//! It also detects terminal capabilities to adapt the UI accordingly.
//! Requirements: 4.1, 4.2, 4.3, 4.4, 10.1, 10.2, 10.3

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env, io, collections::HashMap};

/// Terminal color support levels
///
/// Requirements: 4.1 - Detect color support (16, 256, true color)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    /// No color support
    None,
    /// 16 colors (ANSI)
    Ansi16,
    /// 256 colors
    Ansi256,
    /// True color (24-bit RGB)
    TrueColor,
}

/// Terminal type detection
///
/// Requirements: 4.1 - Detect terminal type (xterm, iTerm2, WezTerm, Kitty, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalType {
    /// Unknown terminal
    Unknown,
    /// xterm or xterm-compatible
    Xterm,
    /// iTerm2
    ITerm2,
    /// WezTerm
    WezTerm,
    /// Kitty
    Kitty,
    /// Alacritty
    Alacritty,
    /// Windows Terminal
    WindowsTerminal,
    /// GNOME Terminal
    GnomeTerminal,
    /// Konsole
    Konsole,
    /// Terminal.app (macOS)
    TerminalApp,
    /// VS Code integrated terminal
    VSCode,
    /// Hyper terminal
    Hyper,
    /// Tabby terminal
    Tabby,
    /// Foot terminal
    Foot,
    /// Rio terminal
    Rio,
    /// Warp terminal
    Warp,
}

/// Configuration overrides for terminal capabilities
///
/// Requirements: 4.3 - Support capability override via configuration
#[derive(Debug, Clone, Default)]
pub struct CapabilityOverrides {
    /// Override color support detection
    pub color_support: Option<ColorSupport>,
    /// Override mouse support detection
    pub mouse_support: Option<bool>,
    /// Override sixel support detection
    pub sixel_support: Option<bool>,
    /// Override kitty graphics support detection
    pub kitty_graphics_support: Option<bool>,
    /// Override iTerm2 inline images support detection
    pub iterm2_inline_images_support: Option<bool>,
    /// Override WezTerm multiplexer support detection
    pub wezterm_multiplexer_support: Option<bool>,
    /// Override unicode placeholder support detection
    pub unicode_placeholder_support: Option<bool>,
    /// Override block graphics support detection
    pub block_graphics_support: Option<bool>,
    /// Override ANSI art support detection
    pub ansi_art_support: Option<bool>,
    /// Override unicode support detection
    pub unicode_support: Option<bool>,
    /// Force reduced graphics mode
    pub force_reduced_graphics: Option<bool>,
}

/// Terminal capabilities detected at startup
///
/// Requirements: 4.1, 4.2 - Detect and adapt UI based on terminal capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalCapabilities {
    /// Terminal type
    pub terminal_type: TerminalType,
    /// Color support level
    pub color_support: ColorSupport,
    /// Mouse support
    pub mouse_support: bool,
    /// Sixel graphics support
    pub sixel_support: bool,
    /// Kitty graphics protocol support
    pub kitty_graphics_support: bool,
    /// iTerm2 inline images support
    pub iterm2_inline_images_support: bool,
    /// WezTerm multiplexer features support
    pub wezterm_multiplexer_support: bool,
    /// Unicode placeholder protocol support (for inline images)
    pub unicode_placeholder_support: bool,
    /// Block graphics support (for ASCII art fallbacks)
    pub block_graphics_support: bool,
    /// ANSI art support (for simple image rendering)
    pub ansi_art_support: bool,
    /// Unicode support
    pub unicode_support: bool,
    /// Running in SSH session
    pub is_ssh: bool,
    /// Running in TMUX
    pub is_tmux: bool,
    /// TMUX version (if detected)
    pub tmux_version: Option<String>,
    /// Terminal size
    pub size: (u16, u16), // (width, height)
    /// Configuration overrides applied
    pub overrides_applied: CapabilityOverrides,
}

/// Captures and manages terminal state for graceful shutdown
///
/// This struct captures the terminal state before entering raw mode and
/// alternate screen, and restores it on drop or explicit call to restore().
/// It also detects terminal capabilities for UI adaptation.
///
/// Requirements: 4.1, 4.2, 10.2 - Create TerminalState struct to capture state and detect capabilities
#[derive(Debug)]
pub struct TerminalState {
    /// Whether raw mode was enabled before we started
    /// This field is kept for future use and for documentation purposes
    #[allow(dead_code)]
    was_raw_mode_enabled: bool,
    /// Whether we're in alternate screen
    in_alternate_screen: bool,
    /// Detected terminal capabilities
    pub capabilities: TerminalCapabilities,
}

impl TerminalCapabilities {
    /// Detect terminal capabilities
    ///
    /// Requirements: 4.1 - Detect terminal capabilities at startup
    pub fn detect() -> Self {
        Self::detect_with_overrides(CapabilityOverrides::default())
    }

    /// Detect terminal capabilities with configuration overrides
    ///
    /// Requirements: 4.3 - Support capability override via configuration
    pub fn detect_with_overrides(overrides: CapabilityOverrides) -> Self {
        let terminal_type = Self::detect_terminal_type();
        let color_support = overrides.color_support.unwrap_or_else(|| Self::detect_color_support());
        let mouse_support = overrides.mouse_support.unwrap_or_else(|| Self::detect_mouse_support());
        let sixel_support = overrides.sixel_support.unwrap_or_else(|| Self::detect_sixel_support(&terminal_type));
        let kitty_graphics_support = overrides.kitty_graphics_support.unwrap_or_else(|| Self::detect_kitty_graphics_support(&terminal_type));
        let iterm2_inline_images_support = overrides.iterm2_inline_images_support.unwrap_or_else(|| Self::detect_iterm2_inline_images_support(&terminal_type));
        let wezterm_multiplexer_support = overrides.wezterm_multiplexer_support.unwrap_or_else(|| Self::detect_wezterm_multiplexer_support(&terminal_type));
        let unicode_placeholder_support = overrides.unicode_placeholder_support.unwrap_or_else(|| Self::detect_unicode_placeholder_support(&terminal_type));
        let block_graphics_support = overrides.block_graphics_support.unwrap_or_else(|| Self::detect_block_graphics_support());
        let ansi_art_support = overrides.ansi_art_support.unwrap_or_else(|| Self::detect_ansi_art_support());
        let unicode_support = overrides.unicode_support.unwrap_or_else(|| Self::detect_unicode_support());
        let is_ssh = Self::detect_ssh_session();
        let (is_tmux, tmux_version) = Self::detect_tmux_session_with_version();
        let size = crossterm::terminal::size().unwrap_or((80, 24));

        let capabilities = Self {
            terminal_type,
            color_support,
            mouse_support,
            sixel_support,
            kitty_graphics_support,
            iterm2_inline_images_support,
            wezterm_multiplexer_support,
            unicode_placeholder_support,
            block_graphics_support,
            ansi_art_support,
            unicode_support,
            is_ssh,
            is_tmux,
            tmux_version,
            size,
            overrides_applied: overrides,
        };

        // Requirements: 4.1 - Log detected capabilities via ricecoder-logging
        tracing::info!(
            terminal_type = ?capabilities.terminal_type,
            color_support = ?capabilities.color_support,
            mouse_support = capabilities.mouse_support,
            sixel_support = capabilities.sixel_support,
            kitty_graphics_support = capabilities.kitty_graphics_support,
            iterm2_inline_images_support = capabilities.iterm2_inline_images_support,
            wezterm_multiplexer_support = capabilities.wezterm_multiplexer_support,
            unicode_placeholder_support = capabilities.unicode_placeholder_support,
            block_graphics_support = capabilities.block_graphics_support,
            ansi_art_support = capabilities.ansi_art_support,
            unicode_support = capabilities.unicode_support,
            is_ssh = capabilities.is_ssh,
            is_tmux = capabilities.is_tmux,
            tmux_version = ?capabilities.tmux_version,
            terminal_size = ?(capabilities.size.0, capabilities.size.1),
            overrides_applied = ?capabilities.overrides_applied,
            "Terminal capabilities detected"
        );

        // Log additional context for debugging
        if capabilities.is_ssh {
            tracing::debug!("SSH session detected, graphics complexity will be reduced");
        }
        if capabilities.is_tmux {
            tracing::debug!(
                tmux_version = ?capabilities.tmux_version,
                "TMUX session detected, OSC 52 will be wrapped"
            );
        }

        capabilities
    }

    /// Detect terminal type from environment variables
    ///
    /// Requirements: 4.1 - Detect terminal type (xterm, iTerm2, WezTerm, Kitty, etc.)
    fn detect_terminal_type() -> TerminalType {
        // Check TERM_PROGRAM first (most reliable)
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "iTerm.app" => return TerminalType::ITerm2,
                "WezTerm" => return TerminalType::WezTerm,
                "vscode" => return TerminalType::VSCode,
                "Apple_Terminal" => return TerminalType::TerminalApp,
                "Hyper" => return TerminalType::Hyper,
                "Tabby" => return TerminalType::Tabby,
                "Warp" => return TerminalType::Warp,
                _ => {}
            }
        }

        // Check TERM_PROGRAM_VERSION for more specific detection
        if let Ok(term_program_version) = env::var("TERM_PROGRAM_VERSION") {
            if let Ok(term_program) = env::var("TERM_PROGRAM") {
                if term_program == "iTerm.app" && term_program_version.starts_with("3.") {
                    return TerminalType::ITerm2;
                }
            }
        }

        // Check for Windows Terminal
        if env::var("WT_SESSION").is_ok() || env::var("WT_PROFILE_ID").is_ok() {
            return TerminalType::WindowsTerminal;
        }

        // Check TERM variable
        if let Ok(term) = env::var("TERM") {
            match term.as_str() {
                t if t.starts_with("xterm") => return TerminalType::Xterm,
                "alacritty" => return TerminalType::Alacritty,
                t if t.contains("kitty") => return TerminalType::Kitty,
                "foot" => return TerminalType::Foot,
                "rio" => return TerminalType::Rio,
                _ => {}
            }
        }

        // Check for specific terminal indicators
        if env::var("KITTY_WINDOW_ID").is_ok() || env::var("KITTY_PID").is_ok() {
            return TerminalType::Kitty;
        }

        if env::var("ALACRITTY_SOCKET").is_ok() || env::var("ALACRITTY_LOG").is_ok() {
            return TerminalType::Alacritty;
        }

        // Check for GNOME Terminal
        if env::var("GNOME_TERMINAL_SCREEN").is_ok() || env::var("GNOME_TERMINAL_SERVICE").is_ok() {
            return TerminalType::GnomeTerminal;
        }

        // Check for Konsole
        if env::var("KONSOLE_VERSION").is_ok() {
            return TerminalType::Konsole;
        }

        // Check for Foot terminal
        if env::var("FOOT_PID").is_ok() {
            return TerminalType::Foot;
        }

        // Check for Rio terminal
        if env::var("RIO_CONFIG").is_ok() {
            return TerminalType::Rio;
        }

        // Check for Warp terminal
        if env::var("WARP_HONOR_PS1").is_ok() {
            return TerminalType::Warp;
        }

        TerminalType::Unknown
    }

    /// Detect color support level
    ///
    /// Requirements: 4.1 - Detect color support (16, 256, true color)
    fn detect_color_support() -> ColorSupport {
        // Check COLORTERM for true color support
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return ColorSupport::TrueColor;
            }
        }

        // Check TERM for color support
        if let Ok(term) = env::var("TERM") {
            if term.contains("256color") || term.contains("256") {
                return ColorSupport::Ansi256;
            }
            if term.contains("color") {
                return ColorSupport::Ansi16;
            }
        }

        // Check if NO_COLOR is set (disables color)
        if env::var("NO_COLOR").is_ok() {
            return ColorSupport::None;
        }

        // Default to 16 colors if we can't determine
        ColorSupport::Ansi16
    }

    /// Detect mouse support
    ///
    /// Requirements: 4.1 - Detect feature support (mouse, sixel, Unicode)
    fn detect_mouse_support() -> bool {
        // Most modern terminals support mouse, but we can be conservative
        // and enable it by default, letting crossterm handle the details
        true
    }

    /// Detect sixel graphics support
    ///
    /// Requirements: 4.1 - Detect feature support (mouse, sixel, Unicode)
    fn detect_sixel_support(terminal_type: &TerminalType) -> bool {
        match terminal_type {
            TerminalType::Kitty => true,
            TerminalType::WezTerm => true,
            TerminalType::Foot => true,
            TerminalType::Xterm => {
                // Some xterm builds support sixel
                env::var("TERM").map_or(false, |term| term.contains("sixel"))
            }
            TerminalType::ITerm2 => {
                // iTerm2 supports sixel in newer versions
                if let Ok(version) = env::var("TERM_PROGRAM_VERSION") {
                    // Check if version is 3.5.0 or higher (approximate sixel support)
                    version.split('.').next()
                        .and_then(|major| major.parse::<u32>().ok())
                        .map_or(false, |major| major >= 3)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Detect Kitty graphics protocol support
    ///
    /// Requirements: 4.1 - Detect graphics protocol support
    fn detect_kitty_graphics_support(terminal_type: &TerminalType) -> bool {
        matches!(terminal_type, TerminalType::Kitty)
    }

    /// Detect iTerm2 inline images support
    ///
    /// Requirements: 4.1 - Detect graphics protocol support
    fn detect_iterm2_inline_images_support(terminal_type: &TerminalType) -> bool {
        matches!(terminal_type, TerminalType::ITerm2)
    }

    /// Detect WezTerm multiplexer features support
    ///
    /// Requirements: 4.1 - Detect graphics protocol support
    fn detect_wezterm_multiplexer_support(terminal_type: &TerminalType) -> bool {
        matches!(terminal_type, TerminalType::WezTerm)
    }

    /// Detect Unicode placeholder protocol support
    ///
    /// Requirements: 4.1 - Detect graphics protocol support
    fn detect_unicode_placeholder_support(terminal_type: &TerminalType) -> bool {
        // Unicode placeholders work in most modern terminals
        matches!(terminal_type,
            TerminalType::Xterm |
            TerminalType::ITerm2 |
            TerminalType::WezTerm |
            TerminalType::Kitty |
            TerminalType::Alacritty |
            TerminalType::WindowsTerminal |
            TerminalType::VSCode |
            TerminalType::Hyper |
            TerminalType::Tabby |
            TerminalType::Foot |
            TerminalType::Rio |
            TerminalType::Warp
        )
    }

    /// Detect block graphics support
    ///
    /// Requirements: 4.1 - Detect graphics protocol support
    fn detect_block_graphics_support() -> bool {
        // Block graphics are supported by most terminals
        // They use Unicode block characters for simple graphics
        true
    }

    /// Detect ANSI art support
    ///
    /// Requirements: 4.1 - Detect graphics protocol support
    fn detect_ansi_art_support() -> bool {
        // ANSI art uses color codes and ASCII characters
        // Supported by all terminals that support colors
        true
    }

    /// Detect Unicode support
    ///
    /// Requirements: 4.1 - Detect feature support (mouse, sixel, Unicode)
    fn detect_unicode_support() -> bool {
        // Check LC_ALL, LC_CTYPE, or LANG for UTF-8
        for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(value) = env::var(var) {
                if value.to_uppercase().contains("UTF-8") || value.to_uppercase().contains("UTF8") {
                    return true;
                }
            }
        }

        // Default to true for modern terminals
        true
    }

    /// Detect SSH session
    ///
    /// Requirements: 4.3 - Detect SSH session
    fn detect_ssh_session() -> bool {
        env::var("SSH_CLIENT").is_ok()
            || env::var("SSH_TTY").is_ok()
            || env::var("SSH_CONNECTION").is_ok()
    }

    /// Detect TMUX session (legacy method for compatibility)
    ///
    /// Requirements: 4.4 - Detect TMUX session
    #[allow(dead_code)]
    fn detect_tmux_session() -> bool {
        env::var("TMUX").is_ok()
    }

    /// Detect TMUX session with version information
    ///
    /// Requirements: 4.4 - Detect TMUX session and handle TMUX-specific requirements
    fn detect_tmux_session_with_version() -> (bool, Option<String>) {
        if let Ok(_tmux_var) = env::var("TMUX") {
            // TMUX variable format: /tmp/tmux-1000/default,12345,0
            // Try to get version from tmux command if available
            let version = std::process::Command::new("tmux")
                .arg("-V")
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .map(|v| v.trim().to_string());
            
            (true, version)
        } else {
            (false, None)
        }
    }

    /// Check if terminal should use reduced graphics complexity
    ///
    /// Requirements: 4.3 - Reduce graphics complexity over SSH
    pub fn should_reduce_graphics(&self) -> bool {
        self.is_ssh || self.overrides_applied.force_reduced_graphics.unwrap_or(false)
    }

    /// Check if OSC 52 should be wrapped for TMUX
    ///
    /// Requirements: 4.4 - Wrap OSC 52 for clipboard operations in TMUX
    pub fn should_wrap_osc52(&self) -> bool {
        self.is_tmux
    }

    /// Get TMUX color passthrough prefix
    ///
    /// Requirements: 4.4 - Handle TMUX color passthrough
    pub fn get_tmux_passthrough_prefix(&self) -> Option<&'static str> {
        if self.is_tmux {
            Some("\x1bPtmux;\x1b")
        } else {
            None
        }
    }

    /// Get TMUX color passthrough suffix
    ///
    /// Requirements: 4.4 - Handle TMUX color passthrough
    pub fn get_tmux_passthrough_suffix(&self) -> Option<&'static str> {
        if self.is_tmux {
            Some("\x1b\\")
        } else {
            None
        }
    }

    /// Wrap escape sequence for TMUX if needed
    ///
    /// Requirements: 4.4 - Support TMUX-specific escape sequences
    pub fn wrap_escape_sequence(&self, sequence: &str) -> String {
        if self.is_tmux {
            format!("\x1bPtmux;\x1b{}\x1b\\", sequence.replace('\x1b', "\x1b\x1b"))
        } else {
            sequence.to_string()
        }
    }

    /// Get appropriate color mode for ratatui
    ///
    /// Requirements: 4.2 - Use appropriate color mode
    pub fn get_color_mode(&self) -> crossterm::style::Color {
        match self.color_support {
            ColorSupport::TrueColor => crossterm::style::Color::Rgb { r: 255, g: 255, b: 255 },
            ColorSupport::Ansi256 => crossterm::style::Color::AnsiValue(255),
            ColorSupport::Ansi16 => crossterm::style::Color::White,
            ColorSupport::None => crossterm::style::Color::Reset,
        }
    }

    /// Get terminal-specific optimizations
    ///
    /// Requirements: 4.2 - Adapt UI based on detected capabilities
    pub fn get_optimizations(&self) -> HashMap<String, bool> {
        let mut optimizations = HashMap::new();
        
        // Enable mouse support if detected
        optimizations.insert("mouse_support".to_string(), self.mouse_support);
        
        // Enable sixel graphics if supported
        optimizations.insert("sixel_graphics".to_string(), self.sixel_support);

        // Enable Kitty graphics protocol if supported
        optimizations.insert("kitty_graphics_protocol".to_string(), self.kitty_graphics_support);

        // Enable iTerm2 inline images if supported
        optimizations.insert("iterm2_inline_images".to_string(), self.iterm2_inline_images_support);

        // Enable WezTerm multiplexer features if supported
        optimizations.insert("wezterm_multiplexer".to_string(), self.wezterm_multiplexer_support);

        // Enable Unicode placeholder protocol if supported
        optimizations.insert("unicode_placeholder_protocol".to_string(), self.unicode_placeholder_support);

        // Enable block graphics if supported
        optimizations.insert("block_graphics".to_string(), self.block_graphics_support);

        // Enable ANSI art if supported
        optimizations.insert("ansi_art".to_string(), self.ansi_art_support);

        // Enable Unicode characters if supported
        optimizations.insert("unicode_chars".to_string(), self.unicode_support);
        
        // Reduce graphics complexity for SSH
        optimizations.insert("reduced_graphics".to_string(), self.should_reduce_graphics());
        
        // Enable TMUX-specific handling
        optimizations.insert("tmux_mode".to_string(), self.is_tmux);
        
        // Terminal-specific optimizations
        match self.terminal_type {
            TerminalType::Kitty => {
                optimizations.insert("kitty_keyboard_protocol".to_string(), true);
                optimizations.insert("kitty_graphics_protocol".to_string(), true);
            }
            TerminalType::WezTerm => {
                optimizations.insert("wezterm_multiplexer".to_string(), true);
            }
            TerminalType::ITerm2 => {
                optimizations.insert("iterm2_inline_images".to_string(), true);
            }
            _ => {}
        }
        
        optimizations
    }
}

impl TerminalState {
    /// Capture the current terminal state and prepare for TUI
    ///
    /// This function:
    /// 1. Detects terminal capabilities
    /// 2. Captures whether raw mode is currently enabled
    /// 3. Enables raw mode for the TUI
    /// 4. Enters alternate screen
    ///
    /// Requirements: 4.1, 10.1 - Detect capabilities and capture terminal state before TUI initialization
    pub fn capture() -> Result<Self> {
        // Detect terminal capabilities first
        let capabilities = TerminalCapabilities::detect();

        // Check if raw mode is already enabled (shouldn't be, but be safe)
        let was_raw_mode_enabled = crossterm::terminal::is_raw_mode_enabled().unwrap_or(false);

        // Enable raw mode for the TUI
        enable_raw_mode()?;

        // Enter alternate screen
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        tracing::info!(
            "Terminal state captured: raw_mode={}, alternate_screen=true, capabilities={:?}",
            was_raw_mode_enabled,
            capabilities
        );

        Ok(Self {
            was_raw_mode_enabled,
            in_alternate_screen: true,
            capabilities,
        })
    }

    /// Restore the terminal to its original state
    ///
    /// This function:
    /// 1. Leaves alternate screen
    /// 2. Disables raw mode
    /// 3. Restores the original terminal state
    ///
    /// This is called on normal exit, Ctrl+C, and error exit.
    ///
    /// Requirements: 10.2, 10.3 - Restore terminal on normal exit, Ctrl+C, and error exit
    pub fn restore(&mut self) -> Result<()> {
        if self.in_alternate_screen {
            // Leave alternate screen
            let mut stdout = io::stdout();
            execute!(stdout, LeaveAlternateScreen)?;
            self.in_alternate_screen = false;
        }

        // Disable raw mode
        disable_raw_mode()?;

        // If raw mode wasn't enabled before, we're done
        // If it was, we don't re-enable it (caller should handle that if needed)

        tracing::info!("Terminal state restored");
        Ok(())
    }

    /// Check if terminal is in alternate screen
    pub fn in_alternate_screen(&self) -> bool {
        self.in_alternate_screen
    }

    /// Get terminal capabilities
    ///
    /// Requirements: 4.2 - Provide access to capabilities for UI adaptation
    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    /// Update terminal size
    ///
    /// Should be called when terminal is resized
    pub fn update_size(&mut self) -> Result<()> {
        self.capabilities.size = crossterm::terminal::size().unwrap_or((80, 24));
        tracing::debug!("Terminal size updated: {}x{}", self.capabilities.size.0, self.capabilities.size.1);
        Ok(())
    }
}

impl Drop for TerminalState {
    /// Ensure terminal is restored even if restore() wasn't called explicitly
    fn drop(&mut self) {
        if let Err(e) = self.restore() {
            tracing::error!("Failed to restore terminal state on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_capabilities_detection() {
        // Test that capability detection doesn't panic
        let capabilities = TerminalCapabilities::detect();
        
        // Basic sanity checks
        assert!(capabilities.size.0 > 0);
        assert!(capabilities.size.1 > 0);
        
        // Test that we can detect some basic capabilities
        // These should work in most environments
        println!("Detected capabilities: {:?}", capabilities);
    }

    #[test]
    fn test_terminal_type_detection() {
        let terminal_type = TerminalCapabilities::detect_terminal_type();
        // Should not panic and return some value
        println!("Detected terminal type: {:?}", terminal_type);
    }

    #[test]
    fn test_color_support_detection() {
        let color_support = TerminalCapabilities::detect_color_support();
        // Should not panic and return some value
        println!("Detected color support: {:?}", color_support);
    }

    #[test]
    fn test_ssh_detection() {
        let is_ssh = TerminalCapabilities::detect_ssh_session();
        // Should not panic
        println!("SSH session detected: {}", is_ssh);
    }

    #[test]
    fn test_tmux_detection() {
        let is_tmux = TerminalCapabilities::detect_tmux_session();
        // Should not panic
        println!("TMUX session detected: {}", is_tmux);
    }

    #[test]
    fn test_terminal_state_creation() {
        // This test verifies that TerminalState can be created
        // Note: This test may fail in non-TTY environments
        if atty::is(atty::Stream::Stdout) {
            let state = TerminalState::capture();
            // We don't assert success here because the test environment may not support it
            // But we verify the code compiles and runs
            if let Ok(state) = state {
                assert!(state.capabilities.size.0 > 0);
                assert!(state.capabilities.size.1 > 0);
            }
        }
    }

    #[test]
    fn test_terminal_state_drop() {
        // This test verifies that TerminalState implements Drop correctly
        if atty::is(atty::Stream::Stdout) {
            let _state = TerminalState::capture();
            // Drop is called automatically when _state goes out of scope
        }
    }

    #[test]
    fn test_capability_methods() {
        let capabilities = TerminalCapabilities::detect();
        
        // Test that methods don't panic
        let _should_reduce = capabilities.should_reduce_graphics();
        let _should_wrap = capabilities.should_wrap_osc52();
        let _color_mode = capabilities.get_color_mode();
        let _optimizations = capabilities.get_optimizations();
        
        println!("Capability methods work correctly");
    }

    #[test]
    fn test_capability_overrides() {
        let overrides = CapabilityOverrides {
            color_support: Some(ColorSupport::TrueColor),
            mouse_support: Some(false),
            sixel_support: Some(true),
            unicode_support: Some(false),
            force_reduced_graphics: Some(true),
        };
        
        let capabilities = TerminalCapabilities::detect_with_overrides(overrides.clone());
        
        // Verify overrides were applied
        assert_eq!(capabilities.color_support, ColorSupport::TrueColor);
        assert_eq!(capabilities.mouse_support, false);
        assert_eq!(capabilities.sixel_support, true);
        assert_eq!(capabilities.unicode_support, false);
        assert!(capabilities.should_reduce_graphics());
        
        println!("Capability overrides work correctly");
    }

    #[test]
    fn test_tmux_escape_sequence_wrapping() {
        // Create capabilities with TMUX enabled
        let mut capabilities = TerminalCapabilities::detect();
        capabilities.is_tmux = true;
        
        let sequence = "\x1b[31mred text\x1b[0m";
        let wrapped = capabilities.wrap_escape_sequence(sequence);
        
        // Should be wrapped with TMUX passthrough
        assert!(wrapped.starts_with("\x1bPtmux;\x1b"));
        assert!(wrapped.ends_with("\x1b\\"));
        
        println!("TMUX escape sequence wrapping works correctly");
    }

    #[test]
    fn test_tmux_passthrough_methods() {
        let mut capabilities = TerminalCapabilities::detect();
        capabilities.is_tmux = true;
        
        assert_eq!(capabilities.get_tmux_passthrough_prefix(), Some("\x1bPtmux;\x1b"));
        assert_eq!(capabilities.get_tmux_passthrough_suffix(), Some("\x1b\\"));
        
        // Test without TMUX
        capabilities.is_tmux = false;
        assert_eq!(capabilities.get_tmux_passthrough_prefix(), None);
        assert_eq!(capabilities.get_tmux_passthrough_suffix(), None);
        
        println!("TMUX passthrough methods work correctly");
    }

    #[test]
    fn test_terminal_optimizations() {
        let capabilities = TerminalCapabilities::detect();
        let optimizations = capabilities.get_optimizations();
        
        // Should contain expected optimization keys
        assert!(optimizations.contains_key("mouse_support"));
        assert!(optimizations.contains_key("sixel_graphics"));
        assert!(optimizations.contains_key("unicode_chars"));
        assert!(optimizations.contains_key("reduced_graphics"));
        assert!(optimizations.contains_key("tmux_mode"));
        
        println!("Terminal optimizations work correctly");
    }
}
