//! Banner rendering with SVG support and fallback options.

use std::path::Path;

use tracing::{debug, warn};
use usvg::TreeParsing;

use crate::error::{ImageError, ImageResult};

/// Banner renderer that supports multiple output formats.
pub struct BannerRenderer {
    cache: BannerCache,
}

/// Cache for rendered banners to improve performance.
pub struct BannerCache {
    svg_cache: Option<(String, Vec<u8>)>, // (svg_path, rendered_data)
}

/// Configuration for banner rendering.
#[derive(Debug, Clone)]
pub struct BannerConfig {
    pub enabled: bool,
    pub height: u16,
    pub svg_path: Option<std::path::PathBuf>,
    pub fallback_text: String,
}

impl Default for BannerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 7,
            svg_path: None,
            fallback_text: "RiceCoder".to_string(),
        }
    }
}

/// Terminal capabilities for banner rendering.
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub supports_sixel: bool,
    pub supports_unicode: bool,
    pub supports_color: bool,
    pub color_depth: ColorDepth,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorDepth {
    Monochrome,
    Color16,
    Color256,
    TrueColor,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self {
            supports_sixel: false,
            supports_unicode: true,
            supports_color: true,
            color_depth: ColorDepth::Color256,
        }
    }
}

/// Banner rendering output format.
#[derive(Debug, Clone)]
pub enum BannerOutput {
    Sixel(String),
    Unicode(String),
    Ascii(String),
    Text(String),
}

impl BannerRenderer {
    /// Create a new banner renderer.
    pub fn new() -> Self {
        Self {
            cache: BannerCache::new(),
        }
    }

    /// Render banner based on terminal capabilities.
    pub fn render(
        &mut self,
        config: &BannerConfig,
        capabilities: &TerminalCapabilities,
        theme_colors: Option<&ThemeColors>,
    ) -> ImageResult<BannerOutput> {
        if !config.enabled {
            return Ok(BannerOutput::Text(String::new()));
        }

        // Try SVG rendering first if path is provided
        if let Some(svg_path) = &config.svg_path {
            if svg_path.exists() {
                match self.render_svg(svg_path, config.height, capabilities, theme_colors) {
                    Ok(output) => return Ok(output),
                    Err(e) => {
                        warn!("SVG rendering failed: {}, falling back", e);
                    }
                }
            }
        }

        // Fall back to text banner
        Ok(BannerOutput::Text(
            self.render_text_banner(&config.fallback_text, theme_colors),
        ))
    }

    /// Render SVG to appropriate format based on capabilities.
    fn render_svg(
        &mut self,
        svg_path: &Path,
        height: u16,
        capabilities: &TerminalCapabilities,
        theme_colors: Option<&ThemeColors>,
    ) -> ImageResult<BannerOutput> {
        debug!("Rendering SVG banner from: {:?}", svg_path);

        // Load and parse SVG
        let svg_data = std::fs::read_to_string(svg_path)
            .map_err(|e| ImageError::BannerError(format!("Failed to read SVG file: {}", e)))?;

        // Check cache
        if let Some(cached) = self.cache.get_cached(&svg_data) {
            debug!("Using cached SVG render");
            return self.format_cached_output(cached, capabilities, theme_colors);
        }

        // Parse SVG
        let tree = usvg::Tree::from_str(&svg_data, &usvg::Options::default())
            .map_err(|e| ImageError::BannerError(format!("Failed to parse SVG: {}", e)))?;

        // Calculate dimensions maintaining aspect ratio
        let svg_size = tree.size;
        let aspect_ratio = svg_size.width() / svg_size.height();
        let target_width = (height as f32 * aspect_ratio * 2.0) as u32; // 2:1 char ratio
        let target_height = height as u32;

        // Render to pixel buffer
        let mut pixmap = tiny_skia::Pixmap::new(target_width, target_height)
            .ok_or_else(|| ImageError::BannerError("Failed to create pixmap".to_string()))?;

        let transform = tiny_skia::Transform::from_scale(
            target_width as f32 / svg_size.width(),
            target_height as f32 / svg_size.height(),
        );

        resvg::Tree::from_usvg(&tree).render(transform, &mut pixmap.as_mut());

        // Cache the rendered data
        let pixel_data = pixmap.data().to_vec();
        self.cache.cache_render(svg_data, pixel_data.clone());

        // Convert to appropriate output format
        self.convert_pixels_to_output(
            &pixel_data,
            target_width,
            target_height,
            capabilities,
            theme_colors,
        )
    }

