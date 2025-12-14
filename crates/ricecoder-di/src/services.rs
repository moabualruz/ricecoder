//! Service registration for the DI container
//!
//! This module provides functions to register all the services
//! needed by the RiceCoder application across all crates.

use crate::{DIContainer, DIResult};
use ricecoder_agents::use_cases::{
    ProviderCommunityUseCase, ProviderFailoverUseCase, ProviderHealthUseCase,
    ProviderModelUseCase, ProviderPerformanceUseCase, ProviderSwitchingUseCase,
    SessionLifecycleUseCase, SessionSharingUseCase, SessionStateManagementUseCase,
};
use ricecoder_providers::provider::manager::ProviderManager;
use ricecoder_sessions::{SessionManager, SessionStore, ShareService};
use std::sync::Arc;

// Optional dependencies - only register if available
#[cfg(feature = "storage")]
use ricecoder_storage::{StorageManager, FileStorage, MemoryStorage};
#[cfg(feature = "research")]
use ricecoder_research::{ResearchManager, CodebaseScanner, SemanticIndexer};
#[cfg(feature = "workflows")]
use ricecoder_workflows::{WorkflowEngine, WorkflowManager};
#[cfg(feature = "execution")]
use ricecoder_execution::{ExecutionEngine, CommandExecutor};
#[cfg(feature = "mcp")]
use ricecoder_mcp::{MCPClient, MCPServer};
#[cfg(feature = "tools")]
use ricecoder_tools::{ToolRegistry, ToolExecutor};
#[cfg(feature = "config")]
use ricecoder_config::{ConfigManager, ConfigLoader};
#[cfg(feature = "activity-log")]
use ricecoder_activity_log::{ActivityLogger, AuditLogger, SessionTracker};
#[cfg(feature = "orchestration")]
use ricecoder_orchestration::{WorkspaceOrchestrator, OperationManager};
#[cfg(feature = "specs")]
use ricecoder_specs::{SpecManager, SpecValidator, SpecCache};
#[cfg(feature = "undo-redo")]
use ricecoder_undo_redo::{UndoManager, RedoManager, HistoryManager};
#[cfg(feature = "vcs")]
use ricecoder_vcs::{VCSManager, GitIntegration, RepositoryManager};
#[cfg(feature = "permissions")]
use ricecoder_permissions::{PermissionManager, PermissionChecker, AuditLogger as PermissionAuditLogger};
#[cfg(feature = "security")]
use ricecoder_security::{AccessControl, EncryptionService, ValidationService};
#[cfg(feature = "cache")]
use ricecoder_cache::{CacheManager, CacheStorage, CacheStrategy};
#[cfg(feature = "domain")]
use ricecoder_domain::{DomainService, Repository, EntityManager};
#[cfg(feature = "learning")]
use ricecoder_learning::{LearningManager, PatternCapturer, RuleValidator};
#[cfg(feature = "industry")]
use ricecoder_industry::{AuthService, ComplianceManager, ConnectionManager};
#[cfg(feature = "safety")]
use ricecoder_safety::{SafetyMonitor, RiskAssessor, ConstraintValidator};
#[cfg(feature = "files")]
use ricecoder_files::{FileManager, FileWatcher, TransactionManager};
#[cfg(feature = "themes")]
use ricecoder_themes::{ThemeManager, ThemeLoader, ThemeRegistry};
#[cfg(feature = "images")]
use ricecoder_images::{ImageHandler, ImageAnalyzer, ImageCache};

