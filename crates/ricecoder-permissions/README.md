# ricecoder-permissions

**Purpose**: Fine-grained access control system with allow/ask/deny levels, per-agent overrides, glob patterns, and comprehensive audit logging for RiceCoder

## Overview

`ricecoder-permissions` implements a sophisticated permission system that provides granular access control for RiceCoder operations. It supports hierarchical permissions with allow/ask/deny levels, per-agent overrides, glob pattern matching, and complete audit logging to ensure security and compliance.

## Features

- **Hierarchical Permissions**: Organization → Team → Project → User permission inheritance
- **Permission Levels**: Allow, Ask (user prompt), and Deny for flexible control
- **Per-Agent Overrides**: Custom permissions for specific AI agents and tools
- **Glob Pattern Matching**: File and command pattern-based access control
- **Audit Logging**: Comprehensive logging of all permission decisions and actions
- **Interactive Prompts**: User-friendly permission requests with context
- **Repository Storage**: Persistent permission configurations with multiple backends
- **Security Headers**: HTTP security headers for API protection

## Architecture

### Responsibilities
- Permission evaluation and enforcement for all operations
- User interaction for permission requests and approvals
- Audit logging and compliance tracking
- Permission configuration management and validation
- Agent-specific permission overrides
- Pattern-based access control evaluation

### Dependencies
- **Pattern Matching**: `glob` for file pattern matching
- **Async Runtime**: `tokio` for concurrent permission checks
- **Storage**: `ricecoder-storage` for audit log persistence
- **HTTP**: Security header management for API calls

### Integration Points
- **All Crates**: Enforces permissions across the entire RiceCoder ecosystem
- **Agents**: Applies agent-specific permission overrides
- **TUI**: Provides permission request interfaces
- **Storage**: Persists audit logs and permission configurations
- **Commands**: Validates command execution permissions

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-permissions = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_permissions::{PermissionManager, PermissionLevel};

// Create permission manager
let manager = PermissionManager::new(config).await?;

// Check permission for operation
let allowed = manager.check_permission(
    user_id,
    PermissionLevel::Allow,
    "file:read",
    Some("/etc/passwd")
).await?;

if allowed {
    // Execute operation
    read_file(path).await?;
}
```

### Permission Configuration

```rust
use ricecoder_permissions::{PermissionConfig, ToolPermission};

// Create permission configuration
let config = PermissionConfig {
    default_level: PermissionLevel::Ask,
    tools: vec![
        ToolPermission {
            name: "file:*".to_string(),
            level: PermissionLevel::Allow,
            patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
        },
        ToolPermission {
            name: "command:execute".to_string(),
            level: PermissionLevel::Deny,
            patterns: vec!["rm *".to_string(), "sudo *".to_string()],
        },
    ],
    ..Default::default()
};
```

### Interactive Permission Prompts

```rust
use ricecoder_permissions::prompt::{PermissionPrompt, UserDecision};

// Create permission prompt
let prompt = PermissionPrompt::new(
    "Execute dangerous command",
    "This command may modify system files",
    &context
);

// Request user decision
match prompt.request_decision().await? {
    UserDecision::Allow => execute_command().await?,
    UserDecision::AllowAlways => {
        // Update permissions to allow permanently
        manager.update_permission(user_id, permission, PermissionLevel::Allow).await?;
        execute_command().await?;
    }
    UserDecision::Deny => return Err(PermissionDenied),
}
```

### Audit Logging

```rust
use ricecoder_permissions::audit::AuditLogger;

// Create audit logger
let logger = AuditLogger::new(storage_config).await?;

// Log permission decision
logger.log_decision(AuditLogEntry {
    user_id: user_id.to_string(),
    operation: "file:write".to_string(),
    resource: path.to_string(),
    decision: PermissionLevel::Allow,
    timestamp: Utc::now(),
    context: operation_context,
}).await?;
```

## Configuration

Permission system configuration via YAML:

```yaml
permissions:
  # Default permission level
  default_level: "ask"

  # Tool-specific permissions
  tools:
    - name: "file:read"
      level: "allow"
      patterns: ["*.rs", "*.toml", "*.md"]
      agents: ["code-assistant", "reviewer"]

    - name: "file:write"
      level: "ask"
      patterns: ["*.rs", "*.toml"]
      exclude_patterns: ["*.lock", "target/*"]

    - name: "command:execute"
      level: "deny"
      patterns: ["rm *", "sudo *", "format *"]

  # Agent overrides
  agents:
    admin-assistant:
      level: "allow"
      tools: ["*"]
    code-assistant:
      level: "ask"
      tools: ["file:*", "git:*"]

  # Audit settings
  audit:
    enabled: true
    log_level: "info"
    retention_days: 90
    storage: "file"
```

## API Reference

### Key Types

- **`PermissionManager`**: Central permission evaluation and management
- **`PermissionConfig`**: Permission configuration structure
- **`AuditLogger`**: Audit logging and compliance tracking
- **`PermissionPrompt`**: Interactive user permission requests
- **`GlobMatcher`**: Pattern-based permission matching

### Key Functions

- **`check_permission()`**: Evaluate permission for operation
- **`request_decision()`**: Interactive permission prompting
- **`log_decision()`**: Audit logging of permission decisions
- **`update_permission()`**: Modify permission configurations

## Error Handling

```rust
use ricecoder_permissions::Error;

match manager.check_permission(user, level, operation, resource).await {
    Ok(true) => execute_operation().await?,
    Ok(false) => return Err(PermissionDenied),
    Err(Error::ConfigurationError(msg)) => eprintln!("Config error: {}", msg),
    Err(Error::StorageError(msg)) => eprintln!("Storage error: {}", msg),
}
```

## Testing

Run comprehensive permission tests:

```bash
# Run all tests
cargo test -p ricecoder-permissions

# Run property tests for permission logic
cargo test -p ricecoder-permissions property

# Test audit logging
cargo test -p ricecoder-permissions audit

# Test pattern matching
cargo test -p ricecoder-permissions glob
```

Key test areas:
- Permission evaluation correctness
- Pattern matching accuracy
- Audit logging completeness
- Interactive prompt handling
- Configuration validation

## Performance

- **Permission Check**: < 5ms for typical evaluations
- **Pattern Matching**: < 2ms for glob pattern evaluation
- **Audit Logging**: < 10ms for log entry persistence
- **Configuration Loading**: < 20ms for permission config loading
- **Concurrent Checks**: Safe for multiple simultaneous evaluations

## Contributing

When working with `ricecoder-permissions`:

1. **Security First**: Ensure permission checks cannot be bypassed
2. **Auditability**: All permission decisions must be logged
3. **User Experience**: Make permission prompts clear and actionable
4. **Performance**: Optimize for low-latency permission checks
5. **Testing**: Test both allowed and denied scenarios thoroughly

## License

MIT