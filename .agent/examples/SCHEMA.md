# Markdown Configuration Schema

This document describes the YAML frontmatter schema for markdown configuration files in RiceCoder.

## Overview

All markdown configuration files use YAML frontmatter (delimited by `---`) to define metadata, followed by markdown content that serves as documentation or prompts.

```markdown
---
# YAML frontmatter with configuration
name: example
description: Example configuration
---

# Markdown content
This is the markdown content that follows the frontmatter.
```

## Agent Configuration Schema

Agent configurations define AI agents with specific capabilities and parameters.

**File Pattern**: `*.agent.md`

**Location**: `~/.ricecoder/agents/` or `projects/ricecoder/.agent/`

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique identifier for the agent (lowercase, no spaces) |
| `description` | string | No | Human-readable description of the agent's purpose |
| `model` | string | No | LLM model to use (e.g., "gpt-4", "gpt-3.5-turbo") |
| `temperature` | float | No | Temperature for model (0.0-2.0, default: 0.7) |
| `max_tokens` | integer | No | Maximum tokens in response (default: 2000) |
| `tools` | array | No | List of tools available to the agent |

### Example

```yaml
---
name: code-review
description: AI-powered code review agent
model: gpt-4
temperature: 0.7
max_tokens: 2000
tools:
  - syntax-analyzer
  - security-checker
  - best-practices-validator
---

# Code Review Agent

This agent performs comprehensive code reviews...
```

### Validation Rules

- `name`: Must be lowercase alphanumeric with hyphens only
- `description`: Must be non-empty string
- `model`: Must be a valid model identifier
- `temperature`: Must be between 0.0 and 2.0
- `max_tokens`: Must be positive integer
- `tools`: Must be array of strings

## Mode Configuration Schema

Mode configurations define editor modes with specific behaviors and keybindings.

**File Pattern**: `*.mode.md`

**Location**: `~/.ricecoder/modes/` or `projects/ricecoder/.agent/`

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique identifier for the mode (lowercase, no spaces) |
| `description` | string | No | Human-readable description of the mode |
| `prompt` | string | No | Prompt text for the mode (from markdown content) |
| `keybinding` | string | No | Keyboard shortcut to activate mode (e.g., "ctrl+shift+f") |
| `enabled` | boolean | No | Whether the mode is enabled (default: true) |

### Example

```yaml
---
name: focus
description: Focus mode for distraction-free coding
keybinding: "ctrl+shift+f"
enabled: true
---

# Focus Mode

Focus mode provides a distraction-free coding environment...
```

### Validation Rules

- `name`: Must be lowercase alphanumeric with hyphens only
- `description`: Must be non-empty string
- `keybinding`: Must be valid keybinding format (e.g., "ctrl+shift+f")
- `enabled`: Must be boolean

## Command Configuration Schema

Command configurations define custom commands with parameters and templates.

**File Pattern**: `*.command.md`

**Location**: `~/.ricecoder/commands/` or `projects/ricecoder/.agent/`

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique identifier for the command (lowercase, no spaces) |
| `description` | string | No | Human-readable description of the command |
| `template` | string | Yes | Command template with parameter placeholders |
| `parameters` | array | No | List of parameter definitions |
| `keybinding` | string | No | Keyboard shortcut to invoke command |

### Parameter Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Parameter name (used in template as `{{name}}`) |
| `description` | string | No | Human-readable description of the parameter |
| `required` | boolean | No | Whether the parameter is required (default: false) |
| `default` | string | No | Default value if not provided |

### Example

```yaml
---
name: test
description: Run tests for the current project
template: "cargo test {{test_filter}}"
keybinding: "ctrl+shift+t"
parameters:
  - name: test_filter
    description: Optional filter for specific tests
    required: false
    default: ""
---

# Test Command

The test command runs tests for the current project...
```

### Validation Rules

- `name`: Must be lowercase alphanumeric with hyphens only
- `description`: Must be non-empty string
- `template`: Must be non-empty string with valid placeholders
- `keybinding`: Must be valid keybinding format
- Parameter `name`: Must be valid identifier (alphanumeric, underscores)
- Parameter `required`: Must be boolean
- Placeholders in template must match parameter names

## Keybinding Format

Keybindings use the following format:

```
[modifier+]key
```

### Modifiers

- `ctrl` - Control key
- `shift` - Shift key
- `alt` - Alt/Option key
- `meta` - Meta/Command key (macOS)

### Examples

- `ctrl+s` - Control+S
- `ctrl+shift+f` - Control+Shift+F
- `alt+enter` - Alt+Enter
- `meta+k` - Command+K (macOS)

## Discovery and Loading

Configuration files are discovered in the following locations (in priority order):

1. **Project-level**: `projects/ricecoder/.agent/`
2. **User-level**: `~/.ricecoder/agents/`, `~/.ricecoder/modes/`, `~/.ricecoder/commands/`
3. **System-level**: `/etc/ricecoder/agents/` (Linux/macOS)

Files are loaded based on their extension:
- `*.agent.md` - Agent configurations
- `*.mode.md` - Mode configurations
- `*.command.md` - Command configurations

## Hot-Reload

Configuration files are monitored for changes. When a file is modified:

1. File is detected within 5 seconds
2. Configuration is re-parsed and validated
3. If valid, configuration is updated in the registry
4. If invalid, error is logged and previous configuration is retained

## Error Handling

### Parsing Errors

If a markdown file cannot be parsed:
- Error is logged with file path and line number
- File is skipped
- System continues loading other files

### Validation Errors

If configuration fails validation:
- All validation errors are reported
- File is skipped
- System continues loading other files

### Example Error Message

```
Error loading configuration: projects/ricecoder/.agent/code-review.agent.md
  - Line 3: Invalid field 'temperature': must be between 0.0 and 2.0 (got 2.5)
  - Line 4: Invalid field 'max_tokens': must be positive integer (got -100)
```

## Best Practices

### Naming

- Use lowercase names with hyphens: `code-review`, `focus-mode`, `run-tests`
- Avoid spaces and special characters
- Use descriptive names that indicate purpose

### Documentation

- Always include a description
- Add markdown content explaining the configuration
- Include usage examples
- Document any special requirements

### Parameters

- Keep parameter names short and descriptive
- Provide default values when possible
- Document parameter constraints
- Use consistent naming conventions

### Organization

- Group related configurations in directories
- Use consistent file naming
- Keep configurations simple and focused
- Avoid duplication across files

## Examples

See the following example files for complete configurations:

- `code-review.agent.md` - Agent configuration example
- `focus.mode.md` - Mode configuration example
- `test.command.md` - Command configuration example

## See Also

- [Configuration Guide](https://github.com/moabualruz/ricecoder/wiki/Configuration.md)
- [CLI Commands Reference](https://github.com/moabualruz/ricecoder/wiki/CLI-Commands.md)
- [Markdown Configuration Module](../../../projects/ricecoder/crates/ricecoder-storage/src/markdown_config/)
