# ricegrep-core

Core domain and application layer for RiceGrep - pure business logic without infrastructure dependencies.

## Overview

This crate provides the foundational layers for RiceGrep following Domain-Driven Design (DDD) and Clean Architecture principles:

- **Domain Layer**: Pure business logic, value objects, aggregates, and domain events
- **Application Layer**: Use cases, repository traits (ports), and service orchestration

## Architecture

```text
┌─────────────────────────────────────┐
│  Consumers (MCP, REST, CLI, etc.)   │
├─────────────────────────────────────┤
│  Application Layer (this crate)     │  ← Use cases, repository traits
├─────────────────────────────────────┤
│  Domain Layer (this crate)          │  ← Pure business logic
├─────────────────────────────────────┤
│  Infrastructure (consumer provides) │  ← File I/O, indexing, etc.
└─────────────────────────────────────┘
```

## Design Principles

- **Minimal Dependencies**: Only `regex` for pattern matching
- **Pure Domain**: Domain layer has zero external dependencies
- **Ports & Adapters**: Repository traits allow custom implementations
- **Testable**: Mock implementations provided for all traits

## Modules

### Domain Layer (`domain/`)

- `value_objects.rs` - Immutable domain primitives (FilePath, EditPattern, SearchQuery)
- `aggregates.rs` - Domain entities (FileEdit, SearchResult, SearchMatch)
- `events.rs` - Domain events for event-driven patterns
- `errors.rs` - Domain-specific error types

### Application Layer (`application/`)

- `ports.rs` - Repository traits (FileRepository, IndexRepository, EventPublisher)
- `use_cases/` - Business operation orchestrators:
  - `edit_file.rs` - File editing operations with validation
  - `search_files.rs` - Search operations with pattern matching
  - `write_file.rs` - File writing operations with conflict detection
- `services.rs` - Dependency injection container (AppServices)
- `errors.rs` - Application-level error types

## Usage

```rust
use ricegrep_core::domain::{FilePath, EditPattern};
use ricegrep_core::application::{
    FileRepository, IndexRepository, EventPublisher,
    EditFileUseCase, EditFileRequest,
    AppServices, AppServicesBuilder,
};

// Implement repository traits for your infrastructure
struct MyFileRepo;
impl FileRepository for MyFileRepo { /* ... */ }

struct MyIndexRepo;
impl IndexRepository for MyIndexRepo { /* ... */ }

struct MyEventPublisher;
impl EventPublisher for MyEventPublisher { /* ... */ }

// Build services with dependency injection
let services = AppServicesBuilder::new()
    .with_file_repository(MyFileRepo)
    .with_index_repository(MyIndexRepo)
    .with_event_publisher(MyEventPublisher)
    .build();

// Or use individual use cases
let use_case = EditFileUseCase::new(my_file_repo, my_event_publisher);
let request = EditFileRequest {
    path: FilePath::new("/path/to/file.rs").unwrap(),
    pattern: EditPattern::new("old_text", "new_text").unwrap(),
    dry_run: false,
};
let response = use_case.execute(request)?;
```

## Testing

The crate includes comprehensive tests for all layers:

```bash
# Run all tests
cargo test -p ricegrep-core

# Run with verbose output
cargo test -p ricegrep-core -- --nocapture
```

## License

MIT License - see repository root for details.
