//! Property-based tests for version coordination
//! **Feature: ricecoder-orchestration, Property 7: Version Coordination Consistency**
//! **Validates: Requirements 3.4**

use proptest::prelude::*;
use ricecoder_orchestration::{
    VersionCoordinator, Project, ProjectStatus, DependencyGraph, Version, VersionValidator,
};
use std::path::PathBuf;

// Strategy for generating valid semantic versions
fn version_strategy() -> impl Strategy<Value = String> {
    (0u32..10, 0u32..20, 0u32..30)
        .prop_map(|(major, minor, patch)| format!("{}.{}.{}", major, minor, patch))
}

// Strategy for generating version constraints
fn constraint_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        version_strategy().prop_map(|v| format!("^{}", v)),
        version_strategy().prop_map(|v| format!("~{}", v)),
        version_strategy().prop_map(|v| format!(">={}", v)),
    ]
}

// Strategy for generating project names
fn project_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,19}".prop_map(|s| s.to_string())
}

// Helper to create a test project
fn create_project(name: String, version: String) -> Project {
    Project {
        path: PathBuf::from(format!("/path/to/{}", name)),
        name,
        project_type: "rust".to_string(),
        version,
        status: ProjectStatus::Healthy,
    }
}

// Property 7: Version Coordination Consistency
// For any version update to a project, the VersionCoordinator SHALL propagate compatible
// version updates to all dependent projects without breaking version constraints.
//
// This property tests that:
// 1. Compatible updates propagate correctly
// 2. Version constraints are maintained
// 3. Breaking changes are detected
// 4. Affected projects are identified

