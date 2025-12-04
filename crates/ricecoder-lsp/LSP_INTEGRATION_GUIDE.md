# LSP Integration Guide

This guide explains how to use RiceCoder's Language Server Protocol (LSP) integration with your IDE.

## Table of Contents

1. [Installation](#installation)
2. [Configuration](#configuration)
3. [IDE Integration](#ide-integration)
4. [Features](#features)
5. [Troubleshooting](#troubleshooting)

## Installation

### Prerequisites

- RiceCoder installed and in your PATH
- A compatible IDE or editor (VS Code, Neovim, Emacs, etc.)
- Rust, TypeScript, or Python project (or any supported language)

### Starting the LSP Server

The LSP server is started via the RiceCoder CLI:

```bash
ricecoder lsp start
```

This starts the server in stdio mode, ready to accept LSP client connections.

### Configuration

The LSP server can be configured via environment variables:

```bash
# Set logging level (trace, debug, info, warn, error)
export RICECODER_LSP_LOG_LEVEL=debug

# Set cache size in MB (default: 100)
export RICECODER_LSP_CACHE_SIZE=200

# Set analysis timeout in milliseconds (default: 5000)
export RICECODER_LSP_TIMEOUT_MS=10000

# Start the server with configuration
ricecoder lsp start
```

Or via configuration file at `~/.ricecoder/lsp.yaml`:

```yaml
lsp:
  log_level: debug
  cache_size_mb: 200
  timeout_ms: 10000
  
  # Language-specific settings
  languages:
    rust:
      enabled: true
      diagnostics: true
      code_actions: true
    
    typescript:
      enabled: true
      diagnostics: true
      code_actions: true
    
    python:
      enabled: true
      diagnostics: true
      code_actions: true
```

## IDE Integration

### VS Code

#### Installation

1. Install the RiceCoder extension from the VS Code marketplace (or build from source)
2. Configure the extension to use the RiceCoder LSP server

#### Configuration

Add to your `.vscode/settings.json`:

```json
{
  "[rust]": {
    "editor.defaultFormatter": "ricecoder.ricecoder",
    "editor.formatOnSave": true
  },
  "[typescript]": {
    "editor.defaultFormatter": "ricecoder.ricecoder",
    "editor.formatOnSave": true
  },
  "[python]": {
    "editor.defaultFormatter": "ricecoder.ricecoder",
    "editor.formatOnSave": true
  },
  "ricecoder.lsp.enabled": true,
  "ricecoder.lsp.logLevel": "info",
  "ricecoder.lsp.cacheSize": 200
}
```

#### Features

- **Hover Information**: Hover over symbols to see type information and documentation
- **Diagnostics**: See errors, warnings, and hints inline
- **Code Actions**: Use quick fixes (Ctrl+.) to apply suggestions
- **Go to Definition**: Jump to symbol definitions
- **Find References**: Find all uses of a symbol

### Neovim

#### Installation

1. Install the RiceCoder LSP client plugin:

```vim
" Using vim-plug
Plug 'ricecoder/ricecoder-nvim'

" Or using packer.nvim
use 'ricecoder/ricecoder-nvim'
```

2. Configure the LSP client in your `init.lua`:

```lua
require('ricecoder').setup({
  lsp = {
    enabled = true,
    log_level = 'info',
    cache_size = 200,
  }
})
```

#### Features

- **Hover Information**: Use `K` to see hover information
- **Diagnostics**: See errors and warnings in the gutter
- **Code Actions**: Use `<leader>ca` to apply code actions
- **Go to Definition**: Use `gd` to jump to definitions
- **Find References**: Use `gr` to find references

### Emacs

#### Installation

1. Install `lsp-mode` and `lsp-ui`:

```elisp
(use-package lsp-mode
  :ensure t
  :hook (prog-mode . lsp))

(use-package lsp-ui
  :ensure t
  :commands lsp-ui-mode)
```

2. Configure RiceCoder LSP client:

```elisp
(lsp-register-client
  (make-lsp-client
    :new-connection (lsp-stdio-connection '("ricecoder" "lsp" "start"))
    :major-modes '(rust-mode typescript-mode python-mode)
    :server-id 'ricecoder-lsp))
```

#### Features

- **Hover Information**: Use `lsp-ui-doc-show` to see hover information
- **Diagnostics**: See errors and warnings in the buffer
- **Code Actions**: Use `lsp-execute-code-action` to apply fixes
- **Go to Definition**: Use `lsp-find-definition` to jump to definitions

### Sublime Text

#### Installation

1. Install the LSP client package:

```
Package Control: Install Package → LSP
```

2. Configure RiceCoder LSP in your settings:

```json
{
  "clients": {
    "ricecoder": {
      "enabled": true,
      "command": ["ricecoder", "lsp", "start"],
      "languages": [
        {
          "languageId": "rust",
          "scopes": ["source.rust"],
          "syntaxes": ["Packages/Rust/Rust.sublime-syntax"]
        },
        {
          "languageId": "typescript",
          "scopes": ["source.ts"],
          "syntaxes": ["Packages/TypeScript/TypeScript.sublime-syntax"]
        },
        {
          "languageId": "python",
          "scopes": ["source.python"],
          "syntaxes": ["Packages/Python/Python.sublime-syntax"]
        }
      ]
    }
  }
}
```

#### Features

- **Hover Information**: Hover over symbols to see information
- **Diagnostics**: See errors and warnings in the gutter
- **Code Actions**: Use the command palette to apply fixes
- **Go to Definition**: Use `Goto Definition` command

## Features

### Semantic Analysis

The LSP server analyzes code structure and extracts semantic information:

- **Symbols**: Functions, types, variables, classes, interfaces, etc.
- **Imports**: Track dependencies and imports
- **Definitions**: Find where symbols are defined
- **References**: Find all uses of a symbol

### Diagnostics

The server generates diagnostics for code issues:

- **Errors**: Critical issues that prevent compilation
- **Warnings**: Potential issues that should be addressed
- **Hints**: Style suggestions and improvements

**Language-Specific Diagnostics**:

- **Rust**: Unused imports, unused variables, naming conventions
- **TypeScript**: Type errors, unused variables, missing imports
- **Python**: Type errors, unused variables, naming conventions

### Code Actions

The server suggests fixes for identified issues:

- **Fix Unused Imports**: Remove or organize imports
- **Fix Naming**: Rename symbols to follow conventions
- **Extract Function**: Extract code into a new function
- **Inline Variable**: Inline variable definitions

### Hover Information

Hover over symbols to see:

- **Type Information**: The type of the symbol
- **Documentation**: Comments and docstrings
- **Definition Location**: Where the symbol is defined
- **Usage Count**: How many times the symbol is used

## Troubleshooting

### Issue: LSP server doesn't start

**Symptoms**: IDE shows "LSP server not running" or similar error

**Solutions**:

1. Check that RiceCoder is installed:
   ```bash
   ricecoder --version
   ```

2. Check that the LSP command works:
   ```bash
   ricecoder lsp start
   ```

3. Check logs for errors:
   ```bash
   RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start
   ```

4. Verify IDE configuration points to correct command

### Issue: Diagnostics are not showing

**Symptoms**: No errors or warnings appear in the editor

**Solutions**:

1. Check that diagnostics are enabled in configuration:
   ```yaml
   languages:
     rust:
       diagnostics: true
   ```

2. Check that the file language is correctly detected:
   - Rust files should have `.rs` extension
   - TypeScript files should have `.ts` extension
   - Python files should have `.py` extension

3. Check logs for analysis errors:
   ```bash
   RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start
   ```

4. Try analyzing a simple file to verify basic functionality

### Issue: Hover information is not showing

**Symptoms**: Hovering over symbols shows no information

**Solutions**:

1. Check that hover is enabled in configuration:
   ```yaml
   lsp:
     hover_provider: true
   ```

2. Check that the symbol is recognized:
   - Hover over function names, type names, variable names
   - Hover over imported symbols

3. Check logs for hover errors:
   ```bash
   RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start
   ```

### Issue: Code actions are not available

**Symptoms**: Quick fix menu is empty or shows no suggestions

**Solutions**:

1. Check that code actions are enabled:
   ```yaml
   lsp:
     code_action_provider: true
   ```

2. Check that there are diagnostics to fix:
   - Code actions are only available for identified issues
   - Check that diagnostics are showing

3. Check logs for code action errors:
   ```bash
   RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start
   ```

### Issue: Performance is slow

**Symptoms**: Analysis takes a long time, IDE feels sluggish

**Solutions**:

1. Increase cache size:
   ```bash
   export RICECODER_LSP_CACHE_SIZE=500
   ricecoder lsp start
   ```

2. Increase timeout:
   ```bash
   export RICECODER_LSP_TIMEOUT_MS=15000
   ricecoder lsp start
   ```

3. Check file size:
   - Large files (>100KB) may take longer to analyze
   - Consider splitting into smaller files

4. Check logs for performance issues:
   ```bash
   RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start
   ```

### Issue: Unsupported language error

**Symptoms**: "Unsupported language" error for a file

**Solutions**:

1. Check that the language is supported:
   - Rust (.rs files)
   - TypeScript (.ts files)
   - Python (.py files)

2. Check file extension:
   - Ensure file has correct extension
   - Some editors may not detect language correctly

3. For unsupported languages:
   - Basic analysis is still available
   - Check logs for details

### Issue: Memory usage is high

**Symptoms**: LSP server uses a lot of memory

**Solutions**:

1. Reduce cache size:
   ```bash
   export RICECODER_LSP_CACHE_SIZE=50
   ricecoder lsp start
   ```

2. Restart the server periodically:
   - Close and reopen the IDE
   - Or use IDE command to restart LSP server

3. Check for large files:
   - Very large files (>1MB) may use significant memory
   - Consider splitting into smaller files

## Advanced Configuration

### Custom Diagnostic Rules

Create a custom rules file at `~/.ricecoder/lsp-rules.yaml`:

```yaml
diagnostics:
  rust:
    - rule: unused_imports
      enabled: true
      severity: warning
    
    - rule: naming_convention
      enabled: true
      severity: hint
      pattern: "^[a-z_]+$"
  
  typescript:
    - rule: type_errors
      enabled: true
      severity: error
    
    - rule: unused_variables
      enabled: true
      severity: warning
```

### Custom Code Actions

Create a custom actions file at `~/.ricecoder/lsp-actions.yaml`:

```yaml
code_actions:
  rust:
    - action: fix_unused_imports
      enabled: true
      auto_apply: false
    
    - action: fix_naming
      enabled: true
      auto_apply: false
  
  typescript:
    - action: add_missing_imports
      enabled: true
      auto_apply: false
```

## Performance Tips

1. **Use Incremental Sync**: Enable incremental document synchronization for faster updates
2. **Increase Cache Size**: Larger cache improves performance for repeated analysis
3. **Disable Unused Features**: Disable diagnostics or code actions you don't use
4. **Use Smaller Files**: Smaller files analyze faster
5. **Monitor Performance**: Use logs to identify slow operations

## Related Documentation

- **API Documentation**: See `README.md` for API details
- **Requirements**: `.kiro/specs/ricecoder-lsp/requirements.md`
- **Design**: `.kiro/specs/ricecoder-lsp/design.md`
- **LSP Specification**: https://microsoft.github.io/language-server-protocol/

## Support

For issues or questions:

1. Check this guide's troubleshooting section
2. Check the logs with debug logging enabled
3. Open an issue on GitHub with logs and reproduction steps
4. Check the LSP specification for protocol details

## License

Part of the RiceCoder project. See LICENSE for details.
