//! AI Provider port interfaces and value objects
//!
//! AI Provider Implementations
//!
//! This module contains the contracts for AI provider integrations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::*;

// ============================================================================
// AI Provider Value Objects
// ============================================================================

/// Role of a message sender in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// User/human message
    User,
    /// AI assistant message
    Assistant,
    /// System instructions
    System,
}

/// A message in a chat conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: ChatRole,
    /// Content of the message
    pub content: String,
}

impl ChatMessage {
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }
}

/// Request to an AI provider for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatRequest {
    /// Model identifier (e.g., "gpt-4", "claude-3-opus")
    pub model: String,
    /// Conversation messages
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0-2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

impl AiChatRequest {
    /// Create a new chat request
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            stop: None,
        }
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set stop sequences
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
}

/// Token usage statistics from an AI response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Tokens used in the prompt
    pub prompt_tokens: u32,
    /// Tokens used in the completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Reason why the AI stopped generating
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop (end of response)
    Stop,
    /// Hit max token limit
    Length,
    /// Content filtered
    ContentFilter,
    /// Tool/function call
    ToolCalls,
    /// Error occurred
    Error,
}

/// Response from an AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatResponse {
    /// Generated content
    pub content: String,
    /// Model that generated the response
    pub model: String,
    /// Token usage statistics
    pub usage: TokenUsage,
    /// Why the model stopped generating
    pub finish_reason: FinishReason,
}

/// Model capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    /// Basic chat completion
    Chat,
    /// Code generation and analysis
    Code,
    /// Image/vision understanding
    Vision,
    /// Streaming responses
    Streaming,
    /// Function/tool calling
    FunctionCalling,
    /// Embeddings generation
    Embeddings,
}

/// Information about an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Provider identifier
    pub provider: String,
    /// Context window size (tokens)
    pub context_window: u32,
    /// Available capabilities
    pub capabilities: Vec<ModelCapability>,
    /// Whether the model is free to use
    pub is_free: bool,
}

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderHealthStatus {
    /// Provider is healthy and responding
    Healthy,
    /// Provider is degraded (slow but working)
    Degraded,
    /// Provider is unhealthy (errors or timeout)
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall health status
    pub status: ProviderHealthStatus,
    /// Response latency in milliseconds
    pub latency_ms: Option<u64>,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Timestamp of the check
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(latency_ms: u64) -> Self {
        Self {
            status: ProviderHealthStatus::Healthy,
            latency_ms: Some(latency_ms),
            error: None,
            checked_at: chrono::Utc::now(),
        }
    }

    /// Create a degraded result
    pub fn degraded(latency_ms: u64) -> Self {
        Self {
            status: ProviderHealthStatus::Degraded,
            latency_ms: Some(latency_ms),
            error: None,
            checked_at: chrono::Utc::now(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(error: impl Into<String>) -> Self {
        Self {
            status: ProviderHealthStatus::Unhealthy,
            latency_ms: None,
            error: Some(error.into()),
            checked_at: chrono::Utc::now(),
        }
    }
}

// ============================================================================
// AI Provider Ports (ISP-Compliant Split)
// ============================================================================

/// Provider information interface (ISP: 4 methods)
///
///  AI Provider Implementations
///
/// Read-only provider metadata. Clients that only need to inspect
/// provider details should depend on this trait.
pub trait AiProviderInfo: Send + Sync {
    /// Get the provider's unique identifier (e.g., "openai", "anthropic")
    fn id(&self) -> &str;

    /// Get the provider's human-readable name (e.g., "OpenAI", "Anthropic")
    fn name(&self) -> &str;

    /// Get the list of available models
    fn models(&self) -> Vec<ModelInfo>;

    /// Get the default model ID for this provider
    fn default_model(&self) -> Option<String> {
        self.models().first().map(|m| m.id.clone())
    }
}

/// Provider chat capabilities (ISP: 4 methods)
///
///  AI Provider Implementations
///
/// Chat and token operations. Clients that need chat functionality
/// should depend on this trait.
#[async_trait]
pub trait AiProviderChat: Send + Sync {
    /// Send a chat completion request
    ///
    /// # Errors
    ///
    /// Returns `DomainError::ExternalServiceError` on API failures,
    /// `DomainError::ValidationError` on invalid requests.
    async fn chat(&self, request: AiChatRequest) -> DomainResult<AiChatResponse>;

    /// Count tokens for content
    ///
    /// Used for request sizing and cost estimation.
    fn count_tokens(&self, content: &str, model: &str) -> DomainResult<usize>;

    /// Check if the provider is available and healthy
    async fn health_check(&self) -> DomainResult<HealthCheckResult>;

    /// Check if the provider supports a specific capability
    fn supports_capability(&self, capability: ModelCapability) -> bool;
}

/// Combined AI provider interface (Info + Chat)
///
///  AI Provider Implementations
///
/// This trait combines provider information and chat capabilities.
/// Implementations are provided by the infrastructure layer (ricecoder-providers).
///
/// # Design Notes
///
/// - Stateless: Providers should not maintain mutable state between calls
/// - Async: All I/O operations are async for non-blocking execution
/// - Error Mapping: Provider implementations map their errors to DomainError
///
/// # ISP Compliance
///
/// Clients with more focused needs should depend on role-specific traits:
/// - Info only: `AiProviderInfo`
/// - Chat only: `AiProviderChat`
///
/// # Example
///
/// ```ignore
/// let provider: &dyn AiProvider = &openai_provider;
/// let request = AiChatRequest::new("gpt-4", vec![ChatMessage::user("Hello!")]);
/// let response = provider.chat(request).await?;
/// ```
pub trait AiProvider: AiProviderInfo + AiProviderChat {}

/// Blanket implementation: Any type implementing Info + Chat gets AiProvider
impl<T: AiProviderInfo + AiProviderChat> AiProvider for T {}

/// Port interface for streaming AI responses
///
/// Optional extension for providers that support streaming.
#[async_trait]
pub trait StreamingAiProvider: AiProvider {
    /// Stream type for chat responses
    type Stream: futures_core::Stream<Item = DomainResult<String>> + Send;

    /// Send a chat completion request with streaming response
    async fn chat_stream(&self, request: AiChatRequest) -> DomainResult<Self::Stream>;
}

// ============================================================================
// Provider Registry Port
// ============================================================================

/// Port interface for managing multiple AI providers
#[async_trait]
pub trait AiProviderRegistry: Send + Sync {
    /// Register a provider
    async fn register(&self, provider: Box<dyn AiProvider>) -> DomainResult<()>;

    /// Get a provider by ID
    async fn get(&self, id: &str) -> DomainResult<Option<&dyn AiProvider>>;

    /// List all registered provider IDs
    fn list_provider_ids(&self) -> Vec<String>;

    /// Get the default provider
    async fn default_provider(&self) -> DomainResult<Option<&dyn AiProvider>>;

    /// Set the default provider by ID
    async fn set_default(&self, id: &str) -> DomainResult<()>;
}
