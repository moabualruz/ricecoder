#!/bin/bash
# Update Performance Baselines Script
# Updates performance baselines with current measurements

set -e

echo "📊 Updating RiceCoder Performance Baselines"

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

# Update performance baselines
echo "🔄 Updating performance baselines..."
$PERF_TOOL update-baseline --binary $BINARY_PATH --baseline $BASELINE_FILE

echo "✅ Performance baselines updated successfully!"