# Project Overview

## Purpose
RiceCoder Domain is the core domain entities and business logic crate for RiceCoder, a system for AI-powered code analysis. It implements Domain-Driven Design principles to provide the foundational business rules, entities, value objects, and services for the RiceCoder platform.

## Tech Stack
- **Language**: Rust (edition 2021)
- **Architecture**: Domain-Driven Design (DDD)
- **Key Dependencies**:
  - `serde` & `serde_json`: Serialization/deserialization
  - `thiserror`: Error handling
  - `uuid`: Unique identifier generation
  - `chrono`: Date/time handling
  - `regex`: Regular expression validation
  - `url`: URL validation
  - `mime_guess`: MIME type detection
  - `async-trait`: Async trait support
  - `futures-core`: Futures utilities
- **Testing**: `proptest` for property-based testing, `tokio` for async tests

## Codebase Structure
- `src/entities/`: Core business entities (Project, CodeFile, Session, AnalysisResult, Provider)
- `src/value_objects/`: Immutable domain concepts (identifiers, ProgrammingLanguage, SemanticVersion, etc.)
- `src/services/`: Domain services for business logic
- `src/repositories/`: Repository interfaces for data persistence
- `src/ports/`: Port interfaces for external dependencies
- `src/events/`: Domain events
- `src/specification/`: Specification pattern implementations
- `src/project/`, `src/session/`: Specific domain modules
- `tests/`: Unit tests, property-based tests, and business rule tests

## Related Crates
- `ricecoder-storage`: Repository implementations
- `ricecoder-security`: Security services
- `ricecoder-providers`: AI provider management
- `ricecoder-sessions`: Session services
- `ricecoder-analysis`: Analysis services