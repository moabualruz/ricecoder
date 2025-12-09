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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test team
    fn create_test_team() -> Team {
        Team {
            id: "team-1".to_string(),
            name: "Test Team".to_string(),
            organization_id: Some("org-1".to_string()),
            members: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // Helper function to create a test team member
    fn create_test_member() -> TeamMember {
        TeamMember {
            id: "member-1".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            role: TeamRole::Member,
            joined_at: Utc::now(),
        }
    }

    // Helper function to create test standards
    fn create_test_standards() -> TeamStandards {
        TeamStandards {
            id: "standards-1".to_string(),
            team_id: "team-1".to_string(),
            code_review_rules: vec![CodeReviewRule {
                id: "rule-1".to_string(),
                name: "Test Rule".to_string(),
                description: "A test rule".to_string(),
                enabled: true,
            }],
            templates: vec![Template {
                id: "template-1".to_string(),
                name: "Test Template".to_string(),
                description: "A test template".to_string(),
                content: "template content".to_string(),
            }],
            steering_docs: vec![SteeringDoc {
                id: "doc-1".to_string(),
                name: "Test Doc".to_string(),
                content: "doc content".to_string(),
            }],
            compliance_requirements: vec![ComplianceRequirement {
                id: "compliance-1".to_string(),
                name: "Test Compliance".to_string(),
                description: "A test compliance requirement".to_string(),
            }],
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_team_serialization_to_json() {
        let team = create_test_team();
        let json = serde_json::to_string(&team).expect("Failed to serialize to JSON");
        assert!(json.contains("\"id\":\"team-1\""));
        assert!(json.contains("\"name\":\"Test Team\""));
    }

    #[test]
    fn test_team_deserialization_from_json() {
        let json = r#"{"id":"team-1","name":"Test Team","organization_id":"org-1","members":[],"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#;
        let team: Team = serde_json::from_str(json).expect("Failed to deserialize from JSON");
        assert_eq!(team.id, "team-1");
        assert_eq!(team.name, "Test Team");
        assert_eq!(team.organization_id, Some("org-1".to_string()));
    }

    #[test]
    fn test_team_serialization_to_yaml() {
        let team = create_test_team();
        let yaml = serde_yaml::to_string(&team).expect("Failed to serialize to YAML");
        assert!(yaml.contains("id: team-1"));
        assert!(yaml.contains("name: Test Team"));
    }

    #[test]
    fn test_team_deserialization_from_yaml() {
        let yaml = r#"
id: team-1
name: Test Team
organization_id: org-1
members: []
created_at: 2024-01-01T00:00:00Z
updated_at: 2024-01-01T00:00:00Z
"#;
        let team: Team = serde_yaml::from_str(yaml).expect("Failed to deserialize from YAML");
        assert_eq!(team.id, "team-1");
        assert_eq!(team.name, "Test Team");
    }

    #[test]
    fn test_team_member_serialization_to_json() {
        let member = create_test_member();
        let json = serde_json::to_string(&member).expect("Failed to serialize to JSON");
        assert!(json.contains("\"id\":\"member-1\""));
        assert!(json.contains("\"email\":\"john@example.com\""));
    }

    #[test]
    fn test_team_member_deserialization_from_json() {
        let json = r#"{"id":"member-1","name":"John Doe","email":"john@example.com","role":"Member","joined_at":"2024-01-01T00:00:00Z"}"#;
        let member: TeamMember = serde_json::from_str(json).expect("Failed to deserialize from JSON");
        assert_eq!(member.id, "member-1");
        assert_eq!(member.name, "John Doe");
        assert_eq!(member.role, TeamRole::Member);
    }

    #[test]
    fn test_team_role_as_str() {
        assert_eq!(TeamRole::Admin.as_str(), "admin");
        assert_eq!(TeamRole::Member.as_str(), "member");
        assert_eq!(TeamRole::Viewer.as_str(), "viewer");
    }

    #[test]
    fn test_team_role_from_str() {
        assert_eq!(TeamRole::from_str("admin"), Some(TeamRole::Admin));
        assert_eq!(TeamRole::from_str("member"), Some(TeamRole::Member));
        assert_eq!(TeamRole::from_str("viewer"), Some(TeamRole::Viewer));
        assert_eq!(TeamRole::from_str("invalid"), None);
    }

    #[test]
    fn test_team_standards_serialization_to_json() {
        let standards = create_test_standards();
        let json = serde_json::to_string(&standards).expect("Failed to serialize to JSON");
        assert!(json.contains("\"id\":\"standards-1\""));
        assert!(json.contains("\"team_id\":\"team-1\""));
        assert!(json.contains("\"version\":1"));
    }

    #[test]
    fn test_team_standards_deserialization_from_json() {
        let standards = create_test_standards();
        let json = serde_json::to_string(&standards).expect("Failed to serialize");
        let deserialized: TeamStandards =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.id, standards.id);
        assert_eq!(deserialized.team_id, standards.team_id);
        assert_eq!(deserialized.version, standards.version);
        assert_eq!(deserialized.code_review_rules.len(), 1);
        assert_eq!(deserialized.templates.len(), 1);
        assert_eq!(deserialized.steering_docs.len(), 1);
        assert_eq!(deserialized.compliance_requirements.len(), 1);
    }

    #[test]
    fn test_team_standards_serialization_to_yaml() {
        let standards = create_test_standards();
        let yaml = serde_yaml::to_string(&standards).expect("Failed to serialize to YAML");
        assert!(yaml.contains("id: standards-1"));
        assert!(yaml.contains("team_id: team-1"));
        assert!(yaml.contains("version: 1"));
    }

    #[test]
    fn test_team_standards_deserialization_from_yaml() {
        let standards = create_test_standards();
        let yaml = serde_yaml::to_string(&standards).expect("Failed to serialize");
        let deserialized: TeamStandards =
            serde_yaml::from_str(&yaml).expect("Failed to deserialize");
        assert_eq!(deserialized.id, standards.id);
        assert_eq!(deserialized.team_id, standards.team_id);
        assert_eq!(deserialized.version, standards.version);
    }

    #[test]
    fn test_shared_rule_serialization_to_json() {
        let rule = SharedRule {
            id: "rule-1".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule".to_string(),
            scope: RuleScope::Team,
            enforced: true,
            promoted_by: "admin-1".to_string(),
            promoted_at: Utc::now(),
            version: 1,
        };
        let json = serde_json::to_string(&rule).expect("Failed to serialize to JSON");
        assert!(json.contains("\"id\":\"rule-1\""));
        assert!(json.contains("\"scope\":\"Team\""));
    }

    #[test]
    fn test_shared_rule_deserialization_from_json() {
        let json = r#"{"id":"rule-1","name":"Test Rule","description":"A test rule","scope":"Team","enforced":true,"promoted_by":"admin-1","promoted_at":"2024-01-01T00:00:00Z","version":1}"#;
        let rule: SharedRule = serde_json::from_str(json).expect("Failed to deserialize from JSON");
        assert_eq!(rule.id, "rule-1");
        assert_eq!(rule.scope, RuleScope::Team);
        assert_eq!(rule.version, 1);
    }

    #[test]
    fn test_rule_scope_as_str() {
        assert_eq!(RuleScope::Project.as_str(), "project");
        assert_eq!(RuleScope::Team.as_str(), "team");
        assert_eq!(RuleScope::Organization.as_str(), "organization");
    }

    #[test]
    fn test_rule_scope_from_str() {
        assert_eq!(RuleScope::from_str("project"), Some(RuleScope::Project));
        assert_eq!(RuleScope::from_str("team"), Some(RuleScope::Team));
        assert_eq!(RuleScope::from_str("organization"), Some(RuleScope::Organization));
        assert_eq!(RuleScope::from_str("invalid"), None);
    }

    #[test]
    fn test_adoption_metrics_serialization() {
        let metrics = AdoptionMetrics {
            rule_id: "rule-1".to_string(),
            total_members: 10,
            adopting_members: 8,
            adoption_percentage: 80.0,
            adoption_trend: vec![(Utc::now(), 75.0), (Utc::now(), 80.0)],
        };
        let json = serde_json::to_string(&metrics).expect("Failed to serialize to JSON");
        assert!(json.contains("\"rule_id\":\"rule-1\""));
        assert!(json.contains("\"total_members\":10"));
        assert!(json.contains("\"adopting_members\":8"));
    }

    #[test]
    fn test_effectiveness_metrics_serialization() {
        let metrics = EffectivenessMetrics {
            rule_id: "rule-1".to_string(),
            positive_outcomes: 15,
            negative_outcomes: 2,
            effectiveness_score: 0.88,
            impact_trend: vec![(Utc::now(), 0.85), (Utc::now(), 0.88)],
        };
        let json = serde_json::to_string(&metrics).expect("Failed to serialize to JSON");
        assert!(json.contains("\"rule_id\":\"rule-1\""));
        assert!(json.contains("\"positive_outcomes\":15"));
    }

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            id: "log-1".to_string(),
            team_id: "team-1".to_string(),
            user_id: "user-1".to_string(),
            action: "create_rule".to_string(),
            resource: "rule-1".to_string(),
            result: "success".to_string(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&entry).expect("Failed to serialize to JSON");
        assert!(json.contains("\"id\":\"log-1\""));
        assert!(json.contains("\"action\":\"create_rule\""));
    }

    #[test]
    fn test_team_analytics_report_serialization() {
        let report = TeamAnalyticsReport {
            team_id: "team-1".to_string(),
            total_members: 10,
            adoption_metrics: vec![],
            effectiveness_metrics: vec![],
            generated_at: Utc::now(),
        };
        let json = serde_json::to_string(&report).expect("Failed to serialize to JSON");
        assert!(json.contains("\"team_id\":\"team-1\""));
        assert!(json.contains("\"total_members\":10"));
    }

    #[test]
    fn test_round_trip_team_json() {
        let original = create_test_team();
        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: Team = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.organization_id, deserialized.organization_id);
    }

    #[test]
    fn test_round_trip_team_yaml() {
        let original = create_test_team();
        let yaml = serde_yaml::to_string(&original).expect("Failed to serialize");
        let deserialized: Team = serde_yaml::from_str(&yaml).expect("Failed to deserialize");
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.organization_id, deserialized.organization_id);
    }

    #[test]
    fn test_round_trip_standards_json() {
        let original = create_test_standards();
        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: TeamStandards = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.team_id, deserialized.team_id);
        assert_eq!(original.version, deserialized.version);
        assert_eq!(original.code_review_rules.len(), deserialized.code_review_rules.len());
    }

    #[test]
    fn test_round_trip_standards_yaml() {
        let original = create_test_standards();
        let yaml = serde_yaml::to_string(&original).expect("Failed to serialize");
        let deserialized: TeamStandards = serde_yaml::from_str(&yaml).expect("Failed to deserialize");
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.team_id, deserialized.team_id);
        assert_eq!(original.version, deserialized.version);
    }

    #[test]
    fn test_team_member_with_different_roles() {
        let admin = TeamMember {
            id: "admin-1".to_string(),
            name: "Admin User".to_string(),
            email: "admin@example.com".to_string(),
            role: TeamRole::Admin,
            joined_at: Utc::now(),
        };

        let member = TeamMember {
            id: "member-1".to_string(),
            name: "Regular Member".to_string(),
            email: "member@example.com".to_string(),
            role: TeamRole::Member,
            joined_at: Utc::now(),
        };

        let viewer = TeamMember {
            id: "viewer-1".to_string(),
            name: "Viewer User".to_string(),
            email: "viewer@example.com".to_string(),
            role: TeamRole::Viewer,
            joined_at: Utc::now(),
        };

        assert_eq!(admin.role, TeamRole::Admin);
        assert_eq!(member.role, TeamRole::Member);
        assert_eq!(viewer.role, TeamRole::Viewer);
    }

    #[test]
    fn test_standards_override_serialization() {
        let override_data = StandardsOverride {
            project_id: "project-1".to_string(),
            overridden_standards: vec!["rule-1".to_string(), "rule-2".to_string()],
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&override_data).expect("Failed to serialize to JSON");
        assert!(json.contains("\"project_id\":\"project-1\""));
        assert!(json.contains("\"overridden_standards\""));
    }

    #[test]
    fn test_merged_standards_serialization() {
        let merged = MergedStandards {
            organization_standards: Some(create_test_standards()),
            team_standards: Some(create_test_standards()),
            project_standards: None,
            final_standards: create_test_standards(),
        };
        let json = serde_json::to_string(&merged).expect("Failed to serialize to JSON");
        assert!(json.contains("\"organization_standards\""));
        assert!(json.contains("\"team_standards\""));
        assert!(json.contains("\"final_standards\""));
    }
}
