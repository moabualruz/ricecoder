# Code Style and Conventions

## Rust Conventions
- **Naming**: 
  - Types (structs, enums): PascalCase (e.g., ContinuousImprovementPipeline)
  - Functions/methods: snake_case (e.g., generate_recommendations)
  - Modules: snake_case (e.g., feedback_pipeline)
- **Structs**: Use derive macros for common traits (Debug, Clone, etc.)
- **Error Handling**: Use thiserror for custom errors, anyhow for generic errors
- **Async**: Use tokio for async runtime, async-trait for async traits
- **Documentation**: Use /// for doc comments
- **Imports**: Group by std, external crates, internal crates

## Project-Specific Patterns
- **Configuration**: Extensive use of config structs with Default implementations
- **Pipelines**: Each pipeline has its own module with config, types, and implementation
- **Health Checks**: ComponentHealth enum for monitoring
- **Enterprise Focus**: Many configs have enterprise-specific options

## Formatting
- Use `cargo fmt` for consistent formatting
- Use `cargo clippy` for linting and style checks