#!/bin/bash

################################################################################
# RiceCoder Installation Script for Linux/macOS
#
# This script compiles and installs ricecoder from source.
# Supports: Linux (x86_64, aarch64), macOS (Intel, Apple Silicon)
#
# Usage:
#   ./scripts/install.sh [OPTIONS]
#
# Options:
#   --prefix PATH       Installation prefix (default: ~/.local)
#   --release           Build in release mode (default)
#   --debug             Build in debug mode
#   --no-strip          Don't strip binaries
#   --help              Show this help message
#
# Examples:
#   ./scripts/install.sh                          # Install to ~/.local
#   ./scripts/install.sh --prefix /usr/local      # Install to /usr/local
#   ./scripts/install.sh --debug                  # Debug build
#
################################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PREFIX="${HOME}/.local"
BUILD_MODE="release"
STRIP_BINARIES=true
VERBOSE=false

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        OS_NAME="linux"
        ;;
    Darwin)
        OS_NAME="macos"
        ;;
    *)
        echo -e "${RED}Error: Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH_NAME="x86_64"
        ;;
    aarch64|arm64)
        ARCH_NAME="aarch64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --prefix)
                PREFIX="$2"
                shift 2
                ;;
            --release)
                BUILD_MODE="release"
                shift
                ;;
            --debug)
                BUILD_MODE="debug"
                shift
                ;;
            --no-strip)
                STRIP_BINARIES=false
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                echo -e "${RED}Error: Unknown option: $1${NC}"
                show_help
                exit 1
                ;;
        esac
    done
}

show_help() {
    head -n 30 "$0" | tail -n +2 | sed 's/^# //'
}

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $*"
}

log_error() {
    echo -e "${RED}[✗]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $*"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed"
        echo "Install Rust from: https://rustup.rs/"
        exit 1
    fi
    
    local rust_version
    rust_version=$(rustc --version)
    log_success "Found: $rust_version"
    
    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed"
        exit 1
    fi
    
    # Check Git
    if ! command -v git &> /dev/null; then
        log_error "Git is not installed"
        exit 1
    fi
    
    log_success "All prerequisites met"
}

# Update Rust toolchain
update_rust() {
    log_info "Updating Rust toolchain..."
    rustup update
    log_success "Rust toolchain updated"
}

# Build ricecoder
build_ricecoder() {
    log_info "Building ricecoder (${BUILD_MODE} mode)..."
    
    cd "$PROJECT_ROOT"
    
    local cargo_args=()
    
    if [[ "$BUILD_MODE" == "release" ]]; then
        cargo_args+=(--release)
    fi
    
    if [[ "$VERBOSE" == true ]]; then
        cargo_args+=(--verbose)
    fi
    
    # Build all binaries
    cargo build "${cargo_args[@]}"
    
    log_success "Build completed successfully"
}

