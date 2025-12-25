//! Banner component widget for ricecoder-tui.

use std::path::PathBuf;

use ratatui::{
    buffer::Buffer,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use ricecoder_images::{
    BannerConfig, BannerOutput, BannerRenderer, ColorDepth, TerminalCapabilities,
    ThemeColors as ImageThemeColors,
};
use tracing::{debug, warn};

use crate::layout::Rect as LayoutRect;

/// Banner size variants for adaptive rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerSize {
    /// Text-only tagline: `r[ Plan. Think. Code` - 1 line
    Text,
    /// Compact logo (uses Ascii1.txt) - approximately 18 lines
    Compact,
    /// Medium banner (uses Ascii3.txt) - approximately 31 lines
    Medium,
    /// Full banner (uses Ascii2.txt) - approximately 23 lines
    Full,
}

impl BannerSize {
    /// Get the compact text tagline for minimal space
    pub const TAGLINE: &'static str = "r[ Plan. Think. Code";
    
    /// Get minimum height required for this size
    pub fn min_height(&self) -> u16 {
        match self {
            BannerSize::Text => 1,
            BannerSize::Compact => 18,
            BannerSize::Medium => 31,
            BannerSize::Full => 23,
        }
    }
    
    /// Select appropriate size based on available height
    pub fn from_available_height(height: u16) -> Self {
        if height >= 31 {
            BannerSize::Medium
        } else if height >= 23 {
            BannerSize::Full
        } else if height >= 18 {
            BannerSize::Compact
        } else {
            BannerSize::Text
        }
    }
}

/// Banner component widget that integrates with the ricecoder-images banner renderer.
pub struct BannerComponent {
    renderer: BannerRenderer,
    config: BannerConfig,
    cached_output: Option<String>,
    theme_colors: Option<ImageThemeColors>,
    ascii_fallbacks: AsciiFallbacks,
    size: BannerSize,
}

/// ASCII art fallback content for different sizes.
#[derive(Debug, Clone)]
struct AsciiFallbacks {
    compact: Option<String>,
    medium: Option<String>,
    full: Option<String>,
}

impl AsciiFallbacks {
    /// Load ASCII art fallbacks from branding directory.
    fn load(branding_dir: &std::path::Path) -> Self {
        Self {
            compact: Self::load_file(branding_dir.join("Ascii1.txt")),
            medium: Self::load_file(branding_dir.join("Ascii3.txt")),
            full: Self::load_file(branding_dir.join("Ascii2.txt")),
        }
    }

    fn load_file(path: std::path::PathBuf) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn get(&self, size: BannerSize) -> Option<&str> {
        match size {
            BannerSize::Text => Some(BannerSize::TAGLINE),
            BannerSize::Compact => self.compact.as_deref(),
            BannerSize::Medium => self.medium.as_deref(),
            BannerSize::Full => self.full.as_deref(),
        }
    }
}

/// Configuration for the banner component.
#[derive(Debug, Clone)]
pub struct BannerComponentConfig {
    pub enabled: bool,
    pub height: u16,
    pub svg_path: Option<PathBuf>,
    pub fallback_text: String,
    pub show_border: bool,
    pub border_style: Style,
    pub branding_dir: Option<PathBuf>,
    pub size: BannerSize,
}

impl Default for BannerComponentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 20, // Default to medium size
            svg_path: None,
            fallback_text: "RiceCoder".to_string(),
            show_border: true,
            border_style: Style::default().fg(Color::Cyan),
            branding_dir: None,
            size: BannerSize::Medium,
        }
    }
}

