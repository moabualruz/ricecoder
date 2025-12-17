# Contributing to RiceCoder

Thank you for your interest in contributing to RiceCoder! This document provides guidelines for contributing to the RiceCoder project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Environment Setup](#development-environment-setup)
4. [Contribution Types](#contribution-types)
5. [Development Workflow](#development-workflow)
6. [Code Review Process](#code-review-process)
7. [Testing Requirements](#testing-requirements)
8. [Documentation Requirements](#documentation-requirements)
9. [Release Process](#release-process)
10. [Support](#support)

## Code of Conduct

RiceCoder follows a code of conduct to ensure a welcoming environment for all contributors. We expect all contributors to:

- Be respectful and professional in all interactions
- Welcome diverse perspectives and experiences
- Provide constructive feedback
- Focus on what is best for the project
- Show empathy towards other community members

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust**: Version 1.75+ ([Install Rust](https://rustup.rs/))
- **Git**: Version control system
- **Terminal**: Modern terminal emulator (iTerm2, Windows Terminal, GNOME Terminal)
- **Optional**: Docker for containerized development

### Quick Setup

1. **Fork and Clone**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/ricecoder.git
   cd ricecoder
   git remote add upstream https://github.com/moabualruz/ricecoder.git
   ```

2. **Setup Development Environment**:
   ```bash
   # Run the automated setup script
   ./scripts/setup-dev-environment.sh
   ```

3. **Verify Setup**:
   ```bash
   cargo build
   cargo test
   rice --version
   ```

## Development Environment Setup

RiceCoder provides automated development environment setup to ensure consistency across all contributor machines.

### Automated Setup

Run the development environment setup script:

```bash
# Linux/macOS
./scripts/setup-dev-environment.sh

# Windows (PowerShell)
.\scripts\setup-dev-environment.ps1
```

This script will:
- ✅ Install required Rust toolchain and components
- ✅ Configure pre-commit hooks for code quality
- ✅ Set up development tools (clippy, rustfmt, cargo-audit)
- ✅ Install git hooks for automated checks
- ✅ Configure IDE settings (VS Code, IntelliJ)
- ✅ Set up local testing environment

### Manual Setup

If you prefer manual setup:

1. **Install Rust Toolchain**:
   ```bash
   rustup install 1.75
   rustup default 1.75
   rustup component add clippy rustfmt
   ```

2. **Install Development Tools**:
   ```bash
   cargo install cargo-audit cargo-outdated cargo-tarpaulin
   ```

3. **Configure Git Hooks**:
   ```bash
   cp scripts/pre-commit .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

4. **Setup Local Configuration**:
   ```bash
   cp config/dev.example.yaml config/dev.yaml
   # Edit config/dev.yaml with your settings
   ```

### IDE Configuration

#### VS Code
Install recommended extensions:
- rust-analyzer
- CodeLLDB
- Better TOML
- GitLens

#### IntelliJ/CLion
- Install Rust plugin
- Configure Rust toolchain in settings

### Testing Environment

RiceCoder includes comprehensive testing infrastructure:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with coverage
cargo tarpaulin --out Html

# Run integration tests
cargo test --test integration

# Run performance benchmarks
cargo bench
```

## Contribution Types

### 1. Bug Fixes

**Process**:
1. Create an issue describing the bug
2. Write a failing test case
3. Implement the fix
4. Ensure all tests pass

**Requirements**:
- Include test case demonstrating the fix
- Update documentation if needed
- Follow existing code patterns

### 2. New Features

**Process**:
1. Create an issue with feature specification
2. Discuss design and implementation approach
3. Implement feature with tests
4. Update documentation and examples

**Requirements**:
- Feature must align with project goals
- Include comprehensive tests
- Update user documentation
- Consider backward compatibility

### 3. Documentation

**Process**:
1. Identify documentation gap
2. Create or update documentation
3. Get review from maintainers

**Requirements**:
- Clear and concise writing
- Include examples where appropriate
- Follow documentation standards
- Test documentation builds correctly

### 4. Performance Improvements

**Process**:
1. Identify performance bottleneck
2. Create benchmark demonstrating issue
3. Implement optimization
4. Verify improvement with benchmarks

**Requirements**:
- Include before/after benchmarks
- Document performance impact
- Consider memory vs CPU trade-offs

### 5. Security Enhancements

**Process**:
1. Identify security issue or improvement
2. Implement secure solution
3. Add security tests
4. Update security documentation

**Requirements**:
- Follow security best practices
- Include security-focused tests
- Update SECURITY.md if needed

## Development Workflow

### 1. Choose an Issue

- Check [GitHub Issues](https://github.com/moabualruz/ricecoder/issues) for open tasks
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Branch

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-number-description

# Or for documentation
git checkout -b docs/update-contributing-guide
```

### 3. Make Changes

- Follow the [coding standards](#coding-standards)
- Write tests for new functionality
- Update documentation
- Commit frequently with clear messages

### 4. Run Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Run security audit
cargo audit

# Check coverage
cargo tarpaulin --fail-under 80
```

### 5. Commit Changes

```bash
# Stage changes
git add .

# Commit with conventional format
git commit -m "feat: add new feature description

- Add feature implementation
- Include tests
- Update documentation

Closes #123"
```

### 6. Push and Create Pull Request

```bash
# Push branch
git push origin feature/your-feature-name

# Create pull request on GitHub
# Fill out the PR template completely
```

## Code Review Process

RiceCoder uses a structured code review process to maintain code quality and consistency.

### Automated Checks

All pull requests must pass automated checks:

- ✅ **Build**: `cargo build --release`
- ✅ **Tests**: `cargo test` (minimum 80% coverage)
- ✅ **Linting**: `cargo clippy` (zero warnings)
- ✅ **Formatting**: `cargo fmt --check`
- ✅ **Security**: `cargo audit`
- ✅ **Performance**: Benchmark regression check

### Code Review Checklist

Reviewers will check:

#### Architecture & Design
- [ ] Follows hexagonal architecture principles
- [ ] Proper separation of concerns
- [ ] Dependency injection used correctly
- [ ] No circular dependencies
- [ ] Clean interfaces and abstractions

#### Code Quality
- [ ] Clear, readable code with good naming
- [ ] Appropriate error handling
- [ ] No unsafe code without justification
- [ ] Proper resource management
- [ ] Follows Rust best practices

#### Testing
- [ ] Unit tests for all public functions
- [ ] Integration tests for complex features
- [ ] Property-based tests where appropriate
- [ ] Error cases tested
- [ ] Test coverage maintained

#### Documentation
- [ ] Public APIs documented
- [ ] Complex logic explained
- [ ] Examples provided where helpful
- [ ] Changelog updated

#### Security
- [ ] Input validation implemented
- [ ] No sensitive data logged
- [ ] Secure defaults used
- [ ] Authentication/authorization checked

### Review Process

1. **Automated Checks**: PR must pass all CI checks
2. **Self-Review**: Author reviews their own code first
3. **Peer Review**: At least one maintainer reviews
4. **Discussion**: Address review comments
5. **Approval**: Maintainers approve when ready
6. **Merge**: Squash merge with conventional commit message

### Review Guidelines

**For Reviewers**:
- Be constructive and respectful
- Explain reasoning for suggestions
- Focus on code quality and maintainability
- Consider the bigger picture
- Acknowledge good work

**For Authors**:
- Address all review comments
- Explain disagreements with reasoning
- Make requested changes promptly
- Ask questions if unclear

## Testing Requirements

### Unit Tests

All code must include comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = "test";
        let expected = "result";

        // Act
        let result = process(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_handling() {
        // Test error cases
        let result = process("");
        assert!(result.is_err());
    }
}
```

### Integration Tests

For features spanning multiple components:

```rust
#[cfg(test)]
mod integration_tests {
    use ricecoder::{Config, App};

    #[tokio::test]
    async fn test_full_workflow() {
        // Setup
        let config = Config::default();
        let app = App::new(config).await.unwrap();

        // Execute workflow
        let result = app.process_request(request).await;

        // Verify
        assert!(result.is_ok());
    }
}
```

### Property-Based Tests

For complex logic, use property-based testing:

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parser_doesnt_crash(s in "\\PC*") {
            let result = parse(&s);
            // Property: parser should not crash on any input
            assert!(result.is_ok() || matches!(result, Err(ParseError::InvalidInput(_))));
        }
    }
}
```

### Performance Tests

Critical paths must include benchmarks:

```rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn benchmark_critical_function(c: &mut Criterion) {
        c.bench_function("critical_function", |b| {
            b.iter(|| {
                black_box(critical_function(black_box(test_data())))
            })
        });
    }

    criterion_group!(benches, benchmark_critical_function);
    criterion_main!(benches);
}
```

## Documentation Requirements

### Code Documentation

All public APIs must be documented:

```rust
/// Processes user input and returns formatted result.
///
/// This function handles the main processing pipeline for user requests,
/// including validation, transformation, and output formatting.
///
/// # Arguments
///
/// * `input` - The raw user input string
/// * `config` - Processing configuration
///
/// # Returns
///
/// Returns a `Result` containing the processed output or an error.
///
/// # Examples
///
/// ```
/// let result = process_input("hello", &config)?;
/// assert_eq!(result, "HELLO");
/// ```
pub fn process_input(input: &str, config: &Config) -> Result<String, Error> {
    // Implementation
}
```

### User Documentation

For new features, update appropriate documentation:

- **README.md**: High-level feature overview
- **Wiki**: Detailed usage guides
- **API Reference**: Technical API documentation
- **Examples**: Code examples and tutorials

### Changelog

Update CHANGELOG.md for all user-facing changes:

```markdown
## [0.1.8] - 2025-12-16

### Added
- New feature description (#123)

### Changed
- Modified behavior description

### Fixed
- Bug fix description (#124)

### Security
- Security improvement description
```

## Release Process

RiceCoder follows semantic versioning and automated releases.

### Version Types

- **Patch** (`0.1.8`): Bug fixes, no breaking changes
- **Minor** (`0.2.0`): New features, backward compatible
- **Major** (`1.0.0`): Breaking changes

### Release Checklist

- [ ] All tests pass
- [ ] Code coverage >= 80%
- [ ] No clippy warnings
- [ ] Security audit passes
- [ ] Performance benchmarks pass
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Release notes written

### Automated Release

Releases are automated via GitHub Actions:

1. Create release branch: `git checkout -b release/v0.1.8`
2. Update version in Cargo.toml
3. Update CHANGELOG.md
4. Create pull request
5. Merge to main triggers release

## Support

### Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Discord**: Real-time community support

### Issue Reporting

When reporting issues, include:

- RiceCoder version (`rice --version`)
- Rust version (`rustc --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Error messages and stack traces
- Configuration (redact sensitive data)

### Feature Requests

Feature requests should include:

- Clear description of the feature
- Use case and motivation
- Proposed implementation (optional)
- Impact on existing functionality

---

## Quick Reference

### Development Commands

```bash
# Setup
./scripts/setup-dev-environment.sh

# Development
cargo build                    # Build project
cargo test                     # Run tests
cargo clippy                   # Run linter
cargo fmt                      # Format code
cargo audit                    # Security audit

# Testing
cargo test --test integration  # Integration tests
cargo bench                    # Benchmarks
cargo tarpaulin                # Coverage

# Documentation
cargo doc --open               # Generate docs
```

### Git Workflow

```bash
# Start work
git checkout -b feature/name
git commit -m "feat: description"

# Update from main
git fetch upstream
git rebase upstream/main

# Push and create PR
git push origin feature/name
```

### Code Standards

- Use `rustfmt` for formatting
- Follow `clippy` suggestions
- Write comprehensive tests
- Document public APIs
- Use meaningful names
- Handle errors properly

---

Thank you for contributing to RiceCoder! Your contributions help make RiceCoder better for everyone.

*Last updated: 2025-12-16*
