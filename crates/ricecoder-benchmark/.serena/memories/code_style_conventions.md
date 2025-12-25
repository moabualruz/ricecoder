# Code Style and Conventions

## Rust Standards
- Follow standard Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use 4 spaces for indentation
- Use rustfmt for formatting (cargo fmt)
- Use clippy for linting (cargo clippy)

## Documentation
- Module-level docs with `//!` comments
- Function/struct docs with `///` comments
- Include examples where helpful

## Async Programming
- Use `async fn` for asynchronous functions
- Use `await` for awaiting futures
- Use tokio runtime for async execution
- Use `#[tokio::test]` for async tests

## Error Handling
- Use `Result<T, E>` for fallible operations
- Define custom error types with `thiserror`
- Use `anyhow` for application-level errors
- Use `?` operator for error propagation

## Testing
- Unit tests with `#[test]`
- Async tests with `#[tokio::test]`
- Integration tests in `tests/` directory (if any)
- Use descriptive test names
- Test both success and failure cases

## Dependencies
- Use workspace dependencies where possible
- Minimize external dependencies
- Use feature flags appropriately

## Code Organization
- Separate concerns into modules (cli, error, evaluator, etc.)
- Use `pub mod` for module declarations
- Keep functions reasonably sized
- Use meaningful variable names

## CLI Design
- Use clap derive API for command-line parsing
- Provide helpful descriptions and defaults
- Use subcommands for different operations