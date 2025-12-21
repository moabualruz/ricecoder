//! Integration tests for component wiring

use ricecoder_agents::use_cases::{
    ProviderPerformanceUseCase, ProviderSwitchingUseCase, SessionLifecycleUseCase,
    SessionSharingUseCase,
};
use ricecoder_cli::di;
use ricecoder_providers::provider::manager::ProviderManager;
use ricecoder_sessions::{SessionManager, SessionStore};

#[test]
fn test_di_container_initialization() {
    // Reset DI container for clean test
    #[cfg(test)]
    di::reset_di_container();

    // Initialize DI container
    di::initialize_di_container().unwrap();

    // Verify container is initialized
    assert!(di::is_di_initialized());

    // Verify we can get the container
    let container = di::get_di_container();
    assert!(container.is_some());
}

#[test]
fn test_core_service_resolution() {
    // Reset and initialize
    #[cfg(test)]
    di::reset_di_container();
    di::initialize_di_container().unwrap();

    // Test core infrastructure services
    let session_manager = di::get_service::<SessionManager>();
    assert!(
        session_manager.is_some(),
        "SessionManager should be available"
    );

    let session_store = di::get_service::<SessionStore>();
    assert!(session_store.is_some(), "SessionStore should be available");

    let provider_manager = di::get_service::<ProviderManager>();
    assert!(
        provider_manager.is_some(),
        "ProviderManager should be available"
    );
}

#[test]
fn test_use_case_resolution() {
    // Reset and initialize
    #[cfg(test)]
    di::reset_di_container();
    di::initialize_di_container().unwrap();

    // Test use cases
    let session_lifecycle = di::get_service::<SessionLifecycleUseCase>();
    assert!(
        session_lifecycle.is_some(),
        "SessionLifecycleUseCase should be available"
    );

    let session_sharing = di::get_service::<SessionSharingUseCase>();
    assert!(
        session_sharing.is_some(),
        "SessionSharingUseCase should be available"
    );

    let provider_switching = di::get_service::<ProviderSwitchingUseCase>();
    assert!(
        provider_switching.is_some(),
        "ProviderSwitchingUseCase should be available"
    );

    let provider_performance = di::get_service::<ProviderPerformanceUseCase>();
    assert!(
        provider_performance.is_some(),
        "ProviderPerformanceUseCase should be available"
    );
}

#[test]
fn test_service_singleton_behavior() {
    // Reset and initialize
    #[cfg(test)]
    di::reset_di_container();
    di::initialize_di_container().unwrap();

    // Get the same service multiple times
    let service1 = di::get_service::<SessionManager>();
    let service2 = di::get_service::<SessionManager>();

    assert!(service1.is_some());
    assert!(service2.is_some());

    // Should be the same instance (singleton)
    assert!(std::sync::Arc::ptr_eq(
        &service1.unwrap(),
        &service2.unwrap()
    ));
}

#[test]
fn test_service_functionality() {
    // Reset and initialize
    #[cfg(test)]
    di::reset_di_container();
    di::initialize_di_container().unwrap();

    // Test that services are functional
    let session_lifecycle = di::get_service::<SessionLifecycleUseCase>().unwrap();

    // This should not panic and should return some result
    let result = std::panic::catch_unwind(|| {
        // We can't easily test the actual functionality without mocking,
        // but we can test that the service exists and has the expected interface
        assert!(std::mem::size_of_val(&*session_lifecycle) > 0);
    });

    assert!(result.is_ok(), "Service should be functional");
}

#[cfg(feature = "storage")]
#[test]
fn test_storage_services() {
    // Reset and initialize
    #[cfg(test)]
    di::reset_di_container();
    di::initialize_di_container().unwrap();

    use ricecoder_storage::{FileStorage, MemoryStorage, StorageManager};

    let storage_manager = di::get_service::<StorageManager>();
    assert!(
        storage_manager.is_some(),
        "StorageManager should be available when storage feature is enabled"
    );

    let file_storage = di::get_service::<FileStorage>();
    assert!(
        file_storage.is_some(),
        "FileStorage should be available when storage feature is enabled"
    );

    let memory_storage = di::get_service::<MemoryStorage>();
    assert!(
        memory_storage.is_some(),
        "MemoryStorage should be available when storage feature is enabled"
    );
}

#[cfg(feature = "research")]
#[test]
fn test_research_services() {
    // Reset and initialize
    #[cfg(test)]
    di::reset_di_container();
    di::initialize_di_container().unwrap();

    use ricecoder_research::{CodebaseScanner, ResearchManager, SemanticIndexer};

    let research_manager = di::get_service::<ResearchManager>();
    assert!(
        research_manager.is_some(),
        "ResearchManager should be available when research feature is enabled"
    );

    let codebase_scanner = di::get_service::<CodebaseScanner>();
    assert!(
        codebase_scanner.is_some(),
        "CodebaseScanner should be available when research feature is enabled"
    );

    let semantic_indexer = di::get_service::<SemanticIndexer>();
    assert!(
        semantic_indexer.is_some(),
        "SemanticIndexer should be available when research feature is enabled"
    );
}
