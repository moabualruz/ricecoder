//! Token estimation and tracking for AI conversations
//!
//! This module provides accurate token counting using tiktoken and cost estimation
//! for various AI models. It integrates with the session system to track token usage
//! and provide warnings when approaching limits.

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
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
#[derive(Debug)]
pub struct TokenEstimator {
    /// Cached tokenizers by model
    tokenizers: HashMap<String, Arc<CoreBPE>>,
    /// Model pricing information
    pricing: HashMap<String, ModelPricing>,
    /// Default model for estimation
    default_model: String,
}

impl TokenEstimator {
    /// Get the default model
    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    /// Get the pricing information
    pub fn pricing(&self) -> &HashMap<String, ModelPricing> {
        &self.pricing
    }
}

/// Pricing information for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per 1M input tokens (USD) - matches OpenCode pricing model
    pub input_per_1m: f64,
    /// Cost per 1M output tokens (USD)
    pub output_per_1m: f64,
    /// Cost per 1M cache read tokens (USD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_per_1m: Option<f64>,
    /// Cost per 1M cache write tokens (USD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_per_1m: Option<f64>,
    /// Maximum context window (tokens)
    pub max_tokens: usize,
    /// Maximum output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<usize>,
    /// Experimental pricing for contexts over 200K tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_over_200k: Option<Over200KPricing>,
}

/// Pricing for contexts over 200K tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Over200KPricing {
    pub input_per_1m: f64,
    pub output_per_1m: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_per_1m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_per_1m: Option<f64>,
}

