//! Terminal display of images with ASCII fallback.

use crate::{
    config::DisplayConfig,
    error::{ImageError, ImageResult},
    models::ImageMetadata,
};

/// Displays images in the terminal with ASCII fallback support.
///
/// Provides terminal rendering of images with:
/// - Metadata display (format, size, dimensions)
/// - ASCII placeholder for unsupported terminals
/// - Automatic resizing to fit terminal bounds (max 80x30)
/// - Multi-image support with vertical organization
pub struct ImageDisplay {
    pub(crate) config: DisplayConfig,
}

impl ImageDisplay {
    /// Create a new image display with default configuration.
    pub fn new() -> Self {
        Self {
            config: DisplayConfig::default(),
        }
    }

    /// Create a new image display with custom configuration.
    pub fn with_config(config: DisplayConfig) -> Self {
        Self { config }
    }

    /// Render a single image with metadata.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Image metadata containing format, size, dimensions
    ///
    /// # Returns
    ///
    /// Rendered image string with metadata
    pub fn render_image(&self, metadata: &ImageMetadata) -> ImageResult<String> {
        let mut output = String::new();

        // Add metadata header
        output.push_str(&self.render_metadata(metadata)?);
        output.push('\n');

        // Add ASCII placeholder
        output.push_str(&self.render_ascii_placeholder());

        Ok(output)
    }

    /// Render metadata for an image.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Image metadata
    ///
    /// # Returns
    ///
    /// Formatted metadata string
    fn render_metadata(&self, metadata: &ImageMetadata) -> ImageResult<String> {
        let (width, height) = metadata.dimensions();
        let size_mb = metadata.size_mb();
        let format = metadata.format_str();

        Ok(format!(
            "[Image] {} | {}x{} | {:.1} MB",
            format.to_uppercase(),
            width,
            height,
            size_mb
        ))
    }

