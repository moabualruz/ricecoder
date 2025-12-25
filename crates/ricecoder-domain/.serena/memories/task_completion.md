# Task Completion Checklist

When completing a development task, run these commands in order:

1. **Format Code**: `cargo fmt` - Ensure consistent formatting
2. **Lint Code**: `cargo clippy` - Check for code quality issues
3. **Run Tests**: `cargo test` - Verify all tests pass
4. **Build**: `cargo build` - Ensure code compiles successfully
5. **Documentation**: `cargo doc` - Generate updated documentation

## CI/CD Equivalent
In automated environments, these checks should be equivalent to:
- Format check: `cargo fmt --check`
- Lint: `cargo clippy -- -D warnings`
- Tests: `cargo test`
- Build: `cargo build --release`

## Before Committing
Always run the full checklist before committing changes to ensure code quality and prevent CI failures.