# ricecoder-local-models

## Purpose

Local model management crate providing Ollama integration and offline AI capabilities for RiceCoder. Handles model pulling, removal, updates, and version management through the Ollama API.

## DDD Layer

**Infrastructure** - Implements external service integration (Ollama API).

## Responsibilities

- Connect to and communicate with Ollama server
- Pull models from Ollama registry
- Remove models from local storage
- Update models to latest versions
- Query model information and metadata
- List all available local models
- Check model availability
- Health check for Ollama server connectivity

## Dependencies

### Internal (RiceCoder Crates)

- `ricecoder-providers`: Provider trait definitions and abstractions
- `ricecoder-storage`: Configuration and storage utilities

### External Libraries

- `reqwest`: HTTP client for Ollama API communication
- `serde` / `serde_json`: Serialization for API requests/responses
- `thiserror`: Error type definitions
- `tracing`: Structured logging
- `chrono`: DateTime handling for model metadata
- `tokio`: Async runtime

## Key Types

- `LocalModelManager`: Main entry point for all Ollama operations
- `LocalModel`: Model information including name, size, digest, and metadata
- `ModelMetadata`: Format, family, parameter size, and quantization level
- `PullProgress`: Progress tracking for model download operations
- `LocalModelError`: Comprehensive error types for all operations

## Usage

```rust
use ricecoder_local_models::{LocalModelManager, LocalModel};

// Initialize manager with default localhost endpoint
let manager = LocalModelManager::with_default_endpoint()?;

// Or with custom endpoint
let manager = LocalModelManager::new("http://localhost:11434".to_string())?;

// Check if Ollama server is reachable
if manager.health_check().await? {
    println!("Ollama server is running");
}

// List available models
let models = manager.list_models().await?;
for model in models {
    println!("Model: {} ({})", model.name, model.metadata.parameter_size);
}

// Pull a model
let progress = manager.pull_model("mistral:latest").await?;
for update in progress {
    println!("{}% - {}", update.percentage(), update.status);
}

// Check if model exists
if manager.model_exists("mistral:latest").await? {
    println!("Model is available");
}

// Get model info
let model = manager.get_model_info("mistral:latest").await?;
println!("Family: {}, Size: {}", model.metadata.family, model.metadata.parameter_size);

// Remove a model
manager.remove_model("mistral:latest").await?;
```

## Error Handling

All operations return `Result<T, LocalModelError>`. Error variants include:

- `ModelNotFound`: Model does not exist locally
- `PullFailed`: Model download failed
- `RemovalFailed`: Model deletion failed
- `InvalidModelName`: Empty or malformed model name
- `NetworkError`: Connection issues with Ollama server
- `ConfigError`: Invalid configuration (e.g., empty base URL)
- `Timeout`: Operation timed out

## License

MIT
