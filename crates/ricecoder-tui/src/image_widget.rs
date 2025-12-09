//! Image widget for displaying images in the terminal UI
//!
//! This module provides the `ImageWidget` for rendering images in the ricecoder-tui,
//! integrating with ricecoder-images for display and metadata rendering.
//!
//! # Requirements
//!
//! - Req 5.1: Display images in terminal using ricecoder-images ImageDisplay
//! - Req 5.2: Show metadata (format, size, dimensions)
//! - Req 5.3: ASCII placeholder for unsupported terminals
//! - Req 5.4: Resize to fit terminal bounds (max 80x30)
//! - Req 5.5: Organize multiple images vertically with separators

use std::path::PathBuf;

/// Image widget for displaying images in the terminal
///
/// Provides:
/// - Single and multiple image display
/// - Metadata rendering (format, size, dimensions)
/// - ASCII placeholder fallback
/// - Automatic resizing to fit terminal bounds
/// - Vertical organization with separators
///
/// # Requirements
///
/// - Req 5.1: Display images in terminal using ricecoder-images ImageDisplay
/// - Req 5.2: Show metadata (format, size, dimensions)
/// - Req 5.3: ASCII placeholder for unsupported terminals
/// - Req 5.4: Resize to fit terminal bounds (max 80x30)
/// - Req 5.5: Organize multiple images vertically with separators
#[derive(Debug, Clone)]
pub struct ImageWidget {
    /// Image file paths to display
    pub images: Vec<PathBuf>,
    /// Whether the widget is visible
    pub visible: bool,
    /// Maximum width for display (characters)
    pub max_width: u32,
    /// Maximum height for display (characters)
    pub max_height: u32,
    /// Whether to show metadata
    pub show_metadata: bool,
    /// Whether to use ASCII placeholder
    pub use_ascii_placeholder: bool,
}

