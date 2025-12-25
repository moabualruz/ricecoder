# Suggested Commands

## Build Commands
- `cargo build` - Compile the project
- `cargo build --release` - Compile optimized release build

## Test Commands
- `cargo test` - Run all tests
- `cargo test --doc` - Run documentation tests
- `cargo test --release` - Run tests in release mode

## Code Quality
- `cargo fmt` - Format code according to Rust style guidelines
- `cargo clippy` - Lint code for common mistakes and improvements
- `cargo doc` - Generate documentation

## Development Workflow
- `cargo check` - Check code without building
- `cargo update` - Update dependencies
- `cargo clean` - Clean build artifacts

## Utility Commands (Windows)
- `git status` - Check git status
- `dir` - List directory contents (equivalent to `ls`)
- `cd <path>` - Change directory
- `findstr <pattern> <file>` - Search for patterns in files (equivalent to `grep`)
- `type <file>` - Display file contents (equivalent to `cat`)

## Running the Project
This is a library crate, so it doesn't have a direct executable. It provides APIs for other parts of RiceCoder to use for activity logging.