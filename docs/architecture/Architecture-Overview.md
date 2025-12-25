# RiceCoder Architecture Overview

**Version**: Alpha v0.1.7  
**Last Updated**: 2025-12-25  
**Status**: Complete (56 crates documented)

## System Architecture

RiceCoder follows **Domain-Driven Design (DDD)** with a clean layered architecture:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      PRESENTATION LAYER                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │    cli      │ │    tui      │ │   themes    │ │   help      │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                   │
│  │ ricecoder-  │ │ ricecoder-  │ │  ricegrep   │                   │
│  │  keybinds   │ │   images    │ │   (CLI)     │                   │
│  └─────────────┘ └─────────────┘ └─────────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      APPLICATION LAYER                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │ application │ │   agents    │ │  workflows  │ │  execution  │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │  sessions   │ │   modes     │ │   specs     │ │ generation  │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │  research   │ │  commands   │ │   hooks     │ │refactoring  │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │                   │
│  │orchestration│ │  learning   │ │domain-agents│                   │
│  └─────────────┘ └─────────────┘ └─────────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        DOMAIN LAYER                                  │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    ricecoder-domain                          │    │
│  │  Entities | Value Objects | Aggregates | Domain Services    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│  ┌─────────────┐ ┌─────────────┐                                   │
│  │ ricecoder-  │ │ ricecoder-  │                                   │
│  │  patterns   │ │  industry   │                                   │
│  └─────────────┘ └─────────────┘                                   │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    INFRASTRUCTURE LAYER                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │     di      │ │ persistence │ │   storage   │ │   config    │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │  providers  │ │    mcp      │ │local-models │ │   files     │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │    vcs      │ │   github    │ │    lsp      │ │external-lsp │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │   │
│  │ completion  │ │  parsers    │ │  security   │ │ monitoring  │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ ricecoder-  │ │ ricecoder-  │ │ ricecoder-  │ │ ricegrep-   │   │
│  │ permissions │ │   tools     │ │   cache     │ │   core      │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Layer Responsibilities

### Presentation Layer (8 crates)
- **Purpose**: User interface and interaction
- **Responsibilities**: CLI parsing, TUI rendering, themes, keybinds, help system
- **Dependencies**: Application layer only (no direct infrastructure access)

### Application Layer (15 crates)
- **Purpose**: Business logic orchestration
- **Responsibilities**: Use cases, workflows, agents, session management
- **Dependencies**: Domain layer for entities, Infrastructure via interfaces

### Domain Layer (3 crates)
- **Purpose**: Core business concepts
- **Responsibilities**: Entities, value objects, domain services
- **Dependencies**: None (pure domain logic)

### Infrastructure Layer (30 crates)
- **Purpose**: External integrations and persistence
- **Responsibilities**: AI providers, file system, VCS, LSP, storage
- **Dependencies**: Domain layer for interfaces to implement

## Key Design Principles

### SOLID Compliance
All 56 crates follow SOLID principles:
- **SRP**: Each crate has single responsibility
- **OCP**: Extension via traits and interfaces
- **LSP**: Consistent trait implementations
- **ISP**: Well-segregated interfaces
- **DIP**: Depend on abstractions, not concretions

### Dependency Injection
- `ricecoder-di` provides centralized DI container
- Services registered at startup
- Lazy initialization for performance
- Thread-safe with `Arc<RwLock<>>`

### Event-Driven Architecture
- `ricecoder-hooks` provides event system
- Pre/post hooks for extensibility
- Async event handling with `tokio`

## Integration Points

### AI Providers
```
ricecoder-providers ──► OpenAI, Anthropic, Ollama, 75+ providers
ricecoder-local-models ──► Ollama integration for offline-first
ricecoder-mcp ──► Model Context Protocol for tools
```

### IDE Integration
```
ricecoder-lsp ──► Language Server Protocol
ricecoder-external-lsp ──► rust-analyzer, tsserver, pylsp
ricecoder-ide ──► VS Code, JetBrains, Neovim plugins
```

### Version Control
```
ricecoder-vcs ──► Git repository abstraction
ricecoder-github ──► GitHub API, PRs, Issues
```

## RiceGrep Subsystem

Separate AI-enhanced code search tool:
```
ricegrep (CLI) ──► ricegrep-core (library)
                    ├── Heuristic AI processing
                    ├── ripgrep compatibility
                    └── MCP server integration
```

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Startup Time | < 3s | ✅ Met |
| Response Time | < 500ms | ✅ Met |
| Memory Usage | < 300MB | ✅ Met |
| Large Projects | 500+ crates | ✅ Met |
| Concurrent Sessions | 10+ | ✅ Met |

## Quality Metrics

| Metric | Value |
|--------|-------|
| Total Crates | 56 |
| Total Tests | 4000+ |
| Test Coverage | 85%+ |
| Clippy Warnings | 0 |
| TODOs/FIXMEs | 0 (or documented) |

## Related Documentation

- [Crate-Index.md](./Crate-Index.md) - Complete crate listing
- [DDD-Layering-and-Boundaries.md](./DDD-Layering-and-Boundaries.md) - Layer rules
- [SOLID Dashboard](../compliance/solid-dashboard.md) - SOLID compliance scores
