# RiceCoder Usage Examples

This directory contains comprehensive usage examples demonstrating common RiceCoder workflows and integrations.

## Table of Contents

- [CLI Usage Examples](#cli-usage-examples)
- [MCP Integration Examples](#mcp-integration-examples)
- [Session Sharing Examples](#session-sharing-examples)
- [Provider Configuration Examples](#provider-configuration-examples)
- [Advanced Workflows](#advanced-workflows)

## CLI Usage Examples

### Basic Chat Session

```bash
# Start an interactive chat session
rice chat

# Chat with a specific provider
rice chat --provider openai

# Chat with a specific model
rice chat --model gpt-4

# Start chat with a system prompt
rice chat --system "You are a helpful coding assistant specialized in Rust."
```

### Code Generation from Specs

```bash
# Generate code from a specification file
rice gen --spec my-feature.spec.md

# Generate with specific provider
rice gen --spec api-endpoint.spec.md --provider anthropic

# Generate and apply changes directly
rice gen --spec database-model.spec.md --apply

# Generate with custom context
rice gen --spec auth-system.spec.md --context "Use JWT tokens and bcrypt hashing"
```

### Code Review

```bash
# Review a single file
rice review src/main.rs

# Review multiple files
rice review src/*.rs

# Review with specific criteria
rice review --criteria "security,performance" src/auth.rs

# Review a pull request (if integrated with GitHub)
rice review --pr 123
```

### Project Analysis

```bash
# Analyze current project
rice analyze

# Analyze with specific focus
rice analyze --focus security

# Analyze dependencies
rice analyze --deps

# Generate project report
rice analyze --report project-report.md
```

## MCP Integration Examples

### Setting up MCP Servers

```yaml
# mcp-servers.yaml
mcpServers:
  filesystem:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    env:
      NODE_ENV: "production"

  git:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-git", "--repository", "."]
    env:
      GIT_AUTHOR_NAME: "RiceCoder"
      GIT_AUTHOR_EMAIL: "ricecoder@example.com"

  sqlite:
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "./data.db"]
```

### Using MCP Tools in Chat

```bash
# Start chat with MCP tools enabled
rice chat --mcp

# List available MCP tools
rice mcp list

# Execute MCP tool directly
rice mcp exec filesystem read_file --path README.md

# Use MCP tools in generation
rice gen --spec database-query.spec.md --mcp
```

### MCP Server Management

```bash
# Add a new MCP server
rice mcp add my-server --command "python" --args "server.py"

# Start MCP servers
rice mcp start

# Check MCP server status
rice mcp status

# Stop specific MCP server
rice mcp stop filesystem

# Restart all MCP servers
rice mcp restart
```

## Session Sharing Examples

### Creating Shareable Sessions

```bash
# Start a session and make it shareable
rice chat --share

# Create a session with expiration
rice chat --share --expire 24h

# Share with specific permissions
rice chat --share --read-only

# Generate shareable link for existing session
rice session share my-session-id
```

### Accessing Shared Sessions

```bash
# Access a shared session via URL
rice session join https://ricecoder.app/s/abc123

# Join session with authentication
rice session join https://ricecoder.app/s/abc123 --token my-token

# View session metadata
rice session info abc123

# List accessible shared sessions
rice session list-shared
```

### Session Management

```bash
# List all sessions
rice session list

# Switch to a different session
rice session switch session-uuid

# Export session data
rice session export my-session --output session.json

# Import session data
rice session import session.json

# Delete old sessions
rice session cleanup --older-than 30d
```

## Provider Configuration Examples

### OpenAI Configuration

```yaml
# config/providers.yaml
providers:
  openai:
    api_key: "sk-..."
    models:
      - gpt-4
      - gpt-3.5-turbo
    settings:
      temperature: 0.7
      max_tokens: 4000
```

```bash
# Configure OpenAI provider
rice provider config openai --api-key sk-...

# Test OpenAI connection
rice provider test openai

# Set as default provider
rice provider default openai
```

### Anthropic Configuration

```yaml
# config/providers.yaml
providers:
  anthropic:
    api_key: "sk-ant-..."
    models:
      - claude-3-opus-20240229
      - claude-3-sonnet-20240229
    settings:
      temperature: 0.7
      max_tokens: 4000
```

```bash
# Configure Anthropic provider
rice provider config anthropic --api-key sk-ant-...

# Use Claude for code generation
rice gen --spec my-spec.md --provider anthropic --model claude-3-opus-20240229
```

### Ollama Local Models

```yaml
# config/providers.yaml
providers:
  ollama:
    base_url: "http://localhost:11434"
    models:
      - llama2:13b
      - codellama:34b
    settings:
      temperature: 0.1
      num_ctx: 4096
```

```bash
# Start Ollama server
ollama serve

# Pull a model
ollama pull llama2:13b

# Configure RiceCoder to use Ollama
rice provider config ollama --base-url http://localhost:11434

# Chat with local model
rice chat --provider ollama --model llama2:13b
```

### Multi-Provider Setup

```yaml
# config/providers.yaml
providers:
  primary: openai
  fallback:
    - anthropic
    - ollama
  routing:
    code: openai  # Use OpenAI for code tasks
    chat: anthropic  # Use Anthropic for chat
    analysis: ollama  # Use local models for analysis
```

```bash
# Set up provider routing
rice provider route code openai
rice provider route chat anthropic
rice provider route analysis ollama

# Enable automatic failover
rice provider failover enable

# Monitor provider performance
rice provider monitor
```

## Advanced Workflows

### Spec-Driven Development

```bash
# Create a specification
cat > user-auth.spec.md << 'EOF'
# User Authentication System

## Requirements
- User registration with email verification
- JWT-based authentication
- Password hashing with bcrypt
- Role-based access control

## API Endpoints
- POST /auth/register
- POST /auth/login
- GET /auth/me
- POST /auth/logout

## Security Requirements
- Password strength validation
- Rate limiting on auth endpoints
- Secure token storage
EOF

# Generate implementation
rice gen --spec user-auth.spec.md --apply

# Review generated code
rice review src/auth/

# Run tests
rice test auth
```

### Team Collaboration

```bash
# Create a team workspace
rice team create my-team

# Invite team members
rice team invite user@example.com

# Share a session with the team
rice session share --team my-team --read-only

# Review team activity
rice team activity

# Set up team permissions
rice team permissions set user@example.com --role reviewer
```

### CI/CD Integration

```yaml
# .github/workflows/ricecoder.yml
name: RiceCoder Code Review
on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install RiceCoder
        run: curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash
      - name: Run Code Review
        run: rice review --pr ${{ github.event.pull_request.number }} --output review.md
      - name: Comment PR
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const review = fs.readFileSync('review.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: review
            });
```

### Performance Optimization

```bash
# Run performance benchmarks
rice benchmark run

# Profile a specific operation
rice profile chat --duration 60s

# Optimize provider settings
rice provider optimize openai --metric latency

# Monitor resource usage
rice monitor resources

# Generate performance report
rice report performance --output perf-report.md
```

## Configuration Examples

### Project Configuration

```toml
# .agent/ricecoder.toml
[project]
name = "my-rust-app"
language = "rust"
framework = "axum"

[providers]
default = "openai"
fallback = ["anthropic", "ollama"]

[mcp]
enabled = true
auto_start = true

[sessions]
max_concurrent = 5
persistence = true
sharing = true
```

### Global Configuration

```yaml
# ~/.ricecoder/config.yaml
user:
  name: "John Doe"
  email: "john@example.com"

providers:
  openai:
    api_key: "${OPENAI_API_KEY}"
    models: ["gpt-4", "gpt-3.5-turbo"]

  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
    models: ["claude-3-opus-20240229"]

mcp:
  servers:
    filesystem: true
    git: true

ui:
  theme: "dark"
  syntax_highlighting: true
  animations: true
```

## Troubleshooting Examples

### Common Issues

```bash
# Check RiceCoder status
rice doctor

# Validate configuration
rice config validate

# Test provider connections
rice provider test all

# Check MCP server health
rice mcp health

# View logs
rice logs --tail 100

# Reset configuration
rice config reset
```

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
rice chat

# Run with verbose output
rice --verbose gen --spec my-spec.md

# Debug MCP connections
rice mcp debug

# Profile performance issues
rice profile --output profile.json chat
```

This collection of examples covers the most common RiceCoder use cases. For more detailed documentation, see the [RiceCoder Wiki](https://github.com/moabualruz/ricecoder/wiki).