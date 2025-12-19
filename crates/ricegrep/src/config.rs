//! Configuration management for RiceGrep

use figment::{Figment, providers::{Format, Toml, Env, Serialized}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure for RiceGrep
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiceGrepConfig {
    // Core search settings
    pub ai_enabled: bool,
    pub fuzzy_distance: usize,
    pub max_search_results: Option<usize>,

    // Watch mode settings
    pub watch_enabled: bool,
    pub watch_timeout_seconds: Option<u64>,
    pub watch_clear_screen: bool,
    pub watch_poll_interval_ms: Option<u64>,

    // Output settings
    pub output_format: OutputFormat,
    pub pretty_json: bool,
    pub show_line_numbers: bool,

    // AI settings
    pub ai_model_path: Option<PathBuf>,
    pub ai_timeout_seconds: u64,
    pub ai_memory_limit_mb: usize,

    // Spelling correction settings
    pub spelling_enabled: bool,
    pub spelling_max_distance: usize,
    pub spelling_min_score: f64,

    // Language detection settings
    pub language_detection_enabled: bool,
    pub language_ranking_boosts: HashMap<String, f32>,

    // Index settings
    pub index_dir: PathBuf,
    pub max_file_size_mb: usize,

    // MCP server settings
    pub mcp_enabled: bool,
    pub mcp_port: Option<u16>,

    // Plugin settings
    pub plugin_dirs: Vec<PathBuf>,
    pub plugin_enabled: Vec<String>,

    // Store settings
    pub store_enabled: bool,
    pub store_path: PathBuf,
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Color output options
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl Default for RiceGrepConfig {
    fn default() -> Self {
        Self {
            ai_enabled: false,
            fuzzy_distance: 2,
            max_search_results: Some(1000),

            watch_enabled: false,
            watch_timeout_seconds: None,
            watch_clear_screen: false,
            watch_poll_interval_ms: None,

            output_format: OutputFormat::Text,
            pretty_json: false,
            show_line_numbers: true,

            ai_model_path: None,
            ai_timeout_seconds: 30,
            ai_memory_limit_mb: 1024,

            spelling_enabled: true,
            spelling_max_distance: 2,
            spelling_min_score: 0.8,

            language_detection_enabled: true,
            language_ranking_boosts: {
                let mut boosts = HashMap::new();
                boosts.insert("rust".to_string(), 1.2);
                boosts.insert("python".to_string(), 1.1);
                boosts.insert("javascript".to_string(), 1.0);
                boosts.insert("typescript".to_string(), 1.05);
                boosts.insert("go".to_string(), 1.1);
                boosts.insert("java".to_string(), 0.9);
                boosts.insert("c".to_string(), 0.8);
                boosts.insert("cpp".to_string(), 0.85);
                boosts
            },

            index_dir: PathBuf::from(".ricegrep"),
            max_file_size_mb: 10,

            mcp_enabled: false,
            mcp_port: None,

            plugin_dirs: vec![PathBuf::from(".ricegrep/plugins")],
            plugin_enabled: vec![],

            store_enabled: true,
            store_path: PathBuf::from(".ricegrep/store"),
        }
    }
}

impl RiceGrepConfig {
    /// Load configuration from all sources with cascading priority
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            // Start with defaults
            .merge(Serialized::defaults(Self::default()))
            // Global configuration file
            .merge(Toml::file(expand_path("~/.ricegrep.toml")).nested())
            // Local project configuration
            .merge(Toml::file(".ricegrep.toml").nested())
            // Environment variables (RICEGREP_*)
            .merge(Env::prefixed("RICEGREP_").global())
            // Extract final configuration
            .extract()
    }

    /// Save configuration to local file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::create_dir_all(".ricegrep")?;
        std::fs::write(".ricegrep.toml", toml_string)?;
        Ok(())
    }
}

/// Expand tilde in path
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(path.replacen('~', &home.to_string_lossy(), 1));
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RiceGrepConfig::default();
        assert!(!config.ai_enabled);
        assert_eq!(config.fuzzy_distance, 2);
        assert_eq!(config.output_format, OutputFormat::Text);
    }

    #[test]
    fn test_config_save_load() {
        use tempfile::tempdir;

        let config = RiceGrepConfig::default();
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Save config to temp file
        let toml_string = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, &toml_string).unwrap();

        // Load config from temp file using toml crate directly
        let loaded_toml: RiceGrepConfig = toml::from_str(&toml_string).unwrap();

        assert_eq!(config.ai_enabled, loaded_toml.ai_enabled);
        assert_eq!(config.fuzzy_distance, loaded_toml.fuzzy_distance);
    }
}