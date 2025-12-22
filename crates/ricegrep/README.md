# RiceGrep Quick Start

RiceGrep is a modern, AI-enhanced search tool that combines ripgrep compatibility with advanced features like MCP integration, assistant plugins, and production observability.

## Installation

```bash
git clone https://github.com/moabualruz/ricecoder.git
cd ricecoder
cargo build --release --bin ricegrep
```

## Basic Usage

### Search Files
```bash
# Local search (requires an index)
ricegrep index build src/
ricegrep search "fn main" src/

# Ripgrep-compatible mode
ricegrep "fn main" src/

# Server mode (disabled by default; requires --server and build feature)
ricegrep search --server --endpoint http://localhost:3000 "async fn" src/
```

### File Operations
```bash
# Find files by name
ricegrep files "*.rs"

# Replace text safely
ricegrep replace "old_fn" "new_fn" src/main.rs --dry-run
```

### Indexing and Watching
```bash
# Build search index
ricegrep index build .

# Watch for changes
ricegrep watch src/
```

### Assistant Integration
```bash
# Install plugins
ricegrep install claude-code
ricegrep install opencode@v1.0

# Install with isolated config root (for testing)
ricegrep install opencode --force --config-root ./tmp-config

# Uninstall
ricegrep uninstall claude-code
```

### MCP Server
```bash
# Start MCP server for AI assistants
ricegrep mcp
```

### Health and Diagnostics
```bash
# Check server status (requires --server and server feature)
ricegrep health --server
```

## Configuration

Create `.ricegrep.toml` in your project:

```toml
endpoint = "http://localhost:3000"
json = false
quiet = false
```

Server mode is disabled by default; the endpoint is only used when `--server` is provided
and the binary is built with the `server` feature.

Or use environment variables:
```bash
export RICEGREP_ENDPOINT="https://my-server.com"
export RICEGREP_JSON=true
```

`RICEGREP_ENDPOINT` is only consulted when `--server` is enabled.

**Note**: GPU acceleration and other advanced features are planned for future releases. See the wiki for details.

## Common Workflows

### Development Setup
```bash
# Index your codebase
ricegrep index build .

# Search with context
ricegrep search "TODO" --before-context 2 --after-context 2

# Watch for changes during development
ricegrep watch src/
```

### AI Assistant Integration
```bash
# Install your preferred assistant
ricegrep install claude-code

# Export skills for documentation
ricegrep export-skills --format json
```

### Production Monitoring
```bash
# Check server health (requires --server and server feature)
ricegrep health --server --json

# Monitor with JSON output (local index)
ricegrep search "error" --json | jq '.results | length'
```

### System Testing
```bash
# Run local end-to-end checks with JSON report
ricegrep e2e --output ricegrep-e2e-report.json
```

## Next Steps

- **Full Documentation**: See `projects/ricecoder.wiki/RiceGrep.md` for comprehensive guides
- **Command Help**: Run `ricegrep --help` or `ricegrep <command> --help`
- **Planned Features**: Check the active spec for upcoming capabilities
- **Contributing**: See `CONTRIBUTING.md` for development guidelines