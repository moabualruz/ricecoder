# Language Configuration

This directory contains language-specific configurations for the ricecoder LSP integration.

## Overview

Each language configuration file defines:
- **Language identifier**: Unique identifier for the language (e.g., "rust", "typescript", "python")
- **File extensions**: List of file extensions for this language
- **Parser plugin**: Reference to the tree-sitter parser plugin
- **Diagnostic rules**: Rules for generating diagnostics (errors, warnings, hints)
- **Code actions**: Transformations for fixing issues and refactoring

## Configuration Format

Configurations are defined in YAML format for readability. Each configuration file must follow the schema defined in `schema.json`.

### Example Structure

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

## Configuration Files

### rust.yaml
Configuration for Rust language support.
- **Extensions**: `.rs`
- **Parser**: tree-sitter-rust
- **Diagnostic Rules**: unused imports, missing semicolons, naming conventions
- **Code Actions**: remove imports, add semicolons, convert naming

### typescript.yaml
Configuration for TypeScript/JavaScript language support.
- **Extensions**: `.ts`, `.tsx`, `.js`, `.jsx`
- **Parser**: tree-sitter-typescript
- **Diagnostic Rules**: unused variables, missing return types, naming conventions
- **Code Actions**: remove variables, add return types, convert naming

### python.yaml
Configuration for Python language support.
- **Extensions**: `.py`
- **Parser**: tree-sitter-python
- **Diagnostic Rules**: unused imports, missing docstrings, naming conventions
- **Code Actions**: remove imports, add docstrings, convert naming

## Schema

The configuration schema is defined in `schema.json` and validates:
- Required fields: `language`, `extensions`
- Optional fields: `parser_plugin`, `diagnostic_rules`, `code_actions`
- Diagnostic rule fields: `name`, `pattern`, `severity`, `message`, `code`, `fix_template`
- Code action fields: `name`, `title`, `kind`, `transformation`

### Severity Levels

- **error**: Critical issues that prevent code execution
- **warning**: Issues that may cause problems but don't prevent execution
- **info**: Suggestions and informational messages

### Action Kinds

- **quickfix**: Quick fixes for specific issues
- **refactor**: Refactoring suggestions
- **source**: Source code generation or organization

## Adding New Languages

To add support for a new language:

1. Create a new YAML configuration file (e.g., `go.yaml`)
2. Define the language identifier, extensions, and parser plugin
3. Add diagnostic rules for common issues in that language
4. Add code actions for fixing those issues
5. Validate the configuration against `schema.json`
6. Place the file in this directory

Example:

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

## Configuration Loading

Configurations are loaded at runtime using the `ConfigLoader`:

```rust
use ricecoder_lsp::ConfigLoader;
use std::path::Path;

// Load a single configuration
let config = ConfigLoader::load(Path::new("config/rust.yaml"))?;

// Load all configurations from a directory
let registry = ConfigLoader::load_directory(Path::new("config"))?;
```

## Validation

All configurations are validated when loaded:
- Language name must not be empty
- Extensions list must not be empty
- Diagnostic rules must have valid severity levels
- Code actions must have valid kinds

Invalid configurations will return a `ConfigError` with a descriptive message.

## Hot-Reload

Configurations can be reloaded at runtime without restarting the LSP server:

```rust
let mut registry = ConfigRegistry::new();

// Load initial configurations
let config = ConfigLoader::load(Path::new("config/rust.yaml"))?;
registry.register(config)?;

// Later, reload configuration
let updated_config = ConfigLoader::load(Path::new("config/rust.yaml"))?;
registry.register(updated_config)?;  // Updates existing configuration
```

## Best Practices

1. **Keep patterns simple**: Use simple regex patterns that are easy to understand and maintain
2. **Provide clear messages**: Diagnostic messages should be actionable and specific
3. **Test configurations**: Validate configurations against the schema before deploying
4. **Document custom rules**: Add comments explaining non-obvious rules
5. **Version configurations**: Track configuration changes in version control
6. **Organize by language**: Keep each language's configuration in a separate file

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
