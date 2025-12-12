# ricecoder-providers

AI Provider Integration for RiceCoder - Business Logic Layer

## Overview

`ricecoder-providers` provides comprehensive AI provider integration that was extracted from the TUI during the architectural refactoring. This crate handles all AI provider communication, authentication, and management independently of the user interface.

## Features

- **Multi-Provider Support**: Anthropic, OpenAI, Google, Azure, Ollama, and more
- **Unified Interface**: Consistent API across all providers
- **Authentication Management**: API key and OAuth handling
- **Rate Limiting**: Intelligent request throttling and retry logic
- **Streaming Support**: Real-time response streaming
- **Token Management**: Accurate token counting and limits
- **Health Monitoring**: Provider availability and performance tracking
- **Caching**: Response caching and optimization

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

### 🔗 Integration Points:
- **Storage**: Uses `ricecoder-storage` for configuration and caching
- **Sessions**: Provides AI responses to session management
- **TUI**: Displays provider status (but doesn't depend on TUI)

## Supported Providers

| Provider | Authentication | Models | Streaming |
|----------|----------------|--------|-----------|
| Anthropic | API Key | Claude 3.x | ✅ |
| OpenAI | API Key | GPT-4, GPT-3.5 | ✅ |
| Google | API Key/OAuth | Gemini | ✅ |
| Azure OpenAI | API Key | GPT-4, GPT-3.5 | ✅ |
| Ollama | None | Local models | ✅ |
| OpenRouter | API Key | Multiple providers | ✅ |

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

## Contributing

When working with `ricecoder-providers`:

1. **Keep provider logic here**: AI integration belongs in this crate
2. **Use interfaces**: Don't depend on UI components
3. **Test thoroughly**: Provider failures affect user experience
4. **Document providers**: Keep provider documentation up-to-date

## License

MIT
