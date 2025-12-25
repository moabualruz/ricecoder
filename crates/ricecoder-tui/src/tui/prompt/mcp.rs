//! MCP (Model Context Protocol) integration for RiceCoder TUI
//!
//! Displays MCP server status and available tools:
//! - Server connection status
//! - Available tools list
//! - Tool execution hints
//!
//! # DDD Layer: Application
//! MCP integration for the prompt system.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP server status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum McpServerStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl McpServerStatus {
    pub fn color(&self) -> Color {
        match self {
            Self::Connected => Color::Green,
            Self::Connecting => Color::Yellow,
            Self::Disconnected => Color::Gray,
            Self::Error => Color::Red,
        }
    }
    
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Connected => "●",
            Self::Connecting => "◐",
            Self::Disconnected => "○",
            Self::Error => "✗",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Self::Connected => "Connected",
            Self::Connecting => "Connecting...",
            Self::Disconnected => "Disconnected",
            Self::Error => "Error",
        }
    }
}

/// An MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub server_name: String,
    #[serde(default)]
    pub parameters: Vec<McpToolParam>,
}

/// MCP tool parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolParam {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    #[serde(rename = "type")]
    pub param_type: String,
}

/// An MCP server definition
#[derive(Debug, Clone)]
pub struct McpServer {
    pub name: String,
    pub status: McpServerStatus,
    pub tools: Vec<McpTool>,
    pub error: Option<String>,
}

impl McpServer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: McpServerStatus::Disconnected,
            tools: Vec::new(),
            error: None,
        }
    }
    
    pub fn with_status(mut self, status: McpServerStatus) -> Self {
        self.status = status;
        self
    }
    
    pub fn with_tools(mut self, tools: Vec<McpTool>) -> Self {
        self.tools = tools;
        self
    }
    
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
        self.status = McpServerStatus::Error;
    }
    
    pub fn clear_error(&mut self) {
        self.error = None;
        if self.status == McpServerStatus::Error {
            self.status = McpServerStatus::Disconnected;
        }
    }
}

/// MCP manager - tracks all MCP servers and tools
#[derive(Debug, Default)]
pub struct McpManager {
    servers: HashMap<String, McpServer>,
    /// Cached flat list of all tools
    all_tools: Vec<McpTool>,
}

impl McpManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a server
    pub fn register_server(&mut self, server: McpServer) {
        self.servers.insert(server.name.clone(), server);
        self.rebuild_tools_cache();
    }
    
    /// Update server status
    pub fn update_status(&mut self, name: &str, status: McpServerStatus) {
        if let Some(server) = self.servers.get_mut(name) {
            server.status = status;
        }
    }
    
    /// Update server tools
    pub fn update_tools(&mut self, name: &str, tools: Vec<McpTool>) {
        if let Some(server) = self.servers.get_mut(name) {
            server.tools = tools;
            self.rebuild_tools_cache();
        }
    }
    
    /// Set server error
    pub fn set_error(&mut self, name: &str, error: impl Into<String>) {
        if let Some(server) = self.servers.get_mut(name) {
            server.set_error(error);
        }
    }
    
    /// Get server by name
    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }
    
    /// Get all servers
    pub fn servers(&self) -> impl Iterator<Item = &McpServer> {
        self.servers.values()
    }
    
    /// Get all tools (flattened)
    pub fn tools(&self) -> &[McpTool] {
        &self.all_tools
    }
    
    /// Get tools for a specific server
    pub fn server_tools(&self, name: &str) -> Option<&[McpTool]> {
        self.servers.get(name).map(|s| s.tools.as_slice())
    }
    
    /// Find tool by name
    pub fn find_tool(&self, name: &str) -> Option<&McpTool> {
        self.all_tools.iter().find(|t| t.name == name)
    }
    
    /// Get connected server count
    pub fn connected_count(&self) -> usize {
        self.servers.values().filter(|s| s.status == McpServerStatus::Connected).count()
    }
    
    /// Get total server count
    pub fn total_count(&self) -> usize {
        self.servers.len()
    }
    
    /// Rebuild the flat tools cache
    fn rebuild_tools_cache(&mut self) {
        self.all_tools = self.servers
            .values()
            .flat_map(|s| s.tools.clone())
            .collect();
    }
    
    /// Get status summary for display
    pub fn status_summary(&self) -> McpStatusSummary {
        let total = self.servers.len();
        let connected = self.connected_count();
        let error_count = self.servers.values().filter(|s| s.status == McpServerStatus::Error).count();
        let tool_count = self.all_tools.len();
        
        McpStatusSummary {
            total_servers: total,
            connected_servers: connected,
            error_servers: error_count,
            total_tools: tool_count,
        }
    }
}

