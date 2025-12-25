# ricecoder-completion

**Purpose**: Language-agnostic code completion engine with external LSP integration and ghost text support for RiceCoder

## DDD Layer

**Infrastructure** - Code completion infrastructure integrating external LSP servers with internal providers.

## Overview

`ricecoder-completion` provides a comprehensive code completion system that integrates with external Language Server Protocol (LSP) servers while maintaining graceful fallback to internal providers. It supports multiple programming languages with semantic completions, context-aware suggestions, and inline ghost text display.

## Features

- **External LSP Integration**: Primary completions from language servers (rust-analyzer, tsserver, pylsp, etc.)
- **Multi-Language Support**: Rust, TypeScript, Python, Go, Java, Kotlin, Dart, and generic text completion
- **Ghost Text**: Inline completion suggestions with partial acceptance
- **Context Analysis**: Code-aware completion based on scope and symbols
- **Intelligent Ranking**: Relevance-based sorting with frequency and recency scoring
- **Fallback Providers**: Graceful degradation when external LSP unavailable
- **Configuration-Driven**: Language-specific completion rules and snippets

## Architecture

### Responsibilities
- Completion request orchestration and routing
- External LSP server communication and response transformation
- Context analysis for semantic completion
- Completion ranking and filtering
- Ghost text generation and state management
- Language-specific provider management

### Dependencies
- **LSP Integration**: `lsp-types` for protocol communication
- **Async Runtime**: `tokio` for concurrent operations
- **Parsing**: `tree-sitter` for syntax analysis
- **Storage**: `ricecoder-storage` for configuration and caching

### Integration Points
- **LSP**: Consumes semantic completion data from external language servers
- **TUI**: Provides completion suggestions for terminal interface
- **Storage**: Persists completion history and configuration
- **History**: Tracks completion usage for ranking improvements

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-completion = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_completion::{GenericCompletionEngine, ProviderRegistry, BasicCompletionRanker};
use ricecoder_completion::context::GenericContextAnalyzer;
use ricecoder_completion::engine::BasicCompletionGenerator;
use std::sync::Arc;

// Create completion components
let context_analyzer = Arc::new(GenericContextAnalyzer);
let generator = Arc::new(BasicCompletionGenerator);
let ranker = Arc::new(BasicCompletionRanker::default_weights());
let registry = ProviderRegistry::new();

// Create completion engine
let engine = GenericCompletionEngine::new(
    context_analyzer,
    generator,
    ranker,
    registry,
);

// Generate completions
let completions = engine.generate_completions(
    "fn main() { let x = ",
    Position::new(0, 20),
    "rust",
).await?;
```

### Advanced Usage with External LSP

```rust
use ricecoder_completion::external_lsp_proxy::ExternalLspCompletionProxy;

// Create LSP proxy for external server integration
let lsp_proxy = ExternalLspCompletionProxy::new("rust-analyzer".to_string());

// Generate completions with LSP integration
let completions = engine.generate_completions_with_lsp(
    code,
    position,
    "rust",
    &lsp_proxy,
).await?;
```

### Ghost Text Integration

```rust
use ricecoder_completion::ghost_text::{BasicGhostTextGenerator, GhostTextStateManager};

// Generate ghost text from completion
let generator = BasicGhostTextGenerator;
let ghost_text = generator.generate_ghost_text(&completion, position)?;

// Manage ghost text state
let mut state_manager = BasicGhostTextStateManager::new();
state_manager.show_ghost_text(ghost_text);
```

## Configuration

Completion behavior is configured via YAML:

```yaml
completion:
  # Language-specific settings
  languages:
    rust:
      enabled: true
      external_lsp: "rust-analyzer"
      fallback_provider: "internal"
    typescript:
      enabled: true
      external_lsp: "typescript-language-server"
      snippets:
        - trigger: "cl"
          body: "console.log($1);"

  # Ranking weights
  ranking:
    relevance_weight: 0.4
    frequency_weight: 0.3
    recency_weight: 0.3

  # Ghost text settings
  ghost_text:
    enabled: true
    style: "dimmed"
    partial_acceptance: true
```

## API Reference

### Key Types

- **`GenericCompletionEngine`**: Main completion engine orchestrating all components
- **`CompletionItem`**: Represents a single completion suggestion
- **`Position`**: Code position for completion requests
- **`ExternalLspCompletionProxy`**: Handles external LSP server communication
- **`GhostTextStateManager`**: Manages inline suggestion display

### Key Functions

- **`generate_completions()`**: Generate completion suggestions for code
- **`generate_completions_with_lsp()`**: Generate completions using external LSP
- **`generate_ghost_text()`**: Create inline suggestion text
- **`rank_completions()`**: Sort completions by relevance

## Error Handling

```rust
use ricecoder_completion::CompletionError;

match engine.generate_completions(code, position, language).await {
    Ok(completions) => println!("Found {} completions", completions.len()),
    Err(CompletionError::LspError(msg)) => eprintln!("LSP error: {}", msg),
    Err(CompletionError::ProviderError(msg)) => eprintln!("Provider error: {}", msg),
    Err(CompletionError::ConfigError(msg)) => eprintln!("Configuration error: {}", msg),
}
```

## Testing

Run comprehensive completion tests:

```bash
# Run all tests
cargo test -p ricecoder-completion

# Run property tests for completion correctness
cargo test -p ricecoder-completion property

# Test LSP integration
cargo test -p ricecoder-completion lsp

# Test ghost text functionality
cargo test -p ricecoder-completion ghost_text
```

Key test areas:
- Completion generation accuracy
- LSP proxy communication
- Context analysis correctness
- Ranking algorithm validation
- Ghost text state management

## Performance

- **Completion Generation**: < 100ms for cached results, < 500ms with LSP
- **Context Analysis**: < 50ms for typical code contexts
- **Ranking**: < 10ms for 100-1000 completion items
- **Memory**: Efficient caching with configurable limits
- **Concurrent**: Safe for concurrent completion requests

## Contributing

When working with `ricecoder-completion`:

1. **LSP First**: Prefer external LSP integration over internal providers
2. **Fallback Robustness**: Ensure graceful degradation when LSP unavailable
3. **Language Agnostic**: Keep core engine language-independent
4. **Performance**: Maintain sub-100ms response times for completions
5. **Testing**: Test both LSP and fallback code paths thoroughly

## License

MIT