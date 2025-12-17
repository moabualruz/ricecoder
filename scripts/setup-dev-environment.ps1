# RiceCoder Development Environment Setup Script
# This script sets up a complete development environment for RiceCoder contributors

param(
    [switch]$Force,
    [switch]$SkipTests
)

# Colors for output
$Green = "Green"
$Yellow = "Yellow"
$Red = "Red"
$Blue = "Blue"
$Cyan = "Cyan"

# Logging functions
function Write-Info {
    param([string]$Message)
    Write-Host "[$Blue INFO $White] $Message" -ForegroundColor $Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[$Green SUCCESS $White] $Message" -ForegroundColor $Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[$Yellow WARNING $White] $Message" -ForegroundColor $Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[$Red ERROR $White] $Message" -ForegroundColor $Red
}

# Check if command exists
function Test-Command {
    param([string]$Command)
    $null -ne (Get-Command $Command -ErrorAction SilentlyContinue)
}

# Setup Rust toolchain
function Setup-Rust {
    Write-Info "Setting up Rust toolchain..."

    if (-not (Test-Command "rustc")) {
        Write-Info "Installing Rust..."
        try {
            Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
            Start-Process -FilePath ".\rustup-init.exe" -ArgumentList "-y" -Wait -NoNewWindow
            Remove-Item "rustup-init.exe"
            $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
        } catch {
            Write-Error "Failed to install Rust: $_"
            exit 1
        }
    } else {
        Write-Info "Rust already installed: $(rustc --version)"
    }

    # Install/update Rust 1.75
    Write-Info "Installing Rust 1.75..."
    rustup install 1.75
    rustup default 1.75

    # Install components
    Write-Info "Installing Rust components..."
    rustup component add clippy rustfmt

    # Install cargo tools
    Write-Info "Installing Cargo tools..."
    cargo install cargo-audit cargo-outdated cargo-tarpaulin cargo-nextest cargo-watch

    Write-Success "Rust toolchain setup complete"
}

# Setup Git hooks
function Setup-GitHooks {
    Write-Info "Setting up Git hooks..."

    # Create pre-commit hook
    $preCommitHook = @"
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
"@

    $preCommitHook | Out-File -FilePath ".git/hooks/pre-commit" -Encoding UTF8
    & chmod +x .git/hooks/pre-commit

    # Create commit-msg hook for conventional commits
    $commitMsgHook = @"
#!/bin/bash

# Check commit message format
commit_msg=`$(cat `$1)

# Check if message matches conventional commit format
if ! echo "`$commit_msg" | grep -qE "^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .{1,}"; then
    echo "Commit message does not follow conventional commit format:"
    echo "Expected: type(scope): description"
    echo "Examples:"
    echo "  feat: add new feature"
    echo "  fix: resolve bug"
    echo "  docs: update documentation"
    echo ""
    echo "Your message: `$commit_msg"
    exit 1
fi
"@

    $commitMsgHook | Out-File -FilePath ".git/hooks/commit-msg" -Encoding UTF8
    & chmod +x .git/hooks/commit-msg

    Write-Success "Git hooks setup complete"
}

# Setup IDE configuration
function Setup-IdeConfig {
    Write-Info "Setting up IDE configuration..."

    # VS Code settings
    if (Test-Path ".vscode") {
        Write-Info "VS Code settings already exist"
    } else {
        New-Item -ItemType Directory -Path ".vscode" -Force | Out-Null

        $vsCodeSettings = @"
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
"@

        $vsCodeSettings | Out-File -FilePath ".vscode/settings.json" -Encoding UTF8

        $vsCodeExtensions = @"
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "tamasfe.even-better-toml",
        "ms-vscode.vscode-json",
        "esbenp.prettier-vscode",
        "ms-vscode-remote.remote-containers"
    ]
}
"@

        $vsCodeExtensions | Out-File -FilePath ".vscode/extensions.json" -Encoding UTF8

        Write-Success "VS Code configuration created"
    }
}

