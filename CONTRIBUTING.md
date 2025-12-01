# Contributing to RiceCoder

Thank you for your interest in contributing to RiceCoder! This document provides guidelines for contributing.

---

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.

---

## How to Contribute

### Reporting Bugs

1. Check if the issue already exists
2. Create a new issue with:
   - Clear title
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)

### Suggesting Features

1. Check existing issues and discussions
2. Create a feature request with:
   - Clear description of the feature
   - Use case / why it's needed
   - Possible implementation approach

### Submitting Code

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run lints: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit with clear messages
8. Push and create a Pull Request

---

## Development Setup

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Git

### Getting Started

```bash
# Clone the repo
git clone https://github.com/yourusername/ricecoder.git
cd ricecoder

# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

---

## Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write doc comments for public APIs
- Add tests for new functionality

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add support for Ollama provider
fix: handle empty file gracefully
docs: update README with installation instructions
refactor: simplify command routing logic
test: add property tests for template rendering
```

Prefixes:
- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation
- `refactor:` — Code refactoring
- `test:` — Tests
- `chore:` — Maintenance

---

## Project Structure

```
ricecoder/
├── src/
│   ├── cli/          # CLI commands
│   ├── tui/          # Terminal UI
│   ├── providers/    # AI providers
│   ├── generation/   # Code generation
│   ├── specs/        # Spec system
│   └── ...
├── tests/            # Integration tests
├── docs/             # Documentation
└── .branding/        # Logo and assets
```

---

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add tests for new functionality
4. Request review from maintainers
5. Address feedback
6. Squash commits if requested

---

## License

By contributing, you agree that your contributions will be licensed under the same [CC BY-NC-SA 4.0](LICENSE.md) license as the project.

---

## Questions?

Open an issue or start a discussion. We're happy to help!

---

Thank you for contributing! 🎉
