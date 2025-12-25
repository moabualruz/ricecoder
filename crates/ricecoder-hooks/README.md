# ricecoder-hooks

**Purpose**: Event-driven hooks system providing automation, extensibility, and workflow integration for RiceCoder

## DDD Layer

**Infrastructure** - Event-driven infrastructure providing cross-cutting automation capabilities.

## Features

- **Event-Driven Architecture**: Comprehensive event system for workspace and file operations
- **Hook Chaining**: Sequential execution of multiple hooks with error handling and rollback
- **Configuration Management**: YAML-based hook configuration with conditional execution
- **Plugin Integration**: Extensible plugin system for custom hook implementations
- **Audit Logging**: Complete logging of hook execution, results, and performance metrics

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-hooks = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_hooks::{HookManager, HookEvent};

// Create hook manager
let manager = HookManager::new();

// Register a hook for file save events
manager.register_hook("file:save", |event: HookEvent| async move {
    match event {
        HookEvent::FileSaved { path, content } => {
            // Run linter or formatter
            run_linter(&path).await?;
            Ok(())
        }
        _ => Ok(())
    }
}).await?;

// Trigger hook execution
manager.trigger_hook("file:save", HookEvent::FileSaved {
    path: "main.rs".into(),
    content: "fn main() {}".into(),
}).await?;
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-hooks).

## License

MIT
