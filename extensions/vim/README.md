# RiceCoder Vim/Neovim Plugin

Provides IDE integration with RiceCoder for Vim and Neovim, enabling code completion, diagnostics, hover information, and go-to-definition functionality.

## Features

- **Code Completion**: Omnifunc-based completion with RiceCoder suggestions
- **Diagnostics**: Real-time error and warning display
- **Hover Information**: Show symbol information on hover
- **Go to Definition**: Jump to symbol definitions
- **LSP Integration**: Queries external LSP servers for semantic intelligence
- **Graceful Fallback**: Falls back to configured rules or built-in providers when LSP unavailable

## Installation

### Using vim-plug

```vim
Plug 'ricecoder/vim-ricecoder', { 'rtp': 'extensions/vim' }
```

### Using Vundle

```vim
Plugin 'ricecoder/vim-ricecoder'
```

### Manual Installation

Copy the plugin files to your vim configuration directory:

```bash
cp -r extensions/vim/* ~/.vim/
```

## Configuration

Add to your `.vimrc` or `init.vim`:

```vim
" Enable RiceCoder
let g:ricecoder_enabled = 1

" Set RiceCoder host and port
let g:ricecoder_host = 'localhost'
let g:ricecoder_port = 9000

" Set request timeout (milliseconds)
let g:ricecoder_timeout = 5000

" Enable debug logging
let g:ricecoder_debug = 0
```

## Usage

### Code Completion

Use Vim's omnifunc completion:

```vim
" In insert mode, press Ctrl+X Ctrl+O to trigger completion
```

### Hover Information

```vim
" Press K to show hover information at cursor
```

### Go to Definition

```vim
" Press Ctrl+] to go to definition
" Press Ctrl+W ] to go to definition in split
" Press Ctrl+W Ctrl+] to go to definition in vertical split
```

### Show Diagnostics

```vim
" Press Ctrl+E to show diagnostics at cursor
```

## Keybindings

| Key | Command | Description |
|-----|---------|-------------|
| `Ctrl+X Ctrl+O` | Completion | Trigger code completion |
| `K` | Hover | Show hover information |
| `Ctrl+]` | Definition | Go to definition |
| `Ctrl+W ]` | Definition Split | Go to definition in split |
| `Ctrl+W Ctrl+]` | Definition VSplit | Go to definition in vertical split |
| `Ctrl+E` | Diagnostics | Show diagnostics at cursor |

## Customization

### Custom Keybindings

Add to your `.vimrc`:

```vim
" Custom keybinding for completion
inoremap <buffer> <C-l> <C-x><C-o>

" Custom keybinding for hover
nnoremap <buffer> <leader>h :call ricecoder#hover#show_on_demand()<CR>

" Custom keybinding for go to definition
nnoremap <buffer> <leader>d :call ricecoder#goto_definition()<CR>
```

### Custom Configuration

Create `~/.ricecoder/vim-config.yaml`:

```yaml
# Vim/Neovim specific configuration
vim:
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
    show_on_move: true
```

## Troubleshooting

### Connection Issues

If you see "RPC Connection Error", ensure:

1. RiceCoder backend is running on the configured host and port
2. Firewall allows connections to the RiceCoder port
3. Check the host and port configuration in your `.vimrc`

### Completion Not Working

1. Ensure omnifunc is enabled: `:set omnifunc=ricecoder#complete`
2. Check that RiceCoder is running
3. Verify the file type is recognized: `:set filetype?`

### Diagnostics Not Showing

1. Check that diagnostics are enabled: `let g:ricecoder_enabled = 1`
2. Verify the file type is supported
3. Check RiceCoder logs for errors

## Requirements

- Vim 8.0+ or Neovim 0.4+
- RiceCoder backend running and accessible
- curl (for HTTP requests)
- JSON support in Vim/Neovim

## Supported Languages

- Rust (via rust-analyzer)
- TypeScript/JavaScript (via typescript-language-server)
- Python (via pylsp)
- And any language with configured LSP server

## Architecture

The plugin communicates with RiceCoder via JSON-RPC over HTTP:

```
Vim/Neovim Plugin
    ↓ (JSON-RPC HTTP)
RiceCoder Backend
    ↓ (Queries)
External LSP Servers (rust-analyzer, tsserver, pylsp, etc.)
```

## Contributing

Contributions are welcome! Please ensure:

1. Code follows Vim script conventions
2. All functions are properly documented
3. Error handling is comprehensive
4. Changes are tested in both Vim and Neovim

## License

See LICENSE.md in the ricecoder project root.

## Support

For issues and feature requests, visit:
- GitHub Issues: https://github.com/ricecoder/ricecoder/issues
- GitHub Discussions: https://github.com/ricecoder/ricecoder/discussions
