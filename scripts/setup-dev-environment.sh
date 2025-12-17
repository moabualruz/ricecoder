#!/bin/bash

# RiceCoder Development Environment Setup Script
# This script sets up a complete development environment for RiceCoder contributors

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Setup Rust toolchain
setup_rust() {
    log_info "Setting up Rust toolchain..."

    if ! command_exists rustc; then
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        log_info "Rust already installed: $(rustc --version)"
    fi

    # Install/update Rust 1.75
    log_info "Installing Rust 1.75..."
    rustup install 1.75
    rustup default 1.75

    # Install components
    log_info "Installing Rust components..."
    rustup component add clippy rustfmt

    # Install cargo tools
    log_info "Installing Cargo tools..."
    cargo install cargo-audit cargo-outdated cargo-tarpaulin cargo-nextest cargo-watch

    log_success "Rust toolchain setup complete"
}

# Setup Git hooks
setup_git_hooks() {
    log_info "Setting up Git hooks..."

    # Create pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

# RiceCoder pre-commit hook
# Runs basic quality checks before committing

set -e

echo "Running pre-commit checks..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Cargo not found, skipping checks"
    exit 0
fi

# Run format check
echo "Checking code formatting..."
if ! cargo fmt --check; then
    echo "Code is not properly formatted. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy
echo "Running clippy..."
if ! cargo clippy -- -D warnings; then
    echo "Clippy found issues. Fix them before committing."
    exit 1
fi

# Run tests
echo "Running tests..."
if ! cargo test; then
    echo "Tests failed. Fix them before committing."
    exit 1
fi

echo "All checks passed!"
EOF

    chmod +x .git/hooks/pre-commit

    # Create commit-msg hook for conventional commits
    cat > .git/hooks/commit-msg << 'EOF'
#!/bin/bash

# Check commit message format
commit_msg=$(cat $1)

# Check if message matches conventional commit format
if ! echo "$commit_msg" | grep -qE "^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .{1,}"; then
    echo "Commit message does not follow conventional commit format:"
    echo "Expected: type(scope): description"
    echo "Examples:"
    echo "  feat: add new feature"
    echo "  fix: resolve bug"
    echo "  docs: update documentation"
    echo ""
    echo "Your message: $commit_msg"
    exit 1
fi
EOF

    chmod +x .git/hooks/commit-msg

    log_success "Git hooks setup complete"
}

# Setup IDE configuration
setup_ide_config() {
    log_info "Setting up IDE configuration..."

    # VS Code settings
    if [ -d ".vscode" ]; then
        log_info "VS Code settings already exist"
    else
        mkdir -p .vscode
        cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": true,
        "source.organizeImports": true
    },
    "rust-analyzer.server.path": "rust-analyzer",
    "rust-analyzer.cargo.target": null,
    "rust-analyzer.checkOnSave.allTargets": false,
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "rust-analyzer.cargo.extraArgs": ["--profile=dev"],
    "rust-analyzer.completion.postfix.enable": false,
    "rust-analyzer.diagnostics.disabled": [
        "inactive-code"
    ]
}
EOF

        cat > .vscode/extensions.json << 'EOF'
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "tamasfe.even-better-toml",
        "ms-vscode.vscode-json",
        "esbenp.prettier-vscode",
        "ms-vscode.vscode-typescript-next",
        "bradlc.vscode-tailwindcss",
        "ms-vscode-remote.remote-containers"
    ]
}
EOF
        log_success "VS Code configuration created"
    fi

    # IntelliJ IDEA settings (if applicable)
    if [ -d ".idea" ]; then
        log_info "IntelliJ settings already exist"
    else
        log_info "IntelliJ settings can be configured manually if needed"
    fi
}

# Setup development configuration
setup_dev_config() {
    log_info "Setting up development configuration..."

    # Create dev config if it doesn't exist
    if [ -f "config/dev.yaml" ]; then
        log_info "Development config already exists"
    else
        mkdir -p config
        cat > config/dev.yaml << 'EOF'
# RiceCoder Development Configuration
# This file contains development-specific settings

development:
  # Enable debug logging
  debug: true

  # Development database (SQLite)
  database:
    url: "sqlite:dev.db"

  # Development providers (use test keys)
  providers:
    openai:
      api_key: "sk-test-key-replace-with-real-key"
      model: "gpt-3.5-turbo"
    anthropic:
      api_key: "sk-ant-test-key-replace-with-real-key"
      model: "claude-3-haiku-20240307"

  # Development features
  features:
    enable_experimental: true
    enable_debug_tools: true
    enable_performance_monitoring: true

  # Testing configuration
  testing:
    enable_slow_tests: false
    enable_integration_tests: true
    test_timeout_seconds: 300

logging:
  level: "debug"
  format: "json"
  file: "logs/dev.log"
EOF
        log_success "Development config created at config/dev.yaml"
        log_warning "Remember to replace test API keys with real ones for development"
    fi
}

# Setup testing environment
setup_testing() {
    log_info "Setting up testing environment..."

    # Create test directories
    mkdir -p tests/fixtures
    mkdir -p tests/integration
    mkdir -p tests/property

    # Create basic test fixtures
    if [ ! -f "tests/fixtures/sample_project.rs" ]; then
        cat > tests/fixtures/sample_project.rs << 'EOF'
// Sample Rust project for testing
pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, World!");
    }
}
EOF
        log_success "Test fixtures created"
    fi

    # Run initial test to verify setup
    log_info "Running initial test suite..."
    if cargo test --lib --quiet; then
        log_success "Initial tests passed"
    else
        log_warning "Some tests failed - this is normal for initial setup"
    fi
}

# Setup documentation tools
setup_docs() {
    log_info "Setting up documentation tools..."

    # Install mdbook if not present
    if ! command -v mdbook &> /dev/null; then
        log_info "Installing mdbook..."
        cargo install mdbook
        cargo install mdbook-linkcheck
    fi

    # Create docs directory structure if needed
    mkdir -p docs/guides
    mkdir -p docs/api
    mkdir -p docs/examples

    log_success "Documentation tools setup complete"
}

# Main setup function
main() {
    log_info "🍚 RiceCoder Development Environment Setup"
    log_info "=========================================="

    OS=$(detect_os)
    log_info "Detected OS: $OS"

    # Check prerequisites
    log_info "Checking prerequisites..."

    if ! command -v git &> /dev/null; then
        log_error "Git is required but not installed. Please install Git first."
        exit 1
    fi

    if ! command -v curl &> /dev/null; then
        log_error "curl is required but not installed. Please install curl first."
        exit 1
    fi

    log_success "Prerequisites check passed"

    # Run setup steps
    setup_rust
    setup_git_hooks
    setup_ide_config
    setup_dev_config
    setup_testing
    setup_docs

    # Final verification
    log_info "Running final verification..."

    if cargo check --quiet; then
        log_success "✅ Project builds successfully"
    else
        log_error "❌ Project build failed"
        exit 1
    fi

    if cargo test --lib --quiet; then
        log_success "✅ Tests run successfully"
    else
        log_warning "⚠️  Some tests failed - check test output above"
    fi

    # Print success message
    log_success ""
    log_success "🎉 Development environment setup complete!"
    log_success ""
    log_info "Next steps:"
    echo "  1. Configure your API keys in config/dev.yaml"
    echo "  2. Run 'cargo build' to build the project"
    echo "  3. Run 'cargo test' to run the full test suite"
    echo "  4. Run 'rice --help' to see available commands"
    echo "  5. Check CONTRIBUTING.md for contribution guidelines"
    log_success ""
    log_info "Happy coding! 🚀"
}

# Run main function
main "$@"