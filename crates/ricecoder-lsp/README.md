# ricecoder-lsp

**Purpose**: Language Server Protocol integration providing semantic code analysis, diagnostics, and code intelligence.

## DDD Layer

**Infrastructure** - Implements external protocol integration (LSP) with semantic analysis capabilities.

## Overview

This crate implements a Language Server Protocol server that enables semantic understanding of code across multiple programming languages. It provides comprehensive code intelligence features including diagnostics, code actions, hover information, and symbol analysis.

## Responsibilities

- Implement LSP protocol handler (initialize, shutdown, document sync)
- Provide semantic analysis for Rust, TypeScript, and Python
- Generate diagnostics, code actions, and hover information
- Cache ASTs and symbol indexes for performance
- Proxy to external LSP servers when available

## Features

The crate provides:

- **Semantic Analysis**: Parse and analyze code structure for Rust, TypeScript, and Python
- **Diagnostics**: Generate errors, warnings, and hints for code issues
- **Code Actions**: Suggest fixes and refactorings for identified issues
- **Hover Information**: Display type information and documentation on hover
- **Multi-Language Support**: Extensible architecture for adding new languages
- **Performance Optimization**: Caching and performance tracking for efficient analysis

## Features

### Supported Languages

- **Rust**: Full semantic analysis with tree-sitter
- **TypeScript**: Full semantic analysis with tree-sitter
- **Python**: Full semantic analysis with tree-sitter
- **Unknown Languages**: Graceful degradation with basic analysis

### Core Capabilities

- **Symbol Extraction**: Extract functions, types, variables, classes, and other symbols
- **Import Tracking**: Track dependencies and imports
- **Diagnostic Generation**: Identify syntax errors, style issues, and potential bugs
- **Code Actions**: Suggest fixes for common issues
- **Hover Information**: Display type information and documentation
- **Caching**: Cache parsed ASTs and symbol indexes for performance
- **Performance Tracking**: Monitor analysis time and resource usage

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    LSP Client (IDE)                         │
└────────────────────────┬────────────────────────────────────┘
                         │ LSP Protocol (JSON-RPC)
                         │
┌────────────────────────▼────────────────────────────────────┐
│                  LSP Server (ricecoder-lsp)                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              LSP Protocol Handler                     │  │
│  │  - Initialize/Shutdown                               │  │
│  │  - Document Synchronization                          │  │
│  │  - Request Routing                                   │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Semantic Analysis Engine                    │  │
│  │  ┌────────────────┐  ┌────────────────┐              │  │
│  │  │  AST Parser    │  │  Symbol Index  │              │  │
│  │  │  - Rust        │  │  - Lookup      │              │  │
│  │  │  - TypeScript  │  │  - References  │              │  │
│  │  │  - Python      │  │  - Definitions │              │  │
│  │  └────────────────┘  └────────────────┘              │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Diagnostics & Code Actions                 │  │
│  │  - Issue Detection                                   │  │
│  │  - Fix Suggestions                                   │  │
│  │  - Code Transformations                              │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

### Internal (RiceCoder Crates)

- `ricecoder-storage`: Caching and configuration persistence
- `ricecoder-completion`: Code completion engine integration
- `ricecoder-refactoring`: Safe refactoring operations

### External Libraries

- `tree-sitter`: AST parsing for multiple languages
- `tree-sitter-rust`, `tree-sitter-typescript`, `tree-sitter-python`: Language grammars
- `serde`, `serde_json`: JSON-RPC serialization
- `tokio`: Async runtime for I/O operations
- `tracing`: Structured logging
- `ratatui`: TUI widget integration

## Key Types

- `LspServer`: Main LSP server handling initialization, shutdown, and request routing
- `SemanticAnalyzer`: Trait for language-specific semantic analysis
- `DiagnosticsEngine`: Trait for generating code diagnostics
- `CodeActionsEngine`: Trait for suggesting code fixes
- `HoverProvider`: Hover information provider
- `LspProxy`: External LSP server proxy for fallback support
- `Position`, `Range`, `Diagnostic`: Core LSP data types

## Integration Points

- **TUI**: Provides code intelligence for the terminal interface via `tui_integration` module
- **Completion**: Supplies semantic completion data via `CompletionHandler`
- **Refactoring**: Enables language-aware refactoring via `RefactoringHandler`
- **External LSP**: Proxies to rust-analyzer, tsserver, pylsp when available

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-lsp = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_lsp::LspServer;

// Create and run LSP server
let mut server = LspServer::new();
server.run().await?;
```

### Advanced Usage

```rust
use ricecoder_lsp::semantic::{SemanticAnalyzer, RustAnalyzer};

// Create language-specific analyzer
let analyzer = RustAnalyzer::new();

