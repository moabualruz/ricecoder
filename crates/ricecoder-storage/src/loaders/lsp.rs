//! LSP server configuration loader for RiceCoder
//!
//! Loads LSP server configurations from `config/lsp/*.yaml` files.
//! Each YAML file contains configuration for a single LSP server.
//!
//! # Configuration Schema
//!
//! ```yaml
//! language: rust
//! extensions:
//!   - .rs
//! executable: rust-analyzer
//! args: []
//! env: {}
//! init_options:
//!   checkOnSave:
//!     command: clippy
//! enabled: true
//! timeout_ms: 10000
//! max_restarts: 3
//! idle_timeout_ms: 300000
//! ```
//!
//! # Usage
//!
//! ```rust
//! use ricecoder_storage::loaders::LspConfigLoader;
//!
//! let loader = LspConfigLoader::with_default_path();
//! let config = loader.get_config("rust");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::error::{IoOperation, StorageError, StorageResult};

/// LSP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    /// Language identifier (e.g., "rust", "typescript", "python")
    pub language: String,
    
    /// File extensions this server handles (e.g., [".rs"])
    pub extensions: Vec<String>,
    
    /// Executable command (e.g., "rust-analyzer")
    pub executable: String,
    
    /// Command-line arguments
    #[serde(default)]
    pub args: Vec<String>,
    
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    /// Initialization options sent to the LSP server
    #[serde(default)]
    pub init_options: Option<serde_json::Value>,
    
    /// Whether this server is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Request timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    
    /// Maximum restart attempts
    #[serde(default = "default_max_restarts")]
    pub max_restarts: u32,
    
    /// Idle timeout before shutdown in milliseconds
    #[serde(default = "default_idle_timeout_ms")]
    pub idle_timeout_ms: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_timeout_ms() -> u64 {
    5000
}

fn default_max_restarts() -> u32 {
    3
}

fn default_idle_timeout_ms() -> u64 {
    300000 // 5 minutes
}

/// Cached LSP configurations
#[derive(Debug, Clone)]
struct LspConfigCache {
    /// Language -> Configuration mapping
    configs: HashMap<String, LspConfig>,
    /// Whether cache has been populated
    loaded: bool,
}

/// Loader for LSP server configuration files
pub struct LspConfigLoader {
    config_dir: PathBuf,
    cache: Arc<RwLock<LspConfigCache>>,
}

