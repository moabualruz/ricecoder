//! Ricecoder AI Providers - Unified abstraction layer for multiple AI providers
//!
//! This crate provides a consistent interface for interacting with different AI providers
//! (OpenAI, Anthropic, ollama, Google, etc.) without changing your workflow.

pub mod api_key;
pub mod audit_log;
pub mod cache;
pub mod community;
pub mod config;
pub mod curation;
pub mod error;
pub mod evaluation;
pub mod health_check;
pub mod integration;
pub mod models;
pub mod performance_monitor;
pub mod provider;
pub mod providers;
pub mod rate_limiter;
pub mod redaction;
pub mod security_headers;
pub mod streaming;
pub mod sync;
pub mod token_counter;

// Re-export commonly used types
pub use api_key::ApiKeyManager;
pub use audit_log::{AuditEventType, AuditLogger, AuditLogEntry};
pub use cache::ProviderCache;
pub use error::ProviderError;
pub use health_check::{HealthCheckCache, HealthCheckResult};
pub use integration::ProviderIntegration;
pub use models::{
    Capability, ChatRequest, ChatResponse, FinishReason, Message, ModelInfo, TokenUsage,
};
pub use performance_monitor::{
    PerformanceSummary, PerformanceThresholds, ProviderMetrics, ProviderPerformanceMonitor,
};
pub use community::{
    CommunityProviderConfig, CommunityProviderRegistry, ContributionMetadata,
    ContributionReview, ContributionStatus, ProviderAnalytics, ProviderUpdate,
    ProviderUsage, UpdateType,
};
pub use curation::{
    CurationConfig, ProviderCurator, QualityScore, ReliabilityStatus, SelectionConstraints,
};
pub use evaluation::{
    BenchmarkResult, ContinuousEvaluator, PerformanceMetrics, ProviderEvaluation, ProviderEvaluator,
};
pub use sync::{
    CommunityDatabaseConfig, CommunityDatabaseSync, ContributionValidator, SyncStatus, ValidationRules,
};
pub use provider::manager::{ConnectionState, ModelFilter, ModelFilterCriteria, ProviderStatus};
pub use provider::{Provider, ProviderManager, ProviderRegistry};
pub use providers::{
    AnthropicProvider, AzureOpenAiProvider, CohereProvider, GcpVertexProvider, GoogleProvider, OllamaProvider, OpenAiProvider, ReplicateProvider, TogetherProvider, ZenProvider,
};
pub use rate_limiter::{ExponentialBackoff, RateLimiterRegistry, TokenBucketLimiter};
pub use redaction::{contains_sensitive_info, redact, Redacted, RedactionFilter};
pub use security_headers::{SecurityHeadersBuilder, SecurityHeadersValidator};
pub use streaming::{simulate_stream, simulate_word_stream, SimulatedStream, WordStream};
pub use token_counter::{TokenCounter, TokenCounterTrait};
