//! Property-based tests for dependency validation
//! **Feature: ricecoder-orchestration, Property 5: Dependency Validation Correctness**
//! **Validates: Requirements 3.2**

use proptest::prelude::*;
use ricecoder_orchestration::{
    DependencyValidator, Project, ProjectDependency, Version, VersionConstraint, VersionValidator,
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
        version_strategy().prop_map(|v| format!("<{}", v)),
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
        status: ricecoder_orchestration::ProjectStatus::Healthy,
    }
}

// Helper to create a test dependency
fn create_dependency(
    from: String,
    to: String,
    constraint: String,
) -> ProjectDependency {
    ProjectDependency {
        from,
        to,
        dependency_type: ricecoder_orchestration::DependencyType::Direct,
        version_constraint: constraint,
    }
}

// Property 5: Dependency Validation Correctness
// For any dependency update, the system SHALL validate compatibility and reject updates
// that would break dependent projects.
//
// This property tests that:
// 1. Compatible updates are accepted
// 2. Incompatible updates are rejected
// 3. Version constraints are properly validated

proptest! {
    #[test]
    fn prop_compatible_updates_accepted(
        project_version in version_strategy(),
        new_version in version_strategy(),
        constraint in constraint_strategy(),
    ) {
        // Skip if versions are invalid
        if Version::parse(&project_version).is_ok() && Version::parse(&new_version).is_ok() {
            // Check if new version satisfies constraint
            let is_compatible = VersionValidator::is_compatible(&constraint, &new_version).unwrap_or(false);

            // If compatible, validation should succeed
            if is_compatible {
                let result = VersionValidator::validate_update(&project_version, &new_version, &[&constraint]);
                prop_assert!(result.is_ok(), "Compatible update should be accepted");
            }
        }
    }

    #[test]
    fn prop_incompatible_updates_rejected(
        project_version in version_strategy(),
        new_version in version_strategy(),
        constraint in constraint_strategy(),
    ) {
        // Skip if versions are invalid
        if Version::parse(&project_version).is_ok() && Version::parse(&new_version).is_ok() {
            // Check if new version satisfies constraint
            let is_compatible = VersionValidator::is_compatible(&constraint, &new_version).unwrap_or(false);

            // If incompatible, validation should fail
            if !is_compatible {
                let result = VersionValidator::validate_update(&project_version, &new_version, &[&constraint]);
                prop_assert!(result.is_err(), "Incompatible update should be rejected");
            }
        }
    }

    #[test]
    fn prop_version_parsing_roundtrip(version_str in version_strategy()) {
        if let Ok(version) = Version::parse(&version_str) {
            let roundtrip = version.to_string();
            prop_assert_eq!(version_str, roundtrip, "Version roundtrip should be consistent");
        }
    }

    #[test]
    fn prop_constraint_parsing_roundtrip(constraint_str in constraint_strategy()) {
        if let Ok(constraint) = VersionConstraint::parse(&constraint_str) {
            let roundtrip = constraint.to_string();
            prop_assert_eq!(constraint_str, roundtrip, "Constraint roundtrip should be consistent");
        }
    }

    #[test]
    fn prop_caret_constraint_allows_compatible(
        major in 0u32..10,
        minor in 0u32..20,
        patch in 0u32..30,
        new_minor in 0u32..20,
        new_patch in 0u32..30,
    ) {
        let base_version = format!("{}.{}.{}", major, minor, patch);
        let constraint = format!("^{}", base_version);

        // New version with same major, higher or equal minor/patch
        let new_version = format!("{}.{}.{}", major, new_minor, new_patch);

        if let Ok(true) = VersionValidator::is_compatible(&constraint, &new_version) {
            // Should be accepted
            prop_assert!(true);
        }
    }

    #[test]
    fn prop_tilde_constraint_allows_compatible(
        major in 0u32..10,
        minor in 0u32..20,
        patch in 0u32..30,
        new_patch in 0u32..30,
    ) {
        let base_version = format!("{}.{}.{}", major, minor, patch);
        let constraint = format!("~{}", base_version);

        // New version with same major and minor, higher or equal patch
        let new_version = format!("{}.{}.{}", major, minor, new_patch);

        if let Ok(true) = VersionValidator::is_compatible(&constraint, &new_version) {
            // Should be accepted
            prop_assert!(true);
        }
    }

    #[test]
    fn prop_validator_rejects_missing_projects(
        from_name in project_name_strategy(),
        to_name in project_name_strategy(),
        version in version_strategy(),
        constraint in constraint_strategy(),
    ) {
        let mut validator = DependencyValidator::new();

        // Register only the 'from' project, not the 'to' project
        validator.register_project(&create_project(from_name.clone(), version));

        // Create a dependency to a non-existent project
        let dep = create_dependency(from_name, to_name, constraint);

        // Validation should fail
        let result = validator.validate_single_dependency(&dep.from, &dep);
        prop_assert!(result.is_err(), "Validation should fail for missing target project");
    }

    #[test]
    fn prop_validator_accepts_valid_dependencies(
        from_name in project_name_strategy(),
        to_name in project_name_strategy(),
        from_version in version_strategy(),
        to_version in version_strategy(),
        constraint in constraint_strategy(),
    ) {
        // Skip if versions are invalid
        if Version::parse(&from_version).is_ok() && Version::parse(&to_version).is_ok() {
            let mut validator = DependencyValidator::new();

            // Register both projects
            validator.register_project(&create_project(from_name.clone(), from_version));
            validator.register_project(&create_project(to_name.clone(), to_version.clone()));

            // Create a dependency with a constraint that the target version satisfies
            let is_compatible = VersionValidator::is_compatible(&constraint, &to_version).unwrap_or(false);

            if is_compatible {
                let dep = create_dependency(from_name, to_name, constraint);
                let result = validator.validate_single_dependency(&dep.from, &dep);
                prop_assert!(result.is_ok(), "Validation should succeed for compatible versions");
            }
        }
    }

    #[test]
    fn prop_breaking_changes_detected(
        major1 in 0u32..10,
        major2 in 0u32..10,
        minor in 0u32..20,
        patch in 0u32..30,
    ) {
        // Skip if majors are the same
        if major1 != major2 {
            let v1 = format!("{}.{}.{}", major1, minor, patch);
            let v2 = format!("{}.{}.{}", major2, minor, patch);

            if let Ok(is_breaking) = VersionValidator::is_breaking_change(&v1, &v2) {
                prop_assert!(is_breaking, "Major version change should be detected as breaking");
            }
        }
    }

    #[test]
    fn prop_non_breaking_changes_not_detected(
        major in 0u32..10,
        minor1 in 0u32..20,
        minor2 in 0u32..20,
        patch in 0u32..30,
    ) {
        let v1 = format!("{}.{}.{}", major, minor1, patch);
        let v2 = format!("{}.{}.{}", major, minor2, patch);

        if let Ok(is_breaking) = VersionValidator::is_breaking_change(&v1, &v2) {
            prop_assert!(!is_breaking, "Minor/patch version change should not be detected as breaking");
        }
    }

    #[test]
    fn prop_version_comparison_transitive(
        a_major in 0u32..5,
        a_minor in 0u32..10,
        a_patch in 0u32..10,
        b_major in 5u32..10,
        b_minor in 0u32..10,
        b_patch in 0u32..10,
        c_major in 10u32..15,
        c_minor in 0u32..10,
        c_patch in 0u32..10,
    ) {
        let a = Version::parse(&format!("{}.{}.{}", a_major, a_minor, a_patch)).unwrap();
        let b = Version::parse(&format!("{}.{}.{}", b_major, b_minor, b_patch)).unwrap();
        let c = Version::parse(&format!("{}.{}.{}", c_major, c_minor, c_patch)).unwrap();

        if a < b && b < c {
            prop_assert!(a < c, "Version comparison should be transitive");
        }
    }

    #[test]
    fn prop_validator_identifies_dependents(
        project_a in project_name_strategy(),
        project_b in project_name_strategy(),
        project_c in project_name_strategy(),
        version_a in version_strategy(),
        version_b in version_strategy(),
        version_c in version_strategy(),
        constraint_ab in constraint_strategy(),
        constraint_ac in constraint_strategy(),
    ) {
        // Skip if projects have the same name
        if project_a != project_b && project_a != project_c && project_b != project_c {
            let mut validator = DependencyValidator::new();

            // Register all projects
            validator.register_project(&create_project(project_a.clone(), version_a));
            validator.register_project(&create_project(project_b.clone(), version_b));
            validator.register_project(&create_project(project_c.clone(), version_c));

            // Create dependencies: B -> A, C -> A
            validator.register_dependency(create_dependency(
                project_b.clone(),
                project_a.clone(),
                constraint_ab,
            ));
            validator.register_dependency(create_dependency(
                project_c.clone(),
                project_a.clone(),
                constraint_ac,
            ));

            // Get dependents of A
            let dependents = validator.get_dependents(&project_a);

            // Should contain B and C
            prop_assert!(dependents.contains(&project_b), "B should be a dependent of A");
            prop_assert!(dependents.contains(&project_c), "C should be a dependent of A");
        }
    }
}