impl BannerComponent {
    /// Create a new banner component.
    pub fn new(config: BannerComponentConfig) -> Self {
        let banner_config = BannerConfig {
            enabled: config.enabled,
            height: config.height,
            svg_path: config.svg_path.clone(),
            fallback_text: config.fallback_text.clone(),
        };

        // Load ASCII fallbacks from branding directory
        let ascii_fallbacks = if let Some(ref branding_dir) = config.branding_dir {
            AsciiFallbacks::load(branding_dir)
        } else {
            // Try crate's assets directory first (compile-time path)
            let crate_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/branding");
            if crate_assets.exists() {
                AsciiFallbacks::load(&crate_assets)
            } else {
                // Fallback: try relative to working directory
                let fallback_branding = PathBuf::from("assets/branding");
                if fallback_branding.exists() {
                    AsciiFallbacks::load(&fallback_branding)
                } else {
                    AsciiFallbacks {
                        compact: None,
                        medium: None,
                        full: None,
                    }
                }
            }
        };

        Self {
            renderer: BannerRenderer::new(),
            config: banner_config,
            cached_output: None,
            theme_colors: None,
            ascii_fallbacks,
            size: config.size,
        }
    }

    /// Update the banner configuration.
    pub fn update_config(&mut self, config: BannerComponentConfig) {
        self.config = BannerConfig {
            enabled: config.enabled,
            height: config.height,
            svg_path: config.svg_path.clone(),
            fallback_text: config.fallback_text.clone(),
        };
        self.size = config.size;
        
        // Reload ASCII fallbacks if directory changed
        if let Some(ref branding_dir) = config.branding_dir {
            self.ascii_fallbacks = AsciiFallbacks::load(branding_dir);
        }
        
        // Clear cache when config changes
        self.cached_output = None;
    }

    /// Set theme colors for banner rendering.
    pub fn set_theme_colors(&mut self, colors: ImageThemeColors) {
        self.theme_colors = Some(colors);
        // Clear cache when theme changes
        self.cached_output = None;
    }

    /// Clear the cached banner output.
    pub fn clear_cache(&mut self) {
        self.cached_output = None;
    }

    /// Detect terminal capabilities for banner rendering.
    fn detect_terminal_capabilities(&self) -> TerminalCapabilities {
        // For now, use conservative defaults
        // In a full implementation, this would query the terminal
        TerminalCapabilities {
            supports_sixel: false,             // Most terminals don't support sixel
            supports_unicode: true,            // Most modern terminals support Unicode
            supports_color: true,              // Most terminals support color
            color_depth: ColorDepth::Color256, // Common color depth
        }
    }

    /// Render the banner and return the output string.
    pub fn render_banner(&mut self) -> String {
        if !self.config.enabled {
            return String::new();
        }

        // Use cached output if available
        if let Some(cached) = &self.cached_output {
            return cached.clone();
        }

        let capabilities = self.detect_terminal_capabilities();

        match self
            .renderer
            .render(&self.config, &capabilities, self.theme_colors.as_ref())
        {
            Ok(output) => {
                let rendered = match output {
                    BannerOutput::Sixel(data) => data,
                    BannerOutput::Unicode(data) => data,
                    BannerOutput::Ascii(data) => data,
                    BannerOutput::Text(_) => {
                        // Try ASCII fallback before plain text
                        if let Some(ascii) = self.ascii_fallbacks.get(self.size) {
                            ascii.to_string()
                        } else {
                            format!("=== {} ===", self.config.fallback_text)
                        }
                    }
                };

                // Cache the output
                self.cached_output = Some(rendered.clone());
                rendered
            }
            Err(e) => {
                warn!("Banner rendering failed: {}, trying ASCII fallback", e);
                
                // Try ASCII fallback
                if let Some(ascii) = self.ascii_fallbacks.get(self.size) {
                    ascii.to_string()
                } else {
                    // Final fallback to simple text
                    format!("=== {} ===", self.config.fallback_text)
                }
            }
        }
    }

    /// Get the configured banner height.
    pub fn height(&self) -> u16 {
        if self.config.enabled {
            self.config.height
        } else {
            0
        }
    }

