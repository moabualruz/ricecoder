# Code Style and Conventions

## Naming Conventions
- **Types/Structs/Enums**: PascalCase (e.g., `DIContainer`, `ServiceLifetime`)
- **Functions/Methods/Variables**: snake_case (e.g., `register_service`, `resolve`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case (e.g., `services.rs`, `usage.rs`)

## Documentation
- Use `///` for public API documentation (functions, structs, modules)
- Use `//!` for module-level documentation
- Include examples in docstrings where helpful
- Document error conditions and return values

## Code Organization
- Group related functionality in modules
- Use feature flags (`#[cfg(feature = "...")]`) for optional dependencies
- Separate concerns: core container logic vs. service registrations
- Use builder pattern for complex construction

## Error Handling
- Use `thiserror` for custom error types
- Return `Result<T, DIError>` for fallible operations
- Provide descriptive error messages
- Handle errors gracefully in service resolution

## Thread Safety
- All services must be `Send + Sync`
- Use `Arc` for shared ownership
- Use `RwLock` for mutable shared state
- Prefer read locks over write locks

## Async/Await
- Use `async-trait` for async trait methods
- Use `tokio` for async runtime
- Mark async functions with `async fn`
- Use `await` for async operations

## Type Safety
- Use generics for type-safe service resolution
- Leverage Rust's type system for compile-time guarantees
- Use `TypeId` for runtime type identification
- Avoid `Any` downcasting where possible

## Testing
- Use standard `#[test]` attributes
- Test both success and failure cases
- Test concurrent access patterns
- Use property-based testing for complex scenarios

## Performance
- Cache singleton instances after first resolution
- Use efficient data structures (HashMap with TypeId keys)
- Minimize allocations in hot paths
- Provide benchmarks for performance-critical code