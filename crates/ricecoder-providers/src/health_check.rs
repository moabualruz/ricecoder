//! Health check system with caching and timeout support

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::ProviderError;
use crate::provider::Provider;

/// Health check result with timestamp
#[derive(Clone, Debug)]
pub struct HealthCheckResult {
    /// Whether the provider is healthy
    pub is_healthy: bool,
    /// When the check was performed
    pub checked_at: Instant,
    /// Error if the check failed
    pub error: Option<String>,
}

impl HealthCheckResult {
    /// Check if the result is still valid (not expired)
    pub fn is_valid(&self, ttl: Duration) -> bool {
        self.checked_at.elapsed() < ttl
    }
}

/// Health check cache for providers
pub struct HealthCheckCache {
    /// Cache of health check results by provider ID
    cache: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    /// Time-to-live for cached results
    ttl: Duration,
    /// Timeout for health check operations
    timeout: Duration,
}

impl HealthCheckCache {
    /// Create a new health check cache
    pub fn new(ttl: Duration, timeout: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            timeout,
        }
    }
}

impl Default for HealthCheckCache {
    /// Create a new health check cache with default settings
    /// - TTL: 5 minutes
    /// - Timeout: 10 seconds
    fn default() -> Self {
        Self::new(Duration::from_secs(300), Duration::from_secs(10))
    }
}

impl HealthCheckCache {
    /// Check provider health with caching
    pub async fn check_health(&self, provider: &Arc<dyn Provider>) -> Result<bool, ProviderError> {
        let provider_id = provider.id();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(result) = cache.get(provider_id) {
                if result.is_valid(self.ttl) {
                    debug!(
                        "Using cached health check for provider: {} (healthy: {})",
                        provider_id, result.is_healthy
                    );
                    return if result.is_healthy {
                        Ok(true)
                    } else {
                        Err(ProviderError::ProviderError(
                            result
                                .error
                                .clone()
                                .unwrap_or_else(|| "Provider unhealthy".to_string()),
                        ))
                    };
                }
            }
        }

        // Perform health check with timeout
        debug!("Performing health check for provider: {}", provider_id);
        let result = match tokio::time::timeout(self.timeout, provider.health_check()).await {
            Ok(Ok(is_healthy)) => HealthCheckResult {
                is_healthy,
                checked_at: Instant::now(),
                error: None,
            },
            Ok(Err(e)) => {
                warn!("Health check failed for provider {}: {}", provider_id, e);
                HealthCheckResult {
                    is_healthy: false,
                    checked_at: Instant::now(),
                    error: Some(e.to_string()),
                }
            }
            Err(_) => {
                warn!("Health check timeout for provider: {}", provider_id);
                HealthCheckResult {
                    is_healthy: false,
                    checked_at: Instant::now(),
                    error: Some("Health check timeout".to_string()),
                }
            }
        };

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(provider_id.to_string(), result.clone());
        }

        if result.is_healthy {
            Ok(true)
        } else {
            Err(ProviderError::ProviderError(
                result
                    .error
                    .unwrap_or_else(|| "Provider unhealthy".to_string()),
            ))
        }
    }

    /// Invalidate cache for a specific provider
    pub async fn invalidate(&self, provider_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(provider_id);
        debug!(
            "Invalidated health check cache for provider: {}",
            provider_id
        );
    }

    /// Invalidate all cached results
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Invalidated all health check cache");
    }

    /// Get cached result without performing a new check
    pub async fn get_cached(&self, provider_id: &str) -> Option<HealthCheckResult> {
        let cache = self.cache.read().await;
        cache.get(provider_id).cloned()
    }

    /// Set TTL for cached results
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set timeout for health check operations
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
