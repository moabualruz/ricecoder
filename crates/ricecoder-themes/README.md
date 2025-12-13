# ricecoder-themes

**Purpose**: Theme management system for RiceCoder's terminal user interface, providing built-in themes, runtime switching, and syntax highlighting integration.

## Overview

The `ricecoder-themes` crate implements a comprehensive theme system for RiceCoder's TUI (Terminal User Interface). It provides a collection of professionally designed themes, runtime theme switching capabilities, theme validation, and seamless integration with syntax highlighting for code display.

## Key Features

- **6 Built-in Themes**: Dark, Light, Monokai, Dracula, Nord, and High Contrast themes
- **Runtime Theme Switching**: Change themes instantly without application restart
- **Theme Registry**: Manage built-in and custom themes with metadata
- **Syntax Highlighting Integration**: Coordinated color schemes for code syntax elements
- **Theme Validation**: Ensure theme integrity and color compatibility
- **Event System**: Listen for theme changes to update UI components
- **Persistence**: Save and restore theme preferences

## Built-in Themes

### Dark Theme (Default)
- **Primary**: White text on black background
- **Secondary**: Light gray accents
- **Syntax**: Orange keywords, green strings, yellow numbers, gray comments
- **Best for**: Low-light environments, extended coding sessions

### Light Theme
- **Primary**: Black text on white background
- **Secondary**: Dark gray accents
- **Syntax**: Blue keywords, green strings, orange numbers
- **Best for**: Bright environments, presentations

### Monokai
- **Primary**: Cream text on dark charcoal background
- **Secondary**: Bright cyan and yellow accents
- **Syntax**: Bright orange keywords, green strings, yellow numbers
- **Best for**: Classic coding aesthetic, vibrant displays

### Dracula
- **Primary**: Light text on dark purple-tinted background
- **Secondary**: Pink and cyan accents
- **Syntax**: Pink keywords, green strings, yellow numbers
- **Best for**: Modern dark theme with purple tones

### Nord
- **Primary**: Off-white text on dark blue-gray background
- **Secondary**: Frost blue and aurora accents
- **Syntax**: Frost blue keywords, aurora green strings
- **Best for**: Arctic-inspired color palette, cold environments

### High Contrast
- **Primary**: Bright white on pure black
- **Secondary**: Maximum contrast colors
- **Syntax**: High-contrast syntax colors for accessibility
- **Best for**: Accessibility, visual impairments, high-visibility needs

## Theme Structure

Each theme contains comprehensive color definitions:

```rust
Theme {
    name: String,           // Theme identifier
    primary: Color,         // Main text color
    secondary: Color,       // Secondary UI elements
    background: Color,      // Main background
    foreground: Color,      // Primary text
    accent: Color,          // Interactive elements
    error: Color,           // Error states
    warning: Color,         // Warning states
    success: Color,         // Success states
    syntax: SyntaxTheme {   // Code syntax colors
        keyword: Color,     // Keywords (fn, let, if)
        string: Color,      // String literals
        number: Color,      // Numeric literals
        comment: Color,     // Comments
        function: Color,    // Function names
        variable: Color,    // Variable names
        type: Color,        // Type names
        constant: Color,    // Constants
    }
}
```

## Dependencies

- `ricecoder-storage` - For theme persistence and configuration integration
- External crates: `ratatui`, `syntect`, `serde`, `tokio`, `anyhow`

## Usage Examples

### Basic Theme Management

```rust
use ricecoder_themes::ThemeManager;

let mut theme_manager = ThemeManager::new();

// Get current theme
let current_theme = theme_manager.current()?;
println!("Current theme: {}", current_theme.name);

// Switch to a built-in theme
theme_manager.switch_by_name("dracula")?;

// Get available themes
let available = Theme::available_themes();
println!("Available themes: {:?}", available);
```

### Theme Event Listening

```rust
use ricecoder_themes::ThemeManager;

// Listen for theme changes to update UI
theme_manager.add_listener(|theme| {
    println!("Theme changed to: {}", theme.name);
    // Update terminal colors, refresh UI components
    update_terminal_colors(&theme);
    refresh_ui();
});
```

### Custom Theme Creation

```rust
use ricecoder_themes::{Theme, SyntaxTheme};
use ratatui::style::Color;

let custom_theme = Theme {
    name: "My Custom Theme".to_string(),
    primary: Color::Rgb(255, 255, 255),
    secondary: Color::Rgb(128, 128, 128),
    background: Color::Rgb(32, 32, 32),
    foreground: Color::Rgb(240, 240, 240),
    accent: Color::Rgb(0, 255, 255),
    error: Color::Rgb(255, 100, 100),
    warning: Color::Rgb(255, 200, 100),
    success: Color::Rgb(100, 255, 100),
    syntax: SyntaxTheme {
        keyword: Color::Rgb(255, 100, 200),
        string: Color::Rgb(100, 255, 100),
        number: Color::Rgb(255, 255, 100),
        comment: Color::Rgb(128, 128, 128),
        function: Color::Rgb(100, 200, 255),
        variable: Color::Rgb(255, 255, 255),
        r#type: Color::Rgb(200, 100, 255),
        constant: Color::Rgb(255, 150, 100),
    },
};

// Register the custom theme
theme_manager.register_theme(custom_theme)?;
theme_manager.switch_by_name("My Custom Theme")?;
```

### Theme Registry Operations

```rust
use ricecoder_themes::ThemeRegistry;

let registry = ThemeRegistry::new();

// Get a theme by name
if let Some(theme) = registry.get("dark") {
    println!("Found theme: {}", theme.name);
}

// Register a custom theme
let custom_theme = Theme::default(); // Create your theme
registry.register("my-theme", custom_theme)?;
```

### Theme Validation

```rust
use ricecoder_themes::Theme;

// Validate theme integrity
let theme = Theme::default();
match theme.validate() {
    Ok(()) => println!("Theme is valid"),
    Err(e) => println!("Theme validation error: {}", e),
}
```

## Integration with RiceCoder TUI

The theme system integrates deeply with RiceCoder's terminal interface:

- **Automatic Color Application**: Themes automatically apply to all UI components
- **Syntax Highlighting**: Code display uses theme's syntax colors
- **Component Styling**: Buttons, menus, dialogs use theme colors
- **Accessibility**: High contrast theme for improved accessibility
- **Persistence**: Theme preferences saved across sessions

## Configuration

Themes can be configured via RiceCoder's configuration system:

```toml
[theme]
current = "dracula"

[ui]
theme = "dracula"  # Sync with theme system
```

## Testing

Comprehensive test suite covering:

```bash
cargo test -p ricecoder-themes
```

**Test Coverage:**
- Theme loading and validation
- Runtime theme switching
- Registry operations
- Event system functionality
- Color compatibility
- Syntax highlighting integration
- Property-based tests for theme generation