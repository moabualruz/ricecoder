//! Unit tests for VersionCoordinator
//! Tests version update propagation, constraint validation, and edge cases

use ricecoder_orchestration::{
    DependencyGraph, Project, ProjectStatus, Version, VersionCoordinator, VersionValidator,
};
use std::path::PathBuf;

fn create_project(name: &str, version: &str) -> Project {
    Project {
        path: PathBuf::from(format!("/path/to/{}", name)),
        name: name.to_string(),
        project_type: "rust".to_string(),
        version: version.to_string(),
        status: ProjectStatus::Healthy,
    }
}

#[test]
fn test_version_coordinator_creation() {
    let graph = DependencyGraph::new(false);
    let coordinator = VersionCoordinator::new(graph);

    assert_eq!(coordinator.get_all_projects().len(), 0);
}

#[test]
fn test_register_single_project() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    assert_eq!(
        coordinator.get_version("test-project"),
        Some("1.0.0".to_string())
    );
    assert_eq!(coordinator.get_all_projects().len(), 1);
}

#[test]
fn test_register_multiple_projects() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project1 = create_project("project1", "1.0.0");
    let project2 = create_project("project2", "2.0.0");
    let project3 = create_project("project3", "3.0.0");

    coordinator.register_project(&project1);
    coordinator.register_project(&project2);
    coordinator.register_project(&project3);

    assert_eq!(coordinator.get_all_projects().len(), 3);
    assert_eq!(
        coordinator.get_version("project1"),
        Some("1.0.0".to_string())
    );
    assert_eq!(
        coordinator.get_version("project2"),
        Some("2.0.0".to_string())
    );
    assert_eq!(
        coordinator.get_version("project3"),
        Some("3.0.0".to_string())
    );
}

#[test]
fn test_register_constraint() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    coordinator.register_constraint("test-project", "^1.0.0".to_string());

    let constraints = coordinator.get_constraints("test-project");
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0], "^1.0.0");
}

#[test]
fn test_register_multiple_constraints() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    coordinator.register_constraint("test-project", "^1.0.0".to_string());
    coordinator.register_constraint("test-project", "~1.2.0".to_string());
    coordinator.register_constraint("test-project", ">=1.0.0".to_string());

    let constraints = coordinator.get_constraints("test-project");
    assert_eq!(constraints.len(), 3);
    assert!(constraints.contains(&"^1.0.0".to_string()));
    assert!(constraints.contains(&"~1.2.0".to_string()));
    assert!(constraints.contains(&">=1.0.0".to_string()));
}

#[test]
fn test_update_version_success() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let result = coordinator.update_version("test-project", "1.1.0").unwrap();

    assert!(result.success);
    assert_eq!(result.old_version, "1.0.0");
    assert_eq!(result.new_version, "1.1.0");
    assert_eq!(
        coordinator.get_version("test-project"),
        Some("1.1.0".to_string())
    );
}

#[test]
fn test_update_version_invalid_format() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let result = coordinator.update_version("test-project", "invalid");
    assert!(result.is_err());
}

#[test]
fn test_update_version_not_found() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let result = coordinator.update_version("nonexistent", "1.0.0");
    assert!(result.is_err());
}

#[test]
fn test_validate_version_update_compatible() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);
    coordinator.register_constraint("test-project", "^1.0.0".to_string());

    // 1.1.0 satisfies ^1.0.0
    assert!(coordinator
        .validate_version_update("test-project", "1.1.0")
        .unwrap());
}

#[test]
fn test_validate_version_update_incompatible() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);
    coordinator.register_constraint("test-project", "^1.0.0".to_string());

    // 2.0.0 does not satisfy ^1.0.0
    assert!(coordinator
        .validate_version_update("test-project", "2.0.0")
        .is_err());
}

#[test]
fn test_validate_version_update_no_constraints() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    // No constraints, any valid version should be accepted
    assert!(coordinator
        .validate_version_update("test-project", "2.0.0")
        .unwrap());
}

#[test]
fn test_is_breaking_change_major_version() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    // Major version change is breaking
    assert!(coordinator
        .is_breaking_change("test-project", "2.0.0")
        .unwrap());
}

#[test]
fn test_is_breaking_change_minor_version() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    // Minor version change is not breaking
    assert!(!coordinator
        .is_breaking_change("test-project", "1.1.0")
        .unwrap());
}

