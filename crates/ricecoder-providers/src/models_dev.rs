//! Models.dev API integration for dynamic model discovery
//!
//! This module fetches model metadata from https://models.dev/api.json
//! with caching, fallback support, and merge logic for local overrides.
//!
//! # Features
//!
//! - Async HTTP fetching with timeouts
//! - 24-hour file-based cache in user config directory
//! - Merge strategy: local config overrides API models
//! - Graceful fallback on network failures

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::{error::ProviderError, models::ModelInfo, models::Capability, models::Pricing};

/// Models.dev API URL
const MODELS_DEV_API: &str = "https://models.dev/api.json";

/// Cache TTL (24 hours as per requirements)
const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Fetch timeout (10 seconds)
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Models.dev API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevResponse {
    pub models: Vec<ModelsDevModel>,
}

/// Model metadata from models.dev
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: usize,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub experimental: bool,
}

/// Cached models.dev response with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevCache {
    pub models: Vec<ModelInfo>,
    pub fetched_at: SystemTime,
}

impl ModelsDevCache {
    /// Check if cache is still valid (within TTL)
    pub fn is_valid(&self) -> bool {
        if let Ok(elapsed) = self.fetched_at.elapsed() {
            elapsed < CACHE_TTL
        } else {
            false
        }
    }
}

/// Models fetcher with caching and merge capabilities
pub struct ModelsFetcher {
    cache_path: PathBuf,
    http_client: reqwest::Client,
}

impl ModelsFetcher {
    /// Create a new fetcher with default cache directory
    ///
    /// Cache is stored in `~/.ricecoder/cache/models.json`
    pub fn new() -> Result<Self, ProviderError> {
        let cache_dir = Self::get_cache_directory()?;
        let cache_path = cache_dir.join("models.json");
        
        let http_client = reqwest::Client::builder()
            .timeout(FETCH_TIMEOUT)
            .build()
            .map_err(|e| ProviderError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            cache_path,
            http_client,
        })
    }

    /// Create a fetcher with custom cache path
    pub fn with_cache_path(cache_path: PathBuf) -> Result<Self, ProviderError> {
        let http_client = reqwest::Client::builder()
            .timeout(FETCH_TIMEOUT)
            .build()
            .map_err(|e| ProviderError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            cache_path,
            http_client,
        })
    }

    /// Get the default cache directory (~/.ricecoder/cache)
    fn get_cache_directory() -> Result<PathBuf, ProviderError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| ProviderError::ConfigError("Cannot determine config directory".to_string()))?;
        
        let cache_dir = config_dir.join("ricecoder").join("cache");
        
        // Ensure directory exists
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).map_err(|e| {
                ProviderError::ConfigError(format!("Failed to create cache directory: {}", e))
            })?;
        }

        Ok(cache_dir)
    }

    /// Fetch models from models.dev API
    pub async fn fetch(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        tracing::info!("Fetching models from {}", MODELS_DEV_API);
        
        let response = self.http_client
            .get(MODELS_DEV_API)
            .header("User-Agent", "ricecoder/0.1")
            .send()
            .await
            .map_err(|e| {
                tracing::warn!("Failed to fetch from models.dev: {}", e);
                ProviderError::NetworkError(e.to_string())
            })?;

        let api_response: ModelsDevResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::warn!("Failed to parse models.dev response: {}", e);
                ProviderError::SerializationError(e.to_string())
            })?;

        let models: Vec<ModelInfo> = api_response
            .models
            .into_iter()
            .map(convert_model)
            .collect();

        tracing::info!("Fetched {} models from models.dev", models.len());
        Ok(models)
    }

    /// Get cached models if available and valid
    pub fn get_cached(&self) -> Option<Vec<ModelInfo>> {
        if !self.cache_path.exists() {
            tracing::debug!("Cache file does not exist: {:?}", self.cache_path);
            return None;
        }

        match fs::read_to_string(&self.cache_path) {
            Ok(content) => {
                match serde_json::from_str::<ModelsDevCache>(&content) {
                    Ok(cache) => {
                        if cache.is_valid() {
                            tracing::info!("Using valid cached models ({} models)", cache.models.len());
                            Some(cache.models)
                        } else {
                            tracing::info!("Cache expired, will fetch fresh data");
                            None
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cache: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read cache file: {}", e);
                None
            }
        }
    }

    /// Save models to cache
    pub fn save_cache(&self, models: &[ModelInfo]) -> Result<(), ProviderError> {
        let cache = ModelsDevCache {
            models: models.to_vec(),
            fetched_at: SystemTime::now(),
        };

        let content = serde_json::to_string_pretty(&cache)
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        fs::write(&self.cache_path, content)
            .map_err(|e| ProviderError::ConfigError(format!("Failed to write cache: {}", e)))?;

        tracing::info!("Cached {} models to {:?}", models.len(), self.cache_path);
        Ok(())
    }

    /// Merge API models with local models (local overrides API)
    ///
    /// Strategy:
    /// - Start with all API models
    /// - For each local model with matching ID, replace API model
    /// - Add any local models not in API
    pub fn merge_models(api_models: Vec<ModelInfo>, local_models: Vec<ModelInfo>) -> Vec<ModelInfo> {
        let mut merged = HashMap::new();

        // Start with API models
        for model in api_models {
            merged.insert(model.id.clone(), model);
        }

        // Override with local models (local wins)
        for model in local_models {
            tracing::debug!(
                "Local override for model: {} (provider: {})",
                model.id,
                model.provider
            );
            merged.insert(model.id.clone(), model);
        }

        merged.into_values().collect()
    }

    /// Fetch models with cache fallback
    ///
    /// 1. Try to get from cache
    /// 2. If cache miss or expired, fetch from API
    /// 3. Save to cache on successful fetch
    /// 4. Return models or error
    pub async fn fetch_with_cache(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        // Try cache first
        if let Some(cached) = self.get_cached() {
            return Ok(cached);
        }

        // Fetch from API
        let models = self.fetch().await?;
        
        // Save to cache (log but don't fail on cache errors)
        if let Err(e) = self.save_cache(&models) {
            tracing::warn!("Failed to save cache: {}", e);
        }

        Ok(models)
    }
}

impl Default for ModelsFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create ModelsFetcher")
    }
}

