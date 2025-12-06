//! Unit tests for StatusReporter
//!
//! Tests metric collection, report generation, and aggregation logic

use ricecoder_orchestration::{
    AggregatedMetrics, ComplianceSummary, DependencyType, HealthStatus, Project, ProjectDependency,
    ProjectHealthIndicator, ProjectStatus, StatusReport, StatusReporter, Workspace, WorkspaceConfig,
    WorkspaceMetrics, WorkspaceRule, RuleType,
};
use std::path::PathBuf;

/// Helper function to create a test workspace with multiple projects
fn create_complex_workspace() -> Workspace {
    let mut workspace = Workspace::default();

    // Create 5 projects with different statuses
    workspace.projects = vec![
        Project {
            path: PathBuf::from("/workspace/project-core"),
            name: "project-core".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: ProjectStatus::Healthy,
        },
        Project {
            path: PathBuf::from("/workspace/project-api"),
            name: "project-api".to_string(),
            project_type: "rust".to_string(),
            version: "0.5.0".to_string(),
            status: ProjectStatus::Healthy,
        },
        Project {
            path: PathBuf::from("/workspace/project-cli"),
            name: "project-cli".to_string(),
            project_type: "rust".to_string(),
            version: "0.3.0".to_string(),
            status: ProjectStatus::Warning,
        },
        Project {
            path: PathBuf::from("/workspace/project-web"),
            name: "project-web".to_string(),
            project_type: "typescript".to_string(),
            version: "2.0.0".to_string(),
            status: ProjectStatus::Healthy,
        },
        Project {
            path: PathBuf::from("/workspace/project-mobile"),
            name: "project-mobile".to_string(),
            project_type: "typescript".to_string(),
            version: "1.5.0".to_string(),
            status: ProjectStatus::Critical,
        },
    ];

    // Create dependencies
    workspace.dependencies = vec![
        ProjectDependency {
            from: "project-api".to_string(),
            to: "project-core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        },
        ProjectDependency {
            from: "project-cli".to_string(),
            to: "project-core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        },
        ProjectDependency {
            from: "project-cli".to_string(),
            to: "project-api".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.5.0".to_string(),
        },
        ProjectDependency {
            from: "project-web".to_string(),
            to: "project-api".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.5.0".to_string(),
        },
        ProjectDependency {
            from: "project-mobile".to_string(),
            to: "project-api".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.5.0".to_string(),
        },
        ProjectDependency {
            from: "project-mobile".to_string(),
            to: "project-core".to_string(),
            dependency_type: DependencyType::Transitive,
            version_constraint: "^1.0.0".to_string(),
        },
    ];

    // Configure workspace rules
    workspace.config = WorkspaceConfig {
        rules: vec![
            WorkspaceRule {
                name: "no-circular-dependencies".to_string(),
                rule_type: RuleType::DependencyConstraint,
                enabled: true,
            },
            WorkspaceRule {
                name: "naming-convention".to_string(),
                rule_type: RuleType::NamingConvention,
                enabled: true,
            },
            WorkspaceRule {
                name: "architectural-boundary".to_string(),
                rule_type: RuleType::ArchitecturalBoundary,
                enabled: false,
            },
        ],
        settings: serde_json::json!({}),
    };

    // Set workspace metrics
    workspace.metrics = WorkspaceMetrics {
        total_projects: 5,
        total_dependencies: 6,
        compliance_score: 0.85,
        health_status: HealthStatus::Warning,
    };

    workspace
}

#[test]
fn test_status_reporter_metric_collection() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let metrics = reporter.collect_metrics().expect("metrics collection failed");

    // Verify metric counts
    assert_eq!(metrics.total_rules, 3);
    assert_eq!(metrics.enabled_rules, 2);
    assert_eq!(metrics.disabled_rules, 1);

    // Verify percentages
    assert_eq!(metrics.healthy_percentage, 60.0); // 3 out of 5
    assert_eq!(metrics.warning_percentage, 20.0); // 1 out of 5
    assert_eq!(metrics.critical_percentage, 20.0); // 1 out of 5

    // Verify dependency metrics
    assert_eq!(metrics.avg_dependencies_per_project, 1.2); // 6 dependencies / 5 projects
    assert_eq!(metrics.max_dependencies, 2); // project-mobile has 2
    // min_dependencies is 1 because only projects with dependencies are counted
    assert_eq!(metrics.min_dependencies, 1);
}

#[test]
fn test_status_report_generation() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let report = reporter.generate_report().expect("report generation failed");

    // Verify report structure
    assert_eq!(report.total_projects, 5);
    assert_eq!(report.total_dependencies, 6);
    assert_eq!(report.healthy_projects, 3);
    assert_eq!(report.warning_projects, 1);
    assert_eq!(report.critical_projects, 1);
    assert_eq!(report.unknown_projects, 0);

    // Verify compliance score
    assert_eq!(report.compliance_score, 0.85);

    // Verify health status
    assert_eq!(report.health_status, HealthStatus::Warning);

    // Verify project status breakdown
    assert_eq!(report.project_statuses.len(), 5);
    assert_eq!(
        report.project_statuses.get("project-core"),
        Some(&ProjectStatus::Healthy)
    );
    assert_eq!(
        report.project_statuses.get("project-mobile"),
        Some(&ProjectStatus::Critical)
    );
}

