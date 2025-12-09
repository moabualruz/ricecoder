# VS Code Communication Protocol

This document describes the JSON-RPC communication protocol between the VS Code extension and the RiceCoder backend.

## Overview

The VS Code extension communicates with the RiceCoder backend using JSON-RPC 2.0 protocol over TCP sockets. The protocol supports:

1. **Request/Response**: Synchronous request-response pairs with timeouts
2. **Streaming**: Long-running operations that stream results back to the client
3. **Notifications**: One-way messages from server to client

## Connection

### Establishing Connection

```typescript
const client = new RicecoderClient('localhost', 9000, 5000);
await client.connect();
```

### Configuration

- **Host**: Server hostname (default: `localhost`)
- **Port**: Server port (default: `9000`)
- **Request Timeout**: Timeout for requests in milliseconds (default: `5000`)

### Disconnecting

```typescript
await client.disconnect();
```

## Request/Response Protocol

### Format

All requests and responses follow JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "completion/provide",
  "params": {
    "language": "rust",
    "file_path": "/path/to/file.rs",
    "position": { "line": 10, "character": 5 },
    "context": "fn main() {"
  }
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [
    {
      "label": "println!",
      "kind": "Macro",
      "insert_text": "println!(\"$1\");",
      "documentation": "Print to stdout"
    }
  ]
}
```

### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "Missing required field: language"
  }
}
```

## IDE Feature Requests

### Completion

**Method**: `completion/provide`

**Parameters**:
```typescript
{
  language: string;           // Programming language (e.g., "rust", "typescript")
  file_path: string;          // Absolute path to the file
  position: Position;         // Cursor position { line, character }
  context: string;            // Line context up to cursor
  trigger_character?: string; // Character that triggered completion (e.g., ".")
}
```

**Response**:
```typescript
[
  {
    label: string;              // Display text
    kind: string;               // Completion kind (Function, Variable, etc.)
    detail?: string;            // Additional details
    documentation?: string;     // Markdown documentation
    insert_text: string;        // Text to insert
    sort_text?: string;         // Sort key
    filter_text?: string;       // Filter key
  }
]
```

### Diagnostics

**Method**: `diagnostics/provide`

**Parameters**:
```typescript
{
  language: string;   // Programming language
  file_path: string;  // Absolute path to the file
  source: string;     // Full file content
}
```

**Response**:
```typescript
[
  {
    range: Range;                    // Error location
    severity: "Error" | "Warning" | "Information" | "Hint";
    message: string;                 // Error message
    source: string;                  // Source (e.g., "RiceCoder")
    code?: string;                   // Error code
    quick_fixes?: QuickFix[];        // Available fixes
  }
]
```

### Hover

**Method**: `hover/provide`

**Parameters**:
```typescript
{
  language: string;   // Programming language
  file_path: string;  // Absolute path to the file
  position: Position; // Cursor position
  word: string;       // Word at cursor
}
```

**Response**:
```typescript
{
  contents: string;           // Main hover content (markdown)
  type_info?: string;         // Type information
  documentation?: string;     // Documentation (markdown)
  source?: string;            // Source location
  range?: Range;              // Hover range
}
```

### Definition

**Method**: `definition/provide`

**Parameters**:
```typescript
{
  language: string;   // Programming language
  file_path: string;  // Absolute path to the file
  position: Position; // Cursor position
}
```

**Response**:
```typescript
{
  file_path: string;  // File containing definition
  range: Range;       // Definition location
}
```

## Command Requests

### Chat

**Method**: `chat/send`

**Parameters**:
```typescript
{
  message: string;    // User message
  context: string;    // Code context
  language: string;   // Programming language
  file_path: string;  // Current file path
}
```

**Response**:
```typescript
{
  response: string;   // Chat response
  stream_id?: string; // Stream ID if streaming
}
```

### Code Review

**Method**: `review/code`

**Parameters**:
```typescript
{
  language: string;   // Programming language
  file_path: string;  // File path
  source: string;     // Code to review
}
```

**Response**:
```typescript
{
  review: string;     // Review text
  issues: ReviewIssue[];
}
```

