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
use async_trait::async_trait;

/// Trait for services that need lifecycle management
#[async_trait]
pub trait Lifecycle: Send + Sync {
    /// Initialize the service (called after all dependencies are resolved)
    async fn initialize(&self) -> DIResult<()> {
        Ok(())
    }

    /// Cleanup the service (called during shutdown)
    async fn cleanup(&self) -> DIResult<()> {
        Ok(())
    }
}

/// Lifecycle manager for handling service initialization and cleanup
pub struct LifecycleManager {
    services: Vec<Arc<dyn Lifecycle>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    /// Register a service for lifecycle management
    pub fn register_service(&mut self, service: Arc<dyn Lifecycle>) {
        self.services.push(service);
    }

    /// Initialize all registered services
    pub async fn initialize_all(&self) -> DIResult<()> {
        for service in &self.services {
            service.initialize().await?;
        }
        Ok(())
    }

    /// Cleanup all registered services
    pub async fn cleanup_all(&self) -> DIResult<()> {
        // Cleanup in reverse order
        for service in self.services.iter().rev() {
            service.cleanup().await?;
        }
        Ok(())
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

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
#[cfg(feature = "completion")]
use ricecoder_completion::engine::GenericCompletionEngine;
#[cfg(feature = "lsp")]
use ricecoder_lsp::types::LspResult;
#[cfg(feature = "modes")]
use ricecoder_modes::ModeManager;
#[cfg(feature = "commands")]
use ricecoder_commands::{CommandManager, CommandRegistry};
#[cfg(feature = "hooks")]
use ricecoder_hooks::registry::HookRegistry;
#[cfg(feature = "keybinds")]
use ricecoder_keybinds::KeybindManager;
#[cfg(feature = "teams")]
use ricecoder_teams::TeamManager;
#[cfg(feature = "refactoring")]
use ricecoder_refactoring::{ConfigManager, ProviderRegistry};
#[cfg(feature = "parsers")]
use ricecoder_parsers::Parser;
#[cfg(feature = "generation")]
use ricecoder_generation::{GenerationManager, CodeGenerator, SpecProcessor, TemplateEngine};
#[cfg(feature = "config")]
use ricecoder_config::ConfigManager as AppConfigManager;
#[cfg(feature = "github")]
use ricecoder_github::managers::{GitHubManager, IssueManager, PrManager, ReleaseManager, DiscussionManager, GistManager, BranchManager, ProjectManager, DocumentationGenerator, CodeReviewAgent, RepositoryAnalyzer, WebhookHandler};
#[cfg(feature = "domain-agents")]
use ricecoder_domain_agents::{DomainAgentRegistryManager, KnowledgeBaseManager};
#[cfg(feature = "local-models")]
use ricecoder_local_models::LocalModelManager;
#[cfg(feature = "cli")]
use ricecoder_cli::{CommandRouter, BrandingManager};

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

    #[cfg(feature = "monitoring")]
    register_monitoring_services(container)?;

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

    #[cfg(feature = "completion")]
    register_completion_services(container)?;

    #[cfg(feature = "lsp")]
    register_lsp_services(container)?;

    #[cfg(feature = "modes")]
    register_modes_services(container)?;

    #[cfg(feature = "commands")]
    register_commands_services(container)?;

    #[cfg(feature = "hooks")]
    register_hooks_services(container)?;

    #[cfg(feature = "keybinds")]
    register_keybinds_services(container)?;

    #[cfg(feature = "teams")]
    register_teams_services(container)?;

    #[cfg(feature = "refactoring")]
    register_refactoring_services(container)?;

    #[cfg(feature = "parsers")]
    register_parsers_services(container)?;

    #[cfg(feature = "generation")]
    register_generation_services(container)?;



    #[cfg(feature = "github")]
    register_github_services(container)?;

    #[cfg(feature = "domain-agents")]
    register_domain_agents_services(container)?;

    #[cfg(feature = "local-models")]
    register_local_models_services(container)?;

    #[cfg(feature = "cli")]
    register_cli_services(container)?;

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

/// Register completion services (optional feature)
#[cfg(feature = "completion")]
pub fn register_completion_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_completion::engine::GenericCompletionEngine;

    container.register(|_| {
        let completion_engine = Arc::new(GenericCompletionEngine::new());
        Ok(completion_engine)
    })?;

    Ok(())
}

/// Register LSP services (optional feature)
#[cfg(feature = "lsp")]
pub fn register_lsp_services(container: &DIContainer) -> DIResult<()> {
    // LSP services are primarily used through external integration
    // The main LSP types are available through ricecoder_lsp crate
    // For now, we don't register specific services as LSP is more of a protocol interface
    Ok(())
}

/// Register modes services (optional feature)
#[cfg(feature = "modes")]
pub fn register_modes_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_modes::ModeManager;

