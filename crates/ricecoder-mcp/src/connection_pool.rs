//! Connection pool for managing MCP server connections

use std::{collections::VecDeque, sync::Arc};

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{Error, Result};

/// Represents a pooled connection to an MCP server
#[derive(Debug, Clone)]
pub struct PooledConnection {
    pub id: String,
    pub server_id: String,
    pub is_valid: bool,
    pub last_used: std::time::Instant,
}

impl PooledConnection {
    /// Creates a new pooled connection
    pub fn new(id: String, server_id: String) -> Self {
        Self {
            id,
            server_id,
            is_valid: true,
            last_used: std::time::Instant::now(),
        }
    }

    /// Marks the connection as used
    pub fn mark_used(&mut self) {
        self.last_used = std::time::Instant::now();
    }

    /// Checks if the connection is still valid
    pub fn is_still_valid(&self) -> bool {
        self.is_valid
    }

    /// Invalidates the connection
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }
}

/// Configuration for the connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub min_connections: usize,
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub idle_timeout_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connection_timeout_ms: 2000,
            idle_timeout_ms: 30000,
        }
    }
}

/// Connection pool for managing MCP server connections
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    config: PoolConfig,
    available: Arc<RwLock<VecDeque<PooledConnection>>>,
    in_use: Arc<RwLock<Vec<PooledConnection>>>,
    connection_counter: Arc<RwLock<u64>>,
}

impl ConnectionPool {
    /// Creates a new connection pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Creates a new connection pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            config,
            available: Arc::new(RwLock::new(VecDeque::new())),
            in_use: Arc::new(RwLock::new(Vec::new())),
            connection_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Acquires a connection from the pool
    ///
    /// # Arguments
    /// * `server_id` - The server ID for which to acquire a connection
    ///
    /// # Returns
    /// A pooled connection
    ///
    /// # Errors
    /// Returns error if pool is at max capacity or connection creation fails
    pub async fn acquire(&self, server_id: &str) -> Result<PooledConnection> {
        debug!("Acquiring connection for server: {}", server_id);

        // Try to get an available connection
        let mut available = self.available.write().await;
        if let Some(mut conn) = available.pop_front() {
            if conn.is_still_valid() {
                conn.mark_used();
                let mut in_use = self.in_use.write().await;
                in_use.push(conn.clone());
                info!("Reused connection from pool for server: {}", server_id);
                return Ok(conn);
            }
        }

        // Check if we can create a new connection
        let in_use = self.in_use.read().await;
        if in_use.len() >= self.config.max_connections {
            return Err(Error::ConnectionError(
                "Connection pool at maximum capacity".to_string(),
            ));
        }
        drop(in_use);

        // Create a new connection
        let mut counter = self.connection_counter.write().await;
        *counter += 1;
        let conn_id = format!("conn-{}", counter);
        drop(counter);

        let conn = PooledConnection::new(conn_id, server_id.to_string());
        let mut in_use = self.in_use.write().await;
        in_use.push(conn.clone());

        info!(
            "Created new connection for server: {} (total: {})",
            server_id,
            in_use.len()
        );
        Ok(conn)
    }

    /// Releases a connection back to the pool
    ///
    /// # Arguments
    /// * `connection` - The connection to release
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn release(&self, connection: PooledConnection) -> Result<()> {
        debug!("Releasing connection: {}", connection.id);

        let mut in_use = self.in_use.write().await;
        in_use.retain(|c| c.id != connection.id);
        drop(in_use);

        if connection.is_still_valid() {
            let mut available = self.available.write().await;
            available.push_back(connection);
            info!("Connection returned to pool");
        } else {
            info!("Connection invalidated, not returned to pool");
        }

        Ok(())
    }

    /// Validates a connection
    ///
    /// # Arguments
    /// * `connection` - The connection to validate
    ///
    /// # Returns
    /// True if connection is valid, false otherwise
    pub async fn validate(&self, connection: &PooledConnection) -> bool {
        debug!("Validating connection: {}", connection.id);

        // Check if connection is still in use
        let in_use = self.in_use.read().await;
        let is_in_use = in_use.iter().any(|c| c.id == connection.id);

        if !is_in_use {
            warn!("Connection not in use: {}", connection.id);
            return false;
        }

        // Check if connection is still valid
        if !connection.is_still_valid() {
            warn!("Connection is invalid: {}", connection.id);
            return false;
        }

        true
    }

