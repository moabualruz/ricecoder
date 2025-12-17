<div align="center">

 <img src=".branding/banner.svg" alt="rice[oder logo">

**Plan. Think. Code.**

[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

</div>

---

## What is RiceCoder?

RiceCoder (`rice`) is a terminal-first, spec-driven coding assistant that understands your project before generating code. Unlike other AI coding tools, RiceCoder follows a research-first approach-analyzing your codebase, understanding your patterns, and generating code that fits your project's style.

### Key Features

- **🔬 Research-First** - Analyzes your project context before generating code
- **📋 Spec-Driven** - Systematic, repeatable development from specifications
- **💻 Terminal-Native** - Beautiful CLI/TUI that works anywhere
- **🔒 Offline-First** - Local models via Ollama for privacy and offline work
- **🤖 Multi-Agent** - Specialized agents for different tasks
- **🎨 Multi-Provider** - OpenAI, Anthropic, Ollama, and more
- **📊 Token Tracking** - Real-time token usage monitoring with cost estimation
- **🚀 Project Bootstrap** - Automatic project detection and configuration
- **🎯 Session Management** - Persistent sessions with token-aware message handling
- **🔧 Dependency Injection** - Modular architecture with service container for clean component wiring

---

## Installation

### Quick Start

Choose your preferred installation method:

| Method | Command | Best For |
|--------|---------|----------|
| **Cargo** | `cargo install ricecoder` | Rust developers, easy updates |
| **Curl** | `curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install \| bash` | Quick setup, any platform |
| **Docker** | `docker pull moabualruz/ricecoder:latest` | Isolated environments |
| **npm** | `npm install -g ricecoder` | Node.js developers |
| **Homebrew** | `brew install ricecoder` | macOS users |
| **Scoop** | `scoop install ricecoder` | Windows users |
| **Winget** | `winget install RiceCoder.RiceCoder` | Windows users |
| **From Source** | `git clone ... && ./scripts/install.sh` | Development, customization |

### Installation Methods

#### 1. Cargo (Recommended)

Install from [crates.io](https://crates.io/crates/ricecoder):

```bash
# Install
cargo install ricecoder

# Verify
rice --version

# Update
cargo install --force ricecoder
```

**Requirements**: Rust 1.75+ ([Install Rust](https://rustup.rs/))

**Platforms**: Linux, macOS, Windows

---

#### 2. Curl Script (Quick Setup)

Build and install from source with a single command:

```bash
# Linux/macOS - Standard installation
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash

# With custom prefix
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash -s -- --prefix /usr/local

# Debug build
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash -s -- --debug

# Verify
rice --version
```

**Features**:

- Detects OS and architecture automatically
- Builds from source with automatic compilation
- Verifies prerequisites (Rust, Cargo, Git)
- Automatic Rust toolchain update
- Installs to custom prefix or default location
- Automatic installation verification

**Platforms**: Linux (x86_64, ARM64), macOS (Intel, Apple Silicon)

**Troubleshooting**: See [Installation Troubleshooting](#installation-troubleshooting)

---

#### 3. Docker

Run in a containerized environment:

```bash
# Pull image
docker pull moabualruz/ricecoder:latest

# Run
docker run --rm moabualruz/ricecoder:latest --version

# Run with workspace access
docker run -it -v $(pwd):/workspace moabualruz/ricecoder:latest chat
```

**Features**:

- Isolated environment
- No system dependencies
- Consistent across platforms
- Easy cleanup

**Platforms**: Any platform with Docker

**Requirements**: Docker ([Install Docker](https://docs.docker.com/get-docker/))

---

#### 4. npm (Node.js Developers)

Install via npm registry:

```bash
# Install globally
npm install -g ricecoder

# Verify
rice --version

# Update
npm install -g ricecoder@latest
```

**Features**:

- Familiar npm workflow
- Easy version management
- Works with Node.js projects

**Platforms**: Linux, macOS, Windows

**Requirements**: Node.js 14+ and npm

---

#### 5. Homebrew (macOS)

Install via Homebrew:

```bash
# Install
brew install ricecoder

# Verify
rice --version

# Update
brew upgrade ricecoder
```

**Features**:

- Native macOS package manager
- Easy updates and uninstall
- Integrates with system

**Platforms**: macOS

**Requirements**: Homebrew ([Install Homebrew](https://brew.sh/))

---

#### 6. Scoop (Windows)

Install via Scoop:

```powershell
# Install
scoop install ricecoder

# Verify
rice --version

# Update
scoop update ricecoder
```

**Features**:

- Native Windows package manager
- Easy updates and uninstall
- Integrates with Windows

**Platforms**: Windows

**Requirements**: Scoop ([Install Scoop](https://scoop.sh/))

---

#### 7. Winget (Windows)

Install via Windows Package Manager:

```powershell
# Install
winget install RiceCoder.RiceCoder

# Verify
rice --version

# Update
winget upgrade RiceCoder.RiceCoder
```

**Features**:

- Official Microsoft package manager
- Enterprise-friendly
- Integrated with Windows

**Platforms**: Windows

**Requirements**: Windows Package Manager (winget) - included with Windows 10/11

---

#### 8. Build from Source

Clone and build locally with automatic installation:

**Using Installation Scripts (Recommended)**:

```bash
# Clone repository
git clone https://github.com/moabualruz/ricecoder.git
cd ricecoder

# Linux/macOS - Automatic installation
chmod +x scripts/install.sh
./scripts/install.sh

# Windows (PowerShell)
.\scripts\install.ps1

# Windows (CMD)
scripts\install.bat
```

**Installation Script Options**:

```bash
# Linux/macOS
./scripts/install.sh --prefix /usr/local    # Custom prefix
./scripts/install.sh --debug                # Debug build
./scripts/install.sh --verbose              # Verbose output

# Windows (PowerShell)
.\scripts\install.ps1 -Prefix "C:\Program Files\ricecoder"
.\scripts\install.ps1 -Debug
```

**Manual Build and Install**:

```bash
# Clone repository
git clone https://github.com/moabualruz/ricecoder.git
cd ricecoder

# Build and install
cargo install --path projects/ricecoder

# Verify
rice --version
```

**Features**:

- Latest development version
- Customizable build (release/debug)
- Full source access
- Automatic prerequisite checking
- Automatic Rust toolchain update
- Automatic PATH configuration

**Platforms**: Linux, macOS, Windows

**Requirements**: Rust 1.75+, Git, C compiler

**See Also**: [Installation Guide](./INSTALLATION.md) - Comprehensive build and installation documentation

---

### Platform Support Matrix

| Platform | Arch | Cargo | Curl | Docker | npm | Homebrew | Scoop | Winget | Source |
|----------|------|-------|------|--------|-----|----------|-------|--------|--------|
| **Linux** | x86_64 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| **Linux** | ARM64 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| **macOS** | Intel | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| **macOS** | Apple Silicon | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| **Windows** | x86_64 | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| **Windows** | ARM64 | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |

**Legend**: ✅ Supported | ❌ Not available | ⚠️ Limited support

---

### System Requirements

**Minimum**:

- OS: Linux, macOS, or Windows
- RAM: 512 MB
- Disk: 100 MB

**Recommended**:

- OS: Linux (Ubuntu 18.04+), macOS (10.13+), Windows 10+
- RAM: 2 GB
- Disk: 500 MB
- Terminal: Modern terminal emulator (iTerm2, Windows Terminal, GNOME Terminal)

**For Building from Source**:

- Rust 1.75+ ([Install Rust](https://rustup.rs/))
- Git
- C compiler (gcc, clang, or MSVC)

---

### Verification

After installation, verify it works:

```bash
# Check version
rice --version

# Show help
rice --help

# Initialize project
rice init

# Test connection (if configured)
rice chat --test
```

---

### Installation Troubleshooting

#### "Command not found: rice"

**Solution**:

1. Restart your terminal
2. Check PATH: `echo $PATH`
3. Verify installation: `which rice` or `where rice` (Windows)
4. Reinstall if needed

See [Installation Setup Guide](https://github.com/moabualruz/ricecoder/wiki/Installation-Setup) for detailed troubleshooting.

---

#### "Permission denied"

**Solution**:

1. Check file permissions: `ls -la ~/.cargo/bin/rice`
2. Fix permissions: `chmod +x ~/.cargo/bin/rice`
3. Ensure directory is in PATH

---

#### "Checksum verification failed" (Curl)

**Solution**:

1. Re-run installation script
2. Check network connection
3. Try alternative installation method

---

#### "Docker image not found"

**Solution**:

1. Pull image: `docker pull moabualruz/ricecoder:latest`
2. Check Docker is running: `docker ps`
3. Verify internet connection

---

### Uninstallation

Remove RiceCoder:

```bash
# Cargo
cargo uninstall ricecoder

# npm
npm uninstall -g ricecoder

# Homebrew
brew uninstall ricecoder

# Docker
docker rmi moabualruz/ricecoder:latest

# From source
cd ricecoder && cargo uninstall --path projects/ricecoder
```

Remove configuration:

```bash
# Remove global config
rm -rf ~/.ricecoder/

# Remove project config
rm -rf .agent/
```

---

## Frequently Asked Questions (FAQ)

### General Questions

#### What is RiceCoder?

RiceCoder is a terminal-first, spec-driven coding assistant that understands your project before generating code. Unlike traditional AI coding tools, RiceCoder analyzes your codebase, understands your patterns, and generates code that fits your project's style and architecture.

#### How is RiceCoder different from other AI coding assistants?

- **Research-First**: Analyzes your project context before generating code
- **Spec-Driven**: Systematic development from specifications
- **Terminal-Native**: Beautiful CLI/TUI that works anywhere
- **Offline-First**: Local models via Ollama for privacy
- **Multi-Agent**: Specialized agents for different tasks
- **Enterprise-Ready**: SOC 2 compliance, audit logging, RBAC

#### Is RiceCoder free?

RiceCoder is licensed under CC BY-NC-SA 4.0, making it free for personal and non-commercial use. Commercial use requires a separate license.

#### What programming languages does RiceCoder support?

RiceCoder supports all major programming languages including Rust, Python, JavaScript/TypeScript, Go, Java, C++, and more. It uses LSP (Language Server Protocol) for comprehensive language support.

### Installation & Setup

#### Which installation method should I use?

- **Cargo**: Best for Rust developers, easy updates
- **Curl**: Quick setup for any platform
- **Docker**: Isolated environments
- **Package Managers**: Homebrew (macOS), Scoop (Windows), Winget (Windows), npm
- **From Source**: For development or customization

#### I'm getting "Command not found: rice"

1. Restart your terminal
2. Check PATH: `echo $PATH`
3. Verify installation: `which rice` (Unix) or `where rice` (Windows)
4. Reinstall if needed

#### How do I configure AI providers?

```bash
# Configure OpenAI
rice provider config openai --api-key sk-your-key

# Configure Anthropic
rice provider config anthropic --api-key sk-ant-your-key

# Test connection
rice provider test openai

# Set default provider
rice provider default openai
```

#### Can I use local models?

Yes! RiceCoder supports Ollama for offline-first development:

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Start server
ollama serve

# Pull a model
ollama pull llama2:13b

# Configure RiceCoder
rice provider config ollama --base-url http://localhost:11434
```

### Usage Questions

#### How do I start using RiceCoder?

```bash
# Initialize project
rice init

# Start interactive chat
rice chat

# Generate code from spec
rice gen --spec my-feature.spec.md

# Review code
rice review src/main.rs
```

#### What are specs and how do I write them?

Specs are Markdown or YAML files that describe what you want to build. Example:

```markdown
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
```

#### How do I share sessions with my team?

```bash
# Create shareable session
rice chat --share

# Share with team
rice session share --team my-team

# Join shared session
rice session join https://ricecoder.app/s/session-id
```

#### What is MCP and how do I use it?

MCP (Model Context Protocol) allows RiceCoder to connect to external tools. Basic setup:

```bash
# Install MCP servers
npm install -g @modelcontextprotocol/server-filesystem

# Configure in RiceCoder
rice mcp add filesystem --command npx --args "-y,@modelcontextprotocol/server-filesystem,/workspace"

# Start MCP servers
rice mcp start

# Use in chat
rice chat --mcp
```

### Performance & Troubleshooting

#### Why is RiceCoder slow?

Common causes:
1. Large codebase analysis - use `--focus` flag
2. Slow provider - try different model or provider
3. Network issues - check connection
4. Resource constraints - check memory/CPU usage

#### How do I optimize performance?

```bash
# Run performance validation
./scripts/run-performance-validation.sh

# Profile specific operations
rice profile chat --duration 30s

# Optimize provider settings
rice provider optimize openai --metric latency

# Monitor resources
rice monitor resources
```

#### I'm getting API rate limit errors

Solutions:
1. Reduce request frequency
2. Switch to different provider
3. Upgrade API plan
4. Use local models (Ollama)

#### How do I debug issues?

```bash
# Enable debug logging
export RUST_LOG=debug
rice chat

# Check system health
rice doctor

# View logs
rice logs --tail 100

# Validate configuration
rice config validate
```

### Enterprise & Security

#### Is RiceCoder SOC 2 compliant?

Yes, RiceCoder includes SOC 2 Type II compliance features:
- Comprehensive audit logging
- Customer-managed encryption keys
- Access control and RBAC
- Security monitoring and threat detection

#### How do I set up enterprise security?

```bash
# Enable enterprise features
rice enterprise enable

# Configure audit logging
rice audit enable

# Set up RBAC
rice rbac configure

# Configure encryption
rice encryption setup
```

#### Can I use RiceCoder with corporate firewalls?

Yes, RiceCoder supports proxy configuration and can work behind corporate firewalls. Configure proxy settings in your provider configuration.

### Development & Contributing

#### How do I contribute to RiceCoder?

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

#### How do I report bugs?

Use GitHub Issues: https://github.com/moabualruz/ricecoder/issues

Include:
- RiceCoder version (`rice --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Logs (`rice logs`)

#### Where can I get help?

- **Discord**: https://discord.gg/BRsr7bDX
- **GitHub Discussions**: https://github.com/moabualruz/ricecoder/discussions
- **Documentation**: https://github.com/moabualruz/ricecoder/wiki

---

## Troubleshooting Guide

### Installation Issues

#### "Permission denied" during installation

**Unix/Linux/macOS**:
```bash
# Fix permissions
chmod +x ~/.cargo/bin/rice

# Or reinstall with proper permissions
sudo curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash
```

**Windows**:
- Run terminal as Administrator
- Check User Account Control settings
- Use Scoop or Winget for easier installation

#### Rust toolchain issues

```bash
# Update Rust
rustup update

# Install specific version
rustup install 1.75

# Set default version
rustup default 1.75
```

#### Build failures from source

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Build with verbose output
cargo build --verbose

# Check for missing system dependencies
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install

# Windows
# Install Visual Studio Build Tools
```

### Configuration Issues

#### Provider configuration not working

```bash
# Validate configuration
rice config validate

# Test provider connection
rice provider test openai

# Check API key format
echo $OPENAI_API_KEY | head -c 10  # Should start with sk-

# Reset provider config
rice provider reset openai
```

#### MCP server connection failures

```bash
# Check MCP server status
rice mcp status

# Restart MCP servers
rice mcp restart

# Debug MCP connections
rice mcp debug

# Check server logs
rice mcp logs filesystem
```

#### Session persistence issues

```bash
# Check session storage
rice session list

# Validate session files
rice session validate

# Reset session storage
rice session reset

# Check disk space
df -h
```

### Runtime Issues

#### High memory usage

```bash
# Monitor memory usage
rice monitor memory

# Clear caches
rice cache clear

# Reduce concurrent sessions
rice config set max_concurrent_sessions 3

# Use smaller models
rice provider config ollama --model phi:2.7b
```

#### Slow response times

```bash
# Check network latency
ping api.openai.com

# Test provider performance
rice provider benchmark openai

# Switch to faster provider
rice provider default anthropic

# Enable caching
rice cache enable
```

#### Crashes or panics

```bash
# Enable crash reporting
rice config set crash_reporting enabled

# Run with debug symbols
RUST_BACKTRACE=1 rice chat

# Check system resources
rice doctor

# Update to latest version
rice update
```

### Network and Connectivity

#### Proxy configuration

```bash
# Set HTTP proxy
rice config set proxy http://proxy.company.com:8080

# Set HTTPS proxy
rice config set https_proxy http://proxy.company.com:8080

# Bypass proxy for local
rice config set no_proxy localhost,127.0.0.1
```

#### SSL/TLS issues

```bash
# Disable SSL verification (not recommended for production)
rice config set ssl_verify false

# Use custom CA certificates
rice config set ca_certs /path/to/ca-bundle.crt

# Check certificate validity
openssl s_client -connect api.openai.com:443
```

#### Firewall blocking connections

```bash
# Test connectivity
rice network test api.openai.com

# Check firewall rules
# Linux
sudo ufw status

# Windows
netsh advfirewall show allprofiles

# macOS
sudo pfctl -s rules
```

### Provider-Specific Issues

#### OpenAI API errors

```bash
# Check API key
rice provider test openai

# Check rate limits
rice provider limits openai

# Switch to different model
rice provider config openai --model gpt-3.5-turbo

# Check account status
# Visit https://platform.openai.com/account
```

#### Anthropic/Claude issues

```bash
# Test connection
rice provider test anthropic

# Check API key format (should start with sk-ant-)
echo $ANTHROPIC_API_KEY | head -c 10

# Try different model
rice provider config anthropic --model claude-3-haiku-20240307
```

#### Ollama local model issues

```bash
# Check Ollama status
ollama list

# Restart Ollama
ollama serve

# Pull model again
ollama pull llama2:13b

# Check system resources
# Ollama needs significant RAM for larger models
```

### Performance Optimization

#### Profiling and benchmarking

```bash
# Run performance benchmarks
rice benchmark run

# Profile specific command
rice profile "gen --spec my-spec.md" --output profile.json

# Monitor system resources
rice monitor resources --interval 5

# Generate performance report
rice report performance --output perf-report.md
```

#### Memory optimization

```bash
# Check memory usage
rice monitor memory

# Clear all caches
rice cache clear all

# Reduce session history
rice config set max_session_history 100

# Use streaming responses
rice config set streaming enabled
```

#### Storage optimization

```bash
# Check disk usage
rice monitor disk

# Clean old sessions
rice session cleanup --older-than 90d

# Compress session data
rice session compress

# Move cache to faster storage
rice config set cache_dir /fast/ssd/cache
```

### Advanced Troubleshooting

#### Debug logging

```bash
# Enable detailed logging
export RUST_LOG=ricecoder=debug

# Log to file
rice chat 2>&1 | tee ricecoder-debug.log

# Filter specific components
export RUST_LOG=ricecoder::mcp=trace,ricecoder::providers=debug

# View recent logs
rice logs --since 1h
```

#### System diagnostics

```bash
# Run full system check
rice doctor --full

# Check dependencies
rice doctor --deps

# Validate all configurations
rice doctor --config

# Generate diagnostic report
rice doctor --report diagnostic.md
```

#### Recovery procedures

```bash
# Backup current configuration
rice config backup --output config-backup.yaml

# Reset to defaults
rice config reset

# Restore from backup
rice config restore config-backup.yaml

# Emergency cleanup
rice emergency cleanup
```

### Getting Help

If you can't resolve an issue:

1. **Check the documentation**: https://github.com/moabualruz/ricecoder/wiki
2. **Search existing issues**: https://github.com/moabualruz/ricecoder/issues
3. **Ask the community**: https://discord.gg/BRsr7bDX
4. **File a bug report**: Include version, OS, steps to reproduce, and logs

For urgent enterprise support, contact enterprise@ricecoder.com

---

### Getting Started

After installation, initialize your first project:

```bash
# Initialize project
rice init

# Start interactive chat
rice chat

# Generate code from a spec
rice gen --spec my-feature

# Review code
rice review src/main.rs
```

For detailed setup instructions, see [Installation Setup Guide](https://github.com/moabualruz/ricecoder/wiki/Installation-Setup).

---

### Next Steps

- Explore [interactive chat mode](docs/chat-mode.md) for free-form coding assistance
- Learn about [spec-driven development](docs/specs.md) for systematic coding
- Configure [multiple AI providers](docs/providers.md) for optimal performance
- Set up [local models](docs/ollama.md) for offline privacy

---

## Performance

RiceCoder is designed for high performance with strict performance targets:

### Performance Targets ✅

- **🚀 Startup Time**: < 3 seconds (cold start)
- **⚡ Response Time**: < 500ms (typical operations)
- **🧠 Memory Usage**: < 300MB (typical sessions)
- **🏗️ Large Projects**: 500+ crates, 50K+ lines with incremental analysis
- **🔄 Concurrent Sessions**: Up to 10+ parallel sessions

### Performance Validation

Run performance validation to ensure targets are met:

```bash
# Run performance validation
./scripts/run-performance-validation.sh

# Update performance baselines
./scripts/update-performance-baselines.sh

# Check for regressions
ricecoder-performance check-regression --binary ./target/release/ricecoder --baseline performance-baselines.json
```

Performance baselines are automatically tracked and regression detection alerts when performance degrades beyond acceptable thresholds.

---

## Why RiceCoder?

| Feature | RiceCoder | Others |
|---------|-----------|--------|
| Terminal-native | ✅ | ❌ IDE-focused |
| Spec-driven development | ✅ | ❌ Ad-hoc |
| Offline-first (local models) | ✅ | ❌ Cloud-only |
| Research before coding | ✅ | ❌ Generate immediately |
| Multi-agent framework | ✅ | ⚠️ Limited |

---

## Community

Join our community to discuss RiceCoder, ask questions, and share ideas:

- **[Discord Server](https://discord.gg/BRsr7bDX)** - Real-time chat and community support
- **[GitHub Discussions](https://github.com/moabualruz/ricecoder/discussions)** - Async discussions and Q&A
- **[GitHub Issues](https://github.com/moabualruz/ricecoder/issues)** - Bug reports and feature requests

---

## Documentation

Complete documentation is available in the [RiceCoder Wiki](https://github.com/moabualruz/ricecoder/wiki):

### Core Guides

- **[Quick Start Guide](https://github.com/moabualruz/ricecoder/wiki/Quick-Start)** - Get started in 5 minutes
- **[CLI Commands Reference](https://github.com/moabualruz/ricecoder/wiki/CLI-Commands)** - All available commands
- **[Configuration Guide](https://github.com/moabualruz/ricecoder/wiki/Configuration)** - Configure RiceCoder
- **[TUI Interface Guide](https://github.com/moabualruz/ricecoder/wiki/TUI-Interface)** - Terminal UI navigation and shortcuts
- **[AI Providers Guide](https://github.com/moabualruz/ricecoder/wiki/AI-Providers)** - Set up OpenAI, Anthropic, Ollama, and more
- **[Local Models Setup](https://github.com/moabualruz/ricecoder/wiki/Local-Models)** - Use Ollama for offline-first development
- **[Spec-Driven Development](https://github.com/moabualruz/ricecoder/wiki/Spec-Driven-Development)** - Systematic development with specs

### Phase 2 Features

- **[Code Generation](https://github.com/moabualruz/ricecoder/wiki/Code-Generation)** - Generate code from specs with AI enhancement and validation
- **[Multi-Agent Framework](https://github.com/moabualruz/ricecoder/wiki/Multi-Agent-Framework)** - Specialized agents for code review, testing, documentation, and refactoring
- **[Workflows & Execution](https://github.com/moabualruz/ricecoder/wiki/Workflows-Execution)** - Declarative workflows with state management, approval gates, and risk scoring
- **[Execution Plans](https://github.com/moabualruz/ricecoder/wiki/Execution-Plans)** - Risk scoring, approval gates, test integration, pause/resume, and rollback
- **[Sessions](https://github.com/moabualruz/ricecoder/wiki/Sessions)** - Multi-session support with persistence, sharing, and background agent execution
- **[Modes](https://github.com/moabualruz/ricecoder/wiki/Modes)** - Code/Ask/Vibe modes with Think More extended reasoning
- **[Conversation Sharing](https://github.com/moabualruz/ricecoder/wiki/Conversation-Sharing)** - Share sessions with team members via shareable links with read-only access

### Phase 3 Features

- **[LSP Integration](https://github.com/moabualruz/ricecoder/wiki/LSP-Integration)** - Language Server Protocol for IDE integration with multi-language semantic analysis
- **[Code Completion](https://github.com/moabualruz/ricecoder/wiki/Code-Completion)** - Context-aware code completion with intelligent ranking and ghost text
- **[Hooks System](https://github.com/moabualruz/ricecoder/wiki/Hooks-System)** - Event-driven automation with hook chaining and configuration

### Phase 5 Foundation Features

- **[Enhanced Tools](https://github.com/moabualruz/ricecoder/wiki/Enhanced-Tools)** - Webfetch, Patch, Todo, Web Search with hybrid MCP provider architecture
- **[Webfetch Tool](https://github.com/moabualruz/ricecoder/wiki/Enhanced-Tools)** - Fetch web content with timeout and truncation
- **[Patch Tool](https://github.com/moabualruz/ricecoder/wiki/Enhanced-Tools)** - Apply unified diffs with conflict detection
- **[Todo Tool](https://github.com/moabualruz/ricecoder/wiki/Enhanced-Tools)** - Persistent task management
- **[Web Search Tool](https://github.com/moabualruz/ricecoder/wiki/Enhanced-Tools)** - Search with free APIs or local MCP servers
- **[Refactoring Engine](https://github.com/moabualruz/ricecoder/wiki/Refactoring-Engine)** - Safe refactoring with multi-language support
- **[Markdown Configuration](https://github.com/moabualruz/ricecoder/wiki/Configuration)** - Markdown-based configuration system
- **[Keybind Customization](https://github.com/moabualruz/ricecoder/wiki/Configuration)** - Custom keybind profiles and management

### Phase 6 Infrastructure Features

- **[Orchestration](https://github.com/moabualruz/ricecoder/wiki/Orchestration)** - Multi-project workspace management with cross-project operations
- **[Domain-Specific Agents](https://github.com/moabualruz/ricecoder/wiki/Domain-Agents)** - Specialized agents for frontend, backend, DevOps, data engineering, mobile, and cloud
- **[Learning System](https://github.com/moabualruz/ricecoder/wiki/Learning-System)** - User interaction tracking and personalized recommendations

### Phase 7 Integration Features

- **[GitHub Integration](https://github.com/moabualruz/ricecoder/wiki/GitHub-Integration)** - GitHub API integration, PR/Issue creation, repository analysis (✅ Complete)
- **[Conversation Sharing](https://github.com/moabualruz/ricecoder/wiki/Conversation-Sharing)** - Share sessions with shareable links and read-only access (✅ Complete)
- **[Team Collaboration](https://github.com/moabualruz/ricecoder/wiki/Team-Collaboration)** - Team workspaces, shared knowledge base, permissions (✅ Complete)
- **[IDE Integration](https://github.com/moabualruz/ricecoder/wiki/IDE-Integration)** - VS Code, JetBrains, Neovim plugins with external LSP-first architecture (✅ Complete)
- **[Installation Methods](https://github.com/moabualruz/ricecoder/wiki/Installation-Methods)** - Curl, package managers, Docker, binaries for all platforms (✅ Complete)
- **[Theme System](https://github.com/moabualruz/ricecoder/wiki/Theme-System)** - Built-in and custom themes with hot-reload support (✅ Complete)
- **[Image Support](https://github.com/moabualruz/ricecoder/wiki/Image-Support)** - Drag-and-drop images with AI analysis, caching, and terminal display (✅ Complete)

### Additional Resources

- **[FAQ](https://github.com/moabualruz/ricecoder/wiki/FAQ)** - Frequently asked questions
- **[Troubleshooting Guide](https://github.com/moabualruz/ricecoder/wiki/Troubleshooting)** - Common issues and solutions
- **[Architecture Overview](https://github.com/moabualruz/ricecoder/wiki/Architecture-Overview)** - System design and architecture

---

## Development Status

### Current Release: Alpha v0.1.7 ✅ (Published on crates.io)

**Status**: Phase 7 complete with all integration features validated and released. GitHub integration, team collaboration, IDE plugins, installation methods, theme system, and image support now available.

### Release Strategy

RiceCoder follows a phased release strategy with extended Alpha testing before production release:

- **Alpha (v0.1.1)** ✅ - Phase 1: Foundation features
- **Alpha (v0.1.2)** ✅ - Phase 2: Enhanced features
- **Alpha (v0.1.3)** ✅ - Phase 3: MVP features
- **Alpha (v0.1.4)** ✅ - Phase 4: Polished and hardened
- **Alpha (v0.1.5)** ✅ - Phase 5: Foundation features
- **Alpha (v0.1.6)** ✅ - Phase 6: Infrastructure features
- **Alpha (v0.1.7)** ✅ - Phase 7: Integration features (current)
- **Alpha (v0.1.8)** 📋 - Phase 8: Production readiness (planned)

**Why Extended Alpha?** We're gathering user feedback, identifying edge cases, optimizing performance, and hardening security before the production release.

### Phase 1: Alpha Foundation ✅ COMPLETE (v0.1.1)

**Status**: 11/11 features complete, 500+ tests, 82% coverage, zero clippy warnings

- [x] CLI Foundation - Commands, shell completion, beautiful UX
- [x] AI Providers - OpenAI, Anthropic, Ollama, 75+ providers
- [x] TUI Interface - Terminal UI with themes and syntax highlighting
- [x] Spec System - YAML/Markdown specs with validation
- [x] File Management - Safe writes, git integration, backups
- [x] Templates & Boilerplates - Template engine with substitution
- [x] Research System - Project analysis and context building
- [x] Permissions System - Fine-grained tool access control
- [x] Custom Commands - User-defined shell commands
- [x] Local Models - Ollama integration for offline-first development
- [x] Storage & Config - Multi-level configuration hierarchy

### Phase 2: Beta Enhanced Features ✅ COMPLETE (v0.1.2)

**Status**: 7/7 features complete, 900+ tests, 86% coverage, zero clippy warnings

- [x] Code Generation - Spec-driven code generation with AI enhancement, validation, conflict detection, and rollback
- [x] Multi-Agent Framework - Specialized agents for code review, testing, documentation, and refactoring
- [x] Workflows - Declarative workflow execution with state management, approval gates, and risk scoring
- [x] Execution Plans - Risk scoring, approval gates, test integration, pause/resume, and rollback
- [x] Sessions - Multi-session persistence, sharing, and background agent execution
- [x] Modes - Code/Ask/Vibe modes with Think More extended reasoning
- [x] Conversation Sharing - Share sessions with team members via shareable links with read-only access and permission-based filtering

**Timeline**: Completed December 8, 2025

### Phase 3: Beta MVP Features ✅ COMPLETE (v0.1.3)

**Status**: 3/3 features complete, 544 tests, 86% coverage, zero clippy warnings

- [x] LSP Integration - Language Server Protocol for IDE integration with multi-language semantic analysis
- [x] Code Completion - Tab completion and ghost text suggestions with context awareness
- [x] Hooks System - Event-driven automation with hook chaining and configuration

**Timeline**: Completed December 5, 2025

### Phase 4: Alpha Validation and Hardening ✅ COMPLETE (v0.1.4)

**Status**: 7/7 feature areas complete, 1000+ tests, 85%+ coverage, zero clippy warnings

- [x] Performance Optimization - Profiling, caching, memory optimization
- [x] Security Hardening - Security audit, best practices, hardening
- [x] User Experience Polish - Error messages, onboarding, accessibility
- [x] Documentation & Support - Comprehensive docs, guides, support resources
- [x] External LSP Integration - Integration with external LSP servers (rust-analyzer, tsserver, pylsp)
- [x] Final Validation - Comprehensive testing, validation, community feedback
- [x] Alpha Release - v0.1.4 released and available

**Timeline**: Completed December 5, 2025

### Phase 5: Foundation Features ✅ COMPLETE (v0.1.5)

**Status**: 7/7 features complete, 1100+ tests, 85%+ coverage, zero clippy warnings

- [x] Enhanced Tools - Webfetch, Patch, Todo, Web Search with hybrid MCP provider architecture
- [x] Webfetch Tool - Fetch web content with timeout and truncation
- [x] Patch Tool - Apply unified diffs with conflict detection
- [x] Todo Tools - Persistent task management
- [x] Web Search Tool - Search with free APIs or local MCP servers
- [x] Refactoring System - Safe refactoring with multi-language support
- [x] Markdown Configuration - Markdown-based configuration system
- [x] Keybind Customization - Custom keybind profiles and management

**Timeline**: Completed December 5, 2025

### Phase 6: Infrastructure Features ✅ COMPLETE (v0.1.6)

**Status**: 3/3 features complete, 1200+ tests, 85%+ coverage, zero clippy warnings

- [x] Orchestration - Multi-project workspace management with cross-project operations
- [x] Domain-Specific Agents - Specialized agents for frontend, backend, DevOps, data engineering, mobile, and cloud
- [x] Learning System - User interaction tracking and personalized recommendations

**Timeline**: Completed December 6, 2025

### Phase 7 (v0.1.7) - Integration Features ✅ COMPLETE

**Status**: 7/7 features complete, 1300+ tests, 88% coverage, zero clippy warnings

- [x] GitHub Integration - GitHub API integration and PR/Issue creation
- [x] Conversation Sharing - Share sessions with shareable links and read-only access
- [x] Team Collaboration - Team workspaces and shared knowledge base
- [x] IDE Integration - VS Code, JetBrains, Neovim plugins with external LSP-first architecture
- [x] Installation Methods - Curl, package managers, Docker, binaries for all platforms
- [x] Theme System - Built-in and custom themes with hot-reload support
- [x] Image Support - Drag-and-drop images with AI analysis, caching, and terminal display

**Timeline**: Completed December 9, 2025

#### Image Support ✅ COMPLETE

RiceCoder now supports drag-and-drop image support with AI analysis, intelligent caching, and terminal display:

**Features**:

- **Drag-and-Drop**: Simply drag images into the terminal to include them in your prompts
- **Multi-Format Support**: PNG, JPG, GIF, and WebP formats
- **AI Analysis**: Automatic image analysis via your configured AI provider (OpenAI, Anthropic, Ollama, etc.)
- **Smart Caching**: Cached analysis results with 24-hour TTL and LRU eviction (100 MB limit)
- **Terminal Display**: Beautiful image rendering in the terminal with ASCII fallback for unsupported terminals
- **Automatic Optimization**: Large images (>10 MB) are automatically optimized before analysis
- **Session Integration**: Images are stored in session history for persistence and sharing

**Usage**:

```bash
# Start interactive chat
rice chat

# Drag and drop an image into the terminal
# The image will be analyzed and included in your prompt

# Example: Ask about an image
# "What's in this screenshot?"
# "Analyze this diagram"
# "Review this design mockup"
```

**Configuration**:

Image support is configured in `projects/ricecoder/config/images.yaml`:

```yaml
images:
  # Supported formats
  formats:
    - png
    - jpg
    - jpeg
    - gif
    - webp
  
  # Display settings
  display:
    max_width: 80          # Max width for terminal display
    max_height: 30         # Max height for terminal display
    placeholder_char: "█"  # ASCII placeholder character
  
  # Cache settings
  cache:
    enabled: true
    ttl_seconds: 86400     # 24 hours
    max_size_mb: 100       # LRU limit
  
  # Analysis settings
  analysis:
    timeout_seconds: 10    # Provider timeout
    max_image_size_mb: 10  # Optimization threshold
    optimize_large_images: true
```

**Performance**:

- Drag-and-drop detection: < 100ms
- Format validation: < 500ms
- Image analysis: < 10 seconds
- Cache lookup: < 50ms
- Display rendering: < 200ms

**Supported Providers**:

- OpenAI (GPT-4 Vision)
- Anthropic (Claude 3 Vision)
- Google (Gemini Vision)
- Ollama (with vision models)
- Zen (with vision support)

See [Image Support Guide](https://github.com/moabualruz/ricecoder/wiki/Image-Support) for detailed documentation.

### Phase 8: Published Issues Fixes 🔧 IN PROGRESS

**Status**: 11 consolidated issues identified and being fixed

After publishing v0.1.7 on crates.io, 11 issues were discovered through manual testing and code analysis:

1. **TUI Event Loop** - Event polling implementation
2. **Configuration System** - First-run setup and config loading
3. **Provider Standards** - Zen provider URL and model discovery
4. **Path Resolution** - Unified path handling across commands
5. **Default to TUI** - Make `rice` command default to TUI
6. **Non-Interactive Init** - Support CI/CD initialization
7. **Provider Registry** - Unified provider management
8. **Config Command** - Actual config loading and saving
9. **Error Handling** - Robust error recovery
10. **Graceful Shutdown** - Terminal state restoration
11. **Code Reusability** - Centralized utilities

**Fixes**: Configuration-driven architecture with maximum code reusability and feature configurability.


### Production (v1.0.0) 📋 PLANNED

**After Phase 8 completion** - Production release with all issues fixed and features validated

See [Development Roadmap](https://github.com/moabualruz/ricecoder/wiki/Development-Roadmap) for details.

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

This project is licensed under [CC BY-NC-SA 4.0](LICENSE.md).

- ✅ Free for personal and non-commercial use
- ✅ Fork, modify, and share
- ❌ Commercial use requires a separate license

---

## Acknowledgments

Built with ❤️ using Rust.

Inspired by [Aider](https://github.com/paul-gauthier/aider), [OpenCode](https://github.com/sst/opencode), and [Claude Code](https://claude.ai).

---

<div align="center">

**r[** - *Plan. Think. Code.*

</div>
