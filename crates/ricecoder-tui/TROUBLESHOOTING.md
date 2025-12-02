# RiceCoder TUI Troubleshooting Guide

This guide helps you resolve common issues with the RiceCoder Terminal User Interface.

## Common Issues and Solutions

### Terminal Display Issues

#### Problem: Text appears garbled or misaligned

**Causes:**
- Terminal doesn't support the required color depth
- Terminal size is too small (minimum 80x24)
- Font rendering issues

**Solutions:**
1. Check terminal size: `echo $COLUMNS x $LINES` (Unix/Linux) or use terminal settings
2. Ensure terminal supports at least 256 colors
3. Try a different terminal emulator (Windows Terminal, iTerm2, Alacritty, etc.)
4. Update terminal font to a monospace font (Fira Code, Consolas, etc.)

#### Problem: Colors don't display correctly

**Causes:**
- Terminal color capability detection failed
- Theme incompatible with terminal color depth
- Terminal color palette misconfigured

**Solutions:**
1. Check terminal color support: `echo $TERM`
2. Try switching to a different theme: `ricecoder-tui --theme light`
3. Enable high contrast mode for better visibility: `ricecoder-tui --high-contrast`
4. Update terminal color palette settings

### Performance Issues

#### Problem: TUI is slow or laggy

**Causes:**
- Large chat history (>1000 messages)
- Large diffs (>10MB files)
- Syntax highlighting overhead
- Terminal rendering performance

**Solutions:**
1. Clear chat history to reduce memory usage
2. Disable syntax highlighting for large diffs: `ricecoder-tui --no-syntax-highlight`
3. Reduce animation frame rate: `ricecoder-tui --animation-fps 30`
4. Use a faster terminal emulator (Windows Terminal, Alacritty)
5. Check system resources (CPU, memory) with system monitor

#### Problem: Theme switching is slow

**Causes:**
- Theme file is large or complex
- Rendering all UI elements with new theme
- Disk I/O for theme loading

**Solutions:**
1. Use built-in themes instead of custom themes
2. Simplify custom theme definitions
3. Ensure theme files are on fast storage (SSD)
4. Check system load with `top` or Task Manager

### Input and Navigation Issues

#### Problem: Keyboard shortcuts don't work

**Causes:**
- Terminal intercepts key combinations
- Keybindings not configured correctly
- Mode-specific keybindings not active

**Solutions:**
1. Check current mode (Chat, Command, Diff, Help)
2. Verify keybindings in config file: `~/.ricecoder/config.yaml`
3. Try alternative keybindings
4. Check terminal keybinding settings (may override app keybindings)

#### Problem: Mouse input not working

**Causes:**
- Mouse support disabled in config
- Terminal doesn't support mouse events
- Mouse events not properly handled

**Solutions:**
1. Enable mouse support: `ricecoder-tui --mouse`
2. Check terminal mouse support settings
3. Try using keyboard navigation instead (Tab, arrow keys)
4. Verify mouse works in other terminal applications

### Chat and Streaming Issues

#### Problem: Messages don't stream in real-time

**Causes:**
- Network latency
- Streaming handler not properly initialized
- Buffer size too large

**Solutions:**
1. Check network connection: `ping api.openai.com`
2. Verify AI provider configuration
3. Check system resources (CPU, memory)
4. Try reducing buffer size in config

#### Problem: Chat history not persisting

**Causes:**
- Storage not configured
- Permissions issue with storage directory
- Storage backend not initialized

**Solutions:**
1. Check storage configuration in config file
2. Verify directory permissions: `ls -la ~/.ricecoder/`
3. Ensure storage directory exists and is writable
4. Check disk space: `df -h`

### Diff Display Issues

#### Problem: Diff lines are cut off or wrapped incorrectly

**Causes:**
- Terminal width too narrow
- Long lines in diff
- Text wrapping not configured correctly

**Solutions:**
1. Increase terminal width (minimum 80 characters)
2. Use side-by-side view for better readability
3. Enable line wrapping in config
4. Try a wider terminal or zoom out

#### Problem: Syntax highlighting not working in diffs

**Causes:**
- Language not detected correctly
- Syntax highlighting disabled
- Theme doesn't support syntax highlighting

**Solutions:**
1. Verify file extension is correct
2. Enable syntax highlighting: `ricecoder-tui --syntax-highlight`
3. Try a different theme with better syntax highlighting
4. Check if language is supported by syntect

### Accessibility Issues

#### Problem: Screen reader not announcing content

**Causes:**
- Screen reader not enabled in config
- Accessibility features not initialized
- Screen reader not compatible with terminal

**Solutions:**
1. Enable screen reader: `ricecoder-tui --screen-reader`
2. Check screen reader compatibility with your terminal
3. Verify accessibility config in `~/.ricecoder/config.yaml`
4. Try a different screen reader (NVDA, JAWS, VoiceOver)

#### Problem: High contrast mode not working

**Causes:**
- High contrast mode not enabled
- Theme doesn't support high contrast
- Terminal color palette not configured

**Solutions:**
1. Enable high contrast mode: `ricecoder-tui --high-contrast`
2. Use a theme designed for high contrast (e.g., "high-contrast")
3. Configure terminal color palette for high contrast
4. Check color contrast ratios (should be 4.5:1 or higher)

### Configuration Issues

#### Problem: Configuration file not being read

**Causes:**
- Config file in wrong location
- Config file format incorrect
- Permissions issue

**Solutions:**
1. Check config file location: `~/.ricecoder/config.yaml`
2. Validate YAML syntax: `yamllint ~/.ricecoder/config.yaml`
3. Check file permissions: `ls -la ~/.ricecoder/config.yaml`
4. Try creating config file from template

#### Problem: Configuration changes not taking effect

**Causes:**
- Config file not saved
- Application not restarted
- Config file syntax error

**Solutions:**
1. Save config file and verify changes
2. Restart the application
3. Check for YAML syntax errors
4. Verify config file is in correct location

## Getting Help

If you encounter an issue not covered in this guide:

1. **Check logs**: Look for error messages in `~/.ricecoder/logs/`
2. **Enable debug mode**: `ricecoder-tui --debug` for verbose logging
3. **Report issue**: Include logs, terminal info, and reproduction steps
4. **Check documentation**: See `README.md` for more information

## Performance Tuning

### For Large Chat Histories

```yaml
# ~/.ricecoder/config.yaml
performance:
  lazy_load_enabled: true
  chunk_size: 50
  max_chunks: 10
```

### For Large Diffs

```yaml
# ~/.ricecoder/config.yaml
performance:
  diff_max_lines: 1000
  disable_syntax_highlight_above: 5000
```

### For Slow Terminals

```yaml
# ~/.ricecoder/config.yaml
animations: false
animation_fps: 15
mouse: false
```

## System Requirements

- **Terminal**: 80x24 minimum, 256-color support recommended
- **OS**: Windows, macOS, Linux
- **Memory**: 50MB minimum, 200MB recommended
- **CPU**: Modern processor (2+ cores recommended)
- **Network**: For AI provider integration

## Supported Terminals

- **Windows**: Windows Terminal, ConEmu, Cmder
- **macOS**: Terminal.app, iTerm2, Alacritty
- **Linux**: GNOME Terminal, Konsole, Alacritty, xterm

## Additional Resources

- [RiceCoder Documentation](https://github.com/ricecoder/ricecoder)
- [Ratatui Documentation](https://docs.rs/ratatui/)
- [Crossterm Documentation](https://docs.rs/crossterm/)
