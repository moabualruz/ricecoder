# Code Style and Conventions

## Rust Standards
- Follows standard Rust formatting (`cargo fmt`)
- Uses Clippy lints (`cargo clippy`)
- 4-space indentation
- 100 character line width
- Uses `rustfmt` defaults

## Documentation
- All public APIs have `///` documentation comments
- Module-level documentation uses `//!`
- Includes architecture decisions and AC (Acceptance Criteria) references
- Examples: `//! Project Application Service`

## Naming Conventions
- **Modules**: snake_case (e.g., `project_service.rs`)
- **Types**: PascalCase (e.g., `ProjectService`, `ApplicationError`)
- **Functions/Methods**: snake_case (e.g., `create_project`, `get_project`)
- **Variables**: snake_case (e.g., `project_id`, `cmd`)
- **Constants**: SCREAMING_SNAKE_CASE

## Error Handling
- Uses `thiserror` for custom error types
- Returns `Result<T, ApplicationError>` for fallible operations
- Maps domain errors to application errors
- Uses `?` operator for error propagation

## Async/Await
- Uses Tokio async runtime
- All service methods are `async fn`
- Uses `async_trait` for trait definitions
- Proper `Send + Sync + 'static` bounds for generics

## Dependency Injection
- Services are generic over repository and port types
- Uses `Arc<T>` for shared ownership
- Constructor injection pattern: `Service::new(repo, uow, events)`

## Testing
- Unit tests in same file with `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Uses `tokio::test` for async tests
- Mock implementations for repositories and ports
- Property-based testing with `proptest`

## Architecture Patterns
- **Clean Architecture**: Strict separation of concerns
- **Dependency Inversion**: Depends on abstractions, not concretions
- **Unit of Work**: Transaction boundaries for data operations
- **Domain Events**: Application events for cross-cutting concerns
- **DTO Pattern**: Separate data transfer objects for API boundaries

## Code Organization
- One concept per file
- Clear module hierarchy
- Re-exports in `lib.rs` for public API
- Private implementation details hidden

## Type Safety
- Strong typing with generics
- Value objects for domain concepts (IDs, enums)
- Comprehensive error types
- No `unwrap()` in production code (use proper error handling)