# ricecoder-commands

**Purpose**: Custom command system with template substitution and output injection for RiceCoder

## Overview

`ricecoder-commands` provides a comprehensive system for defining, managing, and executing custom commands in RiceCoder. Commands support template substitution, argument validation, timeout handling, and automatic output injection into chat sessions. This crate enables users to extend RiceCoder's functionality with custom shell commands and scripts.

## Features

- **Command Definition**: Structured command definitions with arguments, descriptions, and metadata
- **Template Substitution**: Dynamic command generation with `{{variable}}` syntax
- **Argument Validation**: Type-safe arguments (string, number, boolean) with validation
- **Output Injection**: Automatic injection of command output into chat sessions
- **Timeout Handling**: Configurable execution timeouts to prevent hanging commands
- **Command Registry**: Centralized command management and discovery
- **Configuration Management**: Persistent command definitions and settings
- **Execution Context**: Rich execution context with environment variables and working directory
- **Error Handling**: Comprehensive error types and recovery mechanisms

## Architecture

### Responsibilities
- Command definition parsing and validation
- Template processing and variable substitution
- Command execution with timeout and error handling
- Output formatting and chat injection
- Command registry management and persistence
- Configuration loading and validation

### Dependencies
- **Infrastructure**: `serde`, `tokio`, `thiserror`, `tracing`
- **Template Processing**: `regex` for variable substitution
- **Data Handling**: `uuid`, `chrono` for command metadata
- **Integration**: Works with `ricecoder-storage` for persistence

### Integration Points
- **CLI**: Command execution and management through CLI interface
- **TUI**: Command selection and execution through terminal interface
- **Sessions**: Output injection into active chat sessions
- **Storage**: Command definitions persisted via storage layer

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-commands = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_commands::{CommandRegistry, CommandDefinition, CommandArgument, ArgumentType};

// Create a command registry
let mut registry = CommandRegistry::new();

// Define a custom command
let cmd = CommandDefinition::new(
    "greet",
    "Greet User",
    "echo 'Hello {{name}}!'"
)
.with_description("Greet a user by name")
.with_argument(
    CommandArgument::new("name", ArgumentType::String)
        .with_description("User name")
        .with_required(true)
)
.with_inject_output(true);

// Register the command
registry.register(cmd)?;
```

### Advanced Usage

```rust
use ricecoder_commands::{CommandExecutor, CommandContext, OutputInjector};

// Create execution context
let context = CommandContext {
    working_dir: Some("/tmp".into()),
    env_vars: vec![("DEBUG".into(), "1".into())],
    timeout_seconds: 30,
};

// Execute a command
let executor = CommandExecutor::new();
let result = executor.execute("greet", &[("name", "Alice")], &context).await?;

// Inject output into chat
let injector = OutputInjector::new();
injector.inject_output(&result.output, OutputFormat::Markdown)?;
```

### Command Definition with Templates

```rust
use ricecoder_commands::{CommandDefinition, CommandArgument, ArgumentType};

// Command with multiple arguments and complex template
let cmd = CommandDefinition::new(
    "deploy",
    "Deploy Application",
    "cd {{project_dir}} && ./deploy.sh --env {{environment}} --version {{version}}"
)
.with_description("Deploy application to specified environment")
.with_argument(
    CommandArgument::new("project_dir", ArgumentType::String)
        .with_description("Project directory path")
        .with_required(true)
)
.with_argument(
    CommandArgument::new("environment", ArgumentType::String)
        .with_description("Target environment (dev/staging/prod)")
        .with_default("dev")
)
.with_argument(
    CommandArgument::new("version", ArgumentType::String)
        .with_description("Application version")
        .with_required(true)
)
.with_timeout_seconds(300) // 5 minute timeout
.with_tags(vec!["deployment".into(), "automation".into()]);
```

## Configuration

Commands can be configured via YAML or JSON:

```yaml
commands:
  - id: "build"
    name: "Build Project"
    description: "Build the current project"
    command: "cargo build --release"
    enabled: true
    inject_output: false
    timeout_seconds: 300
    tags: ["development", "rust"]

  - id: "test"
    name: "Run Tests"
    description: "Execute test suite"
    command: "cargo test {{test_filter}}"
    arguments:
      - name: "test_filter"
        type: "string"
        description: "Test filter pattern"
        required: false
        default: ""
    enabled: true
    inject_output: true
    timeout_seconds: 600
    tags: ["testing", "quality"]
```

## API Reference

### Key Types

- **`CommandDefinition`**: Core command structure with metadata and arguments
- **`CommandRegistry`**: Central registry for command management
- **`CommandExecutor`**: Handles command execution with timeout and error handling
- **`CommandContext`**: Execution context (working directory, environment, timeout)
- **`OutputInjector`**: Handles output formatting and chat injection
- **`TemplateProcessor`**: Processes template substitution in commands

### Key Functions

- **`CommandRegistry::register()`**: Register a new command definition
- **`CommandRegistry::get()`**: Retrieve command by ID
- **`CommandExecutor::execute()`**: Execute command with arguments and context
- **`TemplateProcessor::process()`**: Substitute variables in command templates
- **`OutputInjector::inject_output()`**: Inject command output into chat

## Error Handling

```rust
use ricecoder_commands::{CommandError, Result};

match result {
    Ok(output) => println!("Command executed: {}", output),
    Err(CommandError::CommandNotFound(id)) => eprintln!("Command '{}' not found", id),
    Err(CommandError::ArgumentError(msg)) => eprintln!("Invalid arguments: {}", msg),
    Err(CommandError::ExecutionError(msg)) => eprintln!("Execution failed: {}", msg),
    Err(CommandError::TimeoutError) => eprintln!("Command timed out"),
    Err(CommandError::TemplateError(msg)) => eprintln!("Template error: {}", msg),
}
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test -p ricecoder-commands

# Run with coverage
cargo tarpaulin -p ricecoder-commands

# Run property tests
cargo test -p ricecoder-commands -- --ignored
```

Key test areas:
- Command definition validation
- Template substitution correctness
- Execution timeout handling
- Output injection formatting
- Registry operations
- Error condition handling

## Performance

- **Template Processing**: Regex-based substitution, optimized for common patterns
- **Command Execution**: Async execution with configurable timeouts
- **Registry Operations**: HashMap-based lookups, O(1) average case
- **Memory Usage**: Minimal overhead, streams large command outputs
- **Concurrent Execution**: Safe for concurrent command execution

## Contributing

When working with `ricecoder-commands`:

1. **Template Safety**: Ensure template substitution doesn't allow code injection
2. **Timeout Defaults**: Set reasonable default timeouts for commands
3. **Error Clarity**: Provide clear, actionable error messages
4. **Testing**: Test both success and failure scenarios thoroughly
5. **Documentation**: Document command arguments and expected behavior

## License

MIT