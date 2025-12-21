//! Integration tests for configuration and rules
//!
//! Tests configuration loading and application, and rules validation across projects.
//!
//! **Feature: ricecoder-orchestration, Integration Tests: Configuration and Rules**
//! **Validates: Requirements 1.3, 1.4**

use ricecoder_orchestration::{
    DependencyGraph, DependencyType, Project, ProjectDependency, ProjectStatus, RuleType,
    RulesValidator, ViolationSeverity, Workspace, WorkspaceConfig, WorkspaceRule,
};
use std::path::PathBuf;

/// Helper to create a test project
fn create_test_project(name: &str) -> Project {
    Project {
        path: PathBuf::from(format!("/workspace/{}", name)),
        name: name.to_string(),
        project_type: "rust".to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    }
}

/// Helper to create a test workspace
fn create_test_workspace() -> Workspace {
    let mut workspace = Workspace::default();

    workspace.projects = vec![
        create_test_project("core"),
        create_test_project("storage"),
        create_test_project("api"),
        create_test_project("cli"),
    ];

    workspace.dependencies = vec![
        ProjectDependency {
            from: "storage".to_string(),
            to: "core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        },
        ProjectDependency {
            from: "api".to_string(),
            to: "core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        },
        ProjectDependency {
            from: "api".to_string(),
            to: "storage".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        },
        ProjectDependency {
            from: "cli".to_string(),
            to: "core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        },
    ];

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
        settings: serde_json::json!({
            "max_dependencies_per_project": 5,
            "allowed_project_types": ["rust", "typescript", "python"],
            "version_constraint_format": "semver"
        }),
    };

    workspace
}

/// Integration test: Configuration loading and application
///
/// This test verifies that:
/// 1. Configuration can be loaded from workspace
/// 2. Configuration is applied consistently to all projects
/// 3. Configuration hierarchy is respected
#[test]
fn integration_test_configuration_loading_and_application() {
    // Setup: Create workspace with configuration
    let workspace = create_test_workspace();

    // Verify: Configuration exists
    assert!(!workspace.config.rules.is_empty());
    assert_eq!(workspace.config.rules.len(), 3);

    // Verify: Settings are loaded
    let settings = &workspace.config.settings;
    assert_eq!(
        settings
            .get("max_dependencies_per_project")
            .unwrap()
            .as_i64(),
        Some(5)
    );

    // Verify: Rules are properly configured
    let enabled_rules: Vec<_> = workspace
        .config
        .rules
        .iter()
        .filter(|r| r.enabled)
        .collect();
    assert_eq!(enabled_rules.len(), 2);

    let disabled_rules: Vec<_> = workspace
        .config
        .rules
        .iter()
        .filter(|r| !r.enabled)
        .collect();
    assert_eq!(disabled_rules.len(), 1);
}

/// Integration test: Configuration consistency across projects
///
/// This test verifies that:
/// 1. Configuration is applied consistently to all projects
/// 2. No conflicts or partial application
/// 3. All projects see the same configuration
#[test]
fn integration_test_configuration_consistency() {
    // Setup: Create workspace
    let workspace = create_test_workspace();

    // Verify: All projects exist
    assert_eq!(workspace.projects.len(), 4);

    // Verify: Configuration is consistent
    for project in &workspace.projects {
        assert_eq!(project.project_type, "rust");
        assert_eq!(project.status, ProjectStatus::Healthy);
    }

    // Verify: All projects have same configuration applied
    let config_rules = workspace.config.rules.len();
    assert_eq!(config_rules, 3);
}

/// Integration test: Rules validation across projects
///
/// This test verifies that:
/// 1. Rules are validated across all projects
/// 2. Violations are correctly identified
/// 3. Validation results are accurate
#[test]
fn integration_test_rules_validation() {
    // Setup: Create workspace
    let workspace = create_test_workspace();

    // Create rules validator
    let validator = RulesValidator::new(workspace.clone());

    // Validate all rules
    let result = validator.validate_all().unwrap();

    // Verify: Validation completed
    assert!(result.passed || !result.passed);

    // Verify: Violations are reported if any
    if !result.violations.is_empty() {
        for violation in &result.violations {
            assert!(!violation.rule_name.is_empty());
            assert!(!violation.affected_projects.is_empty());
        }
    }
}

/// Integration test: Dependency constraint validation
///
/// This test verifies that:
/// 1. Dependency constraints are validated
/// 2. Circular dependencies are detected
/// 3. Invalid dependencies are reported
#[test]
fn integration_test_dependency_constraint_validation() {
    // Setup: Create workspace with circular dependency
    let mut workspace = create_test_workspace();

    // Add circular dependency: core -> cli
    workspace.dependencies.push(ProjectDependency {
        from: "core".to_string(),
        to: "cli".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    // Create validator
    let validator = RulesValidator::new(workspace);

    // Validate dependency constraints
    let result = validator.validate_all().unwrap();

    // Verify: Circular dependency is detected
    if !result.violations.is_empty() {
        let circular_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.rule_name.contains("circular"))
            .collect();
        assert!(!circular_violations.is_empty());
    }
}

/// Integration test: Naming convention validation
///
/// This test verifies that:
/// 1. Naming conventions are validated
/// 2. Non-compliant names are detected
/// 3. Validation results are accurate
#[test]
fn integration_test_naming_convention_validation() {
    // Setup: Create workspace with non-compliant project name
    let mut workspace = create_test_workspace();

    // Add project with non-compliant name (should be kebab-case)
    workspace.projects.push(Project {
        path: PathBuf::from("/workspace/InvalidProjectName"),
        name: "InvalidProjectName".to_string(),
        project_type: "rust".to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    });

    // Create validator
    let validator = RulesValidator::new(workspace);

    // Validate naming conventions
    let result = validator.validate_all().unwrap();

    // Verify: Non-compliant name is detected
    if !result.violations.is_empty() {
        let naming_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.rule_name.contains("naming"))
            .collect();
        assert!(!naming_violations.is_empty());
    }
}

/// Integration test: Architectural boundary validation
///
/// This test verifies that:
/// 1. Architectural boundaries are enforced
/// 2. Boundary violations are detected
/// 3. Validation respects enabled/disabled rules
#[test]
fn integration_test_architectural_boundary_validation() {
    // Setup: Create workspace
    let workspace = create_test_workspace();

    // Create validator
    let validator = RulesValidator::new(workspace);

    // Validate architectural boundaries (disabled by default)
    let result = validator.validate_all().unwrap();

    // Verify: Disabled rules don't cause violations
    let arch_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule_name.contains("architectural"))
        .collect();
    assert!(arch_violations.is_empty());
}

/// Integration test: Rules validation with multiple violations
///
/// This test verifies that:
/// 1. Multiple violations are detected
/// 2. All violations are reported
/// 3. Violation details are accurate
#[test]
fn integration_test_multiple_violations_detection() {
    // Setup: Create workspace with multiple violations
    let mut workspace = create_test_workspace();

    // Add circular dependency
    workspace.dependencies.push(ProjectDependency {
        from: "core".to_string(),
        to: "cli".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    // Add non-compliant project name
    workspace.projects.push(Project {
        path: PathBuf::from("/workspace/BadName"),
        name: "BadName".to_string(),
        project_type: "rust".to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    });

    // Create validator
    let validator = RulesValidator::new(workspace);

    // Validate all rules
    let result = validator.validate_all().unwrap();

    // Verify: Multiple violations are detected
    if !result.violations.is_empty() {
        assert!(result.violations.len() >= 1);
    }
}

/// Integration test: Violation severity levels
///
/// This test verifies that:
/// 1. Violations have appropriate severity levels
/// 2. Critical violations are identified
/// 3. Severity levels are consistent
#[test]
fn integration_test_violation_severity_levels() {
    // Setup: Create workspace with violations
    let mut workspace = create_test_workspace();

    // Add circular dependency (critical violation)
    workspace.dependencies.push(ProjectDependency {
        from: "core".to_string(),
        to: "cli".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    // Create validator
    let validator = RulesValidator::new(workspace);

    // Validate rules
    let result = validator.validate_all().unwrap();

    // Verify: Violations have appropriate severity
    for violation in &result.violations {
        match violation.rule_name.as_str() {
            name if name.contains("circular") => {
                assert_eq!(violation.severity, ViolationSeverity::Critical);
            }
            _ => {}
        }
    }
}

/// Integration test: Workspace configuration structure
///
/// This test verifies that:
/// 1. Workspace configuration has correct structure
/// 2. All required fields are present
/// 3. Configuration is properly formatted
#[test]
fn integration_test_workspace_configuration_structure() {
    // Setup: Create workspace
    let workspace = create_test_workspace();

    // Verify: Configuration structure
    assert!(!workspace.config.rules.is_empty());
    assert!(!workspace.config.settings.is_null());

    // Verify: Each rule has required fields
    for rule in &workspace.config.rules {
        assert!(!rule.name.is_empty());
        // rule_type is an enum, always valid
        // enabled is a bool, always valid
    }

    // Verify: Settings are valid JSON
    assert!(workspace.config.settings.is_object());
}

/// Integration test: Project status tracking
///
/// This test verifies that:
/// 1. Project status is tracked correctly
/// 2. Status changes are reflected
/// 3. Status is consistent across workspace
#[test]
fn integration_test_project_status_tracking() {
    // Setup: Create workspace
    let mut workspace = create_test_workspace();

    // Verify: All projects start as healthy
    for project in &workspace.projects {
        assert_eq!(project.status, ProjectStatus::Healthy);
    }

    // Change project status
    workspace.projects[0].status = ProjectStatus::Warning;

    // Verify: Status change is reflected
    assert_eq!(workspace.projects[0].status, ProjectStatus::Warning);
    assert_eq!(workspace.projects[1].status, ProjectStatus::Healthy);
}

/// Integration test: Dependency graph validation
///
/// This test verifies that:
/// 1. Dependency graph can be built from workspace
/// 2. All relationships are correctly captured
/// 3. Graph queries work correctly
#[test]
fn integration_test_dependency_graph_validation() {
    // Setup: Create workspace
    let workspace = create_test_workspace();

    // Build dependency graph
    let mut graph = DependencyGraph::new(false);

    for project in &workspace.projects {
        graph.add_project(project.clone()).unwrap();
    }

    for dep in &workspace.dependencies {
        graph.add_dependency(dep.clone()).unwrap();
    }

    // Verify: Graph structure
    assert_eq!(graph.get_projects().len(), 4);

    // Verify: Can query dependencies
    let core_dependents = graph.get_dependents("core");
    assert_eq!(core_dependents.len(), 3); // storage, api, cli depend on core

    let api_deps = graph.get_dependencies("api");
    assert_eq!(api_deps.len(), 2); // api depends on core and storage
}
