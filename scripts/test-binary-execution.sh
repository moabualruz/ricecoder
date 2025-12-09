#!/bin/bash

# Test binary execution on all platforms
# This script verifies that binaries execute correctly on their respective platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results
PASSED=0
FAILED=0
SKIPPED=0

# Function to print test result
print_result() {
    local test_name=$1
    local result=$2
    local message=$3
    
    if [ "$result" = "PASS" ]; then
        echo -e "${GREEN}✅ PASS${NC}: $test_name"
        ((PASSED++))
    elif [ "$result" = "FAIL" ]; then
        echo -e "${RED}❌ FAIL${NC}: $test_name - $message"
        ((FAILED++))
    elif [ "$result" = "SKIP" ]; then
        echo -e "${YELLOW}⏭️  SKIP${NC}: $test_name - $message"
        ((SKIPPED++))
    fi
}

# Function to test binary
test_binary() {
    local binary_path=$1
    local platform=$2
    local arch=$3
    
    echo ""
    echo "Testing $platform $arch binary..."
    echo "=================================="
    
    # Check if binary exists
    if [ ! -f "$binary_path" ]; then
        print_result "Binary exists" "SKIP" "Binary not found at $binary_path"
        return
    fi
    
    # Check if binary is executable
    if [ ! -x "$binary_path" ]; then
        print_result "Binary is executable" "FAIL" "Binary is not executable"
        return
    fi
    print_result "Binary is executable" "PASS"
    
    # Test --version command
    if output=$("$binary_path" --version 2>&1); then
        if echo "$output" | grep -q "ricecoder"; then
            print_result "--version command" "PASS"
        else
            print_result "--version command" "FAIL" "Output doesn't contain 'ricecoder': $output"
        fi
    else
        print_result "--version command" "FAIL" "Command failed with exit code $?"
    fi
    
    # Test --help command
    if output=$("$binary_path" --help 2>&1); then
        if echo "$output" | grep -q -E "(Usage|Commands|Options)"; then
            print_result "--help command" "PASS"
        else
            print_result "--help command" "FAIL" "Output doesn't contain expected help text"
        fi
    else
        print_result "--help command" "FAIL" "Command failed with exit code $?"
    fi
    
    # Test binary file properties
    if file "$binary_path" > /dev/null 2>&1; then
        file_info=$(file "$binary_path")
        print_result "Binary file properties" "PASS" "File type: $file_info"
    else
        print_result "Binary file properties" "FAIL" "Could not determine file type"
    fi
    
    # Test binary size
    binary_size=$(du -h "$binary_path" | cut -f1)
    print_result "Binary size" "PASS" "Size: $binary_size"
}

# Main test execution
echo "RiceCoder Binary Execution Tests"
echo "=================================="
echo "Platform: $(uname -s)"
echo "Architecture: $(uname -m)"
echo ""

# Detect current platform and architecture
PLATFORM=$(uname -s)
ARCH=$(uname -m)

case "$PLATFORM" in
    Linux)
        case "$ARCH" in
            x86_64)
                echo "Detected: Linux x86_64"
                BINARY_PATH="target/x86_64-unknown-linux-musl/release/ricecoder"
                test_binary "$BINARY_PATH" "Linux" "x86_64"
                ;;
            aarch64)
                echo "Detected: Linux ARM64"
                BINARY_PATH="target/aarch64-unknown-linux-musl/release/ricecoder"
                test_binary "$BINARY_PATH" "Linux" "ARM64"
                ;;
            *)
                print_result "Platform detection" "SKIP" "Unsupported architecture: $ARCH"
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                echo "Detected: macOS x86_64"
                BINARY_PATH="target/x86_64-apple-darwin/release/ricecoder"
                test_binary "$BINARY_PATH" "macOS" "x86_64"
                ;;
            arm64)
                echo "Detected: macOS ARM64 (Apple Silicon)"
                BINARY_PATH="target/aarch64-apple-darwin/release/ricecoder"
                test_binary "$BINARY_PATH" "macOS" "ARM64"
                ;;
            *)
                print_result "Platform detection" "SKIP" "Unsupported architecture: $ARCH"
                ;;
        esac
        ;;
    MINGW64_NT*|MSYS_NT*|CYGWIN_NT*)
        case "$ARCH" in
            x86_64)
                echo "Detected: Windows x86_64"
                BINARY_PATH="target/x86_64-pc-windows-msvc/release/ricecoder.exe"
                test_binary "$BINARY_PATH" "Windows" "x86_64"
                ;;
            *)
                print_result "Platform detection" "SKIP" "Unsupported architecture: $ARCH"
                ;;
        esac
        ;;
    *)
        print_result "Platform detection" "SKIP" "Unsupported platform: $PLATFORM"
        ;;
esac

# Print summary
echo ""
echo "=================================="
echo "Test Summary"
echo "=================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Skipped: $SKIPPED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed!${NC}"
    exit 1
fi
