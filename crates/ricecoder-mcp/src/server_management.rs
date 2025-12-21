//! MCP Server Management
//!
//! Server discovery, registration, health monitoring, and lifecycle management
//! for MCP servers and their associated tools.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::metadata::ToolMetadata;
use crate::transport::{MCPMessage, MCPRequest, MCPTransport, TransportConfig, TransportFactory};

/// Server connection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerState {
    Disconnected,
    Connecting,
    Connected,
    Error,
    Disabled,
    Starting,
    Stopped,
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
    pub transport: Option<Arc<dyn MCPTransport>>,
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
    analytics: Option<Arc<crate::analytics::MCPAnalyticsAggregator>>,
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
            analytics: None,
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
    pub fn with_compliance_monitor(
        mut self,
        compliance_monitor: Arc<crate::compliance::MCPComplianceMonitor>,
    ) -> Self {
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
            analytics: None,
            rbac_manager: None,
            compliance_monitor: None,
            _health_task: health_task,
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

    /// Register a server with the manager
    pub async fn register_server(&self, config: ServerConfig) -> Result<()> {
        self.register_server_with_auth(config, None).await
    }

    /// Register a server with the manager and authorization check
    pub async fn register_server_with_auth(
        &self,
        config: ServerConfig,
        user_id: Option<&str>,
    ) -> Result<()> {
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
                let _ = compliance_monitor
                    .record_violation(
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
                    )
                    .await;
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
            let _ = audit_logger
                .log_server_registration(&config, user_id.map(|s| s.to_string()), None)
                .await;
        }

        if config.auto_start {
            drop(servers);
            self.start_server(&config.id).await?;
        }

        Ok(())
    }

    /// Attempt to reconnect to a server
    pub async fn attempt_reconnection(&self, server_id: &str, config: &ServerConfig) -> Result<()> {
        debug!("Attempting reconnection to server: {}", server_id);

        // Create new transport
        let transport_result = TransportFactory::create(&config.transport_config);

        match transport_result {
            Ok(transport) => {
                let transport_clone = transport.clone();
                let mut servers = self.servers.write().await;
                if let Some(registration) = servers.get_mut(server_id) {
                    registration.transport = Some(transport);

                    // Try to discover tools again
                    match self
                        .discover_tools_from_server(config, &*transport_clone)
                        .await
                    {
                        Ok(tools) => {
                            registration.tools = tools.clone();
                            registration.health.tools_available = tools.len();
                            registration.health.state = ServerState::Connected;
                            registration.health.last_seen = Some(SystemTime::now());
                            registration.health.connection_attempts += 1;

                            info!(
                                "Successfully reconnected to server: {} with {} tools",
                                server_id,
                                tools.len()
                            );

                            // Audit logging
                            if let Some(ref audit_logger) = self.audit_logger {
                                let _ = audit_logger
                                    .log_server_connection(server_id, true, None, None, None)
                                    .await;
                            }

                            Ok(())
                        }
                        Err(e) => {
                            registration.health.state = ServerState::Error;
                            registration.health.last_error =
                                Some(format!("Tool discovery failed after reconnection: {}", e));
                            Err(e)
                        }
                    }
                } else {
                    Err(Error::ServerError(format!(
                        "Server {} not found during reconnection",
                        server_id
                    )))
                }
            }
            Err(e) => {
                let mut servers = self.servers.write().await;
                if let Some(registration) = servers.get_mut(server_id) {
                    registration.health.connection_attempts += 1;
                    registration.health.last_error =
                        Some(format!("Transport creation failed: {}", e));
                }
                Err(e)
            }
        }
    }

    /// Discover tools from a server after connection
    async fn discover_tools_from_server(
        &self,
        config: &ServerConfig,
        transport: &dyn MCPTransport,
    ) -> Result<Vec<ToolMetadata>> {
        debug!("Discovering tools from server: {}", config.id);

        // Send a tools/list request
        let request = MCPMessage::Request(MCPRequest {
            id: format!("discover-{}", config.id),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        });

        match transport.send(&request).await {
            Ok(_) => {
                // In a real implementation, we'd wait for the response
                // For now, return some mock tools based on the server config
                let tools = match config.id.as_str() {
                    "filesystem" => vec![
                        ToolMetadata {
                            id: "read_file".to_string(),
                            name: "Read File".to_string(),
                            description: "Read the contents of a file".to_string(),
                            category: "filesystem".to_string(),
                            parameters: vec![crate::metadata::ParameterMetadata {
                                name: "path".to_string(),
                                type_: "string".to_string(),
                                description: "Path to the file to read".to_string(),
                                required: true,
                                default: None,
                            }],
                            return_type: "string".to_string(),
                            source: crate::metadata::ToolSource::Mcp(config.id.clone()),
                            server_id: Some(config.id.clone()),
                        },
                        ToolMetadata {
                            id: "list_dir".to_string(),
                            name: "List Directory".to_string(),
                            description: "List contents of a directory".to_string(),
                            category: "filesystem".to_string(),
                            parameters: vec![crate::metadata::ParameterMetadata {
                                name: "path".to_string(),
                                type_: "string".to_string(),
                                description: "Path to the directory".to_string(),
                                required: true,
                                default: None,
                            }],
                            return_type: "array".to_string(),
                            source: crate::metadata::ToolSource::Mcp(config.id.clone()),
                            server_id: Some(config.id.clone()),
                        },
                    ],
                    "git" => vec![ToolMetadata {
                        id: "git_status".to_string(),
                        name: "Git Status".to_string(),
                        description: "Get the status of a git repository".to_string(),
                        category: "git".to_string(),
                        parameters: vec![crate::metadata::ParameterMetadata {
                            name: "repo_path".to_string(),
                            type_: "string".to_string(),
                            description: "Path to the git repository".to_string(),
                            required: true,
                            default: None,
                        }],
                        return_type: "object".to_string(),
                        source: crate::metadata::ToolSource::Mcp(config.id.clone()),
                        server_id: Some(config.id.clone()),
                    }],
                    _ => vec![ToolMetadata {
                        id: format!("{}_tool", config.id),
                        name: format!("{} Tool", config.name),
                        description: format!("A tool from {}", config.name),
                        category: "general".to_string(),
                        parameters: vec![],
                        return_type: "string".to_string(),
                        source: crate::metadata::ToolSource::Mcp(config.id.clone()),
                        server_id: Some(config.id.clone()),
                    }],
                };

                Ok(tools)
            }
            Err(e) => Err(Error::ConnectionError(format!(
                "Failed to send tools/list request: {}",
                e
            ))),
        }
    }

    /// Discover available servers using all registered discovery providers
    pub async fn discover_servers(&self) -> Result<Vec<DiscoveryResult>> {
        let mut all_results = Vec::new();

        for provider in &self.discovery_providers {
            match provider.discover_servers().await {
                Ok(results) => all_results.extend(results),
                Err(e) => {
                    warn!("Server discovery provider failed: {}", e);
                    // Continue with other providers
                }
            }
        }

        Ok(all_results)
    }

    /// Start a server by ID
    pub async fn start_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            if registration.health.state == ServerState::Stopped
                || registration.health.state == ServerState::Disconnected
            {
                // In a real implementation, this would start the transport
                // For now, just set the state
                registration.health.state = ServerState::Starting;
                info!("Starting server: {}", server_id);
                // Simulate starting
                registration.health.state = ServerState::Connected;
                registration.health.last_seen = Some(SystemTime::now());
            }
        } else {
            return Err(Error::ServerNotFound(server_id.to_string()));
        }
        Ok(())
    }

    /// Stop a server by ID
    pub async fn stop_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            if registration.health.state == ServerState::Connected
                || registration.health.state == ServerState::Connecting
            {
                registration.health.state = ServerState::Stopped;
                registration.transport = None;
                info!("Stopped server: {}", server_id);

                // Audit logging
                if let Some(ref audit_logger) = self.audit_logger {
                    let _ = audit_logger
                        .log_server_connection(server_id, false, None, None, None)
                        .await;
                }
            }
        } else {
            return Err(Error::ServerNotFound(server_id.to_string()));
        }
        Ok(())
    }

    /// Restart a server by ID
    pub async fn restart_server(&self, server_id: &str) -> Result<()> {
        self.stop_server(server_id).await?;
        self.start_server(server_id).await
    }

    /// List all registered servers
    pub async fn list_servers(&self) -> Result<Vec<ServerRegistration>> {
        let servers = self.servers.read().await;
        Ok(servers.values().cloned().collect())
    }

    /// Get server registration by ID
    pub async fn get_server(&self, server_id: &str) -> Result<ServerRegistration> {
        let servers = self.servers.read().await;
        servers
            .get(server_id)
            .cloned()
            .ok_or_else(|| Error::ServerNotFound(server_id.to_string()))
    }

    /// Enable a tool for a server
    pub async fn enable_tool(&self, server_id: &str, tool_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            registration
                .config
                .enabled_tools
                .insert(tool_id.to_string());
            info!("Enabled tool {} for server {}", tool_id, server_id);
            Ok(())
        } else {
            Err(Error::ServerNotFound(server_id.to_string()))
        }
    }

    /// Disable a tool for a server
    pub async fn disable_tool(&self, server_id: &str, tool_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(registration) = servers.get_mut(server_id) {
            registration.config.enabled_tools.remove(tool_id);
            info!("Disabled tool {} for server {}", tool_id, server_id);
            Ok(())
        } else {
            Err(Error::ServerNotFound(server_id.to_string()))
        }
    }

    /// Check if a tool is enabled for a server
    pub async fn is_tool_enabled(&self, server_id: &str, tool_id: &str) -> Result<bool> {
        let servers = self.servers.read().await;
        if let Some(registration) = servers.get(server_id) {
            Ok(registration.config.enabled_tools.contains(tool_id))
        } else {
            Err(Error::ServerNotFound(server_id.to_string()))
        }
    }

    /// Unregister a server
    pub async fn unregister_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if servers.remove(server_id).is_some() {
            info!("Unregistered server: {}", server_id);

            // Audit logging
            if let Some(ref audit_logger) = self.audit_logger {
                let _ = audit_logger
                    .log_server_unregistration(server_id, None, None)
                    .await;
            }

            Ok(())
        } else {
            Err(Error::ServerNotFound(server_id.to_string()))
        }
    }

    /// Add a discovery provider
    pub fn add_discovery_provider(&mut self, provider: Box<dyn ServerDiscoveryProvider>) {
        self.discovery_providers.push(provider);
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

    /// Parse a server configuration file
    async fn parse_server_config(&self, path: &std::path::Path) -> Result<ServerConfig>;
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

                // Implement reconnection logic
                if registration.health.connection_attempts
                    < registration.config.max_reconnect_attempts
                {
                    warn!(
                        "Server {} disconnected, attempting reconnection (attempt {}/{})",
                        server_id,
                        registration.health.connection_attempts + 1,
                        registration.config.max_reconnect_attempts
                    );

                    // Try to reconnect
                    if let Err(reconnect_err) = self
                        .attempt_reconnection(server_id, &registration.config)
                        .await
                    {
                        error!(
                            "Reconnection failed for server {}: {}",
                            server_id, reconnect_err
                        );
                        registration.health.last_error =
                            Some(format!("Reconnection failed: {}", reconnect_err));
                    }
                } else {
                    error!("Max reconnection attempts reached for server {}", server_id);
                    registration.health.state = ServerState::Error;
                }
            }

            registration.health.tools_available = registration.tools.len();
        }

        Ok(())
    }

    async fn attempt_reconnection(&self, server_id: &str, config: &ServerConfig) -> Result<()> {
        debug!("Attempting reconnection to server: {}", server_id);

        // Create new transport
        let transport_result = crate::transport::TransportFactory::create(&config.transport_config);

        match transport_result {
            Ok(transport) => {
                let mut servers = self.servers.write().await;
                if let Some(registration) = servers.get_mut(server_id) {
                    registration.transport = Some(transport);
                    registration.health.state = ServerState::Connected;
                    registration.health.last_seen = Some(SystemTime::now());
                    registration.health.connection_attempts += 1;
                    registration.health.last_error = None;

                    info!("Successfully reconnected to server: {}", server_id);
                    Ok(())
                } else {
                    Err(crate::error::Error::ServerError(format!(
                        "Server {} not found during reconnection",
                        server_id
                    )))
                }
            }
            Err(e) => {
                let mut servers = self.servers.write().await;
                if let Some(registration) = servers.get_mut(server_id) {
                    registration.health.connection_attempts += 1;
                    registration.health.last_error =
                        Some(format!("Transport creation failed: {}", e));
                }
                Err(e)
            }
        }
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
    async fn parse_server_config(&self, path: &std::path::Path) -> Result<ServerConfig> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| Error::IoError(e))?;

        let extension = path.extension().unwrap_or_default();

        let config: ServerConfig = if extension == "json" {
            serde_json::from_str(&content).map_err(|e| Error::SerializationError(e))?
        } else if extension == "yaml" || extension == "yml" {
            serde_yaml::from_str(&content)
                .map_err(|e| Error::ConfigError(format!("YAML parsing error: {}", e)))?
        } else {
            return Err(Error::ValidationError(
                "Unsupported config file format".to_string(),
            ));
        };

        Ok(config)
    }

    async fn discover_servers(&self) -> Result<Vec<DiscoveryResult>> {
        let mut results = Vec::new();

        for path in &self.search_paths {
            if let Ok(entries) = tokio::fs::read_dir(path).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(file_type) = entry.file_type().await {
                        if file_type.is_file() {
                            if let Some(extension) = entry.path().extension() {
                                if extension == "json" || extension == "yaml" || extension == "yml"
                                {
                                    // Parse server configuration files
                                    match <FileSystemDiscoveryProvider as ServerDiscoveryProvider>::parse_server_config(self, &entry.path()).await {
                                        Ok(config) => {
                                            results.push(DiscoveryResult {
                                                server_id: config.id,
                                                server_name: config.name,
                                                tools: Vec::new(), // Tools will be discovered when connecting
                                                capabilities: HashMap::new(),
                                            });
                                        }
                                        Err(e) => {
                                            warn!("Failed to parse server config {}: {}", entry.path().display(), e);
                                        }
                                    }
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
        assert!(manager
            .is_tool_enabled("test_server", "grep")
            .await
            .unwrap());

        // Disable the tool
        assert!(manager.disable_tool("test_server", "grep").await.is_ok());
        assert!(!manager
            .is_tool_enabled("test_server", "grep")
            .await
            .unwrap());
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
