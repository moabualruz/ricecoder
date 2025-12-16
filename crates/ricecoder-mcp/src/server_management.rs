//! MCP Server Management
//!
//! Server discovery, registration, health monitoring, and lifecycle management
//! for MCP servers and their associated tools.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, mpsc};
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::metadata::ToolMetadata;
use crate::transport::{MCPTransport, TransportConfig, TransportFactory};

/// Server connection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerState {
    Disconnected,
    Connecting,
    Connected,
    Error,
    Disabled,
}

/// Server health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub state: ServerState,
    pub last_seen: Option<SystemTime>,
    pub last_error: Option<String>,
    pub connection_attempts: u32,
    pub uptime_seconds: u64,
    pub tools_available: usize,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub transport_config: TransportConfig,
    pub auto_start: bool,
    pub health_check_interval_seconds: u64,
    pub max_reconnect_attempts: u32,
    pub auth_config: Option<AuthConfig>,
    pub enabled_tools: HashSet<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub credentials: HashMap<String, String>,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    ApiKey,
    Custom(String),
}

/// Server registration information
#[derive(Clone)]
pub struct ServerRegistration {
    pub config: ServerConfig,
    pub transport: Option<Arc<Box<dyn MCPTransport>>>,
    pub health: ServerHealth,
    pub tools: Vec<ToolMetadata>,
    pub registered_at: SystemTime,
}

/// Server discovery result
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub server_id: String,
    pub server_name: String,
    pub tools: Vec<ToolMetadata>,
    pub capabilities: HashMap<String, serde_json::Value>,
}

/// Server manager for MCP server lifecycle management
pub struct ServerManager {
    servers: Arc<RwLock<HashMap<String, ServerRegistration>>>,
    discovery_providers: Vec<Box<dyn ServerDiscoveryProvider>>,
    health_monitor: Arc<HealthMonitor>,
    audit_logger: Option<Arc<crate::audit::MCPAuditLogger>>,
    rbac_manager: Option<Arc<crate::rbac::MCRBACManager>>,
    compliance_monitor: Option<Arc<crate::compliance::MCPComplianceMonitor>>,
    _health_task: tokio::task::JoinHandle<()>,
}

impl ServerManager {
    /// Create a new server manager
    pub fn new() -> Self {
        let servers = Arc::new(RwLock::new(HashMap::new()));
        let health_monitor = Arc::new(HealthMonitor::new(servers.clone()));

        let health_monitor_clone = health_monitor.clone();
        let health_task = tokio::spawn(async move {
            health_monitor_clone.run().await;
        });

        Self {
            servers,
            discovery_providers: Vec::new(),
            health_monitor,
            audit_logger: None,
            rbac_manager: None,
            compliance_monitor: None,
            _health_task: health_task,
        }
    }

    /// Set RBAC manager
    pub fn with_rbac_manager(mut self, rbac_manager: Arc<crate::rbac::MCRBACManager>) -> Self {
        self.rbac_manager = Some(rbac_manager);
        self
    }

    /// Set compliance monitor
    pub fn with_compliance_monitor(mut self, compliance_monitor: Arc<crate::compliance::MCPComplianceMonitor>) -> Self {
        self.compliance_monitor = Some(compliance_monitor);
        self
    }

    /// Create a new server manager with audit logging
    pub fn with_audit_logger(audit_logger: Arc<crate::audit::MCPAuditLogger>) -> Self {
        let servers = Arc::new(RwLock::new(HashMap::new()));
        let health_monitor = Arc::new(HealthMonitor::new(servers.clone()));

        let health_monitor_clone = health_monitor.clone();
        let health_task = tokio::spawn(async move {
            health_monitor_clone.run().await;
        });

        Self {
            servers,
            discovery_providers: Vec::new(),
            health_monitor,
            audit_logger: Some(audit_logger),
            rbac_manager: None,
            compliance_monitor: None,
            _health_task: health_task,
        }
    }

    /// Register a server with the manager
    pub async fn register_server(&self, config: ServerConfig) -> Result<()> {
        self.register_server_with_auth(config, None).await
    }