#[test]
fn test_project_health_indicators() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let indicators = reporter
        .get_project_health_indicators()
        .expect("health indicators failed");

    assert_eq!(indicators.len(), 5);

    // Find specific projects and verify their indicators
    let core_indicator = indicators
        .iter()
        .find(|i| i.name == "project-core")
        .expect("project-core not found");
    assert_eq!(core_indicator.status, ProjectStatus::Healthy);
    assert_eq!(core_indicator.dependency_count, 0); // No outgoing dependencies
    assert_eq!(core_indicator.dependent_count, 3); // 3 projects depend on it

    let api_indicator = indicators
        .iter()
        .find(|i| i.name == "project-api")
        .expect("project-api not found");
    assert_eq!(api_indicator.status, ProjectStatus::Healthy);
    assert_eq!(api_indicator.dependency_count, 1); // Depends on project-core
    assert_eq!(api_indicator.dependent_count, 3); // 3 projects depend on it

    let mobile_indicator = indicators
        .iter()
        .find(|i| i.name == "project-mobile")
        .expect("project-mobile not found");
    assert_eq!(mobile_indicator.status, ProjectStatus::Critical);
    assert_eq!(mobile_indicator.dependency_count, 2); // Depends on api and core
    assert_eq!(mobile_indicator.dependent_count, 0); // No dependents
}

#[test]
fn test_track_individual_project_health() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let health = reporter
        .track_project_health("project-cli")
        .expect("health tracking failed");

    assert_eq!(health.name, "project-cli");
    assert_eq!(health.status, ProjectStatus::Warning);
    assert_eq!(health.dependency_count, 2); // Depends on core and api
    assert_eq!(health.dependent_count, 0); // No dependents
}

#[test]
fn test_track_project_health_error_handling() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let result = reporter.track_project_health("nonexistent-project");

    assert!(result.is_err());
    match result {
        Err(e) => {
            assert!(e.to_string().contains("Project not found"));
        }
        Ok(_) => panic!("Expected error for nonexistent project"),
    }
}

#[test]
fn test_compliance_summary_generation() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let summary = reporter
        .generate_compliance_summary()
        .expect("compliance summary failed");

    assert_eq!(summary.total_rules, 3);
    assert_eq!(summary.enabled_rules, 2);
    assert_eq!(summary.compliance_score, 0.85);
    assert!(!summary.is_compliant); // Has critical project
    assert!(!summary.violations.is_empty());
}

#[test]
fn test_compliance_summary_with_all_healthy() {
    let mut workspace = create_complex_workspace();
    // Make all projects healthy
    for project in &mut workspace.projects {
        project.status = ProjectStatus::Healthy;
    }
    workspace.metrics.health_status = HealthStatus::Healthy;
    workspace.metrics.compliance_score = 1.0;

    let reporter = StatusReporter::new(workspace);
    let summary = reporter
        .generate_compliance_summary()
        .expect("compliance summary failed");

    assert!(summary.is_compliant);
    assert!(summary.violations.is_empty());
}