/// Register all infrastructure services
pub fn register_infrastructure_services(container: &DIContainer) -> DIResult<()> {
    // Register session infrastructure
    container.register(|_| {
        let session_store = Arc::new(SessionStore::new().map_err(|e| {
            crate::DIError::DependencyResolutionFailed {
                message: format!("Failed to create session store: {}", e),
            }
        })?);
        Ok(session_store)
    })?;

    container.register(|_| {
        let session_manager = Arc::new(SessionManager::new(10)); // max 10 sessions
        Ok(session_manager)
    })?;

    container.register(|_| {
        let share_service = Arc::new(ShareService::new());
        Ok(share_service)
    })?;

    // Register provider infrastructure
    container.register(|_| {
        let registry = ricecoder_providers::provider::ProviderRegistry::new();
        let provider_manager = Arc::new(ProviderManager::new(registry, "openai".to_string()));
        Ok(provider_manager)
    })?;

    // Register optional infrastructure services
    #[cfg(feature = "storage")]
    register_storage_services(container)?;

    #[cfg(feature = "research")]
    register_research_services(container)?;

    #[cfg(feature = "workflows")]
    register_workflow_services(container)?;

    #[cfg(feature = "execution")]
    register_execution_services(container)?;

    #[cfg(feature = "mcp")]
    register_mcp_services(container)?;

    #[cfg(feature = "tools")]
    register_tool_services(container)?;

    #[cfg(feature = "config")]
    register_config_services(container)?;

    #[cfg(feature = "activity-log")]
    register_activity_log_services(container)?;

    #[cfg(feature = "orchestration")]
    register_orchestration_services(container)?;

    #[cfg(feature = "specs")]
    register_specs_services(container)?;

    #[cfg(feature = "undo-redo")]
    register_undo_redo_services(container)?;

    #[cfg(feature = "vcs")]
    register_vcs_services(container)?;

    #[cfg(feature = "permissions")]
    register_permissions_services(container)?;

    #[cfg(feature = "security")]
    register_security_services(container)?;

    #[cfg(feature = "cache")]
    register_cache_services(container)?;

    #[cfg(feature = "domain")]
    register_domain_services(container)?;

    #[cfg(feature = "learning")]
    register_learning_services(container)?;

    #[cfg(feature = "industry")]
    register_industry_services(container)?;

    #[cfg(feature = "safety")]
    register_safety_services(container)?;

    #[cfg(feature = "files")]
    register_files_services(container)?;

    #[cfg(feature = "themes")]
    register_themes_services(container)?;

    #[cfg(feature = "images")]
    register_images_services(container)?;

    Ok(())
}

/// Register storage services (optional feature)
#[cfg(feature = "storage")]
pub fn register_storage_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_storage::{StorageManager, FileStorage, MemoryStorage};

    container.register(|_| {
        let storage_manager = Arc::new(StorageManager::new());
        Ok(storage_manager)
    })?;

    container.register(|_| {
        let file_storage = Arc::new(FileStorage::new("./data"));
        Ok(file_storage)
    })?;

    container.register(|_| {
        let memory_storage = Arc::new(MemoryStorage::new());
        Ok(memory_storage)
    })?;

    Ok(())
}

/// Register research services (optional feature)
#[cfg(feature = "research")]
pub fn register_research_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_research::{ResearchManager, CodebaseScanner, SemanticIndexer};

    container.register(|_| {
        let research_manager = Arc::new(ResearchManager::new());
        Ok(research_manager)
    })?;

    container.register(|_| {
        let codebase_scanner = Arc::new(CodebaseScanner::new());
        Ok(codebase_scanner)
    })?;

    container.register(|_| {
        let semantic_indexer = Arc::new(SemanticIndexer::new());
        Ok(semantic_indexer)
    })?;

    Ok(())
}

/// Register workflow services (optional feature)
#[cfg(feature = "workflows")]
pub fn register_workflow_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_workflows::{WorkflowEngine, WorkflowManager};

    container.register(|_| {
        let workflow_engine = Arc::new(WorkflowEngine::new());
        Ok(workflow_engine)
    })?;

    container.register(|_| {
        let workflow_manager = Arc::new(WorkflowManager::new());
        Ok(workflow_manager)
    })?;

    Ok(())
}

