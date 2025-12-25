# Post-Task Commands

After completing any code changes or additions, run the following commands in sequence to ensure code quality and correctness:

1. `cargo fmt` - Format the code to maintain consistent style
2. `cargo clippy` - Run the linter to catch potential issues and style violations
3. `cargo check` - Verify that the code compiles without errors
4. `cargo test` - Run all tests to ensure functionality is preserved

If any of these commands fail, fix the issues before considering the task complete. The linter and tests are particularly important for maintaining code quality in this project.