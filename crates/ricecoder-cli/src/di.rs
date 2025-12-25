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

/// Get a service from the global DI container.
///
/// Note: This is a temporary stub. The proper implementation should use
/// an application container passed through the call stack.
/// Currently returns None as container-based DI is not yet fully wired.
pub fn get_service<T>() -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    // TODO: Wire up proper container-based DI
    // The application should create a container at startup and pass it through
    None
}

/// Initialize the DI container.
///
/// Note: This is a temporary stub. The proper implementation should
/// create and configure an application container with all required services.
pub fn initialize_di_container() -> Result<(), String> {
    // TODO: Wire up proper container initialization
    // For now, services are created on-demand via factory pattern
    Ok(())
}

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
        // BrandingManager - Application branding (unit struct)
        ServiceEntry::new::<BrandingManager>(Arc::new(BrandingManager)),
        // CommandRouter - Command routing (unit struct)
        ServiceEntry::new::<CommandRouter>(Arc::new(CommandRouter)),
        // AccessibilitySettings - Accessibility configuration
        ServiceEntry::new::<AccessibilitySettings>(Arc::new(AccessibilitySettings::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_cli_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"cli"), "Factory should be registered");
    }
}
