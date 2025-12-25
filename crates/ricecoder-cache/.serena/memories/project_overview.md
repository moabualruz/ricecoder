# RiceCoder Cache Project Overview

## Project Purpose
RiceCoder Cache is a multi-level caching infrastructure library for the RiceCoder project. It provides intelligent caching with support for memory, disk, and remote storage backends, along with various invalidation strategies and performance monitoring.

## Tech Stack
- **Language**: Rust (2021 edition)
- **Async Runtime**: Tokio
- **Serialization**: Serde with JSON support
- **Time Handling**: Chrono
- **Error Handling**: Thiserror
- **Async Traits**: async-trait
- **Compression**: Flate2 (gzip)
- **Testing**: Proptest for property testing, Criterion for benchmarking
- **Domain Logic**: ricecoder-domain (workspace dependency)

## Codebase Structure
```
src/
├── lib.rs           # Main library exports and module declarations
├── cache.rs         # Core cache implementation with multi-level support
├── storage.rs       # Storage backends (Memory, Disk, Remote)
├── strategy.rs      # Cache invalidation strategies (TTL, File change, etc.)
├── metrics.rs       # Performance monitoring and statistics
├── compression.rs   # Data compression utilities
└── error.rs         # Error types and handling
```

## Key Features
- Multi-level caching (L1/L2/L3)
- Intelligent invalidation strategies
- Performance monitoring and metrics
- Async operations throughout
- Generic data type support
- Builder pattern for configuration
- Strategy pattern for extensibility
- Optional compression

## Architecture
The cache follows a hierarchical design with pluggable storage backends and invalidation strategies. It supports async operations and provides detailed metrics for performance monitoring.

## Development Environment
- Windows-based development
- Cargo for build system and dependency management
- Standard Rust tooling (fmt, clippy, test, bench)