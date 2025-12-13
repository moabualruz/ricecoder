//! Image widget for displaying images in the terminal
//!
//! This module provides a widget for displaying images using ratatui-image,
//! supporting multiple protocols: sixel, kitty, iTerm2, and unicode fallbacks.

use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};
use std::path::PathBuf;
use crate::terminal_state::TerminalCapabilities;

/// Image format (kept for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG format
    Png,
    /// JPEG format
    Jpeg,
    /// GIF format
    Gif,
    /// WebP format
    WebP,
    /// SVG format
    Svg,
}

impl ImageFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "gif" => Some(ImageFormat::Gif),
            "webp" => Some(ImageFormat::WebP),
            "svg" => Some(ImageFormat::Svg),
            _ => None,
        }
    }

    /// Get the format name
    pub fn name(&self) -> &'static str {
        match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::Gif => "GIF",
            ImageFormat::WebP => "WebP",
            ImageFormat::Svg => "SVG",
        }
    }
}

/// Image rendering mode (kept for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Sixel format (high quality, limited terminal support)
    Sixel,
    /// Unicode block characters (good quality, wide support)
    UnicodeBlocks,
    /// ASCII art (basic, universal support)
    Ascii,
}

/// Image widget for displaying images
pub struct ImageWidget {
    /// ratatui-image picker for protocol detection
    picker: Option<Picker>,
    /// ratatui-image stateful protocol for rendering
    protocol: Option<StatefulProtocol>,
    /// Image path (for backward compatibility)
    path: Option<PathBuf>,
    /// Image format (for backward compatibility)
    format: Option<ImageFormat>,
    /// Image width
    width: u32,
    /// Image height
    height: u32,
    /// Rendering mode (for backward compatibility)
    render_mode: RenderMode,
    /// Whether to maintain aspect ratio
    maintain_aspect_ratio: bool,
    /// Title for the widget
    title: String,
    /// Whether to show borders
    show_borders: bool,
    /// Whether image is loaded
    loaded: bool,
    /// Terminal capabilities for protocol selection
    capabilities: TerminalCapabilities,
}

impl ImageWidget {
    /// Create a new image widget with terminal capabilities
    pub fn new(capabilities: &TerminalCapabilities) -> Self {
        Self {
            picker: None,
            protocol: None,
            path: None,
            format: None,
            width: 0,
            height: 0,
            render_mode: Self::select_render_mode(capabilities),
            maintain_aspect_ratio: true,
            title: "Image".to_string(),
            show_borders: true,
            loaded: false,
            capabilities: capabilities.clone(),
        }
    }

    /// Select the best render mode based on terminal capabilities
    fn select_render_mode(capabilities: &TerminalCapabilities) -> RenderMode {
        if capabilities.sixel_support {
            RenderMode::Sixel
        } else {
            RenderMode::UnicodeBlocks
        }
    }

    /// Load an image from a file
    pub fn load_from_file(&mut self, path: impl Into<PathBuf>) -> Result<(), String> {
        let path = path.into();

        // Check if file exists
        if !path.exists() {
            return Err(format!("File not found: {}", path.display()));
        }

        // Detect format from extension (for backward compatibility)
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| "No file extension".to_string())?;

        let format = ImageFormat::from_extension(ext)
            .ok_or_else(|| format!("Unsupported image format: {}", ext))?;

        // Initialize ratatui-image picker if not already done
        if self.picker.is_none() {
            // Use font size based on terminal capabilities or defaults
            let font_size = (7, 16); // Default font size, could be made configurable
            self.picker = Some(Picker::from_fontsize(font_size));
        }

        // Load image using ratatui-image
        let dyn_img = image::ImageReader::open(&path)
            .map_err(|e| format!("Failed to open image: {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image: {}", e))?;

        // Create stateful protocol with automatic protocol detection
        let protocol = self.picker.as_ref().unwrap()
            .new_resize_protocol(dyn_img);

        self.path = Some(path);
        self.format = Some(format);
        self.protocol = Some(protocol);
        self.loaded = true;

