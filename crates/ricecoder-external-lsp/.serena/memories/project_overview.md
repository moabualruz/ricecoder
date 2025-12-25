# Project Overview

## Purpose
ricecoder-external-lsp is a Rust crate that provides integration with external Language Server Protocol (LSP) servers to deliver semantic code intelligence for RiceCoder. It manages LSP server lifecycle, request routing, response transformation, and graceful fallback to internal providers when external servers are unavailable.

## Tech Stack
- **Language**: Rust (edition from workspace)
- **Async Runtime**: Tokio
- **Serialization**: Serde (JSON/YAML)
- **Error Handling**: Thiserror, Anyhow
- **Logging**: Tracing
- **Dependencies**: ricecoder-storage, ricecoder-lsp, ricecoder-completion (workspace crates)
- **Testing**: Proptest for property testing, Tokio-test

## Codebase Structure
- `src/client/`: LSP client communication and protocol handling
- `src/mapping/`: Output mapping and transformation from LSP responses
- `src/merger/`: Response merging from external and internal sources
- `src/process/`: LSP server process management and health monitoring
- `src/registry/`: LSP server registry and configuration management
- `src/types.rs`: Core data structures
- `src/error.rs`: Error types
- `src/semantic.rs`: Semantic features integration
- `src/storage_integration.rs`: Configuration persistence
- `src/lib.rs`: Main library interface

## Key Features
- Configuration-driven LSP server setup via YAML
- Automatic process management (spawn, monitor, restart)
- Multi-language support (Rust, TypeScript, Python, Go, Java)
- Response transformation to RiceCoder internal models
- Graceful degradation with fallback to internal providers
- Connection pooling and health monitoring

## Supported LSP Servers (Tier 1)
- rust-analyzer (Rust)
- typescript-language-server (TypeScript/JavaScript)
- pylsp (Python)