impl LspConfigLoader {
    /// Create a new LSP config loader with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            cache: Arc::new(RwLock::new(LspConfigCache {
                configs: HashMap::new(),
                loaded: false,
            })),
        }
    }

    /// Create a loader with the default config path (config/lsp)
    pub fn with_default_path() -> Self {
        let config_dir = Self::find_config_dir().unwrap_or_else(|| PathBuf::from("config/lsp"));
        Self::new(config_dir)
    }

    /// Find the config/lsp directory by searching up the directory tree
    fn find_config_dir() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        // Check common locations
        let candidates = [
            current.join("config").join("lsp"),
            current.join("ricecoder").join("config").join("lsp"),
        ];

        for candidate in &candidates {
            if candidate.is_dir() {
                return Some(candidate.clone());
            }
        }

        // Walk up the directory tree
        loop {
            let lsp_dir = current.join("config").join("lsp");
            if lsp_dir.is_dir() {
                return Some(lsp_dir);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Load all LSP configurations from the config directory
    fn load_all(&self) -> StorageResult<HashMap<String, LspConfig>> {
        // Check cache first
        {
            let cache = self.cache.read().map_err(|_| {
                StorageError::Internal("Failed to acquire read lock on LSP config cache".to_string())
            })?;
            if cache.loaded {
                return Ok(cache.configs.clone());
            }
        }

        // Load from files
        let mut configs = HashMap::new();

        if !self.config_dir.exists() {
            tracing::debug!(
                "LSP config directory not found: {:?}",
                self.config_dir
            );
            
            // Populate with hardcoded fallbacks
            configs.extend(Self::hardcoded_fallbacks());
            
            // Update cache even with fallbacks
            {
                let mut cache = self.cache.write().map_err(|_| {
                    StorageError::Internal("Failed to acquire write lock on LSP config cache".to_string())
                })?;
                cache.configs = configs.clone();
                cache.loaded = true;
            }
            
            return Ok(configs);
        }

        let entries = fs::read_dir(&self.config_dir).map_err(|e| {
            StorageError::io_error(self.config_dir.clone(), IoOperation::Read, e)
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                StorageError::io_error(self.config_dir.clone(), IoOperation::Read, e)
            })?;

            let path = entry.path();

            // Only process .yaml and .yml files
            let ext = path.extension().and_then(|s| s.to_str());
            if ext != Some("yaml") && ext != Some("yml") {
                continue;
            }

            match Self::load_config_file(&path) {
                Ok(config) => {
                    tracing::debug!("Loaded LSP config for language: {}", config.language);
                    configs.insert(config.language.clone(), config);
                }
                Err(e) => {
                    tracing::warn!("Failed to load LSP config {:?}: {}", path, e);
                }
            }
        }

        // Add hardcoded fallbacks for languages not found in config files
        let fallbacks = Self::hardcoded_fallbacks();
        for (language, fallback_config) in fallbacks {
            configs.entry(language).or_insert(fallback_config);
        }

        // Update cache
        {
            let mut cache = self.cache.write().map_err(|_| {
                StorageError::Internal("Failed to acquire write lock on LSP config cache".to_string())
            })?;
            cache.configs = configs.clone();
            cache.loaded = true;
        }

        tracing::info!(
            "Loaded {} LSP configurations from {:?}",
            configs.len(),
            self.config_dir
        );

        Ok(configs)
    }

    /// Load a single LSP configuration file
    fn load_config_file(path: &Path) -> StorageResult<LspConfig> {
        let content = fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), IoOperation::Read, e)
        })?;

        let config: LspConfig = serde_yaml::from_str(&content).map_err(|e| {
            StorageError::parse_error(path.to_path_buf(), "YAML", e.to_string())
        })?;

        // Validate configuration
        Self::validate_config(&config, path)?;

        Ok(config)
    }

    /// Validate LSP configuration
    fn validate_config(config: &LspConfig, path: &Path) -> StorageResult<()> {
        if config.language.is_empty() {
            return Err(StorageError::validation_error(
                "language",
                format!("LSP config {:?} has empty language field", path)
            ));
        }

        if config.executable.is_empty() {
            return Err(StorageError::validation_error(
                "executable",
                format!("LSP config for '{}' has empty executable", config.language)
            ));
        }

        if config.extensions.is_empty() {
            return Err(StorageError::validation_error(
                "extensions",
                format!("LSP config for '{}' has no file extensions", config.language)
            ));
        }

        if config.timeout_ms == 0 {
            return Err(StorageError::validation_error(
                "timeout_ms",
                format!("LSP config for '{}' has invalid timeout_ms: 0", config.language)
            ));
        }

        Ok(())
    }

    /// Get hardcoded fallback configurations for common languages
    fn hardcoded_fallbacks() -> HashMap<String, LspConfig> {
        let mut fallbacks = HashMap::new();

        // Rust - rust-analyzer
        fallbacks.insert(
            "rust".to_string(),
            LspConfig {
                language: "rust".to_string(),
                extensions: vec![".rs".to_string()],
                executable: "rust-analyzer".to_string(),
                args: vec![],
                env: HashMap::new(),
                init_options: None,
                enabled: true,
                timeout_ms: 10000,
                max_restarts: 3,
                idle_timeout_ms: 300000,
            },
        );

        // TypeScript/JavaScript - typescript-language-server
        fallbacks.insert(
            "typescript".to_string(),
            LspConfig {
                language: "typescript".to_string(),
                extensions: vec![
                    ".ts".to_string(),
                    ".tsx".to_string(),
                    ".js".to_string(),
                    ".jsx".to_string(),
                ],
                executable: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                env: HashMap::new(),
                init_options: None,
                enabled: true,
                timeout_ms: 5000,
                max_restarts: 3,
                idle_timeout_ms: 300000,
            },
        );

        // Python - pylsp
        fallbacks.insert(
            "python".to_string(),
            LspConfig {
                language: "python".to_string(),
                extensions: vec![".py".to_string()],
                executable: "pylsp".to_string(),
                args: vec![],
                env: HashMap::new(),
                init_options: None,
                enabled: true,
                timeout_ms: 5000,
                max_restarts: 3,
                idle_timeout_ms: 300000,
            },
        );

        // Go - gopls
        fallbacks.insert(
            "go".to_string(),
            LspConfig {
                language: "go".to_string(),
                extensions: vec![".go".to_string()],
                executable: "gopls".to_string(),
                args: vec!["serve".to_string()],
                env: HashMap::new(),
                init_options: None,
                enabled: true,
                timeout_ms: 5000,
                max_restarts: 3,
                idle_timeout_ms: 300000,
            },
        );

        // C/C++ - clangd
        fallbacks.insert(
            "c".to_string(),
            LspConfig {
                language: "c".to_string(),
                extensions: vec![".c".to_string(), ".h".to_string()],
                executable: "clangd".to_string(),
                args: vec![],
                env: HashMap::new(),
                init_options: None,
                enabled: true,
                timeout_ms: 10000,
                max_restarts: 3,
                idle_timeout_ms: 300000,
            },
        );

        fallbacks.insert(
            "cpp".to_string(),
            LspConfig {
                language: "cpp".to_string(),
                extensions: vec![".cpp".to_string(), ".hpp".to_string(), ".cc".to_string()],
                executable: "clangd".to_string(),
                args: vec![],
                env: HashMap::new(),
                init_options: None,
                enabled: true,
                timeout_ms: 10000,
                max_restarts: 3,
                idle_timeout_ms: 300000,
            },
        );

        fallbacks
    }

    /// Get configuration for a specific language
    ///
    /// Returns None if no configuration is found (neither external nor hardcoded fallback)
    pub fn get_config(&self, language: &str) -> Option<LspConfig> {
        // Try cache first
        {
            if let Ok(cache) = self.cache.read() {
                if cache.loaded {
                    return cache.configs.get(language).cloned();
                }
            }
        }

        // Load and try again
        if let Ok(configs) = self.load_all() {
            return configs.get(language).cloned();
        }

        None
    }

    /// Get all LSP configurations
    pub fn get_all(&self) -> StorageResult<HashMap<String, LspConfig>> {
        self.load_all()
    }

    /// Check if a language has an LSP configuration
    pub fn has_config(&self, language: &str) -> bool {
        self.get_config(language).is_some()
    }

    /// Reload configurations from disk (clears cache)
    pub fn reload(&self) -> StorageResult<()> {
        // Clear cache
        {
            let mut cache = self.cache.write().map_err(|_| {
                StorageError::Internal("Failed to acquire write lock on LSP config cache".to_string())
            })?;
            cache.configs.clear();
            cache.loaded = false;
        }

        // Reload
        self.load_all()?;
        Ok(())
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}

