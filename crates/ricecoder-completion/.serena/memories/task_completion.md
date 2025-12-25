# What to Do When a Task is Completed

After completing any development task (adding features, fixing bugs, refactoring), run the following commands in sequence:

1. **Format code**: `cargo fmt -p ricecoder-completion`
   - Ensures consistent code formatting

2. **Lint code**: `cargo clippy -p ricecoder-completion`
   - Catches potential issues and enforces best practices

3. **Run tests**: `cargo test -p ricecoder-completion`
   - Verifies functionality and prevents regressions

4. **Build project**: `cargo build -p ricecoder-completion`
   - Ensures the code compiles successfully

## Quick Check Commands
- **Format check only**: `cargo fmt --check -p ricecoder-completion`
- **Lint check only**: `cargo clippy -p ricecoder-completion -- -D warnings`
- **Test check only**: `cargo test -p ricecoder-completion`

## Integration with Git
Before committing, ensure all checks pass:
```bash
cargo fmt --check -p ricecoder-completion && cargo clippy -p ricecoder-completion && cargo test -p ricecoder-completion
```

## Performance Considerations
- Tests include property tests for correctness verification
- LSP integration tests verify external server communication
- Ghost text tests ensure inline completion functionality

## Error Handling
If any command fails:
- Fix the reported issues
- Re-run the failed command
- Continue with remaining checks
- Do not commit until all checks pass