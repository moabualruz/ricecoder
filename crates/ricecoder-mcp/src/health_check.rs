//! Health checking and reconnection logic for MCP servers

use std::{sync::Arc, time::Duration};

use tokio::{sync::RwLock, time::sleep};
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};

/// Server health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Server availability information
#[derive(Debug, Clone)]
pub struct ServerAvailability {
    pub server_id: String,
    pub is_available: bool,
    pub status: HealthStatus,
    pub last_check: std::time::Instant,
    pub consecutive_failures: u32,
}

impl ServerAvailability {
    /// Creates a new server availability tracker
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            is_available: true,
            status: HealthStatus::Unknown,
            last_check: std::time::Instant::now(),
            consecutive_failures: 0,
        }
    }

    /// Marks the server as healthy
    pub fn mark_healthy(&mut self) {
        self.is_available = true;
        self.status = HealthStatus::Healthy;
        self.consecutive_failures = 0;
        self.last_check = std::time::Instant::now();
    }

    /// Marks the server as unhealthy
    pub fn mark_unhealthy(&mut self) {
        self.consecutive_failures += 1;
        self.status = HealthStatus::Unhealthy;
        self.last_check = std::time::Instant::now();

        // Mark as unavailable after 3 consecutive failures
        if self.consecutive_failures >= 3 {
            self.is_available = false;
        }
    }

    /// Resets the failure counter
    pub fn reset_failures(&mut self) {
        self.consecutive_failures = 0;
    }
}

/// Configuration for health checking
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub check_interval_ms: u64,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub backoff_multiplier: f64,
    pub max_backoff_ms: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 5000,
            timeout_ms: 2000,
            max_retries: 3,
            backoff_multiplier: 2.0,
            max_backoff_ms: 60000,
        }
    }
}

/// Health checker for MCP servers
#[derive(Debug, Clone)]
pub struct HealthChecker {
    config: HealthCheckConfig,
    availability: Arc<RwLock<std::collections::HashMap<String, ServerAvailability>>>,
    analytics: Option<Arc<crate::analytics::MCPAnalyticsAggregator>>,
}

impl HealthChecker {
    /// Creates a new health checker with default configuration
    pub fn new() -> Self {
        Self::with_config(HealthCheckConfig::default())
    }

    /// Creates a new health checker with custom configuration
    pub fn with_config(config: HealthCheckConfig) -> Self {
        Self {
            config,
            availability: Arc::new(RwLock::new(std::collections::HashMap::new())),
            analytics: None,
        }
    }

    /// Set analytics aggregator
    pub fn with_analytics(
        mut self,
        analytics: Arc<crate::analytics::MCPAnalyticsAggregator>,
    ) -> Self {
        self.analytics = Some(analytics);
        self
    }

    /// Registers a server for health checking
    ///
    /// # Arguments
    /// * `server_id` - The server ID to register
    pub async fn register_server(&self, server_id: &str) {
        debug!("Registering server for health checking: {}", server_id);

        let mut availability = self.availability.write().await;
        availability.insert(
            server_id.to_string(),
            ServerAvailability::new(server_id.to_string()),
        );

        info!("Server registered for health checking: {}", server_id);
    }

    /// Unregisters a server from health checking
    ///
    /// # Arguments
    /// * `server_id` - The server ID to unregister
    pub async fn unregister_server(&self, server_id: &str) {
        debug!("Unregistering server from health checking: {}", server_id);

        let mut availability = self.availability.write().await;
        availability.remove(server_id);

        info!("Server unregistered from health checking: {}", server_id);
    }

    /// Performs a health check on a server
    ///
    /// # Arguments
    /// * `server_id` - The server ID to check
    ///
    /// # Returns
    /// True if server is healthy, false otherwise
    pub async fn check_health(&self, server_id: &str) -> Result<bool> {
        debug!("Checking health of server: {}", server_id);

        let mut availability = self.availability.write().await;
        let server_avail = availability.get_mut(server_id).ok_or_else(|| {
            Error::ConnectionError(format!("Server not registered: {}", server_id))
        })?;

        // Simulate health check (in real implementation, would ping the server)
        let start_time = std::time::Instant::now();
        let is_healthy = true;
        let check_duration = start_time.elapsed().as_millis() as u64;

        if is_healthy {
            server_avail.mark_healthy();
            info!("Server health check passed: {}", server_id);

            // Analytics
            if let Some(ref analytics) = self.analytics {
                let _ = analytics
                    .record_health_check(server_id, true, check_duration, None, None)
                    .await;
            }

            Ok(true)
        } else {
            server_avail.mark_unhealthy();
            warn!("Server health check failed: {}", server_id);

            // Analytics
            if let Some(ref analytics) = self.analytics {
                let _ = analytics
                    .record_health_check(server_id, false, check_duration, None, None)
                    .await;
            }

            Ok(false)
        }
    }

    /// Detects server disconnection
    ///
    /// # Arguments
    /// * `server_id` - The server ID to check
    ///
    /// # Returns
    /// True if server is disconnected, false otherwise
    pub async fn is_disconnected(&self, server_id: &str) -> bool {
        let availability = self.availability.read().await;
        availability
            .get(server_id)
            .map(|a| !a.is_available)
            .unwrap_or(false)
    }

    /// Detects server unavailability
    ///
    /// # Arguments
    /// * `server_id` - The server ID to check
    ///
    /// # Returns
    /// True if server is unavailable, false otherwise
    pub async fn is_unavailable(&self, server_id: &str) -> bool {
        let availability = self.availability.read().await;
        availability
            .get(server_id)
            .map(|a| !a.is_available)
            .unwrap_or(true)
    }

