//! Model registry for dynamic model loading
//!
//! This module provides a centralized registry for loading and converting
//! model definitions from config/models.json into ModelInfo structures
//! that providers can use.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use crate::{
    error::ProviderError,
    models::{Capability, ModelInfo, Pricing},
};

/// Model registry that loads from config/models.json
pub struct ModelRegistry {
    /// Cached models by provider ID
    models_cache: Arc<RwLock<HashMap<String, Vec<ModelInfo>>>>,
    /// Last refresh timestamp
    last_refresh: Arc<RwLock<Option<SystemTime>>>,
    /// Cache TTL
    cache_ttl: Duration,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            models_cache: Arc::new(RwLock::new(HashMap::new())),
            last_refresh: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(60 * 60), // 1 hour
        }
    }

    /// Create a new model registry with custom cache TTL
    pub fn with_cache_ttl(ttl: Duration) -> Self {
        Self {
            models_cache: Arc::new(RwLock::new(HashMap::new())),
            last_refresh: Arc::new(RwLock::new(None)),
            cache_ttl: ttl,
        }
    }

    /// Check if cache needs refresh
    fn needs_refresh(&self) -> bool {
        let last = self.last_refresh.read().unwrap();
        match *last {
            None => true,
            Some(timestamp) => {
                SystemTime::now()
                    .duration_since(timestamp)
                    .map(|d| d > self.cache_ttl)
                    .unwrap_or(true)
            }
        }
    }

    /// Load models from config/models.json using ModelLoader
    fn load_models(&self) -> Result<HashMap<String, Vec<ModelInfo>>, ProviderError> {
        use ricecoder_storage::loaders::ModelLoader;

        let loader = ModelLoader::with_default_path();
        let providers = loader
            .load_with_cache()
            .map_err(|e| ProviderError::ConfigError(format!("Failed to load models: {}", e)))?;

        let mut models_by_provider = HashMap::new();

        for (provider_id, provider) in providers {
            let models: Vec<ModelInfo> = provider
                .models
                .into_iter()
                .map(|m| Self::convert_model(&provider_id, m))
                .collect();

            models_by_provider.insert(provider_id, models);
        }

        Ok(models_by_provider)
    }

    /// Load models with optional models.dev API integration
    ///
    /// When `models-api` feature is enabled:
    /// 1. Try to fetch from models.dev API (with cache)
    /// 2. Merge with local models (local overrides API)
    /// 3. Fall back to local-only on API failure
    #[cfg(feature = "models-api")]
    fn load_models_with_api(&self) -> Result<HashMap<String, Vec<ModelInfo>>, ProviderError> {
        use crate::models_dev::ModelsFetcher;
        use ricecoder_storage::loaders::ModelLoader;

        // Load local models first
        let local_models = self.load_models()?;
        let local_flat: Vec<ModelInfo> = local_models
            .values()
            .flat_map(|models| models.iter().cloned())
            .collect();

        // Try to fetch from API
        let fetcher = match ModelsFetcher::new() {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("Failed to create ModelsFetcher, using local models only: {}", e);
                return Ok(local_models);
            }
        };

        // Attempt async fetch (we need to block here since load_models is sync)
        let api_models = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                match handle.block_on(fetcher.fetch_with_cache()) {
                    Ok(models) => {
                        tracing::info!("Successfully fetched {} models from models.dev API", models.len());
                        models
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch from models.dev API, using local models only: {}", e);
                        return Ok(local_models);
                    }
                }
            }
            Err(_) => {
                tracing::warn!("No tokio runtime available, using local models only");
                return Ok(local_models);
            }
        };

        // Merge: local overrides API
        let merged = ModelsFetcher::merge_models(api_models, local_flat);

        // Group by provider
        let mut models_by_provider: HashMap<String, Vec<ModelInfo>> = HashMap::new();
        for model in merged {
            models_by_provider
                .entry(model.provider.clone())
                .or_insert_with(Vec::new)
                .push(model);
        }

        tracing::info!(
            "Merged models from API and local config: {} total models across {} providers",
            models_by_provider.values().map(|v| v.len()).sum::<usize>(),
            models_by_provider.len()
        );

        Ok(models_by_provider)
    }

    /// Convert storage::Model to ModelInfo
    fn convert_model(
        provider_id: &str,
        model: ricecoder_storage::loaders::Model,
    ) -> ModelInfo {
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

        let pricing = model.pricing.map(|p| Pricing {
            input_per_1k_tokens: p.input_per_1k,
            output_per_1k_tokens: p.output_per_1k,
        });

        ModelInfo {
            id: model.id,
            name: model.name,
            provider: provider_id.to_string(),
            context_window: model.context_window,
            capabilities,
            pricing,
            is_free: model.is_free,
        }
    }

    /// Refresh the cache
    fn refresh_cache(&self) -> Result<(), ProviderError> {
        #[cfg(feature = "models-api")]
        let models = self.load_models_with_api()?;
        
        #[cfg(not(feature = "models-api"))]
        let models = self.load_models()?;

        {
            let mut cache = self.models_cache.write().unwrap();
            *cache = models;
        }

        {
            let mut last = self.last_refresh.write().unwrap();
            *last = Some(SystemTime::now());
        }

        Ok(())
    }

    /// Get models for a specific provider
    ///
    /// This method is safe to call from Provider::models() as it's synchronous
    /// and handles caching internally.
    pub fn get_provider_models(&self, provider_id: &str) -> Vec<ModelInfo> {
        // Check if refresh needed
        if self.needs_refresh() {
            // Attempt refresh (failures are logged but don't block)
            if let Err(e) = self.refresh_cache() {
                tracing::warn!(
                    provider = provider_id,
                    error = %e,
                    "Failed to refresh model cache, using stale data if available"
                );
            }
        }

        // Return cached models
        let cache = self.models_cache.read().unwrap();
        cache.get(provider_id).cloned().unwrap_or_default()
    }

    /// Get all models across all providers
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        if self.needs_refresh() {
            let _ = self.refresh_cache();
        }

        let cache = self.models_cache.read().unwrap();
        cache
            .values()
            .flat_map(|models| models.iter().cloned())
            .collect()
    }

    /// Force a cache refresh
    pub fn force_refresh(&self) -> Result<(), ProviderError> {
        self.refresh_cache()
    }

    /// Get a specific model by ID (searches all providers)
    pub fn find_model(&self, model_id: &str) -> Option<ModelInfo> {
        if self.needs_refresh() {
            let _ = self.refresh_cache();
        }

        let cache = self.models_cache.read().unwrap();
        cache
            .values()
            .flat_map(|models| models.iter())
            .find(|m| m.id == model_id)
            .cloned()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global model registry instance
static GLOBAL_REGISTRY: once_cell::sync::Lazy<ModelRegistry> =
    once_cell::sync::Lazy::new(ModelRegistry::new);

/// Get the global model registry instance
pub fn global_registry() -> &'static ModelRegistry {
    &GLOBAL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ModelRegistry::new();
        assert!(registry.needs_refresh());
    }

    #[test]
    fn test_cache_ttl_configuration() {
        let registry = ModelRegistry::with_cache_ttl(Duration::from_secs(300));
        assert!(registry.needs_refresh());
    }

    #[test]
    fn test_global_registry() {
        let registry = global_registry();
        // Should be able to access it multiple times
        let registry2 = global_registry();
        assert!(std::ptr::eq(registry, registry2));
    }
}
