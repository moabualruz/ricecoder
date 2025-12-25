# Suggested Commands for Development

## Building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode
- `cargo build --release --bin ricecoder-benchmark` - Build the benchmark binary specifically

## Running
- `cargo run --bin ricecoder-benchmark -- run --model openai/gpt-4 --exercises-dir exercises --results-dir results` - Run the benchmark
- `cargo run --bin ricecoder-benchmark -- list --exercises-dir exercises` - List available exercises
- `cargo run --bin ricecoder-benchmark -- summary --results-dir results` - Show results summary

## Testing
- `cargo test` - Run all tests
- `cargo test --lib` - Run library tests only
- `cargo test --bin ricecoder-benchmark` - Run binary tests only

## Code Quality
- `cargo fmt` - Format code
- `cargo clippy` - Lint code
- `cargo check` - Check for compilation errors without building

## Setup
- `./scripts/setup-benchmark.sh` - Download exercises (from README, assumes script exists)

## Utility Commands (Windows)
- `dir` - List files in directory (equivalent to `ls`)
- `cd <path>` - Change directory
- `findstr <pattern> <file>` - Search for pattern in file (equivalent to `grep`)
- `git status` - Check git status
- `git add .` - Stage all changes
- `git commit -m "message"` - Commit changes
- `git log --oneline` - View commit history

## When Task is Completed
Run these commands in order:
1. `cargo fmt` - Format the code
2. `cargo clippy` - Check for linting issues
3. `cargo test` - Run tests to ensure everything works
4. `cargo build --release --bin ricecoder-benchmark` - Build the final binary