/// Register execution services (optional feature)
#[cfg(feature = "execution")]
pub fn register_execution_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_execution::{ExecutionEngine, CommandExecutor};

    container.register(|_| {
        let execution_engine = Arc::new(ExecutionEngine::new());
        Ok(execution_engine)
    })?;

    container.register(|_| {
        let command_executor = Arc::new(CommandExecutor::new());
        Ok(command_executor)
    })?;

    Ok(())
}

/// Register MCP services (optional feature)
#[cfg(feature = "mcp")]
pub fn register_mcp_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_mcp::{MCPClient, MCPServer};

    container.register(|_| {
        let mcp_client = Arc::new(MCPClient::new());
        Ok(mcp_client)
    })?;

    container.register(|_| {
        let mcp_server = Arc::new(MCPServer::new());
        Ok(mcp_server)
    })?;

    Ok(())
}

/// Register tool services (optional feature)
#[cfg(feature = "tools")]
pub fn register_tool_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_tools::{ToolRegistry, ToolExecutor};

    container.register(|_| {
        let tool_registry = Arc::new(ToolRegistry::new());
        Ok(tool_registry)
    })?;

    container.register(|_| {
        let tool_executor = Arc::new(ToolExecutor::new());
        Ok(tool_executor)
    })?;

    Ok(())
}

/// Register config services (optional feature)
#[cfg(feature = "config")]
pub fn register_config_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_config::{ConfigManager, ConfigLoader};

    container.register(|_| {
        let config_manager = Arc::new(ConfigManager::new());
        Ok(config_manager)
    })?;

    container.register(|_| {
        let config_loader = Arc::new(ConfigLoader::new());
        Ok(config_loader)
    })?;

    Ok(())
}

/// Register activity log services (optional feature)
#[cfg(feature = "activity-log")]
pub fn register_activity_log_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_activity_log::{ActivityLogger, AuditLogger, SessionTracker};

    container.register(|_| {
        let activity_logger = Arc::new(ActivityLogger::new());
        Ok(activity_logger)
    })?;

    container.register(|_| {
        let audit_logger = Arc::new(AuditLogger::new());
        Ok(audit_logger)
    })?;

    container.register(|_| {
        let session_tracker = Arc::new(SessionTracker::new());
        Ok(session_tracker)
    })?;

    Ok(())
}

/// Register orchestration services (optional feature)
#[cfg(feature = "orchestration")]
pub fn register_orchestration_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_orchestration::{WorkspaceOrchestrator, OperationManager};

    container.register(|_| {
        let workspace_orchestrator = Arc::new(WorkspaceOrchestrator::new());
        Ok(workspace_orchestrator)
    })?;

    container.register(|_| {
        let operation_manager = Arc::new(OperationManager::new());
        Ok(operation_manager)
    })?;

    Ok(())
}

/// Register specs services (optional feature)
#[cfg(feature = "specs")]
pub fn register_specs_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_specs::{SpecManager, SpecValidator, SpecCache};

    container.register(|_| {
        let spec_manager = Arc::new(SpecManager::new());
        Ok(spec_manager)
    })?;

    container.register(|_| {
        let spec_validator = Arc::new(SpecValidator::new());
        Ok(spec_validator)
    })?;

    container.register(|_| {
        let spec_cache = Arc::new(SpecCache::new());
        Ok(spec_cache)
    })?;

    Ok(())
}

/// Register undo-redo services (optional feature)
#[cfg(feature = "undo-redo")]
pub fn register_undo_redo_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_undo_redo::{UndoManager, RedoManager, HistoryManager};

    container.register(|_| {
        let undo_manager = Arc::new(UndoManager::new());
        Ok(undo_manager)
    })?;

    container.register(|_| {
        let redo_manager = Arc::new(RedoManager::new());
        Ok(redo_manager)
    })?;

    container.register(|_| {
        let history_manager = Arc::new(HistoryManager::new());
        Ok(history_manager)
    })?;

    Ok(())
}

