//! Error types for image operations.

use thiserror::Error;

/// Result type for image operations.
pub type ImageResult<T> = Result<T, ImageError>;

/// Errors that can occur during image operations.
#[derive(Debug, Error)]
pub enum ImageError {
    /// Image format is not supported.
    #[error("Format not supported: {0}. Supported formats: PNG, JPG, GIF, WebP")]
    FormatNotSupported(String),

    /// Image file exceeds maximum size (10 MB).
    #[error("File too large: {size_mb:.1} MB exceeds maximum of 10 MB")]
    FileTooLarge { size_mb: f64 },

    /// File is not a valid image.
    #[error("Invalid image file: {0}")]
    InvalidFile(String),

    /// Image analysis failed.
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    /// Cache operation failed.
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Display operation failed.
    #[error("Display error: {0}")]
    DisplayError(String),

    /// Banner rendering failed.
    #[error("Banner error: {0}")]
    BannerError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Path traversal attempt detected.
    #[error("Invalid file path: path traversal detected")]
    PathTraversalError,

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_yaml::Error> for ImageError {
    fn from(err: serde_yaml::Error) -> Self {
        ImageError::ConfigError(err.to_string())
    }
}

impl From<serde_json::Error> for ImageError {
    fn from(err: serde_json::Error) -> Self {
        ImageError::SerializationError(err.to_string())
    }
}
