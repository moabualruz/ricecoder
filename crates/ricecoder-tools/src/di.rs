//! Dependency injection support for ricecoder-tools
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::ProviderRegistry;

// Auto-register tools services with the DI container
inventory::submit! {
    ServiceFactory::new("tools", create_tools_services)
}

/// Create all tools services for registration.
///
/// This factory function creates instances of all tools services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_tools_services() -> Vec<ServiceEntry> {
    vec![
        // ProviderRegistry - Tool provider management with fallback chain
        ServiceEntry::new::<ProviderRegistry>(Arc::new(ProviderRegistry::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_tools_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"tools"),
            "Tools factory should be registered"
        );
    }

    #[test]
    fn test_create_tools_services() {
        let services = create_tools_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that ProviderRegistry is in the list
        let has_provider_registry = services.iter().any(|s| {
            s.type_name.contains("ProviderRegistry")
        });
        assert!(has_provider_registry, "Should include ProviderRegistry");
    }
}
