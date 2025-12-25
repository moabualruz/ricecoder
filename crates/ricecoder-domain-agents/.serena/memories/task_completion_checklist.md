# Task Completion Checklist

After completing any development task, run the following commands to ensure code quality:

1. **Format Code**: `cargo fmt`
   - Ensures consistent code formatting

2. **Lint Code**: `cargo clippy`
   - Checks for code quality issues and potential bugs
   - Fix any warnings or errors reported

3. **Run Tests**: `cargo test -p ricecoder-domain-agents`
   - Ensures all existing functionality still works
   - Runs unit tests and integration tests

4. **Build Check**: `cargo check`
   - Verifies code compiles without errors

5. **Documentation Check**: `cargo doc`
   - Ensures documentation builds correctly

## Guidelines for Tasks
- **Domain Expertise**: Ensure agents have deep knowledge of their technology stack
- **Best Practices**: Implement current best practices for each framework
- **Testing**: Test agents with real-world scenarios for each domain
- **Documentation**: Document supported frameworks and limitations
- **Error Handling**: Provide meaningful error messages and proper error propagation