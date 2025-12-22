//! Server lifecycle management for MCP servers

use std::{sync::Arc, time::Duration};

use tokio::{sync::RwLock, time::timeout};
use tracing::{debug, error, info};

use crate::{
    config::MCPServerConfig,
    error::{Error, Result},
    health_check::HealthChecker,
};

/// Server lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

/// Server lifecycle information
#[derive(Debug, Clone)]
pub struct ServerLifecycleInfo {
    pub server_id: String,
    pub state: ServerState,
    pub started_at: Option<std::time::Instant>,
    pub stopped_at: Option<std::time::Instant>,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

impl ServerLifecycleInfo {
    /// Creates a new server lifecycle info
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            state: ServerState::Stopped,
            started_at: None,
            stopped_at: None,
            restart_count: 0,
            last_error: None,
        }
    }

    /// Gets the uptime in milliseconds
    pub fn uptime_ms(&self) -> Option<u128> {
        self.started_at.map(|start| start.elapsed().as_millis())
    }

    /// Checks if the server is running
    pub fn is_running(&self) -> bool {
        self.state == ServerState::Running
    }

    /// Checks if the server has failed
    pub fn has_failed(&self) -> bool {
        self.state == ServerState::Failed
    }
}

/// Server lifecycle manager
#[derive(Debug, Clone)]
pub struct ServerLifecycle {
    config: Arc<MCPServerConfig>,
    health_checker: Arc<HealthChecker>,
    lifecycle_info: Arc<RwLock<ServerLifecycleInfo>>,
}

impl ServerLifecycle {
    /// Creates a new server lifecycle manager
    pub fn new(config: MCPServerConfig, health_checker: Arc<HealthChecker>) -> Self {
        Self {
            config: Arc::new(config.clone()),
            health_checker,
            lifecycle_info: Arc::new(RwLock::new(ServerLifecycleInfo::new(config.id.clone()))),
        }
    }

    /// Starts the server with timeout handling
    ///
    /// # Arguments
    /// * `startup_timeout_ms` - Timeout for server startup in milliseconds
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn start(&self, startup_timeout_ms: Option<u64>) -> Result<()> {
        let mut info = self.lifecycle_info.write().await;

        if info.state == ServerState::Running {
            debug!("Server {} is already running", self.config.id);
            return Ok(());
        }

        info.state = ServerState::Starting;
        drop(info);

        debug!("Starting server: {}", self.config.id);

        let timeout_duration =
            Duration::from_millis(startup_timeout_ms.unwrap_or(self.config.timeout_ms));

        match timeout(timeout_duration, self.perform_startup()).await {
            Ok(Ok(())) => {
                let mut info = self.lifecycle_info.write().await;
                info.state = ServerState::Running;
                info.started_at = Some(std::time::Instant::now());
                info.last_error = None;

                self.health_checker.register_server(&self.config.id).await;

                info!("Server started successfully: {}", self.config.id);
                Ok(())
            }
            Ok(Err(e)) => {
                let mut info = self.lifecycle_info.write().await;
                info.state = ServerState::Failed;
                info.last_error = Some(e.to_string());

                error!("Server startup failed: {}: {}", self.config.id, e);
                Err(e)
            }
            Err(_) => {
                let mut info = self.lifecycle_info.write().await;
                info.state = ServerState::Failed;
                let error_msg = format!(
                    "Server startup timeout after {}ms",
                    timeout_duration.as_millis()
                );
                info.last_error = Some(error_msg.clone());

                error!("Server startup timeout: {}", self.config.id);
                Err(Error::TimeoutError(timeout_duration.as_millis() as u64))
            }
        }
    }

    /// Performs the actual server startup
    async fn perform_startup(&self) -> Result<()> {
        // In a real implementation, this would spawn the server process
        // For now, we simulate successful startup
        debug!("Performing startup for server: {}", self.config.id);
        Ok(())
    }

    /// Shuts down the server and performs cleanup
    pub async fn shutdown(&self) -> Result<()> {
        let mut info = self.lifecycle_info.write().await;

        if info.state == ServerState::Stopped {
            debug!("Server {} is already stopped", self.config.id);
            return Ok(());
        }

        info.state = ServerState::Stopping;
        drop(info);

        debug!("Shutting down server: {}", self.config.id);

        // Unregister from health checker
        self.health_checker.unregister_server(&self.config.id).await;

        // Perform cleanup
        self.perform_cleanup().await?;

        let mut info = self.lifecycle_info.write().await;
        info.state = ServerState::Stopped;
        info.stopped_at = Some(std::time::Instant::now());

        info!("Server shut down successfully: {}", self.config.id);
        Ok(())
    }

    /// Performs cleanup operations
    async fn perform_cleanup(&self) -> Result<()> {
        debug!("Performing cleanup for server: {}", self.config.id);
        // In a real implementation, this would clean up resources
        Ok(())
    }

