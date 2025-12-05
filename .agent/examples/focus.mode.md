---
name: focus
description: Focus mode for distraction-free coding with minimal UI
keybinding: "ctrl+shift+f"
enabled: true
---

# Focus Mode

Focus mode provides a distraction-free coding environment by:

- Hiding non-essential UI elements
- Maximizing editor space
- Reducing visual clutter
- Enabling keyboard-only navigation

## Features

### Minimal UI
- Hides sidebar and status bar
- Shows only editor and command palette
- Removes decorative elements
- Focuses on code content

### Keyboard Navigation
- All commands accessible via keyboard
- Vim-style keybindings supported
- Quick command palette (Ctrl+P)
- No mouse required

### Customizable
- Adjust UI elements to hide
- Configure keybindings
- Set font size and line height
- Choose color scheme

## Usage

### Entering Focus Mode

Press `Ctrl+Shift+F` or use the command:

```
ricecoder mode focus
```

### Exiting Focus Mode

Press `Ctrl+Shift+F` again or use:

```
ricecoder mode normal
```

## Configuration

Focus mode is configured in `~/.ricecoder/modes/focus.mode.md`:

```yaml
name: focus
description: Focus mode for distraction-free coding
keybinding: "ctrl+shift+f"
enabled: true
```

### Customization

To customize focus mode, edit the configuration:

```yaml
# Hide specific UI elements
hide_sidebar: true
hide_status_bar: true
hide_minimap: true

# Set editor preferences
font_size: 14
line_height: 1.6
word_wrap: true

# Choose color scheme
color_scheme: "dark"
```

## Examples

### Example 1: Entering Focus Mode

```
$ ricecoder mode focus
✓ Focus mode activated
  - Sidebar hidden
  - Status bar hidden
  - Editor maximized
```

### Example 2: Exiting Focus Mode

```
$ ricecoder mode normal
✓ Normal mode activated
  - Sidebar visible
  - Status bar visible
```

## Tips

1. **Use keyboard shortcuts**: Learn the keyboard shortcuts for common commands
2. **Customize keybindings**: Adjust keybindings to match your workflow
3. **Combine with other modes**: Use focus mode with other modes for different workflows
4. **Save your preferences**: Focus mode remembers your settings

## See Also

- [Modes Guide](https://github.com/moabualruz/ricecoder/wiki/Sessions-Modes.md)
- [Keybindings Reference](https://github.com/moabualruz/ricecoder/wiki/Keybindings.md)
- [Configuration Guide](https://github.com/moabualruz/ricecoder/wiki/Configuration.md)
