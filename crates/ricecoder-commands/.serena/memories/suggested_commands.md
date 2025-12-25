# Suggested Commands

## Building
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version

## Testing
- `cargo test` - Run all tests
- `cargo test -- --ignored` - Run property tests
- `cargo tarpaulin` - Run tests with coverage (if tarpaulin installed)

## Code Quality
- `cargo fmt` - Format code
- `cargo clippy` - Lint code
- `cargo check` - Check for compilation errors without building

## Other
- `cargo doc --open` - Generate and open documentation

When a task is completed, run: cargo fmt, cargo clippy, cargo test