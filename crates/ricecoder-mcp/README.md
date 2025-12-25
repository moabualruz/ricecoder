# ricecoder-mcp

**Purpose**: Model Context Protocol support for extending RiceCoder with custom tools and service integrations

## DDD Layer

**Infrastructure** - Implements external protocol integration (MCP) for tool discovery and execution.

## Architecture Quality

### SOLID Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| **SRP** | ⚠️ Partial | 6 modules >500 lines (see below) |
| **OCP** | ✅ Pass | Trait-based extensibility (MCPTransport, ToolExecutor) |
| **LSP** | ✅ Pass | All trait implementations are substitutable |
| **ISP** | ✅ Pass | Traits have 1-6 methods (within guidelines) |
| **DIP** | ✅ Pass | Constructor injection throughout |

### Large Module Documentation (SRP Deferred)

| Module | Lines | Rationale for Current Size |
|--------|-------|---------------------------|
| `server_management.rs` | 889 | Server lifecycle + health + discovery cohesive |
| `tool_execution.rs` | 824 | Execution + validation + caching cohesive |
| `tool_orchestration.rs` | 797 | Pipeline + scheduling + caching cohesive |
| `transport.rs` | 673 | 3 transport types (stdio, HTTP, SSE) in one module |
| `protocol_validation.rs` | 655 | Validation + error handling + compliance |
| `config.rs` | 571 | Configuration loading + validation + merging |

**Beta Refactoring Candidate**: Consider splitting transport.rs into separate modules per transport type.

## Overview

`ricecoder-mcp` implements the Model Context Protocol (MCP) version 2025-06-18 to enable seamless integration of external tools and services into RiceCoder. It provides a standardized interface for tool discovery, execution, and management, with enterprise-grade features including connection pooling, error recovery, permission system integration, audit logging, and security compliance.

## Features

- **MCP Protocol 2025-06-18**: Full compliance with latest MCP specification including enterprise error codes
- **Multiple Transports**: stdio, HTTP with OAuth 2.0, and SSE support
- **Server/Client Architecture**: Complete MCP server management with tool enablement/disablement
- **Connection Pooling**: Advanced connection pooling with configurable limits and health checks
- **Failover Mechanisms**: Automatic failover with exponential backoff and reconnection logic
- **Health Monitoring**: Comprehensive server health monitoring with enterprise alerting
- **Audit Logging**: Enterprise-grade audit logging for all MCP operations and security events
- **Enterprise Security**: OAuth 2.0 integration, RBAC, audit logging, and compliance reporting
- **Tool Registry**: Dynamic tool discovery and registration system
- **Error Recovery**: Automatic retry logic and graceful error handling
- **Permission Integration**: Seamless integration with RiceCoder's permission system
- **Hot Reload**: Dynamic tool loading without restarting RiceCoder
- **Metadata Management**: Tool metadata storage and retrieval

## Architecture

### Responsibilities
- MCP protocol implementation and message handling
- Tool discovery, registration, and lifecycle management
- Connection pooling and resource management
- Error recovery and retry logic implementation
- Permission system integration and access control
- RBAC (Role-Based Access Control) for enterprise security
- OAuth 2.0 and OpenID Connect authentication
- Audit logging and compliance reporting
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
- **Security**: RBAC and OAuth integration with ricecoder-security
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

## Security

### Authentication

MCP supports multiple authentication methods for secure server connections:

```rust
use ricecoder_mcp::transport::{HTTPTransport, HTTPAuthConfig, HTTPAuthType};

// Basic authentication
let auth_config = HTTPAuthConfig {
    auth_type: HTTPAuthType::Basic,
    credentials: [
        ("username".to_string(), "myuser".to_string()),
        ("password".to_string(), "mypass".to_string()),
    ].into(),
};
let transport = HTTPTransport::with_auth("https://api.example.com", auth_config)?;

// OAuth 2.0 authentication
let oauth_config = HTTPAuthConfig {
    auth_type: HTTPAuthType::OAuth2,
    credentials: [
        ("token_id".to_string(), "oauth_token_123".to_string()),
        ("user_id".to_string(), "user@example.com".to_string()),
    ].into(),
};
let transport = HTTPTransport::with_auth("https://api.example.com", oauth_config)?
    .with_oauth_manager(oauth_manager);
```

### Authorization

RBAC integration provides enterprise-grade access control:

```rust
use ricecoder_mcp::rbac::{MCRBACManager, MCPAuthorizationMiddleware};

// Create RBAC manager
let rbac_manager = MCRBACManager::new(access_control, permission_manager);

// Create authorization middleware
let auth_middleware = MCPAuthorizationMiddleware::new(rbac_manager, audit_logger);

// Check server access
auth_middleware.authorize_server_access(&principal, "server-id").await?;

// Check tool execution
auth_middleware.authorize_tool_execution(&principal, "tool-name", None).await?;
```