### Code Generation

**Method**: `generate/code`

**Parameters**:
```typescript
{
  prompt: string;     // Generation prompt
  language: string;   // Programming language
  file_path: string;  // Current file path
  context: string;    // Code context
}
```

**Response**:
```typescript
{
  code: string;           // Generated code
  explanation?: string;   // Explanation
}
```

### Refactoring

**Method**: `refactor/code`

**Parameters**:
```typescript
{
  refactoring_type: string; // Type of refactoring
  language: string;         // Programming language
  file_path: string;        // File path
  source: string;           // Code to refactor
}
```

**Response**:
```typescript
{
  refactored: string;  // Refactored code
  changes: Change[];   // List of changes
}
```

## Streaming Protocol

### Starting a Stream

For long-running operations, use streaming:

```typescript
const streamId = await client.startStream('chat/send', {
  message: 'Generate a complex function',
  context: 'fn main() {}',
  language: 'rust',
  file_path: '/path/to/main.rs'
});

// Listen for chunks
client.onStreamChunk(streamId, (chunk) => {
  console.log('Received chunk:', chunk);
});

// Listen for completion
client.onStreamComplete(streamId, (chunks) => {
  console.log('Stream complete, total chunks:', chunks.length);
});

// Listen for errors
client.onStreamError(streamId, (error) => {
  console.error('Stream error:', error);
});
```

### Stream Notifications

The server sends stream notifications:

```json
{
  "jsonrpc": "2.0",
  "method": "stream/chunk",
  "params": {
    "stream_id": "stream_1",
    "chunk": "Generated code...",
    "index": 0,
    "total": 5
  }
}
```

### Cancelling a Stream

```typescript
await client.cancelStream(streamId);
```

## Notification Protocol

### Server Notifications

The server can send notifications without a request:

```json
{
  "jsonrpc": "2.0",
  "method": "config/changed",
  "params": {
    "setting": "providerSelection",
    "value": "lsp-first"
  }
}
```

### Handling Notifications

```typescript
client.on('notification', (data) => {
  console.log('Received notification:', data.method, data.params);
});
```

## Error Handling

### Error Codes

- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `1000`: Connection error
- `1001`: Timeout error
- `1002`: Stream error
- `1003`: Configuration error

### Handling Errors

```typescript
try {
  const result = await client.request('completion/provide', params);
} catch (error) {
  if (error instanceof Error) {
    console.error('Request failed:', error.message);
  }
}
```

## Message Format

### Position

```typescript
{
  line: number;      // 0-based line number
  character: number; // 0-based character position
}
```

### Range

```typescript
{
  start: Position;   // Start position
  end: Position;     // End position
}
```

## Configuration

### VS Code Settings

The extension reads configuration from VS Code settings:

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

## Examples

### Requesting Completions

```typescript
const completions = await client.request('completion/provide', {
  language: 'rust',
  file_path: '/path/to/main.rs',
  position: { line: 10, character: 5 },
  context: 'fn main() {',
  trigger_character: '.'
});
```

### Streaming Chat Response

```typescript
const streamId = await client.startStream('chat/send', {
  message: 'Explain this code',
  context: 'fn main() { println!("Hello"); }',
  language: 'rust',
  file_path: '/path/to/main.rs'
});

let fullResponse = '';
client.onStreamChunk(streamId, (chunk) => {
  fullResponse += chunk;
  console.log('Received:', chunk);
});

client.onStreamComplete(streamId, () => {
  console.log('Full response:', fullResponse);
});
```

### Handling Diagnostics

```typescript
const diagnostics = await client.request('diagnostics/provide', {
  language: 'rust',
  file_path: '/path/to/main.rs',
  source: 'fn main() { let x = ; }'
});

diagnostics.forEach(diag => {
  console.log(`${diag.severity}: ${diag.message} at line ${diag.range.start.line}`);
});
```

## Protocol Version

Current protocol version: **1.0.0**

## See Also

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [VS Code Extension API](https://code.visualstudio.com/api)
- [RiceCoder Backend Documentation](../../docs/)
