//! Ricecoder AI Providers - Unified abstraction layer for multiple AI providers
//!
//! This crate provides a consistent interface for interacting with different AI providers
//! (OpenAI, Anthropic, ollama, Google, etc.) without changing your workflow.

pub mod api_key;
pub mod cache;
pub mod config;
pub mod error;
pub mod health_check;
pub mod models;
pub mod provider;
pub mod providers;
pub mod redaction;
pub mod token_counter;

// Re-export commonly used types
pub use api_key::ApiKeyManager;
pub use cache::ProviderCache;
pub use error::ProviderError;
pub use health_check::{HealthCheckCache, HealthCheckResult};
pub use models::{
    Capability, ChatRequest, ChatResponse, FinishReason, Message, ModelInfo, TokenUsage,
};
pub use provider::{Provider, ProviderManager, ProviderRegistry};
pub use providers::{
    AnthropicProvider, GoogleProvider, OllamaProvider, OpenAiProvider, ZenProvider,
};
pub use redaction::{contains_sensitive_info, redact, Redacted, RedactionFilter};
pub use token_counter::{TokenCounter, TokenCounterTrait};