    /// Convert pixel data to appropriate output format.
    fn convert_pixels_to_output(
        &self,
        pixel_data: &[u8],
        width: u32,
        height: u32,
        capabilities: &TerminalCapabilities,
        theme_colors: Option<&ThemeColors>,
    ) -> ImageResult<BannerOutput> {
        if capabilities.supports_sixel {
            self.convert_to_sixel(pixel_data, width, height)
        } else if capabilities.supports_unicode {
            self.convert_to_unicode(pixel_data, width, height, theme_colors)
        } else {
            self.convert_to_ascii(pixel_data, width, height)
        }
    }

    /// Convert pixel data to sixel format.
    fn convert_to_sixel(
        &self,
        _pixel_data: &[u8],
        width: u32,
        height: u32,
    ) -> ImageResult<BannerOutput> {
        // For now, return a placeholder sixel sequence
        // In a full implementation, this would convert RGBA data to sixel format
        debug!("Converting to sixel format: {}x{}", width, height);

        // Basic sixel header and footer
        let sixel_data = format!("\x1bPq\"1;1;{};{}\x1b\\", width, height);

        Ok(BannerOutput::Sixel(sixel_data))
    }

    /// Convert pixel data to Unicode block characters.
    fn convert_to_unicode(
        &self,
        pixel_data: &[u8],
        width: u32,
        height: u32,
        _theme_colors: Option<&ThemeColors>,
    ) -> ImageResult<BannerOutput> {
        debug!("Converting to Unicode art: {}x{}", width, height);

        let mut output = String::new();
        let char_width = (width / 2).max(1); // Use half-block characters
        let char_height = (height / 2).max(1);

        for y in 0..char_height {
            for x in 0..char_width {
                // Sample pixel at this position
                let pixel_x = (x * 2).min(width - 1);
                let pixel_y = (y * 2).min(height - 1);
                let pixel_idx = ((pixel_y * width + pixel_x) * 4) as usize;

                if pixel_idx + 3 < pixel_data.len() {
                    let alpha = pixel_data[pixel_idx + 3];

                    // Choose character based on alpha/brightness
                    let char = if alpha > 200 {
                        '█' // Full block
                    } else if alpha > 150 {
                        '▓' // Dark shade
                    } else if alpha > 100 {
                        '▒' // Medium shade
                    } else if alpha > 50 {
                        '░' // Light shade
                    } else {
                        ' ' // Transparent
                    };

                    output.push(char);
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        Ok(BannerOutput::Unicode(output))
    }

    /// Convert pixel data to ASCII characters.
    fn convert_to_ascii(
        &self,
        pixel_data: &[u8],
        width: u32,
        height: u32,
    ) -> ImageResult<BannerOutput> {
        debug!("Converting to ASCII art: {}x{}", width, height);

        let mut output = String::new();
        let char_width = (width / 8).max(1); // ASCII is lower resolution
        let char_height = (height / 16).max(1);

        for y in 0..char_height {
            for x in 0..char_width {
                // Sample pixel at this position
                let pixel_x = (x * 8).min(width - 1);
                let pixel_y = (y * 16).min(height - 1);
                let pixel_idx = ((pixel_y * width + pixel_x) * 4) as usize;

                if pixel_idx + 3 < pixel_data.len() {
                    let alpha = pixel_data[pixel_idx + 3];

                    // Choose ASCII character based on alpha/brightness
                    let char = if alpha > 200 {
                        '#'
                    } else if alpha > 150 {
                        '*'
                    } else if alpha > 100 {
                        '+'
                    } else if alpha > 50 {
                        '.'
                    } else {
                        ' '
                    };

                    output.push(char);
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        Ok(BannerOutput::Ascii(output))
    }

    /// Render text-only banner as fallback.
    fn render_text_banner(&self, text: &str, theme_colors: Option<&ThemeColors>) -> String {
        // Simple text banner with optional styling
        if let Some(_colors) = theme_colors {
            // In a full implementation, this would apply ANSI color codes
            format!("=== {} ===", text)
        } else {
            format!("=== {} ===", text)
        }
    }

    /// Format cached output based on capabilities.
    fn format_cached_output(
        &self,
        cached_data: &[u8],
        capabilities: &TerminalCapabilities,
        theme_colors: Option<&ThemeColors>,
    ) -> ImageResult<BannerOutput> {
        // For cached data, we need to know the original dimensions
        // This is a simplified implementation
        let width = 80; // Default width
        let height = 7; // Default height

        self.convert_pixels_to_output(cached_data, width, height, capabilities, theme_colors)
    }
}

impl Default for BannerRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl BannerCache {
    /// Create a new banner cache.
    pub fn new() -> Self {
        Self { svg_cache: None }
    }

    /// Get cached render if available.
    pub fn get_cached(&self, svg_data: &str) -> Option<&[u8]> {
        if let Some((cached_path, cached_data)) = &self.svg_cache {
            if cached_path == svg_data {
                return Some(cached_data);
            }
        }
        None
    }

    /// Cache a rendered banner.
    pub fn cache_render(&mut self, svg_data: String, rendered_data: Vec<u8>) {
        self.svg_cache = Some((svg_data, rendered_data));
    }

    /// Clear the cache.
    pub fn clear(&mut self) {
        self.svg_cache = None;
    }
}

/// Theme colors for banner styling.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub primary: (u8, u8, u8),
    pub secondary: (u8, u8, u8),
    pub accent: (u8, u8, u8),
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: (255, 255, 255),   // White
            secondary: (128, 128, 128), // Gray
            accent: (0, 255, 255),      // Cyan
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, path::PathBuf};

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_banner_config_default() {
        let config = BannerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.height, 7);
        assert_eq!(config.fallback_text, "RiceCoder");
    }

    #[test]
    fn test_terminal_capabilities_default() {
        let caps = TerminalCapabilities::default();
        assert!(!caps.supports_sixel);
        assert!(caps.supports_unicode);
        assert!(caps.supports_color);
    }

    #[test]
    fn test_banner_renderer_creation() {
        let renderer = BannerRenderer::new();
        assert!(renderer.cache.svg_cache.is_none());
    }

    #[test]
    fn test_render_disabled_banner() {
        let mut renderer = BannerRenderer::new();
        let config = BannerConfig {
            enabled: false,
            ..Default::default()
        };
        let caps = TerminalCapabilities::default();

        let result = renderer.render(&config, &caps, None).unwrap();
        match result {
            BannerOutput::Text(text) => assert!(text.is_empty()),
            _ => panic!("Expected empty text output for disabled banner"),
        }
    }

    #[test]
    fn test_render_text_fallback() {
        let mut renderer = BannerRenderer::new();
        let config = BannerConfig {
            enabled: true,
            svg_path: None,
            fallback_text: "Test".to_string(),
            ..Default::default()
        };
        let caps = TerminalCapabilities::default();

        let result = renderer.render(&config, &caps, None).unwrap();
        match result {
            BannerOutput::Text(text) => assert!(text.contains("Test")),
            _ => panic!("Expected text output for fallback"),
        }
    }

    #[test]
    fn test_render_with_nonexistent_svg() {
        let mut renderer = BannerRenderer::new();
        let config = BannerConfig {
            enabled: true,
            svg_path: Some(PathBuf::from("/nonexistent/file.svg")),
            fallback_text: "Fallback".to_string(),
            ..Default::default()
        };
        let caps = TerminalCapabilities::default();

        let result = renderer.render(&config, &caps, None).unwrap();
        match result {
            BannerOutput::Text(text) => assert!(text.contains("Fallback")),
            _ => panic!("Expected fallback text for nonexistent SVG"),
        }
    }

    #[test]
    fn test_render_with_valid_svg() {
        let mut renderer = BannerRenderer::new();

        // Create a temporary SVG file
        let mut temp_file = NamedTempFile::new().unwrap();
        let svg_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="100" height="50" xmlns="http://www.w3.org/2000/svg">
  <rect width="100" height="50" fill="blue"/>
  <text x="50" y="25" text-anchor="middle" fill="white">Test</text>
</svg>"#;
        temp_file.write_all(svg_content.as_bytes()).unwrap();

        let config = BannerConfig {
            enabled: true,
            svg_path: Some(temp_file.path().to_path_buf()),
            fallback_text: "Fallback".to_string(),
            height: 10,
        };
        let caps = TerminalCapabilities {
            supports_unicode: true,
            ..Default::default()
        };

        let result = renderer.render(&config, &caps, None).unwrap();
        match result {
            BannerOutput::Unicode(text) => {
                assert!(!text.is_empty());
                assert!(text.contains('\n')); // Should have multiple lines
            }
            _ => panic!("Expected Unicode output for valid SVG with Unicode support"),
        }
    }

    #[test]
    fn test_banner_cache() {
        let mut cache = BannerCache::new();
        assert!(cache.get_cached("test").is_none());

        cache.cache_render("test".to_string(), vec![1, 2, 3, 4]);
        assert!(cache.get_cached("test").is_some());
        assert_eq!(cache.get_cached("test").unwrap(), &[1, 2, 3, 4]);

        cache.clear();
        assert!(cache.get_cached("test").is_none());
    }

    #[test]
    fn test_theme_colors_default() {
        let colors = ThemeColors::default();
        assert_eq!(colors.primary, (255, 255, 255));
        assert_eq!(colors.secondary, (128, 128, 128));
        assert_eq!(colors.accent, (0, 255, 255));
    }
}