/// Fetch models from models.dev API (compatibility function)
pub async fn fetch_models(http_client: &reqwest::Client) -> Result<Vec<ModelInfo>, ProviderError> {
    let response = http_client
        .get(MODELS_DEV_API)
        .timeout(FETCH_TIMEOUT)
        .header("User-Agent", "ricecoder/0.1")
        .send()
        .await
        .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

    let api_response: ModelsDevResponse = response
        .json()
        .await
        .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

    Ok(api_response.models.into_iter().map(convert_model).collect())
}

/// Convert models.dev model to ModelInfo
fn convert_model(model: ModelsDevModel) -> ModelInfo {
    let capabilities = model
        .capabilities
        .iter()
        .filter_map(|c| match c.as_str() {
            "chat" => Some(Capability::Chat),
            "code" => Some(Capability::Code),
            "vision" => Some(Capability::Vision),
            "function_calling" => Some(Capability::FunctionCalling),
            "streaming" => Some(Capability::Streaming),
            _ => None,
        })
        .collect();

    ModelInfo {
        id: model.id,
        name: model.name,
        provider: model.provider,
        context_window: model.context_window,
        capabilities,
        pricing: None,
        is_free: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_cache_validity() {
        let cache = ModelsDevCache {
            models: vec![],
            fetched_at: SystemTime::now(),
        };
        assert!(cache.is_valid());
    }

    #[test]
    fn test_cache_expiry() {
        let cache = ModelsDevCache {
            models: vec![],
            fetched_at: SystemTime::now() - StdDuration::from_secs(25 * 60 * 60), // 25 hours ago
        };
        assert!(!cache.is_valid());
    }

    #[test]
    fn test_convert_model() {
        let model = ModelsDevModel {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            provider: "test".to_string(),
            context_window: 4096,
            capabilities: vec!["chat".to_string(), "code".to_string()],
            status: "stable".to_string(),
            experimental: false,
        };

        let info = convert_model(model);
        assert_eq!(info.id, "test-model");
        assert_eq!(info.name, "Test Model");
        assert_eq!(info.provider, "test");
        assert_eq!(info.context_window, 4096);
        assert_eq!(info.capabilities.len(), 2);
        assert!(info.capabilities.contains(&Capability::Chat));
        assert!(info.capabilities.contains(&Capability::Code));
    }

    #[test]
    fn test_merge_models_local_overrides_api() {
        let api_models = vec![
            ModelInfo {
                id: "model-1".to_string(),
                name: "API Model 1".to_string(),
                provider: "api-provider".to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat],
                pricing: None,
                is_free: false,
            },
            ModelInfo {
                id: "model-2".to_string(),
                name: "API Model 2".to_string(),
                provider: "api-provider".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Code],
                pricing: None,
                is_free: false,
            },
        ];

        let local_models = vec![
            ModelInfo {
                id: "model-1".to_string(),
                name: "Local Model 1".to_string(),
                provider: "local-provider".to_string(),
                context_window: 16384,
                capabilities: vec![Capability::Chat, Capability::Vision],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.01,
                    output_per_1k_tokens: 0.03,
                }),
                is_free: true,
            },
        ];

        let merged = ModelsFetcher::merge_models(api_models, local_models);
        
        assert_eq!(merged.len(), 2);
        
        // Find model-1 in merged
        let model1 = merged.iter().find(|m| m.id == "model-1").unwrap();
        assert_eq!(model1.name, "Local Model 1"); // Local overrides API
        assert_eq!(model1.context_window, 16384);
        assert!(model1.is_free);
        assert!(model1.pricing.is_some());
        
        // model-2 should remain from API
        let model2 = merged.iter().find(|m| m.id == "model-2").unwrap();
        assert_eq!(model2.name, "API Model 2");
        assert_eq!(model2.context_window, 8192);
    }

    #[test]
    fn test_merge_models_adds_local_only() {
        let api_models = vec![
            ModelInfo {
                id: "api-only".to_string(),
                name: "API Only Model".to_string(),
                provider: "api".to_string(),
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                is_free: false,
            },
        ];

        let local_models = vec![
            ModelInfo {
                id: "local-only".to_string(),
                name: "Local Only Model".to_string(),
                provider: "local".to_string(),
                context_window: 8192,
                capabilities: vec![],
                pricing: None,
                is_free: true,
            },
        ];

        let merged = ModelsFetcher::merge_models(api_models, local_models);
        
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|m| m.id == "api-only"));
        assert!(merged.iter().any(|m| m.id == "local-only"));
    }

    #[test]
    fn test_merge_models_empty_api() {
        let api_models = vec![];
        let local_models = vec![
            ModelInfo {
                id: "local-1".to_string(),
                name: "Local Model".to_string(),
                provider: "local".to_string(),
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                is_free: false,
            },
        ];

        let merged = ModelsFetcher::merge_models(api_models, local_models);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "local-1");
    }

    #[test]
    fn test_merge_models_empty_local() {
        let api_models = vec![
            ModelInfo {
                id: "api-1".to_string(),
                name: "API Model".to_string(),
                provider: "api".to_string(),
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                is_free: false,
            },
        ];
        let local_models = vec![];

        let merged = ModelsFetcher::merge_models(api_models, local_models);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "api-1");
    }

    #[tokio::test]
    async fn test_fetcher_cache_operations() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let cache_path = temp_dir.path().join("test_models.json");
        
        let fetcher = ModelsFetcher::with_cache_path(cache_path.clone()).unwrap();
        
        // Initially no cache
        assert!(fetcher.get_cached().is_none());
        
        // Save some models
        let models = vec![
            ModelInfo {
                id: "test".to_string(),
                name: "Test".to_string(),
                provider: "test".to_string(),
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                is_free: false,
            },
        ];
        
        fetcher.save_cache(&models).unwrap();
        
        // Should be able to retrieve
        let cached = fetcher.get_cached().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].id, "test");
    }
}
