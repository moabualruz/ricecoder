//! Model loader for RiceCoder
//!
//! Loads AI model definitions from `config/models.json`.
//! Supports both static JSON files and dynamic fetching from models.dev API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::error::{StorageError, StorageResult};

/// Model definition with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Unique model identifier (e.g., "claude-3-opus-20250219")
    pub id: String,
    /// Human-readable name (e.g., "Claude 3 Opus")
    pub name: String,
    /// Context window in tokens
    pub context_window: usize,
    /// Maximum output tokens
    #[serde(default)]
    pub max_output_tokens: Option<usize>,
    /// Model capabilities
    pub capabilities: Vec<String>,
    /// Pricing information
    #[serde(default)]
    pub pricing: Option<ModelPricing>,
    /// Whether model is free to use
    #[serde(default)]
    pub is_free: bool,
}

/// Pricing information for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per 1K input tokens (USD)
    pub input_per_1k: f64,
    /// Cost per 1K output tokens (USD)
    pub output_per_1k: f64,
    /// Cost per 1K cached tokens read (USD)
    #[serde(default)]
    pub cache_read_per_1k: Option<f64>,
    /// Cost per 1K cached tokens written (USD)
    #[serde(default)]
    pub cache_write_per_1k: Option<f64>,
}

/// Provider definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Provider name (e.g., "Anthropic")
    pub name: String,
    /// API base URL
    #[serde(default)]
    pub base_url: Option<String>,
    /// Available models
    pub models: Vec<Model>,
}

/// Root models configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelsConfig {
    /// Schema version
    #[serde(default)]
    pub version: String,
    /// Last update timestamp
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Providers with their models
    pub providers: HashMap<String, Provider>,
}

/// Cache entry for models.dev API response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelsCacheEntry {
    /// Cached providers
    pub providers: HashMap<String, Provider>,
    /// Cache timestamp
    pub cached_at: SystemTime,
}

/// Loader for model configuration files
pub struct ModelLoader {
    config_dir: PathBuf,
    /// Optional cache directory for models.dev responses
    cache_dir: Option<PathBuf>,
    /// Cache TTL in seconds (default: 3600 = 1 hour)
    cache_ttl: Duration,
}

impl ModelLoader {
    /// Create a new model loader with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            cache_dir: None,
            cache_ttl: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Create a model loader using the default config path
    pub fn with_default_path() -> Self {
        use crate::manager::PathResolver;
        use crate::types::StorageDirectory;

        // Use global config path
        if let Ok(global_path) = PathResolver::resolve_global_path() {
            let config_dir = global_path.join(StorageDirectory::Config.dir_name());
            return Self::new(config_dir);
        }

        // Fallback to current directory
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(cwd.join("config"))
    }

    /// Set cache directory for models.dev API responses
    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Self {
        self.cache_dir = Some(cache_dir);
        self
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Load models from models.json file
    pub fn load_from_file(&self) -> StorageResult<HashMap<String, Provider>> {
        let path = self.config_dir.join("models.json");
        
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            StorageError::io_error(path.clone(), crate::error::IoOperation::Read, e)
        })?;

        let config: ModelsConfig = serde_json::from_str(&content).map_err(|e| {
            StorageError::parse_error(path, "JSON", e.to_string())
        })?;

        Ok(config.providers)
    }

    /// Load cached models from cache directory
    fn load_from_cache(&self) -> StorageResult<Option<HashMap<String, Provider>>> {
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir,
            None => return Ok(None),
        };

        let cache_path = cache_dir.join("models_cache.json");
        if !cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_path).map_err(|e| {
            StorageError::io_error(cache_path.clone(), crate::error::IoOperation::Read, e)
        })?;

        let cache_entry: ModelsCacheEntry = serde_json::from_str(&content).map_err(|e| {
            StorageError::parse_error(cache_path, "JSON", e.to_string())
        })?;

        // Check if cache is still valid
        let now = SystemTime::now();
        let age = now
            .duration_since(cache_entry.cached_at)
            .unwrap_or(Duration::from_secs(u64::MAX));

        if age > self.cache_ttl {
            return Ok(None); // Cache expired
        }

        Ok(Some(cache_entry.providers))
    }

    /// Save models to cache
    fn save_to_cache(&self, providers: &HashMap<String, Provider>) -> StorageResult<()> {
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir,
            None => return Ok(()), // No cache directory configured
        };

        // Ensure cache directory exists
        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir).map_err(|e| {
                StorageError::io_error(cache_dir.clone(), crate::error::IoOperation::Write, e)
            })?;
        }

        let cache_path = cache_dir.join("models_cache.json");
        let cache_entry = ModelsCacheEntry {
            providers: providers.clone(),
            cached_at: SystemTime::now(),
        };

        let content = serde_json::to_string_pretty(&cache_entry).map_err(|e| {
            StorageError::Internal(format!("Failed to serialize cache: {}", e))
        })?;

        fs::write(&cache_path, content).map_err(|e| {
            StorageError::io_error(cache_path, crate::error::IoOperation::Write, e)
        })?;

        Ok(())
    }

    /// Load models with cache-first strategy
    ///
    /// 1. Try loading from cache (if not expired)
    /// 2. Fall back to models.json file
    /// 3. Return empty map if neither exists
    pub fn load_with_cache(&self) -> StorageResult<HashMap<String, Provider>> {
        // Try cache first
        if let Some(cached) = self.load_from_cache()? {
            return Ok(cached);
        }

        // Fall back to file
        self.load_from_file()
    }

    /// Get all models from a specific provider
    pub fn get_provider_models(&self, provider_id: &str) -> StorageResult<Vec<Model>> {
        let providers = self.load_with_cache()?;
        
        Ok(providers
            .get(provider_id)
            .map(|p| p.models.clone())
            .unwrap_or_default())
    }

    /// Get a specific model by ID (searches all providers)
    pub fn get_model(&self, model_id: &str) -> StorageResult<Option<(String, Model)>> {
        let providers = self.load_with_cache()?;

        for (provider_id, provider) in providers {
            if let Some(model) = provider.models.iter().find(|m| m.id == model_id) {
                return Ok(Some((provider_id, model.clone())));
            }
        }

        Ok(None)
    }

    /// Get all models across all providers
    pub fn get_all_models(&self) -> StorageResult<Vec<(String, Model)>> {
        let providers = self.load_with_cache()?;
        let mut models = Vec::new();

        for (provider_id, provider) in providers {
            for model in provider.models {
                models.push((provider_id.clone(), model));
            }
        }

        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_loader_creation() {
        let loader = ModelLoader::with_default_path();
        assert!(loader.config_dir.to_string_lossy().contains("config"));
    }

    #[test]
    fn test_cache_ttl_configuration() {
        let loader = ModelLoader::with_default_path()
            .with_cache_ttl(Duration::from_secs(1800));
        assert_eq!(loader.cache_ttl, Duration::from_secs(1800));
    }
}
