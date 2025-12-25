# Code Style and Conventions for ricecoder-config

## Naming Conventions
- **Structs and Enums**: PascalCase (e.g., `AppConfig`, `EditorConfig`)
- **Fields and Variables**: snake_case (e.g., `tab_size`, `insert_spaces`)
- **Functions and Methods**: snake_case (e.g., `load_config`, `save_config`)
- **Modules**: snake_case (e.g., `tui_config`, `manager`)

## Documentation
- Use `///` for doc comments on public items (structs, fields, functions)
- Provide brief, descriptive documentation for all public APIs
- Document parameters and return values where necessary

## Code Structure
- Use `#[derive(...)]` for common traits like `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`
- Implement `Default` for configuration structs with sensible defaults
- Use `pub` for public fields in configuration structs
- Group related functionality into modules (e.g., `types.rs` for data structures, `manager.rs` for logic)

## Error Handling
- Use `thiserror` for custom error types
- Use `anyhow` for generic error handling in functions
- Prefer `Result<T, E>` over panics

## Dependencies
- Use `serde` for serialization/deserialization
- Use `tokio` for async operations
- Follow workspace dependency versions

## Testing
- Use `proptest` for property-based testing
- Include unit tests for validation logic
- Test configuration loading from different sources

## Formatting
- Follow `cargo fmt` standard formatting
- No trailing whitespace
- Consistent indentation