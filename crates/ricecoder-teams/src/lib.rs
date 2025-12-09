/// RiceCoder Team Collaboration System
///
/// This crate provides team-level standards, shared configurations, and collaborative workflows
/// for development teams. It enables teams to share code review rules, templates, steering documents,
/// and compliance requirements across organization, team, and project levels with inheritance and
/// override capabilities.
///
/// The module integrates with:
/// - ricecoder-storage: Configuration management and path resolution
/// - ricecoder-learning: Rule promotion and analytics
/// - ricecoder-permissions: Access control and audit logging

pub mod access;
pub mod analytics;
pub mod config;
pub mod error;
pub mod manager;
pub mod models;
pub mod rules;
pub mod sync;

// Re-export public types
pub use access::AccessControlManager;
pub use analytics::AnalyticsDashboard;
pub use config::TeamConfigManager;
pub use error::{Result, TeamError};
pub use manager::TeamManager;
pub use models::{
    AdoptionMetrics, AuditLogEntry, CodeReviewRule, ComplianceRequirement, EffectivenessMetrics,
    MergedStandards, RuleScope, SharedRule, StandardsOverride, Team, TeamAnalyticsReport,
    TeamMember, TeamRole, TeamStandards, Template, SteeringDoc,
};
pub use rules::SharedRulesManager;
pub use sync::SyncService;