    container.register(|_| {
        let mode_manager = Arc::new(ModeManager::new(Default::default()));
        Ok(mode_manager)
    })?;

    Ok(())
}

/// Register commands services (optional feature)
#[cfg(feature = "commands")]
pub fn register_commands_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_commands::{CommandManager, CommandRegistry};

    container.register(|_| {
        let command_registry = Arc::new(CommandRegistry::new());
        Ok(command_registry)
    })?;

    container.register(|_| {
        let command_manager = Arc::new(CommandManager::new());
        Ok(command_manager)
    })?;

    Ok(())
}

/// Register hooks services (optional feature)
#[cfg(feature = "hooks")]
pub fn register_hooks_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_hooks::registry::HookRegistry;

    container.register(|_| {
        let hook_registry = Arc::new(HookRegistry::new());
        Ok(hook_registry)
    })?;

    Ok(())
}

/// Register keybinds services (optional feature)
#[cfg(feature = "keybinds")]
pub fn register_keybinds_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_keybinds::KeybindManager;

    container.register(|_| {
        let keybind_manager = Arc::new(KeybindManager::new());
        Ok(keybind_manager)
    })?;

    Ok(())
}

/// Register teams services (optional feature)
#[cfg(feature = "teams")]
pub fn register_teams_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_teams::TeamManager;

    container.register(|_| {
        let team_manager = Arc::new(TeamManager::new());
        Ok(team_manager)
    })?;

    Ok(())
}

/// Register refactoring services (optional feature)
#[cfg(feature = "refactoring")]
pub fn register_refactoring_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_refactoring::{ConfigManager, ProviderRegistry};

    container.register(|_| {
        let config_manager = Arc::new(ConfigManager::new());
        Ok(config_manager)
    })?;

    container.register(|_| {
        let provider_registry = Arc::new(ProviderRegistry::new());
        Ok(provider_registry)
    })?;

    Ok(())
}

/// Register parsers services (optional feature)
#[cfg(feature = "parsers")]
pub fn register_parsers_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_parsers::Parser;

    container.register(|_| {
        let parser = Arc::new(Parser::new());
        Ok(parser)
    })?;

    Ok(())
}

/// Register generation services (optional feature)
#[cfg(feature = "generation")]
pub fn register_generation_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_generation::{GenerationManager, CodeGenerator, SpecProcessor, TemplateEngine};

    container.register(|_| {
        let generation_manager = Arc::new(GenerationManager::new(Default::default()));
        Ok(generation_manager)
    })?;

    container.register(|_| {
        let code_generator = Arc::new(CodeGenerator::new(Default::default()));
        Ok(code_generator)
    })?;

    container.register(|_| {
        let spec_processor = Arc::new(SpecProcessor::new());
        Ok(spec_processor)
    })?;

    container.register(|_| {
        let template_engine = Arc::new(TemplateEngine::new());
        Ok(template_engine)
    })?;

    Ok(())
}

