/// Data models for the team collaboration system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a team in the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub organization_id: Option<String>,
    pub members: Vec<TeamMember>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a member of a team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

/// Role of a team member
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamRole {
    Admin,
    Member,
    Viewer,
}

impl TeamRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamRole::Admin => "admin",
            TeamRole::Member => "member",
            TeamRole::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(TeamRole::Admin),
            "member" => Some(TeamRole::Member),
            "viewer" => Some(TeamRole::Viewer),
            _ => None,
        }
    }
}

/// Team standards and governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStandards {
    pub id: String,
    pub team_id: String,
    pub code_review_rules: Vec<CodeReviewRule>,
    pub templates: Vec<Template>,
    pub steering_docs: Vec<SteeringDoc>,
    pub compliance_requirements: Vec<ComplianceRequirement>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Code review rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

/// Project template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
}

/// Steering document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringDoc {
    pub id: String,
    pub name: String,
    pub content: String,
}

/// Compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Shared rule that can be promoted across scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: RuleScope,
    pub enforced: bool,
    pub promoted_by: String,
    pub promoted_at: DateTime<Utc>,
    pub version: u32,
}

/// Scope of a rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleScope {
    Project,
    Team,
    Organization,
}

impl RuleScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleScope::Project => "project",
            RuleScope::Team => "team",
            RuleScope::Organization => "organization",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "project" => Some(RuleScope::Project),
            "team" => Some(RuleScope::Team),
            "organization" => Some(RuleScope::Organization),
            _ => None,
        }
    }
}

/// Adoption metrics for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionMetrics {
    pub rule_id: String,
    pub total_members: u32,
    pub adopting_members: u32,
    pub adoption_percentage: f64,
    pub adoption_trend: Vec<(DateTime<Utc>, f64)>,
}

/// Effectiveness metrics for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessMetrics {
    pub rule_id: String,
    pub positive_outcomes: u32,
    pub negative_outcomes: u32,
    pub effectiveness_score: f64,
    pub impact_trend: Vec<(DateTime<Utc>, f64)>,
}

/// Standards override for project-level customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardsOverride {
    pub project_id: String,
    pub overridden_standards: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Merged standards from hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedStandards {
    pub organization_standards: Option<TeamStandards>,
    pub team_standards: Option<TeamStandards>,
    pub project_standards: Option<TeamStandards>,
    pub final_standards: TeamStandards,
}

/// Audit log entry for permission changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub result: String,
    pub timestamp: DateTime<Utc>,
}

/// Team analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnalyticsReport {
    pub team_id: String,
    pub total_members: u32,
    pub adoption_metrics: Vec<AdoptionMetrics>,
    pub effectiveness_metrics: Vec<EffectivenessMetrics>,
    pub generated_at: DateTime<Utc>,
}
