# Code Style and Conventions

## General
- Use Rust 2021 edition
- Enable `#![warn(missing_docs)]` for comprehensive documentation
- Follow standard Rust naming conventions
- Use `snake_case` for functions, variables, and modules
- Use `PascalCase` for types (structs, enums, traits)
- Use `SCREAMING_SNAKE_CASE` for constants

## Documentation
- Use `//!` for module-level documentation
- Use `///` for item-level documentation (functions, structs, etc.)
- Document all public APIs
- Include parameter descriptions in doc comments
- Use `#[utoipa::path]` for API endpoint documentation

## Error Handling
- Use `thiserror` for custom error types
- Return `Result<T, Box<dyn std::error::Error>>` for main functions
- Use `?` operator for error propagation
- Provide meaningful error messages

## Async/Await
- Use `tokio` as the async runtime
- Mark async functions with `async fn`
- Use `.await` for awaiting futures
- Prefer async traits with `async-trait` crate

## Serialization
- Use `serde` with `Serialize` and `Deserialize` derives
- Use `#[serde(rename = "field_name")]` for custom field names if needed
- Include `Debug` and `Clone` derives where appropriate

## OpenAPI/Swagger
- Use `utoipa::ToSchema` for all API models
- Include status codes and descriptions in `#[utoipa::path]` attributes
- Use meaningful response descriptions

## Logging
- Use `tracing` for logging
- Set log level to `INFO` in production
- Use appropriate log levels: `info!`, `warn!`, `error!`, `debug!`, `trace!`

## Dependency Injection
- Use the custom `DIContainer` for dependency management
- Initialize dependencies in `AppState::new()`

## Security
- Use `jsonwebtoken` for JWT handling
- Use `bcrypt` for password hashing
- Validate inputs and handle authentication properly

## Testing
- Write unit tests for handlers and utilities
- Use `tokio::test` for async tests
- Use `reqwest` for integration tests if needed

## Code Organization
- Keep handlers focused on HTTP concerns
- Put business logic in separate modules if complex
- Use middleware for cross-cutting concerns (auth, logging, rate limiting)
- Group related functionality in modules