#[test]
fn test_aggregated_metrics_with_single_project() {
    let mut workspace = Workspace::default();
    workspace.projects = vec![Project {
        path: PathBuf::from("/workspace/single"),
        name: "single".to_string(),
        project_type: "rust".to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    }];

    let reporter = StatusReporter::new(workspace);
    let metrics = reporter.collect_metrics().expect("metrics collection failed");

    assert_eq!(metrics.healthy_percentage, 100.0);
    assert_eq!(metrics.warning_percentage, 0.0);
    assert_eq!(metrics.critical_percentage, 0.0);
    assert_eq!(metrics.avg_dependencies_per_project, 0.0);
}

#[test]
fn test_report_timestamp_format() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let report = reporter.generate_report().expect("report generation failed");

    // Verify timestamp is in ISO 8601 format
    assert!(!report.timestamp.is_empty());
    assert!(report.timestamp.contains('T'));
    assert!(report.timestamp.contains('Z') || report.timestamp.contains('+'));
}

#[test]
fn test_empty_workspace_metrics() {
    let workspace = Workspace::default();
    let reporter = StatusReporter::new(workspace);

    let metrics = reporter.collect_metrics().expect("metrics collection failed");

    assert_eq!(metrics.total_rules, 0);
    assert_eq!(metrics.enabled_rules, 0);
    assert_eq!(metrics.disabled_rules, 0);
    assert_eq!(metrics.avg_dependencies_per_project, 0.0);
    assert_eq!(metrics.max_dependencies, 0);
    assert_eq!(metrics.min_dependencies, 0);
}

#[test]
fn test_empty_workspace_report() {
    let workspace = Workspace::default();
    let reporter = StatusReporter::new(workspace);

    let report = reporter.generate_report().expect("report generation failed");

    assert_eq!(report.total_projects, 0);
    assert_eq!(report.total_dependencies, 0);
    assert_eq!(report.healthy_projects, 0);
    assert_eq!(report.warning_projects, 0);
    assert_eq!(report.critical_projects, 0);
    assert_eq!(report.unknown_projects, 0);
}

#[test]
fn test_empty_workspace_health_indicators() {
    let workspace = Workspace::default();
    let reporter = StatusReporter::new(workspace);

    let indicators = reporter
        .get_project_health_indicators()
        .expect("health indicators failed");

    assert_eq!(indicators.len(), 0);
}

#[test]
fn test_dependency_graph_analysis() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let indicators = reporter
        .get_project_health_indicators()
        .expect("health indicators failed");

    // Verify dependency graph structure
    let total_dependencies: usize = indicators.iter().map(|i| i.dependency_count).sum();
    assert_eq!(total_dependencies, 6);

    let total_dependents: usize = indicators.iter().map(|i| i.dependent_count).sum();
    assert_eq!(total_dependents, 6);
}

#[test]
fn test_project_status_distribution() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let report = reporter.generate_report().expect("report generation failed");

    let total = report.healthy_projects + report.warning_projects + report.critical_projects + report.unknown_projects;
    assert_eq!(total, report.total_projects);
}

#[test]
fn test_compliance_score_range() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let report = reporter.generate_report().expect("report generation failed");

    assert!(report.compliance_score >= 0.0);
    assert!(report.compliance_score <= 1.0);
}

#[test]
fn test_metrics_percentage_sum() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let metrics = reporter.collect_metrics().expect("metrics collection failed");

    let total_percentage = metrics.healthy_percentage + metrics.warning_percentage + metrics.critical_percentage;
    // Allow for floating point precision
    assert!((total_percentage - 100.0).abs() < 0.01);
}

#[test]
fn test_rule_count_consistency() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let metrics = reporter.collect_metrics().expect("metrics collection failed");

    assert_eq!(
        metrics.total_rules,
        metrics.enabled_rules + metrics.disabled_rules
    );
}