    /// Register a server with the manager and authorization check
    pub async fn register_server_with_auth(&self, config: ServerConfig, user_id: Option<&str>) -> Result<()> {
        // RBAC check for server registration
        if let (Some(ref rbac_manager), Some(user_id)) = (&self.rbac_manager, user_id) {
            // Create a basic principal for RBAC check
            let principal = ricecoder_security::access_control::Principal {
                id: user_id.to_string(),
                roles: vec![], // Would need to be populated from user context
                attributes: std::collections::HashMap::new(),
            };

            rbac_manager.check_server_access(&principal, &config.id)?;

            // Record compliance event
            if let Some(ref compliance_monitor) = self.compliance_monitor {
                let _ = compliance_monitor.record_violation(
                    crate::compliance::ComplianceReportType::Soc2Type2,
                    crate::compliance::ViolationSeverity::Low,
                    "Server registration performed".to_string(),
                    format!("mcp:server:{}", config.id),
                    Some(user_id.to_string()),
                    serde_json::json!({
                        "action": "server_registration",
                        "server_id": config.id,
                        "auto_start": config.auto_start
                    }),
                ).await;
            }
        }

        let registration = ServerRegistration {
            config: config.clone(),
            transport: None,
            health: ServerHealth {
                state: ServerState::Disconnected,
                last_seen: None,
                last_error: None,
                connection_attempts: 0,
                uptime_seconds: 0,
                tools_available: 0,
            },
            tools: Vec::new(),
            registered_at: SystemTime::now(),
        };

        let mut servers = self.servers.write().await;
        servers.insert(config.id.clone(), registration);

        info!("Registered MCP server: {} ({})", config.name, config.id);

        // Audit logging
        if let Some(ref audit_logger) = self.audit_logger {
            let _ = audit_logger.log_server_registration(&config, user_id.map(|s| s.to_string()), None).await;
        }

        if config.auto_start {
            drop(servers);
            self.start_server(&config.id).await?;
        }

        Ok(())
    }

    /// Unregister a server
    pub async fn unregister_server(&self, server_id: &str) -> Result<bool> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.remove(server_id) {
            if let Some(transport) = registration.transport {
                let _ = transport.close().await;
            }
            info!("Unregistered MCP server: {}", server_id);

            // Audit logging
            if let Some(ref audit_logger) = self.audit_logger {
                let _ = audit_logger.log_server_unregistration(server_id, None, None).await;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Start a server
    pub async fn start_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            if registration.health.state == ServerState::Connected {
                return Ok(());
            }

            registration.health.state = ServerState::Connecting;
            registration.health.connection_attempts += 1;

            // Create transport
            let transport_result = TransportFactory::create(&registration.config.transport_config);
            match transport_result {
                Ok(transport) => {
                    registration.transport = Some(Arc::new(transport));

                    // TODO: Initialize connection and discover tools
                    // For now, mark as connected
                    registration.health.state = ServerState::Connected;
                    registration.health.last_seen = Some(SystemTime::now());

                    info!("Started MCP server: {}", server_id);

                    // Audit logging
                    if let Some(ref audit_logger) = self.audit_logger {
                        let _ = audit_logger.log_server_connection(server_id, true, None, None, None).await;
                    }

                    Ok(())
                }
                Err(e) => {
                    registration.health.state = ServerState::Error;
                    registration.health.last_error = Some(e.to_string());

                    // Audit logging
                    if let Some(ref audit_logger) = self.audit_logger {
                        let _ = audit_logger.log_server_connection(server_id, false, Some(e.to_string()), None, None).await;
                    }

                    Err(e)
                }
            }
        } else {
            Err(Error::ServerError(format!("Server not found: {}", server_id)))
        }
    }

    /// Stop a server
    pub async fn stop_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            if let Some(transport) = registration.transport.take() {
                transport.as_ref().close().await?;
            }
            registration.health.state = ServerState::Disconnected;
            info!("Stopped MCP server: {}", server_id);

            // Audit logging
            if let Some(ref audit_logger) = self.audit_logger {
                let _ = audit_logger.log_server_disconnection(server_id, "manual_stop", None, None).await;
            }

