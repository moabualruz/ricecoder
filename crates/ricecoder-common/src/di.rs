//! Dependency Injection registration types for auto-discovery
//!
//! This module provides the factory-return pattern for distributed service
//! registration across crates using the `inventory` crate.
//!
//! ## Why in ricecoder-common?
//!
//! These types are defined here (not in ricecoder-di) to avoid circular
//! dependencies. ricecoder-common has no dependencies on other ricecoder crates,
//! making it safe for all crates to depend on.
//!
//! ## Factory-Return Pattern
//!
//! Each crate creates its services and returns them as `ServiceEntry` items.
//! ricecoder-di collects all entries and registers them in its container.
//!
//! ## Usage
//!
//! In a feature crate (e.g., ricecoder-storage):
//!
//! ```rust,ignore
//! use ricecoder_common::di::{ServiceEntry, ServiceFactory};
//! use std::sync::Arc;
//!
//! inventory::submit! {
//!     ServiceFactory::new("storage", create_storage_services)
//! }
//!
//! fn create_storage_services() -> Vec<ServiceEntry> {
//!     vec![
//!         ServiceEntry::new::<StorageManager>(Arc::new(StorageManager::new())),
//!         ServiceEntry::new::<FileStorage>(Arc::new(FileStorage::new("./data"))),
//!     ]
//! }
//! ```

use std::any::{Any, TypeId};
use std::sync::Arc;
use tracing::{debug, info};

/// Error type for DI operations (minimal, to avoid dependencies)
#[derive(Debug, thiserror::Error)]
pub enum DIRegistrationError {
    #[error("Service registration failed: {message}")]
    RegistrationFailed { message: String },

    #[error("Service resolution failed: {message}")]
    ResolutionFailed { message: String },

    #[error("Service already registered: {type_name}")]
    ServiceAlreadyRegistered { type_name: String },
}

/// Result type for DI registration operations
pub type DIRegistrationResult<T> = Result<T, DIRegistrationError>;

/// A service entry containing a type-erased service instance.
///
/// Each crate creates `ServiceEntry` items for its services, which are then
/// collected by ricecoder-di and registered in the container.
pub struct ServiceEntry {
    /// The TypeId of the service (used as registration key)
    pub type_id: TypeId,

    /// Human-readable type name for debugging
    pub type_name: &'static str,

    /// The service instance (type-erased)
    pub instance: Arc<dyn Any + Send + Sync>,
}

impl ServiceEntry {
    /// Create a new service entry for a concrete type
    pub fn new<T: Send + Sync + 'static>(instance: Arc<T>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            instance: instance as Arc<dyn Any + Send + Sync>,
        }
    }

    /// Create a service entry from a boxed service (for trait objects)
    pub fn from_arc<T: Send + Sync + 'static>(instance: Arc<T>) -> Self {
        Self::new(instance)
    }
}

impl std::fmt::Debug for ServiceEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceEntry")
            .field("type_id", &self.type_id)
            .field("type_name", &self.type_name)
            .finish()
    }
}

/// A service factory that creates services for a module.
///
/// Each crate submits a `ServiceFactory` via `inventory::submit!`.
/// The factory function is called during container initialization to
/// create the services, which are then registered.
pub struct ServiceFactory {
    /// Name of the service group (e.g., "storage", "research", "mcp")
    pub name: &'static str,

    /// Factory function that creates and returns services
    pub factory_fn: fn() -> Vec<ServiceEntry>,

    /// Priority for registration order (lower = earlier, default = 100)
    pub priority: u32,

    /// Dependencies - names of other service groups that must be registered first
    pub dependencies: &'static [&'static str],
}

// SAFETY: ServiceFactory only contains:
// - &'static str (Sync)
// - fn pointer (Sync)
// - u32 (Sync)
// - &'static [&'static str] (Sync)
unsafe impl Sync for ServiceFactory {}

impl ServiceFactory {
    /// Create a new service factory with default priority
    pub const fn new(name: &'static str, factory_fn: fn() -> Vec<ServiceEntry>) -> Self {
        Self {
            name,
            factory_fn,
            priority: 100,
            dependencies: &[],
        }
    }

    /// Create a new service factory with custom priority
    pub const fn with_priority(
        name: &'static str,
        factory_fn: fn() -> Vec<ServiceEntry>,
        priority: u32,
    ) -> Self {
        Self {
            name,
            factory_fn,
            priority,
            dependencies: &[],
        }
    }

    /// Create a new service factory with dependencies
    pub const fn with_dependencies(
        name: &'static str,
        factory_fn: fn() -> Vec<ServiceEntry>,
        dependencies: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            factory_fn,
            priority: 100,
            dependencies,
        }
    }

    /// Create a new service factory with priority and dependencies
    pub const fn full(
        name: &'static str,
        factory_fn: fn() -> Vec<ServiceEntry>,
        priority: u32,
        dependencies: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            factory_fn,
            priority,
            dependencies,
        }
    }
}

// Collect all ServiceFactory instances across all crates
inventory::collect!(ServiceFactory);

/// Collect all services from discovered factories.
///
/// This function iterates over all `ServiceFactory` entries that were
/// submitted via `inventory::submit!` across all linked crates and calls
/// their factory functions to create services.
///
/// Services are created in priority order (lower priority value = earlier).
///
/// ## Returns
///
/// A vector of all `ServiceEntry` items from all factories, ready to be
/// registered in the DI container.
pub fn collect_all_services() -> Vec<ServiceEntry> {
    // Collect all factories
    let mut factories: Vec<&ServiceFactory> = inventory::iter::<ServiceFactory>().collect();

    // Sort by priority (stable sort to preserve insertion order for equal priorities)
    factories.sort_by_key(|f| f.priority);

    info!(
        "Discovered {} service factories via inventory",
        factories.len()
    );

    let mut all_services = Vec::new();

    // Execute each factory and collect services
    for factory in factories {
        debug!(
            "Creating services for '{}' (priority: {})",
            factory.name, factory.priority
        );
        let services = (factory.factory_fn)();
        debug!(
            "Factory '{}' created {} services",
            factory.name,
            services.len()
        );
        all_services.extend(services);
    }

    info!(
        "Collected {} total services from all factories",
        all_services.len()
    );
    all_services
}

