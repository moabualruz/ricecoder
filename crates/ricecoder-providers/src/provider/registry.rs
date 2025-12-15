//! Provider registry for dynamic provider registration and discovery

use std::collections::HashMap;
use std::sync::Arc;

use super::Provider;
use crate::error::ProviderError;
use crate::models::ModelInfo;

/// Registry for managing available providers
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a new provider
    pub fn register(&mut self, provider: Arc<dyn Provider>) -> Result<(), ProviderError> {
        let id = provider.id().to_string();
        self.providers.insert(id, provider);
        Ok(())
    }

    /// Unregister a provider by ID
    pub fn unregister(&mut self, provider_id: &str) -> Result<(), ProviderError> {
        self.providers
            .remove(provider_id)
            .ok_or_else(|| ProviderError::NotFound(provider_id.to_string()))?;
        Ok(())
    }

    /// Get a provider by ID
    pub fn get(&self, provider_id: &str) -> Result<Arc<dyn Provider>, ProviderError> {
        self.providers
            .get(provider_id)
            .cloned()
            .ok_or_else(|| ProviderError::NotFound(provider_id.to_string()))
    }

    /// Get a provider by name
    pub fn get_by_name(&self, name: &str) -> Result<Arc<dyn Provider>, ProviderError> {
        self.providers
            .values()
            .find(|p| p.name() == name)
            .cloned()
            .ok_or_else(|| ProviderError::NotFound(name.to_string()))
    }

    /// Get all registered providers
    pub fn list_all(&self) -> Vec<Arc<dyn Provider>> {
        self.providers.values().cloned().collect()
    }

    /// Get all provider IDs
    pub fn list_provider_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get all available models across all providers
    pub fn list_all_models(&self) -> Vec<ModelInfo> {
        self.providers
            .values()
            .flat_map(|provider| provider.models())
            .collect()
    }

    /// Get models for a specific provider
    pub fn list_models(&self, provider_id: &str) -> Result<Vec<ModelInfo>, ProviderError> {
        let provider = self.get(provider_id)?;
        Ok(provider.models())
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, provider_id: &str) -> bool {
        self.providers.contains_key(provider_id)
    }

    /// Get the number of registered providers
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}