            Ok(())
        } else {
            Err(Error::ServerError(format!("Server not found: {}", server_id)))
        }
    }

    /// Get server health status
    pub async fn get_server_health(&self, server_id: &str) -> Result<ServerHealth> {
        let servers = self.servers.read().await;
        if let Some(registration) = servers.get(server_id) {
            Ok(registration.health.clone())
        } else {
            Err(Error::ServerError(format!("Server not found: {}", server_id)))
        }
    }

    /// List all registered servers
    pub async fn list_servers(&self) -> Result<Vec<ServerRegistration>> {
        let servers = self.servers.read().await;
        Ok(servers.values().cloned().collect())
    }

    /// Enable a tool on a server
    pub async fn enable_tool(&self, server_id: &str, tool_name: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            registration.config.enabled_tools.insert(tool_name.to_string());
            info!("Enabled tool '{}' on server '{}'", tool_name, server_id);

            // Audit logging
            if let Some(ref audit_logger) = self.audit_logger {
                let _ = audit_logger.log_tool_enablement(server_id, tool_name, true, None, None).await;
            }

            Ok(())
        } else {
            Err(Error::ServerError(format!("Server not found: {}", server_id)))
        }
    }

    /// Disable a tool on a server
    pub async fn disable_tool(&self, server_id: &str, tool_name: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            registration.config.enabled_tools.remove(tool_name);
            info!("Disabled tool '{}' on server '{}'", tool_name, server_id);

            // Audit logging
            if let Some(ref audit_logger) = self.audit_logger {
                let _ = audit_logger.log_tool_enablement(server_id, tool_name, false, None, None).await;
            }

            Ok(())
        } else {
            Err(Error::ServerError(format!("Server not found: {}", server_id)))
        }
    }

    /// Check if a tool is enabled on a server
    pub async fn is_tool_enabled(&self, server_id: &str, tool_name: &str) -> Result<bool> {
        let servers = self.servers.read().await;
        if let Some(registration) = servers.get(server_id) {
            Ok(registration.config.enabled_tools.contains(tool_name))
        } else {
            Err(Error::ServerError(format!("Server not found: {}", server_id)))
        }
    }

    /// Discover available servers
    pub async fn discover_servers(&self) -> Result<Vec<DiscoveryResult>> {
        let mut results = Vec::new();

        for provider in &self.discovery_providers {
            match provider.discover_servers().await {
                Ok(mut provider_results) => {
                    results.append(&mut provider_results);
                }
                Err(e) => {
                    warn!("Server discovery failed for provider: {}", e);
                }
            }
        }

        Ok(results)
    }

    /// Add a discovery provider
    pub fn add_discovery_provider(&mut self, provider: Box<dyn ServerDiscoveryProvider>) {
        self.discovery_providers.push(provider);
    }

    /// Get server registration info
    pub async fn get_server(&self, server_id: &str) -> Result<Option<ServerRegistration>> {
        let servers = self.servers.read().await;
        Ok(servers.get(server_id).cloned())
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Server discovery provider trait
#[async_trait]
pub trait ServerDiscoveryProvider: Send + Sync {
    /// Discover available MCP servers
    async fn discover_servers(&self) -> Result<Vec<DiscoveryResult>>;
}

/// Health monitor for server connections
struct HealthMonitor {
    servers: Arc<RwLock<HashMap<String, ServerRegistration>>>,
}

impl HealthMonitor {
    fn new(servers: Arc<RwLock<HashMap<String, ServerRegistration>>>) -> Self {
        Self { servers }
    }

    async fn run(&self) {
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;
            self.check_server_health().await;
        }
    }

    async fn check_server_health(&self) {
        let server_ids: Vec<String> = {
            let servers = self.servers.read().await;
            servers.keys().cloned().collect()
        };

        for server_id in server_ids {
            if let Err(e) = self.check_single_server(&server_id).await {
                error!("Health check failed for server {}: {}", server_id, e);
            }
        }
    }

    async fn check_single_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            let is_connected = if let Some(ref transport) = registration.transport {
                transport.as_ref().is_connected().await
            } else {
                false
            };

            if is_connected {
                registration.health.state = ServerState::Connected;
                registration.health.last_seen = Some(SystemTime::now());

                if let Some(last_seen) = registration.health.last_seen {
                    if let Ok(duration) = SystemTime::now().duration_since(last_seen) {
                        registration.health.uptime_seconds = duration.as_secs();
                    }
                }
            } else {
                registration.health.state = ServerState::Disconnected;
                registration.health.last_error = Some("Connection lost".to_string());

                // TODO: Implement reconnection logic
                // For now, just mark as disconnected
            }

            registration.health.tools_available = registration.tools.len();
        }

        Ok(())
    }
}