/// Get the count of discovered service factories.
///
/// Useful for debugging and testing.
pub fn discovered_factory_count() -> usize {
    inventory::iter::<ServiceFactory>().count()
}

/// List all discovered service factory names.
///
/// Useful for debugging and diagnostics.
pub fn list_discovered_factories() -> Vec<&'static str> {
    inventory::iter::<ServiceFactory>().map(|f| f.name).collect()
}

// ============================================================================
// Legacy Support - Keep old types for backward compatibility during migration
// ============================================================================

/// Legacy: A service registration descriptor (deprecated, use ServiceFactory)
///
/// This type is kept for backward compatibility during migration.
/// New code should use `ServiceFactory` instead.
#[deprecated(since = "0.2.0", note = "Use ServiceFactory instead")]
pub struct ServiceRegistration {
    /// Name of the service group
    pub name: &'static str,
    /// Registration function (legacy - receives opaque container)
    pub register_fn: fn(&dyn Any) -> DIRegistrationResult<()>,
    /// Priority for registration order
    pub priority: u32,
    /// Dependencies
    pub dependencies: &'static [&'static str],
}

#[allow(deprecated)]
unsafe impl Sync for ServiceRegistration {}

#[allow(deprecated)]
impl ServiceRegistration {
    /// Create a new service registration (legacy)
    pub const fn new(
        name: &'static str,
        register_fn: fn(&dyn Any) -> DIRegistrationResult<()>,
    ) -> Self {
        Self {
            name,
            register_fn,
            priority: 100,
            dependencies: &[],
        }
    }

    /// Create with priority (legacy)
    pub const fn with_priority(
        name: &'static str,
        register_fn: fn(&dyn Any) -> DIRegistrationResult<()>,
        priority: u32,
    ) -> Self {
        Self {
            name,
            register_fn,
            priority,
            dependencies: &[],
        }
    }

    /// Create with dependencies (legacy)
    pub const fn with_dependencies(
        name: &'static str,
        register_fn: fn(&dyn Any) -> DIRegistrationResult<()>,
        dependencies: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            register_fn,
            priority: 100,
            dependencies,
        }
    }

    /// Create with all options (legacy)
    pub const fn full(
        name: &'static str,
        register_fn: fn(&dyn Any) -> DIRegistrationResult<()>,
        priority: u32,
        dependencies: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            register_fn,
            priority,
            dependencies,
        }
    }
}

// Collect legacy ServiceRegistration for backward compatibility
#[allow(deprecated)]
inventory::collect!(ServiceRegistration);

/// Legacy: Register all discovered services (deprecated)
#[deprecated(since = "0.2.0", note = "Use collect_all_services instead")]
#[allow(deprecated)]
pub fn register_all_discovered_services(container: &dyn Any) -> DIRegistrationResult<()> {
    let mut registrations: Vec<&ServiceRegistration> =
        inventory::iter::<ServiceRegistration>().collect();
    registrations.sort_by_key(|r| r.priority);

    info!(
        "Discovered {} legacy service registrations via inventory",
        registrations.len()
    );

    for registration in registrations {
        debug!(
            "Registering services for '{}' (priority: {})",
            registration.name, registration.priority
        );
        (registration.register_fn)(container)?;
    }

    info!("All legacy discovered services registered successfully");
    Ok(())
}

/// Legacy: Get count of discovered registrations
#[deprecated(since = "0.2.0", note = "Use discovered_factory_count instead")]
#[allow(deprecated)]
pub fn discovered_registration_count() -> usize {
    inventory::iter::<ServiceRegistration>().count()
}

/// Legacy: List discovered registration names
#[deprecated(since = "0.2.0", note = "Use list_discovered_factories instead")]
#[allow(deprecated)]
pub fn list_discovered_registrations() -> Vec<&'static str> {
    inventory::iter::<ServiceRegistration>()
        .map(|r| r.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test factory (will be collected by inventory)
    inventory::submit! {
        ServiceFactory::new("test_common_factory", create_test_services)
    }

    fn create_test_services() -> Vec<ServiceEntry> {
        vec![ServiceEntry::new::<String>(Arc::new(
            "test_service".to_string(),
        ))]
    }

    #[test]
    fn test_discovered_factories_include_test() {
        let names = list_discovered_factories();
        assert!(
            names.contains(&"test_common_factory"),
            "Should discover test_common_factory"
        );
    }

    #[test]
    fn test_factory_count() {
        let count = discovered_factory_count();
        assert!(count >= 1, "Should have at least the test factory");
    }

    #[test]
    fn test_collect_all_services() {
        let services = collect_all_services();
        assert!(!services.is_empty(), "Should collect at least test services");

        // Find our test service
        let has_string_service = services
            .iter()
            .any(|s| s.type_id == TypeId::of::<String>());
        assert!(has_string_service, "Should have String service from test factory");
    }

    #[test]
    fn test_service_entry_creation() {
        let service = Arc::new(42i32);
        let entry = ServiceEntry::new::<i32>(service);

        assert_eq!(entry.type_id, TypeId::of::<i32>());
        assert!(entry.type_name.contains("i32"));
    }
}
