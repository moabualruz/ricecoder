# ricecoder-pty

PTY (pseudo-terminal) support for RiceCoder - enables interactive shell sessions and terminal management.

## Features

- ✅ Cross-platform PTY spawning (Windows ConPTY, Unix PTY)
- ✅ Multi-session management (create, list, get, update, delete)
- ✅ Bidirectional I/O (read/write with buffering)
- ✅ Terminal resize support
- ✅ Session lifecycle events
- ✅ Shell integration (auto-detection, login shell support)
- ✅ Environment/cwd configuration per session
- ✅ Graceful cleanup and signal handling

## Architecture

```
ricecoder-pty/
├── src/
│   ├── lib.rs          # Public API
│   ├── domain/         # Domain layer (SOLID, DDD)
│   │   ├── mod.rs
│   │   ├── session.rs      # PtySession entity
│   │   ├── config.rs       # PtyConfig value object
│   │   └── events.rs       # SessionEvent enum
│   ├── application/    # Application layer
│   │   ├── mod.rs
│   │   └── manager.rs      # SessionManager (CRUD operations)
│   ├── infrastructure/ # Infrastructure layer
│   │   ├── mod.rs
│   │   ├── backend.rs      # PtyBackend trait
│   │   └── portable.rs     # portable-pty implementation
│   └── error.rs        # Error types
```

## Usage

```rust
use ricecoder_pty::{SessionManager, PtyConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = SessionManager::new();
    
    // Create session
    let config = PtyConfig::default()
        .with_command("bash")
        .with_args(vec!["-l".to_string()])
        .with_cwd("/home/user");
    
    let session = manager.create(config).await?;
    
    // Write to session
    manager.write(&session.id, "echo hello\n").await?;
    
    // Subscribe to output
    let mut rx = manager.subscribe(&session.id).await?;
    while let Ok(data) = rx.recv().await {
        print!("{}", data);
    }
    
    Ok(())
}
```

## Dependencies

- **portable-pty** - Cross-platform PTY backend (used by WezTerm)
- **tokio** - Async runtime for I/O handling
- **thiserror** - Error type definitions

## OpenCode Compatibility

This crate implements feature parity with OpenCode's PTY module:
- All OpenCode PTY operations supported
- Compatible session lifecycle
- WebSocket-ready output streaming
- Helix-inspired async job patterns
