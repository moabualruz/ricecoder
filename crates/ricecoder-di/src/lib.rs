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

pub mod services;
pub mod usage;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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

        let wrapped_factory = Box::new(move |container: &DIContainer| -> DIResult<Arc<dyn Any + Send + Sync>> {
            let result = factory(container)?;
            Ok(result as Arc<dyn Any + Send + Sync>)
        });

        let descriptor = ServiceDescriptor {
            factory: wrapped_factory,
            lifetime: ServiceLifetime::Singleton,
            instance: None,
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

        let wrapped_factory = Box::new(move |container: &DIContainer| -> DIResult<Arc<dyn Any + Send + Sync>> {
            let result = factory(container)?;
            Ok(result as Arc<dyn Any + Send + Sync>)
        });

        let descriptor = ServiceDescriptor {
            factory: wrapped_factory,
            lifetime: ServiceLifetime::Transient,
            instance: None,
        };

        services.insert(type_id, descriptor);

        debug!("Registered transient service: {}", std::any::type_name::<T>());
        Ok(())
    }

    /// Resolve a service instance
    pub fn resolve<T>(&self) -> DIResult<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();

        let descriptor = services.get_mut(&type_id).ok_or_else(|| {
            DIError::ServiceNotRegistered {
                service_type: std::any::type_name::<T>().to_string(),
            }
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
                let downcasted = instance.downcast::<T>()
                    .map_err(|_| DIError::InvalidServiceType {
                        message: "Service type mismatch during downcast".to_string(),
                    })?;
                descriptor.instance = Some(downcasted.clone());
                Ok(downcasted)
            }
            ServiceLifetime::Transient => {
                // Always create new instance
                let instance = (descriptor.factory)(self)?;
                instance.downcast::<T>()
                    .map_err(|_| DIError::InvalidServiceType {
                        message: "Service type mismatch during downcast".to_string(),
                    })
            }
            ServiceLifetime::Scoped => {
                // Not implemented yet
                Err(DIError::InvalidServiceType {
                    message: "Scoped lifetime not implemented".to_string(),
                })
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
pub use services::{
    create_application_container,
    register_infrastructure_services,
    register_use_cases,
};

#[cfg(feature = "full")]
pub use services::create_full_application_container;

#[cfg(feature = "storage")]
pub use services::register_storage_services;

#[cfg(feature = "research")]
pub use services::register_research_services;

#[cfg(feature = "workflows")]
pub use services::register_workflow_services;

#[cfg(feature = "execution")]
pub use services::register_execution_services;

#[cfg(feature = "mcp")]
pub use services::register_mcp_services;

#[cfg(feature = "tools")]
pub use services::register_tool_services;

#[cfg(feature = "config")]
pub use services::register_config_services;

#[cfg(feature = "activity-log")]
pub use services::register_activity_log_services;

#[cfg(feature = "orchestration")]
pub use services::register_orchestration_services;

#[cfg(feature = "specs")]
pub use services::register_specs_services;

#[cfg(feature = "undo-redo")]
pub use services::register_undo_redo_services;

#[cfg(feature = "vcs")]
pub use services::register_vcs_services;

#[cfg(feature = "permissions")]
pub use services::register_permissions_services;

#[cfg(feature = "security")]
pub use services::register_security_services;

#[cfg(feature = "cache")]
pub use services::register_cache_services;

#[cfg(feature = "domain")]
pub use services::register_domain_services;

#[cfg(feature = "learning")]
pub use services::register_learning_services;

#[cfg(feature = "industry")]
pub use services::register_industry_services;

#[cfg(feature = "safety")]
pub use services::register_safety_services;

#[cfg(feature = "files")]
pub use services::register_files_services;

#[cfg(feature = "themes")]
pub use services::register_themes_services;

#[cfg(feature = "images")]
pub use services::register_images_services;

