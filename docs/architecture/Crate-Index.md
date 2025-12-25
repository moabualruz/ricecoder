# RiceCoder Crate Index

**Version**: Alpha v0.1.7  
**Total Crates**: 56  
**Last Updated**: 2025-12-25

## Crate Summary by Layer

| Layer | Count | Description |
|-------|-------|-------------|
| Presentation | 8 | User interface and interaction |
| Application | 15 | Business logic orchestration |
| Domain | 3 | Core business concepts |
| Infrastructure | 30 | External integrations and persistence |

---

## Presentation Layer (8 crates)

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-cli` | Main CLI entry point, binary `rice` | 5/5 |
| `ricecoder-tui` | Terminal user interface with ratatui | 5/5 |
| `ricecoder-themes` | 6 built-in themes, runtime switching | 5/5 |
| `ricecoder-help` | Help system with search and categories | 5/5 |
| `ricecoder-keybinds` | Cross-platform keybind management | 5/5 |
| `ricecoder-images` | Image handling with AI analysis | 5/5 |
| `ricegrep` | CLI for AI-enhanced code search | 5/5 |
| `ricecoder-ide` | IDE integration (VS Code, JetBrains, Neovim) | 5/5 |

---

## Application Layer (15 crates)

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-application` | Core application services | 5/5 |
| `ricecoder-agents` | Multi-agent framework | 5/5 |
| `ricecoder-domain-agents` | Specialized domain agents | 5/5 |
| `ricecoder-workflows` | Declarative workflow execution | 5/5 |
| `ricecoder-execution` | Execution plans with risk scoring | 5/5 |
| `ricecoder-sessions` | Multi-session persistence and sharing | 5/5 |
| `ricecoder-modes` | Code/Ask/Vibe modes | 5/5 |
| `ricecoder-specs` | YAML/Markdown spec validation | 5/5 |
| `ricecoder-generation` | Spec-driven code generation | 5/5 |
| `ricecoder-research` | Project analysis and context | 5/5 |
| `ricecoder-commands` | User-defined shell commands | 5/5 |
| `ricecoder-hooks` | Event-driven automation | 5/5 |
| `ricecoder-refactoring` | Safe multi-language refactoring | 5/5 |
| `ricecoder-orchestration` | Multi-project workspace management | 5/5 |
| `ricecoder-learning` | User interaction tracking | 5/5 |

---

## Domain Layer (3 crates)

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-domain` | Core entities, value objects, services | 5/5 |
| `ricecoder-patterns` | Design pattern definitions | 5/5 |
| `ricecoder-industry` | Industry-specific domain models | 5/5 |

---

## Infrastructure Layer (30 crates)

### Core Infrastructure

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-di` | Dependency injection container | 4/5 |
| `ricecoder-persistence` | Database abstraction (SurrealDB) | 5/5 |
| `ricecoder-storage` | File storage and caching | 4.5/5 |
| `ricecoder-config` | Configuration management | 5/5 |

### AI & Models

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-providers` | 75+ AI provider integrations | 5/5 |
| `ricecoder-mcp` | Model Context Protocol | 4.5/5 |
| `ricecoder-local-models` | Ollama integration | 5/5 |

### File & VCS

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-files` | File operations with backup | 5/5 |
| `ricecoder-vcs` | Git repository abstraction | 5/5 |
| `ricecoder-github` | GitHub API integration | 5/5 |

### Language Intelligence

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-lsp` | Language Server Protocol | 5/5 |
| `ricecoder-external-lsp` | External LSP servers | 5/5 |
| `ricecoder-completion` | Code completion engine | 5/5 |
| `ricecoder-parsers` | Tree-sitter parsing | 5/5 |

### Security & Monitoring

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-security` | Encryption, ABAC, SOC 2 | 5/5 |
| `ricecoder-monitoring` | Metrics, telemetry, alerting | 5/5 |
| `ricecoder-permissions` | Permission management | 5/5 |

### Tools & Utilities

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-tools` | Webfetch, Patch, Todo, Search | 5/5 |
| `ricecoder-cache` | Caching infrastructure | 5/5 |
| `ricecoder-api` | API client utilities | 5/5 |

### Quality & Performance

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-benchmark` | Performance benchmarks | 5/5 |
| `ricecoder-performance` | Performance monitoring | 5/5 |
| `ricecoder-safety` | Safety validation | 5/5 |

### Teams & Collaboration

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-teams` | Team workspaces | 5/5 |

### System Management

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricecoder-activity-log` | Activity logging | 5/5 |
| `ricecoder-undo-redo` | Undo/redo operations | 5/5 |
| `ricecoder-updates` | Update management | 5/5 |
| `ricecoder-continuous-improvement` | CI/CD integration | 5/5 |
| `ricecoder-beta` | Beta features management | 5/5 |

### RiceGrep Core

| Crate | Purpose | SOLID Score |
|-------|---------|-------------|
| `ricegrep-core` | AI-enhanced search library | 5/5 |

---

## Quality Metrics Summary

| Metric | Value |
|--------|-------|
| Total Crates | 56 |
| SOLID Score 5/5 | 52 crates |
| SOLID Score 4.5/5 | 2 crates |
| SOLID Score 4/5 | 2 crates |
| Clippy Warnings | 0 |
| TODOs/FIXMEs | 0 (documented exceptions) |

## Related Documentation

- [Architecture-Overview.md](./Architecture-Overview.md) - System architecture
- [DDD-Layering-and-Boundaries.md](./DDD-Layering-and-Boundaries.md) - Layer rules
