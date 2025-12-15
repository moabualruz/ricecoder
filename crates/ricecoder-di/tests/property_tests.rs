//! Property-based tests for service resolution and DI container behavior

use proptest::prelude::*;
use ricecoder_di::*;
use std::sync::Arc;

/// Test service for property-based testing
#[derive(Debug, Clone, PartialEq)]
struct TestService {
    value: i32,
    data: Vec<u8>,
}

impl TestService {
    fn new(value: i32, data: Vec<u8>) -> Self {
        Self { value, data }
    }
}

/// Generate arbitrary test services
fn arb_test_service() -> impl Strategy<Value = TestService> {
    (any::<i32>(), prop::collection::vec(any::<u8>(), 0..1000)).prop_map(|(value, data)| TestService::new(value, data))
}

/// Test that singleton services always return the same instance
proptest! {
    #[test]
    fn test_singleton_resolution_consistency(service in arb_test_service()) {
        let container = DIContainer::new();

        // Register singleton service
        container.register(move |_| {
            let svc = Arc::new(service.clone());
            Ok(svc)
        }).unwrap();

        // Resolve multiple times
        let resolved1 = container.resolve::<TestService>().unwrap();
        let resolved2 = container.resolve::<TestService>().unwrap();
        let resolved3 = container.resolve::<TestService>().unwrap();

        // All should be the same instance
        prop_assert!(Arc::ptr_eq(&resolved1, &resolved2));
        prop_assert!(Arc::ptr_eq(&resolved2, &resolved3));
        prop_assert!(Arc::ptr_eq(&resolved1, &resolved3));

        // Values should match
        prop_assert_eq!(resolved1.value, service.value);
        prop_assert_eq!(resolved1.data, service.data);
    }
}

/// Test that transient services return different instances
proptest! {
    #[test]
    fn test_transient_resolution_uniqueness(service in arb_test_service()) {
        let container = DIContainer::new();

        // Register transient service
        container.register_transient(move |_| {
            let svc = Arc::new(service.clone());
            Ok(svc)
        }).unwrap();

        // Resolve multiple times
        let resolved1 = container.resolve::<TestService>().unwrap();
        let resolved2 = container.resolve::<TestService>().unwrap();
        let resolved3 = container.resolve::<TestService>().unwrap();

        // All should be different instances
        prop_assert!(!Arc::ptr_eq(&resolved1, &resolved2));
        prop_assert!(!Arc::ptr_eq(&resolved2, &resolved3));
        prop_assert!(!Arc::ptr_eq(&resolved1, &resolved3));

        // But values should match
        prop_assert_eq!(resolved1.value, service.value);
        prop_assert_eq!(resolved1.data, service.data);
        prop_assert_eq!(resolved2.value, service.value);
        prop_assert_eq!(resolved2.data, service.data);
        prop_assert_eq!(resolved3.value, service.value);
        prop_assert_eq!(resolved3.data, service.data);
    }
}

/// Test scoped service resolution behavior
proptest! {
    #[test]
    fn test_scoped_resolution_behavior(service in arb_test_service()) {
        let container = DIContainer::new();

        // Register scoped service
        container.register_scoped(move |_| {
            let svc = Arc::new(service.clone());
            Ok(svc)
        }).unwrap();

        let scope1 = ServiceScope::new();
        let scope2 = ServiceScope::new();

        // Resolve in different scopes
        let resolved1_scope1 = container.resolve_with_scope(Some(&scope1)).unwrap();
        let resolved2_scope1 = container.resolve_with_scope(Some(&scope1)).unwrap();
        let resolved1_scope2 = container.resolve_with_scope(Some(&scope2)).unwrap();

        // Same scope should return same instance
        prop_assert!(Arc::ptr_eq(&resolved1_scope1, &resolved2_scope1));

        // Different scopes should return different instances
        prop_assert!(!Arc::ptr_eq(&resolved1_scope1, &resolved1_scope2));

        // Values should match
        prop_assert_eq!(resolved1_scope1.value, service.value);
        prop_assert_eq!(resolved1_scope1.data, service.data);
    }
}

