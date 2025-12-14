//! Tests for service registration functionality

use ricecoder_di::*;
use ricecoder_agents::use_cases::{
    ProviderCommunityUseCase, ProviderFailoverUseCase, ProviderHealthUseCase,
    ProviderModelUseCase, ProviderPerformanceUseCase, ProviderSwitchingUseCase,
    SessionLifecycleUseCase, SessionSharingUseCase, SessionStateManagementUseCase,
};
use ricecoder_sessions::{SessionManager, SessionStore, ShareService};
use ricecoder_providers::provider::manager::ProviderManager;

#[test]
fn test_register_infrastructure_services() {
    let container = DIContainer::new();
    register_infrastructure_services(&container).unwrap();

    // Check that core services are registered
    assert!(container.is_registered::<SessionStore>());
    assert!(container.is_registered::<SessionManager>());
    assert!(container.is_registered::<ShareService>());
    assert!(container.is_registered::<ProviderManager>());

    // Check optional services based on features
    #[cfg(feature = "storage")]
    {
        use ricecoder_storage::{StorageManager, FileStorage, MemoryStorage};
        assert!(container.is_registered::<StorageManager>());
        assert!(container.is_registered::<FileStorage>());
        assert!(container.is_registered::<MemoryStorage>());
    }

    #[cfg(feature = "research")]
    {
        use ricecoder_research::{ResearchManager, CodebaseScanner, SemanticIndexer};
        assert!(container.is_registered::<ResearchManager>());
        assert!(container.is_registered::<CodebaseScanner>());
        assert!(container.is_registered::<SemanticIndexer>());
    }

    #[cfg(feature = "workflows")]
    {
        use ricecoder_workflows::{WorkflowEngine, WorkflowManager};
        assert!(container.is_registered::<WorkflowEngine>());
        assert!(container.is_registered::<WorkflowManager>());
    }

    #[cfg(feature = "execution")]
    {
        use ricecoder_execution::{ExecutionEngine, CommandExecutor};
        assert!(container.is_registered::<ExecutionEngine>());
        assert!(container.is_registered::<CommandExecutor>());
    }

    #[cfg(feature = "mcp")]
    {
        use ricecoder_mcp::{MCPClient, MCPServer};
        assert!(container.is_registered::<MCPClient>());
        assert!(container.is_registered::<MCPServer>());
    }

    #[cfg(feature = "tools")]
    {
        use ricecoder_tools::{ToolRegistry, ToolExecutor};
        assert!(container.is_registered::<ToolRegistry>());
        assert!(container.is_registered::<ToolExecutor>());
    }

    #[cfg(feature = "config")]
    {
        use ricecoder_config::{ConfigManager, ConfigLoader};
        assert!(container.is_registered::<ConfigManager>());
        assert!(container.is_registered::<ConfigLoader>());
    }

    #[cfg(feature = "activity-log")]
    {
        use ricecoder_activity_log::{ActivityLogger, AuditLogger, SessionTracker};
        assert!(container.is_registered::<ActivityLogger>());
        assert!(container.is_registered::<AuditLogger>());
        assert!(container.is_registered::<SessionTracker>());
    }

    #[cfg(feature = "orchestration")]
    {
        use ricecoder_orchestration::{WorkspaceOrchestrator, OperationManager};
        assert!(container.is_registered::<WorkspaceOrchestrator>());
        assert!(container.is_registered::<OperationManager>());
    }

    #[cfg(feature = "specs")]
    {
        use ricecoder_specs::{SpecManager, SpecValidator, SpecCache};
        assert!(container.is_registered::<SpecManager>());
        assert!(container.is_registered::<SpecValidator>());
        assert!(container.is_registered::<SpecCache>());
    }

    #[cfg(feature = "undo-redo")]
    {
        use ricecoder_undo_redo::{UndoManager, RedoManager, HistoryManager};
        assert!(container.is_registered::<UndoManager>());
        assert!(container.is_registered::<RedoManager>());
        assert!(container.is_registered::<HistoryManager>());
    }

    #[cfg(feature = "vcs")]
    {
        use ricecoder_vcs::{VCSManager, GitIntegration, RepositoryManager};
        assert!(container.is_registered::<VCSManager>());
        assert!(container.is_registered::<GitIntegration>());
        assert!(container.is_registered::<RepositoryManager>());
    }

    #[cfg(feature = "permissions")]
    {
        use ricecoder_permissions::{PermissionManager, PermissionChecker, AuditLogger as PermissionAuditLogger};
        assert!(container.is_registered::<PermissionManager>());
        assert!(container.is_registered::<PermissionChecker>());
        assert!(container.is_registered::<PermissionAuditLogger>());
    }

    #[cfg(feature = "security")]
    {
        use ricecoder_security::{AccessControl, EncryptionService, ValidationService};
        assert!(container.is_registered::<AccessControl>());
        assert!(container.is_registered::<EncryptionService>());
        assert!(container.is_registered::<ValidationService>());
    }

    #[cfg(feature = "cache")]
    {
        use ricecoder_cache::{CacheManager, CacheStorage, CacheStrategy};
        assert!(container.is_registered::<CacheManager>());
        assert!(container.is_registered::<CacheStorage>());
        assert!(container.is_registered::<CacheStrategy>());
    }

    #[cfg(feature = "domain")]
    {
        use ricecoder_domain::{DomainService, Repository, EntityManager};
        assert!(container.is_registered::<DomainService>());
        assert!(container.is_registered::<Repository>());
        assert!(container.is_registered::<EntityManager>());
    }

    #[cfg(feature = "learning")]
    {
        use ricecoder_learning::{LearningManager, PatternCapturer, RuleValidator};
        assert!(container.is_registered::<LearningManager>());
        assert!(container.is_registered::<PatternCapturer>());
        assert!(container.is_registered::<RuleValidator>());
    }

    #[cfg(feature = "industry")]
    {
        use ricecoder_industry::{AuthService, ComplianceManager, ConnectionManager};
        assert!(container.is_registered::<AuthService>());
        assert!(container.is_registered::<ComplianceManager>());
        assert!(container.is_registered::<ConnectionManager>());
    }

    #[cfg(feature = "safety")]
    {
        use ricecoder_safety::{SafetyMonitor, RiskAssessor, ConstraintValidator};
        assert!(container.is_registered::<SafetyMonitor>());
        assert!(container.is_registered::<RiskAssessor>());
        assert!(container.is_registered::<ConstraintValidator>());
    }

    #[cfg(feature = "files")]
    {
        use ricecoder_files::{FileManager, FileWatcher, TransactionManager};
        assert!(container.is_registered::<FileManager>());
        assert!(container.is_registered::<FileWatcher>());
        assert!(container.is_registered::<TransactionManager>());
    }

    #[cfg(feature = "themes")]
    {
        use ricecoder_themes::{ThemeManager, ThemeLoader, ThemeRegistry};
        assert!(container.is_registered::<ThemeManager>());
        assert!(container.is_registered::<ThemeLoader>());
        assert!(container.is_registered::<ThemeRegistry>());
    }

    #[cfg(feature = "images")]
    {
        use ricecoder_images::{ImageHandler, ImageAnalyzer, ImageCache};
        assert!(container.is_registered::<ImageHandler>());
        assert!(container.is_registered::<ImageAnalyzer>());
        assert!(container.is_registered::<ImageCache>());
    }
}

