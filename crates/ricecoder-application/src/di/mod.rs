//! Dependency Injection Container
//!
//! Provides a type-safe service container with support for three lifetime semantics:
//! - **Singleton**: Created once, shared for application lifetime
//! - **Scoped**: Created once per scope (request/session)
//! - **Transient**: Created every time resolved
//!
//! REQ-ARCH-003: DI Container Architecture
//! AC-3.1: Use DI container for ALL service resolution
//! AC-3.2: Register services with explicit lifetimes
//! AC-3.6: Type-safe service resolution

mod container;
mod error;
mod scope;

pub use container::ServiceContainer;
pub use error::ContainerError;
pub use scope::ScopedContainer;

/// Service lifetime semantics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    /// Created once at registration, shared for application lifetime
    /// Use for: Database pools, configuration, stateless services
    Singleton,
    
    /// Created once per scope (request/session)
    /// Use for: Request handlers, transaction contexts
    Scoped,
    
    /// Created every time resolved
    /// Use for: Lightweight factories, builders
    Transient,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_singleton_created_once() {
        let mut container = ServiceContainer::new();
        
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        container.register_singleton(Arc::new(TestService::new(counter_clone)));
        
        let _s1: Arc<TestService> = container.resolve().unwrap();
        let _s2: Arc<TestService> = container.resolve().unwrap();
        let _s3: Arc<TestService> = container.resolve().unwrap();
        
        // Should only create once for singleton
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_transient_created_every_time() {
        let mut container = ServiceContainer::new();
        
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        container.register_transient(move || {
            TestService::new(Arc::clone(&counter_clone))
        });
        
        let _s1: Arc<TestService> = container.resolve().unwrap();
        let _s2: Arc<TestService> = container.resolve().unwrap();
        let _s3: Arc<TestService> = container.resolve().unwrap();
        
        // Should create each time for transient
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_scoped_created_once_per_scope() {
        let mut container = ServiceContainer::new();
        
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        container.register_scoped(move || {
            TestService::new(Arc::clone(&counter_clone))
        });
        
        // First scope
        {
            let scope = container.create_scope();
            let _s1: Arc<TestService> = scope.resolve().unwrap();
            let _s2: Arc<TestService> = scope.resolve().unwrap();
        }
        
        // Second scope
        {
            let scope = container.create_scope();
            let _s3: Arc<TestService> = scope.resolve().unwrap();
        }
        
        // Should create once per scope (2 scopes = 2 creations)
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_resolve_not_registered() {
        let container = ServiceContainer::new();
        
        let result: Result<Arc<TestService>, _> = container.resolve();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContainerError::ServiceNotRegistered(_)));
    }

    #[test]
    fn test_resolve_trait_object() {
        let mut container = ServiceContainer::new();
        
        container.register_singleton_trait::<dyn TestTrait>(
            Arc::new(TestTraitImpl { value: 42 })
        );
        
        let service: Arc<dyn TestTrait> = container.resolve_trait().unwrap();
        assert_eq!(service.get_value(), 42);
    }

    // Test helpers
    #[derive(Debug)]
    struct TestService {
        _id: usize,
    }

    impl TestService {
        fn new(counter: Arc<AtomicUsize>) -> Self {
            let id = counter.fetch_add(1, Ordering::SeqCst) + 1;
            Self { _id: id }
        }
    }

    trait TestTrait: Send + Sync {
        fn get_value(&self) -> i32;
    }

    struct TestTraitImpl {
        value: i32,
    }

    impl TestTrait for TestTraitImpl {
        fn get_value(&self) -> i32 {
            self.value
        }
    }
}
