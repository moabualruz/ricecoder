# Project Overview: ricecoder-config

## Purpose
The `ricecoder-config` crate provides a comprehensive configuration management system for RiceCoder, handling loading, validation, and persistence of application settings from multiple sources including files (TOML/YAML/JSON), environment variables, and runtime overrides.

## Tech Stack
- **Language**: Rust (2021 edition)
- **Key Dependencies**:
  - `serde` - Serialization/deserialization
  - `config` - Configuration file handling
  - `tokio` - Asynchronous operations
  - `notify` - File system watching
  - `toml`/`serde_yaml`/`serde_json` - Format support
  - `thiserror` - Error handling
  - `tracing` - Logging
  - `dirs` - Directory handling
- **Testing**: `proptest` for property-based testing, `tokio-test` for async tests

## Codebase Structure
- `src/lib.rs` - Main library entry point with module declarations
- `src/types.rs` - Core data structures (`AppConfig`, `EditorConfig`, etc.)
- `src/manager.rs` - Configuration management logic
- `src/tui_config.rs` - Terminal UI specific configuration
- `src/error.rs` - Error types and handling
- `tests/types.rs` - Tests for type-related functionality

## Key Features
- Hierarchical configuration loading with priority order
- Type-safe configuration with validation
- Support for multiple file formats
- Runtime configuration overrides
- Persistence to disk
- TUI integration

## Development Notes
- This is a library crate, not an executable
- Part of a larger RiceCoder workspace
- Follows standard Rust development practices