proptest! {
    #[test]
    fn prop_version_update_maintains_constraints(
        project_name in project_name_strategy(),
        initial_version in version_strategy(),
        new_version in version_strategy(),
        constraint in constraint_strategy(),
    ) {
        // Skip if versions are invalid
        if Version::parse(&initial_version).is_ok() && Version::parse(&new_version).is_ok() {
            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let project = create_project(project_name.clone(), initial_version.clone());
            coordinator.register_project(&project);
            coordinator.register_constraint(&project_name, constraint.clone());

            // Check if new version satisfies constraint
            let is_compatible = VersionValidator::is_compatible(&constraint, &new_version).unwrap_or(false);

            if is_compatible {
                // Update should succeed
                let result = coordinator.update_version(&project_name, &new_version);
                prop_assert!(result.is_ok(), "Compatible update should succeed");

                // Version should be updated
                prop_assert_eq!(
                    coordinator.get_version(&project_name),
                    Some(new_version.clone()),
                    "Version should be updated"
                );
            }
        }
    }

    #[test]
    fn prop_version_update_rejects_incompatible(
        project_name in project_name_strategy(),
        initial_version in version_strategy(),
        new_version in version_strategy(),
        constraint in constraint_strategy(),
    ) {
        // Skip if versions are invalid
        if Version::parse(&initial_version).is_ok() && Version::parse(&new_version).is_ok() {
            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let project = create_project(project_name.clone(), initial_version.clone());
            coordinator.register_project(&project);
            coordinator.register_constraint(&project_name, constraint.clone());

            // Check if new version satisfies constraint
            let is_compatible = VersionValidator::is_compatible(&constraint, &new_version).unwrap_or(false);

            if !is_compatible {
                // Update should fail
                let result = coordinator.validate_version_update(&project_name, &new_version);
                prop_assert!(result.is_err(), "Incompatible update should fail");
            }
        }
    }

    #[test]
    fn prop_breaking_change_detection(
        project_name in project_name_strategy(),
        major1 in 0u32..10,
        major2 in 0u32..10,
        minor in 0u32..20,
        patch in 0u32..30,
    ) {
        // Skip if majors are the same
        if major1 != major2 {
            let v1 = format!("{}.{}.{}", major1, minor, patch);
            let v2 = format!("{}.{}.{}", major2, minor, patch);

            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let project = create_project(project_name.clone(), v1);
            coordinator.register_project(&project);

            if let Ok(is_breaking) = coordinator.is_breaking_change(&project_name, &v2) {
                prop_assert!(is_breaking, "Major version change should be detected as breaking");
            }
        }
    }

    #[test]
    fn prop_non_breaking_change_detection(
        project_name in project_name_strategy(),
        major in 0u32..10,
        minor1 in 0u32..20,
        minor2 in 0u32..20,
        patch in 0u32..30,
    ) {
        let v1 = format!("{}.{}.{}", major, minor1, patch);
        let v2 = format!("{}.{}.{}", major, minor2, patch);

        let graph = DependencyGraph::new(false);
        let mut coordinator = VersionCoordinator::new(graph);

        let project = create_project(project_name.clone(), v1);
        coordinator.register_project(&project);

        if let Ok(is_breaking) = coordinator.is_breaking_change(&project_name, &v2) {
            prop_assert!(!is_breaking, "Minor/patch version change should not be detected as breaking");
        }
    }

    #[test]
    fn prop_version_plan_validity(
        project_name in project_name_strategy(),
        initial_version in version_strategy(),
        new_version in version_strategy(),
    ) {
        // Skip if versions are invalid
        if Version::parse(&initial_version).is_ok() && Version::parse(&new_version).is_ok() {
            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let project = create_project(project_name.clone(), initial_version);
            coordinator.register_project(&project);

            let updates = vec![(project_name, new_version)];
            let plan = coordinator.plan_version_updates(updates).unwrap();

            // Plan should be valid if project exists and version is valid
            prop_assert!(plan.is_valid, "Plan should be valid for existing project with valid version");
        }
    }

    #[test]
    fn prop_version_plan_invalid_for_missing_project(
        project_name in project_name_strategy(),
        new_version in version_strategy(),
    ) {
        // Skip if version is invalid
        if Version::parse(&new_version).is_ok() {
            let graph = DependencyGraph::new(false);
            let coordinator = VersionCoordinator::new(graph);

            // Don't register the project
            let updates = vec![(project_name, new_version)];
            let plan = coordinator.plan_version_updates(updates).unwrap();

            // Plan should be invalid for missing project
            prop_assert!(!plan.is_valid, "Plan should be invalid for missing project");
        }
    }

    #[test]
    fn prop_version_plan_invalid_for_invalid_version(
        project_name in project_name_strategy(),
        initial_version in version_strategy(),
    ) {
        // Skip if initial version is invalid
        if Version::parse(&initial_version).is_ok() {
            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let project = create_project(project_name.clone(), initial_version);
            coordinator.register_project(&project);

            // Use invalid version
            let updates = vec![(project_name, "invalid".to_string())];
            let plan = coordinator.plan_version_updates(updates).unwrap();

            // Plan should be invalid for invalid version
            prop_assert!(!plan.is_valid, "Plan should be invalid for invalid version");
        }
    }

    #[test]
    fn prop_multiple_projects_version_coordination(
        project_a in project_name_strategy(),
        project_b in project_name_strategy(),
        version_a in version_strategy(),
        version_b in version_strategy(),
        new_version_a in version_strategy(),
    ) {
        // Skip if projects have the same name or versions are invalid
        if project_a != project_b
            && Version::parse(&version_a).is_ok()
            && Version::parse(&version_b).is_ok()
            && Version::parse(&new_version_a).is_ok()
        {
            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let proj_a = create_project(project_a.clone(), version_a);
            let proj_b = create_project(project_b.clone(), version_b.clone());

            coordinator.register_project(&proj_a);
            coordinator.register_project(&proj_b);

            // Both projects should be registered
            prop_assert_eq!(coordinator.get_all_projects().len(), 2);

            // Update project A
            let result = coordinator.update_version(&project_a, &new_version_a);
            prop_assert!(result.is_ok(), "Update should succeed");

            // Project A should be updated
            prop_assert_eq!(
                coordinator.get_version(&project_a),
                Some(new_version_a),
                "Project A version should be updated"
            );

            // Project B should remain unchanged
            prop_assert_eq!(
                coordinator.get_version(&project_b),
                Some(version_b),
                "Project B version should remain unchanged"
            );
        }
    }

    #[test]
    fn prop_constraint_registration_and_retrieval(
        project_name in project_name_strategy(),
        constraint1 in constraint_strategy(),
        constraint2 in constraint_strategy(),
    ) {
        let graph = DependencyGraph::new(false);
        let mut coordinator = VersionCoordinator::new(graph);

        coordinator.register_constraint(&project_name, constraint1.clone());
        coordinator.register_constraint(&project_name, constraint2.clone());

        let constraints = coordinator.get_constraints(&project_name);

        // Both constraints should be registered
        prop_assert_eq!(constraints.len(), 2);
        prop_assert!(constraints.contains(&constraint1));
        prop_assert!(constraints.contains(&constraint2));
    }

    #[test]
    fn prop_version_roundtrip(version_str in version_strategy()) {
        if let Ok(version) = Version::parse(&version_str) {
            let roundtrip = version.to_string();
            prop_assert_eq!(version_str, roundtrip, "Version roundtrip should be consistent");
        }
    }

    #[test]
    fn prop_coordinator_clear_removes_all_data(
        project_name in project_name_strategy(),
        version in version_strategy(),
        constraint in constraint_strategy(),
    ) {
        // Skip if version is invalid
        if Version::parse(&version).is_ok() {
            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let project = create_project(project_name.clone(), version);
            coordinator.register_project(&project);
            coordinator.register_constraint(&project_name, constraint);

            // Verify data is registered
            prop_assert_eq!(coordinator.get_all_projects().len(), 1);
            prop_assert!(!coordinator.get_constraints(&project_name).is_empty());

            // Clear
            coordinator.clear();

            // Verify all data is removed
            prop_assert_eq!(coordinator.get_all_projects().len(), 0);
            prop_assert!(coordinator.get_constraints(&project_name).is_empty());
        }
    }

    #[test]
    fn prop_affected_projects_identification(
        project_name in project_name_strategy(),
        version in version_strategy(),
    ) {
        // Skip if version is invalid
        if Version::parse(&version).is_ok() {
            let graph = DependencyGraph::new(false);
            let coordinator = VersionCoordinator::new(graph);

            // Get affected projects (should be empty for new coordinator)
            let affected = coordinator.get_affected_projects(&project_name);

            // Should be empty initially
            prop_assert_eq!(affected.len(), 0);
        }
    }

    #[test]
    fn prop_version_update_result_consistency(
        project_name in project_name_strategy(),
        initial_version in version_strategy(),
        new_version in version_strategy(),
    ) {
        // Skip if versions are invalid
        if Version::parse(&initial_version).is_ok() && Version::parse(&new_version).is_ok() {
            let graph = DependencyGraph::new(false);
            let mut coordinator = VersionCoordinator::new(graph);

            let project = create_project(project_name.clone(), initial_version.clone());
            coordinator.register_project(&project);

            if let Ok(result) = coordinator.update_version(&project_name, &new_version) {
                // Result should be consistent
                prop_assert_eq!(result.project, project_name);
                prop_assert_eq!(result.old_version, initial_version);
                prop_assert_eq!(result.new_version, new_version);
                prop_assert!(result.success);
                prop_assert!(result.error.is_none());
            }
        }
    }
}
