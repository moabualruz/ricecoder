//! Integration tests for ricecoder-providers
//!
//! Tests provider manager, registry, and cross-module functionality.
//! Per R2: Tests should be in tests/ directory.

use std::sync::Arc;
use std::time::Duration;

use ricecoder_providers::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, ProviderManager, ProviderRegistry,
};

/// Test provider manager creation and basic functionality
#[test]
fn test_provider_manager_creation() {
    let registry = ProviderRegistry::new();
    let manager = ProviderManager::new(registry, "default".to_string());

    // Manager should be created with empty registry
    assert!(manager.registry().list_all().is_empty());
}

/// Test provider manager with retry configuration
#[test]
fn test_provider_manager_retry_config() {
    let registry = ProviderRegistry::new();
    let manager = ProviderManager::new(registry, "default".to_string())
        .with_retry_count(5)
        .with_timeout(Duration::from_secs(60));

    // Manager should be configured
    assert!(manager.registry().list_all().is_empty());
}

/// Test provider registry operations
#[test]
fn test_provider_registry_operations() {
    let registry = ProviderRegistry::new();

    // Empty registry should report correct state
    assert_eq!(registry.provider_count(), 0);
    assert!(registry.list_all().is_empty());
    assert!(!registry.has_provider("test"));

    // Get non-existent provider should fail
    assert!(registry.get("non_existent").is_err());
}

/// Test circuit breaker state transitions
#[test]
fn test_circuit_breaker_integration() {
    let config = CircuitBreakerConfig::default()
        .with_failure_threshold(3)
        .with_recovery_timeout(Duration::from_millis(10));

    let cb = CircuitBreaker::new("test_provider", config);

    // Initial state should be closed
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.can_execute());

    // After failures, circuit should open
    cb.record_failure();
    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.can_execute());

    // Wait for recovery timeout
    std::thread::sleep(Duration::from_millis(20));

    // Should be half-open after timeout
    assert!(cb.can_execute()); // This transitions to half-open

    // Success should close circuit
    cb.record_success();
    cb.record_success();
    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
}

/// Test circuit breaker reset functionality
#[test]
fn test_circuit_breaker_reset() {
    let config = CircuitBreakerConfig::default().with_failure_threshold(2);
    let cb = CircuitBreaker::new("test", config);

    // Open the circuit
    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);

    // Reset should restore to closed
    cb.reset();
    assert_eq!(cb.state(), CircuitState::Closed);
    assert_eq!(cb.failure_count(), 0);
}

/// Test force open/close on circuit breaker
#[test]
fn test_circuit_breaker_force_operations() {
    let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());

    // Force open
    cb.force_open();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.can_execute());

    // Force close
    cb.force_close();
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.can_execute());
}