    /// Performs health checking and availability detection
    pub async fn check_health(&self) -> Result<bool> {
        debug!("Checking health of server: {}", self.config.id);

        let info = self.lifecycle_info.read().await;
        if info.state != ServerState::Running {
            return Ok(false);
        }
        drop(info);

        self.health_checker.check_health(&self.config.id).await
    }

    /// Detects server disconnection
    pub async fn is_disconnected(&self) -> bool {
        self.health_checker.is_disconnected(&self.config.id).await
    }

    /// Implements reconnection with exponential backoff
    pub async fn reconnect(&self) -> Result<()> {
        debug!("Attempting to reconnect to server: {}", self.config.id);

        let mut info = self.lifecycle_info.write().await;
        info.restart_count += 1;
        drop(info);

        let server_id = self.config.id.clone();
        let config = self.config.clone();

        self.health_checker
            .reconnect_with_backoff(&server_id, || {
                let config = config.clone();
                Box::pin(async move {
                    debug!("Attempting reconnection to: {}", config.id);
                    Ok(())
                })
            })
            .await?;

        info!("Successfully reconnected to server: {}", self.config.id);
        Ok(())
    }

    /// Supports configurable max retries
    pub fn max_retries(&self) -> u32 {
        self.config.max_retries
    }

    /// Gets the current lifecycle state
    pub async fn get_state(&self) -> ServerState {
        self.lifecycle_info.read().await.state
    }

    /// Gets the lifecycle information
    pub async fn get_info(&self) -> ServerLifecycleInfo {
        self.lifecycle_info.read().await.clone()
    }

    /// Reports the last error
    pub async fn get_last_error(&self) -> Option<String> {
        self.lifecycle_info.read().await.last_error.clone()
    }

    /// Gets the restart count
    pub async fn get_restart_count(&self) -> u32 {
        self.lifecycle_info.read().await.restart_count
    }

    /// Gets the uptime in milliseconds
    pub async fn get_uptime_ms(&self) -> Option<u128> {
        self.lifecycle_info.read().await.uptime_ms()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn create_test_config(id: &str) -> MCPServerConfig {
        MCPServerConfig {
            id: id.to_string(),
            name: format!("Test Server {}", id),
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        }
    }

    #[tokio::test]
    async fn test_create_lifecycle() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        let info = lifecycle.get_info().await;
        assert_eq!(info.server_id, "server1");
        assert_eq!(info.state, ServerState::Stopped);
        assert_eq!(info.restart_count, 0);
    }

    #[tokio::test]
    async fn test_start_server() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        let result = lifecycle.start(Some(5000)).await;
        assert!(result.is_ok());

        let info = lifecycle.get_info().await;
        assert_eq!(info.state, ServerState::Running);
        assert!(info.started_at.is_some());
    }

    #[tokio::test]
    async fn test_shutdown_server() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        lifecycle.start(Some(5000)).await.unwrap();
        let result = lifecycle.shutdown().await;
        assert!(result.is_ok());

        let info = lifecycle.get_info().await;
        assert_eq!(info.state, ServerState::Stopped);
        assert!(info.stopped_at.is_some());
    }

    #[tokio::test]
    async fn test_server_uptime() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        lifecycle.start(Some(5000)).await.unwrap();

        let uptime = lifecycle.get_uptime_ms().await;
        assert!(uptime.is_some());
    }

    #[tokio::test]
    async fn test_restart_count() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        assert_eq!(lifecycle.get_restart_count().await, 0);

        lifecycle.start(Some(5000)).await.unwrap();
        lifecycle.reconnect().await.ok();

        let restart_count = lifecycle.get_restart_count().await;
        assert_eq!(restart_count, 1);
    }

    #[tokio::test]
    async fn test_max_retries() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        assert_eq!(lifecycle.max_retries(), 3);
    }

    #[tokio::test]
    async fn test_is_running() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        let info = lifecycle.get_info().await;
        assert!(!info.is_running());

        lifecycle.start(Some(5000)).await.unwrap();
        let info = lifecycle.get_info().await;
        assert!(info.is_running());
    }

    #[tokio::test]
    async fn test_lifecycle_info_uptime() {
        let mut info = ServerLifecycleInfo::new("server1".to_string());
        assert!(info.uptime_ms().is_none());

        info.started_at = Some(std::time::Instant::now());
        assert!(info.uptime_ms().is_some());
    }

    #[tokio::test]
    async fn test_double_start() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        lifecycle.start(Some(5000)).await.unwrap();
        let result = lifecycle.start(Some(5000)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_double_shutdown() {
        let config = create_test_config("server1");
        let health_checker = Arc::new(HealthChecker::new());
        let lifecycle = ServerLifecycle::new(config, health_checker);

        lifecycle.start(Some(5000)).await.unwrap();
        lifecycle.shutdown().await.unwrap();
        let result = lifecycle.shutdown().await;
        assert!(result.is_ok());
    }
}
