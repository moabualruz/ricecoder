# ricecoder-tools

**Purpose**: Enhanced tools collection providing webfetch, patch, todo, and web search capabilities with hybrid MCP provider architecture for RiceCoder

## Overview

`ricecoder-tools` implements a comprehensive tool ecosystem with webfetch, patch, todo management, and web search capabilities. It uses a hybrid MCP (Model Context Protocol) provider architecture that provides built-in implementations as fallbacks while allowing advanced customization through external MCP servers.

## Features

- **Webfetch Tool**: Fetch web content with timeout and truncation support
- **Patch Tool**: Apply unified diff patches with conflict detection
- **Todo Tools**: Persistent task management with status tracking
- **Web Search Tool**: Search the web using free APIs or local MCP servers
- **Hybrid Architecture**: Built-in implementations with MCP server extensibility
- **Provider Registry**: Pluggable provider system for tool implementations
- **Error Handling**: Comprehensive error types with context and suggestions
- **Result Metadata**: Rich result types with execution metadata

## Architecture

### Responsibilities
- Tool execution orchestration and provider management
- MCP server integration and fallback handling
- Web content fetching and processing
- Patch application and conflict resolution
- Task management and persistence
- Search result processing and ranking
- Provider registration and discovery

### Dependencies
- **HTTP Client**: `reqwest` for web operations
- **Async Runtime**: `tokio` for concurrent tool execution
- **Serialization**: `serde` for result and configuration handling
- **MCP Integration**: Protocol handling for external servers

### Integration Points
- **Commands**: CLI access to all tools
- **Sessions**: Tool results integrated into conversations
- **TUI**: Tool execution and result display
- **Workflows**: Tools used in automated workflows
- **Storage**: Tool results and configuration persistence

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-tools = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_tools::provider::ProviderRegistry;

// Create provider registry
let registry = ProviderRegistry::new();

// Execute a tool
let result = registry.execute_tool("webfetch", "https://example.com").await?;
println!("Content: {}", result.content);
```

### Webfetch Tool

```rust
use ricecoder_tools::webfetch::WebfetchTool;

// Create webfetch tool
let webfetch = WebfetchTool::new();

// Fetch web content with timeout
let result = webfetch.fetch("https://api.example.com/data", Some(5000)).await?;
println!("Status: {}", result.status);
println!("Content length: {}", result.content.len());
```

### Patch Tool

```rust
use ricecoder_tools::patch::PatchTool;

// Create patch tool
let patch_tool = PatchTool::new();

// Apply unified diff
let success = patch_tool.apply_patch("file.txt", &patch_content).await?;
if success {
    println!("Patch applied successfully");
} else {
    println!("Patch conflicts detected");
}
```

### Todo Tools

```rust
use ricecoder_tools::todo::{TodoManager, TodoItem};

// Create todo manager
let todo_manager = TodoManager::new();

// Add a task
let task = TodoItem {
    title: "Implement feature".to_string(),
    description: "Implement the new feature".to_string(),
    priority: Priority::High,
    ..Default::default()
};

todo_manager.add_task(task).await?;
```

### Web Search Tool

```rust
use ricecoder_tools::search::WebSearchTool;

// Create search tool
let search_tool = WebSearchTool::new();

// Perform web search
let results = search_tool.search("rust async programming", Some(10)).await?;
for result in results {
    println!("{}: {}", result.title, result.url);
}
```

## Configuration

Tool configuration via YAML:

```yaml
tools:
  # Webfetch settings
  webfetch:
    timeout_ms: 10000
    max_content_length: 1048576  # 1MB
    user_agent: "RiceCoder/1.0"
    follow_redirects: true

  # Patch settings
  patch:
    backup_original: true
    conflict_resolution: "manual"
    validate_patch: true

  # Todo settings
  todo:
    storage_path: "~/.ricecoder/todos"
    max_tasks: 1000
    auto_archive_completed: true

  # Search settings
  search:
    default_provider: "duckduckgo"
    max_results: 20
    timeout_ms: 5000
    providers:
      - name: "duckduckgo"
        api_url: "https://api.duckduckgo.com/"
      - name: "custom"
        api_url: "${CUSTOM_SEARCH_API}"

  # MCP settings
  mcp:
    enabled: true
    servers:
      - name: "advanced-webfetch"
        url: "http://localhost:3001"
        tools: ["webfetch"]
```

## API Reference

### Key Types

- **`ProviderRegistry`**: Central tool provider management
- **`WebfetchTool`**: Web content fetching with timeout support
- **`PatchTool`**: Unified diff patch application
- **`TodoManager`**: Task management and tracking
- **`WebSearchTool`**: Web search with multiple providers

### Key Functions

- **`execute_tool()`**: Execute any tool by name
- **`fetch()`**: Fetch web content with options
- **`apply_patch()`**: Apply patch to file with conflict detection
- **`search()`**: Perform web search with result ranking

## Error Handling

```rust
use ricecoder_tools::ToolError;

match registry.execute_tool("webfetch", "https://example.com").await {
    Ok(result) => println!("Success: {} bytes", result.content.len()),
    Err(ToolError::NetworkError(msg)) => eprintln!("Network error: {}", msg),
    Err(ToolError::TimeoutError) => eprintln!("Request timed out"),
    Err(ToolError::McpServerUnavailable) => eprintln!("MCP server not available, using built-in"),
}
```

## Testing

Run comprehensive tool tests:

```bash
# Run all tests
cargo test -p ricecoder-tools

# Run property tests for tool behavior
cargo test -p ricecoder-tools property

# Test webfetch functionality
cargo test -p ricecoder-tools webfetch

# Test MCP integration
cargo test -p ricecoder-tools mcp
```

Key test areas:
- Tool execution correctness
- MCP server fallback behavior
- Network error handling
- Patch conflict detection
- Search result validation

## Performance

- **Webfetch**: < 500ms for typical web pages (< 100KB)
- **Patch Application**: < 50ms for typical patches
- **Todo Operations**: < 10ms for task management
- **Web Search**: < 2s for search with 10-20 results
- **MCP Fallback**: Seamless fallback with < 100ms detection

## Contributing

When working with `ricecoder-tools`:

1. **Hybrid Design**: Maintain MCP-optional architecture with robust fallbacks
2. **Error Handling**: Provide clear error messages and recovery suggestions
3. **Performance**: Optimize for common use cases while handling edge cases
4. **Testing**: Test both MCP and built-in code paths
5. **Documentation**: Keep tool usage examples current and comprehensive

## License

MIT