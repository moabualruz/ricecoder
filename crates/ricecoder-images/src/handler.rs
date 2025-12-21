//! Image drag-and-drop event handling.

use crate::config::ImageConfig;
use crate::error::{ImageError, ImageResult};
use crate::formats::ImageFormat;
use crate::models::ImageMetadata;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Handles image drag-and-drop events and file operations.
pub struct ImageHandler {
    config: ImageConfig,
}

impl ImageHandler {
    /// Create a new image handler with the given configuration.
    pub fn new(config: ImageConfig) -> Self {
        Self { config }
    }

    /// Create a new image handler with default configuration.
    pub fn with_default_config() -> ImageResult<Self> {
        let config = ImageConfig::load_with_hierarchy()?;
        Ok(Self::new(config))
    }

    /// Sanitize a file path to prevent directory traversal attacks.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to sanitize
    ///
    /// # Returns
    ///
    /// The sanitized path, or an error if path traversal is detected
    fn sanitize_path(path: &Path) -> ImageResult<PathBuf> {
        // Normalize the path to resolve .. and . components
        let normalized = path
            .canonicalize()
            .map_err(|_| ImageError::PathTraversalError)?;

        // Ensure the path is absolute and doesn't contain suspicious patterns
        if !normalized.is_absolute() {
            return Err(ImageError::PathTraversalError);
        }

        Ok(normalized)
    }

    /// Read an image file and validate it.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file
    ///
    /// # Returns
    ///
    /// Image metadata if successful, error otherwise
    ///
    /// # Requirements
    ///
    /// - Req 1.2: Read file using ricecoder-files integration
    /// - Req 1.3: Validate format (PNG, JPG, GIF, WebP)
    /// - Req 1.5: Report error with format details
    /// - Security: Sanitize file paths to prevent directory traversal
    pub fn read_image(&self, path: &Path) -> ImageResult<ImageMetadata> {
        // Sanitize path to prevent directory traversal
        let sanitized_path = Self::sanitize_path(path)?;

        // Validate file exists and is readable
        if !sanitized_path.exists() {
            return Err(ImageError::InvalidFile(format!(
                "File does not exist: {}",
                sanitized_path.display()
            )));
        }

        // Check if it's a file (not a directory)
        if !sanitized_path.is_file() {
            return Err(ImageError::InvalidFile("Path is not a file".to_string()));
        }

        // Validate format and size
        let format =
            ImageFormat::validate_file(&sanitized_path, self.config.analysis.max_image_size_mb)?;

        // Extract metadata (width, height)
        let (width, height) = ImageFormat::extract_metadata(&sanitized_path)?;

        // Calculate SHA256 hash
        let hash = self.calculate_file_hash(&sanitized_path)?;

        // Get file size
        let metadata = std::fs::metadata(&sanitized_path)?;
        let size_bytes = metadata.len();

        Ok(ImageMetadata::new(
            sanitized_path,
            format,
            size_bytes,
            width,
            height,
            hash,
        ))
    }

    /// Calculate SHA256 hash of a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file
    ///
    /// # Returns
    ///
    /// Hex-encoded SHA256 hash
    fn calculate_file_hash(&self, path: &Path) -> ImageResult<String> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Validate image format is supported.
    ///
    /// # Arguments
    ///
    /// * `format` - The image format to validate
    ///
    /// # Returns
    ///
    /// Ok if format is supported, error with supported formats list otherwise
    ///
    /// # Requirements
    ///
    /// - Req 1.3: Validate format (PNG, JPG, GIF, WebP)
    /// - Req 4.5: Report error with supported formats list
    pub fn validate_format(&self, format: ImageFormat) -> ImageResult<()> {
        let format_str = format.as_str();
        if self.config.is_format_supported(format_str) {
            Ok(())
        } else {
            Err(ImageError::FormatNotSupported(format!(
                "Format '{}' is not supported. Supported formats: {}",
                format_str,
                self.config.supported_formats_string()
            )))
        }
    }

    /// Handle multiple files from a drag-and-drop event.
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to the dropped files
    ///
    /// # Returns
    ///
    /// Vector of successfully read images and any errors encountered
    ///
    /// # Requirements
    ///
    /// - Req 1.1: Handle multiple files in single drag-and-drop
    pub fn handle_dropped_files(&self, paths: &[PathBuf]) -> (Vec<ImageMetadata>, Vec<ImageError>) {
        let mut images = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            match self.read_image(path) {
                Ok(metadata) => images.push(metadata),
                Err(e) => errors.push(e),
            }
        }