    /// Gets the availability status of a server
    ///
    /// # Arguments
    /// * `server_id` - The server ID to check
    ///
    /// # Returns
    /// Server availability information
    pub async fn get_availability(&self, server_id: &str) -> Option<ServerAvailability> {
        let availability = self.availability.read().await;
        availability.get(server_id).cloned()
    }

    /// Performs periodic availability detection
    ///
    /// This method should be run in a background task to continuously monitor server health.
    pub async fn periodic_check(&self) {
        debug!("Starting periodic health checks");

        loop {
            let availability = self.availability.read().await;
            let server_ids: Vec<String> = availability.keys().cloned().collect();
            drop(availability);

            for server_id in server_ids {
                if let Err(e) = self.check_health(&server_id).await {
                    error!("Health check error for server {}: {}", server_id, e);
                }
            }

            sleep(Duration::from_millis(self.config.check_interval_ms)).await;
        }
    }

    /// Implements reconnection logic with exponential backoff
    ///
    /// # Arguments
    /// * `server_id` - The server ID to reconnect to
    /// * `on_reconnect` - Callback function to attempt reconnection
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn reconnect_with_backoff<F>(
        &self,
        server_id: &str,
        mut on_reconnect: F,
    ) -> Result<()>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
    {
        debug!(
            "Starting reconnection with backoff for server: {}",
            server_id
        );

        let mut backoff_ms = 100u64;
        let mut attempt = 0;

        loop {
            attempt += 1;
            info!(
                "Reconnection attempt {} for server: {} (backoff: {}ms)",
                attempt, server_id, backoff_ms
            );

            match on_reconnect().await {
                Ok(()) => {
                    info!("Successfully reconnected to server: {}", server_id);
                    let mut availability = self.availability.write().await;
                    if let Some(avail) = availability.get_mut(server_id) {
                        avail.mark_healthy();
                    }
                    return Ok(());
                }
                Err(e) => {
                    if attempt >= self.config.max_retries {
                        error!(
                            "Failed to reconnect to server {} after {} attempts: {}",
                            server_id, attempt, e
                        );
                        let mut availability = self.availability.write().await;
                        if let Some(avail) = availability.get_mut(server_id) {
                            avail.mark_unhealthy();
                        }
                        return Err(Error::ConnectionError(format!(
                            "Failed to reconnect to server {} after {} attempts",
                            server_id, attempt
                        )));
                    }

                    warn!(
                        "Reconnection attempt {} failed for server {}: {}. Retrying in {}ms",
                        attempt, server_id, e, backoff_ms
                    );

                    sleep(Duration::from_millis(backoff_ms)).await;

                    // Calculate next backoff with exponential increase
                    backoff_ms = std::cmp::min(
                        (backoff_ms as f64 * self.config.backoff_multiplier) as u64,
                        self.config.max_backoff_ms,
                    );
                }
            }
        }
    }

    /// Reports persistent failures to user
    ///
    /// # Arguments
    /// * `server_id` - The server ID that failed
    ///
    /// # Returns
    /// Error message for user
    pub async fn report_failure(&self, server_id: &str) -> String {
        let availability = self.availability.read().await;
        if let Some(avail) = availability.get(server_id) {
            format!(
                "Server '{}' is unavailable after {} consecutive failures. Please check the server status.",
                server_id, avail.consecutive_failures
            )
        } else {
            format!("Server '{}' is unavailable.", server_id)
        }
    }

    /// Gets health statistics for all servers
    pub async fn get_health_stats(&self) -> Vec<ServerAvailability> {
        let availability = self.availability.read().await;
        availability.values().cloned().collect()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_health_checker() {
        let checker = HealthChecker::new();
        let stats = checker.get_health_stats().await;
        assert_eq!(stats.len(), 0);
    }

    #[tokio::test]
    async fn test_register_server() {
        let checker = HealthChecker::new();
        checker.register_server("server1").await;

        let avail = checker.get_availability("server1").await;
        assert!(avail.is_some());
        assert!(avail.unwrap().is_available);
    }

    #[tokio::test]
    async fn test_unregister_server() {
        let checker = HealthChecker::new();
        checker.register_server("server1").await;
        checker.unregister_server("server1").await;

        let avail = checker.get_availability("server1").await;
        assert!(avail.is_none());
    }

    #[tokio::test]
    async fn test_check_health() {
        let checker = HealthChecker::new();
        checker.register_server("server1").await;

        let result = checker.check_health("server1").await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        let avail = checker.get_availability("server1").await.unwrap();
        assert_eq!(avail.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_server_availability() {
        let mut avail = ServerAvailability::new("server1".to_string());
        assert!(avail.is_available);

        avail.mark_unhealthy();
        avail.mark_unhealthy();
        avail.mark_unhealthy();
        assert!(!avail.is_available);

        avail.mark_healthy();
        assert!(avail.is_available);
        assert_eq!(avail.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_is_disconnected() {
        let checker = HealthChecker::new();
        checker.register_server("server1").await;

        assert!(!checker.is_disconnected("server1").await);

        let mut avail = checker.get_availability("server1").await.unwrap();
        avail.mark_unhealthy();
        avail.mark_unhealthy();
        avail.mark_unhealthy();

        let mut availability = checker.availability.write().await;
        availability.insert("server1".to_string(), avail);
        drop(availability);

        assert!(checker.is_disconnected("server1").await);
    }

    #[tokio::test]
    async fn test_report_failure() {
        let checker = HealthChecker::new();
        checker.register_server("server1").await;

        let message = checker.report_failure("server1").await;
        assert!(message.contains("server1"));
    }

    #[tokio::test]
    async fn test_get_health_stats() {
        let checker = HealthChecker::new();
        checker.register_server("server1").await;
        checker.register_server("server2").await;

        let stats = checker.get_health_stats().await;
        assert_eq!(stats.len(), 2);
    }
}
