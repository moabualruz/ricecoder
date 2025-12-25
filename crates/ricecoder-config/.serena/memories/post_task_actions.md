# Actions to Perform When a Task is Completed

After completing any code changes or additions, run the following commands to ensure code quality and correctness:

1. **Format the code**: `cargo fmt`
   - Ensures consistent code formatting according to Rust standards

2. **Run the linter**: `cargo clippy`
   - Checks for common mistakes, performance issues, and style improvements

3. **Run tests**: `cargo test`
   - Verifies that all functionality works correctly and no regressions were introduced

4. **Build the project**: `cargo build`
   - Ensures the code compiles successfully

If any of these commands fail, fix the issues before considering the task complete.