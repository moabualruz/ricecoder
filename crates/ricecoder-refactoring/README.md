# ricecoder-refactoring

**Purpose**: Safe, language-agnostic refactoring engine with impact analysis, preview, and rollback capabilities for RiceCoder

## DDD Layer

**Application** - Refactoring orchestration as an application service layer.

## Overview

`ricecoder-refactoring` implements a comprehensive refactoring system that provides safe, automated code transformations across multiple programming languages. It features impact analysis, change preview, validation, and automatic rollback to ensure reliable refactoring operations.

## Features

- **Language-Agnostic Core**: Generic refactoring engine supporting any programming language
- **Configuration-Driven**: Language-specific behavior defined in YAML/JSON configurations
- **Impact Analysis**: Comprehensive analysis of refactoring effects before execution
- **Change Preview**: Preview all changes before applying them
- **Safe Execution**: Automatic backups and rollback capabilities
- **Multi-Language Support**: Specialized providers for Rust, TypeScript, Python, and generic fallback
- **Validation**: Pre and post-refactoring validation to ensure correctness
- **Batch Operations**: Support for complex multi-file refactoring operations

## Architecture

### Responsibilities
- Refactoring operation orchestration and execution
- Impact analysis and change prediction
- Configuration management for language-specific rules
- Validation and safety checking
- Rollback and recovery mechanisms
- Performance optimization for large codebases

### Dependencies
- **Parsing**: `tree-sitter` for syntax analysis
- **Storage**: `ricecoder-storage` for configuration and backup management
- **Async Runtime**: `tokio` for concurrent operations
- **Serialization**: `serde` for configuration handling

### Integration Points
- **LSP**: Integrates with language servers for semantic refactoring
- **Files**: Safe file operations with backup and rollback
- **TUI**: Refactoring preview and confirmation interfaces
- **Commands**: CLI commands for refactoring operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-refactoring = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_refactoring::{RefactoringEngine, RefactoringConfig};

// Create refactoring engine
let engine = RefactoringEngine::new(config).await?;

// Analyze refactoring impact
let analysis = engine.analyze_refactoring("rename-function", params).await?;
println!("Affected files: {}", analysis.affected_files.len());

// Execute refactoring with preview
let preview = engine.preview_refactoring("rename-function", params).await?;
if user_approves(preview) {
    engine.execute_refactoring("rename-function", params).await?;
}
```

### Language-Specific Refactoring

```rust
use ricecoder_refactoring::languages::RustRefactoringProvider;

// Create language-specific provider
let rust_provider = RustRefactoringProvider::new();

// Perform Rust-specific refactoring
let result = rust_provider.rename_struct("OldStruct", "NewStruct", &files).await?;
println!("Refactored {} files", result.modified_files.len());
```

### Configuration Management

```rust
use ricecoder_refactoring::config::ConfigManager;

// Load refactoring configurations
let config_manager = ConfigManager::new(storage).await?;
let rust_config = config_manager.load_language_config("rust").await?;

// Apply custom refactoring rules
config_manager.apply_custom_rules("rust", custom_rules).await?;
```

## Configuration

Refactoring configuration via YAML:

```yaml
refactoring:
  # Language configurations
  languages:
    rust:
      enabled: true
      rules:
        - name: "rename-struct"
          pattern: "struct\\s+(\\w+)"
          replacement: "struct {{new_name}}"
        - name: "extract-method"
          min_lines: 10
          max_complexity: 15

    typescript:
      enabled: true
      rules:
        - name: "rename-interface"
          pattern: "interface\\s+(\\w+)"
          replacement: "interface {{new_name}}"

  # Safety settings
  safety:
    require_preview: true
    auto_backup: true
    validate_after_refactor: true
    max_files_per_operation: 50

  # Performance settings
  performance:
    parallel_processing: true
    max_concurrent_files: 10
    cache_parsed_files: true
```

## API Reference

### Key Types

- **`RefactoringEngine`**: Main refactoring orchestration engine
- **`RefactoringAnalysis`**: Impact analysis results
- **`RefactoringPreview`**: Preview of planned changes
- **`LanguageProvider`**: Language-specific refactoring logic
- **`ConfigManager`**: Configuration management for rules

### Key Functions

- **`analyze_refactoring()`**: Analyze refactoring impact
- **`preview_refactoring()`**: Preview changes before execution
- **`execute_refactoring()`**: Execute refactoring with safety checks
- **`rollback_refactoring()`**: Rollback to pre-refactoring state

## Error Handling

```rust
use ricecoder_refactoring::RefactoringError;

match engine.execute_refactoring("rename-function", params).await {
    Ok(result) => println!("Refactored {} files", result.modified_files.len()),
    Err(RefactoringError::AnalysisFailed(msg)) => eprintln!("Analysis failed: {}", msg),
    Err(RefactoringError::ValidationFailed(msg)) => eprintln!("Validation failed: {}", msg),
    Err(RefactoringError::RollbackFailed(msg)) => eprintln!("Rollback failed: {}", msg),
}
```

## Testing

Run comprehensive refactoring tests:

```bash
# Run all tests
cargo test -p ricecoder-refactoring

# Run property tests for refactoring correctness
cargo test -p ricecoder-refactoring property

# Test language-specific providers
cargo test -p ricecoder-refactoring languages

# Test impact analysis
cargo test -p ricecoder-refactoring analysis
```

Key test areas:
- Refactoring correctness across languages
- Impact analysis accuracy
- Preview and validation functionality
- Rollback and recovery mechanisms
- Configuration loading and application

## Performance

- **Impact Analysis**: < 500ms for typical codebases (< 100 files)
- **Change Preview**: < 200ms for preview generation
- **Refactoring Execution**: Variable based on scope (100ms - 10s)
- **Validation**: < 300ms for post-refactoring checks
- **Rollback**: < 1s for typical rollback operations

## Contributing

When working with `ricecoder-refactoring`:

1. **Safety First**: Ensure all operations are safe and reversible
2. **Language Agnostic**: Keep core engine language-independent
3. **Validation**: Implement comprehensive pre and post-validation
4. **Performance**: Optimize for large-scale refactoring operations
5. **Testing**: Test refactoring scenarios across multiple languages

## License

MIT
