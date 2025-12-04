# Configuration-Driven Architecture for LSP Integration

## Overview

The ricecoder LSP integration uses a configuration-driven architecture that enables language-agnostic semantic analysis, diagnostics, and code actions. This design allows adding support for new languages through configuration files without modifying code.

## Architecture Principles

### 1. Language-Agnostic Core

The core LSP server is language-agnostic and delegates language-specific behavior to pluggable providers:

- **Semantic Analysis**: `SemanticAnalyzerProvider` trait for language-specific parsing and symbol extraction
- **Diagnostics**: `DiagnosticsProvider` trait for language-specific diagnostic rules
- **Code Actions**: `CodeActionProvider` trait for language-specific transformations

### 2. Configuration-Driven Behavior

Language-specific behavior is defined in configuration files (YAML/JSON) rather than hardcoded:

- **Language Configuration**: Defines parser plugins, diagnostic rules, and code actions
- **Diagnostic Rules**: Pattern-based rules for generating diagnostics
- **Code Action Templates**: Transformation templates for fixing issues

### 3. Graceful Degradation

The system provides basic functionality for unconfigured languages:

- **Fallback Analysis**: Generic text-based analysis for unknown languages
- **Empty Diagnostics**: No diagnostics for unconfigured languages
- **No Errors**: System continues functioning without crashing

### 4. Hot-Reload Support

Configurations can be reloaded at runtime without restarting the LSP server:

- **Configuration Registry**: Manages language configurations
- **Provider Registries**: Manage semantic, diagnostics, and code action providers
- **Runtime Updates**: Configurations can be updated without server restart

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    LSP Client (IDE)                         │
└────────────────────────┬────────────────────────────────────┘
                         │ LSP Protocol (JSON-RPC)
                         │
┌────────────────────────▼────────────────────────────────────┐
│         Configuration Manager                               │
│  - Load language configurations from files                 │
│  - Manage provider registries                              │
│  - Support hot-reload of configurations                    │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│              LSP Server (Language-Agnostic)                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              LSP Protocol Handler                     │  │
│  │  - Initialize/Shutdown                               │  │
│  │  - Document Synchronization                          │  │
│  │  - Request Routing                                   │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │    Generic Semantic Analyzer (with Providers)         │  │
│  │  - Delegates to language-specific providers          │  │
│  │  - Falls back to generic analysis                    │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Generic Diagnostics Engine (with Providers)          │  │
│  │  - Applies configured diagnostic rules               │  │
│  │  - Falls back to empty diagnostics                   │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Generic Code Actions Engine (with Providers)         │  │
│  │  - Applies configured transformations                │  │
│  │  - Falls back to no actions                          │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### Configuration Registry

Manages language configurations loaded from files:

```rust
pub struct ConfigRegistry {
    languages: HashMap<String, LanguageConfig>,
}

impl ConfigRegistry {
    pub fn register(&mut self, config: LanguageConfig) -> ConfigResult<()>;
    pub fn get(&self, language: &str) -> Option<&LanguageConfig>;
    pub fn get_by_extension(&self, extension: &str) -> Option<&LanguageConfig>;
    pub fn has_language(&self, language: &str) -> bool;
    pub fn languages(&self) -> Vec<&str>;
}
```

### Provider Registries

Manage pluggable providers for each component:

```rust
pub struct SemanticAnalyzerRegistry {
    providers: HashMap<String, Box<dyn SemanticAnalyzerProvider>>,
}

pub struct DiagnosticsRegistry {
    providers: HashMap<String, Box<dyn DiagnosticsProvider>>,
}

pub struct CodeActionRegistry {
    providers: HashMap<String, Box<dyn CodeActionProvider>>,
}
```

### Generic Engines

Language-agnostic engines that delegate to providers:

```rust
pub struct GenericSemanticAnalyzer {
    registry: SemanticAnalyzerRegistry,
    fallback: FallbackAnalyzer,
}

pub struct GenericDiagnosticsEngine {
    registry: DiagnosticsRegistry,
}

pub struct GenericCodeActionsEngine {
    registry: CodeActionRegistry,
}
```

### Configuration Manager

Orchestrates loading and managing configurations:

```rust
pub struct ConfigurationManager {
    config_registry: Arc<RwLock<ConfigRegistry>>,
    semantic_registry: Arc<RwLock<SemanticAnalyzerRegistry>>,
    diagnostics_registry: Arc<RwLock<DiagnosticsRegistry>>,
    code_action_registry: Arc<RwLock<CodeActionRegistry>>,
}

impl ConfigurationManager {
    pub fn load_defaults(&self) -> ConfigResult<()>;
    pub fn load_from_directory(&self, path: &Path) -> ConfigResult<()>;
    pub fn load_config_file(&self, path: &Path) -> ConfigResult<()>;
}
```

## Configuration Format

Language configurations are defined in YAML format:

```yaml
language: rust
extensions:
  - rs

parser_plugin: tree-sitter-rust

diagnostic_rules:
  - name: unused-import
    pattern: 'use\s+\w+;'
    severity: warning
    message: "Unused import"
    code: unused-import
    fix_template: "Remove import"

code_actions:
  - name: remove-unused-import
    title: "Remove unused import"
    kind: quickfix
    transformation: "delete_line"
```

## Adding New Languages

To add support for a new language:

