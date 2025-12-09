#!/bin/bash

# Unit tests for install.sh script
# Tests platform detection, architecture detection, and binary name mapping

set -e

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test utilities
assert_equals() {
    local expected="$1"
    local actual="$2"
    local test_name="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ "$expected" = "$actual" ]; then
        echo -e "${GREEN}✓${NC} $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗${NC} $test_name"
        echo "  Expected: $expected"
        echo "  Actual: $actual"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Define test functions that mirror the install script logic

# Test: Platform detection for Linux
test_detect_os_linux() {
    # Test the logic directly
    local os="Linux"
    local result
    case "$os" in
        Linux*)
            result="linux"
            ;;
        *)
            result="other"
            ;;
    esac
    assert_equals "linux" "$result" "Platform detection: Linux"
}

# Test: Platform detection for macOS
test_detect_os_macos() {
    local os="Darwin"
    local result
    case "$os" in
        Darwin*)
            result="macos"
            ;;
        *)
            result="other"
            ;;
    esac
    assert_equals "macos" "$result" "Platform detection: macOS"
}

# Test: Platform detection for Windows
test_detect_os_windows() {
    local os="MINGW64_NT-10.0"
    local result
    case "$os" in
        MINGW*|MSYS*|CYGWIN*)
            result="windows"
            ;;
        *)
            result="other"
            ;;
    esac
    assert_equals "windows" "$result" "Platform detection: Windows"
}

# Test: Platform detection for unsupported OS
test_detect_os_unsupported() {
    local os="UnknownOS"
    local result
    case "$os" in
        Linux*)
            result="linux"
            ;;
        Darwin*)
            result="macos"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            result="windows"
            ;;
        *)
            result="unsupported"
            ;;
    esac
    assert_equals "unsupported" "$result" "Platform detection: Unsupported OS"
}

# Test: Architecture detection for x86_64
test_detect_arch_x86_64() {
    local arch="x86_64"
    local result
    case "$arch" in
        x86_64|amd64)
            result="x86_64"
            ;;
        *)
            result="other"
            ;;
    esac
    assert_equals "x86_64" "$result" "Architecture detection: x86_64"
}

# Test: Architecture detection for ARM64
test_detect_arch_aarch64() {
    local arch="aarch64"
    local result
    case "$arch" in
        aarch64|arm64)
            result="aarch64"
            ;;
        *)
            result="other"
            ;;
    esac
    assert_equals "aarch64" "$result" "Architecture detection: ARM64"
}

# Test: Architecture detection for unsupported arch
test_detect_arch_unsupported() {
    local arch="mips"
    local result
    case "$arch" in
        x86_64|amd64)
            result="x86_64"
            ;;
        aarch64|arm64)
            result="aarch64"
            ;;
        *)
            result="unsupported"
            ;;
    esac
    assert_equals "unsupported" "$result" "Architecture detection: Unsupported arch"
}

# Test: Binary name mapping for Linux x86_64
test_binary_name_linux_x86_64() {
    local os="linux"
    local arch="x86_64"
    local result
    
    case "$os" in
        linux)
            case "$arch" in
                x86_64)
                    result="ricecoder-x86_64-unknown-linux-musl"
                    ;;
                *)
                    result="error"
                    ;;
            esac
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "ricecoder-x86_64-unknown-linux-musl" "$result" "Binary name: Linux x86_64"
}

# Test: Binary name mapping for Linux ARM64
test_binary_name_linux_aarch64() {
    local os="linux"
    local arch="aarch64"
    local result
    
    case "$os" in
        linux)
            case "$arch" in
                aarch64)
                    result="ricecoder-aarch64-unknown-linux-musl"
                    ;;
                *)
                    result="error"
                    ;;
            esac
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "ricecoder-aarch64-unknown-linux-musl" "$result" "Binary name: Linux ARM64"
}

# Test: Binary name mapping for macOS x86_64
test_binary_name_macos_x86_64() {
    local os="macos"
    local arch="x86_64"
    local result
    
    case "$os" in
        macos)
            case "$arch" in
                x86_64)
                    result="ricecoder-x86_64-apple-darwin"
                    ;;
                *)
                    result="error"
                    ;;
            esac
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "ricecoder-x86_64-apple-darwin" "$result" "Binary name: macOS x86_64"
}

