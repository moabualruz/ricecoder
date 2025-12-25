# ricecoder-keybinds

**Purpose**: Keybind customization system with conflict detection, profile management, and cross-platform keyboard handling for RiceCoder

## Features

- **Cross-Platform Support**: Consistent keybinding system across Windows, macOS, and Linux
- **Profile Management**: Multiple keybinding profiles for different workflows and preferences
- **Conflict Detection**: Automatic detection and resolution of keybinding conflicts
- **Customizable Layouts**: User-defined keybinding schemes with import/export capabilities
- **Context-Aware Bindings**: Different keybindings for different modes and contexts

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-keybinds = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_keybinds::{KeybindManager, KeybindProfile};

// Create keybind manager
let manager = KeybindManager::new();

// Load a keybind profile
let profile = KeybindProfile::load("vim")?;

// Register keybindings
manager.register_profile(profile)?;

// Handle key events
if let Some(action) = manager.get_action_for_key(key_event)? {
    match action {
        KeybindAction::Save => save_document()?,
        KeybindAction::Quit => quit_application(),
        _ => {}
    }
}
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-keybinds).

## DDD Layer

**Layer**: Presentation (User Interface)

### Responsibilities
- Keybind definition and management
- Profile management and switching
- Conflict detection and resolution
- Cross-platform keyboard handling
- Context-aware binding dispatch
- Keybind persistence and import/export

### SOLID Analysis
- **SRP**: Focused on keybind management only ✅
- **OCP**: Extensible via custom profiles and actions ✅
- **LSP**: KeybindProfile abstraction supports custom profiles ✅
- **ISP**: Separate interfaces for Manager, Profile, Action ✅
- **DIP**: Depends on abstractions for storage and event handling ✅

### Integration Points
| Dependency | Direction | Purpose |
|------------|-----------|---------|
| ricecoder-storage | Inbound | Keybind persistence |
| ricecoder-tui | Outbound | Key event handling |
| ricecoder-config | Inbound | Keybind configuration |
| crossterm | External | Cross-platform key events |

## License

MIT
