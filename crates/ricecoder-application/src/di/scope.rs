//! Scoped Container Implementation
//!
//! REQ-ARCH-003: Scoped lifetime - created once per scope (request/session)

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::container::ServiceContainer;
use super::error::ContainerError;

/// A scoped container that provides scoped lifetime semantics
///
/// Scoped services are created once per scope and shared within that scope.
/// When the scope is dropped, all scoped instances are released.
///
/// # Example
///
/// ```ignore
/// let scope = container.create_scope();
/// 
/// // First resolution creates the instance
/// let ctx1: Arc<RequestContext> = scope.resolve().unwrap();
/// 
/// // Second resolution returns the same instance
/// let ctx2: Arc<RequestContext> = scope.resolve().unwrap();
/// 
/// assert!(Arc::ptr_eq(&ctx1, &ctx2));
/// // When scope is dropped, ctx1 and ctx2 references remain valid
/// // but no new instances can be created from this scope
/// ```
pub struct ScopedContainer<'a> {
    /// Reference to parent container for factory access
    parent: &'a ServiceContainer,

    /// Scoped instances created within this scope
    /// These are created on first resolution and reused within the scope
    instances: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl<'a> ScopedContainer<'a> {
    /// Create a new scoped container from a parent container
    pub(super) fn new(parent: &'a ServiceContainer) -> Self {
        Self {
            parent,
            instances: RwLock::new(HashMap::new()),
        }
    }

    /// Resolve a service within this scope
    ///
    /// Resolution order:
    /// 1. Singletons from parent container
    /// 2. Scoped instances (created once per scope)
    /// 3. Transient instances (created every time)
    ///
    /// # Errors
    /// - `ServiceNotRegistered` if no service is registered for the type
    /// - `TypeMismatch` if the service cannot be downcast to the requested type
    pub fn resolve<T: 'static + Send + Sync>(&self) -> Result<Arc<T>, ContainerError> {
        let type_id = TypeId::of::<T>();

        // Try singleton from parent first
        if let Some(instance) = self.parent.get_singleton(&type_id) {
            return instance
                .clone()
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()));
        }

        // Try scoped instance (check if already created in this scope)
        {
            let instances = self.instances.read().unwrap();
            if let Some(instance) = instances.get(&type_id) {
                return instance
                    .clone()
                    .downcast::<T>()
                    .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()));
            }
        }

        // Try to create scoped instance from factory
        if let Some(factory) = self.parent.get_scoped_factory(&type_id) {
            let instance = factory();
            let result = instance
                .clone()
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()))?;

            // Store in scope for future resolutions
            let mut instances = self.instances.write().unwrap();
            instances.insert(type_id, instance);

            return Ok(result);
        }

        // Try transient factory (creates new instance every time)
        if let Some(factory) = self.parent.get_transient_factory(&type_id) {
            return factory()
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()));
        }

        Err(ContainerError::ServiceNotRegistered(std::any::type_name::<
            T,
        >()))
    }

    /// Check if a service is registered (in parent or scoped)
    pub fn is_registered<T: 'static>(&self) -> bool {
        self.parent.is_registered::<T>()
    }

    /// Get the number of scoped instances created in this scope
    pub fn scoped_instance_count(&self) -> usize {
        self.instances.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct ScopedService {
        id: usize,
    }

    impl ScopedService {
        fn new(counter: &AtomicUsize) -> Self {
            Self {
                id: counter.fetch_add(1, Ordering::SeqCst) + 1,
            }
        }

        fn id(&self) -> usize {
            self.id
        }
    }

    #[test]
    fn test_scoped_same_instance_within_scope() {
        let mut container = ServiceContainer::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        container.register_scoped(move || ScopedService::new(&counter_clone));

        let scope = container.create_scope();

        let s1: Arc<ScopedService> = scope.resolve().unwrap();
        let s2: Arc<ScopedService> = scope.resolve().unwrap();

        // Same instance within scope
        assert_eq!(s1.id(), s2.id());
        assert!(Arc::ptr_eq(&s1, &s2));
        // Only created once
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_scoped_different_instance_different_scope() {
        let mut container = ServiceContainer::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        container.register_scoped(move || ScopedService::new(&counter_clone));

        // First scope
        let scope1 = container.create_scope();
        let s1: Arc<ScopedService> = scope1.resolve().unwrap();

        // Second scope
        let scope2 = container.create_scope();
        let s2: Arc<ScopedService> = scope2.resolve().unwrap();

        // Different instances in different scopes
        assert_ne!(s1.id(), s2.id());
        assert!(!Arc::ptr_eq(&s1, &s2));
        // Created twice (once per scope)
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_singleton_accessible_from_scope() {
        let mut container = ServiceContainer::new();
        container.register_singleton(Arc::new(ScopedService { id: 42 }));

        let scope = container.create_scope();
        let service: Arc<ScopedService> = scope.resolve().unwrap();

        assert_eq!(service.id(), 42);
    }

    #[test]
    fn test_transient_creates_new_in_scope() {
        let mut container = ServiceContainer::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        container.register_transient(move || ScopedService::new(&counter_clone));

        let scope = container.create_scope();

        let s1: Arc<ScopedService> = scope.resolve().unwrap();
        let s2: Arc<ScopedService> = scope.resolve().unwrap();

        // Different instances even within same scope (transient)
        assert_ne!(s1.id(), s2.id());
        assert!(!Arc::ptr_eq(&s1, &s2));
    }

    #[test]
    fn test_scoped_instance_count() {
        let mut container = ServiceContainer::new();
        container.register_scoped(|| ScopedService { id: 1 });

        let scope = container.create_scope();
        assert_eq!(scope.scoped_instance_count(), 0);

        let _: Arc<ScopedService> = scope.resolve().unwrap();
        assert_eq!(scope.scoped_instance_count(), 1);

        // Second resolve doesn't increase count (same instance)
        let _: Arc<ScopedService> = scope.resolve().unwrap();
        assert_eq!(scope.scoped_instance_count(), 1);
    }

    #[test]
    fn test_not_registered_in_scope() {
        let container = ServiceContainer::new();
        let scope = container.create_scope();

        let result: Result<Arc<ScopedService>, _> = scope.resolve();
        assert!(matches!(
            result,
            Err(ContainerError::ServiceNotRegistered(_))
        ));
    }
}
