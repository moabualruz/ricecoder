//! Data models for providers

use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

/// Information about an available model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique model identifier
    pub id: String,
    /// Human-readable model name
    pub name: String,
    /// Provider name
    pub provider: String,
    /// Maximum context window in tokens
    pub context_window: usize,
    /// Model capabilities
    pub capabilities: Vec<Capability>,
    /// Optional pricing information
    pub pricing: Option<Pricing>,
    /// Whether this model is free to use (no API key required)
    #[serde(default)]
    pub is_free: bool,
}

/// Model capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    /// Chat completion capability
    Chat,
    /// Code generation capability
    Code,
    /// Vision/image understanding capability
    Vision,
    /// Function calling capability
    FunctionCalling,
    /// Streaming capability
    Streaming,
}

/// Pricing information for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    /// Cost per 1K input tokens (in USD)
    pub input_per_1k_tokens: f64,
    /// Cost per 1K output tokens (in USD)
    pub output_per_1k_tokens: f64,
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
}

/// Chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model to use
    pub model: String,
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Temperature for sampling (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,
    /// Whether to stream the response
    pub stream: bool,
}

/// Reason for chat completion finish
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    /// Model finished normally
    Stop,
    /// Maximum tokens reached
    Length,
    /// Model encountered an error
    Error,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: usize,
    /// Number of tokens in the completion
    pub completion_tokens: usize,
    /// Total tokens used
    pub total_tokens: usize,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Generated content
    pub content: String,
    /// Model used
    pub model: String,
    /// Token usage
    pub usage: TokenUsage,
    /// Reason for completion
    pub finish_reason: FinishReason,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Default provider and model settings
    pub defaults: DefaultsConfig,
    /// Per-provider settings
    pub providers: HashMap<String, ProviderSettings>,
}

/// Default provider and model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default provider ID
    pub provider: String,
    /// Default model ID
    pub model: String,
    /// Per-command defaults (gen, refactor, review)
    pub per_command: HashMap<String, String>,
    /// Per-action defaults (analysis, generation)
    pub per_action: HashMap<String, String>,
}

/// Settings for a specific provider
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderSettings {
    /// API key (can be overridden by environment variable)
    pub api_key: Option<String>,
    /// Base URL for the provider (for self-hosted or proxy)
    pub base_url: Option<String>,
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Number of retries on failure
    pub retry_count: Option<usize>,
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Environment variable name for the API key
    pub env_var: String,
    /// Whether to use system keyring for secure storage
    pub secure_storage: bool,
}
