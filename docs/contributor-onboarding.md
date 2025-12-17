# RiceCoder Contributor Onboarding Guide

Welcome to the RiceCoder project! This guide will help you get started as a contributor, understand the project structure, and make your first contribution.

## Table of Contents

1. [Welcome](#welcome)
2. [Project Overview](#project-overview)
3. [Getting Started](#getting-started)
4. [Development Environment](#development-environment)
5. [Understanding the Codebase](#understanding-the-codebase)
6. [Making Your First Contribution](#making-your-first-contribution)
7. [Development Workflow](#development-workflow)
8. [Testing](#testing)
9. [Code Review Process](#code-review-process)
10. [Getting Help](#getting-help)
11. [Next Steps](#next-steps)

## Welcome

Thank you for your interest in contributing to RiceCoder! RiceCoder is a terminal-first, spec-driven coding assistant that helps developers write better code faster. Whether you're fixing bugs, adding features, improving documentation, or helping with testing, your contributions are valuable.

This guide assumes you have basic knowledge of:
- Rust programming language
- Git version control
- Command line interfaces
- Software development concepts

If you're new to any of these, don't worry - we'll guide you through the process.

## Project Overview

### What is RiceCoder?

RiceCoder (`rice`) is a terminal-first, spec-driven coding assistant that understands your project before generating code. Unlike traditional AI coding tools, RiceCoder:

- **Research-First**: Analyzes your codebase, understands patterns, and generates contextually appropriate code
- **Spec-Driven**: Uses specifications to guide systematic development
- **Terminal-Native**: Beautiful CLI/TUI that works anywhere
- **Multi-Provider**: Supports OpenAI, Anthropic, Ollama, and 70+ other providers
- **Enterprise-Ready**: SOC 2 compliance, audit logging, RBAC, and security features

### Key Features

- 🔬 **Research & Analysis**: Project understanding and pattern recognition
- 📋 **Spec-Driven Development**: Systematic code generation from specifications
- 💻 **Terminal Interface**: CLI and TUI for all operations
- 🤖 **Multi-Agent System**: Specialized agents for different tasks
- 🔒 **Enterprise Security**: SOC 2 compliance and enterprise features
- 📊 **Token Tracking**: Real-time usage monitoring and cost estimation
- 🚀 **Project Bootstrap**: Automatic project detection and configuration
- 📱 **Session Management**: Persistent sessions with sharing capabilities

### Architecture

RiceCoder follows **Hexagonal Architecture** (Ports & Adapters):

```
┌─────────────────────────────────────┐
│           Interfaces                │  ← CLI, TUI, API
│          (Adapters)                 │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│         Application                 │  ← Use Cases, Commands, Queries
│           (Core)                    │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│           Domain                    │  ← Entities, Value Objects, Business Rules
│         (Business Logic)            │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│       Infrastructure               │  ← Database, External APIs, File System
│         (Adapters)                  │
└─────────────────────────────────────┘
```

### Technology Stack

- **Language**: Rust 1.75+
- **Architecture**: Hexagonal Architecture with Dependency Injection
- **Testing**: Comprehensive unit, integration, and property-based tests
- **CI/CD**: GitHub Actions with automated quality checks
- **Documentation**: Markdown with automated link checking
- **Package Management**: Cargo with workspace structure

## Getting Started

### Prerequisites

Ensure you have these tools installed:

1. **Rust**: Version 1.75 or later
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Verify installation
   rustc --version  # Should show 1.75+
   cargo --version
   ```

2. **Git**: Version control system
   ```bash
   # Check if installed
   git --version

   # Configure Git (replace with your info)
   git config --global user.name "Your Name"
   git config --global user.email "your.email@example.com"
   ```

3. **Terminal**: Modern terminal emulator
   - Linux/macOS: Default terminal or iTerm2
   - Windows: Windows Terminal or PowerShell

### Fork and Clone

1. **Fork the Repository**:
   - Go to https://github.com/moabualruz/ricecoder
   - Click "Fork" in the top right
   - This creates your own copy of the repository

2. **Clone Your Fork**:
   ```bash
   # Clone your fork
   git clone https://github.com/YOUR_USERNAME/ricecoder.git
   cd ricecoder

   # Add upstream remote for staying updated
   git remote add upstream https://github.com/moabualruz/ricecoder.git
   ```

3. **Verify Setup**:
   ```bash
   # Check remotes
   git remote -v

   # Should show:
   # origin    https://github.com/YOUR_USERNAME/ricecoder.git (fetch)
   # origin    https://github.com/YOUR_USERNAME/ricecoder.git (push)
   # upstream  https://github.com/moabualruz/ricecoder.git (fetch)
   # upstream  https://github.com/moabualruz/ricecoder.git (push)
   ```

## Development Environment

### Automated Setup

RiceCoder provides automated development environment setup:

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
- ✅ Install Git hooks for automated checks
- ✅ Configure IDE settings (VS Code, IntelliJ)
- ✅ Set up local testing environment

### Manual Setup

If you prefer manual setup:

1. **Install Development Tools**:
   ```bash
   # Install additional Cargo tools
   cargo install cargo-audit cargo-outdated cargo-tarpaulin cargo-nextest cargo-watch
   ```

2. **Configure Git Hooks**:
   ```bash
   # Copy pre-commit hook
   cp scripts/pre-commit .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

3. **Setup Development Configuration**:
   ```bash
   # Copy development config
   cp config/dev.example.yaml config/dev.yaml
   # Edit config/dev.yaml with your settings
   ```

### IDE Configuration

#### VS Code (Recommended)

Install these extensions:
- `rust-lang.rust-analyzer` - Rust language support
- `vadimcn.vscode-lldb` - Debugger
- `tamasfe.even-better-toml` - TOML support
- `ms-vscode.vscode-json` - JSON support

VS Code settings are automatically configured by the setup script.

#### IntelliJ IDEA/CLion

1. Install the Rust plugin
2. Configure Rust toolchain in settings
3. Enable Cargo integration

### Verify Environment

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt

# Check security
cargo audit

# Run the application
cargo run -- --help
```

## Understanding the Codebase

### Project Structure

```
ricecoder/
├── src/                          # Main source code
│   ├── cli/                      # Command-line interface
│   ├── tui/                      # Terminal user interface
│   ├── providers/                # AI provider integrations
│   ├── generation/               # Code generation logic
│   ├── research/                 # Project analysis and research
│   ├── sessions/                 # Session management
│   ├── mcp/                      # Model Context Protocol
│   ├── domain/                   # Domain entities and business logic
│   ├── application/              # Application use cases
│   ├── infrastructure/           # Infrastructure adapters
│   └── lib.rs                    # Library entry point
├── tests/                        # Integration tests
│   ├── fixtures/                 # Test data
│   ├── integration/              # Integration test suites
│   └── property/                 # Property-based tests
├── benches/                      # Performance benchmarks
├── config/                       # Configuration files
├── docs/                         # Documentation
├── scripts/                      # Build and utility scripts
├── .github/                      # GitHub Actions and templates
├── Cargo.toml                    # Package configuration
├── CONTRIBUTING.md               # Contribution guidelines
├── README.md                     # Project documentation
└── rustfmt.toml                  # Code formatting configuration
```

### Key Concepts

#### 1. Hexagonal Architecture

RiceCoder follows Hexagonal Architecture to maintain clean separation of concerns:

- **Domain Layer**: Core business logic, entities, value objects
- **Application Layer**: Use cases, commands, queries, orchestration
- **Infrastructure Layer**: Database, external APIs, file system
- **Interface Layer**: CLI, TUI, API endpoints

#### 2. Dependency Injection

Services are wired together using a dependency injection container:

```rust
// Service definition
pub struct CodeGenerationService {
    pub analyzer: Arc<dyn ProjectAnalyzer>,
    pub generator: Arc<dyn CodeGenerator>,
    pub validator: Arc<dyn CodeValidator>,
}

// Usage through DI container
let service = container.get::<CodeGenerationService>()?;
```

#### 3. MCP (Model Context Protocol)

RiceCoder integrates with external tools via MCP:

```rust
// MCP server definition
pub struct McpServer {
    pub tools: Vec<McpTool>,
    pub transport: McpTransport,
}

// Tool execution
let result = server.execute_tool("filesystem.read", params).await?;
```

#### 4. Session Management

Sessions provide persistent context:

```rust
// Session with conversation history
pub struct Session {
    pub id: SessionId,
    pub messages: Vec<Message>,
    pub context: ProjectContext,
    pub metadata: SessionMetadata,
}
```

### Code Organization Patterns

#### Module Structure

Each crate follows this pattern:

```rust
// lib.rs - Public API
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;

// Re-export main types
pub use domain::{Project, Session, Provider};
pub use application::{CodeGenerationUseCase, AnalysisUseCase};
```

#### Error Handling

RiceCoder uses custom error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RiceCoderError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Application error: {0}")]
    Application(#[from] ApplicationError),

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),
}
```

#### Testing Patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_code_generation() {
        // Arrange
        let mut container = TestContainer::new();
        let service = container.get::<CodeGenerationService>();

        // Act
        let result = service.generate(spec).await;

        // Assert
        assert!(result.is_ok());
    }
}
```

## Making Your First Contribution

### Find an Issue

1. **Check GitHub Issues**: Look for issues labeled `good first issue` or `help wanted`
2. **Browse by Category**:
   - 🐛 `bug` - Bug fixes
   - ✨ `enhancement` - New features
   - 📚 `documentation` - Documentation improvements
   - 🧪 `testing` - Test improvements
   - 🔒 `security` - Security enhancements

3. **Comment on Issues**: Indicate you're working on an issue

### Create a Branch

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-number-description

# Or for documentation
git checkout -b docs/update-contributing-guide
```

### Make Changes

1. **Follow Code Standards**:
   - Use `cargo fmt` for formatting
   - Follow `cargo clippy` suggestions
   - Write comprehensive tests
   - Document public APIs

2. **Write Tests First** (TDD approach):
   ```rust
   #[test]
   fn test_my_new_feature() {
       // Test implementation
   }
   ```

3. **Commit Regularly**:
   ```bash
   git add .
   git commit -m "feat: add new feature implementation"
   ```

### Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_my_feature

# Run with coverage
cargo tarpaulin --out Html

# Run integration tests
cargo test --test integration

# Check performance
cargo bench
```

### Submit a Pull Request

1. **Push Your Branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**:
   - Go to your fork on GitHub
   - Click "Compare & pull request"
   - Fill out the PR template
   - Link related issues
   - Request reviewers

3. **PR Template**:
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Documentation
   - [ ] Refactoring

   ## Testing
   - [ ] Unit tests added
   - [ ] Integration tests added
   - [ ] Manual testing performed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Tests pass
   - [ ] Documentation updated
   - [ ] No breaking changes
   ```

## Development Workflow

### Daily Workflow

1. **Update Your Branch**:
   ```bash
   # Fetch latest changes
   git fetch upstream

   # Rebase on main
   git rebase upstream/main
   ```

2. **Make Changes**:
   ```bash
   # Edit files
   # Run tests frequently
   cargo test

   # Commit changes
   git add .
   git commit -m "feat: implement feature"
   ```

3. **Push and Update PR**:
   ```bash
   git push origin feature/branch-name
   ```

### Code Quality Checks

Run these before pushing:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Check security
cargo audit

# Check coverage
cargo tarpaulin --fail-under 80
```

### Commit Message Format

Use conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Testing
- `chore:` - Maintenance

Examples:
```
feat: add MCP server support
fix: handle empty file gracefully
docs: update API documentation
refactor: simplify error handling logic
test: add property tests for parser
```

## Testing

### Testing Strategy

RiceCoder uses comprehensive testing:

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test component interactions
3. **Property-Based Tests**: Test with generated inputs
4. **End-to-End Tests**: Test complete user workflows
5. **Performance Tests**: Ensure performance requirements are met

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# Integration tests only
cargo test --test integration

# With output
cargo test -- --nocapture

# With coverage
cargo tarpaulin --out Html

# Performance benchmarks
cargo bench
```

### Writing Tests

#### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_generation_success() {
        // Arrange
        let spec = CodeSpec::new("test");
        let generator = CodeGenerator::new();

        // Act
        let result = generator.generate(&spec);

        // Assert
        assert!(result.is_ok());
        assert!(!result.unwrap().code.is_empty());
    }

    #[test]
    fn test_code_generation_failure() {
        // Arrange
        let spec = CodeSpec::new(""); // Invalid spec
        let generator = CodeGenerator::new();

        // Act
        let result = generator.generate(&spec);

        // Assert
        assert!(result.is_err());
    }
}
```

#### Integration Test Example

```rust
#[cfg(test)]
mod integration_tests {
    use ricecoder::test_utils::*;
    use ricecoder::{Config, App};

    #[tokio::test]
    async fn test_full_code_generation_workflow() {
        // Setup
        let config = TestConfig::default();
        let app = App::new(config).await.unwrap();

        // Execute workflow
        let result = app.generate_code_from_spec(spec).await;

        // Verify
        assert!(result.is_ok());
        assert!(generated_files_exist(&result.unwrap()));
    }
}
```

#### Property-Based Test Example

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_parser_doesnt_crash_on_any_input(s in "\\PC*") {
            let result = parse_code(&s);
            // Parser should not crash on any input
            assert!(result.is_ok() || matches!(result, Err(ParseError::InvalidInput(_))));
        }

        #[test]
        fn test_generated_code_is_valid_rust(code in valid_rust_code_strategy()) {
            let result = validate_rust_code(&code);
            // Generated code should be valid Rust
            prop_assert!(result.is_ok());
        }
    }
}
```

### Test Coverage

Aim for 80%+ code coverage:

```bash
# Check coverage
cargo tarpaulin --out Html

# Open coverage report
# Opens coverage report in browser
```

## Code Review Process

### Automated Checks

All PRs must pass:

- ✅ **Build**: `cargo build --release`
- ✅ **Tests**: `cargo test` (≥80% coverage)
- ✅ **Linting**: `cargo clippy` (zero warnings)
- ✅ **Formatting**: `cargo fmt --check`
- ✅ **Security**: `cargo audit`
- ✅ **Performance**: Benchmark checks

### Code Review Checklist

Reviewers check:

- [ ] **Architecture**: Follows hexagonal architecture
- [ ] **Code Quality**: Readable, well-documented code
- [ ] **Testing**: Comprehensive test coverage
- [ ] **Security**: No security vulnerabilities
- [ ] **Performance**: Meets performance requirements
- [ ] **Documentation**: Updated documentation

### Review Guidelines

**For Contributors**:
- Address all review comments
- Explain your reasoning
- Make requested changes
- Ask questions if unclear

**For Reviewers**:
- Be constructive and respectful
- Explain reasoning for suggestions
- Focus on important issues
- Acknowledge good work

## Getting Help

### Documentation

- **README.md**: Project overview and installation
- **CONTRIBUTING.md**: Detailed contribution guidelines
- **docs/**: Comprehensive documentation
- **API Docs**: `cargo doc --open`

### Community Support

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Discord**: Real-time community support

### Finding Issues

- **Good First Issues**: Labeled `good first issue`
- **Help Wanted**: Labeled `help wanted`
- **Bugs**: Labeled `bug`
- **Documentation**: Labeled `documentation`

### Asking for Help

When asking for help:

1. **Check Documentation**: Search existing docs first
2. **Search Issues**: Look for similar issues
3. **Be Specific**: Include error messages, code snippets, environment details
4. **Provide Context**: Explain what you're trying to accomplish

## Next Steps

### After Your First Contribution

1. **Celebrate!** 🎉 You've made your first contribution
2. **Learn More**: Explore different areas of the codebase
3. **Get Involved**: Join community discussions
4. **Mentor Others**: Help new contributors

### Advanced Topics

Once comfortable with basics:

- **MCP Integration**: Learn about Model Context Protocol
- **Provider Ecosystem**: Understand AI provider integrations
- **Session Management**: Explore session persistence and sharing
- **Enterprise Features**: Learn about security and compliance
- **Performance Optimization**: Contribute to performance improvements

### Career Development

Contributing to RiceCoder can help you:

- **Learn Rust**: Deepen your Rust programming skills
- **Architecture**: Understand clean architecture patterns
- **Testing**: Master comprehensive testing strategies
- **Open Source**: Gain experience with open source development
- **AI/ML**: Learn about AI integration and prompt engineering
- **DevOps**: Understand CI/CD and automated quality assurance

### Recognition

Contributors are recognized through:

- **GitHub Contributors**: Listed in repository contributors
- **Changelog**: Mentioned in release notes
- **Community**: Featured in community updates
- **Opportunities**: Invited to contributor events and discussions

---

## Quick Reference

### Essential Commands

```bash
# Setup
./scripts/setup-dev-environment.sh

# Development
cargo build                    # Build project
cargo test                     # Run tests
cargo clippy                   # Run linter
cargo fmt                      # Format code
cargo doc --open               # View documentation

# Testing
cargo test --test integration  # Integration tests
cargo bench                    # Benchmarks
cargo tarpaulin                # Coverage

# Git workflow
git checkout -b feature/name   # Create branch
git add .                      # Stage changes
git commit -m "feat: message"  # Commit
git push origin branch-name    # Push
```

### File Locations

- **Source Code**: `src/`
- **Tests**: `tests/`
- **Configuration**: `config/`
- **Documentation**: `docs/`
- **Scripts**: `scripts/`
- **CI/CD**: `.github/`

### Key Files

- **Cargo.toml**: Package configuration
- **rustfmt.toml**: Code formatting rules
- **CONTRIBUTING.md**: Contribution guidelines
- **docs/code-review-checklist.md**: Code review checklist

---

Thank you for joining the RiceCoder community! Your contributions help make RiceCoder better for developers everywhere.

**Happy coding! 🚀**

*Last updated: 2025-12-16*