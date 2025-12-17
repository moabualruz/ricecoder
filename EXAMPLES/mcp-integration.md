# MCP Integration Examples

This directory contains examples of integrating RiceCoder with Model Context Protocol (MCP) servers for enhanced functionality.

## MCP Overview

The Model Context Protocol allows RiceCoder to connect to external tools and services, extending its capabilities beyond built-in features.

## Setting up MCP Servers

### Basic MCP Configuration

```yaml
# config/mcp-servers.yaml or ~/.ricecoder/mcp-servers.yaml
mcpServers:
  filesystem:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
    env:
      NODE_ENV: "production"

  git:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-git", "--repository", "/workspace"]
    env:
      GIT_AUTHOR_NAME: "RiceCoder"
      GIT_AUTHOR_EMAIL: "ricecoder@local"

  sqlite:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "/workspace/data.db"]
```

### Installing MCP Servers

```bash
# Install filesystem server globally
npm install -g @modelcontextprotocol/server-filesystem

# Install git server globally
npm install -g @modelcontextprotocol/server-git

# Install SQLite server globally
npm install -g @modelcontextprotocol/server-sqlite

# Or use npx for on-demand installation (recommended)
npx -y @modelcontextprotocol/server-filesystem --help
```

## Using MCP Tools

### File System Operations

```bash
# Start chat with MCP enabled
rice chat --mcp

# List files in current directory (in chat)
list_files

# Read a specific file
read_file README.md

# Search for files
search_files "*.rs" "fn main"

# Create a new file
create_file example.txt "Hello, World!"

# Edit an existing file
edit_file example.txt "Hello, RiceCoder World!"
```

### Git Operations

```bash
# Check git status
git_status

# View commit history
git_log --limit 10

# Show diff of working directory
git_diff

# Create a commit
git_commit -m "Add new feature" --files "src/new_feature.rs"

# View branches
git_branches
```

### Database Operations

```bash
# Connect to SQLite database
# (Database file specified in MCP server config)

# Execute a query
run_query "SELECT * FROM users LIMIT 5"

# Get table schema
get_schema "users"

# Insert data
run_query "INSERT INTO users (name, email) VALUES ('John', 'john@example.com')"

# Update records
run_query "UPDATE users SET active = 1 WHERE id = 1"
```

## MCP Server Management

### Managing MCP Servers via CLI

```bash
# List configured MCP servers
rice mcp list

# Add a new MCP server
rice mcp add filesystem \
  --command npx \
  --args "-y,@modelcontextprotocol/server-filesystem,/workspace"

# Start all MCP servers
rice mcp start

# Start specific server
rice mcp start filesystem

# Check server status
rice mcp status

# Stop all servers
rice mcp stop

# Stop specific server
rice mcp stop git

# Restart servers
rice mcp restart

# Remove a server
rice mcp remove sqlite
```

### Custom MCP Server Configuration

```yaml
# Advanced MCP server configuration
mcpServers:
  custom-api:
    command: "python3"
    args: ["custom_server.py", "--port", "3001"]
    env:
      API_KEY: "${API_KEY}"
      DATABASE_URL: "${DATABASE_URL}"
    cwd: "/path/to/server"
    timeout: 30
    retry:
      attempts: 3
      delay: 1000

  weather-service:
    command: "node"
    args: ["weather-server.js"]
    env:
      WEATHER_API_KEY: "${WEATHER_API_KEY}"
    health_check:
      endpoint: "http://localhost:3002/health"
      interval: 30
```

## MCP in Code Generation

### Using MCP Tools in Generation

```bash
# Generate code with file system access
rice gen --spec api-design.spec.md --mcp

# In the spec file, you can reference MCP tools:
# "Read the existing API schema from schema.json"
# "Check the current database structure"
# "Review the authentication implementation in auth.py"
```

### Database-Driven Code Generation

```bash
# Generate models from database schema
rice gen --spec database-models.spec.md --mcp

# Spec content example:
# Generate Rust structs for all tables in the 'users' database
# Use the following query to get table information:
# run_query "PRAGMA table_info(users)"
```

### File-Based Code Generation

```bash
# Generate code based on existing files
rice gen --spec refactor-legacy.spec.md --mcp

# Spec example:
# Refactor the legacy authentication code in auth.js
# Read the current implementation: read_file "src/auth.js"
# Generate modern authentication with JWT
# Update the file with the new implementation
```

