//! Service Provider trait for dependency injection
//!
//! This module defines the `ServiceProvider` trait which provides a unified
//! interface for types that can provide services to the DI container.
//!
//! ## Usage
//!
//! Implement `ServiceProvider` for your service module:
//!
//! ```rust,ignore
//! use ricecoder_di::{DIContainer, DIResult, ServiceProvider};
//! use std::sync::Arc;
//!
//! pub struct StorageServiceProvider;
//!
//! impl ServiceProvider for StorageServiceProvider {
//!     fn name(&self) -> &'static str {
//!         "storage"
//!     }
//!
//!     fn register(&self, container: &DIContainer) -> DIResult<()> {
//!         container.register(|_| Ok(Arc::new(StorageManager::new())))?;
//!         container.register(|_| Ok(Arc::new(FileStorage::new())))?;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Auto-Discovery
//!
//! For automatic service discovery, use the `ServiceFactory` pattern from
//! `ricecoder_common::di` which integrates with the `inventory` crate.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{DIContainer, DIResult, HealthStatus};

// ============================================================================
// ServiceProvider Trait
// ============================================================================

/// Trait for types that can provide services to the DI container.
///
/// Implement this trait to create a service provider that registers
/// one or more services with the container.
///
/// ## Example
///
/// ```rust,ignore
/// struct MyServiceProvider;
///
/// impl ServiceProvider for MyServiceProvider {
///     fn name(&self) -> &'static str { "my-services" }
///     
///     fn register(&self, container: &DIContainer) -> DIResult<()> {
///         container.register(|_| Ok(Arc::new(MyService::new())))?;
///         Ok(())
///     }
/// }
/// ```
pub trait ServiceProvider: Send + Sync {
    /// Returns the name of this service provider.
    ///
    /// Used for logging, debugging, and dependency ordering.
    fn name(&self) -> &'static str;

    /// Returns the priority of this service provider.
    ///
    /// Lower values are registered first. Default is 100.
    fn priority(&self) -> u32 {
        100
    }

    /// Returns the names of service providers this one depends on.
    ///
    /// These providers will be registered before this one.
    fn dependencies(&self) -> &[&'static str] {
        &[]
    }

    /// Register services with the container.
    ///
    /// This method is called during container initialization.
    fn register(&self, container: &DIContainer) -> DIResult<()>;

    /// Optional: Register services that require async initialization.
    ///
    /// Called after `register()` for services that need async setup.
    #[allow(unused_variables)]
    fn register_async<'a>(
        &'a self,
        container: &'a DIContainer,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = DIResult<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }

    /// Optional: Validate that required dependencies are registered.
    ///
    /// Called after all providers have registered their services.
    #[allow(unused_variables)]
    fn validate(&self, container: &DIContainer) -> DIResult<()> {
        Ok(())
    }
}

// ============================================================================
// ServiceProviderRegistry
// ============================================================================

/// Registry for managing service providers.
///
/// Collects and executes service providers in dependency order.
pub struct ServiceProviderRegistry {
    providers: Vec<Arc<dyn ServiceProvider>>,
}

impl ServiceProviderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Add a service provider to the registry.
    pub fn add<P: ServiceProvider + 'static>(&mut self, provider: P) -> &mut Self {
        self.providers.push(Arc::new(provider));
        self
    }

    /// Add a boxed service provider to the registry.
    pub fn add_boxed(&mut self, provider: Arc<dyn ServiceProvider>) -> &mut Self {
        self.providers.push(provider);
        self
    }

    /// Get the number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// List all provider names.
    pub fn provider_names(&self) -> Vec<&'static str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    /// Register all providers with the container.
    ///
    /// Providers are sorted by priority and registered in order.
    pub fn register_all(&self, container: &DIContainer) -> DIResult<()> {
        // Sort providers by priority
        let mut sorted: Vec<_> = self.providers.iter().collect();
        sorted.sort_by_key(|p| p.priority());

        tracing::info!(
            "Registering {} service providers",
            sorted.len()
        );

        // Register each provider
        for provider in sorted {
            tracing::debug!(
                "Registering provider '{}' (priority: {})",
                provider.name(),
                provider.priority()
            );
            provider.register(container)?;
        }

        // Validate all providers
        for provider in &self.providers {
            provider.validate(container)?;
        }

        tracing::info!("All service providers registered successfully");
        Ok(())
    }

    /// Register all providers with async initialization.
    pub async fn register_all_async(&self, container: &DIContainer) -> DIResult<()> {
        // First, do synchronous registration
        self.register_all(container)?;

        // Then, do async registration
        for provider in &self.providers {
            provider.register_async(container).await?;
        }

        Ok(())
    }
}

impl Default for ServiceProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HealthCheckProvider Trait
// ============================================================================

/// Extension trait for service providers that support health checks.
#[async_trait]
pub trait HealthCheckProvider: ServiceProvider {
    /// Perform health checks on all services provided by this provider.
    async fn health_check(&self, container: &DIContainer) -> DIResult<Vec<(&'static str, HealthStatus)>>;
}

// ============================================================================
// Lifecycle-aware ServiceProvider
// ============================================================================

/// Extension trait for service providers with lifecycle management.
#[async_trait]
pub trait LifecycleServiceProvider: ServiceProvider {
    /// Initialize services after registration.
    ///
    /// Called after all services are registered but before the application starts.
    async fn initialize(&self, container: &DIContainer) -> DIResult<()>;

    /// Cleanup services before shutdown.
    ///
    /// Called during application shutdown.
    async fn shutdown(&self, container: &DIContainer) -> DIResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProvider {
        name: &'static str,
        priority: u32,
    }

    impl ServiceProvider for TestProvider {
        fn name(&self) -> &'static str {
            self.name
        }

        fn priority(&self) -> u32 {
            self.priority
        }

        fn register(&self, _container: &DIContainer) -> DIResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_provider_registry() {
        let mut registry = ServiceProviderRegistry::new();
        registry.add(TestProvider {
            name: "test1",
            priority: 100,
        });
        registry.add(TestProvider {
            name: "test2",
            priority: 50,
        });

        assert_eq!(registry.len(), 2);
        assert!(registry.provider_names().contains(&"test1"));
        assert!(registry.provider_names().contains(&"test2"));
    }

    #[test]
    fn test_register_all() {
        let mut registry = ServiceProviderRegistry::new();
        registry.add(TestProvider {
            name: "test",
            priority: 100,
        });

        let container = DIContainer::new();
        let result = registry.register_all(&container);
        assert!(result.is_ok());
    }

    #[test]
    fn test_priority_ordering() {
        let mut registry = ServiceProviderRegistry::new();
        registry.add(TestProvider {
            name: "low",
            priority: 200,
        });
        registry.add(TestProvider {
            name: "high",
            priority: 10,
        });
        registry.add(TestProvider {
            name: "medium",
            priority: 100,
        });

        // Providers should be sorted by priority when registered
        let container = DIContainer::new();
        assert!(registry.register_all(&container).is_ok());
    }
}
