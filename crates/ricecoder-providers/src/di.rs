//! Dependency injection support for ricecoder-providers
//!
//! This module provides factory-return DI pattern for provider services.
//! Services are registered via `inventory::submit!` and collected by ricecoder-di.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::redaction::RedactionFilter;

// Auto-register provider services with the DI container
inventory::submit! {
    ServiceFactory::new("providers", create_provider_services)
}

/// Create all provider services for registration.
///
/// This factory function creates instances of all provider services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_provider_services() -> Vec<ServiceEntry> {
    vec![
        // RedactionFilter - Redact sensitive information from logs/output
        // Previously a global static, now a DI service
        ServiceEntry::new::<RedactionFilter>(Arc::new(RedactionFilter::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_providers_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"providers"),
            "Providers factory should be registered"
        );
    }

    #[test]
    fn test_create_provider_services() {
        let services = create_provider_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that RedactionFilter is present
        let has_redaction = services
            .iter()
            .any(|s| s.type_name.contains("RedactionFilter"));
        assert!(has_redaction, "Should include RedactionFilter");
    }
}
