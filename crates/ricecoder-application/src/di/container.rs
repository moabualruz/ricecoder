//! Service Container Implementation
//!
//! REQ-ARCH-003: DI Container with three lifetime semantics

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::error::ContainerError;
use super::scope::ScopedContainer;

/// Factory function type for creating service instances
type ServiceFactory = Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// Service Container with support for Singleton, Scoped, and Transient lifetimes
///
/// # Example
///
/// ```ignore
/// use ricecoder_application::di::ServiceContainer;
/// use std::sync::Arc;
///
/// let mut container = ServiceContainer::new();
///
/// // Register singleton (shared instance)
/// container.register_singleton(Arc::new(MyService::new()));
///
/// // Register transient (new instance each time)
/// container.register_transient(|| MyOtherService::new());
///
/// // Resolve services
/// let service: Arc<MyService> = container.resolve().unwrap();
/// ```
pub struct ServiceContainer {
    /// Singleton instances (created once, shared for lifetime)
    singletons: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,

    /// Scoped factories (create once per scope)
    scoped_factories: HashMap<TypeId, ServiceFactory>,

    /// Transient factories (create every time)
    transient_factories: HashMap<TypeId, ServiceFactory>,

    /// Trait object singletons (for dyn Trait resolution)
    trait_singletons: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceContainer {
    /// Create a new empty service container
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            scoped_factories: HashMap::new(),
            transient_factories: HashMap::new(),
            trait_singletons: RwLock::new(HashMap::new()),
        }
    }

    // ========================================================================
    // Singleton Registration
    // ========================================================================

    /// Register a singleton instance (created once, shared for application lifetime)
    ///
    /// Use for: Database pools, configuration, stateless services
    ///
    /// # Example
    /// ```ignore
    /// container.register_singleton(Arc::new(DatabasePool::new(config)));
    /// ```
    pub fn register_singleton<T: 'static + Send + Sync>(&mut self, instance: Arc<T>) {
        self.singletons.insert(TypeId::of::<T>(), instance);
    }

    /// Register a singleton trait object for dynamic dispatch
    ///
    /// Use when you need to resolve `Arc<dyn MyTrait>` instead of `Arc<MyConcreteType>`
    ///
    /// # Example
    /// ```ignore
    /// container.register_singleton_trait::<dyn Repository>(
    ///     Arc::new(SurrealRepository::new())
    /// );
    /// ```
    pub fn register_singleton_trait<T: ?Sized + 'static + Send + Sync>(
        &mut self,
        instance: Arc<T>,
    ) {
        let boxed: Arc<dyn Any + Send + Sync> = Arc::new(instance);
        self.trait_singletons
            .write()
            .unwrap()
            .insert(TypeId::of::<Arc<T>>(), boxed);
    }

    // ========================================================================
    // Scoped Registration
    // ========================================================================

    /// Register a scoped service factory (created once per scope/request)
    ///
    /// Use for: Request handlers, transaction contexts
    ///
    /// # Example
    /// ```ignore
    /// container.register_scoped(|| RequestContext::new());
    /// ```
    pub fn register_scoped<T, F>(&mut self, factory: F)
    where
        T: 'static + Send + Sync,
        F: Fn() -> T + 'static + Send + Sync,
    {
        let factory = Arc::new(move || Arc::new(factory()) as Arc<dyn Any + Send + Sync>);
        self.scoped_factories.insert(TypeId::of::<T>(), factory);
    }

    // ========================================================================
    // Transient Registration
    // ========================================================================

    /// Register a transient service factory (created every time resolved)
    ///
    /// Use for: Lightweight factories, builders
    ///
    /// # Example
    /// ```ignore
    /// container.register_transient(|| CommandBuilder::new());
    /// ```
    pub fn register_transient<T, F>(&mut self, factory: F)
    where
        T: 'static + Send + Sync,
        F: Fn() -> T + 'static + Send + Sync,
    {
        let factory = Arc::new(move || Arc::new(factory()) as Arc<dyn Any + Send + Sync>);
        self.transient_factories.insert(TypeId::of::<T>(), factory);
    }

    // ========================================================================
    // Resolution
    // ========================================================================

    /// Resolve a service by concrete type
    ///
    /// Resolution order: Singleton → Scoped (as transient) → Transient
    ///
    /// # Errors
    /// - `ServiceNotRegistered` if no service is registered for the type
    /// - `TypeMismatch` if the service cannot be downcast to the requested type
    pub fn resolve<T: 'static + Send + Sync>(&self) -> Result<Arc<T>, ContainerError> {
        // Try singleton first
        if let Some(instance) = self.singletons.get(&TypeId::of::<T>()) {
            return instance
                .clone()
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()));
        }

        // Try scoped factory (creates new instance without scope - use create_scope for proper scoping)
        if let Some(factory) = self.scoped_factories.get(&TypeId::of::<T>()) {
            return factory()
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()));
        }

        // Try transient factory
        if let Some(factory) = self.transient_factories.get(&TypeId::of::<T>()) {
            return factory()
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()));
        }

        Err(ContainerError::ServiceNotRegistered(std::any::type_name::<
            T,
        >()))
    }

    /// Resolve a trait object by type
    ///
    /// Use this when you registered with `register_singleton_trait`
    pub fn resolve_trait<T: ?Sized + 'static + Send + Sync>(
        &self,
    ) -> Result<Arc<T>, ContainerError> {
        let guard = self.trait_singletons.read().unwrap();
        if let Some(instance) = guard.get(&TypeId::of::<Arc<T>>()) {
            return instance
                .clone()
                .downcast::<Arc<T>>()
                .map(|arc| (*arc).clone())
                .map_err(|_| ContainerError::TypeMismatch(std::any::type_name::<T>()));
        }

        Err(ContainerError::ServiceNotRegistered(std::any::type_name::<
            T,
        >()))
    }

    // ========================================================================
    // Scope Management
    // ========================================================================

    /// Create a new scope for scoped service resolution
    ///
    /// Scoped services are created once per scope and shared within that scope.
    ///
    /// # Example
    /// ```ignore
    /// let scope = container.create_scope();
    /// let ctx1: Arc<RequestContext> = scope.resolve().unwrap();
    /// let ctx2: Arc<RequestContext> = scope.resolve().unwrap();
    /// // ctx1 and ctx2 are the same instance within this scope
    /// ```
    pub fn create_scope(&self) -> ScopedContainer<'_> {
        ScopedContainer::new(self)
    }

    // ========================================================================
    // Introspection
    // ========================================================================

    /// Check if a service is registered for the given type
    pub fn is_registered<T: 'static>(&self) -> bool {
        self.singletons.contains_key(&TypeId::of::<T>())
            || self.scoped_factories.contains_key(&TypeId::of::<T>())
            || self.transient_factories.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of registered services
    pub fn service_count(&self) -> usize {
        self.singletons.len()
            + self.scoped_factories.len()
            + self.transient_factories.len()
            + self.trait_singletons.read().unwrap().len()
    }

    // ========================================================================
    // Internal Access (for ScopedContainer)
    // ========================================================================

    pub(super) fn get_scoped_factory(&self, type_id: &TypeId) -> Option<&ServiceFactory> {
        self.scoped_factories.get(type_id)
    }

    pub(super) fn get_singleton(&self, type_id: &TypeId) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.singletons.get(type_id)
    }

    pub(super) fn get_transient_factory(&self, type_id: &TypeId) -> Option<&ServiceFactory> {
        self.transient_factories.get(type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingService {
        id: usize,
    }

    impl CountingService {
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
    fn test_singleton_same_instance() {
        let mut container = ServiceContainer::new();

        container.register_singleton(Arc::new(CountingService { id: 42 }));

        let s1: Arc<CountingService> = container.resolve().unwrap();
        let s2: Arc<CountingService> = container.resolve().unwrap();

        assert_eq!(s1.id(), s2.id());
        assert!(Arc::ptr_eq(&s1, &s2));
    }

    #[test]
    fn test_transient_new_instance() {
        let mut container = ServiceContainer::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        container.register_transient(move || CountingService::new(&counter_clone));

        let s1: Arc<CountingService> = container.resolve().unwrap();
        let s2: Arc<CountingService> = container.resolve().unwrap();

        assert_ne!(s1.id(), s2.id());
        assert!(!Arc::ptr_eq(&s1, &s2));
    }

    #[test]
    fn test_not_registered() {
        let container = ServiceContainer::new();

        let result: Result<Arc<CountingService>, _> = container.resolve();

        assert!(matches!(
            result,
            Err(ContainerError::ServiceNotRegistered(_))
        ));
    }

    #[test]
    fn test_is_registered() {
        let mut container = ServiceContainer::new();

        assert!(!container.is_registered::<CountingService>());

        container.register_singleton(Arc::new(CountingService { id: 1 }));

        assert!(container.is_registered::<CountingService>());
    }

    #[test]
    fn test_service_count() {
        let mut container = ServiceContainer::new();

        assert_eq!(container.service_count(), 0);

        container.register_singleton(Arc::new(CountingService { id: 1 }));
        assert_eq!(container.service_count(), 1);

        container.register_transient(|| CountingService { id: 2 });
        assert_eq!(container.service_count(), 2);
    }
}
