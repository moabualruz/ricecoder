//! Data models for image metadata, analysis results, and cache entries.

use crate::formats::ImageFormat;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metadata about an image file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Path to the image file
    pub path: PathBuf,
    /// Image format (PNG, JPG, GIF, WebP)
    pub format: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// SHA256 hash of the image file (used as cache key)
    pub hash: String,
}

impl ImageMetadata {
    /// Create a new image metadata struct.
    pub fn new(
        path: PathBuf,
        format: ImageFormat,
        size_bytes: u64,
        width: u32,
        height: u32,
        hash: String,
    ) -> Self {
        Self {
            path,
            format: format.as_str().to_string(),
            size_bytes,
            width,
            height,
            hash,
        }
    }

    /// Get the image format as a string.
    pub fn format_str(&self) -> &str {
        &self.format
    }

    /// Get the image dimensions as a tuple.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the file size in MB.
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Result of image analysis by an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalysisResult {
    /// SHA256 hash of the analyzed image
    pub image_hash: String,
    /// Analysis text from the provider
    pub analysis: String,
    /// Name of the provider that performed the analysis
    pub provider: String,
    /// Timestamp when the analysis was performed
    pub timestamp: DateTime<Utc>,
    /// Number of tokens used for the analysis
    pub tokens_used: u32,
}

impl ImageAnalysisResult {
    /// Create a new image analysis result.
    pub fn new(
        image_hash: String,
        analysis: String,
        provider: String,
        tokens_used: u32,
    ) -> Self {
        Self {
            image_hash,
            analysis,
            provider,
            timestamp: Utc::now(),
            tokens_used,
        }
    }

    /// Check if the analysis result is still valid (not expired).
    ///
    /// # Arguments
    ///
    /// * `ttl_seconds` - Time-to-live in seconds
    ///
    /// # Returns
    ///
    /// True if the result is still valid, false if expired
    pub fn is_valid(&self, ttl_seconds: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        age.num_seconds() < ttl_seconds as i64
    }
}

/// Cache entry for image analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCacheEntry {
    /// SHA256 hash of the image (cache key)
    pub hash: String,
    /// The cached analysis result
    pub analysis: ImageAnalysisResult,
    /// When the cache entry was created
    pub created_at: DateTime<Utc>,
    /// When the cache entry expires
    pub expires_at: DateTime<Utc>,
}

impl ImageCacheEntry {
    /// Create a new cache entry.
    pub fn new(hash: String, analysis: ImageAnalysisResult, ttl_seconds: u64) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);

        Self {
            hash,
            analysis,
            created_at: now,
            expires_at,
        }
    }

    /// Check if the cache entry has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get the remaining TTL in seconds.
    pub fn remaining_ttl_seconds(&self) -> i64 {
        let remaining = self.expires_at.signed_duration_since(Utc::now());
        remaining.num_seconds().max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_metadata_creation() {
        let metadata = ImageMetadata::new(
            PathBuf::from("/path/to/image.png"),
            ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "abc123".to_string(),
        );

        assert_eq!(metadata.format_str(), "png");
        assert_eq!(metadata.dimensions(), (800, 600));
        assert_eq!(metadata.size_bytes, 1024 * 1024);
        assert!(metadata.size_mb() > 0.9 && metadata.size_mb() < 1.1);
    }

    #[test]
    fn test_image_analysis_result_creation() {
        let result = ImageAnalysisResult::new(
            "hash123".to_string(),
            "This is an image of a cat".to_string(),
            "openai".to_string(),
            100,
        );

        assert_eq!(result.image_hash, "hash123");
        assert_eq!(result.analysis, "This is an image of a cat");
        assert_eq!(result.provider, "openai");
        assert_eq!(result.tokens_used, 100);
    }

    #[test]
    fn test_image_analysis_result_validity() {
        let result = ImageAnalysisResult::new(
            "hash123".to_string(),
            "Analysis".to_string(),
            "openai".to_string(),
            100,
        );

        // Should be valid with large TTL
        assert!(result.is_valid(86400));

        // Should be invalid with very small TTL
        assert!(!result.is_valid(0));
    }

    #[test]
    fn test_cache_entry_creation() {
        let analysis = ImageAnalysisResult::new(
            "hash123".to_string(),
            "Analysis".to_string(),
            "openai".to_string(),
            100,
        );

        let entry = ImageCacheEntry::new("hash123".to_string(), analysis, 3600);

        assert_eq!(entry.hash, "hash123");
        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl_seconds() > 0);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let analysis = ImageAnalysisResult::new(
            "hash123".to_string(),
            "Analysis".to_string(),
            "openai".to_string(),
            100,
        );

        // Create entry with 0 TTL (already expired)
        let entry = ImageCacheEntry::new("hash123".to_string(), analysis, 0);

        assert!(entry.is_expired());
        assert_eq!(entry.remaining_ttl_seconds(), 0);
    }
}
