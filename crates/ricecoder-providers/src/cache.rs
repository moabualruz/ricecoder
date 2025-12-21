//! Provider response caching layer
//!
//! Caches AI provider responses to avoid redundant API calls.
//! Uses file-based cache with TTL support.

use crate::error::ProviderError;
use crate::models::{ChatRequest, ChatResponse};
use ricecoder_storage::{CacheInvalidationStrategy, CacheManager};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Provider response cache
///
/// Caches AI provider responses to improve performance and reduce API calls.
/// Uses SHA256 hash of request to create cache keys.
pub struct ProviderCache {
    cache: Arc<CacheManager>,
    ttl_seconds: u64,
}

impl ProviderCache {
    /// Create a new provider cache
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory to store cache files
    /// * `ttl_seconds` - Time-to-live for cache entries (default: 86400 = 24 hours)
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created
    pub fn new(cache_dir: impl AsRef<Path>, ttl_seconds: u64) -> Result<Self, ProviderError> {
        let cache = CacheManager::new(cache_dir)
            .map_err(|e| ProviderError::Internal(format!("Failed to create cache: {}", e)))?;

        Ok(Self {
            cache: Arc::new(cache),
            ttl_seconds,
        })
    }

    /// Get a cached response
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name (e.g., "openai", "anthropic")
    /// * `model` - Model name (e.g., "gpt-4", "claude-3")
    /// * `request` - Chat request
    ///
    /// # Returns
    ///
    /// Returns cached response if found and not expired, None otherwise
    pub fn get(
        &self,
        provider: &str,
        model: &str,
        request: &ChatRequest,
    ) -> Result<Option<ChatResponse>, ProviderError> {
        let cache_key = self.make_cache_key(provider, model, request);

        match self.cache.get(&cache_key) {
            Ok(Some(cached_json_str)) => {
                match serde_json::from_str::<ChatResponse>(&cached_json_str) {
                    Ok(response) => {
                        debug!("Cache hit for provider response: {}/{}", provider, model);
                        Ok(Some(response))
                    }
                    Err(e) => {
                        debug!("Failed to deserialize cached response: {}", e);
                        // Invalidate corrupted cache entry
                        let _ = self.cache.invalidate(&cache_key);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Cache miss for provider response: {}/{}", provider, model);
                Ok(None)
            }
            Err(e) => {
                debug!("Cache lookup error: {}", e);
                Ok(None)
            }
        }
    }

    /// Cache a response
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name
    /// * `model` - Model name
    /// * `request` - Chat request
    /// * `response` - Chat response to cache
    ///
    /// # Errors
    ///
    /// Returns error if response cannot be cached
    pub fn set(
        &self,
        provider: &str,
        model: &str,
        request: &ChatRequest,
        response: &ChatResponse,
    ) -> Result<(), ProviderError> {
        let cache_key = self.make_cache_key(provider, model, request);

        let response_json = serde_json::to_string(response)
            .map_err(|e| ProviderError::Internal(format!("Failed to serialize response: {}", e)))?;

        let json_len = response_json.len();

        self.cache
            .set(
                &cache_key,
                response_json,
                CacheInvalidationStrategy::Ttl(self.ttl_seconds),
            )
            .map_err(|e| ProviderError::Internal(format!("Failed to cache response: {}", e)))?;

        debug!(
            "Cached response for {}/{}: {} bytes",
            provider, model, json_len
        );

        Ok(())
    }

    /// Invalidate a cached response
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name
    /// * `model` - Model name
    /// * `request` - Chat request
    ///
    /// # Returns
    ///
    /// Returns Ok(true) if entry was deleted, Ok(false) if entry didn't exist
    pub fn invalidate(
        &self,
        provider: &str,
        model: &str,
        request: &ChatRequest,
    ) -> Result<bool, ProviderError> {
        let cache_key = self.make_cache_key(provider, model, request);

        self.cache
            .invalidate(&cache_key)
            .map_err(|e| ProviderError::Internal(format!("Failed to invalidate cache: {}", e)))
    }

    /// Clear all cached responses
    ///
    /// # Errors
    ///
    /// Returns error if cache cannot be cleared
    pub fn clear(&self) -> Result<(), ProviderError> {
        self.cache
            .clear()
            .map_err(|e| ProviderError::Internal(format!("Failed to clear cache: {}", e)))
    }

    /// Clean up expired cache entries
    ///
    /// # Returns
    ///
    /// Returns the number of entries cleaned up
    pub fn cleanup_expired(&self) -> Result<usize, ProviderError> {
        let cleaned = self
            .cache
            .cleanup_expired()
            .map_err(|e| ProviderError::Internal(format!("Failed to cleanup cache: {}", e)))?;

        if cleaned > 0 {
            info!("Cleaned up {} expired cache entries", cleaned);
        }

        Ok(cleaned)
    }

    /// Create a cache key from provider, model, and request
    fn make_cache_key(&self, provider: &str, model: &str, request: &ChatRequest) -> String {
        // Create a deterministic hash of the request
        let request_json = serde_json::to_string(request).unwrap_or_default();

        let mut hasher = Sha256::new();
        hasher.update(provider.as_bytes());
        hasher.update(b"|");
        hasher.update(model.as_bytes());
        hasher.update(b"|");
        hasher.update(request_json.as_bytes());

        let hash = format!("{:x}", hasher.finalize());

        format!("provider_response_{}", hash)
    }
}
