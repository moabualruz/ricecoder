//! Dependency injection support for ricecoder-safety
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::validation::SafetyValidator;

// Auto-register safety services with the DI container
inventory::submit! {
    ServiceFactory::new("safety", create_safety_services)
}

/// Create all safety services for registration.
///
/// This factory function creates instances of all safety services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_safety_services() -> Vec<ServiceEntry> {
    vec![
        // SafetyValidator - Operation safety validation and approval gates
        ServiceEntry::new::<SafetyValidator>(Arc::new(SafetyValidator::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_safety_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"safety"),
            "Safety factory should be registered"
        );
    }

    #[test]
    fn test_create_safety_services() {
        let services = create_safety_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that SafetyValidator is in the list
        let has_safety_validator = services.iter().any(|s| {
            s.type_name.contains("SafetyValidator")
        });
        assert!(has_safety_validator, "Should include SafetyValidator");
    }
}
