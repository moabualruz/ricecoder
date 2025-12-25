# Technology Stack

## Language
- Rust 2021 Edition

## Core Dependencies
- **serde/serde_json**: Serialization and deserialization
- **tokio**: Async runtime
- **tracing**: Logging framework
- **chrono**: Date/time handling
- **uuid**: Unique identifiers
- **thiserror**: Error handling
- **async-trait**: Async traits
- **futures**: Future utilities
- **regex**: Regular expressions

## Internal Dependencies
- **ricecoder-sessions**: Session management (workspace dependency)

## Development Dependencies
- **proptest**: Property-based testing
- **tokio-test**: Async testing utilities
- **tempfile**: Temporary file creation for tests

## Build System
- Cargo (Rust package manager)
- Workspace-based project structure