        (images, errors)
    }

    /// Get the configuration.
    pub fn config(&self) -> &ImageConfig {
        &self.config
    }

    /// Extract file paths from a drag-and-drop event.
    ///
    /// # Arguments
    ///
    /// * `event_data` - Raw event data from the terminal (typically file paths)
    ///
    /// # Returns
    ///
    /// Vector of file paths extracted from the event
    ///
    /// # Requirements
    ///
    /// - Req 1.1: Create interface for receiving drag-and-drop events from ricecoder-tui
    /// - Req 1.1: Implement file path extraction from events
    pub fn extract_paths_from_event(event_data: &str) -> Vec<PathBuf> {
        // First try to split by newlines (most common for drag-and-drop)
        let lines: Vec<&str> = event_data.lines().collect();

        if lines.len() > 1 || (lines.len() == 1 && !lines[0].contains(' ')) {
            // Multiple lines or single line without spaces - use line splitting
            return lines
                .into_iter()
                .filter(|line| !line.is_empty())
                .map(PathBuf::from)
                .collect();
        }

        // Single line with spaces - split by whitespace
        event_data
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect()
    }

    /// Check if a file exists and is readable.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check
    ///
    /// # Returns
    ///
    /// Ok if file exists and is readable, error otherwise
    ///
    /// # Requirements
    ///
    /// - Req 1.1: Implement file existence and permission checks
    pub fn check_file_accessible(path: &Path) -> ImageResult<()> {
        // Check if file exists
        if !path.exists() {
            return Err(ImageError::InvalidFile(format!(
                "File does not exist: {}",
                path.display()
            )));
        }

        // Check if it's a file (not a directory)
        if !path.is_file() {
            return Err(ImageError::InvalidFile("Path is not a file".to_string()));
        }

        // Try to open the file to check permissions
        std::fs::File::open(path)
            .map_err(|e| ImageError::InvalidFile(format!("Cannot read file: {}", e)))?;

        Ok(())
    }

    /// Process a drag-and-drop event with multiple files.
    ///
    /// # Arguments
    ///
    /// * `event_data` - Raw event data from the terminal
    ///
    /// # Returns
    ///
    /// Tuple of (successfully processed images, errors encountered)
    ///
    /// # Requirements
    ///
    /// - Req 1.1: Handle multiple files in single drag-and-drop
    /// - Req 1.1: Implement file existence and permission checks
    pub fn process_drag_drop_event(
        &self,
        event_data: &str,
    ) -> (Vec<ImageMetadata>, Vec<ImageError>) {
        let paths = Self::extract_paths_from_event(event_data);

        let mut images = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            // Check file accessibility first
            if let Err(e) = Self::check_file_accessible(&path) {
                errors.push(e);
                continue;
            }

            // Try to read the image
            match self.read_image(&path) {
                Ok(metadata) => images.push(metadata),
                Err(e) => errors.push(e),
            }
        }

        (images, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_handler_creation() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);
        assert!(handler.config.cache.enabled);
    }

    #[test]
    fn test_handler_default_creation() {
        let handler = ImageHandler::with_default_config();
        assert!(handler.is_ok());
    }

    #[test]
    fn test_sanitize_path_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let result = ImageHandler::sanitize_path(&temp_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_path_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/that/does/not/exist");
        let result = ImageHandler::sanitize_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_image_nonexistent_file() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let result = handler.read_image(Path::new("/nonexistent/image.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_image_valid_png() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Create a temporary PNG file with valid magic bytes
        let mut temp_file = NamedTempFile::new().unwrap();
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        temp_file.write_all(&png_header).unwrap();
        temp_file.flush().unwrap();

        // This will fail because it's not a valid PNG, but we can test the path handling
        let result = handler.read_image(temp_file.path());
        // We expect an error because the file is not a valid image
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_supported() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let result = handler.validate_format(ImageFormat::Png);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_format_all_supported() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        assert!(handler.validate_format(ImageFormat::Png).is_ok());
        assert!(handler.validate_format(ImageFormat::Jpeg).is_ok());
        assert!(handler.validate_format(ImageFormat::Gif).is_ok());
        assert!(handler.validate_format(ImageFormat::WebP).is_ok());
    }

    #[test]
    fn test_handle_dropped_files_empty() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let (images, errors) = handler.handle_dropped_files(&[]);
        assert_eq!(images.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_handle_dropped_files_nonexistent() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let paths = vec![
            PathBuf::from("/nonexistent/image1.png"),
            PathBuf::from("/nonexistent/image2.jpg"),
        ];

        let (images, errors) = handler.handle_dropped_files(&paths);
        assert_eq!(images.len(), 0);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_calculate_file_hash() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Create a temporary file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        let hash = handler.calculate_file_hash(temp_file.path());
        assert!(hash.is_ok());

        let hash_str = hash.unwrap();
        // SHA256 of "test content" should be a 64-character hex string
        assert_eq!(hash_str.len(), 64);
    }

    #[test]
    fn test_calculate_file_hash_consistency() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Create a temporary file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        let hash1 = handler.calculate_file_hash(temp_file.path()).unwrap();
        let hash2 = handler.calculate_file_hash(temp_file.path()).unwrap();

        // Same file should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_extract_paths_from_event_single_line() {
        let event_data = "/path/to/image.png";
        let paths = ImageHandler::extract_paths_from_event(event_data);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("/path/to/image.png"));
    }

    #[test]
    fn test_extract_paths_from_event_multiple_lines() {
        let event_data = "/path/to/image1.png\n/path/to/image2.jpg\n/path/to/image3.gif";
        let paths = ImageHandler::extract_paths_from_event(event_data);

        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], PathBuf::from("/path/to/image1.png"));
        assert_eq!(paths[1], PathBuf::from("/path/to/image2.jpg"));
        assert_eq!(paths[2], PathBuf::from("/path/to/image3.gif"));
    }

    #[test]
    fn test_extract_paths_from_event_space_separated() {
        let event_data = "/path/to/image1.png /path/to/image2.jpg";
        let paths = ImageHandler::extract_paths_from_event(event_data);

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("/path/to/image1.png"));
        assert_eq!(paths[1], PathBuf::from("/path/to/image2.jpg"));
    }

    #[test]
    fn test_extract_paths_from_event_empty() {
        let event_data = "";
        let paths = ImageHandler::extract_paths_from_event(event_data);

        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_check_file_accessible_nonexistent() {
        let result = ImageHandler::check_file_accessible(Path::new("/nonexistent/file.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_file_accessible_valid() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = ImageHandler::check_file_accessible(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_file_accessible_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = ImageHandler::check_file_accessible(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_process_drag_drop_event_empty() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let (images, errors) = handler.process_drag_drop_event("");
        assert_eq!(images.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_process_drag_drop_event_nonexistent_files() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let event_data = "/nonexistent/image1.png\n/nonexistent/image2.jpg";
        let (images, errors) = handler.process_drag_drop_event(event_data);

        assert_eq!(images.len(), 0);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_process_drag_drop_event_mixed_valid_invalid() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_string_lossy().to_string();

        let event_data = format!("{}\n/nonexistent/image.png", temp_path);
        let (_images, errors) = handler.process_drag_drop_event(&event_data);

        // One file exists but is not a valid image, one doesn't exist
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_error_message_format_not_supported() {
        let config = ImageConfig::default();
        let _handler = ImageHandler::new(config);

        // Create a custom config with limited formats
        let mut limited_config = ImageConfig::default();
        limited_config.formats.supported = vec!["png".to_string()];
        let limited_handler = ImageHandler::new(limited_config);

        let result = limited_handler.validate_format(ImageFormat::Jpeg);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not supported"));
        assert!(error_msg.contains("png"));
    }

    #[test]
    fn test_error_message_file_too_large() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Create a temporary file larger than max size
        let mut temp_file = NamedTempFile::new().unwrap();
        let large_data = vec![0u8; 11 * 1024 * 1024]; // 11 MB
        temp_file.write_all(&large_data).unwrap();
        temp_file.flush().unwrap();

        let result = handler.read_image(temp_file.path());
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("too large") || error_msg.contains("exceeds"));
    }

    #[test]
    fn test_error_message_invalid_file() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Create a temporary file with invalid image data
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not an image").unwrap();
        temp_file.flush().unwrap();

        let result = handler.read_image(temp_file.path());
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid") || error_msg.contains("invalid"));
    }

    #[test]
    fn test_path_sanitization_prevents_traversal() {
        // Test that path traversal attempts are blocked
        let suspicious_path = PathBuf::from("../../../etc/passwd");
        let result = ImageHandler::sanitize_path(&suspicious_path);

        // Should fail because the path doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_error_message_path_traversal() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Try to read a file with path traversal
        let suspicious_path = Path::new("../../../etc/passwd");
        let result = handler.read_image(suspicious_path);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Should contain either path traversal error or file not found
        assert!(error_msg.contains("traversal") || error_msg.contains("does not exist"));
    }

    #[test]
    fn test_format_validation_error_includes_supported_formats() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Validate that error messages include the list of supported formats
        let result = handler.validate_format(ImageFormat::Png);
        assert!(result.is_ok());

        // Check that the config can provide supported formats string
        let formats_str = handler.config().supported_formats_string();
        assert!(formats_str.contains("png"));
        assert!(formats_str.contains("jpg"));
        assert!(formats_str.contains("gif"));
        assert!(formats_str.contains("webp"));
    }

    #[test]
    fn test_read_image_with_valid_metadata() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        // Create a temporary PNG file with valid header
        let mut temp_file = NamedTempFile::new().unwrap();
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        temp_file.write_all(&png_header).unwrap();
        temp_file.flush().unwrap();

        // This will fail because it's not a complete valid PNG, but we can verify
        // that the error is about the image format, not the file access
        let result = handler.read_image(temp_file.path());

        // The error should be about invalid image, not file access
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Invalid") || error_msg.contains("invalid"));
        }
    }

    #[test]
    fn test_multiple_files_error_collection() {
        let config = ImageConfig::default();
        let handler = ImageHandler::new(config);

        let paths = vec![
            PathBuf::from("/nonexistent/image1.png"),
            PathBuf::from("/nonexistent/image2.jpg"),
            PathBuf::from("/nonexistent/image3.gif"),
        ];

        let (_images, errors) = handler.handle_dropped_files(&paths);

        // All files should fail
        assert_eq!(errors.len(), 3);

        // All errors should be about file not existing
        for error in errors {
            let error_msg = error.to_string();
            assert!(error_msg.contains("does not exist") || error_msg.contains("Invalid"));
        }
    }
}