/// Register VCS services (optional feature)
#[cfg(feature = "vcs")]
pub fn register_vcs_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_vcs::{VCSManager, GitIntegration, RepositoryManager};

    container.register(|_| {
        let vcs_manager = Arc::new(VCSManager::new());
        Ok(vcs_manager)
    })?;

    container.register(|_| {
        let git_integration = Arc::new(GitIntegration::new());
        Ok(git_integration)
    })?;

    container.register(|_| {
        let repository_manager = Arc::new(RepositoryManager::new());
        Ok(repository_manager)
    })?;

    Ok(())
}

/// Register permissions services (optional feature)
#[cfg(feature = "permissions")]
pub fn register_permissions_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_permissions::{PermissionManager, PermissionChecker, AuditLogger as PermissionAuditLogger};

    container.register(|_| {
        let permission_manager = Arc::new(PermissionManager::new());
        Ok(permission_manager)
    })?;

    container.register(|_| {
        let permission_checker = Arc::new(PermissionChecker::new());
        Ok(permission_checker)
    })?;

    container.register(|_| {
        let audit_logger = Arc::new(PermissionAuditLogger::new());
        Ok(audit_logger)
    })?;

    Ok(())
}

/// Register security services (optional feature)
#[cfg(feature = "security")]
pub fn register_security_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_security::{AccessControl, EncryptionService, ValidationService};

    container.register(|_| {
        let access_control = Arc::new(AccessControl::new());
        Ok(access_control)
    })?;

    container.register(|_| {
        let encryption_service = Arc::new(EncryptionService::new());
        Ok(encryption_service)
    })?;

    container.register(|_| {
        let validation_service = Arc::new(ValidationService::new());
        Ok(validation_service)
    })?;

    Ok(())
}

/// Register cache services (optional feature)
#[cfg(feature = "cache")]
pub fn register_cache_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_cache::{CacheManager, CacheStorage, CacheStrategy};

    container.register(|_| {
        let cache_manager = Arc::new(CacheManager::new());
        Ok(cache_manager)
    })?;

    container.register(|_| {
        let cache_storage = Arc::new(CacheStorage::new());
        Ok(cache_storage)
    })?;

    container.register(|_| {
        let cache_strategy = Arc::new(CacheStrategy::new());
        Ok(cache_strategy)
    })?;

    Ok(())
}

/// Register domain services (optional feature)
#[cfg(feature = "domain")]
pub fn register_domain_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_domain::{DomainService, Repository, EntityManager};

    container.register(|_| {
        let domain_service = Arc::new(DomainService::new());
        Ok(domain_service)
    })?;

    container.register(|_| {
        let repository = Arc::new(Repository::new());
        Ok(repository)
    })?;

    container.register(|_| {
        let entity_manager = Arc::new(EntityManager::new());
        Ok(entity_manager)
    })?;

    Ok(())
}

/// Register learning services (optional feature)
#[cfg(feature = "learning")]
pub fn register_learning_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_learning::{LearningManager, PatternCapturer, RuleValidator};

    container.register(|_| {
        let learning_manager = Arc::new(LearningManager::new());
        Ok(learning_manager)
    })?;

    container.register(|_| {
        let pattern_capturer = Arc::new(PatternCapturer::new());
        Ok(pattern_capturer)
    })?;

    container.register(|_| {
        let rule_validator = Arc::new(RuleValidator::new());
        Ok(rule_validator)
    })?;

    Ok(())
}

/// Register industry services (optional feature)
#[cfg(feature = "industry")]
pub fn register_industry_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_industry::{AuthService, ComplianceManager, ConnectionManager};

    container.register(|_| {
        let auth_service = Arc::new(AuthService::new());
        Ok(auth_service)
    })?;

    container.register(|_| {
        let compliance_manager = Arc::new(ComplianceManager::new());
        Ok(compliance_manager)
    })?;

    container.register(|_| {
        let connection_manager = Arc::new(ConnectionManager::new());
        Ok(connection_manager)
    })?;

    Ok(())
}

