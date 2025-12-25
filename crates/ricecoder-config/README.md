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

### TUI Configuration
```rust
TuiConfig {
    theme: String,                    // Theme name
    animations: bool,                 // Enable animations
    mouse: bool,                      // Enable mouse support
    width: Option<u16>,               // Terminal width
    height: Option<u16>,              // Terminal height
    accessibility: AccessibilityConfig, // Accessibility settings
    provider: Option<String>,         // AI provider to use
    model: Option<String>,            // Model to use
    vim_mode: bool,                   // Enable vim keybindings
}
```

### Accessibility Configuration
```rust
AccessibilityConfig {
    screen_reader_enabled: bool,      // Screen reader support
    high_contrast_enabled: bool,      // High contrast mode
    animations_disabled: bool,        // Disable animations
    announcements_enabled: bool,      // State announcements
    focus_indicator: FocusIndicatorStyle, // Focus style
    animations: AnimationConfig,      // Animation settings
    font_size_multiplier: f32,        // Font size multiplier (1.0-2.0)
    large_click_targets: bool,        // Large click targets
    auto_advance: bool,               // Auto-advance for forms
}
```

### Configuration Presets
```rust
enum ConfigPreset {
    Developer,      // Dark theme, vim mode enabled
    Accessibility,  // High contrast, screen reader, larger fonts
    Minimal,        // Clean, minimal interface
    Presentation,   // Light theme, optimized for presentations
}
```

## Dependencies

### Internal Dependencies
- None (leaf crate in the dependency graph)

### External Dependencies
- `serde` - Serialization/deserialization with derive macros
- `serde_json` - JSON format support
- `serde_yaml` - YAML format support
- `toml` - TOML format support
- `config` - Configuration library for hierarchical loading
- `dirs` - Platform-specific directories (config dir)
- `notify` - File system watching for hot-reload
- `tokio` - Async runtime for ConfigManager
- `thiserror` - Error type derivation
- `anyhow` - Error handling
- `tracing` - Logging

### Dependents (crates that use ricecoder-config)
- `ricecoder-di` - Uses ConfigManager and ConfigLoader for dependency injection

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

### Runtime Configuration Changes

```rust
use ricecoder_config::tui_config::{ConfigManager, RuntimeConfigChanges};

let mut manager = ConfigManager::new();

// Apply runtime changes without restart
let changes = RuntimeConfigChanges::new()
    .with_theme("dracula")
    .with_vim_mode(true)
    .with_high_contrast(false);

manager.apply_runtime_changes(changes).await?;
```

### Configuration Presets

```rust
use ricecoder_config::tui_config::{ConfigManager, ConfigPreset};

let mut manager = ConfigManager::new();

// Apply a preset
manager.apply_preset(ConfigPreset::Developer).await?;

// List available presets
for (preset, description) in ConfigManager::available_presets() {
    println!("{}: {}", preset, description);
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

## Module Structure

```
src/
├── lib.rs           # Public API exports
├── error.rs         # ConfigError type and Result alias
├── types.rs         # Core config types (AppConfig, EditorConfig, UiConfig, etc.)
├── manager.rs       # ConfigManager implementation for file-based config
└── tui_config.rs    # TuiConfig with hot-reload and runtime changes
```

## Key Types

| Type | Description |
|------|-------------|
| `AppConfig` | Main application configuration containing all settings |
| `EditorConfig` | Editor-specific settings (tabs, line numbers, etc.) |
| `UiConfig` | UI settings (theme, font size, status bar) |
| `KeybindConfig` | Custom keybinding overrides |
| `ThemeConfig` | Theme selection and overrides |
| `TuiConfig` | TUI-specific configuration with accessibility |
| `AccessibilityConfig` | Accessibility settings (screen reader, high contrast) |
| `ConfigManager` | Configuration loader/saver for file-based config |
| `ConfigError` | Error types for configuration operations |
| `ConfigPreset` | Predefined configuration presets |
| `RuntimeConfigChanges` | Runtime changes that don't require restart |