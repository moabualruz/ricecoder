# Project Overview

## Purpose
Safe file operations with atomic writes, backups, rollback support, and comprehensive audit logging for RiceCoder.

## Tech Stack
- Language: Rust
- Async runtime: Tokio
- Serialization: Serde, Serde JSON
- Git integration: git2
- Storage: ricecoder-storage (workspace dependency)

## Codebase Structure
- src/: Core modules for audit, backup, conflict, diff, error, file_repository, git, manager, models, transaction, verifier, watcher, writer
- tests/: Property and integration tests for various features
- Cargo.toml: Package configuration
- README.md: Documentation

## Key Features
- Atomic writes
- Automatic backups
- Transaction support
- Conflict resolution
- Audit logging
- Git integration
- File watching
- Content verification
- Rollback support