#[test]
fn test_is_breaking_change_patch_version() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    // Patch version change is not breaking
    assert!(!coordinator
        .is_breaking_change("test-project", "1.0.1")
        .unwrap());
}

#[test]
fn test_is_breaking_change_not_found() {
    let graph = DependencyGraph::new(false);
    let coordinator = VersionCoordinator::new(graph);

    let result = coordinator.is_breaking_change("nonexistent", "2.0.0");
    assert!(result.is_err());
}

#[test]
fn test_plan_version_updates_single_project() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let updates = vec![("test-project".to_string(), "1.1.0".to_string())];
    let plan = coordinator.plan_version_updates(updates).unwrap();

    assert!(plan.is_valid);
    assert_eq!(plan.updates.len(), 1);
    assert_eq!(plan.updates[0].project, "test-project");
    assert_eq!(plan.updates[0].new_version, "1.1.0");
}

#[test]
fn test_plan_version_updates_multiple_projects() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project1 = create_project("project1", "1.0.0");
    let project2 = create_project("project2", "2.0.0");

    coordinator.register_project(&project1);
    coordinator.register_project(&project2);

    let updates = vec![
        ("project1".to_string(), "1.1.0".to_string()),
        ("project2".to_string(), "2.1.0".to_string()),
    ];
    let plan = coordinator.plan_version_updates(updates).unwrap();

    assert!(plan.is_valid);
    assert_eq!(plan.updates.len(), 2);
}

#[test]
fn test_plan_version_updates_invalid_version() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let updates = vec![("test-project".to_string(), "invalid".to_string())];
    let plan = coordinator.plan_version_updates(updates).unwrap();

    assert!(!plan.is_valid);
    assert!(!plan.validation_errors.is_empty());
}

#[test]
fn test_plan_version_updates_missing_project() {
    let graph = DependencyGraph::new(false);
    let coordinator = VersionCoordinator::new(graph);

    let updates = vec![("nonexistent".to_string(), "1.0.0".to_string())];
    let plan = coordinator.plan_version_updates(updates).unwrap();

    assert!(!plan.is_valid);
    assert!(!plan.validation_errors.is_empty());
}

#[test]
fn test_plan_version_updates_breaking_change() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let updates = vec![("test-project".to_string(), "2.0.0".to_string())];
    let plan = coordinator.plan_version_updates(updates).unwrap();

    assert!(plan.is_valid);
    assert!(plan.updates[0].is_breaking);
}

#[test]
fn test_get_affected_projects_empty() {
    let graph = DependencyGraph::new(false);
    let coordinator = VersionCoordinator::new(graph);

    let affected = coordinator.get_affected_projects("test-project");
    assert_eq!(affected.len(), 0);
}

#[test]
fn test_get_version_not_found() {
    let graph = DependencyGraph::new(false);
    let coordinator = VersionCoordinator::new(graph);

    assert_eq!(coordinator.get_version("nonexistent"), None);
}

#[test]
fn test_get_constraints_empty() {
    let graph = DependencyGraph::new(false);
    let coordinator = VersionCoordinator::new(graph);

    let constraints = coordinator.get_constraints("test-project");
    assert_eq!(constraints.len(), 0);
}

#[test]
fn test_clear_removes_all_data() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project1 = create_project("project1", "1.0.0");
    let project2 = create_project("project2", "2.0.0");

    coordinator.register_project(&project1);
    coordinator.register_project(&project2);
    coordinator.register_constraint("project1", "^1.0.0".to_string());
    coordinator.register_constraint("project2", "^2.0.0".to_string());

    assert_eq!(coordinator.get_all_projects().len(), 2);

    coordinator.clear();

    assert_eq!(coordinator.get_all_projects().len(), 0);
    assert_eq!(coordinator.get_constraints("project1").len(), 0);
    assert_eq!(coordinator.get_constraints("project2").len(), 0);
}

#[test]
fn test_version_update_with_multiple_constraints() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.2.0");
    coordinator.register_project(&project);

    // Register multiple constraints
    coordinator.register_constraint("test-project", "^1.0.0".to_string());
    coordinator.register_constraint("test-project", "~1.2.0".to_string());

    // 1.2.5 satisfies both constraints
    assert!(coordinator
        .validate_version_update("test-project", "1.2.5")
        .unwrap());

    // 1.3.0 satisfies ^1.0.0 but not ~1.2.0
    assert!(coordinator
        .validate_version_update("test-project", "1.3.0")
        .is_err());
}

