# ricecoder-mcp

**Purpose**: Model Context Protocol support for extending RiceCoder with custom tools and service integrations

## Overview

`ricecoder-mcp` implements the Model Context Protocol (MCP) to enable seamless integration of external tools and services into RiceCoder. It provides a standardized interface for tool discovery, execution, and management, with built-in connection pooling, error recovery, and permission system integration.

## Features

- **MCP Client Implementation**: Full MCP protocol support for tool communication
- **Tool Registry**: Dynamic tool discovery and registration system
- **Connection Pooling**: Efficient management of MCP server connections
- **Error Recovery**: Automatic retry logic and graceful error handling
- **Permission Integration**: Seamless integration with RiceCoder's permission system
- **Hot Reload**: Dynamic tool loading without restarting RiceCoder
- **Health Monitoring**: MCP server health checks and status monitoring
- **Metadata Management**: Tool metadata storage and retrieval

## Architecture

### Responsibilities
- MCP protocol implementation and message handling
- Tool discovery, registration, and lifecycle management
- Connection pooling and resource management
- Error recovery and retry logic implementation
- Permission system integration and access control
- Health monitoring and status reporting
- Hot reload capability for dynamic tool updates

### Dependencies
- **Async Runtime**: `tokio` for concurrent MCP operations
- **HTTP/WebSocket**: `reqwest` and `tokio-tungstenite` for MCP transport
- **Serialization**: `serde` for MCP message handling
- **Storage**: `ricecoder-storage` for tool metadata and configuration

### Integration Points
- **Tools**: Extends RiceCoder with custom tools via MCP
- **Agents**: Provides tools for agent execution workflows
- **Permissions**: Integrates tool access with permission system
- **Storage**: Persists tool configurations and metadata

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-mcp = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_mcp::{MCPClient, MCPConfig};

// Create MCP client
let config = MCPConfig::default();
let client = MCPClient::new(config).await?;

// Connect to MCP server
client.connect("http://localhost:3000").await?;

// List available tools
let tools = client.list_tools().await?;
for tool in tools {
    println!("Tool: {} - {}", tool.name, tool.description);
}
```

### Tool Execution

```rust
use ricecoder_mcp::ToolExecutionContext;

// Create execution context
let context = ToolExecutionContext {
    tool_name: "web-scraper".to_string(),
    parameters: serde_json::json!({"url": "https://example.com"}),
    timeout: Some(Duration::from_secs(30)),
};

// Execute tool
let result = client.execute_tool(context).await?;
println!("Result: {:?}", result.output);
```

### Tool Registry

```rust
use ricecoder_mcp::registry::ToolRegistry;

// Create tool registry
let registry = ToolRegistry::new();

// Register MCP server
registry.register_server("my-tools", "http://localhost:3000").await?;

// Discover tools from server
let tools = registry.discover_tools("my-tools").await?;
println!("Discovered {} tools", tools.len());
```

### Connection Pooling

```rust
use ricecoder_mcp::connection_pool::ConnectionPool;

// Create connection pool
let pool_config = PoolConfig {
    max_connections: 10,
    connection_timeout: Duration::from_secs(30),
    ..Default::default()
};
let pool = ConnectionPool::new(pool_config);

// Get connection from pool
let connection = pool.get_connection("my-server").await?;

// Use connection for MCP operations
let result = connection.execute_tool(context).await?;
```

## Configuration

MCP configuration via YAML:

```yaml
mcp:
  # Server configurations
  servers:
    web-tools:
      url: "http://localhost:3000"
      timeout_seconds: 30
      retry_attempts: 3
      auth_token: "${MCP_AUTH_TOKEN}"

    data-tools:
      url: "ws://localhost:3001"
      protocol: "websocket"
      connection_pool_size: 5

  # Tool settings
  tools:
    enable_discovery: true
    cache_metadata: true
    metadata_ttl_seconds: 3600

  # Permission integration
  permissions:
    require_tool_permissions: true
    default_deny: false

  # Error handling
  error_handling:
    enable_retry: true
    max_retry_attempts: 3
    retry_backoff_ms: 1000

  # Health monitoring
  health:
    check_interval_seconds: 60
    unhealthy_threshold: 3
    auto_restart: true
```

## API Reference

### Key Types

- **`MCPClient`**: Main MCP protocol client
- **`ToolRegistry`**: Tool discovery and registration
- **`ConnectionPool`**: Connection pooling for MCP servers
- **`ToolExecutionContext`**: Tool execution parameters
- **`MCPConfig`**: MCP client configuration

### Key Functions

- **`connect()`**: Connect to MCP server
- **`list_tools()`**: Discover available tools
- **`execute_tool()`**: Execute tool with parameters
- **`register_server()`**: Register MCP server endpoint

## Error Handling

```rust
use ricecoder_mcp::MCPError;

match client.execute_tool(context).await {
    Ok(result) => println!("Tool executed: {:?}", result),
    Err(MCPError::ConnectionFailed) => eprintln!("Failed to connect to MCP server"),
    Err(MCPError::ToolNotFound(name)) => eprintln!("Tool '{}' not found", name),
    Err(MCPError::ExecutionTimeout) => eprintln!("Tool execution timed out"),
    Err(MCPError::PermissionDenied) => eprintln!("Insufficient permissions for tool"),
}
```

## Testing

Run comprehensive MCP tests:

```bash
# Run all tests
cargo test -p ricecoder-mcp

# Run property tests for MCP behavior
cargo test -p ricecoder-mcp property

# Test tool execution
cargo test -p ricecoder-mcp tools

# Test connection pooling
cargo test -p ricecoder-mcp pool
```

Key test areas:
- MCP protocol compliance
- Tool discovery and execution
- Connection pooling and management
- Error recovery and retry logic
- Permission integration

## Performance

- **Connection Establishment**: < 500ms for HTTP, < 200ms for WebSocket
- **Tool Discovery**: < 100ms for cached metadata
- **Tool Execution**: Variable based on tool complexity (100ms - 30s)
- **Connection Pooling**: < 10ms connection acquisition
- **Health Checks**: < 50ms per server status check

## Contributing

When working with `ricecoder-mcp`:

1. **Protocol Compliance**: Ensure MCP protocol specification compliance
2. **Security**: Implement proper authentication and authorization
3. **Resource Management**: Efficient connection pooling and cleanup
4. **Error Handling**: Comprehensive error recovery and user feedback
5. **Testing**: Test with real MCP servers and various failure scenarios

## License

MIT
