//! Unit tests for the DI container core functionality
use std::sync::Arc;

use ricecoder_di::*;

#[derive(Debug, PartialEq)]
struct TestService {
    value: i32,
}

#[test]
fn test_register_and_resolve_singleton() {
    let container = DIContainer::new();

    // Register a service
    container
        .register(|_| Ok(Arc::new(TestService { value: 42 })))
        .unwrap();

    // Resolve the service
    let service1 = container.resolve::<TestService>().unwrap();
    let service2 = container.resolve::<TestService>().unwrap();

    // Should be the same instance (singleton)
    assert_eq!(service1.value, 42);
    assert_eq!(service2.value, 42);
    assert!(Arc::ptr_eq(&service1, &service2));
}

#[test]
fn test_register_transient() {
    let container = DIContainer::new();

    // Register a transient service
    container
        .register_transient(|_| Ok(Arc::new(TestService { value: 42 })))
        .unwrap();

    // Resolve the service multiple times
    let service1 = container.resolve::<TestService>().unwrap();
    let service2 = container.resolve::<TestService>().unwrap();

    // Should be different instances (transient)
    assert_eq!(service1.value, 42);
    assert_eq!(service2.value, 42);
    assert!(!Arc::ptr_eq(&service1, &service2));
}

#[test]
fn test_service_not_registered() {
    let container = DIContainer::new();

    let result = container.resolve::<TestService>();
    assert!(matches!(result, Err(DIError::ServiceNotRegistered { .. })));
}

#[test]
fn test_service_already_registered() {
    let container = DIContainer::new();

    // Register once
    container
        .register(|_| Ok(Arc::new(TestService { value: 42 })))
        .unwrap();

    // Try to register again
    let result = container.register(|_| Ok(Arc::new(TestService { value: 24 })));
    assert!(matches!(
        result,
        Err(DIError::ServiceAlreadyRegistered { .. })
    ));
}

#[test]
fn test_builder_pattern() {
    let container = DIContainerBuilder::new()
        .register(|_| Ok(Arc::new(TestService { value: 42 })))
        .unwrap()
        .build();

    let service = container.resolve::<TestService>().unwrap();
    assert_eq!(service.value, 42);
}
