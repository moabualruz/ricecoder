# Code Style and Conventions

## Rust Style Guidelines
- Follow standard Rust formatting (`cargo fmt`)
- Use `cargo clippy` for linting and adhere to its suggestions
- Use snake_case for variables, functions, and modules
- Use CamelCase for types, structs, enums, and traits
- Use SCREAMING_SNAKE_CASE for constants

## Documentation
- Use `///` for public API documentation (appears in `cargo doc`)
- Use `//!` for module-level documentation
- Include examples in doc comments where helpful
- Document error conditions and panics

## Module Organization
- Each module has a `mod.rs` file that re-exports public items
- Use clear, descriptive module names
- Group related functionality into submodules
- Keep module files focused on single responsibilities

## Error Handling
- Use `thiserror` for custom error types
- Prefer `Result<T, Error>` over panics
- Use `anyhow` for generic error handling in tests and internal code
- Provide meaningful error messages

## Async Code
- Use `tokio` for async runtime
- Use `async-trait` for async traits
- Prefer async methods over blocking operations
- Use proper error propagation in async contexts

## Naming Conventions
- Functions: snake_case, descriptive names (e.g., `register_server`, `check_health`)
- Types: CamelCase (e.g., `LspServerConfig`, `HealthStatus`)
- Modules: snake_case (e.g., `process_manager`, `client_connection`)
- Constants: SCREAMING_SNAKE_CASE

## Code Organization
- Group imports by standard library, external crates, then internal crates
- Use blank lines to separate logical sections
- Keep functions reasonably sized (aim for <50 lines)
- Use meaningful variable names

## Testing
- Use descriptive test names (e.g., `test_rust_analyzer_configuration`)
- Use `#[test]` for unit tests
- Use `proptest` for property-based testing
- Include integration tests in `tests/` directory
- Test error conditions and edge cases

## Configuration
- Use YAML for external configuration
- Use Serde for serialization/deserialization
- Provide sensible defaults
- Validate configuration on load

## LSP Protocol Compliance
- Follow LSP specification strictly
- Handle protocol versions appropriately
- Implement proper capability negotiation
- Support graceful fallback when features unavailable

## Resource Management
- Properly manage LSP server processes
- Implement connection pooling
- Monitor health and restart failed servers
- Clean up resources on shutdown

## Logging
- Use `tracing` for structured logging
- Log important events (server start/stop, errors, health checks)
- Use appropriate log levels (info, warn, error, debug)