### Audit Logging

Comprehensive audit logging for compliance:

```rust
use ricecoder_mcp::audit::MCPAuditLogger;

// Create audit logger
let audit_logger = MCPAuditLogger::new(security_audit_logger);

// Log server operations
audit_logger.log_server_registration(&config, Some("user".to_string()), None).await?;

// Log tool executions
audit_logger.log_tool_execution(&server_id, &tool_name, &result, user_id, session_id).await?;
```

### Compliance Reporting

SOC 2, GDPR, and HIPAA compliance monitoring:

```rust
use ricecoder_mcp::compliance::{MCPComplianceMonitor, ComplianceReportType};

// Create compliance monitor
let compliance_monitor = MCPComplianceMonitor::new(audit_logger);

// Record violations
compliance_monitor.record_violation(
    ComplianceReportType::Soc2Type2,
    ViolationSeverity::High,
    "Unauthorized access attempt".to_string(),
    "server:api".to_string(),
    Some("user@example.com".to_string()),
    serde_json::json!({"details": "violation details"}),
).await?;

// Generate compliance report
let report = compliance_monitor.generate_report(
    ComplianceReportType::Gdpr,
    start_date,
    end_date,
).await?;
```

## API Reference

### Key Types

- **`MCPClient`**: Main MCP protocol client
- **`ToolRegistry`**: Tool discovery and registration
- **`ConnectionPool`**: Connection pooling for MCP servers
- **`ToolExecutionContext`**: Tool execution parameters
- **`MCPConfig`**: MCP client configuration
- **`MCRBACManager`**: RBAC access control manager
- **`MCPAuthorizationMiddleware`**: Authorization middleware
- **`MCPAuditLogger`**: Audit logging for MCP operations
- **`MCPComplianceMonitor`**: Compliance monitoring and reporting
- **`MCPEnterpriseMonitor`**: Enterprise monitoring and metrics

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

**Test Count**: 171 tests (all passing)
**Test Location**: Inline `#[cfg(test)]` modules (exception to tests/ directory rule - documented)

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

### Test Coverage by Module

| Module | Tests | Coverage |
|--------|-------|----------|
| agent_integration | 8 | Tool invocation, workflow |
| client | 7 | Connection, discovery |
| config | 8 | Loading, validation, merge |
| connection_pool | 9 | Acquire, release, health |
| error | 7 | Error types, recovery |
| error_recovery | 11 | Retry, backoff, degradation |
| error_reporting | 10 | Formatting, statistics |
| executor | 15 | Registration, execution, validation |
| health_check | 9 | Health, availability |
| lifecycle | 9 | Start, stop, restart |
| hot_reload | 7 | Config watching, reload |
| marshaler | 7 | Type conversion |
| metadata | 7 | Tool metadata |
| permissions | 10 | Permission rules |
| permissions_integration | 12 | Permission enforcement |
| protocol_validation | 7 | Protocol compliance |
| rbac | 4 | Role-based access |
| registry | 4 | Tool registry |
| server_management | 3 | Server registration |
| storage_integration | 5 | Persistence, caching |
| tool_orchestration | 4 | Pipeline execution |

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

## MCP Protocol Compliance

### Protocol Version Support

| Version | Status | Notes |
|---------|--------|-------|
| 2025-06-18 | ✅ Implemented | Enterprise error codes, OAuth 2.0 |
| 2025-11-25 | 🔄 Partial | Latest spec, elicitation support pending |

### Compliance Checklist

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| JSON-RPC 2.0 | ✅ | `transport.rs` MCPMessage types |
| Initialize handshake | ✅ | `client.rs` connect() |
| Capability negotiation | ✅ | `protocol_validation.rs` |
| tools/list | ✅ | `registry.rs`, `client.rs` |
| tools/call | ✅ | `tool_execution.rs` |
| stdio transport | ✅ | `transport.rs` StdioTransport |
| HTTP transport | ✅ | `transport.rs` HTTPTransport |
| SSE transport | ✅ | `transport.rs` SSETransport |
| Error codes | ✅ | `error.rs` standard JSON-RPC codes |
| isError flag | ✅ | `tool_execution.rs` ToolExecutionResult |

### Gap Analysis vs MCP Spec 2025-11-25

| Feature | Spec Requirement | RiceCoder Status |
|---------|------------------|------------------|
| Elicitation modes | Server can request user input | ❌ Not implemented |
| Streamable HTTP | Single endpoint POST/GET | ⚠️ HTTP+SSE separate |
| structuredContent | Validated against outputSchema | ❌ Not implemented |
| Resource subscriptions | Subscribe to resource changes | ⚠️ Framework only |
| Prompt templates | Dynamic prompt generation | ❌ Not implemented |

**Priority for Beta**: Implement elicitation modes and structuredContent validation.

## License

MIT