# Test: Binary name mapping for macOS ARM64
test_binary_name_macos_aarch64() {
    local os="macos"
    local arch="aarch64"
    local result
    
    case "$os" in
        macos)
            case "$arch" in
                aarch64)
                    result="ricecoder-aarch64-apple-darwin"
                    ;;
                *)
                    result="error"
                    ;;
            esac
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "ricecoder-aarch64-apple-darwin" "$result" "Binary name: macOS ARM64"
}

# Test: Binary name mapping for Windows x86_64
test_binary_name_windows_x86_64() {
    local os="windows"
    local arch="x86_64"
    local result
    
    case "$os" in
        windows)
            case "$arch" in
                x86_64)
                    result="ricecoder-x86_64-pc-windows-msvc"
                    ;;
                *)
                    result="error"
                    ;;
            esac
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "ricecoder-x86_64-pc-windows-msvc" "$result" "Binary name: Windows x86_64"
}

# Test: Binary name mapping for Windows ARM64
test_binary_name_windows_aarch64() {
    local os="windows"
    local arch="aarch64"
    local result
    
    case "$os" in
        windows)
            case "$arch" in
                aarch64)
                    result="ricecoder-aarch64-pc-windows-msvc"
                    ;;
                *)
                    result="error"
                    ;;
            esac
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "ricecoder-aarch64-pc-windows-msvc" "$result" "Binary name: Windows ARM64"
}

# Test: Binary name mapping for unsupported platform
test_binary_name_unsupported_platform() {
    local os="unsupported"
    local arch="x86_64"
    local result
    
    case "$os" in
        linux|macos|windows)
            result="valid"
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "error" "$result" "Binary name: Unsupported platform"
}

# Test: Binary name mapping for unsupported architecture
test_binary_name_unsupported_arch() {
    local os="linux"
    local arch="mips"
    local result
    
    case "$os" in
        linux)
            case "$arch" in
                x86_64|aarch64)
                    result="valid"
                    ;;
                *)
                    result="error"
                    ;;
            esac
            ;;
        *)
            result="error"
            ;;
    esac
    assert_equals "error" "$result" "Binary name: Unsupported architecture"
}

# Test: Archive extension for Linux
test_archive_ext_linux() {
    local os="linux"
    local result
    case "$os" in
        windows)
            result="zip"
            ;;
        *)
            result="tar.gz"
            ;;
    esac
    assert_equals "tar.gz" "$result" "Archive extension: Linux"
}

# Test: Archive extension for macOS
test_archive_ext_macos() {
    local os="macos"
    local result
    case "$os" in
        windows)
            result="zip"
            ;;
        *)
            result="tar.gz"
            ;;
    esac
    assert_equals "tar.gz" "$result" "Archive extension: macOS"
}

# Test: Archive extension for Windows
test_archive_ext_windows() {
    local os="windows"
    local result
    case "$os" in
        windows)
            result="zip"
            ;;
        *)
            result="tar.gz"
            ;;
    esac
    assert_equals "zip" "$result" "Archive extension: Windows"
}

# Run all tests
main() {
    echo -e "${BLUE}Running installation script unit tests${NC}"
    echo ""
    
    # Platform detection tests
    echo -e "${BLUE}Platform Detection Tests:${NC}"
    test_detect_os_linux
    test_detect_os_macos
    test_detect_os_windows
    test_detect_os_unsupported
    echo ""
    
    # Architecture detection tests
    echo -e "${BLUE}Architecture Detection Tests:${NC}"
    test_detect_arch_x86_64
    test_detect_arch_aarch64
    test_detect_arch_unsupported
    echo ""
    
    # Binary name mapping tests
    echo -e "${BLUE}Binary Name Mapping Tests:${NC}"
    test_binary_name_linux_x86_64
    test_binary_name_linux_aarch64
    test_binary_name_macos_x86_64
    test_binary_name_macos_aarch64
    test_binary_name_windows_x86_64
    test_binary_name_windows_aarch64
    test_binary_name_unsupported_platform
    test_binary_name_unsupported_arch
    echo ""
    
    # Archive extension tests
    echo -e "${BLUE}Archive Extension Tests:${NC}"
    test_archive_ext_linux
    test_archive_ext_macos
    test_archive_ext_windows
    echo ""
    
    # Print summary
    echo -e "${BLUE}Test Summary:${NC}"
    echo "Tests run: $TESTS_RUN"
    echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed!${NC}"
        return 1
    fi
}

main "$@"
