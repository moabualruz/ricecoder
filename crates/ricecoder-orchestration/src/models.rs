//! Core data models for workspace orchestration

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Represents a workspace containing multiple projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Root path of the workspace
    pub root: PathBuf,

    /// All projects in the workspace
    pub projects: Vec<Project>,

    /// Dependencies between projects
    pub dependencies: Vec<ProjectDependency>,

    /// Workspace-level configuration
    pub config: WorkspaceConfig,

    /// Workspace metrics and health indicators
    pub metrics: WorkspaceMetrics,
}

/// Represents a single project within a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Path to the project
    pub path: PathBuf,

    /// Project name
    pub name: String,

    /// Type of project (e.g., "rust", "typescript", "python")
    pub project_type: String,

    /// Current version of the project
    pub version: String,

    /// Current status of the project
    pub status: ProjectStatus,
}

/// Status of a project
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectStatus {
    /// Project is healthy and operational
    Healthy,

    /// Project has warnings or minor issues
    Warning,

    /// Project has critical issues
    Critical,

    /// Project status is unknown
    Unknown,
}

/// Represents a dependency between two projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDependency {
    /// Name of the project that depends on another
    pub from: String,

    /// Name of the project being depended on
    pub to: String,

    /// Type of dependency
    pub dependency_type: DependencyType,

    /// Version constraint for the dependency
    pub version_constraint: String,
}

/// Type of dependency between projects
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Direct dependency
    Direct,

    /// Transitive dependency (indirect)
    Transitive,

    /// Development-only dependency
    Dev,
}

/// Workspace-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Workspace rules
    pub rules: Vec<WorkspaceRule>,

    /// Additional settings as JSON
    pub settings: serde_json::Value,
}

/// A rule that applies to the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRule {
    /// Name of the rule
    pub name: String,

    /// Type of rule
    pub rule_type: RuleType,

    /// Whether the rule is enabled
    pub enabled: bool,
}

/// Type of workspace rule
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleType {
    /// Constraint on dependencies
    DependencyConstraint,

    /// Naming convention rule
    NamingConvention,

    /// Architectural boundary rule
    ArchitecturalBoundary,
}

/// Metrics about the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetrics {
    /// Total number of projects
    pub total_projects: usize,

    /// Total number of dependencies
    pub total_dependencies: usize,

    /// Compliance score (0.0 to 1.0)
    pub compliance_score: f64,

    /// Overall health status
    pub health_status: HealthStatus,
}

/// Health status of the workspace
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Workspace is healthy
    Healthy,

    /// Workspace has warnings
    Warning,

    /// Workspace has critical issues
    Critical,
}

/// Report of impact analysis for a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    /// Unique identifier for the change
    pub change_id: String,

    /// Projects affected by the change
    pub affected_projects: Vec<String>,

    /// Level of impact
    pub impact_level: ImpactLevel,

    /// Detailed impact information
    pub details: Vec<ImpactDetail>,
}

/// Level of impact a change has
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImpactLevel {
    /// Low impact
    Low,

    /// Medium impact
    Medium,

    /// High impact
    High,

    /// Critical impact
    Critical,
}

/// Details about how a change impacts a specific project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactDetail {
    /// Name of the affected project
    pub project: String,

    /// Reason for the impact
    pub reason: String,

    /// Required actions to address the impact
    pub required_actions: Vec<String>,
}

/// Represents a transaction for multi-project operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: String,

    /// Operations in the transaction
    pub operations: Vec<Operation>,

    /// Current state of the transaction
    pub state: TransactionState,
}

/// An operation within a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation identifier
    pub id: String,

    /// Project affected by the operation
    pub project: String,

    /// Type of operation
    pub operation_type: String,

    /// Operation data
    pub data: serde_json::Value,
}

/// State of a transaction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction is pending
    Pending,

    /// Transaction has been committed
    Committed,

    /// Transaction has been rolled back
    RolledBack,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            projects: Vec::new(),
            dependencies: Vec::new(),
            config: WorkspaceConfig::default(),
            metrics: WorkspaceMetrics::default(),
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            settings: serde_json::json!({}),
        }
    }
}

impl Default for WorkspaceMetrics {
    fn default() -> Self {
        Self {
            total_projects: 0,
            total_dependencies: 0,
            compliance_score: 1.0,
            health_status: HealthStatus::Healthy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project {
            path: PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        };

        assert_eq!(project.name, "test-project");
        assert_eq!(project.project_type, "rust");
        assert_eq!(project.status, ProjectStatus::Healthy);
    }

    #[test]
    fn test_project_dependency_creation() {
        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        assert_eq!(dep.from, "project-a");
        assert_eq!(dep.to, "project-b");
        assert_eq!(dep.dependency_type, DependencyType::Direct);
    }

    #[test]
    fn test_workspace_default() {
        let workspace = Workspace::default();

        assert_eq!(workspace.projects.len(), 0);
        assert_eq!(workspace.dependencies.len(), 0);
        assert_eq!(workspace.metrics.total_projects, 0);
        assert_eq!(workspace.metrics.health_status, HealthStatus::Healthy);
    }

    #[test]
    fn test_workspace_rule_creation() {
        let rule = WorkspaceRule {
            name: "no-circular-deps".to_string(),
            rule_type: RuleType::DependencyConstraint,
            enabled: true,
        };

        assert_eq!(rule.name, "no-circular-deps");
        assert_eq!(rule.rule_type, RuleType::DependencyConstraint);
        assert!(rule.enabled);
    }

    #[test]
    fn test_impact_report_creation() {
        let report = ImpactReport {
            change_id: "change-123".to_string(),
            affected_projects: vec!["project-a".to_string(), "project-b".to_string()],
            impact_level: ImpactLevel::High,
            details: vec![],
        };

        assert_eq!(report.change_id, "change-123");
        assert_eq!(report.affected_projects.len(), 2);
        assert_eq!(report.impact_level, ImpactLevel::High);
    }

    #[test]
    fn test_transaction_creation() {
        let transaction = Transaction {
            id: "txn-123".to_string(),
            operations: vec![],
            state: TransactionState::Pending,
        };

        assert_eq!(transaction.id, "txn-123");
        assert_eq!(transaction.state, TransactionState::Pending);
    }

    #[test]
    fn test_serialization_project() {
        let project = Project {
            path: PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        };

        let json = serde_json::to_string(&project).expect("serialization failed");
        let deserialized: Project = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.project_type, deserialized.project_type);
    }

