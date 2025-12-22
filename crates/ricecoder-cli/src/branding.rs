// Branding and visual identity
// Loads and displays RiceCoder branding

use std::path::Path;

use crate::{error::CliResult, output::OutputStyle};

/// Branding manager
pub struct BrandingManager;

impl BrandingManager {
    /// Load ASCII logo from file
    pub fn load_ascii_logo() -> CliResult<String> {
        // Check for branding file in multiple locations
        let possible_paths = vec![
            ".branding/Ascii1.txt",
            "projects/ricecoder/.branding/Ascii1.txt",
            "/projects/ricecoder/.branding/Ascii1.txt",
        ];

        for path in possible_paths {
            if Path::new(path).exists() {
                match std::fs::read_to_string(path) {
                    Ok(content) => return Ok(content),
                    Err(_) => continue,
                }
            }
        }

        // Fallback to default ASCII art
        Ok(Self::default_ascii_logo())
    }

    /// Get default ASCII logo
    pub fn default_ascii_logo() -> String {
        r#"
  ╔═══════════════════════════════════╗
  ║                                   ║
  ║        🍚 RiceCoder 🍚           ║
  ║                                   ║
  ║   Terminal-first, Spec-driven    ║
  ║      Coding Assistant            ║
  ║                                   ║
  ╚═══════════════════════════════════╝
"#
        .to_string()
    }

    /// Display branding on startup
    pub fn display_startup_banner() -> CliResult<()> {
        let style = OutputStyle::default();
        let logo = Self::load_ascii_logo()?;

        println!("{}", logo);
        println!("{}", style.info("Type 'rice --help' for usage information"));
        println!();

        Ok(())
    }

    /// Display branding on version command
    pub fn display_version_banner(version: &str) -> CliResult<()> {
        let style = OutputStyle::default();
        let logo = Self::load_ascii_logo()?;

        println!("{}", logo);
        println!("{}", style.header(&format!("Version: {}", version)));
        println!();

        Ok(())
    }

    /// Detect terminal capabilities
    pub fn detect_terminal_capabilities() -> TerminalCapabilities {
        TerminalCapabilities {
            supports_colors: atty::is(atty::Stream::Stdout),
            supports_unicode: Self::supports_unicode(),
            supports_images: Self::supports_images(),
            width: Self::get_terminal_width(),
            height: Self::get_terminal_height(),
        }
    }

    /// Check if terminal supports Unicode
    pub fn supports_unicode() -> bool {
        // Check environment variables
        if let Ok(lang) = std::env::var("LANG") {
            return lang.contains("UTF-8") || lang.contains("utf8");
        }
        if let Ok(lc_all) = std::env::var("LC_ALL") {
            return lc_all.contains("UTF-8") || lc_all.contains("utf8");
        }
        // Default to true on most modern terminals
        true
    }

    /// Check if terminal supports images
    pub fn supports_images() -> bool {
        // Check for Kitty, iTerm2, or other image-capable terminals
        if let Ok(term) = std::env::var("TERM") {
            return term.contains("kitty") || term.contains("iterm");
        }
        false
    }

    /// Get terminal width
    pub fn get_terminal_width() -> u16 {
        term_size::dimensions().map(|(w, _)| w as u16).unwrap_or(80)
    }

    /// Get terminal height
    pub fn get_terminal_height() -> u16 {
        term_size::dimensions().map(|(_, h)| h as u16).unwrap_or(24)
    }
}

/// Terminal capabilities
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub supports_colors: bool,
    pub supports_unicode: bool,
    pub supports_images: bool,
    pub width: u16,
    pub height: u16,
}
