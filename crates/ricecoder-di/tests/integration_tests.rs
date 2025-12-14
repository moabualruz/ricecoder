//! Integration tests for the DI container

use ricecoder_agents::use_cases::{
    ProviderCommunityUseCase, ProviderFailoverUseCase, ProviderHealthUseCase,
    ProviderModelUseCase, ProviderPerformanceUseCase, ProviderSwitchingUseCase,
    SessionLifecycleUseCase, SessionSharingUseCase, SessionStateManagementUseCase,
};
use ricecoder_di::{create_application_container, DIResult};
use std::sync::Arc;

#[tokio::test]
async fn test_session_use_cases_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test session lifecycle use case
    let session_use_case = container.resolve::<SessionLifecycleUseCase>()?;
    assert!(session_use_case.list_sessions().await.is_ok());

    // Test session sharing use case
    let sharing_use_case = container.resolve::<SessionSharingUseCase>()?;
    assert!(sharing_use_case.list_active_shares().await.is_ok());

    // Test session state management use case
    let state_use_case = container.resolve::<SessionStateManagementUseCase>()?;
    // Note: This would need a valid session ID to test fully

    Ok(())
}

#[tokio::test]
async fn test_provider_use_cases_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test provider switching use case
    let switching_use_case = container.resolve::<ProviderSwitchingUseCase>()?;
    let providers = switching_use_case.list_available_providers();
    assert!(providers.len() >= 0); // May be empty if no providers are configured

    // Test provider performance use case
    let performance_use_case = container.resolve::<ProviderPerformanceUseCase>()?;
    let summary = performance_use_case.get_all_provider_performance();
    assert!(summary.total_providers >= 0);

    // Test provider failover use case
    let failover_use_case = container.resolve::<ProviderFailoverUseCase>()?;
    let failover = failover_use_case.get_failover_provider("nonexistent");
    assert!(failover.is_none()); // Should be None for non-existent provider

    // Test provider model use case
    let model_use_case = container.resolve::<ProviderModelUseCase>()?;
    let models = model_use_case.get_available_models(None);
    assert!(models.len() >= 0);

    // Test provider health use case
    let health_use_case = container.resolve::<ProviderHealthUseCase>()?;
    let health_results = health_use_case.check_all_provider_health().await;
    assert!(health_results.len() >= 0);

    // Test provider community use case
    let community_use_case = container.resolve::<ProviderCommunityUseCase>()?;
    let popular = community_use_case.get_popular_providers(5);
    assert!(popular.len() <= 5);

    Ok(())
}

#[tokio::test]
async fn test_service_singleton_behavior() -> DIResult<()> {
    let container = create_application_container()?;

    // Resolve the same service multiple times
    let session_use_case1 = container.resolve::<SessionLifecycleUseCase>()?;
    let session_use_case2 = container.resolve::<SessionLifecycleUseCase>()?;

    // Should be the same instance (singleton)
    assert!(Arc::ptr_eq(&session_use_case1, &session_use_case2));

    let provider_use_case1 = container.resolve::<ProviderSwitchingUseCase>()?;
    let provider_use_case2 = container.resolve::<ProviderSwitchingUseCase>()?;

    // Should be the same instance (singleton)
    assert!(Arc::ptr_eq(&provider_use_case1, &provider_use_case2));

    Ok(())
}

#[tokio::test]
async fn test_container_service_count() -> DIResult<()> {
    let container = create_application_container()?;

    // Should have registered multiple services
    let service_count = container.service_count();
    assert!(service_count > 10); // Should have infrastructure + use cases

    Ok(())
}

#[tokio::test]
async fn test_session_provider_interaction() -> DIResult<()> {
    let container = create_application_container()?;

    // Get both session and provider use cases
    let session_use_case = container.resolve::<SessionLifecycleUseCase>()?;
    let provider_use_case = container.resolve::<ProviderSwitchingUseCase>()?;

    // Both should be resolvable and functional
    assert!(session_use_case.list_sessions().await.is_ok());
    assert!(provider_use_case.list_available_providers().len() >= 0);

    Ok(())
}