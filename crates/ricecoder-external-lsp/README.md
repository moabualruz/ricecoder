# ricecoder-external-lsp

**Purpose**: Integration with external Language Server Protocol servers providing semantic intelligence for code completion, diagnostics, and navigation

## Overview

`ricecoder-external-lsp` provides comprehensive integration with external LSP (Language Server Protocol) servers to deliver production-quality semantic code intelligence. It manages LSP server lifecycle, request routing, response transformation, and graceful fallback to internal providers when external servers are unavailable.

## Features

- **Configuration-Driven**: Support for unlimited LSP servers through YAML configuration
- **Process Management**: Automatic spawning, monitoring, and restart of LSP servers
- **Multi-Language Support**: Pre-configured for Rust, TypeScript, Python, Go, Java, and more
- **Response Mapping**: Transform LSP server responses to RiceCoder internal models
- **Graceful Degradation**: Automatic fallback to internal providers when LSP unavailable
- **Connection Pooling**: Efficient management of multiple LSP server connections
- **Health Monitoring**: LSP server health checks and automatic recovery

## Architecture

### Responsibilities
- External LSP server lifecycle management
- Request routing and response transformation
- Configuration-driven server setup
- Connection pooling and health monitoring
- Fallback coordination with internal providers
- Performance optimization and caching

### Dependencies
- **Async Runtime**: `tokio` for concurrent LSP operations
- **Process Management**: `tokio::process` for LSP server spawning
- **Serialization**: `serde` for LSP protocol messages
- **Storage**: `ricecoder-storage` for configuration persistence

### Integration Points
- **LSP**: Provides semantic intelligence to LSP integration layer
- **Completion**: Supplies external LSP completions to completion engine
- **TUI**: LSP diagnostics and hover information in terminal interface
- **Storage**: Persists LSP server configurations and connection state

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-external-lsp = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_external_lsp::{LspServerManager, LspConfig};

// Create LSP server manager
let manager = LspServerManager::new().await?;

// Configure LSP servers
let rust_config = LspConfig {
    name: "rust-analyzer".to_string(),
    command: "rust-analyzer".to_string(),
    args: vec![],
    ..Default::default()
};

manager.register_server(rust_config).await?;

// Make LSP request
let completion = manager.request_completion("rust", position, context).await?;
```

### Server Management

```rust
use ricecoder_external_lsp::server::LspServerPool;

// Create server pool
let pool = LspServerPool::new();

// Start LSP server
let server_id = pool.start_server("rust-analyzer", &config).await?;

// Check server health
let health = pool.check_server_health(server_id).await?;
match health {
    HealthStatus::Healthy => println!("Server is healthy"),
    HealthStatus::Unhealthy => println!("Server needs restart"),
}
```

### Request Routing

```rust
use ricecoder_external_lsp::router::LspRequestRouter;

// Create request router
let router = LspRequestRouter::new();

// Route request to appropriate server
let response = router.route_request("completion", params, "rust").await?;
println!("Completion items: {}", response.items.len());
```

## Configuration

External LSP configuration via YAML:

```yaml
external_lsp:
  # Server configurations
  servers:
    rust-analyzer:
      command: "rust-analyzer"
      args: []
      env: {}
      working_dir: null
      timeout_seconds: 30
      restart_on_crash: true

    typescript-language-server:
      command: "typescript-language-server"
      args: ["--stdio"]
      env: { "TSSERVER_LOG_FILE": "/tmp/tsserver.log" }
      timeout_seconds: 60

    pylsp:
      command: "pylsp"
      args: []
      timeout_seconds: 45

  # Connection settings
  connection:
    max_connections_per_server: 5
    connection_timeout_seconds: 10
    request_timeout_seconds: 30

  # Health monitoring
  health:
    check_interval_seconds: 60
    max_restart_attempts: 3
    restart_delay_seconds: 5

  # Fallback settings
  fallback:
    enable_internal_fallback: true
    fallback_timeout_seconds: 5
```

## API Reference

### Key Types

- **`LspServerManager`**: Main LSP server orchestration
- **`LspServerPool`**: Connection pooling for LSP servers
- **`LspRequestRouter`**: Request routing and load balancing
- **`LspConfig`**: LSP server configuration
- **`HealthStatus`**: Server health monitoring

### Key Functions

- **`register_server()`**: Register new LSP server configuration
- **`request_completion()`**: Request code completion from LSP
- **`check_server_health()`**: Check LSP server availability
- **`route_request()`**: Route request to appropriate server

## Error Handling

```rust
use ricecoder_external_lsp::LspError;

match manager.request_completion("rust", position, context).await {
    Ok(completion) => println!("Got {} completions", completion.items.len()),
    Err(LspError::ServerUnavailable) => {
        // Fall back to internal completion
        use_internal_completion().await?
    }
    Err(LspError::RequestTimeout) => eprintln!("LSP request timed out"),
    Err(LspError::ServerCrashed) => eprintln!("LSP server crashed"),
}
```

## Testing

Run comprehensive external LSP tests:

```bash
# Run all tests
cargo test -p ricecoder-external-lsp

# Run property tests for LSP behavior
cargo test -p ricecoder-external-lsp property

# Test server management
cargo test -p ricecoder-external-lsp server

# Test request routing
cargo test -p ricecoder-external-lsp router
```

Key test areas:
- LSP server lifecycle management
- Request/response transformation
- Connection pooling and health checks
- Fallback behavior and error handling
- Configuration loading and validation

## Performance

- **Server Startup**: < 2s for typical LSP servers
- **Request Latency**: < 100ms for cached responses, 200-2000ms for LSP calls
- **Connection Pooling**: < 10ms connection acquisition
- **Health Checks**: < 50ms per server status check
- **Memory**: Efficient caching with configurable limits

## Contributing

When working with `ricecoder-external-lsp`:

1. **Protocol Compliance**: Ensure LSP protocol compliance across versions
2. **Resource Management**: Properly manage LSP server processes and connections
3. **Fallback Robustness**: Ensure seamless fallback to internal providers
4. **Configuration**: Make server configuration intuitive and discoverable
5. **Testing**: Test with real LSP servers and various failure scenarios

## License

MIT
