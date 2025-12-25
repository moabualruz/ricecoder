# Suggested Commands for Development

## Building and Running
- `cargo build` - Build the project
- `cargo run` - Run the project (if it has a binary)
- `cargo check` - Check for compilation errors without building

## Testing
- `cargo test` - Run all tests
- `cargo test --test tier1_servers_tests` - Run Tier 1 server tests
- `cargo test --test integration_tests` - Run integration tests
- `cargo test --lib` - Run library tests only
- `cargo test --doc` - Run documentation tests

## Code Quality
- `cargo fmt` - Format code according to Rust style guidelines
- `cargo clippy` - Run linter to catch common mistakes and improve code
- `cargo doc --open` - Generate and open documentation

## Dependencies
- `cargo update` - Update dependencies
- `cargo tree` - Show dependency tree

## System Commands (Windows)
- `git status` - Check git status
- `git add .` - Stage all changes
- `git commit -m "message"` - Commit changes
- `git log --oneline` - View commit history
- `dir` - List directory contents
- `type filename` - View file contents
- `findstr pattern filename` - Search for pattern in file
- `where command` - Find executable location

## Project-Specific
- `cargo test -p ricecoder-external-lsp` - Run tests for this specific package
- `cargo test property` - Run property tests
- `cargo test server` - Run server-related tests
- `cargo test router` - Run request routing tests