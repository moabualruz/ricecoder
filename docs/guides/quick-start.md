# Quick Start Guide

Get started with RiceCoder in 5 minutes. This guide covers installation, basic configuration, and your first AI-assisted coding session.

## Prerequisites

- **Operating System**: Linux, macOS, or Windows
- **Memory**: 2GB RAM minimum, 4GB recommended
- **Storage**: 500MB free space
- **Network**: Internet connection for AI providers

## 1. Installation

Choose your preferred installation method:

### Cargo (Recommended)

```bash
cargo install ricecoder
```

### Quick Install Script

```bash
# Linux/macOS
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash

# Windows PowerShell
irm https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1 | iex
```

### Verify Installation

```bash
rice --version
rice --help
```

## 2. Initial Configuration

### Configure AI Provider

RiceCoder supports multiple AI providers. Configure at least one:

```bash
# Set up OpenAI (requires API key)
rice config provider openai --api-key "your-openai-api-key"

# Or use Ollama for local models
rice config provider ollama --model "codellama:7b"
```

### Initialize Project

Navigate to your project and initialize RiceCoder:

```bash
cd your-project
rice init
```

This analyzes your codebase and creates a `.agent/ricecoder.toml` configuration file.

## 3. Your First Session

### Start Interactive Chat

```bash
rice chat
```

This opens RiceCoder's terminal UI for interactive coding assistance.

### Basic Commands

```
help                    # Show available commands
analyze                 # Analyze current project
generate <spec>         # Generate code from spec
review <file>           # Review code file
refactor <pattern>      # Refactor code
```

### Example Session

```
rice> analyze
Analyzing project structure...
Found 15 Rust files, 3 Python files
Project type: Mixed (Rust + Python)
Dependencies: 8 crates, 12 Python packages

rice> generate "Add user authentication API"
Generating authentication API...
Created: src/auth/mod.rs
Created: src/auth/models.rs
Created: src/auth/handlers.rs
Updated: Cargo.toml

rice> review src/auth/mod.rs
Reviewing src/auth/mod.rs...
✅ Code follows project patterns
✅ Error handling implemented
✅ Documentation added
⚠️  Consider adding unit tests
```

## 4. Spec-Driven Development

### Create a Specification

Create a `.spec.md` file:

```markdown
# User Registration API

## Requirements

- User registration endpoint
- Email validation
- Password hashing
- JWT token generation

## API Endpoints

- `POST /api/register` - Register new user
- `POST /api/login` - User login

## Data Models

```rust
struct User {
    id: Uuid,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}
```
```

### Generate from Spec

```bash
rice generate user-registration-api.spec.md
```

## 5. Advanced Features

### Multi-Session Support

```bash
# Start new session
rice session new "feature-auth"

# Switch sessions
rice session switch feature-auth

# List sessions
rice session list
```

### Provider Switching

```bash
# Switch to different provider
rice provider switch anthropic

# Check current provider
rice provider status
```

### Performance Monitoring

```bash
# Check token usage
rice stats tokens

# Performance metrics
rice stats performance
```

## 6. Troubleshooting

### Common Issues

**"Command not found: rice"**
```bash
# Check PATH
echo $PATH

# Restart terminal or source profile
source ~/.bashrc  # or ~/.zshrc
```

**"Provider authentication failed"**
```bash
# Reconfigure provider
rice config provider openai --api-key "new-key"

# Test connection
rice chat --test
```

**"Project analysis failed"**
```bash
# Check project structure
ls -la

# Reinitialize
rm -rf .agent/
rice init
```

### Getting Help

```bash
# Built-in help
rice help

# Command-specific help
rice help generate

# Online documentation
open https://github.com/moabualruz/ricecoder/wiki
```

## Next Steps

- [Configuration Guide](configuration.md) - Advanced configuration options
- [Spec-Driven Development](spec-driven-development.md) - Systematic development workflows
- [AI Providers](providers/) - Configure additional AI providers
- [Enterprise Features](enterprise/) - Team collaboration and enterprise integrations

## Support

- **Issues**: [GitHub Issues](https://github.com/moabualruz/ricecoder/issues)
- **Discussions**: [GitHub Discussions](https://github.com/moabualruz/ricecoder/discussions)
- **Discord**: [RiceCoder Community](https://discord.gg/BRsr7bDX)