    #[test]
    fn test_serialization_workspace() {
        let workspace = Workspace::default();

        let json = serde_json::to_string(&workspace).expect("serialization failed");
        let deserialized: Workspace = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(workspace.projects.len(), deserialized.projects.len());
        assert_eq!(
            workspace.dependencies.len(),
            deserialized.dependencies.len()
        );
    }

    #[test]
    fn test_project_status_variants() {
        assert_eq!(ProjectStatus::Healthy, ProjectStatus::Healthy);
        assert_ne!(ProjectStatus::Healthy, ProjectStatus::Warning);
        assert_ne!(ProjectStatus::Warning, ProjectStatus::Critical);
        assert_ne!(ProjectStatus::Critical, ProjectStatus::Unknown);
    }

    #[test]
    fn test_dependency_type_variants() {
        assert_eq!(DependencyType::Direct, DependencyType::Direct);
        assert_ne!(DependencyType::Direct, DependencyType::Transitive);
        assert_ne!(DependencyType::Transitive, DependencyType::Dev);
    }

    #[test]
    fn test_health_status_variants() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Warning);
        assert_ne!(HealthStatus::Warning, HealthStatus::Critical);
    }

    #[test]
    fn test_impact_level_variants() {
        assert_eq!(ImpactLevel::Low, ImpactLevel::Low);
        assert_ne!(ImpactLevel::Low, ImpactLevel::Medium);
        assert_ne!(ImpactLevel::Medium, ImpactLevel::High);
        assert_ne!(ImpactLevel::High, ImpactLevel::Critical);
    }

    #[test]
    fn test_transaction_state_variants() {
        assert_eq!(TransactionState::Pending, TransactionState::Pending);
        assert_ne!(TransactionState::Pending, TransactionState::Committed);
        assert_ne!(TransactionState::Committed, TransactionState::RolledBack);
    }

    #[test]
    fn test_rule_type_variants() {
        assert_eq!(
            RuleType::DependencyConstraint,
            RuleType::DependencyConstraint
        );
        assert_ne!(RuleType::DependencyConstraint, RuleType::NamingConvention);
        assert_ne!(RuleType::NamingConvention, RuleType::ArchitecturalBoundary);
    }

    #[test]
    fn test_workspace_with_projects() {
        let mut workspace = Workspace::default();
        workspace.projects.push(Project {
            path: PathBuf::from("/path/to/project1"),
            name: "project1".to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        });

        assert_eq!(workspace.projects.len(), 1);
        assert_eq!(workspace.projects[0].name, "project1");
    }

    #[test]
    fn test_workspace_with_dependencies() {
        let mut workspace = Workspace::default();
        workspace.dependencies.push(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        assert_eq!(workspace.dependencies.len(), 1);
        assert_eq!(workspace.dependencies[0].from, "project-a");
    }

    #[test]
    fn test_workspace_metrics_update() {
        let mut metrics = WorkspaceMetrics::default();
        metrics.total_projects = 10;
        metrics.total_dependencies = 15;
        metrics.compliance_score = 0.95;
        metrics.health_status = HealthStatus::Warning;

        assert_eq!(metrics.total_projects, 10);
        assert_eq!(metrics.total_dependencies, 15);
        assert_eq!(metrics.compliance_score, 0.95);
        assert_eq!(metrics.health_status, HealthStatus::Warning);
    }

    #[test]
    fn test_impact_detail_creation() {
        let detail = ImpactDetail {
            project: "project-a".to_string(),
            reason: "Breaking API change".to_string(),
            required_actions: vec!["Update imports".to_string(), "Run tests".to_string()],
        };

        assert_eq!(detail.project, "project-a");
        assert_eq!(detail.reason, "Breaking API change");
        assert_eq!(detail.required_actions.len(), 2);
    }

    #[test]
    fn test_operation_creation() {
        let op = Operation {
            id: "op-123".to_string(),
            project: "project-a".to_string(),
            operation_type: "update".to_string(),
            data: serde_json::json!({"version": "0.2.0"}),
        };

        assert_eq!(op.id, "op-123");
        assert_eq!(op.project, "project-a");
        assert_eq!(op.operation_type, "update");
    }

    #[test]
    fn test_workspace_config_with_rules() {
        let mut config = WorkspaceConfig::default();
        config.rules.push(WorkspaceRule {
            name: "no-circular-deps".to_string(),
            rule_type: RuleType::DependencyConstraint,
            enabled: true,
        });

        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "no-circular-deps");
    }
}
