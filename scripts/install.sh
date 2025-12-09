#!/bin/bash

# RiceCoder Installation Script
# Installs ricecoder binary from GitHub Releases
# Supports: Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows (via Git Bash/WSL)

set -e

# Configuration
GITHUB_REPO="moabualruz/ricecoder"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TEMP_DIR=$(mktemp -d)
SCRIPT_VERSION="1.0.0"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Cleanup on exit
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Detect operating system
detect_os() {
    local os
    os=$(uname -s)
    
    case "$os" in
        Linux*)
            echo "linux"
            ;;
        Darwin*)
            echo "macos"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "windows"
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

# Detect architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

# Map OS and architecture to release binary name
get_binary_name() {
    local os="$1"
    local arch="$2"
    
    case "$os" in
        linux)
            case "$arch" in
                x86_64)
                    echo "ricecoder-x86_64-unknown-linux-musl"
                    ;;
                aarch64)
                    echo "ricecoder-aarch64-unknown-linux-musl"
                    ;;
                *)
                    return 1
                    ;;
            esac
            ;;
        macos)
            case "$arch" in
                x86_64)
                    echo "ricecoder-x86_64-apple-darwin"
                    ;;
                aarch64)
                    echo "ricecoder-aarch64-apple-darwin"
                    ;;
                *)
                    return 1
                    ;;
            esac
            ;;
        windows)
            case "$arch" in
                x86_64)
                    echo "ricecoder-x86_64-pc-windows-msvc"
                    ;;
                aarch64)
                    echo "ricecoder-aarch64-pc-windows-msvc"
                    ;;
                *)
                    return 1
                    ;;
            esac
            ;;
        *)
            return 1
            ;;
    esac
}

# Get archive extension based on OS
get_archive_ext() {
    local os="$1"
    
    case "$os" in
        windows)
            echo "zip"
            ;;
        *)
            echo "tar.gz"
            ;;
    esac
}

# Get latest release version from GitHub API
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name"' | head -1 | cut -d'"' -f4)
    
    if [ -z "$version" ]; then
        return 1
    fi
    
    echo "$version"
}

# Download file with retry logic
download_with_retry() {
    local url="$1"
    local output="$2"
    local max_attempts=3
    local attempt=1
    local backoff=1
    
    while [ $attempt -le $max_attempts ]; do
        print_info "Downloading (attempt $attempt/$max_attempts)..."
        
        if curl -fsSL -o "$output" "$url"; then
            print_success "Download successful"
            return 0
        fi
        
        if [ $attempt -lt $max_attempts ]; then
            print_warning "Download failed, retrying in ${backoff}s..."
            sleep "$backoff"
            backoff=$((backoff * 2))
        fi
        
        attempt=$((attempt + 1))
    done
    
    print_error "Download failed after $max_attempts attempts"
    return 1
}

# Verify SHA256 checksum
verify_checksum() {
    local file="$1"
    local checksum_file="$2"
    
    print_info "Verifying checksum..."
    
    if ! command -v sha256sum &> /dev/null; then
        if ! command -v shasum &> /dev/null; then
            print_warning "sha256sum/shasum not found, skipping verification"
            return 0
        fi
        # macOS uses shasum
        if shasum -a 256 -c "$checksum_file" > /dev/null 2>&1; then
            print_success "Checksum verified"
            return 0
        fi
    else
        if sha256sum -c "$checksum_file" > /dev/null 2>&1; then
            print_success "Checksum verified"
            return 0
        fi
    fi
    
    print_error "Checksum verification failed"
    return 1
}

# Extract archive
extract_archive() {
    local archive="$1"
    local extract_dir="$2"
    local ext="$3"
    
    print_info "Extracting archive..."
    
    case "$ext" in
        tar.gz)
            if ! tar -xzf "$archive" -C "$extract_dir"; then
                print_error "Failed to extract tar.gz"
                return 1
            fi
            ;;
        zip)
            if ! unzip -q "$archive" -d "$extract_dir"; then
                print_error "Failed to extract zip"
                return 1
            fi
            ;;
        *)
            print_error "Unknown archive format: $ext"
            return 1
            ;;
    esac
    
    print_success "Archive extracted"
    return 0
}

# Install binary to PATH
install_binary() {
    local binary_path="$1"
    local install_dir="$2"
    
    print_info "Installing binary to $install_dir..."
    
    # Check if install directory exists and is writable
    if [ ! -d "$install_dir" ]; then
        print_warning "Install directory does not exist: $install_dir"
        print_info "Creating directory..."
        if ! mkdir -p "$install_dir"; then
            print_error "Failed to create directory (may need sudo)"
            return 1
        fi
    fi
    
    if [ ! -w "$install_dir" ]; then
        print_warning "Install directory is not writable: $install_dir"
        print_info "Attempting with sudo..."
        if ! sudo cp "$binary_path" "$install_dir/ricecoder"; then
            print_error "Failed to install binary (permission denied)"
            return 1
        fi
        if ! sudo chmod +x "$install_dir/ricecoder"; then
            print_error "Failed to set executable permissions"
            return 1
        fi
    else
        if ! cp "$binary_path" "$install_dir/ricecoder"; then
            print_error "Failed to copy binary"
            return 1
        fi
        if ! chmod +x "$install_dir/ricecoder"; then
            print_error "Failed to set executable permissions"
            return 1
        fi
    fi
    
    print_success "Binary installed to $install_dir/ricecoder"
    return 0
}