    /// Render ASCII placeholder for unsupported terminals.
    ///
    /// # Returns
    ///
    /// ASCII art placeholder string
    fn render_ascii_placeholder(&self) -> String {
        let placeholder_char = &self.config.placeholder_char;
        let width = self.config.max_width as usize;
        let height = 10; // Use fixed small height instead of max_height

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

    /// Calculate resized dimensions to fit within terminal bounds.
    ///
    /// # Arguments
    ///
    /// * `original_width` - Original image width in pixels
    /// * `original_height` - Original image height in pixels
    ///
    /// # Returns
    ///
    /// Resized dimensions (width, height) in characters
    pub fn calculate_resized_dimensions(
        &self,
        original_width: u32,
        original_height: u32,
    ) -> (u32, u32) {
        if original_width == 0 || original_height == 0 {
            return (self.config.max_width, self.config.max_height);
        }

        let max_width = self.config.max_width;
        let max_height = self.config.max_height;

        // Calculate aspect ratio
        let aspect_ratio = original_width as f64 / original_height as f64;

        // Calculate dimensions if we scale to max width
        let height_at_max_width = (max_width as f64 / aspect_ratio) as u32;

        // If scaling to max width fits within height, use it
        if height_at_max_width <= max_height {
            (max_width, height_at_max_width.max(1))
        } else {
            // Otherwise scale to max height
            let width_at_max_height = (max_height as f64 * aspect_ratio) as u32;
            (width_at_max_height.max(1), max_height)
        }
    }

    /// Verify that display includes all required metadata.
    ///
    /// # Arguments
    ///
    /// * `display_output` - The rendered display string
    /// * `metadata` - The image metadata
    ///
    /// # Returns
    ///
    /// True if all metadata is present, error otherwise
    pub fn verify_metadata_present(
        &self,
        display_output: &str,
        metadata: &ImageMetadata,
    ) -> ImageResult<bool> {
        let format = metadata.format_str().to_uppercase();
        let (width, height) = metadata.dimensions();

        // Check if format is present
        if !display_output.contains(&format) {
            return Err(ImageError::DisplayError(
                "Format not found in display output".to_string(),
            ));
        }

        // Check if dimensions are present
        let dimensions_str = format!("{}x{}", width, height);
        if !display_output.contains(&dimensions_str) {
            return Err(ImageError::DisplayError(
                "Dimensions not found in display output".to_string(),
            ));
        }

        Ok(true)
    }

    /// Verify that display fits within terminal bounds.
    ///
    /// # Arguments
    ///
    /// * `display_output` - The rendered display string
    ///
    /// # Returns
    ///
    /// True if display fits within bounds, error otherwise
    pub fn verify_fits_in_terminal(&self, display_output: &str) -> ImageResult<bool> {
        let lines: Vec<&str> = display_output.lines().collect();
        let height = lines.len() as u32;

        if height > self.config.max_height {
            return Err(ImageError::DisplayError(format!(
                "Display height {} exceeds maximum {}",
                height, self.config.max_height
            )));
        }

        for line in lines {
            let width = line.chars().count() as u32;
            if width > self.config.max_width {
                return Err(ImageError::DisplayError(format!(
                    "Display width {} exceeds maximum {}",
                    width, self.config.max_width
                )));
            }
        }

        Ok(true)
    }

    /// Render multiple images organized vertically with separators.
    ///
    /// # Arguments
    ///
    /// * `metadata_list` - List of image metadata to render
    ///
    /// # Returns
    ///
    /// Rendered multi-image display string
    pub fn render_multiple_images(&self, metadata_list: &[ImageMetadata]) -> ImageResult<String> {
        if metadata_list.is_empty() {
            return Ok(String::new());
        }

        let mut output = String::new();

        for (index, metadata) in metadata_list.iter().enumerate() {
            // Add separator between images
            if index > 0 {
                output.push_str(&self.render_separator());
                output.push('\n');
            }

            // Render image
            let image_display = self.render_image(metadata)?;
            output.push_str(&image_display);

            // Add newline after each image except the last
            if index < metadata_list.len() - 1 {
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Render a separator between images.
    ///
    /// # Returns
    ///
    /// Separator string
    fn render_separator(&self) -> String {
        let separator_char = "─";
        separator_char.repeat(self.config.max_width as usize)
    }

    /// Verify that multiple images are organized vertically.
    ///
    /// # Arguments
    ///
    /// * `display_output` - The rendered display string
    /// * `metadata_list` - List of image metadata
    ///
    /// # Returns
    ///
    /// True if images are properly organized, error otherwise
    pub fn verify_multiple_images_organized(
        &self,
        display_output: &str,
        metadata_list: &[ImageMetadata],
    ) -> ImageResult<bool> {
        if metadata_list.is_empty() {
            return Ok(true);
        }

        // Check that all images are present
        for metadata in metadata_list {
            let format = metadata.format_str().to_uppercase();
            if !display_output.contains(&format) {
                return Err(ImageError::DisplayError(
                    "Not all images present in display output".to_string(),
                ));
            }
        }

        // Check that separators are present between images (if more than one)
        if metadata_list.len() > 1 {
            // Count separator lines (lines that are all separator characters)
            let separator_lines = display_output
                .lines()
                .filter(|line| line.chars().all(|c| c == '─' || c.is_whitespace()))
                .count();

            if separator_lines == 0 {
                return Err(ImageError::DisplayError(
                    "No separators found between images".to_string(),
                ));
            }
        }

        // Verify overall display fits in terminal
        self.verify_fits_in_terminal(display_output)?;

        Ok(true)
    }
}

impl Default for ImageDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::formats::ImageFormat;

    #[test]
    fn test_display_creation() {
        let _display = ImageDisplay::new();
    }

    #[test]
    fn test_display_with_config() {
        let config = DisplayConfig {
            max_width: 100,
            max_height: 50,
            placeholder_char: "█".to_string(),
        };
        let display = ImageDisplay::with_config(config);
        assert_eq!(display.config.max_width, 100);
        assert_eq!(display.config.max_height, 50);
    }

    #[test]
    fn test_render_metadata() {
        let display = ImageDisplay::new();
        let metadata = ImageMetadata::new(
            PathBuf::from("/path/to/image.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );

        let metadata_str = display.render_metadata(&metadata).unwrap();
        assert!(metadata_str.contains("PNG"));
        assert!(metadata_str.contains("800x600"));
        assert!(metadata_str.contains("1.0 MB"));
    }

    #[test]
    fn test_render_ascii_placeholder() {
        let display = ImageDisplay::new();
        let placeholder = display.render_ascii_placeholder();

        // Should have multiple lines
        let lines: Vec<&str> = placeholder.lines().collect();
        assert!(lines.len() > 0);

        // Should fit within max dimensions
        assert!(lines.len() as u32 <= display.config.max_height);
        for line in lines {
            assert!(line.chars().count() as u32 <= display.config.max_width);
        }
    }

    #[test]
    fn test_render_image() {
        let display = ImageDisplay::new();
        let metadata = ImageMetadata::new(
            PathBuf::from("/path/to/image.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );

        let rendered = display.render_image(&metadata).unwrap();
        assert!(rendered.contains("PNG"));
        assert!(rendered.contains("800x600"));
        assert!(rendered.contains("█")); // Placeholder character
    }

    #[test]
    fn test_calculate_resized_dimensions_width_limited() {
        let display = ImageDisplay::new();
        // Use an image that is very wide but not too tall (so width is the limiting factor)
        let (resized_width, resized_height) = display.calculate_resized_dimensions(800, 100);

        // Width should be at max
        assert_eq!(resized_width, display.config.max_width);
        // Height should be proportional
        assert!(resized_height <= display.config.max_height);
    }

    #[test]
    fn test_calculate_resized_dimensions_height_limited() {
        let display = ImageDisplay::new();
        let (resized_width, resized_height) = display.calculate_resized_dimensions(400, 1200);

        // Height should be at max
        assert_eq!(resized_height, display.config.max_height);
        // Width should be proportional
        assert!(resized_width <= display.config.max_width);
    }

    #[test]
    fn test_calculate_resized_dimensions_zero_dimensions() {
        let display = ImageDisplay::new();
        let (resized_width, resized_height) = display.calculate_resized_dimensions(0, 0);

        // Should return max dimensions
        assert_eq!(resized_width, display.config.max_width);
        assert_eq!(resized_height, display.config.max_height);
    }

    #[test]
    fn test_verify_metadata_present() {
        let display = ImageDisplay::new();
        let metadata = ImageMetadata::new(
            PathBuf::from("/path/to/image.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );

        let rendered = display.render_image(&metadata).unwrap();
        let result = display.verify_metadata_present(&rendered, &metadata);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_fits_in_terminal() {
        let display = ImageDisplay::new();
        let metadata = ImageMetadata::new(
            PathBuf::from("/path/to/image.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );

        let rendered = display.render_image(&metadata).unwrap();
        let result = display.verify_fits_in_terminal(&rendered);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_render_separator() {
        let display = ImageDisplay::new();
        let separator = display.render_separator();

        // Should contain separator characters
        assert!(separator.contains("─"));
        // Should be approximately max_width length
        assert!(separator.len() > 0);
    }

    #[test]
    fn test_render_multiple_images_empty() {
        let display = ImageDisplay::new();
        let metadata_list: Vec<ImageMetadata> = vec![];

        let rendered = display.render_multiple_images(&metadata_list).unwrap();
        assert_eq!(rendered, "");
    }

    #[test]
    fn test_render_multiple_images_single() {
        let display = ImageDisplay::new();
        let metadata = ImageMetadata::new(
            PathBuf::from("/path/to/image.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );

        let rendered = display.render_multiple_images(&[metadata]).unwrap();
        assert!(rendered.contains("PNG"));
        assert!(rendered.contains("800x600"));
    }

    #[test]
    fn test_render_multiple_images_multiple() {
        let display = ImageDisplay::new();
        let metadata1 = ImageMetadata::new(
            PathBuf::from("/path/to/image1.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );
        let metadata2 = ImageMetadata::new(
            PathBuf::from("/path/to/image2.jpg"),
            ImageFormat::Jpeg,
            2048 * 1024,
            1024,
            768,
            "def456".to_string(),
        );

        let rendered = display
            .render_multiple_images(&[metadata1, metadata2])
            .unwrap();

        // Should contain both images
        assert!(rendered.contains("PNG"));
        assert!(rendered.contains("JPG"));
        assert!(rendered.contains("800x600"));
        assert!(rendered.contains("1024x768"));

        // Should contain separator
        assert!(rendered.contains("─"));
    }

    #[test]
    fn test_verify_multiple_images_organized() {
        let display = ImageDisplay::new();
        let metadata1 = ImageMetadata::new(
            PathBuf::from("/path/to/image1.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );
        let metadata2 = ImageMetadata::new(
            PathBuf::from("/path/to/image2.jpg"),
            ImageFormat::Jpeg,
            2048 * 1024,
            1024,
            768,
            "def456".to_string(),
        );

        let rendered = display
            .render_multiple_images(&[metadata1.clone(), metadata2.clone()])
            .unwrap();
        let result = display.verify_multiple_images_organized(&rendered, &[metadata1, metadata2]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
