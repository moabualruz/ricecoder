# Suggested Commands for Developing ricecoder-config

## Build Commands
- `cargo build` - Compile the project in debug mode
- `cargo build --release` - Compile the project in release mode
- `cargo check` - Check the project for errors without building

## Test Commands
- `cargo test` - Run all tests
- `cargo test -p ricecoder-config` - Run tests specifically for this crate (if in workspace)

## Code Quality Commands
- `cargo fmt` - Format the code according to Rust style guidelines
- `cargo clippy` - Run the linter to check for common mistakes and improvements
- `cargo doc --open` - Generate and open documentation

## Version Control
- `git status` - Check the status of the repository
- `git add .` - Stage all changes
- `git commit -m "message"` - Commit staged changes
- `git push` - Push commits to remote repository

## Utility Commands (Windows)
- `dir` - List files in current directory (equivalent to `ls` on Unix)
- `cd <path>` - Change directory
- `type <file>` - Display file contents (equivalent to `cat` on Unix)
- `findstr <pattern> <file>` - Search for pattern in file (equivalent to `grep` on Unix)