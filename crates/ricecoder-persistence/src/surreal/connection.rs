//! SurrealDB Connection Management
//!
//! Supports embedded (in-memory) and client-server modes.
//!
//! ## Modes
//!
//! - **Embedded (Memory)**: `kv-mem` - In-memory, no persistence (testing/development)
//! - **Client (WebSocket)**: `protocol-ws` - Remote server connection (production)
//!
//! **Note**: File-based embedded storage (kv-rocksdb, kv-surrealkv) is disabled due to
//! ndarray version conflict with onnxruntime. For persistent storage, use a SurrealDB server.
//!
//! ## Usage
//!
//! ```ignore
//! use ricecoder_persistence::surreal::{SurrealConnection, ConnectionMode};
//!
//! // Embedded in-memory (default, for testing)
//! let conn = SurrealConnection::new(ConnectionMode::Memory).await?;
//!
//! // Remote server (production)
//! let conn = SurrealConnection::new(ConnectionMode::Remote {
//!     url: "ws://localhost:8000".into(),
//!     username: "root".into(),
//!     password: "secret".into(),
//! }).await?;
//! ```

use std::sync::Arc;

use surrealdb::engine::local::{Db as LocalDb, Mem};
use surrealdb::engine::remote::ws::{Client as WsClient, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use thiserror::Error;
use tracing::{debug, info};

/// Connection mode for SurrealDB
#[derive(Debug, Clone)]
pub enum ConnectionMode {
    /// In-memory database (no persistence, fast, for testing/development)
    Memory,
    /// Remote server connection via WebSocket (production)
    Remote {
        url: String,
        username: String,
        password: String,
    },
}

impl Default for ConnectionMode {
    fn default() -> Self {
        Self::Memory
    }
}

/// SurrealDB connection errors
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Failed to connect to SurrealDB: {0}")]
    Connection(String),
    #[error("Authentication failed: {0}")]
    Authentication(String),
    #[error("Database selection failed: {0}")]
    DatabaseSelection(String),
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::Error),
}

/// Unified database client that works with both embedded and remote modes
pub enum DatabaseClient {
    /// Embedded local database (Memory or RocksDB)
    Local(Surreal<LocalDb>),
    /// Remote WebSocket client
    Remote(Surreal<WsClient>),
}

impl DatabaseClient {
    /// Get a reference to the underlying Surreal client for local mode
    pub fn as_local(&self) -> Option<&Surreal<LocalDb>> {
        match self {
            Self::Local(db) => Some(db),
            Self::Remote(_) => None,
        }
    }

    /// Get a reference to the underlying Surreal client for remote mode
    pub fn as_remote(&self) -> Option<&Surreal<WsClient>> {
        match self {
            Self::Local(_) => None,
            Self::Remote(db) => Some(db),
        }
    }
}

/// SurrealDB connection wrapper
///
/// Manages connection lifecycle and provides unified access to database operations.
pub struct SurrealConnection {
    client: DatabaseClient,
    mode: ConnectionMode,
    namespace: String,
    database: String,
}

impl SurrealConnection {
    /// Create a new SurrealDB connection with the specified mode
    ///
    /// Default namespace: "ricecoder"
    /// Default database: "main"
    pub async fn new(mode: ConnectionMode) -> Result<Self, ConnectionError> {
        Self::with_names(mode, "ricecoder", "main").await
    }

    /// Create a new SurrealDB connection with custom namespace and database
    pub async fn with_names(
        mode: ConnectionMode,
        namespace: &str,
        database: &str,
    ) -> Result<Self, ConnectionError> {
        let client = match &mode {
            ConnectionMode::Memory => {
                info!("Connecting to SurrealDB in-memory mode");
                let db = Surreal::new::<Mem>(())
                    .await
                    .map_err(|e| ConnectionError::Connection(e.to_string()))?;
                DatabaseClient::Local(db)
            }

            ConnectionMode::Remote { url, username, password } => {
                info!("Connecting to SurrealDB remote at {}", url);
                let db = Surreal::new::<Ws>(url.as_str())
                    .await
                    .map_err(|e| ConnectionError::Connection(e.to_string()))?;
                
                // Authenticate for remote connections
                db.signin(Root {
                    username: username.as_str(),
                    password: password.as_str(),
                })
                .await
                .map_err(|e| ConnectionError::Authentication(e.to_string()))?;
                
                DatabaseClient::Remote(db)
            }
        };

        // Select namespace and database
        match &client {
            DatabaseClient::Local(db) => {
                db.use_ns(namespace).use_db(database).await
                    .map_err(|e| ConnectionError::DatabaseSelection(e.to_string()))?;
            }
            DatabaseClient::Remote(db) => {
                db.use_ns(namespace).use_db(database).await
                    .map_err(|e| ConnectionError::DatabaseSelection(e.to_string()))?;
            }
        }

        debug!("Connected to SurrealDB namespace={} database={}", namespace, database);

        Ok(Self {
            client,
            mode,
            namespace: namespace.to_string(),
            database: database.to_string(),
        })
    }

    /// Get the connection mode
    pub fn mode(&self) -> &ConnectionMode {
        &self.mode
    }

    /// Get the namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the database
    pub fn database(&self) -> &str {
        &self.database
    }

    /// Get the underlying database client
    pub fn client(&self) -> &DatabaseClient {
        &self.client
    }

    /// Check if connected in embedded mode
    pub fn is_embedded(&self) -> bool {
        matches!(self.mode, ConnectionMode::Memory)
    }

    /// Check if connected in remote mode
    pub fn is_remote(&self) -> bool {
        matches!(self.mode, ConnectionMode::Remote { .. })
    }
}

/// Shared connection pool for SurrealDB
///
/// Thread-safe wrapper for sharing a connection across repositories.
pub type SharedConnection = Arc<SurrealConnection>;

/// Create a shared connection
pub async fn create_shared_connection(mode: ConnectionMode) -> Result<SharedConnection, ConnectionError> {
    Ok(Arc::new(SurrealConnection::new(mode).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_connection() {
        let conn = SurrealConnection::new(ConnectionMode::Memory).await;
        assert!(conn.is_ok());
        let conn = conn.unwrap();
        assert!(conn.is_embedded());
        assert!(!conn.is_remote());
        assert_eq!(conn.namespace(), "ricecoder");
        assert_eq!(conn.database(), "main");
    }

    #[tokio::test]
    async fn test_custom_namespace() {
        let conn = SurrealConnection::with_names(
            ConnectionMode::Memory,
            "custom_ns",
            "custom_db",
        ).await;
        assert!(conn.is_ok());
        let conn = conn.unwrap();
        assert_eq!(conn.namespace(), "custom_ns");
        assert_eq!(conn.database(), "custom_db");
    }
}
