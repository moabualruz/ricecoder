#!/usr/bin/env bash

# Script to download and setup Exercism exercises for benchmarking

set -e

BENCHMARK_REPO="https://github.com/Aider-AI/polyglot-benchmark.git"
BENCHMARK_DIR="tmp.benchmarks/polyglot-benchmark"

echo "Setting up polyglot benchmark exercises..."

# Create benchmark directory
mkdir -p tmp.benchmarks

# Clone or update the benchmark repo
if [ -d "$BENCHMARK_DIR" ]; then
    echo "Updating existing benchmark repository..."
    cd "$BENCHMARK_DIR"
    git pull
    cd -
else
    echo "Cloning benchmark repository..."
    git clone "$BENCHMARK_REPO" "$BENCHMARK_DIR"
fi

echo "Benchmark exercises ready at: $BENCHMARK_DIR"
echo "Available languages:"
ls -1 "$BENCHMARK_DIR" | grep -v README.md