#[test]
fn test_version_update_sequence() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    // Update 1.0.0 -> 1.1.0
    let result1 = coordinator.update_version("test-project", "1.1.0").unwrap();
    assert_eq!(result1.old_version, "1.0.0");
    assert_eq!(result1.new_version, "1.1.0");

    // Update 1.1.0 -> 1.2.0
    let result2 = coordinator.update_version("test-project", "1.2.0").unwrap();
    assert_eq!(result2.old_version, "1.1.0");
    assert_eq!(result2.new_version, "1.2.0");

    // Verify final version
    assert_eq!(
        coordinator.get_version("test-project"),
        Some("1.2.0".to_string())
    );
}

#[test]
fn test_caret_constraint_validation() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.2.3");
    coordinator.register_project(&project);
    coordinator.register_constraint("test-project", "^1.2.3".to_string());

    // ^1.2.3 allows >=1.2.3 and <2.0.0
    assert!(coordinator
        .validate_version_update("test-project", "1.2.3")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "1.2.4")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "1.3.0")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "1.9.9")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "2.0.0")
        .is_err());
}

#[test]
fn test_tilde_constraint_validation() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.2.3");
    coordinator.register_project(&project);
    coordinator.register_constraint("test-project", "~1.2.3".to_string());

    // ~1.2.3 allows >=1.2.3 and <1.3.0
    assert!(coordinator
        .validate_version_update("test-project", "1.2.3")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "1.2.4")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "1.3.0")
        .is_err());
    assert!(coordinator
        .validate_version_update("test-project", "2.0.0")
        .is_err());
}

#[test]
fn test_greater_or_equal_constraint_validation() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);
    coordinator.register_constraint("test-project", ">=1.0.0".to_string());

    // >=1.0.0 allows any version >= 1.0.0
    assert!(coordinator
        .validate_version_update("test-project", "1.0.0")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "1.1.0")
        .unwrap());
    assert!(coordinator
        .validate_version_update("test-project", "2.0.0")
        .unwrap());
}

#[test]
fn test_version_update_result_fields() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let result = coordinator.update_version("test-project", "1.1.0").unwrap();

    assert_eq!(result.project, "test-project");
    assert_eq!(result.old_version, "1.0.0");
    assert_eq!(result.new_version, "1.1.0");
    assert!(result.success);
    assert!(result.error.is_none());
}

#[test]
fn test_version_update_plan_fields() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let updates = vec![("test-project".to_string(), "1.1.0".to_string())];
    let plan = coordinator.plan_version_updates(updates).unwrap();

    assert!(plan.is_valid);
    assert_eq!(plan.updates.len(), 1);
    assert_eq!(plan.total_affected, 0);
    assert!(plan.validation_errors.is_empty());
}

#[test]
fn test_version_update_step_fields() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "1.0.0");
    coordinator.register_project(&project);

    let updates = vec![("test-project".to_string(), "2.0.0".to_string())];
    let plan = coordinator.plan_version_updates(updates).unwrap();

    let step = &plan.updates[0];
    assert_eq!(step.project, "test-project");
    assert_eq!(step.new_version, "2.0.0");
    assert!(step.is_breaking);
}

#[test]
fn test_dependency_graph_integration() {
    let graph = DependencyGraph::new(false);
    let coordinator = VersionCoordinator::new(graph);

    // Verify we can access the dependency graph
    let _graph = coordinator.dependency_graph();
}

#[test]
fn test_edge_case_zero_version() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "0.0.0");
    coordinator.register_project(&project);

    assert_eq!(
        coordinator.get_version("test-project"),
        Some("0.0.0".to_string())
    );

    let result = coordinator.update_version("test-project", "0.0.1").unwrap();
    assert_eq!(result.new_version, "0.0.1");
}

#[test]
fn test_edge_case_large_version_numbers() {
    let graph = DependencyGraph::new(false);
    let mut coordinator = VersionCoordinator::new(graph);

    let project = create_project("test-project", "999.999.999");
    coordinator.register_project(&project);

    assert_eq!(
        coordinator.get_version("test-project"),
        Some("999.999.999".to_string())
    );
}
