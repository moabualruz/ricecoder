# Post-Task Checklist

After completing any code changes, run these commands in sequence:

## 1. Format Code
```bash
cargo fmt
```
Ensures consistent code formatting according to Rust standards.

## 2. Run Linter
```bash
cargo clippy
```
Catches potential bugs, performance issues, and style violations.

## 3. Check Compilation
```bash
cargo check
```
Verifies code compiles without errors (faster than full build).

## 4. Run Tests
```bash
cargo test
```
Executes all unit tests and integration tests to ensure functionality works.

## 5. Build Release (Optional)
```bash
cargo build --release
```
Creates optimized build for performance verification.

## 6. Generate Documentation (Optional)
```bash
cargo doc
```
Updates documentation for any new public APIs.

## Commit Guidelines
- Ensure all checks pass before committing
- Write clear commit messages describing the change
- Use conventional commit format if applicable (feat:, fix:, refactor:, etc.)

## CI/CD Notes
This project likely runs in a CI environment that executes:
- `cargo fmt --check` (fail if not formatted)
- `cargo clippy` (fail on warnings)
- `cargo test` (fail on test failures)
- `cargo build --release` (fail on build errors)

Always run these locally first to avoid CI failures.