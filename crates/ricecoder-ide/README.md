# ricecoder-ide

**Purpose**: IDE integration providing support for VS Code, Vim, Neovim, Emacs, and other editors with RiceCoder functionality

## Features

- **VS Code Integration**: Native VS Code extension with RiceCoder commands and AI assistance
- **Vim/Neovim Support**: Vim plugin with keybindings and integration for modal editing workflows
- **JetBrains IDEs**: IntelliJ, WebStorm, and other JetBrains IDE integration
- **Emacs Integration**: Emacs package with RiceCoder functionality and customization
- **External LSP Architecture**: LSP-first approach ensuring consistent behavior across all editors

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-ide = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_ide::{IdeIntegration, VsCodeExtension};

// Initialize IDE integration
let integration = IdeIntegration::new();

// Register VS Code extension
let vscode = VsCodeExtension::new();
integration.register_extension(vscode)?;

// Handle IDE commands
integration.handle_command("rice:chat", args).await?;
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-ide).

## License

MIT