# Verify installation
verify_installation() {
    print_info "Verifying installation..."
    
    if ! command -v ricecoder &> /dev/null; then
        print_warning "ricecoder not found in PATH"
        print_info "Checking if binary exists at $INSTALL_DIR/ricecoder..."
        
        if [ ! -f "$INSTALL_DIR/ricecoder" ]; then
            print_error "Binary not found at $INSTALL_DIR/ricecoder"
            return 1
        fi
        
        print_warning "Binary exists but not in PATH"
        print_info "You may need to add $INSTALL_DIR to your PATH"
        print_info "Add this line to your shell profile (.bashrc, .zshrc, etc.):"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        return 0
    fi
    
    local version
    version=$(ricecoder --version 2>/dev/null || echo "unknown")
    print_success "Installation verified: $version"
    return 0
}

# Main installation flow
main() {
    print_info "RiceCoder Installation Script v$SCRIPT_VERSION"
    echo ""
    
    # Detect OS and architecture
    print_info "Detecting system..."
    local os
    local arch
    os=$(detect_os)
    arch=$(detect_arch)
    
    if [ "$os" = "unsupported" ]; then
        print_error "Unsupported operating system: $(uname -s)"
        echo "Supported: Linux, macOS, Windows (via Git Bash/WSL)"
        return 1
    fi
    
    if [ "$arch" = "unsupported" ]; then
        print_error "Unsupported architecture: $(uname -m)"
        echo "Supported: x86_64, aarch64 (ARM64)"
        return 1
    fi
    
    print_success "Detected: $os ($arch)"
    echo ""
    
    # Get binary name
    local binary_name
    if ! binary_name=$(get_binary_name "$os" "$arch"); then
        print_error "Unsupported platform/architecture combination: $os/$arch"
        return 1
    fi
    
    # Get archive extension
    local ext
    ext=$(get_archive_ext "$os")
    
    # Get latest version
    print_info "Fetching latest release..."
    local version
    if ! version=$(get_latest_version); then
        print_error "Failed to fetch latest release from GitHub"
        echo "Please check your internet connection and try again"
        return 1
    fi
    
    print_success "Latest version: $version"
    echo ""
    
    # Download binary
    local archive_name="${binary_name}.${ext}"
    local download_url="https://github.com/$GITHUB_REPO/releases/download/$version/$archive_name"
    local checksum_url="https://github.com/$GITHUB_REPO/releases/download/$version/${binary_name}.sha256"
    
    print_info "Downloading ricecoder $version..."
    if ! download_with_retry "$download_url" "$TEMP_DIR/$archive_name"; then
        print_error "Failed to download binary"
        echo "URL: $download_url"
        return 1
    fi
    echo ""
    
    # Download and verify checksum
    print_info "Downloading checksum..."
    if ! download_with_retry "$checksum_url" "$TEMP_DIR/${binary_name}.sha256"; then
        print_warning "Failed to download checksum, skipping verification"
    else
        # Prepare checksum file for verification
        cd "$TEMP_DIR"
        if ! verify_checksum "$archive_name" "${binary_name}.sha256"; then
            print_error "Checksum verification failed"
            echo "The downloaded file may be corrupted"
            return 1
        fi
        cd - > /dev/null
    fi
    echo ""
    
    # Extract archive
    if ! extract_archive "$TEMP_DIR/$archive_name" "$TEMP_DIR" "$ext"; then
        return 1
    fi
    echo ""
    
    # Find extracted binary
    local extracted_binary
    extracted_binary=$(find "$TEMP_DIR" -name "ricecoder" -type f 2>/dev/null | head -1)
    
    if [ -z "$extracted_binary" ]; then
        print_error "Binary not found in extracted archive"
        return 1
    fi
    
    # Install binary
    if ! install_binary "$extracted_binary" "$INSTALL_DIR"; then
        return 1
    fi
    echo ""
    
    # Verify installation
    if ! verify_installation; then
        print_warning "Installation completed but verification failed"
        return 1
    fi
    echo ""
    
    print_success "Installation complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Run 'ricecoder --help' to see available commands"
    echo "  2. Run 'ricecoder init' to initialize configuration"
    echo "  3. Run 'ricecoder chat' to start chatting"
    echo ""
    
    return 0
}

# Run main function
main "$@"
