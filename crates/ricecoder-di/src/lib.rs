//! Dependency Injection Container for RiceCoder
//!
//! This crate provides a service locator pattern implementation for managing
//! dependencies across the RiceCoder application. It enables clean architecture
//! by decoupling component creation and wiring.
//!
//! ## Quick Start
//!
//! ```rust
//! use ricecoder_di::create_application_container;
//!
//! // Create container with core services
//! let container = create_application_container().unwrap();
//!
//! // Resolve services
//! let session_manager = container.resolve::<ricecoder_sessions::SessionManager>().unwrap();
//! ```
//!
//! See [`usage`] module for detailed usage examples.

pub mod provider;
pub mod registration;
pub mod services;
pub mod usage;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tracing::{debug, info, warn};

/// Errors that can occur during dependency injection operations
#[derive(Debug, thiserror::Error)]
pub enum DIError {
    #[error("Service not registered: {service_type}")]
    ServiceNotRegistered { service_type: String },

    #[error("Service already registered: {service_type}")]
    ServiceAlreadyRegistered { service_type: String },

    #[error("Invalid service type: {message}")]
    InvalidServiceType { message: String },

    #[error("Dependency resolution failed: {message}")]
    DependencyResolutionFailed { message: String },

    #[error("Service health check failed: {service_type} - {reason}")]
    HealthCheckFailed {
        service_type: String,
        reason: String,
    },

    #[error("Circular dependency detected: {service_chain}")]
    CircularDependency { service_chain: String },
}

pub type DIResult<T> = Result<T, DIError>;

/// Service lifetime management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    /// Service is created once and reused for the entire application lifetime
    Singleton,
    /// Service is created each time it's requested
    Transient,
    /// Service is created once per scope (not implemented yet)
    Scoped,
}

/// Service descriptor containing registration information
struct ServiceDescriptor {
    factory: Box<dyn Fn(&DIContainer) -> DIResult<Arc<dyn Any + Send + Sync>> + Send + Sync>,
    lifetime: ServiceLifetime,
    instance: Option<Arc<dyn Any + Send + Sync>>,
    health_check:
        Option<Box<dyn Fn(&Arc<dyn Any + Send + Sync>) -> DIResult<HealthStatus> + Send + Sync>>,
}

/// Service scope for managing scoped service instances
#[derive(Debug)]
pub struct ServiceScope {
    scoped_instances: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl ServiceScope {
    /// Create a new service scope
    pub fn new() -> Self {
        Self {
            scoped_instances: RwLock::new(HashMap::new()),
        }
    }

    /// Get a scoped service instance
    pub fn get_scoped<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let instances = self.scoped_instances.read().unwrap();
        instances
            .get(&type_id)
            .and_then(|instance| instance.clone().downcast::<T>().ok())
    }

    /// Set a scoped service instance
    pub fn set_scoped<T>(&self, instance: Arc<T>)
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut instances = self.scoped_instances.write().unwrap();
        instances.insert(type_id, instance as Arc<dyn Any + Send + Sync>);
    }

    /// Clear all scoped instances
    pub fn clear(&self) {
        let mut instances = self.scoped_instances.write().unwrap();
        instances.clear();
    }
}

impl Default for ServiceScope {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for services that support health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform a health check on the service
    async fn health_check(&self) -> DIResult<HealthStatus>;
}

/// Health status of a service
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy and operational
    Healthy,
    /// Service is degraded but still functional
    Degraded(String),
    /// Service is unhealthy and not functional
    Unhealthy(String),
}

/// Service dependency information for validation
#[derive(Debug, Clone)]
pub struct ServiceDependency {
    /// The service type that depends on others
    pub service_type: String,
    /// List of service types this service depends on
    pub dependencies: Vec<String>,
}

/// The dependency injection container
pub struct DIContainer {
    services: RwLock<HashMap<TypeId, ServiceDescriptor>>,
}

