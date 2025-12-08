//! GitHub Managers
//!
//! Specialized managers for different GitHub operations

pub mod code_review_agent;
pub mod code_review_operations;
pub mod discussion_manager;
pub mod discussion_operations;
pub mod documentation_generator;
pub mod documentation_operations;
pub mod github_manager;
pub mod gist_manager;
pub mod gist_operations;
pub mod issue_manager;
pub mod issue_operations;
pub mod pr_manager;
pub mod pr_operations;
pub mod project_manager;
pub mod project_operations;
pub mod release_manager;
pub mod release_operations;
pub mod repository_analyzer;

pub use code_review_agent::{
    CodeQualityIssue, CodeReviewAgent, CodeReviewResult, CodeReviewStandards,
    CodeReviewSuggestion, IssueSeverity,
};
pub use code_review_operations::{
    ApprovalCondition, CodeReviewMetrics, CodeReviewOperations, ConditionalApprovalResult,
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
pub use github_manager::GitHubManager;
pub use gist_manager::{
    GistCreationResult, GistLifecycleResult, GistManager, GistMetadata, GistOptions,
    GistUpdateResult,
};
pub use gist_operations::{
    GistBatchResult, GistOperations, GistOrganizationResult, GistSearchCriteria,
    GistSearchResult, GistSharingConfig, GistSharingResult,
};
pub use issue_manager::{IssueManager, ImplementationPlan, ParsedRequirement, PlanTask};
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
    CodebaseSummary, CodePattern, RepositoryAnalysis, RepositoryAnalyzer,
};
