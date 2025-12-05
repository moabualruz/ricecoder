//! Token counting utilities for different providers
//!
//! This module provides token counting functionality for various AI providers.
//! For production use with OpenAI, consider using tiktoken-rs for accurate token counting.

use crate::error::ProviderError;
use std::collections::HashMap;
use std::sync::Mutex;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counter_empty_string() {
        let counter = TokenCounter::new();
        assert_eq!(counter.count_tokens_openai("", "gpt-4"), 0);
    }

    #[test]
    fn test_token_counter_simple_text() {
        let counter = TokenCounter::new();
        let tokens = counter.count_tokens_openai("Hello world", "gpt-4");
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_counter_caching() {
        let counter = TokenCounter::new();
        let content = "This is a test message";
        let tokens1 = counter.count_tokens_openai(content, "gpt-4");
        let tokens2 = counter.count_tokens_openai(content, "gpt-4");
        assert_eq!(tokens1, tokens2);
        assert_eq!(counter.cache_size(), 1);
    }

    #[test]
    fn test_token_counter_different_models() {
        let counter = TokenCounter::new();
        let content = "Test content";
        let _tokens_gpt4 = counter.count_tokens_openai(content, "gpt-4");
        let _tokens_gpt35 = counter.count_tokens_openai(content, "gpt-3.5-turbo");
        // Both should be cached
        assert_eq!(counter.cache_size(), 2);
    }

    #[test]
    fn test_token_counter_special_characters() {
        let counter = TokenCounter::new();
        let simple = counter.count_tokens_openai("hello", "gpt-4");
        let with_special = counter.count_tokens_openai("hello!!!???", "gpt-4");
        // Special characters should increase token count
        assert!(with_special >= simple);
    }

    #[test]
    fn test_token_counter_clear_cache() {
        let counter = TokenCounter::new();
        counter.count_tokens_openai("test", "gpt-4");
        assert_eq!(counter.cache_size(), 1);
        counter.clear_cache();
        assert_eq!(counter.cache_size(), 0);
    }
}
