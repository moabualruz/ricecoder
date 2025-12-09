# RiceCoder VS Code Extension

AI-powered code assistance integrated directly into VS Code.

## Features

- **Code Completion**: Intelligent code suggestions powered by external LSP servers
- **Diagnostics**: Real-time code analysis and error detection
- **Hover Information**: Detailed information about symbols and types
- **Command Palette Integration**: Quick access to RiceCoder features
- **Chat Interface**: Ask RiceCoder questions about your code
- **Code Review**: Get AI-powered code reviews
- **Code Generation**: Generate code from descriptions
- **Refactoring**: Automated code refactoring

## Installation

1. Install the extension from the VS Code Marketplace
2. Configure RiceCoder server connection in settings
3. Reload VS Code

## Configuration

Configure RiceCoder in VS Code settings:

```json
{
  "ricecoder.enabled": true,
  "ricecoder.serverHost": "localhost",
  "ricecoder.serverPort": 9000,
  "ricecoder.requestTimeout": 5000,
  "ricecoder.completionEnabled": true,
  "ricecoder.diagnosticsEnabled": true,
  "ricecoder.hoverEnabled": true,
  "ricecoder.providerSelection": "lsp-first"
}
```

## Commands

- `ricecoder.chat` - Open chat interface (Ctrl+Shift+R / Cmd+Shift+R)
- `ricecoder.review` - Review code
- `ricecoder.generate` - Generate code
- `ricecoder.refactor` - Refactor code

## Development

### Prerequisites

- Node.js 16+
- npm or yarn

### Setup

```bash
npm install
npm run compile
```

### Build

```bash
npm run compile
```

### Watch Mode

```bash
npm run watch
```

### Lint

```bash
npm run lint
```

### Test

```bash
npm test
```

## Architecture

The extension communicates with the RiceCoder backend using JSON-RPC over TCP:

1. **Extension** (VS Code) ↔ **JSON-RPC** ↔ **RiceCoder Backend**

### Components

- **RicecoderClient**: JSON-RPC client for backend communication
- **CompletionProvider**: VS Code completion provider
- **DiagnosticsProvider**: VS Code diagnostics provider
- **HoverProvider**: VS Code hover provider
- **CommandHandler**: Command palette command handlers

## Error Handling

The extension handles various error scenarios:

- Connection failures: Automatic reconnection with exponential backoff
- Request timeouts: Configurable timeout with fallback
- Server errors: Clear error messages to the user
- Configuration errors: Validation with remediation steps

## Performance

- Debounced diagnostics updates (500ms)
- Request timeouts to prevent hanging
- Efficient JSON-RPC message handling
- Minimal memory footprint

## License

MIT