/// Test that scoped services fail without a scope
proptest! {
    #[test]
    fn test_scoped_requires_scope(service in arb_test_service()) {
        let container = DIContainer::new();

        // Register scoped service
        container.register_scoped(move |_| {
            let svc = Arc::new(service.clone());
            Ok(svc)
        }).unwrap();

        // Should fail without scope
        let result = container.resolve::<TestService>();
        prop_assert!(matches!(result, Err(DIError::InvalidServiceType { .. })));
    }
}

/// Test service registration and resolution with various service counts
proptest! {
    #[test]
    fn test_multiple_service_registration(service_count in 1..50usize) {
        let container = DIContainer::new();

        // Register multiple different services
        for i in 0..service_count {
            let value = i as i32;
            container.register(move |_| {
                let svc = Arc::new(TestService::new(value, vec![value as u8]));
                Ok(svc)
            }).unwrap();
        }

        // Should have registered the expected number of services
        prop_assert_eq!(container.service_count(), service_count);

        // All services should be resolvable
        for i in 0..service_count {
            let resolved = container.resolve::<TestService>().unwrap();
            prop_assert_eq!(resolved.value, (service_count - 1) as i32); // Last registered service
        }
    }
}

/// Test that service resolution is thread-safe
proptest! {
    #[test]
    fn test_thread_safe_resolution(service in arb_test_service()) {
        use std::thread;
        use std::sync::mpsc;

        let container = Arc::new(DIContainer::new());

        // Register singleton service
        container.register({
            let svc = service.clone();
            move |_| {
                let service = Arc::new(svc.clone());
                Ok(service)
            }
        }).unwrap();

        let (tx, rx) = mpsc::channel();
        let thread_count = 10;

        // Spawn multiple threads that resolve the service
        for _ in 0..thread_count {
            let container = Arc::clone(&container);
            let tx = tx.clone();

            thread::spawn(move || {
                let resolved = container.resolve::<TestService>().unwrap();
                tx.send(resolved).unwrap();
            });
        }

        // Collect results from all threads
        let mut results = vec![];
        for _ in 0..thread_count {
            results.push(rx.recv().unwrap());
        }

        // All should be the same instance (singleton behavior)
        for i in 1..results.len() {
            prop_assert!(Arc::ptr_eq(&results[0], &results[i]));
        }

        // Values should match
        for result in &results {
            prop_assert_eq!(result.value, service.value);
            prop_assert_eq!(result.data, service.data);
        }
    }
}

/// Test error handling with invalid service types
proptest! {
    #[test]
    fn test_service_resolution_errors(service in arb_test_service()) {
        let container = DIContainer::new();

        // Don't register any services

        // Should fail to resolve
        let result = container.resolve::<TestService>();
        prop_assert!(matches!(result, Err(DIError::ServiceNotRegistered { .. })));

        // Register one service
        container.register(move |_| {
            let svc = Arc::new(service.clone());
            Ok(svc)
        }).unwrap();

        // Now should succeed
        let resolved = container.resolve::<TestService>().unwrap();
        prop_assert_eq!(resolved.value, service.value);
        prop_assert_eq!(resolved.data, service.data);
    }
}

/// Test builder pattern with property-based inputs
proptest! {
    #[test]
    fn test_builder_pattern_property(service in arb_test_service()) {
        let container = DIContainerBuilder::new()
            .register({
                let svc = service.clone();
                move |_| {
                    let service = Arc::new(svc.clone());
                    Ok(service)
                }
            })
            .unwrap()
            .register_transient({
                let svc = service.clone();
                move |_| {
                    let service = Arc::new(svc.clone());
                    Ok(service)
                }
            })
            .unwrap()
            .build();

        // Should be able to resolve both services
        let singleton1 = container.resolve::<TestService>().unwrap();
        let singleton2 = container.resolve::<TestService>().unwrap();
        let transient1 = container.resolve::<TestService>().unwrap();
        let transient2 = container.resolve::<TestService>().unwrap();

        // Singleton should be same instance
        prop_assert!(Arc::ptr_eq(&singleton1, &singleton2));

        // Transient should be different instances
        prop_assert!(!Arc::ptr_eq(&transient1, &transient2));

        // Values should match
        prop_assert_eq!(singleton1.value, service.value);
        prop_assert_eq!(singleton1.data, service.data);
    }
}