## Advanced MCP Workflows

### Multi-Server Orchestration

```yaml
# config/mcp-servers.yaml
mcpServers:
  primary-fs:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]

  backup-fs:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/backup"]

  database:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "/workspace/app.db"]

  git:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-git", "--repository", "/workspace"]
```

### Workflow Automation

```bash
# Automated backup workflow
rice workflow run backup-workflow

# Workflow definition (in workflow file):
# 1. Check git status
# 2. Create backup of current state
# 3. Run database backup
# 4. Commit changes
# 5. Push to remote
```

### MCP Tool Chaining

```bash
# Chain multiple MCP operations
rice chat --mcp

# In chat:
# 1. First, check what files exist: list_files
# 2. Read the configuration: read_file "config.yaml"
# 3. Check database schema: run_query "PRAGMA table_info(users)"
# 4. Generate code based on both: "Create a user API endpoint that matches the database schema"
```

## MCP Security and Permissions

### Tool Permissions

```yaml
# config/mcp-permissions.yaml
permissions:
  filesystem:
    read: ["*.md", "*.txt", "*.json"]
    write: ["*.md", "*.txt"]
    deny: ["*.key", "*.pem", "secrets/*"]

  database:
    read: ["users", "posts"]
    write: ["user_sessions"]
    deny: ["admin_data", "sensitive_info"]

  git:
    allow: ["status", "log", "diff"]
    deny: ["push", "force-push"]
```

### Enterprise MCP Configuration

```yaml
# Enterprise-grade MCP setup
mcpServers:
  corporate-api:
    command: "corporate-api-server"
    args: ["--config", "/etc/corporate-api/config.json"]
    env:
      API_KEY: "${CORPORATE_API_KEY}"
      TLS_CERT: "/etc/ssl/certs/corporate.crt"
      TLS_KEY: "/etc/ssl/private/corporate.key"
    auth:
      type: "oauth2"
      client_id: "${OAUTH_CLIENT_ID}"
      client_secret: "${OAUTH_CLIENT_SECRET}"
      token_url: "https://auth.corporate.com/oauth2/token"
    monitoring:
      enabled: true
      metrics_endpoint: "http://localhost:9090/metrics"
      health_endpoint: "http://localhost:9090/health"
```

## Troubleshooting MCP

### Common MCP Issues

```bash
# Check MCP server health
rice mcp health

# View MCP server logs
rice mcp logs filesystem

# Test MCP tool execution
rice mcp test filesystem list_files

# Debug MCP connections
rice mcp debug

# Reset MCP configuration
rice mcp reset
```

### MCP Performance Tuning

```yaml
# config/mcp-performance.yaml
performance:
  connection_pool:
    max_connections: 10
    idle_timeout: 300

  caching:
    enabled: true
    ttl_seconds: 300
    max_size_mb: 100

  timeouts:
    connect: 5000
    read: 10000
    write: 5000

  retry:
    max_attempts: 3
    backoff_multiplier: 2.0
```

### Monitoring MCP Usage

```bash
# Monitor MCP server performance
rice mcp monitor

# View MCP usage statistics
rice mcp stats

# Generate MCP performance report
rice mcp report --output mcp-performance.md

# Alert on MCP server failures
rice mcp alerts enable
```

## Custom MCP Server Development

### Creating a Custom MCP Server

```python
# custom_mcp_server.py
import asyncio
from mcp import Server, Tool
from mcp.types import TextContent, PromptMessage

server = Server("custom-server")

@server.tool()
async def custom_tool(query: str) -> str:
    """A custom tool that processes queries."""
    # Your custom logic here
    return f"Processed: {query}"

@server.prompt()
async def custom_prompt(template: str) -> list[PromptMessage]:
    """A custom prompt template."""
    return [
        PromptMessage(
            role="user",
            content=TextContent(
                type="text",
                text=f"Using template: {template}"
            )
        )
    ]

async def main():
    async with server:
        await server.serve()

if __name__ == "__main__":
    asyncio.run(main())
```

### Integrating Custom Server with RiceCoder

```yaml
# config/mcp-servers.yaml
mcpServers:
  custom:
    command: "python3"
    args: ["custom_mcp_server.py"]
    env:
      CUSTOM_CONFIG: "/path/to/config.json"
```

This comprehensive guide covers MCP integration from basic setup to advanced enterprise configurations. MCP extends RiceCoder's capabilities by allowing seamless integration with external tools and services.