/// Register monitoring services (optional feature)
#[cfg(feature = "monitoring")]
pub fn register_monitoring_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_monitoring::{AnalyticsConfig, AlertingConfig, ComplianceConfig, ErrorTrackingConfig, MetricsConfig, MonitoringConfig, PerformanceConfig};

    // Create default monitoring configuration
    let monitoring_config = MonitoringConfig {
        metrics: MetricsConfig {
            enabled: true,
            collection_interval: std::time::Duration::from_secs(60),
            retention_period: std::time::Duration::from_secs(86400 * 30), // 30 days
            exporters: vec![],
        },
        alerting: AlertingConfig {
            enabled: true,
            rules: vec![],
            channels: vec![],
        },
        error_tracking: ErrorTrackingConfig {
            enabled: true,
            dsn: None,
            environment: "production".to_string(),
            release: None,
            sample_rate: 1.0,
        },
        performance: PerformanceConfig {
            enabled: true,
            profiling_enabled: true,
            anomaly_detection_enabled: true,
            thresholds: ricecoder_monitoring::types::PerformanceThresholds {
                max_response_time_ms: 500,
                max_memory_mb: 300,
                max_cpu_percent: 80.0,
            },
        },
        analytics: AnalyticsConfig {
            enabled: true,
            tracking_id: None,
            event_buffer_size: 1000,
            flush_interval: std::time::Duration::from_secs(300),
        },
        compliance: ComplianceConfig {
            enabled: true,
            standards: vec!["SOC2".to_string(), "GDPR".to_string()],
            reporting_interval: std::time::Duration::from_secs(86400), // Daily
            audit_log_retention: std::time::Duration::from_secs(86400 * 2555), // 7 years
        },
    };

    container.register(move |_| {
        let metrics_collector = Arc::new(MetricsCollector::new(monitoring_config.metrics.clone()));
        Ok(metrics_collector)
    })?;

    container.register(move |_| {
        let error_tracker = Arc::new(ErrorTracker::new(monitoring_config.error_tracking.clone()));
        Ok(error_tracker)
    })?;

    container.register(move |_| {
        let performance_monitor = Arc::new(MonitoringPerformanceMonitor::new(monitoring_config.performance.clone()));
        Ok(performance_monitor)
    })?;

    container.register(move |_| {
        let analytics_engine = Arc::new(AnalyticsEngine::new(monitoring_config.analytics.clone()));
        Ok(analytics_engine)
    })?;

    container.register(move |_| {
        let compliance_engine = Arc::new(ComplianceEngine::new(monitoring_config.compliance.clone()));
        Ok(compliance_engine)
    })?;

    container.register(|_| {
        let dashboard_manager = Arc::new(DashboardManager::new());
        Ok(dashboard_manager)
    })?;

    container.register(|_| {
        let report_generator = Arc::new(ReportGenerator::new());
        Ok(report_generator)
    })?;

    Ok(())
}

/// Register app config services (optional feature)
#[cfg(feature = "config")]
pub fn register_app_config_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_config::ConfigManager;

    container.register(|_| {
        let config_manager = Arc::new(ConfigManager::new());
        Ok(config_manager)
    })?;

    Ok(())
}

/// Register GitHub services (optional feature)
#[cfg(feature = "github")]
pub fn register_github_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_github::managers::{GitHubManager, IssueManager, PrManager, ReleaseManager, DiscussionManager, GistManager, BranchManager, ProjectManager, DocumentationGenerator, CodeReviewAgent, RepositoryAnalyzer, WebhookHandler};

    container.register(|_| {
        let github_manager = Arc::new(GitHubManager::new());
        Ok(github_manager)
    })?;

    container.register(|_| {
        let issue_manager = Arc::new(IssueManager::new());
        Ok(issue_manager)
    })?;

    container.register(|_| {
        let pr_manager = Arc::new(PrManager::new());
        Ok(pr_manager)
    })?;

    container.register(|_| {
        let release_manager = Arc::new(ReleaseManager::new());
        Ok(release_manager)
    })?;

    container.register(|_| {
        let discussion_manager = Arc::new(DiscussionManager::new());
        Ok(discussion_manager)
    })?;

    container.register(|_| {
        let gist_manager = Arc::new(GistManager::new());
        Ok(gist_manager)
    })?;

    container.register(|_| {
        let branch_manager = Arc::new(BranchManager::new());
        Ok(branch_manager)
    })?;

    container.register(|_| {
        let project_manager = Arc::new(ProjectManager::new());
        Ok(project_manager)
    })?;

    container.register(|_| {
        let documentation_generator = Arc::new(DocumentationGenerator::new());
        Ok(documentation_generator)
    })?;

    container.register(|_| {
        let code_review_agent = Arc::new(CodeReviewAgent::new());
        Ok(code_review_agent)
    })?;

    container.register(|_| {
        let repository_analyzer = Arc::new(RepositoryAnalyzer::new());
        Ok(repository_analyzer)
    })?;

    container.register(|_| {
        let webhook_handler = Arc::new(WebhookHandler::new(Default::default()));
        Ok(webhook_handler)
    })?;

    Ok(())
}

/// Register domain agents services (optional feature)
#[cfg(feature = "domain-agents")]
pub fn register_domain_agents_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_domain_agents::{DomainAgentRegistryManager, KnowledgeBaseManager};

    container.register(|_| {
        let domain_agent_registry_manager = Arc::new(DomainAgentRegistryManager::with_defaults());
        Ok(domain_agent_registry_manager)
    })?;

    container.register(|_| {
        let knowledge_base_manager = Arc::new(KnowledgeBaseManager::new());
        Ok(knowledge_base_manager)
    })?;

    Ok(())
}

