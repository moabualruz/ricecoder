# ricecoder-persistence

Infrastructure persistence layer for RiceCoder - repository implementations.

## Purpose

This crate implements the **Repository pattern** from Domain-Driven Design, providing concrete implementations of the repository interfaces defined in `ricecoder-domain`. It serves as the infrastructure layer that handles all data persistence concerns.

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                          │
├─────────────────────────────────────────────────────────────────┤
│  memory/                    │  surreal/                          │
│  ─────────                  │  ────────                          │
│  InMemoryProjectRepository  │  SurrealProjectRepository          │
│  InMemorySessionRepository  │  SurrealSessionRepository          │
│  InMemorySpecificationRepository │  SurrealSpecificationRepository │
└─────────────────────────────────────────────────────────────────┘
                             ▲
                             │ implements
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Domain Layer                               │
│  ProjectRepository, SessionRepository, SpecificationRepository   │
└─────────────────────────────────────────────────────────────────┘
```

## Key Types

### Error Handling

- **`PersistenceError`** - Error type for persistence operations with variants:
  - `NotFound` - Entity not found
  - `ConcurrencyConflict` - Optimistic locking failure
  - `Serialization` / `Deserialization` - Data transformation errors
  - `Connection` / `Database` - Infrastructure errors
  - `LockError` - Lock acquisition failures

### In-Memory Repositories (Default Feature)

Thread-safe implementations for testing and development:

- **`InMemoryProjectRepository`** - Stores `Project` aggregates
- **`InMemorySessionRepository`** - Stores `Session` aggregates
- **`InMemorySpecificationRepository`** - Stores `Specification` aggregates

### SurrealDB Repositories (Optional Feature)

Production-ready persistence with SurrealDB:

- **`SurrealProjectRepository`** - Persistent project storage
- **`SurrealSessionRepository`** - Persistent session storage
- **`SurrealSpecificationRepository`** - Persistent specification storage
- **`SurrealConnection`** - Database connection manager
- **`ConnectionMode`** - Connection configuration (Memory, File, Remote)

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `memory` | ✓ | In-memory repository implementations |
| `surrealdb-backend` | | SurrealDB persistent backend |

## Usage Examples

### In-Memory (Testing/Development)

```rust
use ricecoder_persistence::memory::InMemoryProjectRepository;
use ricecoder_domain::repositories::ProjectRepository;
use std::sync::Arc;

let repo: Arc<dyn ProjectRepository> = Arc::new(InMemoryProjectRepository::new());
```

### SurrealDB (Production)

```rust
use ricecoder_persistence::surreal::{
    SurrealConnection, ConnectionMode, SurrealProjectRepository,
};
use ricecoder_domain::repositories::ProjectRepository;
use std::sync::Arc;

// Embedded in-memory (testing)
let conn = Arc::new(SurrealConnection::new(ConnectionMode::Memory).await?);

// File-based persistent (production)
let conn = Arc::new(SurrealConnection::new(ConnectionMode::File("./data".into())).await?);

// Remote server (distributed)
let conn = Arc::new(SurrealConnection::new(ConnectionMode::Remote {
    url: "ws://localhost:8000".into(),
    username: "root".into(),
    password: "secret".into(),
}).await?);

let repo: Arc<dyn ProjectRepository> = Arc::new(SurrealProjectRepository::new(conn));
```

## Dependencies

### Required

| Dependency | Purpose |
|------------|---------|
| `ricecoder-domain` | Domain interfaces and entities |
| `async-trait` | Async trait support |
| `tokio` | Async runtime |
| `parking_lot` | High-performance synchronization primitives |
| `thiserror` | Error derive macros |
| `serde` / `serde_json` | Serialization |
| `uuid` | Unique identifiers |
| `chrono` | Date/time handling |
| `tracing` | Logging and diagnostics |

### Optional

| Dependency | Feature | Purpose |
|------------|---------|---------|
| `surrealdb` | `surrealdb-backend` | SurrealDB database client |

## Dependents

This crate is used by:

- `ricecoder-monitoring` - Uses persistence for metrics storage
- `ricecoder-continuous-improvement` - Uses persistence for improvement data

## Thread Safety

All repository implementations use `parking_lot::RwLock` for thread-safe concurrent access:
- Multiple concurrent readers allowed
- Exclusive write access when modifying data
- Clone-based isolation for returned entities

## Error Conversion

`PersistenceError` automatically converts to `DomainError` for seamless error propagation across architectural layers.
