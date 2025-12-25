//! SurrealDB Repository Implementations
//!
//! Production-ready persistence backend using SurrealDB.
//!
//! ## Modes
//!
//! - **Embedded (Memory)**: In-memory, no persistence (testing/development)
//! - **Embedded (RocksDB)**: File-based, persistent (production)
//! - **Client (WebSocket)**: Remote server connection (distributed)
//!
//! ## Usage
//!
//! ```ignore
//! use ricecoder_persistence::surreal::{
//!     SurrealConnection, ConnectionMode, SharedConnection,
//!     SurrealProjectRepository, SurrealSessionRepository, SurrealSpecificationRepository,
//! };
//! use std::sync::Arc;
//!
//! // Create shared connection (embedded in-memory)
//! let conn = Arc::new(SurrealConnection::new(ConnectionMode::Memory).await?);
//!
//! // Create repositories
//! let project_repo = SurrealProjectRepository::new(conn.clone());
//! let session_repo = SurrealSessionRepository::new(conn.clone());
//! let spec_repo = SurrealSpecificationRepository::new(conn.clone());
//! ```

pub mod connection;
pub mod project_repository;
pub mod session_repository;
pub mod specification_repository;

pub use connection::{
    ConnectionError, ConnectionMode, DatabaseClient, SharedConnection, SurrealConnection,
    create_shared_connection,
};
pub use project_repository::SurrealProjectRepository;
pub use session_repository::SurrealSessionRepository;
pub use specification_repository::SurrealSpecificationRepository;