#[test]
fn test_project_health_indicator_consistency() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let indicators = reporter
        .get_project_health_indicators()
        .expect("health indicators failed");

    // Verify each indicator has valid data
    for indicator in indicators {
        assert!(!indicator.name.is_empty());
        assert!(indicator.rule_compliance >= 0.0);
        assert!(indicator.rule_compliance <= 1.0);
    }
}

#[test]
fn test_multiple_critical_projects() {
    let mut workspace = create_complex_workspace();
    // Make multiple projects critical (in addition to project-mobile which is already critical)
    workspace.projects[0].status = ProjectStatus::Critical;
    workspace.projects[2].status = ProjectStatus::Critical;

    let reporter = StatusReporter::new(workspace);
    let summary = reporter
        .generate_compliance_summary()
        .expect("compliance summary failed");

    assert!(!summary.is_compliant);
    // 3 critical projects: project-core, project-cli, and project-mobile
    assert_eq!(summary.violations.len(), 3);
}

#[test]
fn test_report_serialization() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let report = reporter.generate_report().expect("report generation failed");

    // Verify report can be serialized to JSON
    let json = serde_json::to_string(&report).expect("serialization failed");
    assert!(!json.is_empty());

    // Verify it can be deserialized
    let deserialized: StatusReport =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.total_projects, report.total_projects);
}

#[test]
fn test_metrics_serialization() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let metrics = reporter.collect_metrics().expect("metrics collection failed");

    // Verify metrics can be serialized to JSON
    let json = serde_json::to_string(&metrics).expect("serialization failed");
    assert!(!json.is_empty());

    // Verify it can be deserialized
    let deserialized: AggregatedMetrics =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.total_rules, metrics.total_rules);
}

#[test]
fn test_compliance_summary_serialization() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let summary = reporter
        .generate_compliance_summary()
        .expect("compliance summary failed");

    // Verify summary can be serialized to JSON
    let json = serde_json::to_string(&summary).expect("serialization failed");
    assert!(!json.is_empty());

    // Verify it can be deserialized
    let deserialized: ComplianceSummary =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.total_rules, summary.total_rules);
}

#[test]
fn test_health_indicator_serialization() {
    let workspace = create_complex_workspace();
    let reporter = StatusReporter::new(workspace);

    let indicators = reporter
        .get_project_health_indicators()
        .expect("health indicators failed");

    // Verify indicators can be serialized to JSON
    let json = serde_json::to_string(&indicators).expect("serialization failed");
    assert!(!json.is_empty());

    // Verify it can be deserialized
    let deserialized: Vec<ProjectHealthIndicator> =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.len(), indicators.len());
}

#[test]
fn test_large_workspace_metrics() {
    let mut workspace = Workspace::default();

    // Create 100 projects
    for i in 0..100 {
        workspace.projects.push(Project {
            path: PathBuf::from(format!("/workspace/project-{}", i)),
            name: format!("project-{}", i),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: if i % 10 == 0 {
                ProjectStatus::Warning
            } else {
                ProjectStatus::Healthy
            },
        });
    }

    // Create dependencies
    for i in 0..99 {
        workspace.dependencies.push(ProjectDependency {
            from: format!("project-{}", i + 1),
            to: format!("project-{}", i),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });
    }

    workspace.metrics = WorkspaceMetrics {
        total_projects: 100,
        total_dependencies: 99,
        compliance_score: 0.9,
        health_status: HealthStatus::Healthy,
    };

    let reporter = StatusReporter::new(workspace);

    let report = reporter.generate_report().expect("report generation failed");
    assert_eq!(report.total_projects, 100);
    assert_eq!(report.total_dependencies, 99);

    let metrics = reporter.collect_metrics().expect("metrics collection failed");
    assert_eq!(metrics.avg_dependencies_per_project, 0.99);

    let indicators = reporter
        .get_project_health_indicators()
        .expect("health indicators failed");
    assert_eq!(indicators.len(), 100);
}
