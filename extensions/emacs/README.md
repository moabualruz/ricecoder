# RiceCoder Emacs Integration

Provides IDE integration with RiceCoder for Emacs, enabling code completion, diagnostics, hover information, and go-to-definition functionality.

## Features

- **Code Completion**: Integration with Emacs completion-at-point
- **Diagnostics**: Real-time error and warning display
- **Hover Information**: Show symbol information on demand
- **Go to Definition**: Jump to symbol definitions
- **LSP Integration**: Queries external LSP servers for semantic intelligence
- **Graceful Fallback**: Falls back to configured rules or built-in providers when LSP unavailable

## Installation

### Using use-package

```elisp
(use-package ricecoder
  :load-path "extensions/emacs"
  :config
  (global-ricecoder-mode 1))
```

### Using straight.el

```elisp
(straight-use-package
 '(ricecoder :type git :host github :repo "ricecoder/ricecoder"
             :files ("extensions/emacs/*.el")))
(global-ricecoder-mode 1)
```

### Manual Installation

Copy the ricecoder.el file to your Emacs load path:

```bash
cp extensions/emacs/ricecoder.el ~/.emacs.d/lisp/
```

Then add to your init.el:

```elisp
(add-to-list 'load-path "~/.emacs.d/lisp")
(require 'ricecoder)
(global-ricecoder-mode 1)
```

## Configuration

Add to your init.el:

```elisp
(use-package ricecoder
  :load-path "extensions/emacs"
  :custom
  (ricecoder-host "localhost")
  (ricecoder-port 9000)
  (ricecoder-timeout 5000)
  (ricecoder-enabled t)
  (ricecoder-debug nil)
  :config
  (global-ricecoder-mode 1))
```

## Usage

### Code Completion

Use Emacs' standard completion:

```
M-x completion-at-point
```

Or with company-mode:

```elisp
(use-package company
  :config
  (global-company-mode 1))
```

### Hover Information

```
C-c C-h
```

Shows hover information for the symbol at point.

### Go to Definition

```
C-c C-d
```

Jump to the definition of the symbol at point.

### Go to Definition in Other Window

```
C-c C-o
```

Jump to the definition in another window.

## Keybindings

| Key | Command | Description |
|-----|---------|-------------|
| `M-x completion-at-point` | Completion | Trigger code completion |
| `C-c C-h` | Hover | Show hover information |
| `C-c C-d` | Definition | Go to definition |
| `C-c C-o` | Definition Other | Go to definition in other window |

## Customization

### Custom Keybindings

Add to your init.el:

```elisp
(define-key ricecoder-mode-map (kbd "C-c d") 'ricecoder-goto-definition)
(define-key ricecoder-mode-map (kbd "C-c h") 'ricecoder-show-hover)
```

### Custom Configuration

Create `~/.ricecoder/emacs-config.yaml`:

```yaml
# Emacs specific configuration
emacs:
  enabled: true
  host: localhost
  port: 9000
  timeout_ms: 5000
  
  # Completion settings
  completion:
    enabled: true
    max_items: 20
    
  # Diagnostics settings
  diagnostics:
    enabled: true
    show_on_change: true
    
  # Hover settings
  hover:
    enabled: true
```

## Troubleshooting

### Connection Issues

If you see "RiceCoder Error: RPC Connection Error", ensure:

1. RiceCoder backend is running on the configured host and port
2. Firewall allows connections to the RiceCoder port
3. Check the host and port configuration in your init.el

### Completion Not Working

1. Ensure ricecoder-mode is enabled: `M-x ricecoder-mode`
2. Check that RiceCoder is running
3. Verify the major mode is supported

### Diagnostics Not Showing

1. Check that ricecoder-mode is enabled
2. Verify the file type is supported
3. Check RiceCoder logs for errors

## Requirements

- Emacs 26.1+
- RiceCoder backend running and accessible
- request.el (for HTTP requests)
- json.el (for JSON parsing)

## Supported Languages

- Rust (via rust-analyzer)
- TypeScript/JavaScript (via typescript-language-server)
- Python (via pylsp)
- C/C++ (via clangd)
- Java (via eclipse-jdt-ls)
- Go (via gopls)
- Ruby (via solargraph)
- PHP (via intelephense)
- And any language with configured LSP server

## Architecture

The integration communicates with RiceCoder via JSON-RPC over HTTP:

```
Emacs Integration
    ↓ (JSON-RPC HTTP)
RiceCoder Backend
    ↓ (Queries)
External LSP Servers (rust-analyzer, tsserver, pylsp, etc.)
```

## Integration with Other Packages

### company-mode

```elisp
(use-package company
  :config
  (global-company-mode 1)
  (add-to-list 'company-backends 'company-capf))

(use-package ricecoder
  :load-path "extensions/emacs"
  :config
  (global-ricecoder-mode 1))
```

### flycheck

```elisp
(use-package flycheck
  :config
  (global-flycheck-mode 1))

(use-package ricecoder
  :load-path "extensions/emacs"
  :config
  (global-ricecoder-mode 1))
```

### lsp-mode

```elisp
(use-package lsp-mode
  :config
  (lsp-mode 1))

(use-package ricecoder
  :load-path "extensions/emacs"
  :config
  (global-ricecoder-mode 1))
```

## Contributing

Contributions are welcome! Please ensure:

1. Code follows Emacs Lisp conventions
2. All functions are properly documented
3. Error handling is comprehensive
4. Changes are tested in Emacs

## License

See LICENSE.md in the ricecoder project root.

## Support

For issues and feature requests, visit:
- GitHub Issues: https://github.com/ricecoder/ricecoder/issues
- GitHub Discussions: https://github.com/ricecoder/ricecoder/discussions