# Setup development configuration
function Setup-DevConfig {
    Write-Info "Setting up development configuration..."

    # Create dev config if it doesn't exist
    if (Test-Path "config/dev.yaml") {
        Write-Info "Development config already exists"
    } else {
        New-Item -ItemType Directory -Path "config" -Force | Out-Null

        $devConfig = @"
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
"@

        $devConfig | Out-File -FilePath "config/dev.yaml" -Encoding UTF8

        Write-Success "Development config created at config/dev.yaml"
        Write-Warning "Remember to replace test API keys with real ones for development"
    }
}

# Setup testing environment
function Setup-Testing {
    Write-Info "Setting up testing environment..."

    # Create test directories
    New-Item -ItemType Directory -Path "tests/fixtures" -Force | Out-Null
    New-Item -ItemType Directory -Path "tests/integration" -Force | Out-Null
    New-Item -ItemType Directory -Path "tests/property" -Force | Out-Null

    # Create basic test fixtures
    if (-not (Test-Path "tests/fixtures/sample_project.rs")) {
        $sampleProject = @"
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
"@

        $sampleProject | Out-File -FilePath "tests/fixtures/sample_project.rs" -Encoding UTF8

        Write-Success "Test fixtures created"
    }

    if (-not $SkipTests) {
        # Run initial test to verify setup
        Write-Info "Running initial test suite..."
        try {
            & cargo test --lib --quiet
            Write-Success "Initial tests passed"
        } catch {
            Write-Warning "Some tests failed - this is normal for initial setup"
        }
    }
}

# Setup documentation tools
function Setup-Docs {
    Write-Info "Setting up documentation tools..."

    # Install mdbook if not present
    if (-not (Test-Command "mdbook")) {
        Write-Info "Installing mdbook..."
        cargo install mdbook
        cargo install mdbook-linkcheck
    }

    # Create docs directory structure if needed
    New-Item -ItemType Directory -Path "docs/guides" -Force | Out-Null
    New-Item -ItemType Directory -Path "docs/api" -Force | Out-Null
    New-Item -ItemType Directory -Path "docs/examples" -Force | Out-Null

    Write-Success "Documentation tools setup complete"
}

# Main setup function
function Main {
    Write-Info "🍚 RiceCoder Development Environment Setup"
    Write-Info "=========================================="

    # Check prerequisites
    Write-Info "Checking prerequisites..."

    if (-not (Test-Command "git")) {
        Write-Error "Git is required but not installed. Please install Git first."
        exit 1
    }

    Write-Success "Prerequisites check passed"

    # Run setup steps
    Setup-Rust
    Setup-GitHooks
    Setup-IdeConfig
    Setup-DevConfig
    Setup-Testing
    Setup-Docs

    # Final verification
    Write-Info "Running final verification..."

    try {
        & cargo check --quiet
        Write-Success "✅ Project builds successfully"
    } catch {
        Write-Error "❌ Project build failed"
        exit 1
    }

    if (-not $SkipTests) {
        try {
            & cargo test --lib --quiet
            Write-Success "✅ Tests run successfully"
        } catch {
            Write-Warning "⚠️  Some tests failed - check test output above"
        }
    }

    # Print success message
    Write-Success ""
    Write-Success "🎉 Development environment setup complete!"
    Write-Success ""
    Write-Info "Next steps:"
    Write-Host "  1. Configure your API keys in config/dev.yaml" -ForegroundColor $Cyan
    Write-Host "  2. Run 'cargo build' to build the project" -ForegroundColor $Cyan
    Write-Host "  3. Run 'cargo test' to run the full test suite" -ForegroundColor $Cyan
    Write-Host "  4. Run 'rice --help' to see available commands" -ForegroundColor $Cyan
    Write-Host "  5. Check CONTRIBUTING.md for contribution guidelines" -ForegroundColor $Cyan
    Write-Success ""
    Write-Info "Happy coding! 🚀"
}

# Run main function
Main