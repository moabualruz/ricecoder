# RiceCoder Project Overview

## Purpose
RiceCoder is a terminal-first, spec-driven coding assistant that understands your project before generating code. Unlike traditional AI coding tools, RiceCoder analyzes your codebase, understands your patterns, and generates code that fits your project's style and architecture.

## Key Features
- Research-First approach: Analyzes project context before generating code
- Spec-Driven development: Systematic development from specifications
- Terminal-Native: Beautiful CLI/TUI that works anywhere
- Offline-First: Local models via Ollama for privacy
- Multi-Agent: Specialized agents for different tasks
- Multi-Provider: OpenAI, Anthropic, Ollama, and more
- LSP Integration: Language Server Protocol support
- MCP Ecosystem: Model Context Protocol for AI assistant integration

## Tech Stack
- **Language**: Rust 1.75+
- **Architecture**: Hexagonal architecture with dependency injection
- **Async Runtime**: Tokio
- **TUI Framework**: Ratatui
- **Web Framework**: Axum
- **Database**: SQLite with SQLx
- **Serialization**: Serde (JSON/YAML/TOML)
- **Testing**: Proptest for property-based testing, Criterion for benchmarks
- **Build System**: Cargo workspace with multiple crates

## Rough Codebase Structure
- **crates/**: Modular architecture with specialized crates
  - ricecoder-cli: Main command-line interface
  - ricecoder-tui: Terminal user interface
  - ricecoder-providers: AI provider integrations
  - ricecoder-agents: Multi-agent framework
  - ricecoder-lsp: Language Server Protocol integration
  - ricecoder-mcp: Model Context Protocol support
  - And 40+ other specialized crates
- **config/**: Configuration files and schemas
- **scripts/**: Build, test, and deployment scripts
- **docs/**: Documentation and guides
- **tests/**: Comprehensive test suite

## Development Guidelines
- **Spec-Driven Development**: All features start with specifications
- **Research-First**: Analyze codebase before code generation
- **Hexagonal Architecture**: Clean separation of concerns
- **Dependency Injection**: Modular component wiring
- **Security-First**: SOC 2 compliance features
- **Performance-Focused**: Strict performance targets and validation