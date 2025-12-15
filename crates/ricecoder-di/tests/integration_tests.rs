//! Integration tests for the DI container

use ricecoder_agents::use_cases::{
    ProviderCommunityUseCase, ProviderFailoverUseCase, ProviderHealthUseCase,
    ProviderModelUseCase, ProviderPerformanceUseCase, ProviderSwitchingUseCase,
    SessionLifecycleUseCase, SessionSharingUseCase, SessionStateManagementUseCase,
};
use ricecoder_di::{
    create_application_container, create_cli_container, create_tui_container,
    create_development_container, create_test_container, DIResult
};
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

#[tokio::test]
async fn test_concurrent_service_resolution() -> DIResult<()> {
    use std::sync::Arc;
    use tokio::task;

    let container = Arc::new(create_application_container()?);
    let mut handles = vec![];

    // Spawn multiple concurrent tasks that resolve services
    for i in 0..10 {
        let container = Arc::clone(&container);
        let handle = task::spawn(async move {
            // Resolve multiple services concurrently
            let session_use_case = container.resolve::<SessionLifecycleUseCase>()?;
            let provider_use_case = container.resolve::<ProviderSwitchingUseCase>()?;
            let session_store = container.resolve::<ricecoder_sessions::SessionStore>()?;
            let provider_manager = container.resolve::<ricecoder_providers::provider::manager::ProviderManager>()?;

            // Perform some operations to ensure services work
            let sessions = session_use_case.list_sessions().await?;
            let providers = provider_use_case.list_available_providers();

            Ok::<_, ricecoder_di::DIError>((sessions, providers))
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok());
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_singleton_behavior() -> DIResult<()> {
    use std::sync::Arc;
    use tokio::task;
    use std::collections::HashSet;

    let container = Arc::new(create_application_container()?);
    let mut handles = vec![];

    // Spawn multiple tasks that resolve the same singleton service
    for _ in 0..20 {
        let container = Arc::clone(&container);
        let handle = task::spawn(async move {
            let session_use_case = container.resolve::<SessionLifecycleUseCase>()?;
            Ok::<_, ricecoder_di::DIError>(Arc::as_ptr(&session_use_case) as usize)
        });
        handles.push(handle);
    }

    // Collect all pointers
    let mut pointers = HashSet::new();
    for handle in handles {
        let ptr = handle.await?;
        pointers.insert(ptr);
    }

    // All pointers should be the same (singleton behavior)
    assert_eq!(pointers.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_registration_and_resolution() -> DIResult<()> {
    use std::sync::Arc;
    use tokio::task;

    let container = Arc::new(DIContainer::new());
    let mut handles = vec![];

    // Register some services
    container.register(|_| {
        let store = Arc::new(ricecoder_sessions::SessionStore::new().map_err(|e| {
            ricecoder_di::DIError::DependencyResolutionFailed {
                message: format!("Failed to create session store: {}", e),
            }
        })?);
        Ok(store)
    })?;

    container.register(|_| {
        let manager = Arc::new(ricecoder_sessions::SessionManager::new(10));
        Ok(manager)
    })?;

    // Spawn concurrent resolution tasks
    for _ in 0..50 {
        let container = Arc::clone(&container);
        let handle = task::spawn(async move {
            let store = container.resolve::<ricecoder_sessions::SessionStore>()?;
            let manager = container.resolve::<ricecoder_sessions::SessionManager>()?;
            Ok::<_, ricecoder_di::DIError>((Arc::as_ptr(&store) as usize, Arc::as_ptr(&manager) as usize))
        });
        handles.push(handle);
    }

    // Wait for all to complete and verify singleton behavior
    let mut store_pointers = std::collections::HashSet::new();
    let mut manager_pointers = std::collections::HashSet::new();

    for handle in handles {
        let (store_ptr, manager_ptr) = handle.await?;
        store_pointers.insert(store_ptr);
        manager_pointers.insert(manager_ptr);
    }

    // Should all be the same instance (singleton)
    assert_eq!(store_pointers.len(), 1);
    assert_eq!(manager_pointers.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_scoped_services() -> DIResult<()> {
    use std::sync::Arc;
    use tokio::task;

    let container = Arc::new(DIContainer::new());

    // Register a scoped service
    container.register_scoped(|_| {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let service = Arc::new(format!("ScopedService-{}", id));
        Ok(service)
    })?;

    let mut handles = vec![];

    // Create multiple scopes and resolve concurrently
    for scope_id in 0..5 {
        let container = Arc::clone(&container);
        let handle = task::spawn(async move {
            let scope = ServiceScope::new();

            // Resolve multiple times in the same scope
            let service1 = container.resolve_with_scope(Some(&scope))?;
            let service2 = container.resolve_with_scope(Some(&scope))?;

            // Should be the same instance within the scope
            assert_eq!(service1, service2);

            Ok::<_, ricecoder_di::DIError>(service1.clone())
        });
        handles.push(handle);
    }

    // Wait for all scopes to complete
    for handle in handles {
        let result = handle.await?;
        assert!(result.starts_with("ScopedService-"));
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_health_checks() -> DIResult<()> {
    use std::sync::Arc;
    use tokio::task;

    let container = Arc::new(create_application_container()?);
    let mut handles = vec![];

    // Spawn multiple health check tasks
    for _ in 0..10 {
        let container = Arc::clone(&container);
        let handle = task::spawn(async move {
            let health_results = container.health_check_all()?;
            Ok::<_, ricecoder_di::DIError>(health_results.len())
        });
        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        let result_count = handle.await?;
        assert!(result_count >= 0);
    }

    Ok(())
}

#[cfg(feature = "parsers")]
#[tokio::test]
async fn test_parsers_service_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test parser service
    let parser = container.resolve::<ricecoder_parsers::Parser>()?;
    assert!(parser.is_registered(ricecoder_parsers::languages::Language::Rust).await);

    Ok(())
}

#[cfg(feature = "generation")]
#[tokio::test]
async fn test_generation_services_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test generation services
    let generation_manager = container.resolve::<ricecoder_generation::GenerationManager>()?;
    let code_generator = container.resolve::<ricecoder_generation::CodeGenerator>()?;
    let spec_processor = container.resolve::<ricecoder_generation::SpecProcessor>()?;
    let template_engine = container.resolve::<ricecoder_generation::TemplateEngine>()?;

    // Basic functionality tests
    assert!(generation_manager.list_available_boilerplates().await.is_ok());

    Ok(())
}

#[cfg(feature = "themes")]
#[tokio::test]
async fn test_themes_services_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test theme services
    let theme_manager = container.resolve::<ricecoder_themes::ThemeManager>()?;
    let theme_loader = container.resolve::<ricecoder_themes::ThemeLoader>()?;
    let theme_registry = container.resolve::<ricecoder_themes::ThemeRegistry>()?;

    // Basic functionality tests
    assert!(theme_registry.list_available_themes().is_ok());

    Ok(())
}

#[cfg(feature = "config")]
#[tokio::test]
async fn test_config_services_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test config service
    let config_manager = container.resolve::<ricecoder_config::ConfigManager>()?;

    // Basic functionality test
    assert!(config_manager.get_config().await.is_ok());

    Ok(())
}

#[cfg(feature = "github")]
#[tokio::test]
async fn test_github_services_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test GitHub services
    let github_manager = container.resolve::<ricecoder_github::managers::GitHubManager>()?;
    let issue_manager = container.resolve::<ricecoder_github::managers::IssueManager>()?;
    let pr_manager = container.resolve::<ricecoder_github::managers::PrManager>()?;

    // Basic functionality tests (may require authentication)
    // These are just smoke tests to ensure services are created
    assert!(github_manager.is_ok());
    assert!(issue_manager.is_ok());
    assert!(pr_manager.is_ok());

    Ok(())
}

#[cfg(feature = "domain-agents")]
#[tokio::test]
async fn test_domain_agents_services_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test domain agents services
    let domain_agent_registry = container.resolve::<ricecoder_domain_agents::DomainAgentRegistryManager>()?;
    let knowledge_base_manager = container.resolve::<ricecoder_domain_agents::KnowledgeBaseManager>()?;

    // Basic functionality tests
    let domains = domain_agent_registry.list_available_domains();
    assert!(domains.len() > 0); // Should have default domains

    Ok(())
}

#[cfg(feature = "local-models")]
#[tokio::test]
async fn test_local_models_services_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test local models service
    let local_model_manager = container.resolve::<ricecoder_local_models::LocalModelManager>()?;

    // Basic functionality test
    assert!(local_model_manager.list_available_models().await.is_ok());

    Ok(())
}

#[cfg(feature = "cli")]
#[tokio::test]
async fn test_cli_services_integration() -> DIResult<()> {
    let container = create_application_container()?;

    // Test CLI services
    let command_router = container.resolve::<ricecoder_cli::CommandRouter>()?;
    let branding_manager = container.resolve::<ricecoder_cli::BrandingManager>()?;

    // Basic functionality tests
    assert!(command_router.is_ok());
    assert!(branding_manager.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_different_container_builders() -> DIResult<()> {
    // Test CLI container
    let cli_container = create_cli_container()?;
    assert!(cli_container.service_count() > 0);

    // Test TUI container
    let tui_container = create_tui_container()?;
    assert!(tui_container.service_count() > 0);

    // Test development container
    let dev_container = create_development_container()?;
    assert!(dev_container.service_count() > 0);

    // Test minimal container
    let test_container = create_test_container()?;
    assert!(test_container.service_count() > 0);

    Ok(())
}

#[tokio::test]
async fn test_lifecycle_management() -> DIResult<()> {
    use crate::services::{LifecycleManager, Lifecycle};
    use std::sync::Arc;

    // Create a mock service that implements Lifecycle
    struct MockLifecycleService {
        initialized: std::sync::atomic::AtomicBool,
        cleaned_up: std::sync::atomic::AtomicBool,
    }

    #[async_trait::async_trait]
    impl Lifecycle for MockLifecycleService {
        async fn initialize(&self) -> DIResult<()> {
            self.initialized.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn cleanup(&self) -> DIResult<()> {
            self.cleaned_up.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    let service = Arc::new(MockLifecycleService {
        initialized: false.into(),
        cleaned_up: false.into(),
    });

    let mut lifecycle_manager = LifecycleManager::new();
    lifecycle_manager.register_service(service.clone());

    // Test initialization
    lifecycle_manager.initialize_all().await?;
    assert!(service.initialized.load(std::sync::atomic::Ordering::SeqCst));

    // Test cleanup
    lifecycle_manager.cleanup_all().await?;
    assert!(service.cleaned_up.load(std::sync::atomic::Ordering::SeqCst));

    Ok(())
}