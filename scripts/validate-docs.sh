#!/bin/bash

# RiceCoder Documentation Validation Script
# Runs all documentation validation checks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🍜 RiceCoder Documentation Validation"
echo "====================================="
echo

cd "$PROJECT_ROOT"

# Check if Node.js is available
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is required for documentation validation"
    exit 1
fi

# Check if Rust toolchain is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is required for documentation validation"
    exit 1
fi

echo "📋 Running documentation link validation..."
if node scripts/validate-documentation-links.js; then
    echo "✅ Link validation passed"
else
    echo "❌ Link validation failed"
    exit 1
fi

echo
echo "📚 Running documentation completeness check..."
if node scripts/check-documentation-completeness.js; then
    echo "✅ Completeness check passed"
else
    echo "❌ Completeness check failed"
    exit 1
fi

echo
echo "🧪 Testing documentation examples..."
if cargo test --doc --all-features; then
    echo "✅ Documentation examples compile successfully"
else
    echo "❌ Documentation examples failed to compile"
    exit 1
fi

echo
echo "🎉 All documentation validation checks passed!"
echo
echo "Reports generated:"
echo "- .kiro/docs-validation-report.md"
echo "- .kiro/docs-validation-report.json"
echo "- .kiro/docs-completeness-report.md"
echo "- .kiro/docs-completeness-report.json"