/// Register safety services (optional feature)
#[cfg(feature = "safety")]
pub fn register_safety_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_safety::{SafetyMonitor, RiskAssessor, ConstraintValidator};

    container.register(|_| {
        let safety_monitor = Arc::new(SafetyMonitor::new());
        Ok(safety_monitor)
    })?;

    container.register(|_| {
        let risk_assessor = Arc::new(RiskAssessor::new());
        Ok(risk_assessor)
    })?;

    container.register(|_| {
        let constraint_validator = Arc::new(ConstraintValidator::new());
        Ok(constraint_validator)
    })?;

    Ok(())
}

/// Register files services (optional feature)
#[cfg(feature = "files")]
pub fn register_files_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_files::{FileManager, FileWatcher, TransactionManager};

    container.register(|_| {
        let file_manager = Arc::new(FileManager::new());
        Ok(file_manager)
    })?;

    container.register(|_| {
        let file_watcher = Arc::new(FileWatcher::new());
        Ok(file_watcher)
    })?;

    container.register(|_| {
        let transaction_manager = Arc::new(TransactionManager::new());
        Ok(transaction_manager)
    })?;

    Ok(())
}

/// Register themes services (optional feature)
#[cfg(feature = "themes")]
pub fn register_themes_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_themes::{ThemeManager, ThemeLoader, ThemeRegistry};

    container.register(|_| {
        let theme_manager = Arc::new(ThemeManager::new());
        Ok(theme_manager)
    })?;

    container.register(|_| {
        let theme_loader = Arc::new(ThemeLoader::new());
        Ok(theme_loader)
    })?;

    container.register(|_| {
        let theme_registry = Arc::new(ThemeRegistry::new());
        Ok(theme_registry)
    })?;

    Ok(())
}

/// Register images services (optional feature)
#[cfg(feature = "images")]
pub fn register_images_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_images::{ImageHandler, ImageAnalyzer, ImageCache};

    container.register(|_| {
        let image_handler = Arc::new(ImageHandler::new());
        Ok(image_handler)
    })?;

    container.register(|_| {
        let image_analyzer = Arc::new(ImageAnalyzer::new());
        Ok(image_analyzer)
    })?;

    container.register(|_| {
        let image_cache = Arc::new(ImageCache::new());
        Ok(image_cache)
    })?;

    Ok(())
}

/// Register all application use cases
pub fn register_use_cases(container: &DIContainer) -> DIResult<()> {
    // Register session use cases
    container.register(|container| {
        let session_manager = container.resolve::<SessionManager>()?;
        let session_store = container.resolve::<SessionStore>()?;
        let use_case = Arc::new(SessionLifecycleUseCase::new(
            session_manager,
            session_store,
        ));
        Ok(use_case)
    })?;

    container.register(|container| {
        let share_service = container.resolve::<ShareService>()?;
        let session_store = container.resolve::<SessionStore>()?;
        let use_case = Arc::new(SessionSharingUseCase::new(share_service, session_store));
        Ok(use_case)
    })?;

    container.register(|container| {
        let session_manager = container.resolve::<SessionManager>()?;
        let use_case = Arc::new(SessionStateManagementUseCase::new(session_manager));
        Ok(use_case)
    })?;

    // Register provider use cases
    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderSwitchingUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderPerformanceUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderFailoverUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderModelUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderHealthUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderCommunityUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    Ok(())
}

/// Register all services using the builder pattern
pub fn create_application_container() -> DIResult<DIContainer> {
    let mut builder = crate::DIContainerBuilder::new();

    // Register core services (always available)
    builder = builder.register_infrastructure_services()?;
    builder = builder.register_use_cases()?;

    let container = builder.build();
    Ok(container)
}

/// Create a full-featured container with all optional services enabled
#[cfg(feature = "full")]
pub fn create_full_application_container() -> DIResult<DIContainer> {
    let container = crate::DIContainerBuilder::new()
        .register_infrastructure_services()?
        .register_use_cases()?
        .build();

    Ok(container)
}