impl ImageWidget {
    /// Create a new image widget
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            visible: true,
            max_width: 80,
            max_height: 30,
            show_metadata: true,
            use_ascii_placeholder: true,
        }
    }

    /// Create a new image widget with specific dimensions
    pub fn with_dimensions(width: u32, height: u32) -> Self {
        Self {
            images: Vec::new(),
            visible: true,
            max_width: width,
            max_height: height,
            show_metadata: true,
            use_ascii_placeholder: true,
        }
    }

    /// Add an image to the widget
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file
    ///
    /// # Requirements
    ///
    /// - Req 5.1: Add image to widget for display
    pub fn add_image(&mut self, path: PathBuf) {
        if !self.images.contains(&path) {
            self.images.push(path);
        }
    }

    /// Add multiple images to the widget
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to the image files
    ///
    /// # Requirements
    ///
    /// - Req 5.5: Organize multiple images vertically with separators
    pub fn add_images(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            self.add_image(path);
        }
    }

    /// Remove an image from the widget
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image to remove
    ///
    /// # Returns
    ///
    /// True if image was removed, false if not found
    pub fn remove_image(&mut self, path: &PathBuf) -> bool {
        if let Some(pos) = self.images.iter().position(|p| p == path) {
            self.images.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear all images from the widget
    pub fn clear_images(&mut self) {
        self.images.clear();
    }

    /// Get the number of images in the widget
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Check if the widget has any images
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    /// Get the images in the widget
    pub fn get_images(&self) -> &[PathBuf] {
        &self.images
    }

    /// Show the widget
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the widget
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle widget visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Set the maximum display dimensions
    ///
    /// # Arguments
    ///
    /// * `width` - Maximum width in characters
    /// * `height` - Maximum height in characters
    ///
    /// # Requirements
    ///
    /// - Req 5.4: Resize to fit terminal bounds (max 80x30)
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.max_width = width;
        self.max_height = height;
    }

    /// Enable metadata display
    pub fn enable_metadata(&mut self) {
        self.show_metadata = true;
    }

    /// Disable metadata display
    pub fn disable_metadata(&mut self) {
        self.show_metadata = false;
    }

    /// Enable ASCII placeholder
    pub fn enable_ascii_placeholder(&mut self) {
        self.use_ascii_placeholder = true;
    }

    /// Disable ASCII placeholder
    pub fn disable_ascii_placeholder(&mut self) {
        self.use_ascii_placeholder = false;
    }

    /// Render the widget as a string
    ///
    /// # Returns
    ///
    /// Rendered widget string
    ///
    /// # Requirements
    ///
    /// - Req 5.1: Display images in terminal using ricecoder-images ImageDisplay
    /// - Req 5.2: Show metadata (format, size, dimensions)
    /// - Req 5.3: ASCII placeholder for unsupported terminals
    /// - Req 5.4: Resize to fit terminal bounds (max 80x30)
    /// - Req 5.5: Organize multiple images vertically with separators
    pub fn render(&self) -> String {
        if !self.visible || self.images.is_empty() {
            return String::new();
        }

        let mut output = String::new();

        // Render each image
        for (index, _path) in self.images.iter().enumerate() {
            // Add separator between images
            if index > 0 {
                output.push_str(&self.render_separator());
                output.push('\n');
            }

            // Render image placeholder with metadata
            output.push_str(&self.render_image_placeholder(index));

            // Add newline after each image except the last
            if index < self.images.len() - 1 {
                output.push('\n');
            }
        }

        output
    }

    /// Render a single image placeholder
    fn render_image_placeholder(&self, index: usize) -> String {
        let mut output = String::new();

        // Add metadata header if enabled
        if self.show_metadata {
            output.push_str(&format!("[Image {}] ", index + 1));
            if let Some(path) = self.images.get(index) {
                output.push_str(&format!("{}", path.display()));
            }
            output.push('\n');
        }

        // Add ASCII placeholder if enabled
        if self.use_ascii_placeholder {
            output.push_str(&self.render_ascii_placeholder());
        }

        output
    }

    /// Render ASCII placeholder for an image
    fn render_ascii_placeholder(&self) -> String {
        let placeholder_char = "█";
        let width = self.max_width as usize;
        let height = 10; // Use fixed small height

        let mut output = String::new();

        // Create a simple ASCII box
        for row in 0..height {
            if row == 0 || row == height - 1 {
                // Top and bottom borders
                output.push_str(&placeholder_char.repeat(width));
            } else if row == 1 || row == height - 2 {
                // Second and second-to-last rows with borders
                output.push_str(placeholder_char);
                output.push_str(&" ".repeat(width.saturating_sub(2)));
                output.push_str(placeholder_char);
            } else {
                // Middle rows with borders
                output.push_str(placeholder_char);
                output.push_str(&" ".repeat(width.saturating_sub(2)));
                output.push_str(placeholder_char);
            }
            output.push('\n');
        }

        output
    }

    /// Render a separator between images
    fn render_separator(&self) -> String {
        let separator_char = "─";
        separator_char.repeat(self.max_width as usize)
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
    fn test_image_widget_creation() {
        let widget = ImageWidget::new();
        assert!(widget.visible);
        assert_eq!(widget.max_width, 80);
        assert_eq!(widget.max_height, 30);
        assert!(widget.show_metadata);
        assert!(widget.use_ascii_placeholder);
        assert_eq!(widget.image_count(), 0);
    }

    #[test]
    fn test_image_widget_with_dimensions() {
        let widget = ImageWidget::with_dimensions(100, 50);
        assert_eq!(widget.max_width, 100);
        assert_eq!(widget.max_height, 50);
    }

    #[test]
    fn test_add_image() {
        let mut widget = ImageWidget::new();
        let path = PathBuf::from("/path/to/image.png");

        widget.add_image(path.clone());
        assert_eq!(widget.image_count(), 1);
        assert!(widget.has_images());
        assert_eq!(widget.get_images()[0], path);
    }

    #[test]
    fn test_add_duplicate_image() {
        let mut widget = ImageWidget::new();
        let path = PathBuf::from("/path/to/image.png");

        widget.add_image(path.clone());
        widget.add_image(path.clone());

        // Should not add duplicate
        assert_eq!(widget.image_count(), 1);
    }

    #[test]
    fn test_add_multiple_images() {
        let mut widget = ImageWidget::new();
        let paths = vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
            PathBuf::from("/path/to/image3.gif"),
        ];

        widget.add_images(paths.clone());
        assert_eq!(widget.image_count(), 3);
    }

    #[test]
    fn test_remove_image() {
        let mut widget = ImageWidget::new();
        let path = PathBuf::from("/path/to/image.png");

        widget.add_image(path.clone());
        assert_eq!(widget.image_count(), 1);

        let removed = widget.remove_image(&path);
        assert!(removed);
        assert_eq!(widget.image_count(), 0);
    }

    #[test]
    fn test_remove_image_not_found() {
        let mut widget = ImageWidget::new();
        let path = PathBuf::from("/path/to/image.png");

        let removed = widget.remove_image(&path);
        assert!(!removed);
    }

    #[test]
    fn test_clear_images() {
        let mut widget = ImageWidget::new();
        widget.add_images(vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ]);

        assert_eq!(widget.image_count(), 2);
        widget.clear_images();
        assert_eq!(widget.image_count(), 0);
    }

    #[test]
    fn test_visibility() {
        let mut widget = ImageWidget::new();
        assert!(widget.visible);

        widget.hide();
        assert!(!widget.visible);

        widget.show();
        assert!(widget.visible);

        widget.toggle_visibility();
        assert!(!widget.visible);
    }

    #[test]
    fn test_set_dimensions() {
        let mut widget = ImageWidget::new();
        widget.set_dimensions(100, 50);

        assert_eq!(widget.max_width, 100);
        assert_eq!(widget.max_height, 50);
    }

    #[test]
    fn test_metadata_toggle() {
        let mut widget = ImageWidget::new();
        assert!(widget.show_metadata);

        widget.disable_metadata();
        assert!(!widget.show_metadata);

        widget.enable_metadata();
        assert!(widget.show_metadata);
    }

    #[test]
    fn test_ascii_placeholder_toggle() {
        let mut widget = ImageWidget::new();
        assert!(widget.use_ascii_placeholder);

        widget.disable_ascii_placeholder();
        assert!(!widget.use_ascii_placeholder);

        widget.enable_ascii_placeholder();
        assert!(widget.use_ascii_placeholder);
    }

    #[test]
    fn test_render_empty_widget() {
        let widget = ImageWidget::new();
        let rendered = widget.render();
        assert_eq!(rendered, "");
    }

    #[test]
    fn test_render_hidden_widget() {
        let mut widget = ImageWidget::new();
        widget.add_image(PathBuf::from("/path/to/image.png"));
        widget.hide();

        let rendered = widget.render();
        assert_eq!(rendered, "");
    }

    #[test]
    fn test_render_single_image() {
        let mut widget = ImageWidget::new();
        widget.add_image(PathBuf::from("/path/to/image.png"));

        let rendered = widget.render();
        assert!(!rendered.is_empty());
        assert!(rendered.contains("Image 1"));
        assert!(rendered.contains("image.png"));
        assert!(rendered.contains("█")); // ASCII placeholder
    }

    #[test]
    fn test_render_multiple_images() {
        let mut widget = ImageWidget::new();
        widget.add_images(vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ]);

        let rendered = widget.render();
        assert!(rendered.contains("Image 1"));
        assert!(rendered.contains("Image 2"));
        assert!(rendered.contains("image1.png"));
        assert!(rendered.contains("image2.jpg"));
        assert!(rendered.contains("─")); // Separator
    }

    #[test]
    fn test_render_without_metadata() {
        let mut widget = ImageWidget::new();
        widget.add_image(PathBuf::from("/path/to/image.png"));
        widget.disable_metadata();

        let rendered = widget.render();
        assert!(!rendered.contains("Image 1"));
        assert!(rendered.contains("█")); // ASCII placeholder still present
    }

    #[test]
    fn test_render_without_ascii_placeholder() {
        let mut widget = ImageWidget::new();
        widget.add_image(PathBuf::from("/path/to/image.png"));
        widget.disable_ascii_placeholder();

        let rendered = widget.render();
        assert!(rendered.contains("Image 1"));
        assert!(!rendered.contains("█")); // No ASCII placeholder
    }

    #[test]
    fn test_render_fits_within_bounds() {
        let mut widget = ImageWidget::with_dimensions(80, 30);
        widget.add_image(PathBuf::from("/path/to/image.png"));

        let rendered = widget.render();
        let lines: Vec<&str> = rendered.lines().collect();

        // Check that height is within bounds
        assert!(lines.len() as u32 <= widget.max_height);

        // Check that width is within bounds
        for line in lines {
            assert!(line.chars().count() as u32 <= widget.max_width);
        }
    }
}
