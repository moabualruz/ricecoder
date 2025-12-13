# ricecoder-local-models

**Purpose**: Local model management providing Ollama integration and offline AI capabilities for RiceCoder

## Features

- **Ollama Integration**: Seamless integration with Ollama for running local AI models
- **Model Management**: Download, update, and manage multiple local models
- **Offline Capability**: Full functionality without internet connectivity for local models
- **Performance Optimization**: Efficient model loading and memory management
- **Fallback Support**: Automatic fallback to remote models when local models unavailable

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-local-models = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_local_models::{OllamaManager, LocalModel};

// Initialize Ollama manager
let manager = OllamaManager::new("http://localhost:11434").await?;

// List available models
let models = manager.list_models().await?;
for model in models {
    println!("Available model: {}", model.name);
}

// Run inference with a local model
let model = LocalModel::new("llama2");
let response = manager.generate(&model, "Explain Rust ownership").await?;
println!("Response: {}", response.text);
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-local-models).

## License

MIT
