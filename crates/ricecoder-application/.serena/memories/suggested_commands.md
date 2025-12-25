# Essential Commands for RiceCoder Application Development

## Build Commands
- `cargo build` - Compile the project in debug mode
- `cargo build --release` - Compile optimized release build
- `cargo check` - Check code for errors without building

## Test Commands
- `cargo test` - Run all unit and integration tests
- `cargo test --lib` - Run only library tests (unit tests)
- `cargo test --test service_tests` - Run specific integration test file

## Code Quality
- `cargo fmt` - Format code according to Rust style guidelines
- `cargo clippy` - Run linter to catch common mistakes and style issues
- `cargo doc --open` - Generate and open documentation

## Development Workflow
- `cargo watch -x check` - Watch for changes and run checks (requires cargo-watch)
- `cargo watch -x test` - Watch for changes and run tests

## Utility Commands (Windows)
- `dir` - List directory contents
- `type filename` - Display file contents
- `cd path` - Change directory
- `git status` - Check git status
- `git add .` - Stage all changes
- `git commit -m "message"` - Commit changes

## Running the Application
Since this is a library crate, it doesn't have a main executable. It's meant to be used as a dependency in other crates (likely the presentation/infrastructure layers).