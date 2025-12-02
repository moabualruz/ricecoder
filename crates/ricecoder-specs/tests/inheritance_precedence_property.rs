//! Property-based tests for spec inheritance precedence
//! **Feature: ricecoder-specs, Property 4: Spec Inheritance Precedence**
//! **Validates: Requirements 1.7**

use proptest::prelude::*;
use ricecoder_specs::{
    models::*,
    inheritance::SpecInheritanceResolver,
};
use chrono::Utc;

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_spec_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_spec_name() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,50}".prop_map(|s| s)
}

fn arb_spec_version() -> impl Strategy<Value = String> {
    r"[0-9]\.[0-9]\.[0-9]".prop_map(|s| s)
}

fn arb_precedence_level() -> impl Strategy<Value = u32> {
    0u32..=2u32
}

fn arb_spec_with_inheritance(
    id: String,
    precedence_level: u32,
    parent_id: Option<String>,
) -> Spec {
    Spec {
        id,
        name: format!("Spec {}", precedence_level),
        version: "1.0.0".to_string(),
        requirements: vec![],
        design: None,
        tasks: vec![],
        metadata: SpecMetadata {
            author: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            phase: SpecPhase::Requirements,
            status: SpecStatus::Draft,
        },
        inheritance: Some(SpecInheritance {
            parent_id,
            precedence_level,
            merged_from: vec![],
        }),
    }
}

// ============================================================================
// Property 4: Spec Inheritance Precedence
// ============================================================================

