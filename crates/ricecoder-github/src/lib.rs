//! GitHub Integration for RiceCoder
//!
//! This crate provides comprehensive GitHub API integration for ricecoder, enabling:
//! - Automatic PR creation and management
//! - Issue assignment and tracking
//! - Repository analysis
//! - Project management
//! - Documentation generation
//! - Gist management
//! - Discussion integration
//! - Release management
//! - Code review automation
//! - Dependency management
//! - Webhook integration

pub mod di;
pub mod errors;
pub mod managers;
pub mod models;

pub use errors::GitHubError;
pub use managers::{
    ActionsIntegration, ActionsOperations, ApiDocumentation, ApiParameter, ApprovalCondition,
    AutomationAction, AutomationRule, AutomationTrigger, AutomationWorkflow, BranchCreationResult,
    BranchDeletionResult, BranchInfo, BranchLifecycleResult, BranchManager, BranchProtection,
    CategorizationResult, Changelog, ChangelogEntry, CiFailureDiagnostics, CiResultComment,
    CiResultSummary, CodePattern, CodeQualityIssue, CodeReviewAgent, CodeReviewMetrics,
    CodeReviewOperations, CodeReviewResult, CodeReviewStandards, CodeReviewSuggestion,
    CodebaseSummary, ColumnStatus, ConditionalApprovalResult, DetailedProjectReport,
    DiscussionCategory, DiscussionCreationResult, DiscussionInsight, DiscussionManager,
    DiscussionOperations, DiscussionResponse, DiscussionStatusUpdate, DiscussionSummary,
    DiscussionThread, DocumentationCommit, DocumentationCoverage, DocumentationGenerator,
    DocumentationOperations, DocumentationSection, DocumentationTemplate, EventFilter,
    GistBatchResult, GistCreationResult, GistLifecycleResult, GistManager, GistMetadata,
    GistOperations, GistOptions, GistOrganizationResult, GistSearchCriteria, GistSearchResult,
    GistSharingConfig, GistSharingResult, GistUpdateResult, GitHubManager, ImplementationPlan,
    IssueComment, IssueManager, IssueOperations, IssueSeverity, JobStep, MaintenanceStatus,
    MaintenanceTask, ParsedRequirement, PlanTask, PrComment, PrLink, PrManager, PrOperations,
    PrOptions, PrReview, PrTemplate, PrUpdateOptions, ProgressUpdate, ProjectManager,
    ProjectMetrics, ProjectOperations, ProjectStatusReport, PublishingResult, ReadmeConfig,
    ReleaseHistoryEntry, ReleaseManager, ReleaseNotesOptions, ReleaseOperations, ReleaseOptions,
    ReleasePublishingResult, ReleaseTemplate, ReportSection, RepositoryAnalysis,
    RepositoryAnalyzer, ReviewState, SemanticVersion, StatusChange, SyncResult, TaskContext,
    ThreadComment, TrackingResult, WebhookErrorDetails, WebhookErrorHandlingResult, WebhookEvent,
    WebhookEventLogEntry, WebhookEventLogger, WebhookEventStatistics, WebhookEventType,
    WebhookHandler, WebhookHandlerConfig, WebhookOperations, WebhookProcessingResult,
    WebhookRetryConfig, WorkflowConfig, WorkflowConfigResult, WorkflowIterationResult, WorkflowJob,
    WorkflowRetryResult, WorkflowRun, WorkflowStatus, WorkflowStatusResult, WorkflowTrigger,
    WorkflowTriggerRequest, WorkflowTriggerResult,
};
pub use models::{
    Discussion, Gist, Issue, IssueProgressUpdate, IssueStatus, ProjectCard, PullRequest, Release,
    Repository,
};
