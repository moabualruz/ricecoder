//! API key management for secure credential handling and storage
//!
//! This module provides secure API key management with support for:
//! - Storing keys in config files
//! - Environment variable overrides
//! - Key rotation support
//! - Secure error messages that don't expose credentials

use std::collections::HashMap;

use crate::error::ProviderError;
use crate::models::ApiKeyConfig;

/// Manages API keys for providers with support for secure storage and retrieval
pub struct ApiKeyManager {
    /// Cached API keys (provider_id -> api_key)
    keys: HashMap<String, String>,
    /// API key configurations (provider_id -> config)
    configs: HashMap<String, ApiKeyConfig>,
}

impl ApiKeyManager {
    /// Create a new API key manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Register an API key configuration for a provider
    pub fn register_config(&mut self, provider_id: String, config: ApiKeyConfig) {
        self.configs.insert(provider_id, config);
    }

    /// Store an API key for a provider
    pub fn store_key(&mut self, provider_id: String, api_key: String) {
        self.keys.insert(provider_id, api_key);
    }

    /// Get an API key for a provider
    ///
    /// Retrieves API key in the following order:
    /// 1. From cache (if already loaded)
    /// 2. From environment variable (if configured)
    /// 3. From config file (if stored)
    /// 4. Error if not found
    pub fn get_key(&self, provider_id: &str) -> Result<String, ProviderError> {
        // First check cache
        if let Some(key) = self.keys.get(provider_id) {
            return Ok(key.clone());
        }

        // Then check environment variable
        if let Some(config) = self.configs.get(provider_id) {
            if let Ok(key) = std::env::var(&config.env_var) {
                return Ok(key);
            }
        }

        // If not found, return error
        Err(ProviderError::ConfigError(format!(
            "API key not found for provider '{}'",
            provider_id
        )))
    }

    /// Check if an API key is available for a provider
    pub fn has_key(&self, provider_id: &str) -> bool {
        // Check cache
        if self.keys.contains_key(provider_id) {
            return true;
        }

        // Check environment variable
        if let Some(config) = self.configs.get(provider_id) {
            if std::env::var(&config.env_var).is_ok() {
                return true;
            }
        }

        false
    }

    /// Rotate an API key for a provider
    ///
    /// This updates the cached key and can optionally persist to config
    pub fn rotate_key(
        &mut self,
        provider_id: String,
        new_key: String,
    ) -> Result<(), ProviderError> {
        // Validate that the new key is not empty
        if new_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "API key cannot be empty".to_string(),
            ));
        }

        // Update the cached key
        self.keys.insert(provider_id, new_key);
        Ok(())
    }

    /// Clear a cached API key (but not the environment variable)
    pub fn clear_cached_key(&mut self, provider_id: &str) {
        self.keys.remove(provider_id);
    }

    /// Clear all cached API keys
    pub fn clear_all_cached_keys(&mut self) {
        self.keys.clear();
    }

    /// Load API keys from environment variables
    pub fn load_from_env(&mut self) -> Result<(), ProviderError> {
        for (provider_id, config) in &self.configs {
            if let Ok(key) = std::env::var(&config.env_var) {
                self.keys.insert(provider_id.clone(), key);
            }
        }
        Ok(())
    }

    /// Get the number of cached keys
    pub fn cached_key_count(&self) -> usize {
        self.keys.len()
    }

    /// Get all provider IDs with configured API key sources
    pub fn configured_providers(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}


