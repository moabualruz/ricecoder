# ricecoder-tui

Terminal User Interface for RiceCoder - Pure UI Layer

## Overview

`ricecoder-tui` provides a beautiful, responsive terminal user interface built with [ratatui](https://github.com/ratatui-org/ratatui). This crate is specifically designed to be **independent of business logic**, focusing solely on UI rendering, user interaction, and terminal management.

## Architecture

After the TUI isolation refactoring, `ricecoder-tui` has been separated from business logic concerns:

### ✅ What ricecoder-tui DOES provide:
- Terminal UI widgets and components
- Layout management and responsive design
- User input handling and keybindings
- Theme rendering and styling
- Screen reader accessibility
- Clipboard operations
- File picker interface
- Help system display
- Status bar and information display

### ❌ What ricecoder-tui does NOT provide:
- Session management (moved to `ricecoder-sessions`)
- AI provider integration (moved to `ricecoder-providers`)
- LSP functionality (moved to `ricecoder-lsp`)
- VCS operations (moved to `ricecoder-vcs`)
- File operations (moved to `ricecoder-files`)

## Dependencies

`ricecoder-tui` only depends on infrastructure crates:

```toml
[dependencies]
# Core TUI libraries
ratatui = "0.29"
crossterm = "0.27"
tokio = { version = "1.0", features = ["full"] }

# Infrastructure (no business logic)
ricecoder-storage = { path = "../ricecoder-storage", version = "0.1" }
ricecoder-images = { path = "../ricecoder-images", version = "0.1" }
ricecoder-files = { path = "../ricecoder-files", version = "0.1" }
ricecoder-help = { path = "../ricecoder-help", version = "0.1" }
ricecoder-keybinds = { path = "../ricecoder-keybinds", version = "0.1" }
```

## Usage

```rust
use ricecoder_tui::{App, Theme, LayoutConfig};

// Create TUI application
let mut app = App::new(config);

// Business logic is injected via interfaces
// (session management, providers, etc. are handled externally)
```

## Integration

`ricecoder-tui` is designed to be integrated with business logic through dependency injection or interfaces. The main application (typically `ricecoder-cli`) is responsible for:

1. Creating TUI components
2. Injecting business logic handlers
3. Managing the event loop
4. Coordinating between UI and business logic

## Features

- **Responsive Layout**: Adapts to terminal size changes
- **Theme Support**: 30+ built-in themes with custom theme support
- **Accessibility**: Screen reader support and keyboard navigation
- **Performance**: 60 FPS rendering with efficient updates
- **Cross-platform**: Works on Linux, macOS, Windows, and over SSH

## Module Structure

- `app.rs` - Main application logic
- `components/` - Reusable UI components
- `layout.rs` - Layout management
- `theme.rs` - Theme system
- `input.rs` - Input handling
- `widgets/` - Custom widgets
- `accessibility.rs` - Accessibility features

## Contributing

When adding new features to `ricecoder-tui`:

1. **Keep it UI-only**: Don't add business logic
2. **Use interfaces**: Accept data through parameters/interfaces
3. **Test thoroughly**: Ensure accessibility and cross-platform compatibility
4. **Document clearly**: Update this README for any new modules

## License

MIT
