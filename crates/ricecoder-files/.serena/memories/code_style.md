# Code Style and Conventions

## Naming
- Functions and variables: snake_case
- Types (structs, enums): CamelCase
- Constants: SCREAMING_SNAKE_CASE
- Modules: snake_case

## Documentation
- Use /// for doc comments
- Include examples where appropriate

## Error Handling
- Custom error types in error.rs
- Use Result<T, FileError>

## Async
- Use async/await with tokio

## Safety
- Ensure atomic operations
- Provide rollback paths