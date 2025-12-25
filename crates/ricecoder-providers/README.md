# ricecoder-providers

**Purpose**: Unified AI provider integration providing consistent access to Anthropic, OpenAI, Google, Ollama, and other AI services for RiceCoder

## DDD Layer

**Infrastructure** - This crate implements external AI provider integrations, following the Dependency Inversion Principle by implementing domain-defined port traits (`AiProviderChat`, `AiProviderInfo`) from `ricecoder-domain`.

## Overview

`ricecoder-providers` provides comprehensive AI provider integration that was extracted from the TUI during the architectural refactoring. This crate handles all AI provider communication, authentication, and management independently of the user interface.

## Features

- **75+ Provider Support**: Anthropic, OpenAI, Google, Azure, GCP Vertex, Cohere, Together AI, Replicate, and more
- **Quality Scoring**: Automated provider evaluation and benchmarking
- **Community Ecosystem**: Community-managed provider database with validation
- **Automated Evaluation**: Continuous performance monitoring and scoring
- **Enterprise Integration**: Azure OpenAI and GCP Vertex AI support
- **Unified Interface**: Consistent API across all providers
- **Authentication Management**: API key and OAuth handling
- **Rate Limiting**: Intelligent request throttling and retry logic
- **Streaming Support**: Real-time response streaming
- **Token Management**: Accurate token counting and limits
- **Health Monitoring**: Provider availability and performance tracking
- **Caching**: Response caching and optimization
- **Cost Optimization**: Automatic provider switching based on cost and performance

## Architecture

After the TUI isolation refactoring, provider integration was moved from `ricecoder-tui` to this dedicated crate:

### ✅ Responsibilities:
- AI provider communication
- Authentication and authorization
- Request/response handling
- Rate limiting and retries
- Token counting and limits
- Provider health monitoring
- Streaming response management

### Dependencies
- **HTTP Client**: `reqwest` for API communication
- **Async Runtime**: `tokio` for concurrent operations
- **Serialization**: `serde` for request/response handling
- **Caching**: Custom caching with TTL support
- **Storage**: `ricecoder-storage` for audit logs and configuration