# Install binaries
install_binaries() {
    log_info "Installing binaries to $PREFIX/bin..."
    
    # Create bin directory
    mkdir -p "$PREFIX/bin"
    
    local build_dir
    if [[ "$BUILD_MODE" == "release" ]]; then
        build_dir="$PROJECT_ROOT/target/release"
    else
        build_dir="$PROJECT_ROOT/target/debug"
    fi
    
    # Find and install all binaries
    local binaries=()
    while IFS= read -r -d '' binary; do
        binaries+=("$binary")
    done < <(find "$build_dir" -maxdepth 1 -type f -executable -print0 2>/dev/null || true)
    
    if [[ ${#binaries[@]} -eq 0 ]]; then
        log_warning "No binaries found in $build_dir"
        return
    fi
    
    for binary in "${binaries[@]}"; do
        local binary_name
        binary_name=$(basename "$binary")
        
        # Skip library files
        if [[ "$binary_name" == lib* ]]; then
            continue
        fi
        
        log_info "Installing $binary_name..."
        
        # Copy binary
        cp "$binary" "$PREFIX/bin/$binary_name"
        chmod +x "$PREFIX/bin/$binary_name"
        
        # Strip if requested
        if [[ "$STRIP_BINARIES" == true ]] && command -v strip &> /dev/null; then
            strip "$PREFIX/bin/$binary_name" 2>/dev/null || true
        fi
        
        log_success "Installed $binary_name"
    done
}

# Install configuration files
install_config() {
    log_info "Installing configuration files..."
    
    local config_dir="$PREFIX/etc/ricecoder"
    mkdir -p "$config_dir"
    
    if [[ -d "$PROJECT_ROOT/config" ]]; then
        cp -r "$PROJECT_ROOT/config"/* "$config_dir/" 2>/dev/null || true
        log_success "Configuration files installed to $config_dir"
    fi
}

# Install documentation
install_docs() {
    log_info "Installing documentation..."
    
    local doc_dir="$PREFIX/share/doc/ricecoder"
    mkdir -p "$doc_dir"
    
    if [[ -f "$PROJECT_ROOT/README.md" ]]; then
        cp "$PROJECT_ROOT/README.md" "$doc_dir/"
    fi
    
    if [[ -f "$PROJECT_ROOT/LICENSE.md" ]]; then
        cp "$PROJECT_ROOT/LICENSE.md" "$doc_dir/"
    fi
    
    log_success "Documentation installed to $doc_dir"
}

# Update PATH
update_path() {
    local shell_rc=""
    
    if [[ "$OS_NAME" == "macos" ]]; then
        shell_rc="$HOME/.zprofile"
    else
        shell_rc="$HOME/.bashrc"
    fi
    
    if [[ ! "$PATH" == *"$PREFIX/bin"* ]]; then
        log_warning "Add $PREFIX/bin to your PATH"
        echo ""
        echo "Add this line to $shell_rc:"
        echo "  export PATH=\"$PREFIX/bin:\$PATH\""
        echo ""
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."
    
    if [[ ! -d "$PREFIX/bin" ]]; then
        log_error "Installation directory not found: $PREFIX/bin"
        return 1
    fi
    
    local installed_count=0
    for binary in "$PREFIX/bin"/ricecoder*; do
        if [[ -x "$binary" ]]; then
            local binary_name
            binary_name=$(basename "$binary")
            log_success "Found: $binary_name"
            ((installed_count++))
        fi
    done
    
    if [[ $installed_count -eq 0 ]]; then
        log_warning "No ricecoder binaries found in $PREFIX/bin"
        return 1
    fi
    
    log_success "Installation verified ($installed_count binaries)"
}

# Print summary
print_summary() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║${NC}          RiceCoder Installation Complete!                ${GREEN}║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Installation Details:"
    echo "  OS:              $OS_NAME ($ARCH_NAME)"
    echo "  Build Mode:      $BUILD_MODE"
    echo "  Install Prefix:  $PREFIX"
    echo "  Binaries:        $PREFIX/bin"
    echo "  Config:          $PREFIX/etc/ricecoder"
    echo "  Documentation:   $PREFIX/share/doc/ricecoder"
    echo ""
    echo "Next Steps:"
    echo "  1. Add to PATH: export PATH=\"$PREFIX/bin:\$PATH\""
    echo "  2. Verify:      ricecoder --version"
    echo "  3. Get help:    ricecoder --help"
    echo ""
}

# Main installation flow
main() {
    parse_args "$@"
    
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}        RiceCoder Installation Script                     ${BLUE}║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    log_info "System Information:"
    log_info "  OS:           $OS_NAME"
    log_info "  Architecture: $ARCH_NAME"
    log_info "  Project Root: $PROJECT_ROOT"
    echo ""
    
    check_prerequisites
    update_rust
    build_ricecoder
    install_binaries
    install_config
    install_docs
    verify_installation
    update_path
    print_summary
}

# Run main function
main "$@"
