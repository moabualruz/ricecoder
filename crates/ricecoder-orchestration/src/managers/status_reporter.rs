//! Status reporting for workspace metrics and health indicators

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    models::{HealthStatus, ProjectStatus, Workspace, WorkspaceMetrics},
    OrchestrationError,
};

/// Collects and reports workspace metrics and health indicators
#[derive(Debug, Clone)]
pub struct StatusReporter {
    /// Workspace being reported on
    workspace: Workspace,
}

/// Detailed status report for a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    /// Overall workspace health status
    pub health_status: HealthStatus,

    /// Compliance score (0.0 to 1.0)
    pub compliance_score: f64,

    /// Total number of projects
    pub total_projects: usize,

    /// Total number of dependencies
    pub total_dependencies: usize,

    /// Number of healthy projects
    pub healthy_projects: usize,

    /// Number of projects with warnings
    pub warning_projects: usize,

    /// Number of projects with critical issues
    pub critical_projects: usize,

    /// Number of unknown status projects
    pub unknown_projects: usize,

    /// Project status breakdown
    pub project_statuses: HashMap<String, ProjectStatus>,

    /// Aggregated metrics
    pub metrics: WorkspaceMetrics,

    /// Timestamp of the report (ISO 8601 format)
    pub timestamp: String,
}

/// Aggregated metrics for the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Average project health (0.0 to 1.0)
    pub average_health: f64,

    /// Percentage of healthy projects
    pub healthy_percentage: f64,

    /// Percentage of projects with warnings
    pub warning_percentage: f64,

    /// Percentage of projects with critical issues
    pub critical_percentage: f64,

    /// Average number of dependencies per project
    pub avg_dependencies_per_project: f64,

    /// Maximum dependencies for any single project
    pub max_dependencies: usize,

    /// Minimum dependencies for any single project
    pub min_dependencies: usize,

    /// Total number of rules
    pub total_rules: usize,

    /// Number of enabled rules
    pub enabled_rules: usize,

    /// Number of disabled rules
    pub disabled_rules: usize,
}

/// Health indicator for a specific project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealthIndicator {
    /// Project name
    pub name: String,

    /// Current status
    pub status: ProjectStatus,

    /// Number of dependencies this project has
    pub dependency_count: usize,

    /// Number of projects depending on this project
    pub dependent_count: usize,

    /// Compliance with workspace rules
    pub rule_compliance: f64,
}