### Integration Points
- **Storage**: Uses `ricecoder-storage` for configuration and caching
- **Sessions**: Provides AI responses to session management
- **TUI**: Displays provider status (but doesn't depend on TUI)
- **All Crates**: Powers AI capabilities throughout RiceCoder

## Supported Providers

| Provider | Authentication | Models | Streaming | Enterprise |
|----------|----------------|--------|-----------|------------|
| Anthropic | API Key | Claude 3.x | ✅ | ❌ |
| OpenAI | API Key | GPT-4, GPT-3.5 | ✅ | ❌ |
| Google | API Key/OAuth | Gemini | ✅ | ❌ |
| Azure OpenAI | API Key | GPT-4, GPT-3.5 | ✅ | ✅ |
| GCP Vertex AI | Access Token | Gemini, PaLM | ✅ | ✅ |
| Cohere | API Key | Command, Base | ❌ | ❌ |
| Together AI | API Key | Llama, Mistral, CodeLlama | ❌ | ❌ |
| Replicate | API Key | Various open-source | ❌ | ❌ |
| Ollama | None | Local models | ✅ | ❌ |
| And 65+ more | Various | Various | Varies | Varies |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-providers = "0.1"
```

## Usage

```rust
use ricecoder_providers::{ProviderManager, ChatRequest};

let manager = ProviderManager::new().await?;

// Configure providers
manager.add_provider("anthropic", anthropic_config).await?;

// Make requests
let request = ChatRequest {
    model: "claude-3-sonnet-20240229".to_string(),
    messages: vec![/* messages */],
    temperature: Some(0.7),
    max_tokens: Some(1000),
};

let response = manager.chat("anthropic", request).await?;
```

## Key Components

- **`ProviderManager`**: Main entry point for provider operations
- **`Provider`**: Trait defining provider interface
- **`ChatRequest/ChatResponse`**: Request/response data structures
- **`RateLimiter`**: Request throttling and retry logic
- **`HealthChecker`**: Provider availability monitoring

## Configuration

Providers are configured via `ricecoder-storage`:

```yaml
providers:
  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
    default_model: "claude-3-sonnet-20240229"
  openai:
    api_key: "${OPENAI_API_KEY}"
    default_model: "gpt-4"
```

## Error Handling

The crate provides comprehensive error handling:

```rust
use ricecoder_providers::ProviderError;

match result {
    Ok(response) => println!("Response: {}", response.content),
    Err(ProviderError::RateLimited(duration)) => {
        println!("Rate limited, retry in {:?}", duration);
    }
    Err(ProviderError::AuthError) => {
        println!("Authentication failed");
    }
    // ... other error types
}
```

## Integration

`ricecoder-providers` integrates with other RiceCoder components:

- **CLI Integration**: Main application manages provider lifecycle
- **Session Integration**: Provides AI responses for conversations
- **Storage Integration**: Persists configuration and caches responses
- **TUI Integration**: Provides status information for UI display

## Testing

Per R2 (Test Suite Reconstruction), tests are organized in the `tests/` directory:

```bash
# Run all tests (unit + integration)
cargo test -p ricecoder-providers

# Run integration tests only
cargo test -p ricecoder-providers --test integration_test

# Run specific test modules
cargo test -p ricecoder-providers circuit_breaker
cargo test -p ricecoder-providers domain_adapter
cargo test -p ricecoder-providers curation
```

### Test Organization

| Location | Type | Coverage |
|----------|------|----------|
| `src/**/tests` | Unit tests | Module-level functionality |
| `tests/integration_test.rs` | Integration tests | Cross-module interactions |

### Key Test Areas

- **Circuit Breaker**: State transitions, failure thresholds, recovery
- **Domain Adapter**: Type conversion, error mapping, retry logic
- **Provider Registry**: Registration, lookup, lifecycle
- **Performance Monitor**: Metrics calculation, threshold evaluation
- **Community**: Contribution workflow, analytics
- **Curation**: Quality scoring, provider selection

## Performance

- **Chat Requests**: < 500ms for cached responses, 2-30s for API calls
- **Streaming**: < 100ms initial response, real-time chunk delivery
- **Caching**: < 10ms cache lookups with 90%+ hit rates
- **Health Checks**: < 200ms per provider status check
- **Token Counting**: < 5ms for typical message lengths

## Recent Changes

### SRP Refactoring (December 2024)

**HTTP Client Injection**: Removed direct `reqwest::Client` construction, now injected via DI container following Dependency Inversion Principle.

**Changes**:
- HTTP client now created by `ricecoder-di` container and injected into providers
- Enables centralized HTTP configuration (timeouts, retries, TLS)
- Simplifies testing with mock HTTP clients
- Consistent HTTP behavior across all providers

**Migration**: Provider constructors now accept `Arc<reqwest::Client>` parameter. Legacy constructors with default clients available for backward compatibility.

```rust
// New pattern (DI-injected)
let client = container.resolve::<Arc<reqwest::Client>>()?;
let provider = OpenAIProvider::with_client(config, client);

// Legacy pattern (still supported)
let provider = OpenAIProvider::new(config)?;
```

## Contributing

When working with `ricecoder-providers`:

1. **Keep provider logic here**: AI integration belongs in this crate
2. **Use interfaces**: Don't depend on UI components
3. **Test thoroughly**: Provider failures affect user experience
4. **Document providers**: Keep provider documentation up-to-date

## Gap Analysis (vs Industry Patterns)

Comparison with industry best practices from major Rust projects:

| Pattern | Industry Standard | ricecoder-providers Status |
|---------|-------------------|---------------------------|
| **Provider Trait** | `async_trait` + `Send + Sync` | ✅ Implemented |
| **Dynamic Dispatch** | `Arc<dyn Provider>` | ✅ Implemented |
| **Circuit Breaker** | Three-state (Closed/Open/HalfOpen) | ✅ Implemented |
| **Rate Limiting** | Token bucket with `governor` | ✅ Custom implementation |
| **Health Checks** | Async with configurable intervals | ✅ Implemented with cache |
| **Retry Logic** | Exponential backoff with jitter | ✅ Implemented |
| **Tower Integration** | Service/Layer middleware | ⚠️ Not yet (potential enhancement) |
| **Provider Registry** | Factory pattern with lazy init | ✅ Implemented |

### Potential Enhancements (Beta)

1. **Tower Layer Integration**: Add `tower::Service` implementation for middleware composability
2. **Governor Rate Limiter**: Consider replacing custom implementation with `governor` crate
3. **Observability**: Add OpenTelemetry tracing spans for provider calls

## License

MIT
