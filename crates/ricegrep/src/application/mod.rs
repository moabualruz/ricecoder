//! Application Layer - Orchestration and Ports
//!
//! This module contains the application layer for RiceGrep, following
//! hexagonal architecture (ports and adapters) pattern.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────┐
//! │   MCP Protocol Layer        │ ← Thin adapters, request/response mapping
//! ├─────────────────────────────┤
//! │   Application Layer         │ ← Use cases, repository traits, orchestration (THIS LAYER)
//! ├─────────────────────────────┤
//! │   Domain Layer              │ ← Pure business logic, value objects, aggregates
//! ├─────────────────────────────┤
//! │   Infrastructure Layer      │ ← File I/O, indexing, actual implementations
//! └─────────────────────────────┘
//! ```
//!
//! # Design Principles (REQ-ARCH-002)
//! - Repository traits define ports (interfaces) for infrastructure
//! - No async initially (can add later for performance)
//! - <500 lines total for application layer
//! - Infrastructure implements these traits
//! - Domain types flow through without modification
//!
//! # Modules
//! - `errors` - Application error types
//! - `ports` - Repository traits (interfaces)
//! - `use_cases` - Business operation orchestrators
//! - `services` - Dependency injection container

pub mod errors;
pub mod ports;
pub mod use_cases;
pub mod services;

// Re-export for ergonomic imports
pub use errors::*;
pub use ports::*;
pub use use_cases::*;
pub use services::*;
