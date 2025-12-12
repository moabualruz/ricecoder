//! Token estimation and tracking for AI conversations
//!
//! This module provides accurate token counting using tiktoken and cost estimation
//! for various AI models. It integrates with the session system to track token usage
//! and provide warnings when approaching limits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tiktoken_rs::{get_bpe_from_model, CoreBPE};
use crate::error::{SessionError, SessionResult};

/// Token estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimate {
    /// Estimated token count
    pub tokens: usize,
    /// Model used for estimation
    pub model: String,
    /// Character count of input
    pub characters: usize,
    /// Estimated cost in USD (if pricing available)
    pub estimated_cost: Option<f64>,
}

/// Token usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Total tokens used in session
    pub total_tokens: usize,
    /// Tokens used for prompts
    pub prompt_tokens: usize,
    /// Tokens used for completions
    pub completion_tokens: usize,
    /// Model being used
    pub model: String,
    /// Estimated cost in USD
    pub estimated_cost: f64,
    /// Token limit for the model
    pub token_limit: usize,
}

/// Token estimator with caching and model support
pub struct TokenEstimator {
    /// Cached tokenizers by model
    tokenizers: HashMap<String, Arc<CoreBPE>>,
    /// Model pricing information
    pricing: HashMap<String, ModelPricing>,
    /// Default model for estimation
    default_model: String,
}

/// Pricing information for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per 1K input tokens (USD)
    pub input_per_1k: f64,
    /// Cost per 1K output tokens (USD)
    pub output_per_1k: f64,
    /// Maximum context window (tokens)
    pub max_tokens: usize,
}

impl Default for ModelPricing {
    fn default() -> Self {
        Self {
            input_per_1k: 0.0015, // GPT-3.5 default
            output_per_1k: 0.002,
            max_tokens: 4096,
        }
    }
}

impl TokenEstimator {
    /// Create a new token estimator
    pub fn new() -> Self {
        let mut estimator = Self {
            tokenizers: HashMap::new(),
            pricing: HashMap::new(),
            default_model: "gpt-3.5-turbo".to_string(),
        };

        // Initialize with common model pricing
        estimator.initialize_pricing();
        estimator
    }