1. **Create Configuration File**: `config/{language}.yaml`
   - Define language identifier, extensions, parser plugin
   - Add diagnostic rules for common issues
   - Add code action templates for fixes

2. **Validate Configuration**: Ensure it matches the schema in `config/schema.json`

3. **Load Configuration**: Use `ConfigurationManager::load_config_file()` or `load_from_directory()`

4. **Test**: Verify diagnostics and code actions work correctly

Example for Go:

```yaml
language: go
extensions:
  - go

parser_plugin: tree-sitter-go

diagnostic_rules:
  - name: unused-import
    pattern: 'import\s+"[^"]+"'
    severity: warning
    message: "Unused import"
    code: unused-import

code_actions:
  - name: remove-unused-import
    title: "Remove unused import"
    kind: quickfix
    transformation: "delete_line"
```

## Provider Traits

### SemanticAnalyzerProvider

Provides language-specific semantic analysis:

```rust
pub trait SemanticAnalyzerProvider: Send + Sync {
    fn language(&self) -> &str;
    fn analyze(&self, code: &str) -> ProviderResult<SemanticInfo>;
    fn extract_symbols(&self, code: &str) -> ProviderResult<Vec<Symbol>>;
    fn get_hover_info(&self, code: &str, position: Position) -> ProviderResult<Option<String>>;
}
```

### DiagnosticsProvider

Provides language-specific diagnostic rules:

```rust
pub trait DiagnosticsProvider: Send + Sync {
    fn language(&self) -> &str;
    fn generate_diagnostics(&self, code: &str) -> ProviderResult<Vec<Diagnostic>>;
    fn config(&self) -> Option<&LanguageConfig>;
}
```

### CodeActionProvider

Provides language-specific code actions:

```rust
pub trait CodeActionProvider: Send + Sync {
    fn language(&self) -> &str;
    fn suggest_actions(&self, diagnostic: &Diagnostic, code: &str) -> ProviderResult<Vec<String>>;
    fn apply_action(&self, code: &str, action: &str) -> ProviderResult<String>;
    fn config(&self) -> Option<&LanguageConfig>;
}
```

## Adapters

Adapters wrap existing language-specific implementations to implement provider traits:

```rust
pub struct RustAnalyzerAdapter {
    analyzer: RustAnalyzer,
}

impl SemanticAnalyzerProvider for RustAnalyzerAdapter {
    fn language(&self) -> &str { "rust" }
    fn analyze(&self, code: &str) -> ProviderResult<SemanticInfo> {
        self.analyzer.analyze(code)
            .map_err(|e| ProviderError::Error(e.to_string()))
    }
    // ... other methods
}
```

## Usage Example

### Loading Configurations

```rust
use ricecoder_lsp::ConfigurationManager;
use std::path::Path;

let manager = ConfigurationManager::new();

// Load default providers
manager.load_defaults()?;

// Load configurations from directory
manager.load_from_directory(Path::new("config"))?;

// Or load a single configuration
manager.load_config_file(Path::new("config/rust.yaml"))?;
```

### Using Generic Engines

```rust
use ricecoder_lsp::semantic::GenericSemanticAnalyzer;

let mut analyzer = GenericSemanticAnalyzer::new();

// Register providers
analyzer.register_provider(Box::new(RustAnalyzerAdapter::new()));
analyzer.register_provider(Box::new(TypeScriptAnalyzerAdapter::new()));

// Analyze code
let info = analyzer.analyze("fn main() {}", "rust")?;

// Fallback for unknown language
let info = analyzer.analyze("unknown code", "unknown")?;
```

## Hot-Reload

Configurations can be reloaded at runtime:

```rust
let manager = ConfigurationManager::new();
manager.load_defaults()?;

// Initial configuration
manager.load_config_file(Path::new("config/rust.yaml"))?;

// Later, reload configuration
manager.load_config_file(Path::new("config/rust.yaml"))?;
```

## Testing

### Property-Based Tests

Property tests verify configuration-driven behavior:

- Configured languages work correctly
- Unconfigured languages degrade gracefully
- Configuration changes reload without restart
- Invalid configurations are rejected
- Multiple languages can be configured simultaneously

### Integration Tests

Integration tests verify end-to-end functionality:

- LSP server with multiple language configurations
- Configuration loading and validation
- Hot-reload of language configurations
- Fallback behavior for unconfigured languages
- Generic engines with providers

## Best Practices

1. **Keep Patterns Simple**: Use simple regex patterns that are easy to understand
2. **Provide Clear Messages**: Diagnostic messages should be actionable and specific
3. **Test Configurations**: Validate configurations against the schema before deploying
4. **Document Custom Rules**: Add comments explaining non-obvious rules
5. **Version Configurations**: Track configuration changes in version control
6. **Organize by Language**: Keep each language's configuration in a separate file

## Troubleshooting

### Configuration not loading

- Check that the file exists and is readable
- Validate the YAML syntax
- Verify the configuration against `schema.json`
- Check logs for error messages

### Diagnostics not appearing

- Verify the language is configured
- Check that diagnostic rules have valid patterns
- Ensure the file extension matches the configuration
- Check that the parser plugin is available

### Code actions not working

- Verify the code action is registered
- Check that the transformation template is valid
- Ensure the diagnostic code matches the action name
- Check logs for error messages

## References

- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [Tree-sitter](https://tree-sitter.github.io/)
- [JSON Schema](https://json-schema.org/)
- [Configuration Files](./config/README.md)
