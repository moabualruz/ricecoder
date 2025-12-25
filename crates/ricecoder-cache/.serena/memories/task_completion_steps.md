# Steps to Perform After Completing a Development Task

## Code Quality Checks
1. **Format code**: Run `cargo fmt` to ensure consistent formatting
2. **Lint code**: Run `cargo clippy` to check for potential issues and improvements
3. **Run tests**: Execute `cargo test` to verify all functionality works correctly
4. **Run benchmarks**: If performance changes were made, run `cargo bench` to ensure no regressions

## Build Verification
1. **Build in debug mode**: `cargo build` to check for compilation errors
2. **Build in release mode**: `cargo build --release` for production-ready build

## Documentation
1. **Update docs if needed**: Ensure any new public APIs are documented
2. **Generate docs**: Run `cargo doc` to update documentation

## Commit Preparation
1. **Stage changes**: Use `git add` for relevant files
2. **Commit with descriptive message**: Follow conventional commit format if applicable
3. **Push changes**: If working on a branch, push to remote

## Additional Checks
- Ensure no sensitive information is committed
- Verify that all dependencies are properly managed
- Check that the code follows the established conventions (see code_style_and_conventions.md)