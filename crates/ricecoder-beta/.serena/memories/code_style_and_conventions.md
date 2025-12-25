# Code Style and Conventions

## Naming Conventions
- **Functions/Methods/Variables**: `snake_case`
- **Structs/Enums/Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

## Type Hints
- Rust requires explicit types for function parameters and return values
- Use type aliases for complex types when appropriate
- Prefer concrete types over generics when possible, but use generics for flexibility

## Documentation
- Use `///` for public API documentation (generates docs with `cargo doc`)
- Use `//` for internal comments
- Document complex logic, public functions, and error conditions

## Error Handling
- Use `Result<T, E>` for fallible operations
- Prefer `thiserror` for custom error types
- Use `anyhow` for application-level error handling
- Avoid panicking in library code

## Async Code
- Use `tokio` for async runtime
- Prefer `async fn` for asynchronous functions
- Use `?` operator for error propagation in async contexts

## Testing
- Unit tests in same file with `#[cfg(test)]` module
- Integration tests in `tests/` directory
- Use `proptest` for property-based testing
- Aim for >80% test coverage

## Security and Compliance
- Validate all inputs
- Use secure defaults
- Log security-relevant events
- Follow enterprise compliance requirements (SOC 2, GDPR, HIPAA)

## Patterns
- Builder pattern for complex object construction
- RAII for resource management
- Iterator pattern for collections
- Command pattern for CLI subcommands