//! Image widget for displaying images in the terminal
//!
//! This module provides a widget for displaying images with sixel support and Unicode fallback.

use std::path::PathBuf;

/// Image format
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

/// Image rendering mode
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
    /// Image path
    path: Option<PathBuf>,
    /// Image data (raw bytes)
    data: Option<Vec<u8>>,
    /// Image format
    format: Option<ImageFormat>,
    /// Image width
    width: u32,
    /// Image height
    height: u32,
    /// Rendering mode
    render_mode: RenderMode,
    /// Whether to maintain aspect ratio
    maintain_aspect_ratio: bool,
    /// Title for the widget
    title: String,
    /// Whether to show borders
    show_borders: bool,
    /// Whether image is loaded
    loaded: bool,
}

impl ImageWidget {
    /// Create a new image widget
    pub fn new() -> Self {
        Self {
            path: None,
            data: None,
            format: None,
            width: 0,
            height: 0,
            render_mode: RenderMode::UnicodeBlocks,
            maintain_aspect_ratio: true,
            title: "Image".to_string(),
            show_borders: true,
            loaded: false,
        }
    }

    /// Load an image from a file
    pub fn load_from_file(&mut self, path: impl Into<PathBuf>) -> Result<(), String> {
        let path = path.into();

        // Check if file exists
        if !path.exists() {
            return Err(format!("File not found: {}", path.display()));
        }

        // Detect format from extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| "No file extension".to_string())?;

        let format = ImageFormat::from_extension(ext)
            .ok_or_else(|| format!("Unsupported image format: {}", ext))?;

        // Read file
        let data = std::fs::read(&path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        self.path = Some(path);
        self.data = Some(data);
        self.format = Some(format);
        self.loaded = true;

        Ok(())
    }

    /// Load an image from raw data
    pub fn load_from_data(&mut self, data: Vec<u8>, format: ImageFormat) -> Result<(), String> {
        self.data = Some(data);
        self.format = Some(format);
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

    /// Get the image data
    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    /// Clear the image
    pub fn clear(&mut self) {
        self.path = None;
        self.data = None;
        self.format = None;
        self.width = 0;
        self.height = 0;
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

    /// Get the sixel representation (placeholder)
    pub fn get_sixel_data(&self) -> Option<String> {
        if !self.loaded || self.data.is_none() {
            return None;
        }

        // In a real implementation, this would convert the image data to sixel format
        // For now, we return a placeholder
        Some(format!(
            "Sixel data for {} ({}x{})",
            self.format.map(|f| f.name()).unwrap_or("unknown"),
            self.width,
            self.height
        ))
    }

    /// Get the Unicode block representation (placeholder)
    pub fn get_unicode_blocks(&self) -> Option<String> {
        if !self.loaded || self.data.is_none() {
            return None;
        }

        // In a real implementation, this would convert the image data to Unicode blocks
        // For now, we return a placeholder
        Some(format!(
            "Unicode blocks for {} ({}x{})",
            self.format.map(|f| f.name()).unwrap_or("unknown"),
            self.width,
            self.height
        ))
    }

    /// Get the ASCII art representation (placeholder)
    pub fn get_ascii_art(&self) -> Option<String> {
        if !self.loaded || self.data.is_none() {
            return None;
        }

        // In a real implementation, this would convert the image data to ASCII art
        // For now, we return a placeholder
        Some(format!(
            "ASCII art for {} ({}x{})",
            self.format.map(|f| f.name()).unwrap_or("unknown"),
            self.width,
            self.height
        ))
    }

    /// Get the rendered output based on render mode
    pub fn get_rendered_output(&self) -> Option<String> {
        match self.render_mode {
            RenderMode::Sixel => self.get_sixel_data(),
            RenderMode::UnicodeBlocks => self.get_unicode_blocks(),
            RenderMode::Ascii => self.get_ascii_art(),
        }
    }
}

impl Default for ImageWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("gif"), Some(ImageFormat::Gif));
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
        assert_eq!(ImageFormat::from_extension("svg"), Some(ImageFormat::Svg));
        assert_eq!(ImageFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_image_format_name() {
        assert_eq!(ImageFormat::Png.name(), "PNG");
        assert_eq!(ImageFormat::Jpeg.name(), "JPEG");
        assert_eq!(ImageFormat::Gif.name(), "GIF");
    }

    #[test]
    fn test_image_widget_creation() {
        let widget = ImageWidget::new();
        assert!(!widget.is_loaded());
        assert_eq!(widget.width(), 0);
        assert_eq!(widget.height(), 0);
    }

    #[test]
    fn test_image_widget_set_dimensions() {
        let mut widget = ImageWidget::new();
        widget.set_dimensions(800, 600);

        assert_eq!(widget.width(), 800);
        assert_eq!(widget.height(), 600);
    }

    #[test]
    fn test_image_widget_render_mode() {
        let mut widget = ImageWidget::new();
        assert_eq!(widget.render_mode(), RenderMode::UnicodeBlocks);

        widget.set_render_mode(RenderMode::Sixel);
        assert_eq!(widget.render_mode(), RenderMode::Sixel);
    }

    #[test]
    fn test_image_widget_aspect_ratio() {
        let mut widget = ImageWidget::new();
        assert!(widget.maintain_aspect_ratio());

        widget.set_maintain_aspect_ratio(false);
        assert!(!widget.maintain_aspect_ratio());
    }

    #[test]
    fn test_image_widget_title() {
        let mut widget = ImageWidget::new();
        assert_eq!(widget.title(), "Image");

        widget.set_title("My Image");
        assert_eq!(widget.title(), "My Image");
    }

    #[test]
    fn test_image_widget_borders() {
        let mut widget = ImageWidget::new();
        assert!(widget.show_borders());

        widget.set_show_borders(false);
        assert!(!widget.show_borders());
    }

    #[test]
    fn test_image_widget_load_from_data() {
        let mut widget = ImageWidget::new();
        let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header

        let result = widget.load_from_data(data, ImageFormat::Png);
        assert!(result.is_ok());
        assert!(widget.is_loaded());
        assert_eq!(widget.format(), Some(ImageFormat::Png));
    }

    #[test]
    fn test_image_widget_clear() {
        let mut widget = ImageWidget::new();
        widget.set_dimensions(800, 600);
        let _ = widget.load_from_data(vec![0x89, 0x50, 0x4E, 0x47], ImageFormat::Png);

        widget.clear();
        assert!(!widget.is_loaded());
        assert_eq!(widget.width(), 0);
        assert_eq!(widget.height(), 0);
    }

    #[test]
    fn test_image_widget_calculate_scaled_dimensions() {
        let mut widget = ImageWidget::new();
        widget.set_dimensions(1600, 1200);

        // Test scaling to fit in 800x600
        let (width, height) = widget.calculate_scaled_dimensions(800, 600);
        assert_eq!(width, 800);
        assert_eq!(height, 600);

        // Test with different aspect ratio
        widget.set_dimensions(800, 1200);
        let (width, height) = widget.calculate_scaled_dimensions(800, 600);
        assert_eq!(width, 400);
        assert_eq!(height, 600);
    }

    #[test]
    fn test_image_widget_display_text() {
        let mut widget = ImageWidget::new();
        assert_eq!(widget.get_display_text(), "[No image loaded]");

        widget.set_dimensions(800, 600);
        let _ = widget.load_from_data(vec![0x89, 0x50, 0x4E, 0x47], ImageFormat::Png);
        let text = widget.get_display_text();
        assert!(text.contains("PNG"));
        assert!(text.contains("800"));
        assert!(text.contains("600"));
    }

    #[test]
    fn test_image_widget_rendered_output() {
        let mut widget = ImageWidget::new();
        widget.set_dimensions(800, 600);
        let _ = widget.load_from_data(vec![0x89, 0x50, 0x4E, 0x47], ImageFormat::Png);

        widget.set_render_mode(RenderMode::UnicodeBlocks);
        assert!(widget.get_rendered_output().is_some());

        widget.set_render_mode(RenderMode::Sixel);
        assert!(widget.get_rendered_output().is_some());

        widget.set_render_mode(RenderMode::Ascii);
        assert!(widget.get_rendered_output().is_some());
    }
}
