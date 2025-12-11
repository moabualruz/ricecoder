# ricecoder-help

Help system and documentation for RiceCoder TUI.

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