#[test]
fn test_register_use_cases() {
    let container = DIContainer::new();
    register_infrastructure_services(&container).unwrap();
    register_use_cases(&container).unwrap();

    // Check that use cases are registered
    assert!(container.is_registered::<SessionLifecycleUseCase>());
    assert!(container.is_registered::<SessionSharingUseCase>());
    assert!(container.is_registered::<SessionStateManagementUseCase>());
    assert!(container.is_registered::<ProviderSwitchingUseCase>());
    assert!(container.is_registered::<ProviderPerformanceUseCase>());
    assert!(container.is_registered::<ProviderFailoverUseCase>());
    assert!(container.is_registered::<ProviderModelUseCase>());
    assert!(container.is_registered::<ProviderHealthUseCase>());
    assert!(container.is_registered::<ProviderCommunityUseCase>());
}

#[test]
fn test_create_application_container() {
    let container = create_application_container().unwrap();

    // Should have registered multiple services
    let service_count = container.service_count();
    assert!(service_count > 10); // Should have infrastructure + use cases

    // Should be able to resolve key services
    let session_use_case = container.resolve::<SessionLifecycleUseCase>();
    assert!(session_use_case.is_ok());

    let provider_use_case = container.resolve::<ProviderSwitchingUseCase>();
    assert!(provider_use_case.is_ok());
}

#[cfg(feature = "full")]
#[test]
fn test_create_full_application_container() {
    let container = create_full_application_container().unwrap();

    // Should have even more services when full feature is enabled
    let service_count = container.service_count();
    assert!(service_count > 15); // Should have all services
}