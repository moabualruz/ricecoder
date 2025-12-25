# Development Commands

## Testing
- `cargo test`: Run all unit tests, property-based tests, and integration tests
- `cargo test --lib`: Run only library tests (exclude integration tests)
- `cargo test --doc`: Run documentation tests

## Formatting
- `cargo fmt`: Format all Rust code according to standard conventions
- `cargo fmt --check`: Check if code is properly formatted (CI use)

## Linting
- `cargo clippy`: Run Clippy linter for code quality checks
- `cargo clippy -- -D warnings`: Treat warnings as errors

## Building
- `cargo build`: Build the library in debug mode
- `cargo build --release`: Build optimized release version
- `cargo check`: Check code without building (fast compilation check)

## Documentation
- `cargo doc`: Generate HTML documentation
- `cargo doc --open`: Generate and open documentation in browser

## Dependencies
- `cargo update`: Update dependencies to latest compatible versions
- `cargo tree`: Show dependency tree

## Running Examples (if any)
- `cargo run --example <name>`: Run specific example

## Windows-Specific Commands
- `dir`: List directory contents (equivalent to `ls`)
- `cd <path>`: Change directory
- `findstr <pattern> <file>`: Search for patterns in files (equivalent to `grep`)
- `git status`: Check git status
- `git add .`: Stage all changes
- `git commit -m "message"`: Commit changes
- `git push`: Push to remote repository