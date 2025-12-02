# RiceCoder TUI Usage Guide

This guide provides comprehensive examples and usage patterns for the RiceCoder Terminal User Interface.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Chat Widget](#chat-widget)
3. [Diff Widget](#diff-widget)
4. [Prompt Widget](#prompt-widget)
5. [Configuration](#configuration)
6. [Keyboard Shortcuts](#keyboard-shortcuts)
7. [Advanced Usage](#advanced-usage)

## Getting Started

### Starting the TUI

```bash
# Start with default configuration
ricecoder-tui

# Start with specific theme
ricecoder-tui --theme dracula

# Start with debug logging
ricecoder-tui --debug

# Start with high contrast mode
ricecoder-tui --high-contrast
```

### Basic Navigation

- **Tab**: Move focus between widgets
- **Shift+Tab**: Move focus backward
- **Ctrl+C**: Exit application
- **Ctrl+1**: Switch to Chat mode
- **Ctrl+2**: Switch to Command mode
- **Ctrl+3**: Switch to Diff mode
- **Ctrl+4**: Show Help

## Chat Widget

The Chat Widget provides a conversational interface for interacting with AI assistants.

### Features

- Real-time message streaming
- Markdown rendering with syntax highlighting
- Message history with scrolling
- Copy/paste operations
- Multi-line input

### Usage Examples

#### Sending a Message

```
1. Type your message in the input area
2. Press Enter to send
3. Watch the response stream in real-time
```

#### Navigating Chat History

```
- Up Arrow: Scroll up in message history
- Down Arrow: Scroll down in message history
- Page Up: Jump up one screen
- Page Down: Jump down one screen
- Home: Jump to oldest message
- End: Jump to newest message
```

#### Copying Messages

```
1. Navigate to the message you want to copy
2. Press 'c' to copy the entire message
3. Press 'Ctrl+Shift+C' to copy code blocks only
4. Paste with Ctrl+V in your editor
```

#### Editing Messages

```
1. Navigate to your message
2. Press 'e' to edit
3. Modify the message
4. Press Enter to resubmit
5. Press Escape to cancel
```

### Markdown Support

The Chat Widget supports the following markdown features:

```markdown
# Heading 1
## Heading 2
### Heading 3

**Bold text**
*Italic text*
***Bold italic***

- Bullet point
- Another point
  - Nested point

1. Numbered list
2. Second item

`inline code`

\`\`\`python
# Code block with syntax highlighting
def hello():
    print("Hello, world!")
\`\`\`

[Link text](https://example.com)

> Blockquote
> Multiple lines
```

## Diff Widget

The Diff Widget displays code changes with syntax highlighting and approval workflow.

### Features

- Unified and side-by-side views
- Syntax highlighting for 500+ languages
- Hunk-level navigation and approval
- Line number display
- Collapsible hunks

### Usage Examples

#### Viewing Diffs

```
1. Navigate to Diff mode (Ctrl+3)
2. View the diff in unified format
3. Use arrow keys to navigate
```

#### Switching View Formats

```
- 'u': Switch to unified view
- 's': Switch to side-by-side view
- 'v': Toggle view format
```

#### Navigating Hunks

```
- 'n': Jump to next hunk
- 'p': Jump to previous hunk
- 'j': Jump to specific hunk (enter hunk number)
- 'Home': Jump to first hunk
- 'End': Jump to last hunk
```

#### Approving Changes

```
- 'a': Approve current hunk
- 'r': Reject current hunk
- 'A': Approve all hunks
- 'R': Reject all hunks
- 'Space': Toggle hunk approval
```

#### Collapsing Hunks

```
- 'c': Collapse/expand current hunk
- 'C': Collapse all hunks
- 'E': Expand all hunks
```

### Diff Statistics

The diff header shows:
- Total number of lines
- Number of added lines (+)
- Number of removed lines (-)
- Number of approved hunks

Example:
```
Unified Diff View | 150 lines | +45 -30 | Approved: 2/5
```

## Prompt Widget

The Prompt Widget displays the command prompt with context information.

### Context Information

The prompt displays:
- **Git Branch**: Current git branch (if in a git repository)
- **Project Name**: Name of the current project
- **Mode**: Current application mode (Chat, Command, Diff, Help)
- **Provider**: Active AI provider (e.g., OpenAI, Anthropic)
- **Model**: Active AI model (e.g., gpt-4, claude-3)

### Example Prompt

```
[main] ricecoder (Chat) [OpenAI/gpt-4] ❯ 
```

### Multi-line Input

```
1. Type your command
2. Press Shift+Enter for a new line
3. Press Enter to submit
4. Use Up/Down arrows to navigate input history
```

## Configuration

### Configuration File Location

- **Linux/macOS**: `~/.ricecoder/config.yaml`
- **Windows**: `%APPDATA%\ricecoder\config.yaml`

### Configuration Options

```yaml
# Theme selection
theme: dracula

# Enable/disable animations
animations: true

# Enable/disable mouse support
mouse: true

# Terminal dimensions (auto-detect if not specified)
width: 120
height: 40

# Accessibility options
accessibility:
  screen_reader_enabled: false
  high_contrast_mode: false
  disable_animations: false
  focus_indicator_style: "box"

# Performance options
performance:
  lazy_load_enabled: true
  chunk_size: 50
  max_chunks: 10
  diff_max_lines: 1000
  disable_syntax_highlight_above: 5000

# Keybindings
keybindings:
  vim_mode: false
  custom_bindings: {}
```

### Available Themes

Built-in themes:
- `dark` (default)
- `light`
- `monokai`
- `dracula`
- `nord`
- `high-contrast`

### Custom Themes

Create a custom theme in `~/.ricecoder/themes/my-theme.yaml`:

```yaml
name: my-theme
colors:
  primary: "#007AFF"
  secondary: "#5AC8FA"
  background: "#FFFFFF"
  text: "#000000"
  success: "#34C759"
  error: "#FF3B30"
  warning: "#FF9500"
  info: "#00C7FF"
```

## Keyboard Shortcuts

### Global Shortcuts

| Key | Action |
|-----|--------|
| Ctrl+C | Exit application |
| Ctrl+1 | Switch to Chat mode |
| Ctrl+2 | Switch to Command mode |
| Ctrl+3 | Switch to Diff mode |
| Ctrl+4 | Show Help |
| Ctrl+T | Toggle theme |
| Ctrl+L | Clear screen |
| Ctrl+H | Show keybindings help |

### Chat Mode

| Key | Action |
|-----|--------|
| Enter | Send message |
| Shift+Enter | New line in input |
| Up/Down | Navigate history |
| Page Up/Down | Scroll messages |
| c | Copy message |
| e | Edit message |
| r | Regenerate response |
| Ctrl+A | Select all |
| Ctrl+X | Cut |
| Ctrl+C | Copy |
| Ctrl+V | Paste |

### Diff Mode

| Key | Action |
|-----|--------|
| n | Next hunk |
| p | Previous hunk |
| a | Approve hunk |
| r | Reject hunk |
| A | Approve all |
| R | Reject all |
| u | Unified view |
| s | Side-by-side view |
| c | Collapse hunk |
| C | Collapse all |
| E | Expand all |

### Navigation

| Key | Action |
|-----|--------|
| Tab | Next widget |
| Shift+Tab | Previous widget |
| Arrow Keys | Navigate within widget |
| Home | Jump to start |
| End | Jump to end |
| Page Up | Scroll up |
| Page Down | Scroll down |

## Advanced Usage

### Vim Mode

Enable vim-style keybindings in config:

```yaml
keybindings:
  vim_mode: true
```

Vim keybindings:
- `h/j/k/l`: Navigate (left/down/up/right)
- `w/b`: Jump word forward/backward
- `gg`: Jump to start
- `G`: Jump to end
- `d`: Delete
- `y`: Yank (copy)
- `p`: Paste
- `:q`: Quit
- `:w`: Save

### Screen Reader Support

Enable screen reader support for accessibility:

```bash
ricecoder-tui --screen-reader
```

Or in config:

```yaml
accessibility:
  screen_reader_enabled: true
```

### High Contrast Mode

Enable high contrast mode for better visibility:

```bash
ricecoder-tui --high-contrast
```

Or in config:

```yaml
accessibility:
  high_contrast_mode: true
```

### Performance Optimization

For large chat histories:

```yaml
performance:
  lazy_load_enabled: true
  chunk_size: 100
  max_chunks: 5
```

For large diffs:

```yaml
performance:
  diff_max_lines: 500
  disable_syntax_highlight_above: 2000
```

### Custom Keybindings

Define custom keybindings in config:

```yaml
keybindings:
  custom_bindings:
    "Ctrl+Shift+S": "save"
    "Ctrl+Shift+L": "load"
    "Alt+1": "chat_mode"
    "Alt+2": "command_mode"
```

## Tips and Tricks

### Efficient Navigation

1. Use Tab to move between widgets instead of mouse
2. Use keyboard shortcuts for common operations
3. Enable vim mode if you're familiar with vim
4. Use Page Up/Down for faster scrolling

### Performance

1. Disable animations for slower terminals
2. Use built-in themes instead of custom themes
3. Enable lazy loading for large chat histories
4. Disable syntax highlighting for very large diffs

### Accessibility

1. Enable screen reader support for better accessibility
2. Use high contrast mode for better visibility
3. Disable animations if they cause issues
4. Use keyboard navigation exclusively

### Customization

1. Create custom themes for your preferred colors
2. Define custom keybindings for your workflow
3. Configure performance settings for your system
4. Enable/disable features based on your needs

## Troubleshooting

For common issues and solutions, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

## Additional Resources

- [RiceCoder Documentation](https://github.com/ricecoder/ricecoder)
- [Ratatui Documentation](https://docs.rs/ratatui/)
- [Crossterm Documentation](https://docs.rs/crossterm/)
