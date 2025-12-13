# ricecoder-help

**Purpose**: Comprehensive help system providing contextual assistance, documentation access, and user guidance for RiceCoder

## Overview

`ricecoder-help` implements a rich help system for RiceCoder, offering contextual assistance, searchable documentation, and organized guidance through an intuitive terminal interface. It provides categorized help content with search functionality and keyboard navigation.

## Features

- **Help Dialog Widget**: Scrollable help content with keyboard navigation
- **Search Functionality**: Search help content with Ctrl+F
- **Organized Categories**: Help organized by categories (Getting Started, Commands, etc.)
- **Keyboard Navigation**: Full keyboard support with escape to close
- **Default Content**: Comprehensive built-in help for RiceCoder

## Usage

```rust
use ricecoder_help::{HelpDialog, HelpContent};

// Create help dialog with default RiceCoder content
let mut dialog = HelpDialog::default_ricecoder();

// Show the dialog
dialog.show();

// Handle keyboard input
dialog.handle_key(key_event)?;

// Render in TUI
dialog.render(&mut frame, area);
```

## Custom Help Content

```rust
use ricecoder_help::{HelpContent, HelpCategory};

let content = HelpContent::new()
    .add_category(
        HelpCategory::new("Getting Started")
            .with_description("Basic information")
            .add_item("Welcome", "Welcome to the application!")
            .add_item("Usage", "How to use this application")
    )
    .add_category(
        HelpCategory::new("Commands")
            .add_item("/help", "Show help dialog")
            .add_item("/exit", "Exit application")
    );

let dialog = HelpDialog::new(content);
```

## Keyboard Shortcuts

### Browse Mode
- `ESC`: Close help dialog
- `Ctrl+F`: Start search
- `↑↓←→`: Navigate categories and items
- `Page Up/Down`: Scroll content
- `Home/End`: Go to top/bottom

### Search Mode
- `ESC`: Return to browse mode
- `Enter`: Perform search
- `↑↓`: Navigate search results
- `Backspace`: Edit search query

## Requirements

This crate validates the following requirements from the RiceCoder TUI improvement specification:

- **10.1**: Show help dialog when user types `/help`
- **10.2**: Support navigation with scrolling and search (Ctrl+F)
- **10.3**: Filter content with search functionality
- **10.4**: Organize help by categories (Getting Started, Commands, etc.)

## Dependencies

- **TUI Rendering**: `ratatui` for terminal interface
- **Text Processing**: `regex` for search functionality
- **Async Runtime**: `tokio` for background operations
- **Storage**: `ricecoder-storage` for help content persistence

### Integration Points
- **TUI**: Provides help dialog widget for terminal interface
- **Commands**: Contextual help for command usage
- **Sessions**: Session-specific help and guidance
- **All Crates**: Help content integration throughout RiceCoder

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-help = "0.1"
```

## Configuration

Help system configuration via YAML:

```yaml
help:
  # Content settings
  content:
    max_categories: 20
    max_items_per_category: 50
    enable_rich_formatting: true

  # Search settings
  search:
    enable_fuzzy_search: true
    max_results: 20
    highlight_matches: true

  # UI settings
  ui:
    max_dialog_width: 100
    max_dialog_height: 30
    show_line_numbers: false
    enable_syntax_highlighting: true

  # Keyboard shortcuts
  shortcuts:
    show_help: "f1"
    search: "ctrl+f"
    close: "esc"
```

## API Reference

### Key Types

- **`HelpDialog`**: Main help dialog widget
- **`HelpContent`**: Structured help content container
- **`HelpCategory`**: Categorized help content
- **`HelpSearch`**: Search functionality for help content
- **`HelpWidget`**: TUI integration component

### Key Functions

- **`add_category()`**: Add categorized help content
- **`search()`**: Search help content with query
- **`render()`**: Render help dialog in terminal
- **`get_contextual_help()`**: Get context-specific help

## Error Handling

```rust
use ricecoder_help::HelpError;

match help_system.search(query) {
    Ok(results) => println!("Found {} results", results.len()),
    Err(HelpError::ContentNotFound) => eprintln!("Help content not available"),
    Err(HelpError::SearchFailed(msg)) => eprintln!("Search error: {}", msg),
    Err(HelpError::RenderError(msg)) => eprintln!("Rendering error: {}", msg),
}
```

## Testing

Run comprehensive help system tests:

```bash
# Run all tests
cargo test -p ricecoder-help

# Run property tests for help content
cargo test -p ricecoder-help property

# Test search functionality
cargo test -p ricecoder-help search

# Test TUI integration
cargo test -p ricecoder-help tui
```

Key test areas:
- Content organization and categorization
- Search algorithm correctness
- TUI rendering and interaction
- Keyboard navigation handling
- Content loading and parsing

## Performance

- **Content Loading**: < 50ms for typical help databases
- **Search Operations**: < 10ms for queries in < 1000 help items
- **Dialog Rendering**: < 20ms for help dialog display
- **Navigation**: < 5ms for category and item switching
- **Memory**: Efficient storage with lazy loading of large help content

## Contributing

When working with `ricecoder-help`:

1. **User Experience**: Ensure help content is clear and discoverable
2. **Search Quality**: Optimize search algorithms for relevance and speed
3. **Accessibility**: Support screen readers and keyboard navigation
4. **Content Organization**: Maintain logical categorization and hierarchy
5. **Performance**: Keep help system responsive even with large content

## License

MIT