# Project Overview

## Purpose
The ricecoder-agents crate provides implementations for various AI agents used in the ricecoder system, which is a tool for code generation and assistance. It includes agents for backend development, code review, devops, and web development, along with domain logic for coordination, orchestration, and other functionalities.

## Tech Stack
- Language: Rust
- Key dependencies: tokio, serde, etc. (based on typical Rust async projects)

## Codebase Structure
- src/agents/: Implementations of different agents (backend, code_review, devops, web)
- src/domain/: Domain logic including coordinator, orchestrator, models, registry, etc.
- src/: Core modules like executor, registry, scheduler, tool_invokers, etc.
- tests/: Integration and unit tests