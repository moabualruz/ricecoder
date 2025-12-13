# ricecoder-modes

**Purpose**: Flexible mode system providing different interaction patterns (Code, Ask, Vibe) with extended reasoning capabilities for RiceCoder

## Overview

`ricecoder-modes` implements a sophisticated mode system that adapts RiceCoder's behavior based on user intent and task complexity. It provides specialized interaction patterns optimized for different use cases, from focused code generation to exploratory conversations, with automatic activation of extended reasoning for complex tasks.

## Features

- **Code Mode**: Focused code generation and modification with file system integration
- **Ask Mode**: Question answering and information retrieval without file modifications
- **Vibe Mode**: Free-form exploration and rapid prototyping with relaxed constraints
- **Think More**: Extended reasoning system that activates automatically for complex tasks
- **Mode Switching**: Seamless transitions between modes with context preservation
- **Auto-Enable Logic**: Intelligent activation of extended reasoning based on task complexity
- **Task Configuration**: Per-task mode settings and behavior customization

## Architecture

### Responsibilities
- Mode lifecycle management and transitions
- Task complexity assessment and auto-enable decisions
- Extended reasoning orchestration and performance management
- Context preservation during mode switches
- Mode-specific behavior configuration and validation

### Dependencies
- **Async Runtime**: `tokio` for concurrent mode operations
- **Serialization**: `serde` for mode configuration persistence
- **Storage**: `ricecoder-storage` for mode settings and task configurations

### Integration Points
- **TUI**: Provides mode indicators and switching interfaces
- **Sessions**: Applies mode-specific behavior to conversations
- **Providers**: Configures AI provider behavior based on active mode
- **Storage**: Persists mode preferences and task configurations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-modes = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_modes::{ModeManager, CodeMode, AskMode, VibeMode};

// Create mode manager
let manager = ModeManager::new();

// Switch to code mode for focused development
manager.switch_to(Box::new(CodeMode::new())).await?;

// Switch to ask mode for questions
manager.switch_to(Box::new(AskMode::new())).await?;
```

### Advanced Usage with Think More

```rust
use ricecoder_modes::{ModeManager, ThinkMoreController, TaskComplexity};

// Create think more controller
let think_more = ThinkMoreController::new();

// Assess task complexity
let complexity = TaskComplexity::from_task_description("Implement a distributed caching system")?;

// Auto-enable extended reasoning for complex tasks
if think_more.should_activate(complexity) {
    think_more.activate_extended_reasoning().await?;
}
```

### Mode Switching with Context

```rust
use ricecoder_modes::{ModeSwitcher, ModeContext};

// Create mode switcher
let switcher = ModeSwitcher::new();

// Switch modes with context preservation
let context = ModeContext::current();
switcher.switch_with_context(target_mode, context).await?;
```

## Configuration

Mode behavior is configured via YAML:

```yaml
modes:
  # Default mode settings
  default_mode: "code"

  # Code mode configuration
  code:
    file_modification: true
    syntax_validation: true
    import_optimization: true

  # Ask mode configuration
  ask:
    file_modification: false
    web_search_enabled: true
    knowledge_cutoff: "2024-01"

  # Vibe mode configuration
  vibe:
    constraints_relaxed: true
    experimental_features: true
    rapid_prototyping: true

  # Think More configuration
  think_more:
    auto_enable_threshold: 0.7
    max_reasoning_time: 300
    reasoning_depth: "deep"
    performance_tradeoffs:
      speed_vs_accuracy: "accuracy"
```

## API Reference

### Key Types

- **`ModeManager`**: Central mode lifecycle and switching coordinator
- **`CodeMode`**: Focused code generation and file modification mode
- **`AskMode`**: Question answering and information retrieval mode
- **`VibeMode`**: Free-form exploration and prototyping mode
- **`ThinkMoreController`**: Extended reasoning system controller

### Key Functions

- **`switch_to()`**: Switch to a specific mode
- **`should_activate()`**: Determine if extended reasoning should activate
- **`activate_extended_reasoning()`**: Enable deep thinking for complex tasks
- **`switch_with_context()`**: Switch modes while preserving context

## Error Handling

```rust
use ricecoder_modes::ModeError;

match manager.switch_to(target_mode).await {
    Ok(()) => println!("Mode switched successfully"),
    Err(ModeError::InvalidTransition(from, to)) => eprintln!("Cannot switch from {} to {}", from, to),
    Err(ModeError::ConfigurationError(msg)) => eprintln!("Configuration error: {}", msg),
    Err(ModeError::ThinkMoreTimeout) => eprintln!("Extended reasoning timed out"),
}
```

## Testing

Run comprehensive mode tests:

```bash
# Run all tests
cargo test -p ricecoder-modes

# Run property tests for mode behavior
cargo test -p ricecoder-modes property

# Test mode switching logic
cargo test -p ricecoder-modes switching

# Test think more activation
cargo test -p ricecoder-modes think_more
```

Key test areas:
- Mode transition correctness
- Think more auto-activation logic
- Context preservation during switches
- Performance trade-offs in extended reasoning
- Configuration validation

## Performance

- **Mode Switching**: < 10ms for mode transitions
- **Complexity Assessment**: < 5ms for task analysis
- **Think More Activation**: Variable based on reasoning depth (50ms - 5s)
- **Context Preservation**: < 20ms for state serialization
- **Memory**: Minimal overhead with lazy loading of mode implementations

## Contributing

When working with `ricecoder-modes`:

1. **Mode Clarity**: Keep mode purposes and behaviors clearly distinct
2. **Performance Balance**: Balance reasoning depth with response time
3. **Context Preservation**: Ensure seamless transitions between modes
4. **Auto-Enable Logic**: Test complexity assessment thoroughly
5. **Configuration**: Make mode settings intuitive and discoverable

## License

MIT