impl DIContainer {
    /// Create a new empty DI container
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Register a service with a factory function
    pub fn register<F, T>(&self, factory: F) -> DIResult<()>
    where
        F: Fn(&DIContainer) -> DIResult<Arc<T>> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();

        if services.contains_key(&type_id) {
            return Err(DIError::ServiceAlreadyRegistered {
                service_type: std::any::type_name::<T>().to_string(),
            });
        }

        let wrapped_factory = Box::new(
            move |container: &DIContainer| -> DIResult<Arc<dyn Any + Send + Sync>> {
                let result = factory(container)?;
                Ok(result as Arc<dyn Any + Send + Sync>)
            },
        );

        let descriptor = ServiceDescriptor {
            factory: wrapped_factory,
            lifetime: ServiceLifetime::Singleton,
            instance: None,
            health_check: None,
        };

        services.insert(type_id, descriptor);

        debug!("Registered service: {}", std::any::type_name::<T>());
        Ok(())
    }

    /// Register a transient service
    pub fn register_transient<F, T>(&self, factory: F) -> DIResult<()>
    where
        F: Fn(&DIContainer) -> DIResult<Arc<T>> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();

        if services.contains_key(&type_id) {
            return Err(DIError::ServiceAlreadyRegistered {
                service_type: std::any::type_name::<T>().to_string(),
            });
        }

        let wrapped_factory = Box::new(
            move |container: &DIContainer| -> DIResult<Arc<dyn Any + Send + Sync>> {
                let result = factory(container)?;
                Ok(result as Arc<dyn Any + Send + Sync>)
            },
        );

        let descriptor = ServiceDescriptor {
            factory: wrapped_factory,
            lifetime: ServiceLifetime::Transient,
            instance: None,
            health_check: None,
        };

        services.insert(type_id, descriptor);

        debug!(
            "Registered transient service: {}",
            std::any::type_name::<T>()
        );
        Ok(())
    }

    /// Register a scoped service
    pub fn register_scoped<F, T>(&self, factory: F) -> DIResult<()>
    where
        F: Fn(&DIContainer) -> DIResult<Arc<T>> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();

        if services.contains_key(&type_id) {
            return Err(DIError::ServiceAlreadyRegistered {
                service_type: std::any::type_name::<T>().to_string(),
            });
        }

        let wrapped_factory = Box::new(
            move |container: &DIContainer| -> DIResult<Arc<dyn Any + Send + Sync>> {
                let result = factory(container)?;
                Ok(result as Arc<dyn Any + Send + Sync>)
            },
        );

        let descriptor = ServiceDescriptor {
            factory: wrapped_factory,
            lifetime: ServiceLifetime::Scoped,
            instance: None,
            health_check: None,
        };

        services.insert(type_id, descriptor);

        debug!("Registered scoped service: {}", std::any::type_name::<T>());
        Ok(())
    }

    /// Register a service with a health check
    pub fn register_with_health_check<F, H, T>(&self, factory: F, health_check: H) -> DIResult<()>
    where
        F: Fn(&DIContainer) -> DIResult<Arc<T>> + Send + Sync + 'static,
        H: Fn(&Arc<dyn Any + Send + Sync>) -> DIResult<HealthStatus> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();

        if services.contains_key(&type_id) {
            return Err(DIError::ServiceAlreadyRegistered {
                service_type: std::any::type_name::<T>().to_string(),
            });
        }

        let wrapped_factory = Box::new(
            move |container: &DIContainer| -> DIResult<Arc<dyn Any + Send + Sync>> {
                let result = factory(container)?;
                Ok(result as Arc<dyn Any + Send + Sync>)
            },
        );

        let descriptor = ServiceDescriptor {
            factory: wrapped_factory,
            lifetime: ServiceLifetime::Singleton,
            instance: None,
            health_check: Some(Box::new(move |instance: &Arc<dyn Any + Send + Sync>| {
                health_check(instance)
            })),
        };

        services.insert(type_id, descriptor);

        debug!(
            "Registered service with health check: {}",
            std::any::type_name::<T>()
        );
        Ok(())
    }

    /// Resolve a service instance
    pub fn resolve<T>(&self) -> DIResult<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.resolve_with_scope::<T>(None)
    }

    /// Resolve a service instance with an optional scope
    pub fn resolve_with_scope<T>(&self, scope: Option<&ServiceScope>) -> DIResult<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();

        let descriptor =
            services
                .get_mut(&type_id)
                .ok_or_else(|| DIError::ServiceNotRegistered {
                    service_type: std::any::type_name::<T>().to_string(),
                })?;

        match descriptor.lifetime {
            ServiceLifetime::Singleton => {
                if let Some(instance) = &descriptor.instance {
                    // Try to downcast the existing instance
                    if let Ok(downcasted) = instance.clone().downcast::<T>() {
                        return Ok(downcasted);
                    }
                }

                // Create new instance
                let instance = (descriptor.factory)(self)?;
                let downcasted =
                    instance
                        .downcast::<T>()
                        .map_err(|_| DIError::InvalidServiceType {
                            message: "Service type mismatch during downcast".to_string(),
                        })?;
                descriptor.instance = Some(downcasted.clone());
                Ok(downcasted)
            }
            ServiceLifetime::Transient => {
                // Always create new instance
                let instance = (descriptor.factory)(self)?;
                instance
                    .downcast::<T>()
                    .map_err(|_| DIError::InvalidServiceType {
                        message: "Service type mismatch during downcast".to_string(),
                    })
            }
            ServiceLifetime::Scoped => {
                // Check if we have a scope
                if let Some(scope) = scope {
                    // Try to get existing scoped instance
                    if let Some(instance) = scope.get_scoped::<T>() {
                        return Ok(instance);
                    }

                    // Create new scoped instance
                    let instance = (descriptor.factory)(self)?;
                    let downcasted =
                        instance
                            .downcast::<T>()
                            .map_err(|_| DIError::InvalidServiceType {
                                message: "Service type mismatch during downcast".to_string(),
                            })?;
                    scope.set_scoped(downcasted.clone());
                    Ok(downcasted)
                } else {
                    Err(DIError::InvalidServiceType {
                        message: "Scoped services require a service scope".to_string(),
                    })
                }
            }
        }
    }

    /// Check if a service is registered
    pub fn is_registered<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let services = self.services.read().unwrap();
        services.contains_key(&type_id)
    }

    /// Get the number of registered services
    pub fn service_count(&self) -> usize {
        let services = self.services.read().unwrap();
        services.len()
    }

    /// Clear all registered services
    pub fn clear(&self) {
        let mut services = self.services.write().unwrap();
        services.clear();
        info!("Cleared all services from DI container");
    }

    /// Register a pre-created service instance from a ServiceEntry
    ///
    /// This is used by the factory-return DI pattern where crates create
    /// their own services and return them as ServiceEntry items.
    pub fn register_entry(&self, entry: ricecoder_common::di::ServiceEntry) -> DIResult<()> {
        let mut services = self.services.write().unwrap();

        if services.contains_key(&entry.type_id) {
            // Skip if already registered (allows for priority-based registration)
            debug!(
                "Service already registered, skipping: {}",
                entry.type_name
            );
            return Ok(());
        }

        // Create a factory that returns the pre-created instance
        let instance = entry.instance;
        let wrapped_factory = Box::new(
            move |_container: &DIContainer| -> DIResult<Arc<dyn Any + Send + Sync>> {
                Ok(instance.clone())
            },
        );

        let descriptor = ServiceDescriptor {
            factory: wrapped_factory,
            lifetime: ServiceLifetime::Singleton,
            instance: None, // Will be set on first resolve
            health_check: None,
        };

        services.insert(entry.type_id, descriptor);

        debug!("Registered service from entry: {}", entry.type_name);
        Ok(())
    }

    /// Perform health checks on all registered services that have health checks
    pub fn health_check_all(&self) -> DIResult<Vec<(String, HealthStatus)>> {
        let services = self.services.read().unwrap();
        let mut results = Vec::new();

        for (type_id, descriptor) in services.iter() {
            if let Some(health_check_fn) = &descriptor.health_check {
                // Try to resolve the service
                if let Ok(instance) = (descriptor.factory)(self) {
                    let status = health_check_fn(&instance)?;
                    results.push((format!("{:?}", type_id), status));
                }
            }
        }

        Ok(results)
    }

    /// Validate service dependencies to detect circular dependencies
    pub fn validate_dependencies(&self, dependencies: &[ServiceDependency]) -> DIResult<()> {
        for dep in dependencies {
            if self.has_circular_dependency(dep, &mut Vec::new()) {
                return Err(DIError::CircularDependency {
                    service_chain: dep.service_type.clone(),
                });
            }
        }
        Ok(())
    }

    /// Check for circular dependencies in a service dependency chain
    fn has_circular_dependency(&self, dep: &ServiceDependency, visited: &mut Vec<String>) -> bool {
        if visited.contains(&dep.service_type) {
            return true;
        }

        visited.push(dep.service_type.clone());

        for dependency in &dep.dependencies {
            // For simplicity, we'll just check if the dependency exists
            // In a real implementation, you'd recursively check the dependency graph
            if visited.contains(dependency) {
                return true;
            }
        }

        visited.pop();
        false
    }
}