// Analyze code for symbols and diagnostics
let semantic_info = analyzer.analyze(code)?;
let symbols = analyzer.extract_symbols(code)?;
```

## Configuration

LSP behavior can be configured via environment variables:

```yaml
lsp:
  cache_size_mb: 100
  analysis_timeout_ms: 5000
  log_level: info
  languages:
    rust:
      enabled: true
      diagnostics: true
    typescript:
      enabled: true
      diagnostics: true
    python:
      enabled: true
      diagnostics: true
```

## API Reference

### LSP Server Interface

The main entry point for LSP functionality is the `LspServer` struct:

```rust
use ricecoder_lsp::LspServer;

// Create a new LSP server
let mut server = LspServer::new();

// Run the server (handles stdio communication)
server.run().await?;
```

**Key Methods**:

- `new()`: Create a new LSP server instance
- `run()`: Start the server and handle client requests
- `state()`: Get the current server state (Initializing, Running, Shutdown)

**Server Capabilities**:

- Text document synchronization (full sync)
- Hover information
- Code actions
- Diagnostics

### Semantic Analyzer Interface

The `SemanticAnalyzer` trait provides language-agnostic semantic analysis:

```rust
use ricecoder_lsp::semantic::{SemanticAnalyzer, RustAnalyzer};
use ricecoder_lsp::types::Language;

// Create a Rust analyzer
let analyzer = RustAnalyzer::new();

// Analyze code
let semantic_info = analyzer.analyze(code)?;

// Extract symbols
let symbols = analyzer.extract_symbols(code)?;

// Get hover information
let hover = analyzer.get_hover_info(code, position)?;
```

**Supported Analyzers**:

- `RustAnalyzer`: Rust code analysis
- `TypeScriptAnalyzer`: TypeScript code analysis
- `PythonAnalyzer`: Python code analysis
- `FallbackAnalyzer`: Fallback for unknown languages

**Key Methods**:

- `analyze(code: &str)`: Analyze code and extract semantic information
- `extract_symbols(code: &str)`: Extract all symbols from code
- `get_hover_info(code: &str, position: Position)`: Get hover information at a position
- `language()`: Get the supported language

### Diagnostics Engine Interface

The `DiagnosticsEngine` trait generates diagnostics for code issues:

```rust
use ricecoder_lsp::diagnostics::{DiagnosticsEngine, DefaultDiagnosticsEngine};
use ricecoder_lsp::types::Language;

// Create a diagnostics engine
let engine = DefaultDiagnosticsEngine::new();

// Generate diagnostics
let diagnostics = engine.generate_diagnostics(code, Language::Rust)?;

// Generate diagnostics for a specific range
let range_diagnostics = engine.generate_diagnostics_for_range(
    code,
    Language::Rust,
    range,
)?;
```

**Key Methods**:

- `generate_diagnostics(code: &str, language: Language)`: Generate all diagnostics
- `generate_diagnostics_for_range(code: &str, language: Language, range: Range)`: Generate diagnostics for a specific range

**Diagnostic Severity Levels**:

- `Error`: Critical issues that prevent compilation
- `Warning`: Potential issues that should be addressed
- `Hint`: Style suggestions and improvements

### Code Actions Engine Interface

The `CodeActionsEngine` trait suggests fixes for identified issues:

```rust
use ricecoder_lsp::code_actions::{CodeActionsEngine, DefaultCodeActionsEngine};

// Create a code actions engine
let engine = DefaultCodeActionsEngine::new();

// Get code actions for a diagnostic
let actions = engine.code_actions_for_diagnostic(diagnostic)?;

// Apply a code action
let fixed_code = engine.apply_code_action(code, action)?;
```

**Key Methods**:

- `code_actions_for_diagnostic(diagnostic: &Diagnostic)`: Get applicable code actions
- `apply_code_action(code: &str, action: &CodeAction)`: Apply a code action to code

### Hover Provider Interface

The `HoverProvider` trait provides hover information:

```rust
use ricecoder_lsp::hover::HoverProvider;
use ricecoder_lsp::types::Position;

// Create a hover provider
let provider = HoverProvider::new();

// Get hover information
let hover = provider.hover_at(code, position)?;
```

**Key Methods**:

- `hover_at(code: &str, position: Position)`: Get hover information at a position

## Usage Examples

### Example 1: Basic Semantic Analysis

```rust
use ricecoder_lsp::semantic::{SemanticAnalyzer, RustAnalyzer};

let code = r#"
fn hello(name: &str) {
    println!("Hello, {}", name);
}
"#;

let analyzer = RustAnalyzer::new();
let semantic_info = analyzer.analyze(code)?;

println!("Symbols: {:?}", semantic_info.symbols);
println!("Imports: {:?}", semantic_info.imports);
```

### Example 2: Generating Diagnostics

```rust
use ricecoder_lsp::diagnostics::DefaultDiagnosticsEngine;
use ricecoder_lsp::types::Language;