proptest! {
    /// Property: For any spec hierarchy, resolving inheritance SHALL apply
    /// precedence rules consistently with higher-level specs overriding lower-level specs.
    ///
    /// This property verifies that:
    /// 1. Specs are ordered by precedence level (0 = project, 1 = feature, 2 = task)
    /// 2. Higher precedence specs come before lower precedence specs
    /// 3. All specs in the resolved list maintain their precedence levels
    #[test]
    fn prop_inheritance_respects_precedence_ordering(
        project_id in arb_spec_id(),
        feature_id in arb_spec_id(),
        task_id in arb_spec_id(),
    ) {
        // Create a valid hierarchy: project (0) > feature (1) > task (2)
        let project = arb_spec_with_inheritance(project_id.clone(), 0, None);
        let feature = arb_spec_with_inheritance(feature_id.clone(), 1, Some(project_id.clone()));
        let task = arb_spec_with_inheritance(task_id.clone(), 2, Some(feature_id.clone()));

        let specs = vec![task.clone(), feature.clone(), project.clone()];

        // Resolve the hierarchy
        let result = SpecInheritanceResolver::resolve(&specs);
        prop_assert!(result.is_ok(), "Hierarchy resolution should succeed");

        let resolved = result.unwrap();

        // Property 4.1: All specs should be present
        prop_assert_eq!(resolved.len(), 3, "All specs should be in resolved list");

        // Property 4.2: Specs should be ordered by precedence level
        prop_assert_eq!(
            resolved[0].inheritance.as_ref().unwrap().precedence_level,
            0,
            "First spec should have precedence level 0 (project)"
        );
        prop_assert_eq!(
            resolved[1].inheritance.as_ref().unwrap().precedence_level,
            1,
            "Second spec should have precedence level 1 (feature)"
        );
        prop_assert_eq!(
            resolved[2].inheritance.as_ref().unwrap().precedence_level,
            2,
            "Third spec should have precedence level 2 (task)"
        );

        // Property 4.3: Precedence levels should be non-decreasing
        for i in 0..resolved.len() - 1 {
            let current_level = resolved[i]
                .inheritance
                .as_ref()
                .unwrap()
                .precedence_level;
            let next_level = resolved[i + 1]
                .inheritance
                .as_ref()
                .unwrap()
                .precedence_level;
            prop_assert!(
                current_level <= next_level,
                "Precedence levels should be non-decreasing"
            );
        }
    }

    /// Property: Merging specs SHALL apply parent precedence correctly
    ///
    /// This property verifies that when merging a parent and child spec,
    /// the parent's values override the child's values.
    #[test]
    fn prop_merge_applies_parent_precedence(
        parent_name in "[A-Za-z0-9 ]{1,50}",
        child_name in "[A-Za-z0-9 ]{1,50}",
        parent_version in r"[0-9]\.[0-9]\.[0-9]",
        child_version in r"[0-9]\.[0-9]\.[0-9]",
    ) {
        let parent = Spec {
            id: "parent".to_string(),
            name: parent_name.clone(),
            version: parent_version.clone(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Parent Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Design,
                status: SpecStatus::Approved,
            },
            inheritance: Some(SpecInheritance {
                parent_id: None,
                precedence_level: 0,
                merged_from: vec![],
            }),
        };

        let child = Spec {
            id: "child".to_string(),
            name: child_name.clone(),
            version: child_version.clone(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Child Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: Some(SpecInheritance {
                parent_id: Some("parent".to_string()),
                precedence_level: 1,
                merged_from: vec![],
            }),
        };

        let result = SpecInheritanceResolver::merge(&parent, &child);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 4.4: Parent name should override child name
        prop_assert_eq!(
            merged.name, parent_name,
            "Merged spec should have parent's name"
        );

        // Property 4.5: Parent version should override child version
        prop_assert_eq!(
            merged.version, parent_version,
            "Merged spec should have parent's version"
        );

        // Property 4.6: Parent phase should override child phase
        prop_assert_eq!(
            merged.metadata.phase, SpecPhase::Design,
            "Merged spec should have parent's phase"
        );

        // Property 4.7: Parent status should override child status
        prop_assert_eq!(
            merged.metadata.status, SpecStatus::Approved,
            "Merged spec should have parent's status"
        );
    }

    /// Property: Validation SHALL detect invalid precedence levels
    ///
    /// This property verifies that when a parent has higher or equal precedence
    /// level than its child, validation fails.
    #[test]
    fn prop_validation_detects_invalid_precedence(
        parent_level in 0u32..=2u32,
        child_level in 0u32..=2u32,
    ) {
        // Only test cases where parent_level >= child_level (invalid)
        if parent_level >= child_level {
            let parent = arb_spec_with_inheritance(
                "parent".to_string(),
                parent_level,
                None,
            );
            let child = arb_spec_with_inheritance(
                "child".to_string(),
                child_level,
                Some("parent".to_string()),
            );

            let specs = vec![parent, child];

            let result = SpecInheritanceResolver::validate_chain(&specs);

            // Should fail if precedence levels are invalid
            if parent_level >= child_level {
                prop_assert!(
                    result.is_err(),
                    "Validation should fail when parent precedence >= child precedence"
                );
            }
        }
    }

    /// Property: Circular dependencies SHALL be detected
    ///
    /// This property verifies that when specs form a circular dependency chain,
    /// validation detects and reports it.
    #[test]
    fn prop_circular_dependency_detection(
        spec_a_id in arb_spec_id(),
        spec_b_id in arb_spec_id(),
        spec_c_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if spec_a_id != spec_b_id && spec_b_id != spec_c_id && spec_a_id != spec_c_id {
            let mut spec_a = arb_spec_with_inheritance(spec_a_id.clone(), 0, None);
            let spec_b = arb_spec_with_inheritance(spec_b_id.clone(), 1, Some(spec_a_id.clone()));
            let spec_c = arb_spec_with_inheritance(spec_c_id.clone(), 2, Some(spec_b_id.clone()));

            // Create circular dependency: A -> B -> C -> A
            if let Some(inh) = &mut spec_a.inheritance {
                inh.parent_id = Some(spec_c_id.clone());
            }

            let specs = vec![spec_a, spec_b, spec_c];

            let result = SpecInheritanceResolver::validate_chain(&specs);

            // Should detect circular dependency
            prop_assert!(
                result.is_err(),
                "Validation should detect circular dependency"
            );

            match result {
                Err(ricecoder_specs::SpecError::CircularDependency { specs }) => {
                    prop_assert!(
                        !specs.is_empty(),
                        "Circular dependency should include affected specs"
                    );
                }
                _ => prop_assert!(false, "Should return CircularDependency error"),
            }
        }
    }

    /// Property: Valid hierarchies SHALL pass validation
    ///
    /// This property verifies that valid spec hierarchies with correct
    /// precedence levels pass validation.
    #[test]
    fn prop_valid_hierarchy_passes_validation(
        project_id in arb_spec_id(),
        feature_id in arb_spec_id(),
        task_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if project_id != feature_id && feature_id != task_id && project_id != task_id {
            let project = arb_spec_with_inheritance(project_id.clone(), 0, None);
            let feature = arb_spec_with_inheritance(feature_id.clone(), 1, Some(project_id.clone()));
            let task = arb_spec_with_inheritance(task_id.clone(), 2, Some(feature_id.clone()));

            let specs = vec![project, feature, task];

            let result = SpecInheritanceResolver::validate_chain(&specs);

            // Valid hierarchy should pass validation
            prop_assert!(result.is_ok(), "Valid hierarchy should pass validation");
        }
    }

    /// Property: Merging SHALL preserve parent precedence level
    ///
    /// This property verifies that after merging, the resulting spec
    /// has the parent's precedence level.
    #[test]
    fn prop_merge_preserves_parent_precedence(
        parent_level in 0u32..=1u32,
    ) {
        let child_level = parent_level + 1;

        let parent = arb_spec_with_inheritance(
            "parent".to_string(),
            parent_level,
            None,
        );
        let child = arb_spec_with_inheritance(
            "child".to_string(),
            child_level,
            Some("parent".to_string()),
        );

        let result = SpecInheritanceResolver::merge(&parent, &child);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 4.8: Merged spec should have parent's precedence level
        prop_assert_eq!(
            merged.inheritance.as_ref().unwrap().precedence_level,
            parent_level,
            "Merged spec should have parent's precedence level"
        );

        // Property 4.9: Merged spec should have parent as parent_id
        prop_assert_eq!(
            merged.inheritance.as_ref().unwrap().parent_id.clone(),
            Some("parent".to_string()),
            "Merged spec should have parent as parent_id"
        );
    }

    /// Property: Resolving SHALL maintain spec identity
    ///
    /// This property verifies that resolving a hierarchy doesn't change
    /// the specs' IDs or core identity.
    #[test]
    fn prop_resolve_maintains_spec_identity(
        project_id in arb_spec_id(),
        feature_id in arb_spec_id(),
        task_id in arb_spec_id(),
    ) {
        if project_id != feature_id && feature_id != task_id && project_id != task_id {
            let project = arb_spec_with_inheritance(project_id.clone(), 0, None);
            let feature = arb_spec_with_inheritance(feature_id.clone(), 1, Some(project_id.clone()));
            let task = arb_spec_with_inheritance(task_id.clone(), 2, Some(feature_id.clone()));

            let original_ids: Vec<String> = vec![
                project.id.clone(),
                feature.id.clone(),
                task.id.clone(),
            ];

            let specs = vec![task, feature, project];

            let result = SpecInheritanceResolver::resolve(&specs);
            prop_assert!(result.is_ok(), "Resolve should succeed");

            let resolved = result.unwrap();

            // Property 4.10: All original IDs should be present in resolved specs
            for original_id in original_ids {
                prop_assert!(
                    resolved.iter().any(|s| s.id == original_id),
                    "Resolved specs should contain all original IDs"
                );
            }
        }
    }
}