    /// Initialize default pricing for common models
    fn initialize_pricing(&mut self) {
        // GPT-4 pricing
        self.pricing.insert(
            "gpt-4".to_string(),
            ModelPricing {
                input_per_1k: 0.03,
                output_per_1k: 0.06,
                max_tokens: 8192,
            },
        );

        self.pricing.insert(
            "gpt-4-32k".to_string(),
            ModelPricing {
                input_per_1k: 0.06,
                output_per_1k: 0.12,
                max_tokens: 32768,
            },
        );

        // GPT-3.5 pricing
        self.pricing.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                input_per_1k: 0.0015,
                output_per_1k: 0.002,
                max_tokens: 4096,
            },
        );

        self.pricing.insert(
            "gpt-3.5-turbo-16k".to_string(),
            ModelPricing {
                input_per_1k: 0.003,
                output_per_1k: 0.004,
                max_tokens: 16384,
            },
        );

        // Claude pricing (approximate)
        self.pricing.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                input_per_1k: 0.015,
                output_per_1k: 0.075,
                max_tokens: 200000,
            },
        );

        self.pricing.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                input_per_1k: 0.003,
                output_per_1k: 0.015,
                max_tokens: 200000,
            },
        );

        self.pricing.insert(
            "claude-3-haiku".to_string(),
            ModelPricing {
                input_per_1k: 0.00025,
                output_per_1k: 0.00125,
                max_tokens: 200000,
            },
        );
    }

    /// Set default model for estimation
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Add custom pricing for a model
    pub fn add_pricing(&mut self, model: impl Into<String>, pricing: ModelPricing) {
        self.pricing.insert(model.into(), pricing);
    }

    /// Estimate tokens for text using specified model
    pub fn estimate_tokens(&mut self, text: &str, model: Option<&str>) -> SessionResult<TokenEstimate> {
        let model_name = model.map(|s| s.to_string()).unwrap_or_else(|| self.default_model.clone());
        let tokenizer = self.get_tokenizer(&model_name)?;

        let tokens = tokenizer.encode_with_special_tokens(text).len();
        let characters = text.chars().count();

        let estimated_cost = self.pricing.get(&model_name)
            .map(|pricing| (tokens as f64 / 1000.0) * pricing.input_per_1k);

        Ok(TokenEstimate {
            tokens,
            model: model_name,
            characters,
            estimated_cost,
        })
    }

    /// Estimate tokens for a conversation (multiple messages)
    pub fn estimate_conversation_tokens(&mut self, messages: &[crate::models::Message], model: Option<&str>) -> SessionResult<TokenEstimate> {
        let combined_text = messages.iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content()))
            .collect::<Vec<_>>()
            .join("\n");

        self.estimate_tokens(&combined_text, model)
    }

    /// Create token usage tracker for a session
    pub fn create_usage_tracker(&self, model: &str) -> SessionResult<TokenUsageTracker> {
        let pricing = self.pricing.get(model)
            .cloned()
            .unwrap_or_default();

        Ok(TokenUsageTracker {
            model: model.to_string(),
            total_tokens: 0,
            prompt_tokens: 0,
            completion_tokens: 0,
            estimated_cost: 0.0,
            token_limit: pricing.max_tokens,
            pricing,
        })
    }

    /// Get tokenizer for a model (with caching)
    fn get_tokenizer(&mut self, model: &str) -> SessionResult<&Arc<CoreBPE>> {
        if !self.tokenizers.contains_key(model) {
            let bpe = if model.starts_with("gpt-4") || model.starts_with("gpt-3.5") {
                get_bpe_from_model(model)
                    .map_err(|e| SessionError::TokenEstimation(format!("Failed to load tokenizer for {}: {}", model, e)))?
            } else {
                // Fallback to GPT-3.5 tokenizer for other models
                get_bpe_from_model("gpt-3.5-turbo")
                    .map_err(|e| SessionError::TokenEstimation(format!("Failed to load fallback tokenizer: {}", e)))?
            };

            self.tokenizers.insert(model.to_string(), Arc::new(bpe));
        }

        Ok(self.tokenizers.get(model).unwrap())
    }

    /// Get pricing information for a model
    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.pricing.get(model)
    }

    /// Check if a model is approaching token limits
    pub fn check_token_limits(&self, current_tokens: usize, model: &str) -> TokenLimitStatus {
        let pricing = self.pricing.get(model);

        if let Some(pricing) = pricing {
            let percentage = (current_tokens as f64 / pricing.max_tokens as f64) * 100.0;

            if percentage >= 90.0 {
                TokenLimitStatus::Critical
            } else if percentage >= 75.0 {
                TokenLimitStatus::Warning
            } else {
                TokenLimitStatus::Normal
            }
        } else {
            TokenLimitStatus::Unknown
        }
    }
}

/// Token limit status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenLimitStatus {
    /// Within normal limits
    Normal,
    /// Approaching limit (warning)
    Warning,
    /// Near or at limit (critical)
    Critical,
    /// Cannot determine (no pricing info)
    Unknown,
}

impl TokenLimitStatus {
    /// Get display color for status
    pub fn color(&self) -> &'static str {
        match self {
            TokenLimitStatus::Normal => "green",
            TokenLimitStatus::Warning => "yellow",
            TokenLimitStatus::Critical => "red",
            TokenLimitStatus::Unknown => "gray",
        }
    }

    /// Get status symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            TokenLimitStatus::Normal => "✓",
            TokenLimitStatus::Warning => "⚠",
            TokenLimitStatus::Critical => "🚨",
            TokenLimitStatus::Unknown => "?",
        }
    }
}

/// Token usage tracker for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageTracker {
    /// Model being used
    pub model: String,
    /// Total tokens used
    pub total_tokens: usize,
    /// Tokens used for prompts
    pub prompt_tokens: usize,
    /// Tokens used for completions
    pub completion_tokens: usize,
    /// Estimated cost in USD
    pub estimated_cost: f64,
    /// Token limit for the model
    pub token_limit: usize,
    /// Pricing information
    #[serde(skip)]
    pub pricing: ModelPricing,
}