impl Default for DIContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for configuring the DI container
pub struct DIContainerBuilder {
    container: DIContainer,
}

impl DIContainerBuilder {
    /// Create a new container builder
    pub fn new() -> Self {
        Self {
            container: DIContainer::new(),
        }
    }

    /// Register a service
    pub fn register<F, T>(mut self, factory: F) -> DIResult<Self>
    where
        F: Fn(&DIContainer) -> DIResult<Arc<T>> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.container.register(factory)?;
        Ok(self)
    }

    /// Register a transient service
    pub fn register_transient<F, T>(mut self, factory: F) -> DIResult<Self>
    where
        F: Fn(&DIContainer) -> DIResult<Arc<T>> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.container.register_transient(factory)?;
        Ok(self)
    }

    /// Register a scoped service
    pub fn register_scoped<F, T>(mut self, factory: F) -> DIResult<Self>
    where
        F: Fn(&DIContainer) -> DIResult<Arc<T>> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.container.register_scoped(factory)?;
        Ok(self)
    }

    /// Build the container
    pub fn build(self) -> DIContainer {
        self.container
    }
}

impl Default for DIContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for registering services
#[macro_export]
macro_rules! register_service {
    ($container:expr, $service_type:ty, $factory:expr) => {
        $container.register::<$service_type, _>($factory)
    };
}

/// Convenience macro for resolving services
#[macro_export]
macro_rules! resolve_service {
    ($container:expr, $service_type:ty) => {
        $container.resolve::<$service_type>()
    };
}

// Re-export service registration functions
#[cfg(feature = "full")]
pub use services::create_full_application_container;
pub use services::{
    create_application_container, create_cli_container, create_configured_container,
    create_development_container, create_test_container, create_tui_container,
    register_discovered_services, register_infrastructure_services, register_use_cases,
    ContainerConfig, DIContainerBuilderExt, Lifecycle, LifecycleManager,
};

// Re-export ServiceProvider traits
pub use provider::{
    HealthCheckProvider, LifecycleServiceProvider, ServiceProvider, ServiceProviderRegistry,
};

// Re-export auto-discovery registration types
pub use registration::{
    discovered_registration_count, list_discovered_registrations,
    register_all_discovered_services, ServiceRegistration,
};
