# Code Style and Conventions

## Rust Standards
- Follow standard Rust naming conventions (snake_case for variables/functions, CamelCase for types)
- Use rustfmt for automatic code formatting
- Follow clippy linting rules (zero warnings required)
- Comprehensive documentation for all public APIs using rustdoc
- Proper error handling with thiserror and anyhow
- No unsafe code without explicit justification

## Architecture Patterns
- **Hexagonal Architecture**: Clear separation between core business logic and external interfaces
- **Dependency Injection**: Use ricecoder-di crate for clean component wiring
- **Clean Interfaces**: Well-defined traits and abstractions
- **No Circular Dependencies**: Maintain acyclic dependency graph

## Code Organization
- Each crate has a single responsibility
- Clear module structure with mod.rs files
- Private implementation details, public APIs only when necessary
- Comprehensive test modules alongside implementation

## Naming Conventions
- Functions: snake_case, descriptive names
- Types: CamelCase, noun-based
- Constants: SCREAMING_SNAKE_CASE
- Modules: snake_case, descriptive

## Error Handling
- Use Result<T, E> for fallible operations
- Custom error types with thiserror derive
- Proper error propagation with ? operator
- No panics in library code

## Async Programming
- Use tokio for async runtime
- async fn for asynchronous functions
- Proper use of futures and streams
- Avoid blocking operations in async contexts

## Testing
- Unit tests for all public functions
- Integration tests for cross-crate functionality
- Property-based tests with proptest for complex logic
- Benchmarks with criterion for performance-critical code
- Minimum 80% test coverage required

## Documentation
- rustdoc comments for all public items
- Examples in documentation where helpful
- Clear parameter and return descriptions
- Links to related items

## Security
- Input validation on all external inputs
- No logging of sensitive information
- Secure defaults for all configurations
- Regular security audits with cargo audit