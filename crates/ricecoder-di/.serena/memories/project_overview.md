# Project Overview

## Purpose
RiceCoder DI Container is a thread-safe dependency injection container for RiceCoder that implements the service locator pattern with TypeId-based registration. It enables clean architecture by decoupling component creation and wiring across the RiceCoder application.

## Tech Stack
- **Language**: Rust
- **Build System**: Cargo
- **Key Dependencies**:
  - `async-trait`: For async trait methods
  - `thiserror`: For error handling
  - `tracing`: For logging
  - `tokio`: For async runtime
  - `serde`/`serde_json`: For serialization
  - `anyhow`: For error handling
- **Concurrency**: Uses `Arc` for shared ownership and `RwLock` for thread-safe access

## Architecture
Follows clean architecture principles:
- **Domain Layer**: Business logic and entities
- **Application Layer**: Use cases and application services (registered in DI)
- **Infrastructure Layer**: External interfaces and implementations

## Key Features
- Service Locator Pattern with TypeId-based registration
- Multiple service lifetimes: Singleton, Transient, Scoped
- Thread safety with RwLock
- Builder pattern for fluent configuration
- Health checks for service monitoring
- Feature-gated services based on Cargo features
- Lifecycle management for initialization/cleanup
- Macros for convenience

## Codebase Structure
- `src/lib.rs`: Core DI container implementation
- `src/services.rs`: Service registration functions for all RiceCoder crates
- `src/usage.rs`: Usage examples and documentation
- `tests/`: Unit, integration, property, and service tests
- `benches/`: Performance benchmarks