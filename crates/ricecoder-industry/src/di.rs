//! Dependency injection support for ricecoder-industry
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::connections::ConnectionManager;

// Auto-register industry services with the DI container
inventory::submit! {
    ServiceFactory::new("industry", create_industry_services)
}

/// Create all industry services for registration.
///
/// This factory function creates instances of all industry services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_industry_services() -> Vec<ServiceEntry> {
    vec![
        // ConnectionManager - Enterprise tool connection management
        ServiceEntry::new::<ConnectionManager>(Arc::new(ConnectionManager::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_industry_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"industry"),
            "Industry factory should be registered"
        );
    }

    #[test]
    fn test_create_industry_services() {
        let services = create_industry_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that ConnectionManager is in the list
        let has_connection_manager = services.iter().any(|s| {
            s.type_name.contains("ConnectionManager")
        });
        assert!(has_connection_manager, "Should include ConnectionManager");
    }
}
