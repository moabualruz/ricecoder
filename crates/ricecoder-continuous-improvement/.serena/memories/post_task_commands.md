# Commands to Run When Task is Completed

After completing any code changes, run the following commands to ensure quality:

1. `cargo fmt` - Format the code
2. `cargo clippy` - Check for linting issues
3. `cargo test -p ricecoder-continuous-improvement` - Run the tests
4. `cargo build` - Ensure the code compiles

If using test features: `cargo test -p ricecoder-continuous-improvement --features test-utils`