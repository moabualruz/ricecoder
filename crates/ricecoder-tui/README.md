# ricecoder-tui

**Purpose**: Terminal user interface providing beautiful, responsive UI components and user interaction handling for RiceCoder

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

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-tui = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_tui::{App, Theme, LayoutConfig};

// Create TUI application
let mut app = App::new(config);

// Business logic is injected via interfaces
// (session management, providers, etc. are handled externally)
```

### Advanced Usage

```rust
use ricecoder_tui::{App, ThemeManager, KeybindManager};

// Create with custom theme and keybindings
let theme_manager = ThemeManager::new();
let keybind_manager = KeybindManager::new();

let mut app = App::with_managers(config, theme_manager, keybind_manager);

// Handle application events
while let Some(event) = app.next_event().await {
    match event {
        Event::Key(key) => app.handle_key(key),
        Event::Resize(width, height) => app.handle_resize(width, height),
        Event::Quit => break,
    }
}
```

## Configuration

TUI configuration options:

```yaml
tui:
  theme: "tokyo-night"
  vim_mode: true
  sidebar_width: 30
  max_scrollback: 10000
  accessibility:
    screen_reader: true
    high_contrast: false
  keybindings:
    custom:
      - key: "ctrl+c"
        action: "copy"
```

## API Reference

### Key Types

- **`App`**: Main TUI application structure
- **`ThemeManager`**: Theme loading and management
- **`LayoutConfig`**: Layout configuration and constraints
- **`Component`**: Base trait for UI components
- **`Event`**: TUI events (keyboard, mouse, resize)
- **`Widget`**: Reusable UI widgets

### Key Functions

- **`App::new()`**: Create new TUI application
- **`App::with_managers()`**: Create app with custom managers
- **`ThemeManager::load_theme()`**: Load theme by name
- **`Component::render()`**: Render component to terminal
- **`Component::handle_event()`**: Handle user input events

## Error Handling

```rust
use ricecoder_tui::TuiError;

match result {
    Ok(()) => println!("TUI operation successful"),
    Err(TuiError::ThemeNotFound(name)) => eprintln!("Theme '{}' not found", name),
    Err(TuiError::InvalidLayout(msg)) => eprintln!("Layout error: {}", msg),
    Err(TuiError::TerminalError(msg)) => eprintln!("Terminal error: {}", msg),
    Err(TuiError::AccessibilityError(msg)) => eprintln!("Accessibility error: {}", msg),
}
```

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test -p ricecoder-tui

# Run performance benchmarks
cargo bench -p ricecoder-tui

# Run accessibility tests
cargo test -p ricecoder-tui accessibility

# Test with different terminal sizes
cargo test -p ricecoder-tui layout
```

Key test areas:
- Component rendering and interaction
- Theme loading and application
- Layout responsiveness
- Accessibility compliance
- Cross-platform compatibility
- Performance benchmarks

## Performance

- **Rendering**: 60 FPS target with efficient diff-based updates
- **Memory**: Minimal memory footprint, streaming large content
- **Layout**: Responsive layout calculations, cached when possible
- **Themes**: Fast theme switching with precompiled styles
- **Input**: Low-latency input handling with debouncing

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
5. **Performance**: Maintain 60 FPS rendering target
6. **Accessibility**: Follow WCAG guidelines for screen readers

## DDD Layer

**Layer**: Presentation (User Interface)

### Responsibilities
- Terminal UI rendering and layout management
- User input handling and event processing
- Theme application and visual styling
- Accessibility features (screen reader, keyboard nav)
- Widget composition and component lifecycle
- Cross-platform terminal compatibility

### SOLID Analysis
- **SRP**: Pure UI concerns, no business logic ✅
- **OCP**: Extensible via Component trait and custom widgets ✅
- **LSP**: Component abstraction allows interchangeable widgets ✅
- **ISP**: Separate interfaces for rendering, events, accessibility ✅
- **DIP**: Business logic injected via interfaces, not embedded ✅

### Integration Points
| Dependency | Direction | Purpose |
|------------|-----------|---------|
| ricecoder-storage | Inbound | Theme and layout persistence |
| ricecoder-images | Inbound | Image display in terminal |
| ricecoder-files | Inbound | File picker integration |
| ricecoder-help | Inbound | Help dialog widget |
| ricecoder-keybinds | Inbound | Keybind handling |
| ricecoder-themes | Inbound | Theme management |
| ricecoder-cli | Outbound | Provides TUI to main application |
| ratatui | External | Terminal rendering framework |
| crossterm | External | Cross-platform terminal I/O |

## License

MIT
