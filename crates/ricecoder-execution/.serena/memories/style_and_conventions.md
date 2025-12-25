# Code Style and Conventions for ricecoder-execution

## Rust Conventions
- **Naming**: PascalCase for structs, enums, traits; snake_case for functions, variables, modules
- **Modules**: Each major component in separate module file (e.g., `models.rs`, `manager.rs`)
- **Error Handling**: Use `thiserror` for custom error types, `anyhow` for generic errors if needed
- **Async**: Use `tokio` for async runtime, async functions where appropriate
- **Serialization**: Use `serde` with derive macros for JSON/YAML serialization

## Project-Specific Patterns
- **Structs**: Comprehensive with derive macros (Debug, Clone, Serialize, Deserialize)
- **Enums**: For status types, actions, modes
- **Modules**: Logical separation of concerns (approval, rollback, validation, etc.)
- **Testing**: Property-based testing with proptest, integration tests in tests/ directory
- **Documentation**: Inline docs for public APIs, examples in README

## Guidelines
- Safety first: Implement risk assessment and approval gates
- Test integration: Ensure proper test running and failure handling
- Rollback robustness: Test rollback scenarios thoroughly
- User experience: Clear approval processes and error messages
- Error recovery: Provide clear recovery options