    /// Performs health check on all connections
    ///
    /// # Returns
    /// Number of invalid connections removed
    pub async fn health_check(&self) -> usize {
        debug!("Performing health check on connection pool");

        let mut available = self.available.write().await;
        let initial_count = available.len();

        // Remove invalid connections
        available.retain(|c| c.is_still_valid());

        let removed = initial_count - available.len();
        if removed > 0 {
            info!("Removed {} invalid connections from pool", removed);
        }

        removed
    }

    /// Gets the current pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let available = self.available.read().await;
        let in_use = self.in_use.read().await;

        PoolStats {
            available_connections: available.len(),
            in_use_connections: in_use.len(),
            total_connections: available.len() + in_use.len(),
            max_connections: self.config.max_connections,
        }
    }

    /// Clears all connections from the pool
    pub async fn clear(&self) {
        debug!("Clearing connection pool");

        let mut available = self.available.write().await;
        available.clear();

        let mut in_use = self.in_use.write().await;
        in_use.clear();

        info!("Connection pool cleared");
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available_connections: usize,
    pub in_use_connections: usize,
    pub total_connections: usize,
    pub max_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool() {
        let pool = ConnectionPool::new();
        let stats = pool.get_stats().await;
        assert_eq!(stats.available_connections, 0);
        assert_eq!(stats.in_use_connections, 0);
    }

    #[tokio::test]
    async fn test_acquire_connection() {
        let pool = ConnectionPool::new();
        let conn = pool.acquire("server1").await.unwrap();
        assert_eq!(conn.server_id, "server1");
        assert!(conn.is_still_valid());
    }

    #[tokio::test]
    async fn test_release_connection() {
        let pool = ConnectionPool::new();
        let conn = pool.acquire("server1").await.unwrap();
        let result = pool.release(conn).await;
        assert!(result.is_ok());

        let stats = pool.get_stats().await;
        assert_eq!(stats.available_connections, 1);
        assert_eq!(stats.in_use_connections, 0);
    }

    #[tokio::test]
    async fn test_reuse_connection() {
        let pool = ConnectionPool::new();
        let conn1 = pool.acquire("server1").await.unwrap();
        let conn1_id = conn1.id.clone();

        pool.release(conn1).await.unwrap();

        let conn2 = pool.acquire("server1").await.unwrap();
        assert_eq!(conn2.id, conn1_id);
    }

    #[tokio::test]
    async fn test_max_connections() {
        let config = PoolConfig {
            min_connections: 1,
            max_connections: 2,
            connection_timeout_ms: 2000,
            idle_timeout_ms: 30000,
        };
        let pool = ConnectionPool::with_config(config);

        let conn1 = pool.acquire("server1").await.unwrap();
        let conn2 = pool.acquire("server1").await.unwrap();

        let result = pool.acquire("server1").await;
        assert!(result.is_err());

        pool.release(conn1).await.unwrap();
        pool.release(conn2).await.unwrap();
    }

    #[tokio::test]
    async fn test_validate_connection() {
        let pool = ConnectionPool::new();
        let conn = pool.acquire("server1").await.unwrap();

        let is_valid = pool.validate(&conn).await;
        assert!(is_valid);

        pool.release(conn.clone()).await.unwrap();

        let is_valid = pool.validate(&conn).await;
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_health_check() {
        let pool = ConnectionPool::new();
        let mut conn = pool.acquire("server1").await.unwrap();
        pool.release(conn.clone()).await.unwrap();

        conn.invalidate();
        let mut available = pool.available.write().await;
        available.push_back(conn);
        drop(available);

        let removed = pool.health_check().await;
        assert_eq!(removed, 1);
    }

    #[tokio::test]
    async fn test_clear_pool() {
        let pool = ConnectionPool::new();
        let _conn1 = pool.acquire("server1").await.unwrap();
        let _conn2 = pool.acquire("server1").await.unwrap();

        pool.clear().await;

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 0);
    }
}
