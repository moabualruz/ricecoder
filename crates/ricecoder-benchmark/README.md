# RiceCoder Benchmark

This crate implements the Aider polyglot test suite integration for automated LLM evaluation using Exercism coding exercises.

## Overview

The benchmark evaluates LLMs by having them solve 225+ coding exercises across 6 programming languages (C++, Go, Java, JavaScript, Python, Rust). Each exercise includes:

- Problem description and requirements
- Solution template files
- Unit tests for validation
- Instructions for implementation

## Setup

1. Download the exercises:
   ```bash
   ./scripts/setup-benchmark.sh
   ```

2. Build the benchmark tool:
   ```bash
   cargo build --release --bin ricecoder-benchmark
   ```

## Usage

### Run a benchmark
```bash
./target/release/ricecoder-benchmark run --model openai/gpt-4 --exercises-dir tmp.benchmarks/polyglot-benchmark --results-dir benchmark-results
```

### List available exercises
```bash
./target/release/ricecoder-benchmark list --exercises-dir tmp.benchmarks/polyglot-benchmark
```

### View results
```bash
./target/release/ricecoder-benchmark summary --results-dir benchmark-results
```

## Architecture

- **Exercise Loading**: Parses Exercism exercise structure and metadata
- **LLM Integration**: Uses ricecoder-providers for model interactions
- **Test Execution**: Runs language-specific test commands with proper isolation
- **Results Analysis**: Tracks pass rates, costs, and performance metrics
- **Concurrent Execution**: Supports parallel exercise evaluation

## Supported Languages

- Python: `pytest`
- Rust: `cargo test`
- Go: `go test`
- JavaScript: `npm test`
- Java: `./gradlew test`
- C++: `make test` (assumes Makefile)

## Results Format

Results are saved as JSON with comprehensive metrics:

```json
{
  "run_id": "gpt-4-2024-01-01-12-00-00",
  "model": "openai/gpt-4",
  "total_exercises": 225,
  "completed_exercises": 225,
  "pass_rates": [75.1, 88.4],
  "total_cost": 12.34,
  "average_duration_per_exercise": 45.2,
  "exercise_results": [...]
}
```

## Performance Targets

Based on Aider leaderboard:
- GPT-5: 88.0% pass rate
- Claude 3.5 Sonnet: ~77%
- GPT-4: ~75%

## Integration

The benchmark integrates with ricecoder's provider ecosystem for continuous performance validation and automated regression testing.