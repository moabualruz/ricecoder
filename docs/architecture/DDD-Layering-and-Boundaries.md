# DDD Layering and Boundaries

**Version**: Alpha v0.1.7  
**Last Updated**: 2025-12-25

## Layer Overview

RiceCoder follows Domain-Driven Design (DDD) with strict layer boundaries:

```
┌─────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                        │
│  User interfaces, CLI, TUI, themes, keybinds                │
├─────────────────────────────────────────────────────────────┤
│                    APPLICATION LAYER                         │
│  Use cases, workflows, orchestration, services               │
├─────────────────────────────────────────────────────────────┤
│                      DOMAIN LAYER                            │
│  Entities, value objects, domain services, business rules   │
├─────────────────────────────────────────────────────────────┤
│                   INFRASTRUCTURE LAYER                       │
│  Persistence, external APIs, file system, providers          │
└─────────────────────────────────────────────────────────────┘
```

## Layer Rules

### 1. Dependency Direction

Dependencies MUST flow downward only:

```
Presentation → Application → Domain ← Infrastructure
```

- **Presentation** depends on **Application**
- **Application** depends on **Domain**
- **Infrastructure** implements **Domain** interfaces
- **Domain** has NO external dependencies

### 2. Layer Responsibilities

| Layer | Responsibilities | Examples |
|-------|------------------|----------|
| **Presentation** | User interaction, input/output | CLI parsing, TUI rendering, themes |
| **Application** | Use cases, orchestration | Workflows, sessions, agents |
| **Domain** | Business logic, rules | Entities, value objects, services |
| **Infrastructure** | External integrations | Providers, storage, VCS, LSP |

### 3. Interface Contracts

Domain layer defines interfaces that Infrastructure implements:

```rust
// Domain layer defines
trait FileRepository {
    fn read(&self, path: &Path) -> Result<String>;
    fn write(&self, path: &Path, content: &str) -> Result<()>;
}

// Infrastructure layer implements
impl FileRepository for FileSystemRepository {
    fn read(&self, path: &Path) -> Result<String> { ... }
    fn write(&self, path: &Path, content: &str) -> Result<()> { ... }
}
```

## Layer Assignments

### Presentation Layer (8 crates)

| Crate | Justification |
|-------|---------------|
| `ricecoder-cli` | CLI entry point, command parsing |
| `ricecoder-tui` | Terminal UI rendering |
| `ricecoder-themes` | Visual theming |
| `ricecoder-help` | Help display |
| `ricecoder-keybinds` | User input handling |
| `ricecoder-images` | Image display/analysis |
| `ricegrep` | CLI for search tool |
| `ricecoder-ide` | IDE plugin interfaces |

### Application Layer (15 crates)

| Crate | Justification |
|-------|---------------|
| `ricecoder-application` | Core application services |
| `ricecoder-agents` | Agent orchestration |
| `ricecoder-domain-agents` | Specialized agents |
| `ricecoder-workflows` | Workflow execution |
| `ricecoder-execution` | Execution plans |
| `ricecoder-sessions` | Session management |
| `ricecoder-modes` | Mode handling |
| `ricecoder-specs` | Spec processing |
| `ricecoder-generation` | Code generation |
| `ricecoder-research` | Project analysis |
| `ricecoder-commands` | Command execution |
| `ricecoder-hooks` | Event hooks |
| `ricecoder-refactoring` | Refactoring orchestration |
| `ricecoder-orchestration` | Multi-project management |
| `ricecoder-learning` | Learning system |

### Domain Layer (3 crates)

| Crate | Justification |
|-------|---------------|
| `ricecoder-domain` | Core entities and services |
| `ricecoder-patterns` | Design pattern definitions |
| `ricecoder-industry` | Industry domain models |

### Infrastructure Layer (30 crates)

| Crate | Justification |
|-------|---------------|
| `ricecoder-di` | DI container |
| `ricecoder-persistence` | Database |
| `ricecoder-storage` | File storage |
| `ricecoder-config` | Configuration |
| `ricecoder-providers` | AI providers |
| `ricecoder-mcp` | MCP protocol |
| `ricecoder-local-models` | Ollama |
| `ricecoder-files` | File operations |
| `ricecoder-vcs` | Git integration |
| `ricecoder-github` | GitHub API |
| `ricecoder-lsp` | LSP integration |
| `ricecoder-external-lsp` | External LSP |
| `ricecoder-completion` | Completion engine |
| `ricecoder-parsers` | Parsing |
| `ricecoder-security` | Security |
| `ricecoder-monitoring` | Monitoring |
| `ricecoder-permissions` | Permissions |
| `ricecoder-tools` | Tool integrations |
| `ricecoder-cache` | Caching |
| `ricecoder-api` | API clients |
| `ricecoder-benchmark` | Benchmarks |
| `ricecoder-performance` | Performance |
| `ricecoder-safety` | Safety checks |
| `ricecoder-teams` | Team features |
| `ricecoder-activity-log` | Activity logging |
| `ricecoder-undo-redo` | Undo/redo |
| `ricecoder-updates` | Updates |
| `ricecoder-continuous-improvement` | CI integration |
| `ricecoder-beta` | Beta features |
| `ricegrep-core` | Search library |

## Known Violations

### 1. CLI Cross-Layer Orchestration

**Crate**: `ricecoder-cli`  
**Violation**: Orchestrates across all layers  
**Justification**: Entry point must coordinate components  
**Status**: Documented, acceptable for entry point

### 2. DI Container TypeId Usage

**Crate**: `ricecoder-di`  
**Violation**: Uses concrete TypeId for resolution  
**Justification**: Required for dynamic service resolution  
**Status**: Documented, deferred to Beta

## Boundary Enforcement

### Static Analysis

- Cargo workspace dependencies enforced in `Cargo.toml`
- Clippy lint `disallowed_dependencies` for layer violations
- CI checks for dependency direction

### Code Review

Reviewers should check:
1. No direct Infrastructure imports in Domain
2. Application only accesses Domain types
3. Presentation only accesses Application services

## Migration Guidelines

When refactoring to fix layer violations:

1. **Identify the violation**: Which layer boundary is crossed?
2. **Define an interface**: Create trait in Domain layer
3. **Move implementation**: Infrastructure implements the trait
4. **Update dependencies**: Use dependency injection
5. **Verify**: Run `cargo clippy` and tests

## Related Documentation

- [Architecture-Overview.md](./Architecture-Overview.md) - System architecture
- [Crate-Index.md](./Crate-Index.md) - Complete crate listing
