//! Configuration for image support.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::ImageResult;

/// Image support configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// Supported image formats
    pub formats: FormatsConfig,
    /// Display settings
    pub display: DisplayConfig,
    /// Cache settings
    pub cache: CacheConfig,
    /// Analysis settings
    pub analysis: AnalysisConfig,
}

/// Supported image formats configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatsConfig {
    /// List of supported formats (e.g., "png", "jpg", "gif", "webp")
    pub supported: Vec<String>,
}

/// Display settings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Maximum width for display (characters)
    pub max_width: u32,
    /// Maximum height for display (characters)
    pub max_height: u32,
    /// ASCII placeholder character
    pub placeholder_char: String,
}

/// Cache settings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache TTL in seconds (24 hours = 86400)
    pub ttl_seconds: u64,
    /// Maximum cache size in MB (100 MB)
    pub max_size_mb: u64,
}

/// Analysis settings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Analysis timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum image size for analysis in MB (10 MB)
    pub max_image_size_mb: u64,
    /// Whether to automatically optimize large images
    pub optimize_large_images: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            formats: FormatsConfig::default(),
            display: DisplayConfig::default(),
            cache: CacheConfig::default(),
            analysis: AnalysisConfig::default(),
        }
    }
}

impl Default for FormatsConfig {
    fn default() -> Self {
        Self {
            supported: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ],
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            max_width: 80,
            max_height: 30,
            placeholder_char: "█".to_string(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 86400, // 24 hours
            max_size_mb: 100,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            max_image_size_mb: 10,
            optimize_large_images: true,
        }
    }
}

impl ImageConfig {
    /// Load configuration from a YAML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Configuration loaded from file, or default if file doesn't exist
    pub fn from_file(path: &PathBuf) -> ImageResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration with hierarchy support.
    ///
    /// Configuration hierarchy (highest to lowest priority):
    /// 1. Runtime overrides (not implemented here)
    /// 2. Project-level config (projects/ricecoder/config/images.yaml)
    /// 3. User-level config (~/.ricecoder/config/images.yaml)
    /// 4. Built-in defaults
    ///
    /// # Returns
    ///
    /// Merged configuration from all available sources
    pub fn load_with_hierarchy() -> ImageResult<Self> {
        let mut config = Self::default();

        // Try user-level config
        if let Ok(user_home) = std::env::var("HOME") {
            let user_config_path = PathBuf::from(user_home)
                .join(".ricecoder")
                .join("config")
                .join("images.yaml");
            if let Ok(user_config) = Self::from_file(&user_config_path) {
                config = Self::merge(config, user_config);
            }
        }

        // Try project-level config
        let project_config_path = PathBuf::from("config/images.yaml");
        if let Ok(project_config) = Self::from_file(&project_config_path) {
            config = Self::merge(config, project_config);
        }

        Ok(config)
    }

    /// Merge two configurations, with `override_config` taking precedence.
    fn merge(mut base: Self, override_config: Self) -> Self {
        // Merge formats
        if !override_config.formats.supported.is_empty() {
            base.formats = override_config.formats;
        }

        // Merge display settings
        if override_config.display.max_width != 0 {
            base.display = override_config.display;
        }

        // Merge cache settings
        if override_config.cache.ttl_seconds != 0 {
            base.cache = override_config.cache;
        }

        // Merge analysis settings
        if override_config.analysis.timeout_seconds != 0 {
            base.analysis = override_config.analysis;
        }

        base
    }

    /// Check if a format is supported.
    pub fn is_format_supported(&self, format: &str) -> bool {
        self.formats
            .supported
            .iter()
            .any(|f| f.eq_ignore_ascii_case(format))
    }

    /// Get the list of supported formats as a comma-separated string.
    pub fn supported_formats_string(&self) -> String {
        self.formats.supported.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ImageConfig::default();
        assert!(config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 86400);
        assert_eq!(config.cache.max_size_mb, 100);
        assert_eq!(config.display.max_width, 80);
        assert_eq!(config.display.max_height, 30);
        assert_eq!(config.analysis.timeout_seconds, 10);
        assert_eq!(config.analysis.max_image_size_mb, 10);
    }

    #[test]
    fn test_is_format_supported() {
        let config = ImageConfig::default();
        assert!(config.is_format_supported("png"));
        assert!(config.is_format_supported("PNG"));
        assert!(config.is_format_supported("jpg"));
        assert!(config.is_format_supported("jpeg"));
        assert!(config.is_format_supported("gif"));
        assert!(config.is_format_supported("webp"));
        assert!(!config.is_format_supported("bmp"));
    }

    #[test]
    fn test_supported_formats_string() {
        let config = ImageConfig::default();
        let formats = config.supported_formats_string();
        assert!(formats.contains("png"));
        assert!(formats.contains("jpg"));
        assert!(formats.contains("gif"));
        assert!(formats.contains("webp"));
    }

    #[test]
    fn test_config_serialization() {
        let config = ImageConfig::default();
        let yaml = serde_yaml::to_string(&config).expect("Failed to serialize");
        let deserialized: ImageConfig = serde_yaml::from_str(&yaml).expect("Failed to deserialize");
        assert_eq!(config.cache.ttl_seconds, deserialized.cache.ttl_seconds);
        assert_eq!(config.display.max_width, deserialized.display.max_width);
    }
}
