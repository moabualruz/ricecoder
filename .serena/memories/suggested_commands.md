# Essential Development Commands

## Build and Run
- `cargo build` - Build the entire workspace
- `cargo build --release` - Build optimized release version
- `cargo run --bin ricecoder-cli` - Run the main CLI application
- `cargo run --bin ricecoder-cli -- --help` - Show CLI help

## Testing
- `cargo test` - Run all unit tests
- `cargo test --test integration` - Run integration tests
- `cargo test --package ricecoder-cli` - Test specific crate
- `cargo bench` - Run performance benchmarks
- `cargo tarpaulin --fail-under 80` - Check test coverage (requires cargo-tarpaulin)

## Code Quality
- `cargo fmt` - Format all code with rustfmt
- `cargo clippy` - Run linter and fix suggestions
- `cargo audit` - Check for security vulnerabilities
- `cargo outdated` - Check for outdated dependencies

## Development Scripts
- `./scripts/setup-dev-environment.sh` - Setup development environment (Linux/macOS)
- `.\scripts\setup-dev-environment.ps1` - Setup development environment (Windows)
- `./scripts/cargo-build-recursive.ps1` - Build all crates recursively
- `./scripts/cargo-clippy-recursive.ps1` - Run clippy on all crates
- `./scripts/run-performance-validation.sh` - Run performance validation

## Installation and Verification
- `cargo install --path crates/ricecoder-cli` - Install from source
- `rice --version` - Verify installation
- `rice init` - Initialize project
- `rice chat` - Start interactive chat

## Windows-Specific Commands
- `dir` - List directory contents (equivalent to ls)
- `cd` - Change directory
- `findstr` - Search for strings in files (equivalent to grep)
- `dir /s` - Find files recursively (equivalent to find)
- `git` - Version control operations

## Documentation
- `cargo doc --open` - Generate and open documentation
- `./scripts/validate-docs.sh` - Validate documentation links
- `./scripts/check-documentation-completeness.js` - Check documentation completeness

## Release and Publishing
- `./scripts/generate-release-notes.sh` - Generate release notes
- `./scripts/publish.sh` - Publish to crates.io
- `./scripts/update-performance-baselines.sh` - Update performance baselines