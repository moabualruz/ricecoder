//! Dependency injection support for ricecoder-domain
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::services::{AnalysisService, ValidationService};

// Auto-register domain services with the DI container
inventory::submit! {
    ServiceFactory::new("domain", create_domain_services)
}

/// Create all domain services for registration.
///
/// This factory function creates instances of all domain services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_domain_services() -> Vec<ServiceEntry> {
    vec![
        // ValidationService - Business rule validation
        ServiceEntry::new::<ValidationService>(Arc::new(ValidationService)),
        // AnalysisService - Code analysis operations
        ServiceEntry::new::<AnalysisService>(Arc::new(AnalysisService)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_domain_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"domain"),
            "Domain factory should be registered"
        );
    }

    #[test]
    fn test_create_domain_services() {
        let services = create_domain_services();
        assert_eq!(services.len(), 2, "Should create 2 services");

        // Check that both services are present
        let has_validation = services.iter().any(|s| s.type_name.contains("ValidationService"));
        let has_analysis = services.iter().any(|s| s.type_name.contains("AnalysisService"));
        
        assert!(has_validation, "Should include ValidationService");
        assert!(has_analysis, "Should include AnalysisService");
    }
}