        Ok(())
    }

    /// Load an image from raw data
    pub fn load_from_data(&mut self, data: Vec<u8>, format: ImageFormat) -> Result<(), String> {
        // Initialize ratatui-image picker if not already done
        if self.picker.is_none() {
            // Use font size based on terminal capabilities or defaults
            let font_size = (7, 16); // Default font size, could be made configurable
            self.picker = Some(Picker::from_fontsize(font_size));
        }

        // Load image from memory using ratatui-image
        let dyn_img = image::load_from_memory(&data)
            .map_err(|e| format!("Failed to decode image data: {}", e))?;

        // Create stateful protocol with automatic protocol detection
        let protocol = self.picker.as_ref().unwrap()
            .new_resize_protocol(dyn_img);

        self.format = Some(format);
        self.protocol = Some(protocol);
        self.loaded = true;

        Ok(())
    }

    /// Set the image dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Get the image width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the image height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Set the rendering mode
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
    }

    /// Get the rendering mode
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// Set whether to maintain aspect ratio
    pub fn set_maintain_aspect_ratio(&mut self, maintain: bool) {
        self.maintain_aspect_ratio = maintain;
    }

    /// Check if aspect ratio is maintained
    pub fn maintain_aspect_ratio(&self) -> bool {
        self.maintain_aspect_ratio
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Get the title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set whether to show borders
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Check if borders are shown
    pub fn show_borders(&self) -> bool {
        self.show_borders
    }

    /// Check if image is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get the image format
    pub fn format(&self) -> Option<ImageFormat> {
        self.format
    }

    /// Get the image path
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// Get the image data (for backward compatibility - returns None as data is now managed by ratatui-image)
    pub fn data(&self) -> Option<&[u8]> {
        None // Data is now managed internally by ratatui-image
    }

    /// Clear the image
    pub fn clear(&mut self) {
        self.path = None;
        self.format = None;
        self.width = 0;
        self.height = 0;
        self.protocol = None;
        self.loaded = false;
    }

    /// Add multiple images (for batch loading)
    pub fn add_images(&mut self, _paths: Vec<PathBuf>) {
        // In a real implementation, this would load multiple images
        // For now, we just clear and load the first one if available
        self.clear();
    }

    /// Calculate scaled dimensions to fit in a box
    pub fn calculate_scaled_dimensions(&self, max_width: u32, max_height: u32) -> (u32, u32) {
        if self.width == 0 || self.height == 0 {
            return (max_width, max_height);
        }

        if !self.maintain_aspect_ratio {
            return (max_width, max_height);
        }

        let aspect_ratio = self.width as f32 / self.height as f32;
        let max_aspect_ratio = max_width as f32 / max_height as f32;

        if aspect_ratio > max_aspect_ratio {
            // Width is the limiting factor
            let new_width = max_width;
            let new_height = (max_width as f32 / aspect_ratio) as u32;
            (new_width, new_height)
        } else {
            // Height is the limiting factor
            let new_height = max_height;
            let new_width = (max_height as f32 * aspect_ratio) as u32;
            (new_width, new_height)
        }
    }

    /// Get the display text (for when image can't be rendered)
    pub fn get_display_text(&self) -> String {
        if !self.loaded {
            return "[No image loaded]".to_string();
        }

        match self.format {
            Some(format) => {
                format!(
                    "[Image: {} ({}x{})]",
                    format.name(),
                    self.width,
                    self.height
                )
            }
            None => "[Image: Unknown format]".to_string(),
        }
    }

    /// Get the sixel representation (for backward compatibility - now handled by ratatui-image)
    pub fn get_sixel_data(&self) -> Option<String> {
        if !self.loaded {
            return None;
        }

        // With ratatui-image, sixel rendering is handled automatically
        Some(format!(
            "Sixel rendering handled by ratatui-image for {} ({}x{})",
            self.format.map(|f| f.name()).unwrap_or("unknown"),
            self.width,
            self.height
        ))
    }

    /// Get the Unicode block representation (for backward compatibility - now handled by ratatui-image)
    pub fn get_unicode_blocks(&self) -> Option<String> {
        if !self.loaded {
            return None;
        }

        // With ratatui-image, unicode rendering is handled automatically
        Some(format!(
            "Unicode rendering handled by ratatui-image for {} ({}x{})",
            self.format.map(|f| f.name()).unwrap_or("unknown"),
            self.width,
            self.height
        ))
    }

    /// Get the ASCII art representation (for backward compatibility - now handled by ratatui-image)
    pub fn get_ascii_art(&self) -> Option<String> {
        if !self.loaded {
            return None;
        }

        // With ratatui-image, ASCII fallback is handled automatically
        Some(format!(
            "ASCII fallback handled by ratatui-image for {} ({}x{})",
            self.format.map(|f| f.name()).unwrap_or("unknown"),
            self.width,
            self.height
        ))
    }

    /// Get the ratatui-image widget for rendering
    pub fn widget(&self) -> StatefulImage<StatefulProtocol> {
        StatefulImage::default()
    }

    /// Get the protocol for rendering
    pub fn protocol(&self) -> Option<&StatefulProtocol> {
        self.protocol.as_ref()
    }

    /// Get mutable protocol for rendering
    pub fn protocol_mut(&mut self) -> Option<&mut StatefulProtocol> {
        self.protocol.as_mut()
    }

    /// Get the rendered output based on render mode (for backward compatibility)
    pub fn get_rendered_output(&self) -> Option<String> {
        if !self.loaded {
            return None;
        }

        // With ratatui-image, rendering is handled by the widget itself
        // Return a placeholder for backward compatibility
        Some(format!(
            "Image loaded: {} ({}x{}) - rendered via ratatui-image",
            self.format.map(|f| f.name()).unwrap_or("unknown"),
            self.width,
            self.height
        ))
    }
}

impl Default for ImageWidget {
    fn default() -> Self {
        // Create default capabilities for backward compatibility
        let capabilities = TerminalCapabilities::detect();
        Self::new(&capabilities)
    }
}


