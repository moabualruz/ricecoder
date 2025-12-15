#!/bin/bash
# Performance Validation Script
# Runs comprehensive performance validation for RiceCoder

set -e

echo "🚀 Starting RiceCoder Performance Validation"

# Build the ricecoder binary
echo "📦 Building ricecoder binary..."
cargo build --release --bin ricecoder

# Build the performance validation tool
echo "📦 Building performance validation tool..."
cargo build --release --bin ricecoder-performance

BINARY_PATH="./target/release/ricecoder"
PERF_TOOL="./target/release/ricecoder-performance"
BASELINE_FILE="performance-baselines.json"

echo "✅ Binaries built successfully"

# Run performance validation
echo "🔍 Running performance validation..."
$PERF_TOOL validate --binary $BINARY_PATH --baseline $BASELINE_FILE

echo "🎯 Performance validation completed!"