# VS Code Command Palette Integration

This document describes the VS Code command palette integration for RiceCoder, including available commands, keyboard shortcuts, and context menu integration.

## Overview

RiceCoder provides seamless integration with VS Code's command palette, allowing users to access all RiceCoder features through:

1. **Command Palette** (Ctrl+Shift+P / Cmd+Shift+P)
2. **Keyboard Shortcuts** (Quick access to common operations)
3. **Context Menu** (Right-click in editor)
4. **Editor Commands** (Direct invocation)

## Available Commands

All RiceCoder commands are grouped under the "RiceCoder" category in the command palette.

### 1. Chat with RiceCoder

**Command ID**: `ricecoder.chat`

**Keyboard Shortcut**: 
- Windows/Linux: `Ctrl+Shift+R`
- macOS: `Cmd+Shift+R`

**Description**: Open chat interface to ask RiceCoder questions about your code

**Usage**:
1. Press `Ctrl+Shift+R` (or `Cmd+Shift+R` on Mac)
2. Enter your question or message
3. RiceCoder will respond with AI-powered assistance

**Context**: Works with or without selected text

### 2. Review Code

**Command ID**: `ricecoder.review`

**Keyboard Shortcut**:
- Windows/Linux: `Ctrl+Shift+E`
- macOS: `Cmd+Shift+E`

**Description**: Get AI-powered code review and suggestions for the current file

**Usage**:
1. Press `Ctrl+Shift+E` (or `Cmd+Shift+E` on Mac)
2. RiceCoder will analyze the current file
3. Review suggestions will be displayed

**Context**: Works on the entire current file

### 3. Generate Code

**Command ID**: `ricecoder.generate`

**Keyboard Shortcut**:
- Windows/Linux: `Ctrl+Shift+G`
- macOS: `Cmd+Shift+G`

**Description**: Generate code based on your description

**Usage**:
1. Press `Ctrl+Shift+G` (or `Cmd+Shift+G` on Mac)
2. Enter a description of what you want to generate
3. Generated code will be inserted at the cursor position

**Context**: Inserts code at the current cursor position

### 4. Refactor Code

**Command ID**: `ricecoder.refactor`

**Keyboard Shortcut**:
- Windows/Linux: `Ctrl+Shift+F`
- macOS: `Cmd+Shift+F`

**Description**: Refactor selected code with various refactoring options

**Usage**:
1. Select the code you want to refactor
2. Press `Ctrl+Shift+F` (or `Cmd+Shift+F` on Mac)
3. Choose a refactoring option from the menu
4. Refactored code will replace the selection

**Context**: Requires selected text

**Refactoring Options**:
- Extract Function
- Rename Variable
- Simplify Logic
- Optimize Performance
- Add Documentation

### 5. Show Commands

**Command ID**: `ricecoder.help`

**Description**: Show all available RiceCoder commands

**Usage**:
1. Open command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`)
2. Type "RiceCoder: Show Commands"
3. Select the command
4. A quick pick menu will show all available commands with descriptions

### 6. Show Keyboard Shortcuts

**Command ID**: `ricecoder.shortcuts`

**Description**: Show all RiceCoder keyboard shortcuts

**Usage**:
1. Open command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`)
2. Type "RiceCoder: Show Keyboard Shortcuts"
3. Select the command
4. A dialog will display all keyboard shortcuts

## Accessing Commands

### Via Command Palette

1. Press `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (macOS)
2. Type "RiceCoder" to filter commands
3. Select the desired command
4. Press Enter to execute

### Via Keyboard Shortcuts

Use the keyboard shortcuts listed above for quick access to common operations.

### Via Context Menu

Right-click in the editor to access RiceCoder commands:

- **Chat with RiceCoder** - Always available
- **Review Code** - Always available
- **Generate Code** - Always available
- **Refactor Code** - Only available when text is selected

### Via Command Execution

You can also execute commands programmatically:

```typescript
await vscode.commands.executeCommand('ricecoder.chat');
await vscode.commands.executeCommand('ricecoder.review');
await vscode.commands.executeCommand('ricecoder.generate');
await vscode.commands.executeCommand('ricecoder.refactor');
```

## Configuration

RiceCoder commands respect VS Code settings:

- `ricecoder.enabled` - Enable/disable the extension
- `ricecoder.serverHost` - RiceCoder server host
- `ricecoder.serverPort` - RiceCoder server port
- `ricecoder.requestTimeout` - Request timeout in milliseconds

## Keyboard Shortcut Reference

| Command | Windows/Linux | macOS |
|---------|---------------|-------|
| Chat | Ctrl+Shift+R | Cmd+Shift+R |
| Review | Ctrl+Shift+E | Cmd+Shift+E |
| Generate | Ctrl+Shift+G | Cmd+Shift+G |
| Refactor | Ctrl+Shift+F | Cmd+Shift+F |

## Customizing Keyboard Shortcuts

To customize keyboard shortcuts:

1. Open VS Code settings (`Ctrl+,` / `Cmd+,`)
2. Search for "Keyboard Shortcuts"
3. Click "Open Keyboard Shortcuts (JSON)"
4. Add or modify keybindings:

```json
{
  "key": "ctrl+alt+r",
  "command": "ricecoder.chat",
  "when": "editorTextFocus"
}
```

## Context Conditions

Commands are available based on editor context:

- `editorTextFocus` - Editor has focus
- `editorHasSelection` - Text is selected in editor

For example, the Refactor command only appears in the context menu when text is selected.

## Troubleshooting

### Commands not appearing in command palette

1. Ensure RiceCoder extension is enabled
2. Check that `ricecoder.enabled` is set to `true` in settings
3. Reload VS Code window (`Ctrl+Shift+P` → "Developer: Reload Window")

### Keyboard shortcuts not working

1. Check for conflicting keybindings
2. Ensure editor has focus
3. Verify keyboard shortcut in settings

### Commands not responding

1. Check that RiceCoder server is running
2. Verify server host and port in settings
3. Check VS Code output panel for errors

## Implementation Details

### Command Registration

Commands are registered in `package.json` under `contributes.commands`:

```json
{
  "command": "ricecoder.chat",
  "title": "Chat with RiceCoder",
  "category": "RiceCoder",
  "description": "Open chat interface to ask RiceCoder questions about your code"
}
```

### Keyboard Shortcuts

Keyboard shortcuts are defined in `package.json` under `contributes.keybindings`:

```json
{
  "command": "ricecoder.chat",
  "key": "ctrl+shift+r",
  "mac": "cmd+shift+r",
  "when": "editorTextFocus"
}
```

### Context Menu Integration

Context menu items are defined in `package.json` under `contributes.menus.editor/context`:

```json
{
  "command": "ricecoder.chat",
  "group": "ricecoder@1",
  "when": "editorTextFocus"
}
```

### Command Implementation

Commands are implemented in `src/commands/commandHandler.ts`:

```typescript
export class CommandHandler {
  registerCommands(context: vscode.ExtensionContext): void {
    const chatCommand = vscode.commands.registerCommand('ricecoder.chat', async () => {
      await this.handleChatCommand();
    });
    context.subscriptions.push(chatCommand);
  }
}
```

## See Also

- [VS Code Command Palette Documentation](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette)
- [VS Code Keybindings Documentation](https://code.visualstudio.com/docs/getstarted/keybindings)
- [VS Code Extension API - Commands](https://code.visualstudio.com/api/references/commands)
- [RiceCoder Extension README](./README.md)
- [Communication Protocol](./PROTOCOL.md)
