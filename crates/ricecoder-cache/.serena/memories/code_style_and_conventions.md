# Code Style and Conventions for RiceCoder Cache

## Language and Edition
- **Language**: Rust
- **Edition**: 2021
- **Async Runtime**: Tokio for async operations

## Naming Conventions
- **Types/Structs/Enums**: PascalCase (e.g., `CacheConfig`, `CacheError`)
- **Functions/Methods/Variables**: snake_case (e.g., `default_ttl`, `with_config`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case (e.g., `cache.rs`, `storage.rs`)

## Documentation
- **Module docs**: Use `//!` for module-level documentation
- **Item docs**: Use `///` for structs, functions, etc.
- **Include examples**: Where appropriate, provide code examples in docs

## Code Structure
- **Traits**: Use `async_trait` for async trait methods
- **Error Handling**: Use `thiserror` for custom error types, `Result<T>` alias for `Result<T, CacheError>`
- **Serialization**: Use `serde` with `Serialize`/`Deserialize` derives
- **Builder Pattern**: Use for complex construction (e.g., `CacheBuilder`)
- **Strategy Pattern**: For pluggable invalidation strategies

## Async/Await
- **Full async support**: All cache operations are async
- **Tokio**: Use tokio's async primitives (RwLock, etc.)
- **Arc**: Use `Arc` for shared ownership in async contexts

## Dependencies
- **Workspace dependencies**: Use `{ workspace = true }` for shared deps
- **Feature flags**: Enable necessary features (e.g., tokio features)

## Testing
- **Unit tests**: In same file or separate `tests` module
- **Integration tests**: In `tests/` directory
- **Property testing**: Use `proptest` for complex test cases
- **Benchmarks**: Use `criterion` for performance testing

## Design Patterns
- **Multi-level caching**: L1 (memory), L2 (disk), L3 (remote)
- **Strategy pattern**: For cache invalidation strategies
- **Builder pattern**: For cache configuration
- **Observer pattern**: For metrics collection

## Performance Considerations
- **Async operations**: All I/O operations are async
- **Compression**: Optional gzip compression for storage
- **Metrics**: Built-in performance monitoring
- **Efficient data structures**: HashMap for in-memory storage