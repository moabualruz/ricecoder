//! Dependency injection support for ricecoder-mcp
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::{MCPClient, ToolRegistry};

// Auto-register MCP services with the DI container
inventory::submit! {
    ServiceFactory::new("mcp", create_mcp_services)
}

/// Create all MCP services for registration.
///
/// This factory function creates instances of all MCP services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_mcp_services() -> Vec<ServiceEntry> {
    vec![
        // MCPClient - Main MCP protocol client
        ServiceEntry::new::<MCPClient>(Arc::new(MCPClient::new())),
        // ToolRegistry - Tool discovery and registration
        ServiceEntry::new::<ToolRegistry>(Arc::new(ToolRegistry::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_mcp_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"mcp"),
            "MCP factory should be registered"
        );
    }

    #[test]
    fn test_create_mcp_services() {
        let services = create_mcp_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that MCPClient is in the list
        let has_mcp_client = services.iter().any(|s| {
            s.type_name.contains("MCPClient")
        });
        assert!(has_mcp_client, "Should include MCPClient");

        // Check that ToolRegistry is in the list
        let has_tool_registry = services.iter().any(|s| {
            s.type_name.contains("ToolRegistry")
        });
        assert!(has_tool_registry, "Should include ToolRegistry");
    }
}
