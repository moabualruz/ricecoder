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

pub mod errors;
pub mod managers;
pub mod models;

pub use errors::GitHubError;
pub use managers::{
    ApiDocumentation, ApiParameter, ApprovalCondition, AutomationAction, AutomationRule,
    AutomationTrigger, AutomationWorkflow, Changelog, ChangelogEntry, CategorizationResult,
    CodebaseSummary, CodePattern, CodeQualityIssue, CodeReviewAgent, CodeReviewMetrics,
    CodeReviewOperations, CodeReviewResult, CodeReviewStandards, CodeReviewSuggestion,
    ColumnStatus, ConditionalApprovalResult, DetailedProjectReport, DiscussionCategory,
    DiscussionCreationResult, DiscussionInsight, DiscussionManager, DiscussionOperations,
    DiscussionResponse, DiscussionStatusUpdate, DiscussionSummary, DiscussionThread,
    DocumentationCommit, DocumentationCoverage, DocumentationGenerator, DocumentationOperations,
    DocumentationSection, DocumentationTemplate, GistBatchResult, GistCreationResult,
    GistLifecycleResult, GistManager, GistMetadata, GistOperations, GistOrganizationResult,
    GistOptions, GistSearchCriteria, GistSearchResult, GistSharingConfig, GistSharingResult,
    GistUpdateResult, GitHubManager, IssueComment, IssueManager, IssueOperations,
    ImplementationPlan, IssueSeverity, MaintenanceStatus, MaintenanceTask, PlanTask,
    ParsedRequirement, PrComment, PrLink, PrManager, PrOperations, PrOptions, PrReview,
    PrTemplate, PrUpdateOptions, ProgressUpdate, ProjectManager, ProjectMetrics,
    ProjectOperations, ProjectStatusReport, PublishingResult, ReadmeConfig,
    ReleaseHistoryEntry, ReleaseManager, ReleaseNotesOptions, ReleaseOperations,
    ReleaseOptions, ReleasePublishingResult, ReleaseTemplate, RepositoryAnalysis,
    RepositoryAnalyzer, ReportSection, ReviewState, SemanticVersion, StatusChange, SyncResult,
    TaskContext, ThreadComment, TrackingResult,
};
pub use models::{
    Discussion, Gist, Issue, IssueProgressUpdate, IssueStatus, ProjectCard, PullRequest, Release,
    Repository,
};
