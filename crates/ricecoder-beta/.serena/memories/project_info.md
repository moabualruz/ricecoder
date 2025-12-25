# Project Information

## Purpose
This crate provides comprehensive beta testing capabilities for RiceCoder enterprise validation, including user feedback collection, analytics, enterprise requirements validation, and compliance testing.

## Tech Stack
- **Language**: Rust
- **Key Dependencies**:
  - `tokio`: Asynchronous runtime
  - `clap`: Command-line argument parser
  - `serde`/`serde_json`: Serialization
  - `reqwest`: HTTP client
  - `tracing`: Logging
  - `thiserror`: Error handling
  - `uuid`, `chrono`: Utilities
  - Workspace dependencies: `ricecoder-domain`, `ricecoder-security`, `ricecoder-activity-log`
- **Testing**: `proptest` for property-based testing, `tempfile`

## Codebase Structure
- `src/lib.rs`: Library entry point with module declarations (analytics, compliance, feedback, validation)
- `src/main.rs`: CLI application with command handlers
- `src/analytics.rs`: Analytics functionality
- `src/compliance.rs`: Compliance validation
- `src/feedback.rs`: User feedback collection
- `src/validation.rs`: Enterprise validation
- `Cargo.toml`: Package configuration
- `README.md`: Documentation and usage examples

## Entry Points
- CLI binary: `ricecoder-beta` (built from `src/main.rs`)
- Library: `ricecoder_beta` crate