/// Summary of MCP status for display
#[derive(Debug, Clone, Default)]
pub struct McpStatusSummary {
    pub total_servers: usize,
    pub connected_servers: usize,
    pub error_servers: usize,
    pub total_tools: usize,
}

impl McpStatusSummary {
    /// Get overall status
    pub fn overall_status(&self) -> McpServerStatus {
        if self.total_servers == 0 {
            McpServerStatus::Disconnected
        } else if self.error_servers > 0 {
            McpServerStatus::Error
        } else if self.connected_servers == self.total_servers {
            McpServerStatus::Connected
        } else if self.connected_servers > 0 {
            McpServerStatus::Connecting
        } else {
            McpServerStatus::Disconnected
        }
    }
    
    /// Format for display
    pub fn display(&self) -> String {
        if self.total_servers == 0 {
            "No MCP servers".to_string()
        } else {
            format!(
                "{}/{} servers, {} tools",
                self.connected_servers,
                self.total_servers,
                self.total_tools
            )
        }
    }
}

/// MCP event for prompt integration
#[derive(Debug, Clone)]
pub enum McpEvent {
    ServerConnected(String),
    ServerDisconnected(String),
    ServerError { name: String, error: String },
    ToolsUpdated(String),
    ToolExecuted { tool: String, success: bool },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_status() {
        assert_eq!(McpServerStatus::Connected.color(), Color::Green);
        assert_eq!(McpServerStatus::Error.indicator(), "✗");
    }
    
    #[test]
    fn test_mcp_server() {
        let mut server = McpServer::new("test-server")
            .with_status(McpServerStatus::Connected);
        
        assert_eq!(server.name, "test-server");
        assert_eq!(server.status, McpServerStatus::Connected);
        
        server.set_error("Connection failed");
        assert_eq!(server.status, McpServerStatus::Error);
        assert!(server.error.is_some());
    }
    
    #[test]
    fn test_mcp_manager() {
        let mut manager = McpManager::new();
        
        let server = McpServer::new("server1")
            .with_status(McpServerStatus::Connected)
            .with_tools(vec![
                McpTool {
                    name: "tool1".to_string(),
                    description: Some("A tool".to_string()),
                    server_name: "server1".to_string(),
                    parameters: vec![],
                },
            ]);
        
        manager.register_server(server);
        
        assert_eq!(manager.total_count(), 1);
        assert_eq!(manager.connected_count(), 1);
        assert_eq!(manager.tools().len(), 1);
    }
    
    #[test]
    fn test_status_summary() {
        let mut manager = McpManager::new();
        
        manager.register_server(McpServer::new("s1").with_status(McpServerStatus::Connected));
        manager.register_server(McpServer::new("s2").with_status(McpServerStatus::Error));
        
        let summary = manager.status_summary();
        assert_eq!(summary.total_servers, 2);
        assert_eq!(summary.connected_servers, 1);
        assert_eq!(summary.error_servers, 1);
        assert_eq!(summary.overall_status(), McpServerStatus::Error);
    }
    
    #[test]
    fn test_find_tool() {
        let mut manager = McpManager::new();
        
        let server = McpServer::new("server1")
            .with_tools(vec![
                McpTool {
                    name: "read_file".to_string(),
                    description: Some("Read a file".to_string()),
                    server_name: "server1".to_string(),
                    parameters: vec![],
                },
            ]);
        
        manager.register_server(server);
        
        assert!(manager.find_tool("read_file").is_some());
        assert!(manager.find_tool("nonexistent").is_none());
    }
}
