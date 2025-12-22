//! Image format validation and detection.

use std::path::Path;

use crate::error::{ImageError, ImageResult};

/// Supported image formats.
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
}

impl ImageFormat {
    /// Detect image format from file header (magic bytes).
    pub fn detect_from_file(path: &Path) -> ImageResult<Self> {
        let bytes = std::fs::read(path)?;
        Self::detect_from_bytes(&bytes)
    }

    /// Detect image format from bytes (magic bytes).
    pub fn detect_from_bytes(bytes: &[u8]) -> ImageResult<Self> {
        if bytes.len() < 4 {
            return Err(ImageError::InvalidFile(
                "File too small to be a valid image".to_string(),
            ));
        }

        // PNG: 89 50 4E 47
        if bytes.starts_with(&[0x89, 0x50, 0x4e, 0x47]) {
            return Ok(ImageFormat::Png);
        }

        // JPEG: FF D8 FF
        if bytes.len() >= 3 && bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            return Ok(ImageFormat::Jpeg);
        }

        // GIF: 47 49 46 (GIF87a or GIF89a)
        if bytes.starts_with(b"GIF") {
            return Ok(ImageFormat::Gif);
        }

        // WebP: RIFF ... WEBP
        if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && bytes[8..12] == *b"WEBP" {
            return Ok(ImageFormat::WebP);
        }

        Err(ImageError::InvalidFile(
            "Unable to detect image format from file header".to_string(),
        ))
    }

    /// Get the format as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::WebP => "webp",
        }
    }

    /// Validate image file format and size.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file
    /// * `max_size_mb` - Maximum allowed file size in MB
    ///
    /// # Returns
    ///
    /// Image format if valid, error otherwise
    pub fn validate_file(path: &Path, max_size_mb: u64) -> ImageResult<Self> {
        // Check file exists
        if !path.exists() {
            return Err(ImageError::InvalidFile("File does not exist".to_string()));
        }

        // Check file size
        let metadata = std::fs::metadata(path)?;
        let size_mb = metadata.len() / (1024 * 1024);
        if size_mb > max_size_mb {
            return Err(ImageError::FileTooLarge {
                size_mb: size_mb as f64,
            });
        }

        // Detect format
        Self::detect_from_file(path)
    }

    /// Extract image metadata (width, height).
    pub fn extract_metadata(path: &Path) -> ImageResult<(u32, u32)> {
        let img = image::open(path)
            .map_err(|e| ImageError::InvalidFile(format!("Failed to open image: {}", e)))?;

        Ok((img.width(), img.height()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png_format() {
        let png_bytes = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        let format = ImageFormat::detect_from_bytes(&png_bytes).unwrap();
        assert_eq!(format, ImageFormat::Png);
    }

    #[test]
    fn test_detect_jpeg_format() {
        let jpeg_bytes = vec![0xff, 0xd8, 0xff, 0xe0];
        let format = ImageFormat::detect_from_bytes(&jpeg_bytes).unwrap();
        assert_eq!(format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_gif_format() {
        let gif_bytes = b"GIF89a".to_vec();
        let format = ImageFormat::detect_from_bytes(&gif_bytes).unwrap();
        assert_eq!(format, ImageFormat::Gif);
    }

    #[test]
    fn test_detect_webp_format() {
        let mut webp_bytes = b"RIFF".to_vec();
        webp_bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        webp_bytes.extend_from_slice(b"WEBP");
        let format = ImageFormat::detect_from_bytes(&webp_bytes).unwrap();
        assert_eq!(format, ImageFormat::WebP);
    }

    #[test]
    fn test_invalid_format() {
        let invalid_bytes = vec![0x00, 0x00, 0x00, 0x00];
        let result = ImageFormat::detect_from_bytes(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_as_str() {
        assert_eq!(ImageFormat::Png.as_str(), "png");
        assert_eq!(ImageFormat::Jpeg.as_str(), "jpg");
        assert_eq!(ImageFormat::Gif.as_str(), "gif");
        assert_eq!(ImageFormat::WebP.as_str(), "webp");
    }
}
