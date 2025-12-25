# Project Overview

## Purpose
Domain-specific agents providing specialized language and framework support for frontend, backend, DevOps, and cloud development in RiceCoder. Provides specialized AI agents for different development domains including frontend frameworks, backend technologies, DevOps practices, data engineering, mobile development, and cloud architecture.

## Tech Stack
- **Language**: Rust (2021 edition)
- **Async Runtime**: Tokio
- **Serialization**: Serde (JSON, YAML)
- **Error Handling**: Thiserror
- **Logging**: Tracing
- **Dependencies**: ricecoder-providers, ricecoder-storage, ricecoder-agents (workspace dependencies)
- **Testing**: Proptest, tokio-test, tempfile

## Codebase Structure
- `src/lib.rs`: Main library file with module declarations and public exports
- `src/models.rs`: Data models for domains, agents, knowledge bases
- `src/domain_agents.rs`: Implementation of domain-specific agents (Frontend, Backend, DevOps)
- `src/registry.rs`: Central registry for managing domain agents
- `src/knowledge_base.rs`: Knowledge base management for domain-specific information
- `src/error.rs`: Error types and handling
- `tests/integration_tests.rs`: Integration tests

## Architecture
- Domain Agent Manager: Central coordination for domain agents
- Knowledge Base: Domain-specific knowledge entries and best practices
- Registry: Discovery and management of domain agents
- Models: Data structures for domains and agents

## Integration Points
- Extends base agent framework with domain specialization
- Participates in specialized development workflows
- Provides context-aware assistance based on project technology stack