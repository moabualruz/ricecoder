//! GitHub Managers
//!
//! Specialized managers for different GitHub operations

pub mod actions_integration;
pub mod actions_operations;
pub mod branch_manager;
pub mod code_review_agent;
pub mod code_review_operations;
pub mod dependency_manager;
pub mod dependency_operations;
pub mod discussion_manager;
pub mod discussion_operations;
pub mod documentation_generator;
pub mod documentation_operations;
pub mod gist_manager;
pub mod gist_operations;
pub mod github_manager;
pub mod issue_manager;
pub mod issue_operations;
pub mod pr_manager;
pub mod pr_operations;
pub mod project_manager;
pub mod project_operations;
pub mod release_manager;
pub mod release_operations;
pub mod repository_analyzer;
pub mod webhook_handler;
pub mod webhook_operations;

pub use actions_integration::{
    ActionsIntegration, CiFailureDiagnostics, CiResultSummary, JobStep, WorkflowJob,
    WorkflowRetryResult, WorkflowRun, WorkflowStatus, WorkflowStatusResult, WorkflowTriggerRequest,
    WorkflowTriggerResult,
};
pub use actions_operations::{
    ActionsOperations, CiResultComment, WorkflowConfig, WorkflowConfigResult,
    WorkflowIterationResult,
};
pub use branch_manager::{
    BranchCreationResult, BranchDeletionResult, BranchInfo, BranchLifecycleResult, BranchManager,
    BranchProtection,
};
pub use code_review_agent::{
    CodeQualityIssue, CodeReviewAgent, CodeReviewResult, CodeReviewStandards, CodeReviewSuggestion,
    IssueSeverity,
};
pub use code_review_operations::{
    ApprovalCondition, CodeReviewMetrics, CodeReviewOperations, ConditionalApprovalResult,
};
pub use dependency_manager::{
    Dependency, DependencyError, DependencyManager, DependencyScanResult, DependencyUpdatePrResult,
    DependencyUpdateSuggestion, DependencyUpdateVerificationResult, UpdateReason, UpdateRiskLevel,
    Vulnerability, VulnerabilityReport, VulnerabilitySeverity,
};
pub use dependency_operations::{
    BuildVerificationResult, DependencyOperations, DependencyPinningResult, PinningConfig,
    SecurityReport, UpdatePriority, UpdateRecommendation, VulnerabilityInfo,
};
pub use discussion_manager::{
    DiscussionCreationResult, DiscussionInsight, DiscussionManager, DiscussionResponse,
    DiscussionStatusUpdate, DiscussionSummary,
};
pub use discussion_operations::{
    CategorizationResult, DiscussionCategory, DiscussionOperations, DiscussionThread,
    ThreadComment, TrackingResult,
};
pub use documentation_generator::{
    ApiDocumentation, ApiParameter, DocumentationCoverage, DocumentationGenerator,
    DocumentationSection, ReadmeConfig, SyncResult,
};
pub use documentation_operations::{
    DocumentationCommit, DocumentationOperations, DocumentationTemplate, MaintenanceStatus,
    MaintenanceTask, PublishingResult,
};
pub use gist_manager::{
    GistCreationResult, GistLifecycleResult, GistManager, GistMetadata, GistOptions,
    GistUpdateResult,
};
pub use gist_operations::{
    GistBatchResult, GistOperations, GistOrganizationResult, GistSearchCriteria, GistSearchResult,
    GistSharingConfig, GistSharingResult,
};
pub use github_manager::GitHubManager;
pub use issue_manager::{ImplementationPlan, IssueManager, ParsedRequirement, PlanTask};
pub use issue_operations::{IssueComment, IssueOperations, PrLink, StatusChange};
pub use pr_manager::{PrManager, PrOptions, PrTemplate, TaskContext};
pub use pr_operations::{
    PrComment, PrOperations, PrReview, PrUpdateOptions, ProgressUpdate, ReviewState,
};
pub use project_manager::{
    AutomationRule, ColumnStatus, ProjectManager, ProjectMetrics, ProjectStatusReport,
};
pub use project_operations::{
    AutomationAction, AutomationTrigger, AutomationWorkflow, DetailedProjectReport,
    ProjectOperations, ReportSection,
};
pub use release_manager::{
    ReleaseHistoryEntry, ReleaseManager, ReleaseNotesOptions, ReleaseOptions, SemanticVersion,
};
pub use release_operations::{
    Changelog, ChangelogEntry, ReleaseOperations, ReleasePublishingResult, ReleaseTemplate,
};
pub use repository_analyzer::{
    CodePattern, CodebaseSummary, RepositoryAnalysis, RepositoryAnalyzer,
};
pub use webhook_handler::{
    EventFilter, WebhookEvent, WebhookEventType, WebhookHandler, WebhookHandlerConfig,
    WebhookProcessingResult, WorkflowTrigger,
};
pub use webhook_operations::{
    WebhookErrorDetails, WebhookErrorHandlingResult, WebhookEventLogEntry, WebhookEventLogger,
    WebhookEventStatistics, WebhookOperations, WebhookRetryConfig,
};