impl TokenUsageTracker {
    /// Record prompt tokens
    pub fn record_prompt(&mut self, tokens: usize) {
        self.prompt_tokens += tokens;
        self.total_tokens += tokens;
        self.estimated_cost += (tokens as f64 / 1000.0) * self.pricing.input_per_1k;
    }

    /// Record completion tokens
    pub fn record_completion(&mut self, tokens: usize) {
        self.completion_tokens += tokens;
        self.total_tokens += tokens;
        self.estimated_cost += (tokens as f64 / 1000.0) * self.pricing.output_per_1k;
    }

    /// Get current usage as TokenUsage
    pub fn current_usage(&self) -> TokenUsage {
        TokenUsage {
            total_tokens: self.total_tokens,
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            model: self.model.clone(),
            estimated_cost: self.estimated_cost,
            token_limit: self.token_limit,
        }
    }

    /// Get usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.token_limit == 0 {
            0.0
        } else {
            (self.total_tokens as f64 / self.token_limit as f64) * 100.0
        }
    }

    /// Check if approaching limits
    pub fn limit_status(&self) -> TokenLimitStatus {
        let percentage = self.usage_percentage();

        if percentage >= 90.0 {
            TokenLimitStatus::Critical
        } else if percentage >= 75.0 {
            TokenLimitStatus::Warning
        } else {
            TokenLimitStatus::Normal
        }
    }

    /// Reset usage counters
    pub fn reset(&mut self) {
        self.total_tokens = 0;
        self.prompt_tokens = 0;
        self.completion_tokens = 0;
        self.estimated_cost = 0.0;
    }
}

impl Default for TokenEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimator_creation() {
        let estimator = TokenEstimator::new();
        assert_eq!(estimator.default_model, "gpt-3.5-turbo");
        assert!(!estimator.pricing.is_empty());
    }

    #[test]
    fn test_token_estimation() {
        let mut estimator = TokenEstimator::new();
        let estimate = estimator.estimate_tokens("Hello world", Some("gpt-3.5-turbo")).unwrap();

        assert_eq!(estimate.model, "gpt-3.5-turbo");
        assert_eq!(estimate.characters, 11);
        assert!(estimate.tokens > 0);
        assert!(estimate.estimated_cost.is_some());
    }

    #[test]
    fn test_token_limit_status() {
        let estimator = TokenEstimator::new();

        assert_eq!(estimator.check_token_limits(1000, "gpt-3.5-turbo"), TokenLimitStatus::Normal);
        assert_eq!(estimator.check_token_limits(3500, "gpt-3.5-turbo"), TokenLimitStatus::Warning);
        assert_eq!(estimator.check_token_limits(3800, "gpt-3.5-turbo"), TokenLimitStatus::Critical);
    }

    #[test]
    fn test_usage_tracker() {
        let pricing = ModelPricing {
            input_per_1k: 0.0015,
            output_per_1k: 0.002,
            max_tokens: 4096,
        };

        let mut tracker = TokenUsageTracker {
            model: "gpt-3.5-turbo".to_string(),
            total_tokens: 0,
            prompt_tokens: 0,
            completion_tokens: 0,
            estimated_cost: 0.0,
            token_limit: pricing.max_tokens,
            pricing,
        };

        tracker.record_prompt(100);
        tracker.record_completion(50);

        assert_eq!(tracker.total_tokens, 150);
        assert_eq!(tracker.prompt_tokens, 100);
        assert_eq!(tracker.completion_tokens, 50);
        assert!(tracker.estimated_cost > 0.0);
    }

    #[test]
    fn test_limit_status_colors() {
        assert_eq!(TokenLimitStatus::Normal.color(), "green");
        assert_eq!(TokenLimitStatus::Warning.color(), "yellow");
        assert_eq!(TokenLimitStatus::Critical.color(), "red");
        assert_eq!(TokenLimitStatus::Unknown.color(), "gray");
    }

    #[test]
    fn test_limit_status_symbols() {
        assert_eq!(TokenLimitStatus::Normal.symbol(), "✓");
        assert_eq!(TokenLimitStatus::Warning.symbol(), "⚠");
        assert_eq!(TokenLimitStatus::Critical.symbol(), "🚨");
        assert_eq!(TokenLimitStatus::Unknown.symbol(), "?");
    }
}