impl Default for ModelPricing {
    fn default() -> Self {
        Self {
            input_per_1m: 1500.0, // GPT-3.5 default (1.5K per 1M tokens)
            output_per_1m: 2000.0,
            cache_read_per_1m: None,
            cache_write_per_1m: None,
            max_tokens: 4096,
            max_output_tokens: Some(4096),
            experimental_over_200k: None,
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

    /// Initialize default pricing for common models (per 1M tokens - OpenCode compatible)
    fn initialize_pricing(&mut self) {
        // GPT-4 pricing
        self.pricing.insert(
            "gpt-4".to_string(),
            ModelPricing {
                input_per_1m: 30_000.0,
                output_per_1m: 60_000.0,
                cache_read_per_1m: None,
                cache_write_per_1m: None,
                max_tokens: 8192,
                max_output_tokens: Some(8192),
                experimental_over_200k: None,
            },
        );

        self.pricing.insert(
            "gpt-4-32k".to_string(),
            ModelPricing {
                input_per_1m: 60_000.0,
                output_per_1m: 120_000.0,
                cache_read_per_1m: None,
                cache_write_per_1m: None,
                max_tokens: 32768,
                max_output_tokens: Some(32768),
                experimental_over_200k: None,
            },
        );

        // GPT-3.5 pricing
        self.pricing.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                input_per_1m: 1500.0,
                output_per_1m: 2000.0,
                cache_read_per_1m: None,
                cache_write_per_1m: None,
                max_tokens: 4096,
                max_output_tokens: Some(4096),
                experimental_over_200k: None,
            },
        );

        self.pricing.insert(
            "gpt-3.5-turbo-16k".to_string(),
            ModelPricing {
                input_per_1m: 3000.0,
                output_per_1m: 4000.0,
                cache_read_per_1m: None,
                cache_write_per_1m: None,
                max_tokens: 16384,
                max_output_tokens: Some(16384),
                experimental_over_200k: None,
            },
        );

        // Claude pricing (with cache support)
        self.pricing.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                input_per_1m: 15_000.0,
                output_per_1m: 75_000.0,
                cache_read_per_1m: Some(1_500.0),
                cache_write_per_1m: Some(18_750.0),
                max_tokens: 200_000,
                max_output_tokens: Some(4_096),
                experimental_over_200k: None,
            },
        );

        self.pricing.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                input_per_1m: 3_000.0,
                output_per_1m: 15_000.0,
                cache_read_per_1m: Some(300.0),
                cache_write_per_1m: Some(3_750.0),
                max_tokens: 200_000,
                max_output_tokens: Some(4_096),
                experimental_over_200k: None,
            },
        );

        self.pricing.insert(
            "claude-3-haiku".to_string(),
            ModelPricing {
                input_per_1m: 250.0,
                output_per_1m: 1_250.0,
                cache_read_per_1m: Some(25.0),
                cache_write_per_1m: Some(312.5),
                max_tokens: 200_000,
                max_output_tokens: Some(4_096),
                experimental_over_200k: None,
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
    pub fn estimate_tokens(
        &mut self,
        text: &str,
        model: Option<&str>,
    ) -> SessionResult<TokenEstimate> {
        let model_name = model
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_model.clone());
        let tokenizer = self.get_tokenizer(&model_name)?;

        let tokens = tokenizer.encode_with_special_tokens(text).len();
        let characters = text.chars().count();

        let estimated_cost = self
            .pricing
            .get(&model_name)
            .map(|pricing| (tokens as f64 / 1_000_000.0) * pricing.input_per_1m);

        Ok(TokenEstimate {
            tokens,
            model: model_name,
            characters,
            estimated_cost,
        })
    }

    /// Estimate tokens for a conversation (multiple messages)
    pub fn estimate_conversation_tokens(
        &mut self,
        messages: &[crate::models::Message],
        model: Option<&str>,
    ) -> SessionResult<TokenEstimate> {
        let combined_text = messages
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content()))
            .collect::<Vec<_>>()
            .join("\n");

        self.estimate_tokens(&combined_text, model)
    }

    /// Create token usage tracker for a session
    pub fn create_usage_tracker(&self, model: &str) -> SessionResult<TokenUsageTracker> {
        let pricing = self.pricing.get(model).cloned().unwrap_or_default();

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
                get_bpe_from_model(model).map_err(|e| {
                    SessionError::TokenEstimation(format!(
                        "Failed to load tokenizer for {}: {}",
                        model, e
                    ))
                })?
            } else {
                // Fallback to GPT-3.5 tokenizer for other models
                get_bpe_from_model("gpt-3.5-turbo").map_err(|e| {
                    SessionError::TokenEstimation(format!(
                        "Failed to load fallback tokenizer: {}",
                        e
                    ))
                })?
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
        self.estimated_cost += (tokens as f64 / 1_000_000.0) * self.pricing.input_per_1m;
    }

    /// Record completion tokens
    pub fn record_completion(&mut self, tokens: usize) {
        self.completion_tokens += tokens;
        self.total_tokens += tokens;
        self.estimated_cost += (tokens as f64 / 1_000_000.0) * self.pricing.output_per_1m;
    }

    /// Record cache read tokens (OpenCode parity)
    pub fn record_cache_read(&mut self, tokens: usize) {
        self.prompt_tokens += tokens;
        self.total_tokens += tokens;
        if let Some(cache_read_cost) = self.pricing.cache_read_per_1m {
            self.estimated_cost += (tokens as f64 / 1_000_000.0) * cache_read_cost;
        }
    }

    /// Record cache write tokens (OpenCode parity)
    pub fn record_cache_write(&mut self, tokens: usize) {
        // Cache writes count as input tokens but cost more
        self.prompt_tokens += tokens;
        self.total_tokens += tokens;
        if let Some(cache_write_cost) = self.pricing.cache_write_per_1m {
            self.estimated_cost += (tokens as f64 / 1_000_000.0) * cache_write_cost;
        }
    }

    /// Record reasoning tokens (OpenCode parity - charged at output rate)
    pub fn record_reasoning(&mut self, tokens: usize) {
        self.completion_tokens += tokens;
        self.total_tokens += tokens;
        // Reasoning tokens charged at output rate per OpenCode
        self.estimated_cost += (tokens as f64 / 1_000_000.0) * self.pricing.output_per_1m;
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

/// Global output token maximum (OpenCode parity)
pub const OUTPUT_TOKEN_MAX: usize = 32_000;

/// Prune minimum threshold (OpenCode parity)
pub const PRUNE_MINIMUM: usize = 20_000;

/// Prune protection threshold (OpenCode parity)
pub const PRUNE_PROTECT: usize = 40_000;

/// Check if token usage indicates overflow (OpenCode isOverflow parity)
pub fn is_overflow(
    input_tokens: usize,
    cache_read_tokens: usize,
    output_tokens: usize,
    context_limit: usize,
    model_output_limit: Option<usize>,
) -> bool {
    if context_limit == 0 {
        return false;
    }
    
    // Count excludes reasoning and cache write per OpenCode
    let count = input_tokens + cache_read_tokens + output_tokens;
    
    // Calculate reserved output budget
    let output_budget = model_output_limit
        .map(|limit| limit.min(OUTPUT_TOKEN_MAX))
        .unwrap_or(OUTPUT_TOKEN_MAX);
    
    let usable = context_limit.saturating_sub(output_budget);
    
    count > usable
}

/// Calculate output token budget (OpenCode ProviderTransform.maxOutputTokens parity)
pub fn max_output_tokens(model_limit: Option<usize>) -> usize {
    model_limit
        .map(|limit| limit.min(OUTPUT_TOKEN_MAX))
        .unwrap_or(OUTPUT_TOKEN_MAX)
}
