# Task Completion Checklist

When completing a development task, run the following commands in order to ensure code quality and correctness:

1. **Format Code**: `cargo fmt` - Ensures consistent code formatting
2. **Lint Code**: `cargo clippy` - Checks for common mistakes and style issues
3. **Run Tests**: `cargo test` - Verifies that all functionality works correctly
4. **Build Project**: `cargo build` - Confirms the code compiles successfully

## Optional but Recommended
- `cargo doc` - Generate and check documentation
- `cargo test --release` - Run tests in release mode for additional checks

These commands should be run after any code changes to maintain project standards and catch issues early.