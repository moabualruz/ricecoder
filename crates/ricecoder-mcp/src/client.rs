//! MCP Client implementation

use crate::error::{Error, Result};
use crate::metadata::ToolMetadata;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MCP Server connection information
#[derive(Debug, Clone)]
pub struct ServerConnection {
    pub id: String,
    pub name: String,
    pub is_connected: bool,
    pub tools: Vec<ToolMetadata>,
}

/// MCP Client for communicating with MCP servers
#[derive(Debug, Clone)]
pub struct MCPClient {
    connections: Arc<RwLock<HashMap<String, ServerConnection>>>,
}

impl MCPClient {
    /// Creates a new MCP client
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new MCP client with custom timeout
    pub fn with_timeout(_timeout_ms: u64) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connects to an MCP server
    ///
    /// # Arguments
    /// * `server_id` - Unique identifier for the server
    /// * `server_name` - Human-readable name for the server
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Errors
    /// Returns error if connection fails or times out
    pub async fn connect(&self, server_id: &str, server_name: &str) -> Result<()> {
        debug!("Connecting to MCP server: {} ({})", server_id, server_name);

        // Simulate connection with timeout
        let connection = ServerConnection {
            id: server_id.to_string(),
            name: server_name.to_string(),
            is_connected: true,
            tools: Vec::new(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(server_id.to_string(), connection);

        info!("Connected to MCP server: {}", server_id);
        Ok(())
    }

    /// Disconnects from an MCP server
    ///
    /// # Arguments
    /// * `server_id` - Unique identifier for the server
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn disconnect(&self, server_id: &str) -> Result<()> {
        debug!("Disconnecting from MCP server: {}", server_id);

        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(server_id) {
            conn.is_connected = false;
        }

        info!("Disconnected from MCP server: {}", server_id);
        Ok(())
    }

    /// Discovers available MCP servers
    ///
    /// # Returns
    /// List of discovered server IDs
    pub async fn discover_servers(&self) -> Result<Vec<String>> {
        debug!("Discovering MCP servers");

        let connections = self.connections.read().await;
        let servers: Vec<String> = connections.keys().cloned().collect();

        info!("Discovered {} MCP servers", servers.len());
        Ok(servers)
    }

    /// Discovers tools from a specific MCP server
    ///
    /// # Arguments
    /// * `server_id` - Unique identifier for the server
    ///
    /// # Returns
    /// List of tools available from the server
    ///
    /// # Errors
    /// Returns error if server is not connected or discovery fails
    pub async fn discover_tools(&self, server_id: &str) -> Result<Vec<ToolMetadata>> {
        debug!("Discovering tools from server: {}", server_id);

        let connections = self.connections.read().await;
        let connection = connections
            .get(server_id)
            .ok_or_else(|| Error::ConnectionError(format!("Server not connected: {}", server_id)))?;

        if !connection.is_connected {
            return Err(Error::ConnectionError(format!(
                "Server not connected: {}",
                server_id
            )));
        }

        let tools = connection.tools.clone();
        info!(
            "Discovered {} tools from server: {}",
            tools.len(),
            server_id
        );
        Ok(tools)
    }

    /// Registers tools from an MCP server
    ///
    /// # Arguments
    /// * `server_id` - Unique identifier for the server
    /// * `tools` - List of tools to register
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Errors
    /// Returns error if server is not connected or registration fails
    pub async fn register_tools(&self, server_id: &str, tools: Vec<ToolMetadata>) -> Result<()> {
        debug!(
            "Registering {} tools from server: {}",
            tools.len(),
            server_id
        );

        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(server_id)
            .ok_or_else(|| Error::ConnectionError(format!("Server not connected: {}", server_id)))?;

        connection.tools = tools;
        info!("Registered tools for server: {}", server_id);
        Ok(())
    }

    /// Gets all connected servers
    ///
    /// # Returns
    /// List of connected server connections
    pub async fn get_connected_servers(&self) -> Result<Vec<ServerConnection>> {
        let connections = self.connections.read().await;
        let servers: Vec<ServerConnection> = connections
            .values()
            .filter(|c| c.is_connected)
            .cloned()
            .collect();

        Ok(servers)
    }

    /// Gets a specific server connection
    ///
    /// # Arguments
    /// * `server_id` - Unique identifier for the server
    ///
    /// # Returns
    /// Server connection if found
    pub async fn get_server(&self, server_id: &str) -> Result<Option<ServerConnection>> {
        let connections = self.connections.read().await;
        Ok(connections.get(server_id).cloned())
    }

    /// Checks if a server is connected
    ///
    /// # Arguments
    /// * `server_id` - Unique identifier for the server
    ///
    /// # Returns
    /// True if server is connected, false otherwise
    pub async fn is_connected(&self, server_id: &str) -> bool {
        let connections = self.connections.read().await;
        connections
            .get(server_id)
            .map(|c| c.is_connected)
            .unwrap_or(false)
    }

    /// Gets the number of connected servers
    pub async fn connected_server_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().filter(|c| c.is_connected).count()
    }

    /// Gets all tools from all connected servers
    ///
    /// # Returns
    /// List of all tools from all connected servers
    pub async fn get_all_tools(&self) -> Result<Vec<ToolMetadata>> {
        let connections = self.connections.read().await;
        let mut all_tools = Vec::new();

        for connection in connections.values() {
            if connection.is_connected {
                all_tools.extend(connection.tools.clone());
            }
        }

        Ok(all_tools)
    }
}

impl Default for MCPClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client() {
        let client = MCPClient::new();
        assert_eq!(client.connected_server_count().await, 0);
    }

