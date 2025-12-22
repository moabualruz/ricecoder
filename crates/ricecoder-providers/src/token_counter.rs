//! Token counting utilities for different providers
//!
//! This module provides token counting functionality for various AI providers.
//! For production use with OpenAI, consider using tiktoken-rs for accurate token counting.

use std::{collections::HashMap, sync::Mutex};

use crate::error::ProviderError;

/// Trait for unified token counting across providers
pub trait TokenCounterTrait: Send + Sync {
    /// Count tokens for content in a specific model
    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError>;

    /// Clear the token count cache
    fn clear_cache(&self);

    /// Get cache size
    fn cache_size(&self) -> usize;
}

/// Token counter for estimating token usage
pub struct TokenCounter {
    cache: Mutex<HashMap<String, usize>>,
}

impl TokenCounter {
    /// Create a new token counter
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Count tokens for OpenAI models using estimation
    ///
    /// This uses a heuristic-based approach:
    /// - Average English word is ~4.7 characters
    /// - Average token is ~4 characters
    /// - Special tokens and formatting add overhead
    pub fn count_tokens_openai(&self, content: &str, model: &str) -> usize {
        // Check cache first
        let cache_key = format!("{}:{}", model, content);
        if let Ok(cache) = self.cache.lock() {
            if let Some(&count) = cache.get(&cache_key) {
                return count;
            }
        }

        // Estimate tokens based on model and content
        let estimated = self.estimate_tokens(content, model);

        // Cache the result
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, estimated);
        }

        estimated
    }

    /// Count tokens for content (unified interface)
    ///
    /// This method provides a unified interface for token counting across providers.
    /// It returns a Result type for better error handling.
    pub fn count(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        Ok(self.count_tokens_openai(content, model))
    }

    /// Estimate token count for content
    fn estimate_tokens(&self, content: &str, _model: &str) -> usize {
        if content.is_empty() {
            return 0;
        }

        // Estimate based on character count
        // Heuristic: roughly 1 token per 4 characters
        // This is a conservative estimate that should not exceed content length
        let estimated = (content.len() as f64 / 4.0).ceil() as usize;

        // Ensure at least 1 token for non-empty content
        std::cmp::max(1, estimated)
    }

    /// Clear the token count cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenCounterTrait for TokenCounter {
    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        self.count(content, model)
    }

    fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    fn cache_size(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }
}
