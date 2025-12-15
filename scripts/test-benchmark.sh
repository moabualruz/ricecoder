#!/usr/bin/env bash

# Build and test the benchmark crate

set -e

echo "Building ricecoder-benchmark crate..."
cargo build -p ricecoder-benchmark

echo "Running tests for ricecoder-benchmark..."
cargo test -p ricecoder-benchmark

echo "Building benchmark binary..."
cargo build --release --bin ricecoder-benchmark

echo "Benchmark crate build successful!"