/// Built-in file system discovery provider
pub struct FileSystemDiscoveryProvider {
    search_paths: Vec<std::path::PathBuf>,
}

impl FileSystemDiscoveryProvider {
    pub fn new(search_paths: Vec<std::path::PathBuf>) -> Self {
        Self { search_paths }
    }

    pub fn with_default_paths() -> Self {
        Self {
            search_paths: vec![
                std::path::PathBuf::from("./mcp-servers"),
                std::path::PathBuf::from("./servers"),
                std::path::PathBuf::from("~/.mcp/servers"),
            ],
        }
    }
}

#[async_trait]
impl ServerDiscoveryProvider for FileSystemDiscoveryProvider {
    async fn discover_servers(&self) -> Result<Vec<DiscoveryResult>> {
        let mut results = Vec::new();

        for path in &self.search_paths {
            if let Ok(entries) = tokio::fs::read_dir(path).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(file_type) = entry.file_type().await {
                        if file_type.is_file() {
                            if let Some(extension) = entry.path().extension() {
                                if extension == "json" || extension == "yaml" || extension == "yml" {
                                    // TODO: Parse server configuration files
                                    // For now, create a dummy result
                                    let server_id = format!("file_{}", entry.file_name().to_string_lossy());
                                    results.push(DiscoveryResult {
                                        server_id,
                                        server_name: entry.file_name().to_string_lossy().to_string(),
                                        tools: Vec::new(),
                                        capabilities: HashMap::new(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{StdioConfig, TransportType};

    #[tokio::test]
    async fn test_server_registration() {
        let manager = ServerManager::new();

        let config = ServerConfig {
            id: "test_server".to_string(),
            name: "Test Server".to_string(),
            description: "A test MCP server".to_string(),
            transport_config: TransportConfig {
                transport_type: TransportType::Stdio,
                stdio_config: Some(StdioConfig {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                }),
                http_config: None,
                sse_config: None,
            },
            auto_start: false,
            health_check_interval_seconds: 30,
            max_reconnect_attempts: 3,
            auth_config: None,
            enabled_tools: HashSet::new(),
        };

        assert!(manager.register_server(config).await.is_ok());

        let servers = manager.list_servers().await.unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].config.id, "test_server");
    }

    #[tokio::test]
    async fn test_tool_enablement() {
        let manager = ServerManager::new();

        let config = ServerConfig {
            id: "test_server".to_string(),
            name: "Test Server".to_string(),
            description: "A test MCP server".to_string(),
            transport_config: TransportConfig {
                transport_type: TransportType::Stdio,
                stdio_config: Some(StdioConfig {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                }),
                http_config: None,
                sse_config: None,
            },
            auto_start: false,
            health_check_interval_seconds: 30,
            max_reconnect_attempts: 3,
            auth_config: None,
            enabled_tools: HashSet::new(),
        };

        manager.register_server(config).await.unwrap();

        // Enable a tool
        assert!(manager.enable_tool("test_server", "grep").await.is_ok());
        assert!(manager.is_tool_enabled("test_server", "grep").await.unwrap());

        // Disable the tool
        assert!(manager.disable_tool("test_server", "grep").await.is_ok());
        assert!(!manager.is_tool_enabled("test_server", "grep").await.unwrap());
    }

    #[test]
    fn test_server_config_serialization() {
        let config = ServerConfig {
            id: "test_server".to_string(),
            name: "Test Server".to_string(),
            description: "A test MCP server".to_string(),
            transport_config: TransportConfig {
                transport_type: TransportType::Stdio,
                stdio_config: Some(StdioConfig {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                }),
                http_config: None,
                sse_config: None,
            },
            auto_start: true,
            health_check_interval_seconds: 30,
            max_reconnect_attempts: 3,
            auth_config: None,
            enabled_tools: HashSet::from(["grep".to_string(), "find".to_string()]),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.id, deserialized.id);
        assert_eq!(config.enabled_tools, deserialized.enabled_tools);
    }
}