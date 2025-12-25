# Task Completion Checklist

When completing a development task, ensure the following:

## Code Quality
- [ ] Run `cargo fmt` to format code
- [ ] Run `cargo clippy` to check for linting issues
- [ ] Run `cargo check` to verify compilation
- [ ] Run `cargo test` to ensure all tests pass

## Documentation
- [ ] Update README.md if new CLI commands added
- [ ] Add/update `///` documentation for new public APIs
- [ ] Run `cargo doc` to verify documentation builds

## Testing
- [ ] Add comprehensive tests for new features
- [ ] Ensure >80% test coverage maintained
- [ ] Test with enterprise deployment scenarios
- [ ] Validate compliance implications of changes

## Compliance and Validation
- [ ] Test SOC 2, GDPR, HIPAA compliance if relevant
- [ ] Validate enterprise requirements (performance, integration)
- [ ] Run beta testing program: `ricecoder-beta run --output-dir beta-reports`

## Commit and Review
- [ ] Commit changes with descriptive message
- [ ] Ensure no sensitive information in commits
- [ ] Test in clean environment if possible