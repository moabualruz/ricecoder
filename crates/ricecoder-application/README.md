# ricecoder-application

## Purpose

The Application Layer implements use cases by orchestrating domain aggregates and domain services. It provides stateless services that coordinate multi-step workflows, manage transaction boundaries, and emit application events. This layer serves as the boundary between the Presentation Layer (CLI/TUI) and the Domain Layer, ensuring domain logic is properly encapsulated.

## DDD Layer

**Application Layer**

## Responsibilities

- **Use Case Orchestration**: Coordinate multi-step workflows across domain aggregates
- **Transaction Boundaries**: Wrap atomic operations in UnitOfWork pattern
- **DTO Mapping**: Convert domain objects to/from presentation-safe DTOs
- **Application Events**: Emit use-case-level events for external consumers (audit logs, webhooks, notifications)
- **Error Mapping**: Translate domain errors to application-level errors suitable for API/UI consumption
- **Input Validation**: Validate command inputs before delegating to domain
- **Dependency Injection**: Provide type-safe service container with lifetime management

## Non-Goals

- Domain logic (belongs in Domain Layer)
- Direct I/O operations (belongs in Infrastructure Layer)
- HTTP/CLI handling (belongs in Presentation Layer)

## Dependencies

### Internal (RiceCoder Crates)

- `ricecoder-domain`: Domain aggregates, entities, value objects, repository traits
  - This is the **ONLY** internal dependency, enforcing proper DDD layering

### External Libraries

- `async-trait`: Async trait support for repository/port abstractions
- `tokio`: Async runtime (sync features only)
- `serde`, `serde_json`: DTO serialization/deserialization
- `thiserror`: Consistent error type definitions
- `uuid`: Unique identifier generation
- `chrono`: Timestamp handling with timezone support

## Key Types

### Services

| Service | Purpose |
|---------|---------|
| `ProjectService` | Project lifecycle management (create, archive, delete) |
| `SessionService` | Session lifecycle, message handling, state transitions |
| `SpecificationService` | Spec validation, requirement/task management, traceability |
| `CodeService` | Code generation and analysis orchestration |

### Ports (Abstractions)

| Port | Purpose |
|------|---------|
| `UnitOfWork` | Transaction boundary management |
| `EventPublisher` | Application event publication |

### DTOs

| DTO Category | Types |
|--------------|-------|
| Project | `CreateProjectCommand`, `ProjectSummaryDto`, `ProjectDetailDto` |
| Session | `CreateSessionCommand`, `SessionSummaryDto`, `SessionDetailDto`, `MessageDto` |
| Specification | `CreateSpecificationCommand`, `SpecificationSummaryDto`, `RequirementDto`, `TaskDto` |

### Dependency Injection

| Type | Purpose |
|------|---------|
| `ServiceContainer` | Type-safe DI container with lifetime semantics |
| `ScopedContainer` | Request-scoped service resolution |
| `ServiceLifetime` | Singleton, Scoped, Transient lifetime markers |
| `ContainerError` | DI resolution errors |

### Events

| Event | Trigger |
|-------|---------|
| `ProjectCreated` | Project successfully created |
| `ProjectArchived` | Project archived |
| `ProjectDeleted` | Project deleted |
| `SessionStarted` | New session created |
| `SessionCompleted` | Session marked complete |
| `SpecificationCreated` | New specification created |
| `SpecificationCompleted` | All tasks in spec completed |

### Errors

| Error | When |
|-------|------|
| `ValidationFailed` | Input validation fails |
| `RequiredFieldMissing` | Required field not provided |
| `ProjectNotFound` | Project ID doesn't exist |
| `SessionNotFound` | Session ID doesn't exist |
| `SpecificationNotFound` | Spec ID doesn't exist |
| `OperationNotAllowed` | Operation invalid in current state |
| `BusinessRuleViolation` | Domain rule violated |
| `RepositoryError` | Persistence layer error |
| `TransactionFailed` | Transaction rollback |

## Architecture

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                        Application Layer                                 │
├─────────────────────────────────────────────────────────────────────────┤
│  Services          │ DTOs              │ Ports              │ Events    │
│  ─────────         │ ────              │ ─────              │ ──────    │
│  ProjectService    │ CreateProjectCmd  │ UnitOfWork         │ AppEvent  │
│  SessionService    │ ProjectDto        │ EventPublisher     │           │
│  SpecService       │ SessionDto        │                    │           │
│  CodeService       │ SpecDto           │                    │           │
└─────────────────────────────────────────────────────────────────────────┘
                             ▲
                             │ depends on
                             ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Domain Layer                                     │
│  Aggregates, Entities, Value Objects, Domain Events, Repository Traits  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Usage Examples

### Creating a Project

```rust
use ricecoder_application::services::ProjectService;
use ricecoder_application::dto::CreateProjectCommand;
use ricecoder_domain::value_objects::ProgrammingLanguage;

let service = ProjectService::new(repo, uow, events);

let cmd = CreateProjectCommand {
    name: "my-project".to_string(),
    root_path: "/path/to/project".to_string(),
    language: ProgrammingLanguage::Rust,
    description: Some("A Rust project".to_string()),
};

let project_id = service.create_project(cmd).await?;
```

### Using the DI Container

```rust
use ricecoder_application::di::{ServiceContainer, ServiceLifetime};
use std::sync::Arc;

let mut container = ServiceContainer::new();

// Register singleton (shared instance)
container.register_singleton(Arc::new(MyService::new()));

// Register scoped (per-request instance)
container.register_scoped(|| RequestContext::new());

// Register transient (new instance each time)
container.register_transient(|| CommandBuilder::new());

// Create a scope for request handling
let scope = container.create_scope();
let service: Arc<MyService> = scope.resolve()?;
```

## Testing

Tests are organized in the `tests/` directory:

```
tests/
└── service_tests.rs    # Integration tests with mock repositories
```

Run tests:
```bash
cargo test -p ricecoder-application
```

## SOLID Compliance

| Principle | Implementation |
|-----------|---------------|
| **SRP** | Each service handles one aggregate type; DTOs separate from domain |
| **OCP** | Services are generic over repository traits; new implementations don't modify services |
| **LSP** | Repository trait implementations are fully substitutable (mock ↔ real) |
| **ISP** | `UnitOfWork` and `EventPublisher` are focused single-purpose traits |
| **DIP** | Services depend on abstractions (traits), not concrete implementations |

## Gap Analysis vs OpenCode

| Pattern | RiceCoder | OpenCode | Notes |
|---------|-----------|----------|-------|
| DI Pattern | Constructor injection | AsyncLocalStorage context | Both valid; RiceCoder is more explicit |
| DTOs | Separate structs | Zod schemas | RiceCoder uses idiomatic Rust structs |
| Use Cases | Service methods | `fn(schema, handler)` | Similar intent, different idioms |
| Events | `ApplicationEvent` enum | `BusEvent.define()` | Both typed, RiceCoder uses enum |
| Validation | Service-level checks | Schema validation | Could add Rust validation crate |
| Tool Registry | Not in application layer | `Tool.define()` pattern | Tools live in infrastructure |

### Potential Improvements (Beta)

1. **Validation Framework**: Consider `validator` crate for declarative DTO validation
2. **Event Sourcing**: Add event store capability for audit/replay
3. **CQRS**: Separate read/write models for complex queries
4. **Saga Pattern**: For long-running distributed transactions
