# Commands to Run When Task is Completed

## Code Quality Checks (Required)
- `cargo fmt` - Format code with rustfmt
- `cargo clippy` - Run linter (must pass with zero warnings)
- `cargo test` - Run all tests (must pass)
- `cargo audit` - Security vulnerability check (must pass)

## Coverage and Performance
- `cargo tarpaulin --fail-under 80` - Test coverage check (minimum 80%)
- `cargo bench` - Run benchmarks to ensure no performance regression

## Validation Scripts
- `./scripts/run-performance-validation.sh` - Performance validation
- `./scripts/check-documentation-completeness.js` - Documentation completeness check

## Pre-commit Checks
- All code must be formatted
- All clippy warnings must be resolved
- All tests must pass
- Security audit must pass
- Test coverage must be maintained

## CI/CD Requirements
- Build must succeed: `cargo build --release`
- All automated checks must pass
- No performance regressions
- Documentation must build correctly

## Manual Verification
- `cargo run --bin ricecoder-cli -- --version` - Verify binary builds and runs
- `rice init` - Test initialization (if applicable)
- Manual testing of new features

## Documentation Updates
- Update CHANGELOG.md for user-facing changes
- Update relevant documentation files
- Ensure rustdoc examples work