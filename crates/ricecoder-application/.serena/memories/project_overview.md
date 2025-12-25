# RiceCoder Application Layer

## Purpose
The RiceCoder Application Layer is a Rust crate that implements application services for the RiceCoder system. It orchestrates domain aggregates and use cases, providing stateless services that coordinate multi-step workflows, manage transaction boundaries, and emit application events.

RiceCoder appears to be an AI-powered coding assistant or development tool that manages coding projects and sessions.

## Architecture
This follows Clean Architecture principles:
- **Application Layer**: Orchestrates use cases, manages transactions, emits events
- **Domain Layer**: Contains business logic, aggregates, entities, value objects
- **Infrastructure Layer**: Handles I/O operations, external services

## Tech Stack
- **Language**: Rust (2021 edition)
- **Async Runtime**: Tokio
- **Serialization**: Serde (JSON)
- **Error Handling**: ThisError
- **IDs**: UUID v4
- **Time**: Chrono
- **Testing**: Tokio test framework, Proptest for property-based testing

## Codebase Structure
```
src/
├── di/           # Dependency injection container and scopes
├── dto/          # Data Transfer Objects for API boundaries
├── services/     # Application services (Project, Session, Code, Specification)
├── events.rs     # Application events and publishers
├── errors.rs     # Application-level error types
├── ports.rs      # Abstractions for external dependencies
└── lib.rs        # Main library exports
```

## Key Components
- **ProjectService**: Manages coding projects
- **SessionService**: Handles coding sessions within projects
- **CodeService**: Likely handles code generation/modification
- **SpecificationService**: Manages project specifications
- **Dependency Injection**: Uses generic services with injected repositories and ports