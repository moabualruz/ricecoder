# What to Do When a Task is Completed

## Code Quality Checks
1. **Run Tests**: Execute `cargo test` to ensure all tests pass
2. **Format Code**: Run `cargo fmt` to format code according to Rust standards
3. **Lint Code**: Run `cargo clippy` to catch potential issues and improve code quality
4. **Check Compilation**: Run `cargo check` to verify no compilation errors

## Documentation
1. **Update Documentation**: Ensure any new public APIs are documented with `///` comments
2. **Generate Docs**: Run `cargo doc` to verify documentation builds correctly
3. **Update README**: If functionality changes, update README.md with new features or usage examples

## Testing
1. **Add Tests**: Ensure new functionality has appropriate unit tests
2. **Run Integration Tests**: Execute integration tests to verify end-to-end functionality
3. **Property Tests**: Run proptest-based tests for complex logic
4. **Regression Tests**: Ensure existing functionality still works

## Commit and Push
1. **Stage Changes**: Use `git add .` or selectively stage relevant files
2. **Commit**: Use `git commit -m "descriptive message"` with clear, concise commit messages
3. **Push**: Push changes to remote repository if applicable

## Verification
1. **Build Artifacts**: Ensure the crate builds successfully
2. **Dependency Checks**: Verify no new security vulnerabilities in dependencies
3. **Performance**: Check that changes don't negatively impact performance
4. **Compatibility**: Ensure changes work with supported LSP servers (rust-analyzer, typescript-language-server, pylsp)

## Specific to This Project
- **LSP Compliance**: Verify LSP protocol compliance for any protocol-related changes
- **Fallback Behavior**: Test that fallback to internal providers works when external LSP unavailable
- **Process Management**: Ensure proper cleanup of LSP server processes
- **Configuration**: Test configuration loading and validation
- **Health Monitoring**: Verify health checks and restart logic work correctly