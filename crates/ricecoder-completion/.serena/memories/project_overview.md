# Project Overview

## Purpose
ricecoder-completion is a language-agnostic code completion engine with external LSP integration and ghost text support for RiceCoder. It provides comprehensive code completion that integrates with external Language Server Protocol (LSP) servers while maintaining graceful fallback to internal providers.

## Tech Stack
- **Language**: Rust
- **Async Runtime**: Tokio
- **Parsing**: Tree-sitter (with language-specific parsers for Rust, TypeScript, Python)
- **LSP Integration**: lsp-types for protocol communication
- **Storage**: ricecoder-storage for configuration and caching
- **Dependencies**: Managed via Cargo workspace

## Features
- External LSP Integration (rust-analyzer, tsserver, pylsp, etc.)
- Multi-Language Support: Rust, TypeScript, Python, Go, Java, Kotlin, Dart, generic text
- Ghost Text: Inline completion suggestions with partial acceptance
- Context Analysis: Code-aware completion based on scope and symbols
- Intelligent Ranking: Relevance-based sorting with frequency and recency scoring
- Fallback Providers: Graceful degradation when external LSP unavailable
- Configuration-Driven: Language-specific completion rules and snippets

## Architecture
- **Responsibilities**: Completion request orchestration, external LSP communication, context analysis, completion ranking, ghost text management
- **Integration Points**: LSP servers, TUI interface, storage layer, history tracking

## Codebase Structure
- `src/config.rs`: Configuration loading and language-specific settings
- `src/context.rs`: Code context analysis using tree-sitter
- `src/engine.rs`: Main completion engine and traits
- `src/external_lsp_proxy.rs`: External LSP server communication
- `src/ghost_text.rs`: Ghost text generation
- `src/ghost_text_state.rs`: Ghost text state management
- `src/history.rs`: Completion usage tracking
- `src/language.rs`: Language detection and handling
- `src/providers.rs`: Language-specific completion providers
- `src/ranker.rs`: Completion ranking algorithms
- `src/types.rs`: Core data types and traits
- `tests/`: Comprehensive test suite including property tests