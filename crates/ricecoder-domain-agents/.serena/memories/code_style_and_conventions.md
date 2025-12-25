# Code Style and Conventions

## Documentation
- All public items must have documentation comments
- Use `//!` for module-level documentation
- Use `///` for item documentation
- Compiler warning `#![warn(missing_docs)]` enforces documentation

## Naming Conventions
- **Types/Enums/Structs**: PascalCase (e.g., `DomainAgentConfig`)
- **Functions/Methods/Variables**: snake_case (e.g., `get_agent_for_domain`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case (e.g., `domain_agents`)

## Code Organization
- Comprehensive doc comments for all public APIs
- Use `serde` derives for serialization with `#[serde(rename_all = "lowercase")]` for enums
- Implement standard traits: `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` where appropriate
- Use `chrono` for timestamps in RFC3339 format

## Error Handling
- Use `thiserror` for custom error types
- Define `Result<T>` type alias for convenience
- Provide meaningful error messages

## Async Programming
- Use `tokio` for async runtime
- Use `async-trait` for async trait methods
- Follow async/await patterns

## Serialization
- Use `serde` with `Serialize` and `Deserialize` derives
- Support JSON and YAML formats
- Use `serde_json::Value` for flexible configuration

## Testing
- Use `proptest` for property-based testing
- Use `tokio-test` for async tests
- Use `tempfile` for temporary file handling in tests

## Best Practices
- Implement `Default` where sensible
- Provide constructor methods (e.g., `new()`)
- Use builder patterns for complex configurations
- Follow Rust ownership and borrowing rules strictly