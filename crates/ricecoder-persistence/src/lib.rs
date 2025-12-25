//! RiceCoder Persistence Layer
//!
//! Infrastructure layer providing repository implementations for domain aggregates.
//! This crate implements the repository interfaces defined in `ricecoder-domain`.
//!
//! ## Features
//!
//! - **In-Memory Repositories**: Thread-safe in-memory implementations for testing and development
//! - **Future: SurrealDB**: Production-ready persistence with SurrealDB backend
//!
//! ## Architecture
//!
//! Infrastructure implements domain interfaces
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Infrastructure Layer                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  memory/                    │  surreal/ (future)                 │
//! │  ─────────                  │  ────────────────                  │
//! │  InMemoryProjectRepository  │  SurrealProjectRepository          │
//! │  InMemorySessionRepository  │  SurrealSessionRepository          │
//! │  InMemorySpecRepository     │  SurrealSpecRepository             │
//! └─────────────────────────────────────────────────────────────────┘
//!                              ▲
//!                              │ implements
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                       Domain Layer                               │
//! │  ProjectRepository, SessionRepository, SpecificationRepository   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use ricecoder_persistence::memory::InMemoryProjectRepository;
//! use ricecoder_domain::repositories::ProjectRepository;
//! use std::sync::Arc;
//!
//! let repo: Arc<dyn ProjectRepository> = Arc::new(InMemoryProjectRepository::new());
//! ```

pub mod error;
pub mod memory;

pub use error::PersistenceError;

// Re-export commonly used types
pub use memory::{
    InMemoryProjectRepository,
    InMemorySessionRepository,
    InMemorySpecificationRepository,
};