let code = r#"
fn unused_function() {
    let unused_var = 42;
}
"#;

let engine = DefaultDiagnosticsEngine::new();
let diagnostics = engine.generate_diagnostics(code, Language::Rust)?;

for diagnostic in diagnostics {
    println!("{}: {}", diagnostic.severity, diagnostic.message);
}
```

### Example 3: Getting Hover Information

```rust
use ricecoder_lsp::hover::HoverProvider;
use ricecoder_lsp::types::Position;

let code = r#"
let x: i32 = 42;
"#;

let provider = HoverProvider::new();
let hover = provider.hover_at(code, Position { line: 0, character: 4 })?;

if let Some(info) = hover {
    println!("Type: {}", info.contents);
}
```

### Example 4: Applying Code Actions

```rust
use ricecoder_lsp::code_actions::DefaultCodeActionsEngine;
use ricecoder_lsp::diagnostics::DefaultDiagnosticsEngine;
use ricecoder_lsp::types::Language;

let code = r#"
use std::collections::HashMap;

fn main() {
    println!("Hello");
}
"#;

let diagnostics_engine = DefaultDiagnosticsEngine::new();
let diagnostics = diagnostics_engine.generate_diagnostics(code, Language::Rust)?;

let actions_engine = DefaultCodeActionsEngine::new();
for diagnostic in diagnostics {
    let actions = actions_engine.code_actions_for_diagnostic(&diagnostic)?;
    for action in actions {
        let fixed = actions_engine.apply_code_action(code, &action)?;
        println!("Fixed code:\n{}", fixed);
    }
}
```

## Configuration

The LSP server can be configured via environment variables:

- `RICECODER_LSP_LOG_LEVEL`: Set logging level (trace, debug, info, warn, error)
- `RICECODER_LSP_CACHE_SIZE`: Set cache size in MB (default: 100)
- `RICECODER_LSP_TIMEOUT_MS`: Set analysis timeout in milliseconds (default: 5000)

## Performance

The LSP server is optimized for performance:

- **Caching**: Parsed ASTs and symbol indexes are cached for unchanged documents
- **Incremental Analysis**: Only re-analyze changed portions of code
- **Performance Targets**:
  - < 500ms for files < 10KB
  - < 2s for files < 100KB
  - < 100ms for cached results

## Error Handling

All operations return explicit error types:

```rust
use ricecoder_lsp::semantic::SemanticError;

match analyzer.analyze(code) {
    Ok(info) => println!("Analysis successful"),
    Err(SemanticError::ParseError(msg)) => eprintln!("Parse error: {}", msg),
    Err(SemanticError::UnsupportedLanguage(lang)) => eprintln!("Unsupported: {:?}", lang),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

The crate includes comprehensive tests (97+ tests):

- **Unit Tests**: Test individual components (analyzers, engines, providers)
- **Integration Tests**: Test end-to-end LSP workflows in `tests/`
- **Property Tests**: Verify correctness properties across all inputs

### Test Organization

| Location | Purpose |
|----------|---------|
| `tests/lsp_*_properties.rs` | Property-based tests for correctness |
| `tests/lsp_*_integration.rs` | End-to-end integration tests |
| `src/**/mod.rs` (inline) | Unit tests for internal APIs |

**Note**: Inline `#[cfg(test)]` modules are intentionally kept for unit tests that require access to private internals. Integration and property tests reside in `tests/`.

Run tests with:

```bash
cargo test -p ricecoder-lsp           # All tests
cargo test --lib                       # Unit tests only
cargo test --test '*properties*'       # Property tests only
```

## Troubleshooting

### Issue: Analysis is slow

**Solution**: Check cache hit rates and increase cache size if needed.

```bash
RICECODER_LSP_CACHE_SIZE=200 cargo run
```

### Issue: Unsupported language errors

**Solution**: The crate gracefully degrades for unsupported languages. Check logs for details.

```bash
RICECODER_LSP_LOG_LEVEL=debug cargo run
```

### Issue: Diagnostics are missing

**Solution**: Ensure the language is correctly detected. Check language-specific rules.

```rust
let language = Language::from_extension(path);
println!("Detected language: {:?}", language);
```

## Contributing

When adding new features:

1. Add language-specific analyzers in `src/semantic/`
2. Add diagnostic rules in `src/diagnostics/`
3. Add code actions in `src/code_actions/`
4. Add tests in `tests/`
5. Update this README with examples

## Related Documentation

- **Requirements**: `.ai/specs/ricecoder-lsp/requirements.md`
- **Design**: `.ai/specs/ricecoder-lsp/design.md`
- **Tasks**: `.ai/specs/ricecoder-lsp/tasks.md`
- **LSP Specification**: https://microsoft.github.io/language-server-protocol/

## License

Part of the RiceCoder project. See LICENSE for details.