/// Register local models services (optional feature)
#[cfg(feature = "local-models")]
pub fn register_local_models_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_local_models::LocalModelManager;

    container.register(|_| {
        let local_model_manager = Arc::new(LocalModelManager::new());
        Ok(local_model_manager)
    })?;

    Ok(())
}

/// Register CLI services (optional feature)
#[cfg(feature = "cli")]
pub fn register_cli_services(container: &DIContainer) -> DIResult<()> {
    use ricecoder_cli::{CommandRouter, BrandingManager};

    container.register(|_| {
        let command_router = Arc::new(CommandRouter::new());
        Ok(command_router)
    })?;

    container.register(|_| {
        let branding_manager = Arc::new(BrandingManager::new());
        Ok(branding_manager)
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

/// Create a CLI-only container with minimal services
pub fn create_cli_container() -> DIResult<DIContainer> {
    let mut builder = crate::DIContainerBuilder::new();

    // Register core infrastructure
    builder = builder.register_infrastructure_services()?;

    // Register CLI-specific services
    #[cfg(feature = "cli")]
    {
        builder = builder.register_cli_services()?;
    }

    // Register essential services for CLI
    #[cfg(feature = "commands")]
    {
        builder = builder.register_commands_services()?;
    }

    let container = builder.build();
    Ok(container)
}

/// Create a TUI-only container with UI-focused services
pub fn create_tui_container() -> DIResult<DIContainer> {
    let mut builder = crate::DIContainerBuilder::new();

    // Register core infrastructure
    builder = builder.register_infrastructure_services()?;

    // Register TUI-specific services
    #[cfg(feature = "themes")]
    {
        builder = builder.register_themes_services()?;
    }

    #[cfg(feature = "keybinds")]
    {
        builder = builder.register_keybinds_services()?;
    }

    let container = builder.build();
    Ok(container)
}

/// Create a development container with additional debugging services
pub fn create_development_container() -> DIResult<DIContainer> {
    let mut builder = crate::DIContainerBuilder::new();

    // Register all core services
    builder = builder.register_infrastructure_services()?;
    builder = builder.register_use_cases()?;

    // Register development-specific services
    #[cfg(feature = "activity-log")]
    {
        builder = builder.register_activity_log_services()?;
    }

    #[cfg(feature = "cache")]
    {
        builder = builder.register_cache_services()?;
    }

    let container = builder.build();
    Ok(container)
}

/// Create a minimal container for testing
pub fn create_test_container() -> DIResult<DIContainer> {
    let mut builder = crate::DIContainerBuilder::new();

    // Register minimal infrastructure for testing
    builder = builder.register_infrastructure_services()?;

    let container = builder.build();
    Ok(container)
}

/// Configuration for container creation
#[derive(Debug, Clone, Default)]
pub struct ContainerConfig {
    /// Enable storage services
    pub enable_storage: bool,
    /// Enable research services
    pub enable_research: bool,
    /// Enable workflow services
    pub enable_workflows: bool,
    /// Enable execution services
    pub enable_execution: bool,
    /// Enable MCP services
    pub enable_mcp: bool,
    /// Enable tool services
    pub enable_tools: bool,
    /// Enable config services
    pub enable_config: bool,
    /// Enable activity log services
    pub enable_activity_log: bool,
    /// Enable orchestration services
    pub enable_orchestration: bool,
    /// Enable specs services
    pub enable_specs: bool,
    /// Enable undo-redo services
    pub enable_undo_redo: bool,
    /// Enable VCS services
    pub enable_vcs: bool,
    /// Enable permissions services
    pub enable_permissions: bool,
    /// Enable security services
    pub enable_security: bool,
    /// Enable cache services
    pub enable_cache: bool,
    /// Enable domain services
    pub enable_domain: bool,
    /// Enable learning services
    pub enable_learning: bool,
    /// Enable industry services
    pub enable_industry: bool,
    /// Enable safety services
    pub enable_safety: bool,
    /// Enable files services
    pub enable_files: bool,
    /// Enable themes services
    pub enable_themes: bool,
    /// Enable images services
    pub enable_images: bool,
    /// Enable completion services
    pub enable_completion: bool,
    /// Enable LSP services
    pub enable_lsp: bool,
    /// Enable modes services
    pub enable_modes: bool,
    /// Enable commands services
    pub enable_commands: bool,
    /// Enable hooks services
    pub enable_hooks: bool,
    /// Enable keybinds services
    pub enable_keybinds: bool,
    /// Enable teams services
    pub enable_teams: bool,
    /// Enable refactoring services
    pub enable_refactoring: bool,
    /// Enable parsers services
    pub enable_parsers: bool,
    /// Enable generation services
    pub enable_generation: bool,
    /// Enable app config services
    pub enable_app_config: bool,
    /// Enable GitHub services
    pub enable_github: bool,
    /// Enable domain agents services
    pub enable_domain_agents: bool,
    /// Enable local models services
    pub enable_local_models: bool,
    /// Enable CLI services
    pub enable_cli: bool,
    /// Enable monitoring services
    pub enable_monitoring: bool,
}

/// Create a container based on configuration
pub fn create_configured_container(config: &ContainerConfig) -> DIResult<DIContainer> {
    let mut builder = crate::DIContainerBuilder::new();

    // Always register core infrastructure
    builder = builder.register_infrastructure_services()?;
    builder = builder.register_use_cases()?;

    // Conditionally register services based on config
    if config.enable_storage {
        #[cfg(feature = "storage")]
        {
            builder = builder.register_storage_services()?;
        }
    }

    if config.enable_research {
        #[cfg(feature = "research")]
        {
            builder = builder.register_research_services()?;
        }
    }

    if config.enable_workflows {
        #[cfg(feature = "workflows")]
        {
            builder = builder.register_workflow_services()?;
        }
    }

    if config.enable_execution {
        #[cfg(feature = "execution")]
        {
            builder = builder.register_execution_services()?;
        }
    }

    if config.enable_mcp {
        #[cfg(feature = "mcp")]
        {
            builder = builder.register_mcp_services()?;
        }
    }

    if config.enable_tools {
        #[cfg(feature = "tools")]
        {
            builder = builder.register_tool_services()?;
        }
    }

    if config.enable_config {
        #[cfg(feature = "config")]
        {
            builder = builder.register_config_services()?;
        }
    }

    if config.enable_activity_log {
        #[cfg(feature = "activity-log")]
        {
            builder = builder.register_activity_log_services()?;
        }
    }

    if config.enable_orchestration {
        #[cfg(feature = "orchestration")]
        {
            builder = builder.register_orchestration_services()?;
        }
    }

    if config.enable_specs {
        #[cfg(feature = "specs")]
        {
            builder = builder.register_specs_services()?;
        }
    }

    if config.enable_undo_redo {
        #[cfg(feature = "undo-redo")]
        {
            builder = builder.register_undo_redo_services()?;
        }
    }

    if config.enable_vcs {
        #[cfg(feature = "vcs")]
        {
            builder = builder.register_vcs_services()?;
        }
    }

    if config.enable_permissions {
        #[cfg(feature = "permissions")]
        {
            builder = builder.register_permissions_services()?;
        }
    }

    if config.enable_security {
        #[cfg(feature = "security")]
        {
            builder = builder.register_security_services()?;
        }
    }

    if config.enable_cache {
        #[cfg(feature = "cache")]
        {
            builder = builder.register_cache_services()?;
        }
    }

    if config.enable_domain {
        #[cfg(feature = "domain")]
        {
            builder = builder.register_domain_services()?;
        }
    }

    if config.enable_learning {
        #[cfg(feature = "learning")]
        {
            builder = builder.register_learning_services()?;
        }
    }

    if config.enable_industry {
        #[cfg(feature = "industry")]
        {
            builder = builder.register_industry_services()?;
        }
    }

    if config.enable_safety {
        #[cfg(feature = "safety")]
        {
            builder = builder.register_safety_services()?;
        }
    }

    if config.enable_files {
        #[cfg(feature = "files")]
        {
            builder = builder.register_files_services()?;
        }
    }

    if config.enable_themes {
        #[cfg(feature = "themes")]
        {
            builder = builder.register_themes_services()?;
        }
    }

    if config.enable_images {
        #[cfg(feature = "images")]
        {
            builder = builder.register_images_services()?;
        }
    }

    if config.enable_completion {
        #[cfg(feature = "completion")]
        {
            builder = builder.register_completion_services()?;
        }
    }

    if config.enable_lsp {
        #[cfg(feature = "lsp")]
        {
            builder = builder.register_lsp_services()?;
        }
    }

    if config.enable_modes {
        #[cfg(feature = "modes")]
        {
            builder = builder.register_modes_services()?;
        }
    }

    if config.enable_commands {
        #[cfg(feature = "commands")]
        {
            builder = builder.register_commands_services()?;
        }
    }

    if config.enable_hooks {
        #[cfg(feature = "hooks")]
        {
            builder = builder.register_hooks_services()?;
        }
    }

    if config.enable_keybinds {
        #[cfg(feature = "keybinds")]
        {
            builder = builder.register_keybinds_services()?;
        }
    }

    if config.enable_teams {
        #[cfg(feature = "teams")]
        {
            builder = builder.register_teams_services()?;
        }
    }

    if config.enable_refactoring {
        #[cfg(feature = "refactoring")]
        {
            builder = builder.register_refactoring_services()?;
        }
    }

    if config.enable_parsers {
        #[cfg(feature = "parsers")]
        {
            builder = builder.register_parsers_services()?;
        }
    }

    if config.enable_generation {
        #[cfg(feature = "generation")]
        {
            builder = builder.register_generation_services()?;
        }
    }



    if config.enable_github {
        #[cfg(feature = "github")]
        {
            builder = builder.register_github_services()?;
        }
    }

    if config.enable_domain_agents {
        #[cfg(feature = "domain-agents")]
        {
            builder = builder.register_domain_agents_services()?;
        }
    }

    if config.enable_local_models {
        #[cfg(feature = "local-models")]
        {
            builder = builder.register_local_models_services()?;
        }
    }

    if config.enable_cli {
        #[cfg(feature = "cli")]
        {
            builder = builder.register_cli_services()?;
        }
    }

    if config.enable_monitoring {
        #[cfg(feature = "monitoring")]
        {
            builder = builder.register_monitoring_services()?;
        }
    }

    let container = builder.build();
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

    #[cfg(feature = "completion")]
    fn register_completion_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "lsp")]
    fn register_lsp_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "modes")]
    fn register_modes_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "commands")]
    fn register_commands_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "hooks")]
    fn register_hooks_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "keybinds")]
    fn register_keybinds_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "teams")]
    fn register_teams_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "refactoring")]
    fn register_refactoring_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "parsers")]
    fn register_parsers_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "generation")]
    fn register_generation_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "github")]
    fn register_github_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "domain-agents")]
    fn register_domain_agents_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "local-models")]
    fn register_local_models_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "cli")]
    fn register_cli_services(self) -> DIResult<Self> where Self: Sized;

    #[cfg(feature = "monitoring")]
    fn register_monitoring_services(self) -> DIResult<Self> where Self: Sized;
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

    #[cfg(feature = "completion")]
    fn register_completion_services(self) -> DIResult<Self> {
        register_completion_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "lsp")]
    fn register_lsp_services(self) -> DIResult<Self> {
        register_lsp_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "modes")]
    fn register_modes_services(self) -> DIResult<Self> {
        register_modes_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "commands")]
    fn register_commands_services(self) -> DIResult<Self> {
        register_commands_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "hooks")]
    fn register_hooks_services(self) -> DIResult<Self> {
        register_hooks_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "keybinds")]
    fn register_keybinds_services(self) -> DIResult<Self> {
        register_keybinds_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "teams")]
    fn register_teams_services(self) -> DIResult<Self> {
        register_teams_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "refactoring")]
    fn register_refactoring_services(self) -> DIResult<Self> {
        register_refactoring_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "parsers")]
    fn register_parsers_services(self) -> DIResult<Self> {
        register_parsers_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "generation")]
    fn register_generation_services(self) -> DIResult<Self> {
        register_generation_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "config")]
    fn register_app_config_services(self) -> DIResult<Self> {
        register_app_config_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "github")]
    fn register_github_services(self) -> DIResult<Self> {
        register_github_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "domain-agents")]
    fn register_domain_agents_services(self) -> DIResult<Self> {
        register_domain_agents_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "local-models")]
    fn register_local_models_services(self) -> DIResult<Self> {
        register_local_models_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "cli")]
    fn register_cli_services(self) -> DIResult<Self> {
        register_cli_services(&self.container)?;
        Ok(self)
    }

    #[cfg(feature = "monitoring")]
    fn register_monitoring_services(self) -> DIResult<Self> {
        register_monitoring_services(&self.container)?;
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