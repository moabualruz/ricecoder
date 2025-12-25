# Suggested Commands for ricecoder-execution

## Build Commands
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode

## Test Commands
- `cargo test` - Run all tests
- `cargo test -p ricecoder-execution` - Run tests specifically for this crate
- `cargo test property` - Run property-based tests
- `cargo test approval` - Run approval-related tests
- `cargo test rollback` - Run rollback-related tests

## Code Quality Commands
- `cargo fmt` - Format the code
- `cargo clippy` - Lint the code for potential issues
- `cargo doc --open` - Generate and open documentation

## Utility Commands (Windows)
- `dir` - List files in directory (equivalent to `ls`)
- `cd <path>` - Change directory
- `git status` - Check git status
- `git add .` - Stage all changes
- `git commit -m "message"` - Commit changes
- `findstr "pattern" <file>` - Search for pattern in file (equivalent to `grep`)

## Project-Specific Commands
- No specific entrypoints as this is a library crate. Use as dependency in other projects.