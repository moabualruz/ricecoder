//! Dependency injection support for ricecoder-cli
//!
//! This module provides factory-return DI pattern for CLI services.
//! Services are registered via `inventory::submit!` and collected by ricecoder-di.

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{
    BrandingManager,
    CommandRouter,
    AccessibilitySettings,
    lifecycle::LifecycleManager,
};

inventory::submit! {
    ServiceFactory::new("cli", create_cli_services)
}

/// Create all CLI services for registration.
///
/// This factory function creates instances of all CLI services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_cli_services() -> Vec<ServiceEntry> {
    vec![
        // LifecycleManager - Component lifecycle management (replaces global static)
        ServiceEntry::new::<LifecycleManager>(Arc::new(LifecycleManager::new())),
        // BrandingManager - Application branding
        ServiceEntry::new::<BrandingManager>(Arc::new(BrandingManager::new())),
        // CommandRouter - Command routing
        ServiceEntry::new::<CommandRouter>(Arc::new(CommandRouter::new())),
        // AccessibilitySettings - Accessibility configuration
        ServiceEntry::new::<AccessibilitySettings>(Arc::new(AccessibilitySettings::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories();

    #[test]
    fn test_cli_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"cli"), "Factory should be registered");
    }
}