/// Global LSP config loader singleton
static GLOBAL_LOADER: std::sync::OnceLock<LspConfigLoader> = std::sync::OnceLock::new();

/// Get the global LSP config loader
pub fn global_lsp_configs() -> &'static LspConfigLoader {
    GLOBAL_LOADER.get_or_init(LspConfigLoader::with_default_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_lsp_config() {
        let temp_dir = TempDir::new().unwrap();
        let lsp_dir = temp_dir.path().join("lsp");
        fs::create_dir_all(&lsp_dir).unwrap();

        // Create test config file
        let config_content = r#"
language: rust
extensions:
  - .rs
executable: rust-analyzer
args: []
env: {}
init_options:
  checkOnSave:
    command: clippy
enabled: true
timeout_ms: 10000
max_restarts: 3
idle_timeout_ms: 300000
"#;
        let mut file = fs::File::create(lsp_dir.join("rust-analyzer.yaml")).unwrap();
        write!(file, "{}", config_content).unwrap();

        let loader = LspConfigLoader::new(lsp_dir);
        let config = loader.get_config("rust").unwrap();

        assert_eq!(config.language, "rust");
        assert_eq!(config.executable, "rust-analyzer");
        assert!(config.extensions.contains(&".rs".to_string()));
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_fallback_configs() {
        let temp_dir = TempDir::new().unwrap();
        let lsp_dir = temp_dir.path().join("lsp");
        fs::create_dir_all(&lsp_dir).unwrap();

        let loader = LspConfigLoader::new(lsp_dir);
        
        // Should use hardcoded fallback for rust
        let config = loader.get_config("rust").unwrap();
        assert_eq!(config.language, "rust");
        assert_eq!(config.executable, "rust-analyzer");
    }

    #[test]
    fn test_get_all_configs() {
        let temp_dir = TempDir::new().unwrap();
        let lsp_dir = temp_dir.path().join("lsp");
        fs::create_dir_all(&lsp_dir).unwrap();

        let loader = LspConfigLoader::new(lsp_dir);
        let configs = loader.get_all().unwrap();

        // Should have at least the hardcoded fallbacks
        assert!(configs.contains_key("rust"));
        assert!(configs.contains_key("typescript"));
        assert!(configs.contains_key("python"));
    }

    #[test]
    fn test_cache_works() {
        let temp_dir = TempDir::new().unwrap();
        let lsp_dir = temp_dir.path().join("lsp");
        fs::create_dir_all(&lsp_dir).unwrap();

        let config_content = r#"
language: test
extensions:
  - .test
executable: test-lsp
args: []
env: {}
enabled: true
timeout_ms: 5000
max_restarts: 3
idle_timeout_ms: 300000
"#;
        let mut file = fs::File::create(lsp_dir.join("test.yaml")).unwrap();
        write!(file, "{}", config_content).unwrap();

        let loader = LspConfigLoader::new(lsp_dir.clone());

        // First load
        let config1 = loader.get_config("test");
        assert!(config1.is_some());

        // Modify file (won't be picked up due to cache)
        let modified_content = config_content.replace("timeout_ms: 5000", "timeout_ms: 10000");
        let mut file = fs::File::create(lsp_dir.join("test.yaml")).unwrap();
        write!(file, "{}", modified_content).unwrap();

        let config2 = loader.get_config("test");
        assert_eq!(config1.unwrap().timeout_ms, config2.unwrap().timeout_ms); // Still cached

        // Reload
        loader.reload().unwrap();
        let config3 = loader.get_config("test");
        assert_eq!(config3.unwrap().timeout_ms, 10000); // Updated
    }

    #[test]
    fn test_nonexistent_directory() {
        let loader = LspConfigLoader::new(PathBuf::from("/nonexistent/path"));
        let configs = loader.get_all().unwrap();

        // Should still have hardcoded fallbacks
        assert!(!configs.is_empty());
        assert!(configs.contains_key("rust"));
    }

    #[test]
    fn test_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let lsp_dir = temp_dir.path().join("lsp");
        fs::create_dir_all(&lsp_dir).unwrap();

        let mut file = fs::File::create(lsp_dir.join("invalid.yaml")).unwrap();
        write!(file, "invalid: [yaml").unwrap();

        let loader = LspConfigLoader::new(lsp_dir);
        let configs = loader.get_all().unwrap();

        // Should skip invalid file but load fallbacks
        assert!(!configs.is_empty());
    }
}