impl StatusReporter {
    /// Creates a new status reporter for a workspace
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }

    /// Generates a comprehensive status report
    pub fn generate_report(&self) -> Result<StatusReport, OrchestrationError> {
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Count projects by status
        let mut healthy_count = 0;
        let mut warning_count = 0;
        let mut critical_count = 0;
        let mut unknown_count = 0;
        let mut project_statuses = HashMap::new();

        for project in &self.workspace.projects {
            project_statuses.insert(project.name.clone(), project.status);

            match project.status {
                ProjectStatus::Healthy => healthy_count += 1,
                ProjectStatus::Warning => warning_count += 1,
                ProjectStatus::Critical => critical_count += 1,
                ProjectStatus::Unknown => unknown_count += 1,
            }
        }

        let report = StatusReport {
            health_status: self.workspace.metrics.health_status,
            compliance_score: self.workspace.metrics.compliance_score,
            total_projects: self.workspace.projects.len(),
            total_dependencies: self.workspace.dependencies.len(),
            healthy_projects: healthy_count,
            warning_projects: warning_count,
            critical_projects: critical_count,
            unknown_projects: unknown_count,
            project_statuses,
            metrics: self.workspace.metrics.clone(),
            timestamp,
        };

        Ok(report)
    }

    /// Collects aggregated metrics for the workspace
    pub fn collect_metrics(&self) -> Result<AggregatedMetrics, OrchestrationError> {
        let total_projects = self.workspace.projects.len() as f64;

        if total_projects == 0.0 {
            return Ok(AggregatedMetrics {
                average_health: 1.0,
                healthy_percentage: 100.0,
                warning_percentage: 0.0,
                critical_percentage: 0.0,
                avg_dependencies_per_project: 0.0,
                max_dependencies: 0,
                min_dependencies: 0,
                total_rules: self.workspace.config.rules.len(),
                enabled_rules: self
                    .workspace
                    .config
                    .rules
                    .iter()
                    .filter(|r| r.enabled)
                    .count(),
                disabled_rules: self
                    .workspace
                    .config
                    .rules
                    .iter()
                    .filter(|r| !r.enabled)
                    .count(),
            });
        }

        // Count projects by status
        let mut healthy_count = 0.0;
        let mut warning_count = 0.0;
        let mut critical_count = 0.0;

        for project in &self.workspace.projects {
            match project.status {
                ProjectStatus::Healthy => healthy_count += 1.0,
                ProjectStatus::Warning => warning_count += 1.0,
                ProjectStatus::Critical => critical_count += 1.0,
                ProjectStatus::Unknown => {}
            }
        }

        // Calculate dependencies per project
        let mut project_dep_counts: HashMap<String, usize> = HashMap::new();
        for dep in &self.workspace.dependencies {
            *project_dep_counts.entry(dep.from.clone()).or_insert(0) += 1;
        }

        let (max_deps, min_deps) = if project_dep_counts.is_empty() {
            (0, 0)
        } else {
            let max = *project_dep_counts.values().max().unwrap_or(&0);
            let min = *project_dep_counts.values().min().unwrap_or(&0);
            (max, min)
        };

        let avg_deps = if self.workspace.projects.is_empty() {
            0.0
        } else {
            self.workspace.dependencies.len() as f64 / self.workspace.projects.len() as f64
        };

        Ok(AggregatedMetrics {
            average_health: self.workspace.metrics.compliance_score,
            healthy_percentage: (healthy_count / total_projects) * 100.0,
            warning_percentage: (warning_count / total_projects) * 100.0,
            critical_percentage: (critical_count / total_projects) * 100.0,
            avg_dependencies_per_project: avg_deps,
            max_dependencies: max_deps,
            min_dependencies: min_deps,
            total_rules: self.workspace.config.rules.len(),
            enabled_rules: self
                .workspace
                .config
                .rules
                .iter()
                .filter(|r| r.enabled)
                .count(),
            disabled_rules: self
                .workspace
                .config
                .rules
                .iter()
                .filter(|r| !r.enabled)
                .count(),
        })
    }

    /// Gets health indicators for all projects
    pub fn get_project_health_indicators(
        &self,
    ) -> Result<Vec<ProjectHealthIndicator>, OrchestrationError> {
        let mut indicators = Vec::new();

        for project in &self.workspace.projects {
            // Count dependencies for this project
            let dependency_count = self
                .workspace
                .dependencies
                .iter()
                .filter(|d| d.from == project.name)
                .count();

            // Count projects depending on this project
            let dependent_count = self
                .workspace
                .dependencies
                .iter()
                .filter(|d| d.to == project.name)
                .count();

            // Calculate rule compliance (simplified: 1.0 if no violations)
            let rule_compliance = self.workspace.metrics.compliance_score;

            indicators.push(ProjectHealthIndicator {
                name: project.name.clone(),
                status: project.status,
                dependency_count,
                dependent_count,
                rule_compliance,
            });
        }

        Ok(indicators)
    }

    /// Tracks project health over time (returns current snapshot)
    pub fn track_project_health(
        &self,
        project_name: &str,
    ) -> Result<ProjectHealthIndicator, OrchestrationError> {
        let project = self
            .workspace
            .projects
            .iter()
            .find(|p| p.name == project_name)
            .ok_or_else(|| OrchestrationError::ProjectNotFound(project_name.to_string()))?;

        let dependency_count = self
            .workspace
            .dependencies
            .iter()
            .filter(|d| d.from == project.name)
            .count();

        let dependent_count = self
            .workspace
            .dependencies
            .iter()
            .filter(|d| d.to == project.name)
            .count();

        Ok(ProjectHealthIndicator {
            name: project.name.clone(),
            status: project.status,
            dependency_count,
            dependent_count,
            rule_compliance: self.workspace.metrics.compliance_score,
        })
    }

    /// Generates a summary of workspace compliance
    pub fn generate_compliance_summary(&self) -> Result<ComplianceSummary, OrchestrationError> {
        let total_rules = self.workspace.config.rules.len();
        let enabled_rules = self
            .workspace
            .config
            .rules
            .iter()
            .filter(|r| r.enabled)
            .count();

        let mut violations = Vec::new();

        // Check for critical projects
        for project in &self.workspace.projects {
            if project.status == ProjectStatus::Critical {
                violations.push(format!("Project '{}' has critical status", project.name));
            }
        }

        let compliance_score = self.workspace.metrics.compliance_score;
        let is_compliant = compliance_score >= 0.8 && violations.is_empty();

        Ok(ComplianceSummary {
            total_rules,
            enabled_rules,
            compliance_score,
            is_compliant,
            violations,
        })
    }
}