    /// Check if the banner is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable or disable the banner.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        if !enabled {
            self.cached_output = None;
        }
    }

    /// Set the SVG path for banner rendering.
    pub fn set_svg_path(&mut self, path: Option<PathBuf>) {
        self.config.svg_path = path;
        self.cached_output = None;
    }

    /// Set the fallback text.
    pub fn set_fallback_text(&mut self, text: String) {
        self.config.fallback_text = text;
        self.cached_output = None;
    }

    /// Set the banner size variant.
    pub fn set_size(&mut self, size: BannerSize) {
        self.size = size;
        // Adjust height based on size
        self.config.height = size.min_height();
        self.cached_output = None;
    }
    
    /// Set banner to compact text mode: `r[ Plan. Think. Code`
    pub fn set_text_mode(&mut self) {
        self.set_size(BannerSize::Text);
    }
    
    /// Auto-select size based on available terminal height
    pub fn auto_size(&mut self, available_height: u16) {
        self.set_size(BannerSize::from_available_height(available_height));
    }

    /// Get the current banner size.
    pub fn size(&self) -> BannerSize {
        self.size
    }

    /// Load ASCII fallbacks from a branding directory.
    pub fn load_ascii_fallbacks(&mut self, branding_dir: &std::path::Path) {
        self.ascii_fallbacks = AsciiFallbacks::load(branding_dir);
        self.cached_output = None;
    }
}

/// Widget implementation for BannerComponent.
impl Widget for &mut BannerComponent {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        if !self.config.enabled || area.height == 0 {
            return;
        }

        debug!("Rendering banner in area: {:?}", area);

        let banner_text = self.render_banner();

        // Use the area directly since it's already ratatui::layout::Rect
        let ratatui_area = area;

        // Create lines from the banner text
        let lines: Vec<Line> = banner_text
            .lines()
            .take(area.height as usize)
            .map(|line| {
                // Truncate line if it's too long
                let truncated = if line.len() > area.width as usize {
                    &line[..area.width as usize]
                } else {
                    line
                };
                Line::from(Span::styled(truncated, Style::default().fg(Color::Cyan)))
            })
            .collect();

        // Create paragraph widget
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("RiceCoder"))
            .style(Style::default().fg(Color::White));

        // Render the paragraph
        paragraph.render(ratatui_area, buf);
    }
}

/// Banner area calculation for layout integration.
pub struct BannerArea {
    component: BannerComponent,
    config: BannerComponentConfig,
}

impl BannerArea {
    /// Create a new banner area.
    pub fn new(config: BannerComponentConfig) -> Self {
        let component = BannerComponent::new(config.clone());
        Self { component, config }
    }

    /// Calculate the banner area within the given terminal area.
    pub fn calculate_area(&self, terminal_area: LayoutRect) -> Option<LayoutRect> {
        if !self.config.enabled {
            return None;
        }

        let height = self.config.height.min(terminal_area.height);
        if height == 0 {
            return None;
        }

        Some(LayoutRect::new(
            terminal_area.x,
            terminal_area.y,
            terminal_area.width,
            height,
        ))
    }

    /// Get the remaining area after banner.
    pub fn remaining_area(&self, terminal_area: LayoutRect) -> LayoutRect {
        if let Some(banner_area) = self.calculate_area(terminal_area) {
            LayoutRect::new(
                terminal_area.x,
                terminal_area.y + banner_area.height,
                terminal_area.width,
                terminal_area.height.saturating_sub(banner_area.height),
            )
        } else {
            terminal_area
        }
    }

    /// Get mutable reference to the banner component.
    pub fn component_mut(&mut self) -> &mut BannerComponent {
        &mut self.component
    }

    /// Get reference to the banner component.
    pub fn component(&self) -> &BannerComponent {
        &self.component
    }

    /// Update the configuration.
    pub fn update_config(&mut self, config: BannerComponentConfig) {
        self.component.update_config(config.clone());
        self.config = config;
    }
}
