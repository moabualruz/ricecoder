//! Provider trait and registry

use async_trait::async_trait;

use crate::error::ProviderError;
use crate::models::{ChatRequest, ChatResponse, ModelInfo};

pub mod manager;
pub mod registry;

pub use manager::ProviderManager;
pub use registry::ProviderRegistry;

/// A stream of chat completion responses
pub type ChatStream =
    Box<dyn futures::Stream<Item = Result<ChatResponse, ProviderError>> + Send + Unpin>;

/// Core trait that all providers must implement
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider's unique identifier
    fn id(&self) -> &str;

    /// Get the provider's human-readable name
    fn name(&self) -> &str;

    /// Get the list of available models
    fn models(&self) -> Vec<ModelInfo>;

    /// Send a chat completion request
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;

    /// Stream a chat completion response
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, ProviderError>;

    /// Count tokens for content
    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError>;

    /// Check if the provider is available and healthy
    async fn health_check(&self) -> Result<bool, ProviderError>;
}
