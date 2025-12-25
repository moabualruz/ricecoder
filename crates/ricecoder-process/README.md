# ricecoder-process

**Purpose**: Shared process lifecycle management for RiceCoder

## DDD Layer

**Infrastructure** - Low-level process management utilities used across multiple layers.

## Overview

`ricecoder-process` consolidates process management functionality from multiple locations into a single, well-tested crate. It provides cross-platform process spawning, PID tracking, graceful shutdown, timeout handling, and tree-kill functionality.

## Features

- **Process Spawning**: Async process creation with full stdio control
- **PID Tracking**: Track process IDs for monitoring and cleanup
- **Graceful Shutdown**: SIGTERM→SIGKILL escalation with configurable timeouts
- **Timeout Support**: Per-process and per-operation timeouts
- **Output Capture**: Capture stdout/stderr with buffering
- **Signal Handling**: Cross-platform signal delivery (Unix: SIGTERM/SIGKILL, Windows: taskkill)
- **Process Tree Kill**: Kill process groups on Unix, task trees on Windows

## Architecture

### Responsibilities
- Process spawning and configuration
- PID tracking and state management
- Graceful shutdown with escalation
- Process tree cleanup
- Cross-platform signal handling

### Dependencies
- **Async Runtime**: `tokio` for async process management
- **Signals (Unix)**: `nix` for Unix signal handling
- **Process Discovery (Windows)**: `which` for executable location

### Integration Points
- **LSP**: External LSP server process management
- **Execution**: Command execution and shell processes
- **Sessions**: Background process management
- **Security**: Vulnerability scanner process execution

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-process = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_process::{ProcessManager, ProcessConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create process manager
    let manager = ProcessManager::new();

    // Configure process
    let config = ProcessConfig::new("rust-analyzer")
        .args(&["--stdio"])
        .timeout_secs(30);

    // Spawn process
    let child = manager.spawn(config).await?;

    // Do work...

    // Clean shutdown
    manager.shutdown(child).await?;
    Ok(())
}
```

### Process Tree Kill

```rust
use ricecoder_process::{ProcessManager, ProcessConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ProcessManager::new();

    // Spawn a process that creates children
    let config = ProcessConfig::new("bash")
        .args(&["-c", "sleep 1000 & sleep 2000"]);

    let child = manager.spawn(config).await?;

    // Kill entire process tree
    manager.kill_tree(child).await?;
    Ok(())
}
```

### Environment Variables

```rust
use ricecoder_process::{ProcessManager, ProcessConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ProcessManager::new();

    let config = ProcessConfig::new("my-tool")
        .env("RUST_LOG", "debug")
        .env("API_KEY", "secret");

    let child = manager.spawn(config).await?;
    Ok(())
}
```

## Configuration

Process configuration via `ProcessConfig`:

```rust
use ricecoder_process::ProcessConfig;
use std::time::Duration;

let config = ProcessConfig::new("command")
    .args(&["arg1", "arg2"])
    .working_dir("/path/to/workdir")
    .env("VAR", "value")
    .timeout(Duration::from_secs(30))
    .capture_stdout(true)
    .capture_stderr(true);
```

## API Reference

### Key Types

- **`ProcessManager`**: Main process lifecycle manager
- **`ManagedChild`**: Wrapper around tokio::process::Child with enhanced features
- **`ProcessConfig`**: Process spawn configuration
- **`ProcessError`**: Process management errors

### Key Functions

- **`spawn()`**: Spawn a new managed process
- **`shutdown()`**: Gracefully shutdown a process
- **`kill_tree()`**: Kill process and all descendants
- **`wait()`**: Wait for process to exit (with optional timeout)

## Error Handling

```rust
use ricecoder_process::ProcessError;

match manager.spawn(config).await {
    Ok(child) => println!("Process started: PID {}", child.pid()),
    Err(ProcessError::SpawnFailed(e)) => eprintln!("Spawn failed: {}", e),
    Err(ProcessError::Timeout { seconds }) => eprintln!("Timed out after {}s", seconds),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

Run comprehensive process tests:

```bash
# Run all tests
cargo test -p ricecoder-process

# Test spawn/shutdown
cargo test -p ricecoder-process spawn

# Test kill tree
cargo test -p ricecoder-process kill_tree
```

Key test areas:
- Process spawning and configuration
- Graceful shutdown behavior
- Process tree cleanup
- Timeout handling
- Signal delivery (platform-specific)

## Performance

- **Spawn Time**: < 50ms for typical processes
- **Shutdown Time**: < 200ms (SIGTERM) + 5s max wait
- **Kill Tree Time**: < 250ms (SIGTERM + SIGKILL escalation)
- **Memory**: Minimal overhead (< 1KB per managed process)

## Contributing

When working with `ricecoder-process`:

1. **Cross-Platform**: Test on Windows, macOS, and Linux
2. **Signal Safety**: Use platform-appropriate signal handling
3. **Resource Cleanup**: Ensure processes are always cleaned up
4. **Error Handling**: Provide clear error messages for spawn/kill failures
5. **Testing**: Test edge cases (process already dead, permission denied, etc.)

## License

MIT
