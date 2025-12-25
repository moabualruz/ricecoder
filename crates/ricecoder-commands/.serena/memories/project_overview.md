# Project Overview

Purpose: Custom command system with template substitution and output injection for RiceCoder.

Tech Stack: Rust 2021 edition, async with tokio, serialization with serde, error handling with thiserror, regex for templates.

Structure:
- src/lib.rs: Main library exports
- src/types.rs: Core data structures
- src/error.rs: Error definitions
- src/executor.rs: Command execution logic
- src/registry.rs: Command management
- src/template.rs: Template processing
- src/config.rs: Configuration management
- src/manager.rs: High-level command management
- src/output_injection.rs: Output formatting and injection
- tests/: Integration tests

Dependencies: serde, tokio, thiserror, tracing, regex, uuid, chrono, ricecoder-storage

Guidelines:
- Template safety: Prevent code injection
- Reasonable timeouts
- Clear error messages
- Thorough testing
- Document arguments