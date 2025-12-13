# ricecoder-config

**Purpose**: Configuration management system for RiceCoder that handles loading, validation, and persistence of application settings from multiple sources.

## Overview

The `ricecoder-config` crate provides a comprehensive configuration management system specifically designed for RiceCoder's needs. It manages editor settings, UI preferences, keybindings, and theme configurations with support for multiple configuration sources and runtime validation.

## Key Features

- **Hierarchical Configuration Loading**: Supports configuration from files (TOML/YAML/JSON), environment variables, and runtime overrides
- **Type-Safe Configuration**: Strongly typed configuration structures with serde serialization
- **Validation**: Built-in validation for configuration values (e.g., tab size > 0, font size > 0)
- **Persistence**: Save and load configuration to/from files
- **TUI Integration**: Specialized configuration for terminal user interface settings

## Configuration Sources (Priority Order)

1. **Runtime Overrides**: CLI flags and environment variables (highest priority)
2. **Project Config**: `.ricecoder/config.toml`, `.ricecoder/config.yaml`, `.ricecoder/config.json`
3. **User Config**: `~/.ricecoder/config.toml`, `~/.ricecoder/config.yaml`, `~/.ricecoder/config.json`
4. **Built-in Defaults**: Sensible defaults for all settings (lowest priority)

## Configuration Structure

### Editor Configuration
```rust
EditorConfig {
    tab_size: 4,           // Indentation tab size
    insert_spaces: true,   // Use spaces instead of tabs
    word_wrap: false,      // Wrap long lines
    line_numbers: true,    // Show line numbers
    syntax_highlight: true // Enable syntax highlighting
}
```

### UI Configuration
```rust
UiConfig {
    theme: "dark",         // Current theme name
    font_size: 12,         // Terminal font size
    show_status_bar: true, // Display status bar
    show_command_palette: true // Show command palette
}
```

### Keybinding Configuration
```rust
KeybindConfig {
    custom: HashMap<String, String> // Custom keybinding overrides
}
```

### Theme Configuration
```rust
ThemeConfig {
    current: "dark",       // Active theme name
    overrides: HashMap<String, Value> // Theme-specific overrides
}
```

## Dependencies

- `ricecoder-storage` - For persistent configuration storage and TUI config integration
- External crates: `serde`, `config`, `notify`, `tokio`, `toml`, `dirs`

## Usage Examples

### Basic Configuration Loading

```rust
use ricecoder_config::{ConfigManager, AppConfig};

let mut manager = ConfigManager::new();
let config: AppConfig = manager.load_config()?;

// Access configuration values
println!("Tab size: {}", config.editor.tab_size);
println!("Theme: {}", config.ui.theme);
```

### Custom Configuration Path

```rust
use ricecoder_config::ConfigManager;
use std::path::PathBuf;

let custom_path = PathBuf::from("./my-config.toml");
let mut manager = ConfigManager::with_path(custom_path);
let config = manager.load_config()?;
```

### Saving Configuration

```rust
use ricecoder_config::{ConfigManager, AppConfig};

let mut manager = ConfigManager::new();
let mut config = manager.load_config()?;

// Modify configuration
config.editor.tab_size = 2;
config.ui.theme = "dracula".to_string();

// Save changes
manager.save_config(&config)?;
```

### TUI-Specific Configuration

```rust
use ricecoder_config::TuiConfig;

// Load TUI-specific settings
let tui_config = TuiConfig::load()?;
println!("Animations enabled: {}", tui_config.animations);
println!("Mouse support: {}", tui_config.mouse);
```

### Configuration Validation

```rust
use ricecoder_config::ConfigManager;

let manager = ConfigManager::new();
let config = AppConfig::default();

// Validate configuration
match manager.validate_config(&config) {
    Ok(()) => println!("Configuration is valid"),
    Err(e) => println!("Configuration error: {}", e),
}
```

## Configuration Files

### TOML Format Example
```toml
[editor]
tab_size = 4
insert_spaces = true
word_wrap = false
line_numbers = true
syntax_highlight = true

[ui]
theme = "dark"
font_size = 12
show_status_bar = true
show_command_palette = true

[theme]
current = "dark"

[keybinds]
# Custom keybindings as key-value pairs
"ctrl+s" = "save_file"
"ctrl+q" = "quit"
```

### Environment Variables

Configuration can be overridden using environment variables with the `APP_` prefix:

```bash
export APP_EDITOR_TAB_SIZE=2
export APP_UI_THEME=light
export APP_UI_FONT_SIZE=14
```

## Testing

The crate includes comprehensive tests covering:

```bash
cargo test -p ricecoder-config
```

**Test Coverage:**
- Configuration file parsing (TOML, YAML, JSON)
- Environment variable overrides
- Configuration validation
- Save/load functionality
- TUI configuration integration
- Property-based tests for edge cases