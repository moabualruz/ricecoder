//! Client pool management

use crate::error::Result;
use crate::types::LspServerConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Information about a pooled client
#[derive(Clone)]
struct PooledClient {
    /// Configuration for this client
    config: LspServerConfig,
    /// Last access time
    last_access: Instant,
    /// Number of active references
    ref_count: usize,
}

/// Manages a pool of LSP client connections
pub struct ClientPool {
    /// Map of language to pooled clients
    clients: Arc<RwLock<HashMap<String, PooledClient>>>,
    /// Maximum concurrent processes
    max_processes: usize,
    /// Idle timeout for clients
    idle_timeout: Duration,
}

impl ClientPool {
    /// Create a new client pool
    pub fn new(max_processes: usize, idle_timeout: Duration) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            max_processes,
            idle_timeout,
        }
    }

    /// Get or create a client for a language
    pub async fn get_or_create(&self, config: LspServerConfig) -> Result<Arc<LspServerConfig>> {
        let mut clients = self.clients.write().await;

        // Check if we already have a client for this language
        if let Some(client) = clients.get_mut(&config.language) {
            client.last_access = Instant::now();
            client.ref_count += 1;
            debug!(
                language = %config.language,
                ref_count = client.ref_count,
                "Reusing existing LSP client"
            );
            return Ok(Arc::new(client.config.clone()));
        }

        // Check if we can create a new client
        if clients.len() >= self.max_processes {
            debug!(
                language = %config.language,
                max_processes = self.max_processes,
                "Client pool at capacity, attempting to reuse idle client"
            );

            // Try to find and remove an idle client
            if let Some(idle_language) = self.find_idle_client(&clients) {
                clients.remove(&idle_language);
                info!(
                    language = %idle_language,
                    "Removed idle LSP client to make room"
                );
            } else {
                return Err(crate::error::ExternalLspError::ProtocolError(format!(
                    "Client pool at capacity ({}) and no idle clients to remove",
                    self.max_processes
                )));
            }
        }

        // Create a new client
        let pooled = PooledClient {
            config: config.clone(),
            last_access: Instant::now(),
            ref_count: 1,
        };

        clients.insert(config.language.clone(), pooled);
        info!(
            language = %config.language,
            pool_size = clients.len(),
            "Created new LSP client in pool"
        );

        Ok(Arc::new(config))
    }

    /// Release a client reference
    pub async fn release(&self, language: &str) {
        let mut clients = self.clients.write().await;

        if let Some(client) = clients.get_mut(language) {
            if client.ref_count > 0 {
                client.ref_count -= 1;
                client.last_access = Instant::now();
                debug!(
                    language = language,
                    ref_count = client.ref_count,
                    "Released LSP client reference"
                );
            }
        }
    }

    /// Get the number of active clients
    pub async fn active_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }

    /// Get the number of clients with active references
    pub async fn referenced_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.values().filter(|c| c.ref_count > 0).count()
    }

    /// Clean up idle clients
    pub async fn cleanup_idle(&self) -> usize {
        let mut clients = self.clients.write().await;
        let now = Instant::now();

        let idle_languages: Vec<String> = clients
            .iter()
            .filter(|(_, client)| {
                client.ref_count == 0 && now.duration_since(client.last_access) > self.idle_timeout
            })
            .map(|(lang, _)| lang.clone())
            .collect();

        for language in &idle_languages {
            clients.remove(language);
            debug!(language = language, "Cleaned up idle LSP client");
        }

        idle_languages.len()
    }

    /// Find an idle client to remove
    fn find_idle_client(&self, clients: &HashMap<String, PooledClient>) -> Option<String> {
        let now = Instant::now();

        clients
            .iter()
            .filter(|(_, client)| {
                client.ref_count == 0 && now.duration_since(client.last_access) > self.idle_timeout
            })
            .min_by_key(|(_, client)| client.last_access)
            .map(|(lang, _)| lang.clone())
    }

    /// Clear all clients
    pub async fn clear(&self) {
        let mut clients = self.clients.write().await;
        let count = clients.len();
        clients.clear();
        info!(count = count, "Cleared all LSP clients from pool");
    }
}

impl Default for ClientPool {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(language: &str) -> LspServerConfig {
        LspServerConfig {
            language: language.to_string(),
            extensions: vec![],
            executable: format!("{}-lsp", language),
            args: vec![],
            env: Default::default(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    #[tokio::test]
    async fn test_client_pool_creation() {
        let pool = ClientPool::new(5, Duration::from_secs(300));
        assert_eq!(pool.active_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_or_create_client() {
        let pool = ClientPool::new(5, Duration::from_secs(300));
        let config = create_test_config("rust");

        let client = pool.get_or_create(config).await.unwrap();
        assert_eq!(client.language, "rust");
        assert_eq!(pool.active_count().await, 1);
    }

    #[tokio::test]
    async fn test_reuse_existing_client() {
        let pool = ClientPool::new(5, Duration::from_secs(300));
        let config = create_test_config("rust");

        let _client1 = pool.get_or_create(config.clone()).await.unwrap();
        let _client2 = pool.get_or_create(config).await.unwrap();

        // Should still have only 1 client
        assert_eq!(pool.active_count().await, 1);
    }

    #[tokio::test]
    async fn test_pool_capacity() {
        let pool = ClientPool::new(2, Duration::from_secs(300));

        let config1 = create_test_config("rust");
        let config2 = create_test_config("typescript");
        let config3 = create_test_config("python");

        let _client1 = pool.get_or_create(config1).await.unwrap();
        let _client2 = pool.get_or_create(config2).await.unwrap();

        // Third client should fail (pool at capacity)
        let result = pool.get_or_create(config3).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_release_client() {
        let pool = ClientPool::new(5, Duration::from_secs(300));
        let config = create_test_config("rust");

        let _client = pool.get_or_create(config).await.unwrap();
        assert_eq!(pool.referenced_count().await, 1);

        pool.release("rust").await;
        assert_eq!(pool.referenced_count().await, 0);
    }

    #[tokio::test]
    async fn test_clear_pool() {
        let pool = ClientPool::new(5, Duration::from_secs(300));
        let config1 = create_test_config("rust");
        let config2 = create_test_config("typescript");

        let _client1 = pool.get_or_create(config1).await.unwrap();
        let _client2 = pool.get_or_create(config2).await.unwrap();

        assert_eq!(pool.active_count().await, 2);

        pool.clear().await;
        assert_eq!(pool.active_count().await, 0);
    }
}
