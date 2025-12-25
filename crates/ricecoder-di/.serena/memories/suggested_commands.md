# Suggested Commands for Development

## Building and Compilation
- `cargo build`: Compile the project in debug mode
- `cargo build --release`: Compile the project in release mode with optimizations
- `cargo check`: Check the project for compilation errors without building

## Testing
- `cargo test`: Run all tests (unit, integration, property-based)
- `cargo test --features full`: Run tests with all optional features enabled
- `cargo test -- --nocapture`: Run tests with output capture disabled for debugging

## Code Quality
- `cargo fmt`: Format the code according to Rust style guidelines
- `cargo clippy`: Run the linter to catch common mistakes and improve code quality
- `cargo fmt --check`: Check if code is properly formatted without modifying files
- `cargo clippy -- -D warnings`: Treat clippy warnings as errors

## Performance
- `cargo bench`: Run performance benchmarks
- `cargo build --release --features full`: Build optimized version with all features

## Documentation
- `cargo doc`: Generate documentation for the project
- `cargo doc --open`: Generate and open documentation in browser

## Dependency Management
- `cargo update`: Update dependencies to their latest compatible versions
- `cargo tree`: Display the dependency tree

## Windows-Specific Notes
All commands work the same on Windows as on Unix systems. Cargo handles platform differences automatically. Use PowerShell or Command Prompt as terminal.