/// Summary of workspace compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    /// Total number of rules
    pub total_rules: usize,

    /// Number of enabled rules
    pub enabled_rules: usize,

    /// Overall compliance score
    pub compliance_score: f64,

    /// Whether the workspace is compliant
    pub is_compliant: bool,

    /// List of compliance violations
    pub violations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DependencyType, Project, ProjectDependency, RuleType, WorkspaceConfig, WorkspaceRule,
    };

    fn create_test_workspace() -> Workspace {
        let mut workspace = Workspace::default();

        workspace.projects = vec![
            Project {
                path: "/path/to/project1".into(),
                name: "project1".to_string(),
                project_type: "rust".to_string(),
                version: "0.1.0".to_string(),
                status: ProjectStatus::Healthy,
            },
            Project {
                path: "/path/to/project2".into(),
                name: "project2".to_string(),
                project_type: "rust".to_string(),
                version: "0.1.0".to_string(),
                status: ProjectStatus::Warning,
            },
            Project {
                path: "/path/to/project3".into(),
                name: "project3".to_string(),
                project_type: "rust".to_string(),
                version: "0.1.0".to_string(),
                status: ProjectStatus::Healthy,
            },
        ];

        workspace.dependencies = vec![
            ProjectDependency {
                from: "project1".to_string(),
                to: "project2".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            },
            ProjectDependency {
                from: "project2".to_string(),
                to: "project3".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            },
        ];

        workspace.config = WorkspaceConfig {
            rules: vec![
                WorkspaceRule {
                    name: "no-circular-deps".to_string(),
                    rule_type: RuleType::DependencyConstraint,
                    enabled: true,
                },
                WorkspaceRule {
                    name: "naming-convention".to_string(),
                    rule_type: RuleType::NamingConvention,
                    enabled: true,
                },
            ],
            settings: serde_json::json!({}),
        };

        workspace.metrics = WorkspaceMetrics {
            total_projects: 3,
            total_dependencies: 2,
            compliance_score: 0.95,
            health_status: HealthStatus::Healthy,
        };

        workspace
    }

    #[test]
    fn test_status_reporter_creation() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        assert_eq!(reporter.workspace.projects.len(), 3);
    }

    #[test]
    fn test_generate_report() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let report = reporter
            .generate_report()
            .expect("report generation failed");

        assert_eq!(report.total_projects, 3);
        assert_eq!(report.total_dependencies, 2);
        assert_eq!(report.healthy_projects, 2);
        assert_eq!(report.warning_projects, 1);
        assert_eq!(report.critical_projects, 0);
        assert_eq!(report.compliance_score, 0.95);
    }

    #[test]
    fn test_collect_metrics() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let metrics = reporter
            .collect_metrics()
            .expect("metrics collection failed");

        assert_eq!(metrics.total_rules, 2);
        assert_eq!(metrics.enabled_rules, 2);
        assert_eq!(metrics.disabled_rules, 0);
        assert!(metrics.healthy_percentage > 0.0);
        assert!(metrics.warning_percentage > 0.0);
    }

    #[test]
    fn test_get_project_health_indicators() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let indicators = reporter
            .get_project_health_indicators()
            .expect("health indicators failed");

        assert_eq!(indicators.len(), 3);
        assert_eq!(indicators[0].name, "project1");
        assert_eq!(indicators[0].status, ProjectStatus::Healthy);
    }

    #[test]
    fn test_track_project_health() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let health = reporter
            .track_project_health("project1")
            .expect("health tracking failed");

        assert_eq!(health.name, "project1");
        assert_eq!(health.status, ProjectStatus::Healthy);
    }

    #[test]
    fn test_track_project_health_not_found() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let result = reporter.track_project_health("nonexistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_compliance_summary() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let summary = reporter
            .generate_compliance_summary()
            .expect("compliance summary failed");

        assert_eq!(summary.total_rules, 2);
        assert_eq!(summary.enabled_rules, 2);
        assert!(summary.is_compliant);
    }

    #[test]
    fn test_empty_workspace() {
        let workspace = Workspace::default();
        let reporter = StatusReporter::new(workspace);

        let report = reporter
            .generate_report()
            .expect("report generation failed");

        assert_eq!(report.total_projects, 0);
        assert_eq!(report.total_dependencies, 0);
    }

    #[test]
    fn test_metrics_with_empty_workspace() {
        let workspace = Workspace::default();
        let reporter = StatusReporter::new(workspace);

        let metrics = reporter
            .collect_metrics()
            .expect("metrics collection failed");

        assert_eq!(metrics.avg_dependencies_per_project, 0.0);
        assert_eq!(metrics.max_dependencies, 0);
    }

    #[test]
    fn test_report_timestamp() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let report = reporter
            .generate_report()
            .expect("report generation failed");

        // Verify timestamp is in ISO 8601 format
        assert!(!report.timestamp.is_empty());
        assert!(report.timestamp.contains('T'));
    }

    #[test]
    fn test_project_status_breakdown() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let report = reporter
            .generate_report()
            .expect("report generation failed");

        assert_eq!(report.project_statuses.len(), 3);
        assert_eq!(
            report.project_statuses.get("project1"),
            Some(&ProjectStatus::Healthy)
        );
        assert_eq!(
            report.project_statuses.get("project2"),
            Some(&ProjectStatus::Warning)
        );
    }

    #[test]
    fn test_dependency_counting() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let indicators = reporter
            .get_project_health_indicators()
            .expect("health indicators failed");

        // project1 has 1 dependency (to project2)
        assert_eq!(indicators[0].dependency_count, 1);
        // project2 has 1 dependency (to project3)
        assert_eq!(indicators[1].dependency_count, 1);
        // project3 has 0 dependencies
        assert_eq!(indicators[2].dependency_count, 0);
    }

    #[test]
    fn test_dependent_counting() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let indicators = reporter
            .get_project_health_indicators()
            .expect("health indicators failed");

        // project1 has 0 dependents
        assert_eq!(indicators[0].dependent_count, 0);
        // project2 has 1 dependent (project1)
        assert_eq!(indicators[1].dependent_count, 1);
        // project3 has 1 dependent (project2)
        assert_eq!(indicators[2].dependent_count, 1);
    }

    #[test]
    fn test_compliance_summary_with_critical_project() {
        let mut workspace = create_test_workspace();
        workspace.projects[0].status = ProjectStatus::Critical;

        let reporter = StatusReporter::new(workspace);
        let summary = reporter
            .generate_compliance_summary()
            .expect("compliance summary failed");

        assert!(!summary.violations.is_empty());
    }

    #[test]
    fn test_aggregated_metrics_percentages() {
        let workspace = create_test_workspace();
        let reporter = StatusReporter::new(workspace);

        let metrics = reporter
            .collect_metrics()
            .expect("metrics collection failed");

        let total =
            metrics.healthy_percentage + metrics.warning_percentage + metrics.critical_percentage;
        // Should be approximately 100 (allowing for floating point precision)
        assert!((total - 100.0).abs() < 0.1);
    }
}
