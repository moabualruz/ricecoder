# Suggested Commands

## Build Commands
- `cargo build`: Build the project in debug mode
- `cargo build --release`: Build the project in release mode
- `cargo build -p ricecoder-domain-agents`: Build this specific crate

## Test Commands
- `cargo test`: Run all tests
- `cargo test -p ricecoder-domain-agents`: Run tests for this crate
- `cargo test -p ricecoder-domain-agents frontend`: Run tests matching "frontend"
- `cargo test -p ricecoder-domain-agents backend`: Run tests matching "backend"

## Formatting and Linting
- `cargo fmt`: Format code using rustfmt
- `cargo clippy`: Run Clippy linter for code quality checks
- `cargo clippy -- -D warnings`: Treat warnings as errors

## Development Workflow
- `cargo check`: Check code for compilation errors without building
- `cargo doc --open`: Generate and open documentation
- `cargo run --example <example>`: Run examples (if any)

## Utility Commands (Windows)
- `git status`: Check git status
- `git add .`: Stage all changes
- `git commit -m "message"`: Commit changes
- `dir`: List directory contents (equivalent to `ls`)
- `cd <path>`: Change directory
- `find <pattern>`: Search for files (use with caution)
- `cargo tree`: Show dependency tree

## Workspace Commands
Since this is part of a workspace, use `-p ricecoder-domain-agents` for crate-specific operations.