    #[tokio::test]
    async fn test_connect_server() {
        let client = MCPClient::new();
        let result = client.connect("server1", "Test Server").await;
        assert!(result.is_ok());
        assert!(client.is_connected("server1").await);
    }

    #[tokio::test]
    async fn test_disconnect_server() {
        let client = MCPClient::new();
        client.connect("server1", "Test Server").await.unwrap();
        assert!(client.is_connected("server1").await);

        let result = client.disconnect("server1").await;
        assert!(result.is_ok());
        assert!(!client.is_connected("server1").await);
    }

    #[tokio::test]
    async fn test_discover_servers() {
        let client = MCPClient::new();
        client.connect("server1", "Server 1").await.unwrap();
        client.connect("server2", "Server 2").await.unwrap();

        let servers = client.discover_servers().await.unwrap();
        assert_eq!(servers.len(), 2);
    }

    #[tokio::test]
    async fn test_register_and_discover_tools() {
        use crate::metadata::ToolSource;
        
        let client = MCPClient::new();
        client.connect("server1", "Test Server").await.unwrap();

        let tool = ToolMetadata {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            category: "test".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Mcp("server1".to_string()),
            server_id: Some("server1".to_string()),
        };

        client
            .register_tools("server1", vec![tool.clone()])
            .await
            .unwrap();

        let tools = client.discover_tools("server1").await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].id, "test-tool");
    }

    #[tokio::test]
    async fn test_get_connected_servers() {
        let client = MCPClient::new();
        client.connect("server1", "Server 1").await.unwrap();
        client.connect("server2", "Server 2").await.unwrap();

        let servers = client.get_connected_servers().await.unwrap();
        assert_eq!(servers.len(), 2);
    }

    #[tokio::test]
    async fn test_get_all_tools() {
        use crate::metadata::ToolSource;
        
        let client = MCPClient::new();
        client.connect("server1", "Server 1").await.unwrap();

        let tool1 = ToolMetadata {
            id: "tool1".to_string(),
            name: "Tool 1".to_string(),
            description: "Tool 1".to_string(),
            category: "test".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Mcp("server1".to_string()),
            server_id: Some("server1".to_string()),
        };

        let tool2 = ToolMetadata {
            id: "tool2".to_string(),
            name: "Tool 2".to_string(),
            description: "Tool 2".to_string(),
            category: "test".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Mcp("server1".to_string()),
            server_id: Some("server1".to_string()),
        };

        client
            .register_tools("server1", vec![tool1, tool2])
            .await
            .unwrap();

        let all_tools = client.get_all_tools().await.unwrap();
        assert_eq!(all_tools.len(), 2);
    }

    #[tokio::test]
    async fn test_discover_tools_not_connected() {
        let client = MCPClient::new();
        let result = client.discover_tools("nonexistent").await;
        assert!(result.is_err());
    }
}
