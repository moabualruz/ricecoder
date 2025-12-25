# Project Overview

## Purpose
This crate implements the Aider polyglot test suite integration for automated LLM evaluation using Exercism coding exercises. It evaluates LLMs by having them solve 225+ coding exercises across 6 programming languages (C++, Go, Java, JavaScript, Python, Rust).

## Tech Stack
- **Language**: Rust
- **Async Runtime**: Tokio
- **CLI Framework**: Clap (derive API)
- **Serialization**: Serde (JSON/YAML)
- **Error Handling**: ThisError, Anyhow
- **Logging**: Tracing
- **HTTP Client**: Reqwest
- **Dependencies**: ricecoder-providers, ricecoder-domain, ricecoder-di (workspace crates)
- **Testing**: Tokio-test, Proptest (dev dependencies)

## Supported Languages for Benchmarking
- Python: pytest
- Rust: cargo test
- Go: go test
- JavaScript: npm test
- Java: ./gradlew test
- C++: make test (assumes Makefile)

## Architecture
- Exercise Loading: Parses Exercism exercise structure and metadata
- LLM Integration: Uses ricecoder-providers for model interactions
- Test Execution: Runs language-specific test commands with proper isolation
- Results Analysis: Tracks pass rates, costs, and performance metrics
- Concurrent Execution: Supports parallel exercise evaluation