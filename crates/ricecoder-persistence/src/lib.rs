//! RiceCoder Persistence Layer
//!
//! Infrastructure layer providing repository implementations for domain aggregates.
//! This crate implements the repository interfaces defined in `ricecoder-domain`.
//!
//! ## Features
//!
//! - **In-Memory Repositories**: Thread-safe in-memory implementations for testing and development
//! - **SurrealDB Repositories**: Production-ready persistence with SurrealDB backend
//!
//! ## Architecture
//!
//! Infrastructure implements domain interfaces
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Infrastructure Layer                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  memory/                    │  surreal/                          │
//! │  ─────────                  │  ────────                          │
//! │  InMemoryProjectRepository  │  SurrealProjectRepository          │
//! │  InMemorySessionRepository  │  SurrealSessionRepository          │
//! │  InMemorySpecRepository     │  SurrealSpecificationRepository    │
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
//! ### In-Memory (Testing/Development)
//!
//! ```ignore
//! use ricecoder_persistence::memory::InMemoryProjectRepository;
//! use ricecoder_domain::repositories::ProjectRepository;
//! use std::sync::Arc;
//!
//! let repo: Arc<dyn ProjectRepository> = Arc::new(InMemoryProjectRepository::new());
//! ```
//!
//! ### SurrealDB (Production)
//!
//! ```ignore
//! use ricecoder_persistence::surreal::{
//!     SurrealConnection, ConnectionMode, SurrealProjectRepository,
//! };
//! use ricecoder_domain::repositories::ProjectRepository;
//! use std::sync::Arc;
//!
//! // Embedded in-memory (default)
//! let conn = Arc::new(SurrealConnection::new(ConnectionMode::Memory).await?);
//!
//! // Or file-based persistent
//! let conn = Arc::new(SurrealConnection::new(ConnectionMode::File("./data".into())).await?);
//!
//! // Or remote server
//! let conn = Arc::new(SurrealConnection::new(ConnectionMode::Remote {
//!     url: "ws://localhost:8000".into(),
//!     username: "root".into(),
//!     password: "secret".into(),
//! }).await?);
//!
//! let repo: Arc<dyn ProjectRepository> = Arc::new(SurrealProjectRepository::new(conn));
//! ```

pub mod error;
pub mod memory;

// SurrealDB backend for production persistence
#[cfg(feature = "surrealdb-backend")]
pub mod surreal;

pub use error::PersistenceError;

// Re-export commonly used types
pub use memory::{
    InMemoryProjectRepository,
    InMemorySessionRepository,
    InMemorySpecificationRepository,
};

// SurrealDB exports
#[cfg(feature = "surrealdb-backend")]
pub use surreal::{
    ConnectionError, ConnectionMode, SurrealConnection, SharedConnection,
    SurrealProjectRepository, SurrealSessionRepository, SurrealSpecificationRepository,
    create_shared_connection,
};