/// Extension trait for DIContainerBuilder to add convenience methods
pub trait DIContainerBuilderExt {
    fn register_infrastructure_services(self) -> DIResult<Self> where Self: Sized;
    fn register_use_cases(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "storage")]
    fn register_storage_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "research")]
    fn register_research_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "workflows")]
    fn register_workflow_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "execution")]
    fn register_execution_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "mcp")]
    fn register_mcp_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "tools")]
    fn register_tool_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "config")]
    fn register_config_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "activity-log")]
    fn register_activity_log_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "orchestration")]
    fn register_orchestration_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "specs")]
    fn register_specs_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "undo-redo")]
    fn register_undo_redo_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "vcs")]
    fn register_vcs_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "permissions")]
    fn register_permissions_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "security")]
    fn register_security_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "cache")]
    fn register_cache_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "domain")]
    fn register_domain_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "learning")]
    fn register_learning_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "industry")]
    fn register_industry_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "safety")]
    fn register_safety_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "files")]
    fn register_files_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "themes")]
    fn register_themes_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "images")]
    fn register_images_services(self) -> DIResult<Self> where Self: Sized;
}

impl DIContainerBuilderExt for crate::DIContainerBuilder {
    fn register_infrastructure_services(self) -> DIResult<Self> {
        register_infrastructure_services(&self.container)?;
        Ok(self)
    }

    fn register_use_cases(self) -> DIResult<Self> {
        register_use_cases(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "storage")]
    fn register_storage_services(self) -> DIResult<Self> {
        register_storage_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "research")]
    fn register_research_services(self) -> DIResult<Self> {
        register_research_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "workflows")]
    fn register_workflow_services(self) -> DIResult<Self> {
        register_workflow_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "execution")]
    fn register_execution_services(self) -> DIResult<Self> {
        register_execution_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "mcp")]
    fn register_mcp_services(self) -> DIResult<Self> {
        register_mcp_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "tools")]
    fn register_tool_services(self) -> DIResult<Self> {
        register_tool_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "config")]
    fn register_config_services(self) -> DIResult<Self> {
        register_config_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "activity-log")]
    fn register_activity_log_services(self) -> DIResult<Self> {
        register_activity_log_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "orchestration")]
    fn register_orchestration_services(self) -> DIResult<Self> {
        register_orchestration_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "specs")]
    fn register_specs_services(self) -> DIResult<Self> {
        register_specs_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "undo-redo")]
    fn register_undo_redo_services(self) -> DIResult<Self> {
        register_undo_redo_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "vcs")]
    fn register_vcs_services(self) -> DIResult<Self> {
        register_vcs_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "permissions")]
    fn register_permissions_services(self) -> DIResult<Self> {
        register_permissions_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "security")]
    fn register_security_services(self) -> DIResult<Self> {
        register_security_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "cache")]
    fn register_cache_services(self) -> DIResult<Self> {
        register_cache_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "domain")]
    fn register_domain_services(self) -> DIResult<Self> {
        register_domain_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "learning")]
    fn register_learning_services(self) -> DIResult<Self> {
        register_learning_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "industry")]
    fn register_industry_services(self) -> DIResult<Self> {
        register_industry_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "safety")]
    fn register_safety_services(self) -> DIResult<Self> {
        register_safety_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "files")]
    fn register_files_services(self) -> DIResult<Self> {
        register_files_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "themes")]
    fn register_themes_services(self) -> DIResult<Self> {
        register_themes_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "images")]
    fn register_images_services(self) -> DIResult<Self> {
        register_images_services(&self.container)?;
        Ok(self)
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

        // Should have all services registered
        assert!(container.service_count() > 0);

        // Should be able to resolve key services
        let session_use_case = container.resolve::<SessionLifecycleUseCase>();
        assert!(session_use_case.is_ok());

        let provider_use_case = container.resolve::<ProviderSwitchingUseCase>();
        assert!(provider_use_case.is_ok());
    }