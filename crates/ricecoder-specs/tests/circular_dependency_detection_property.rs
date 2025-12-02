//! Property-based tests for circular dependency detection
//! **Feature: ricecoder-specs, Property 5: Circular Dependency Detection**
//! **Validates: Requirements 2.5**

use proptest::prelude::*;
use ricecoder_specs::{
    models::*,
    query::SpecQueryEngine,
};
use chrono::Utc;

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_spec_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_spec_with_tasks(
    id: String,
    task_requirements: Vec<String>,
) -> Spec {
    let tasks = if task_requirements.is_empty() {
        vec![]
    } else {
        vec![Task {
            id: "1".to_string(),
            description: "Task 1".to_string(),
            subtasks: vec![],
            requirements: task_requirements,
            status: TaskStatus::NotStarted,
            optional: false,
        }]
    };

    Spec {
        id,
        name: "Test Spec".to_string(),
        version: "1.0.0".to_string(),
        requirements: vec![],
        design: None,
        tasks,
        metadata: SpecMetadata {
            author: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            phase: SpecPhase::Requirements,
            status: SpecStatus::Draft,
        },
        inheritance: None,
    }
}

fn arb_requirement(id: String) -> Requirement {
    Requirement {
        id,
        user_story: "Test story".to_string(),
        acceptance_criteria: vec![],
        priority: Priority::Must,
    }
}

// ============================================================================
// Property 5: Circular Dependency Detection
// ============================================================================

proptest! {
    /// Property: For any set of specs with dependencies, the system SHALL detect
    /// and report all circular dependencies with affected spec IDs.
    ///
    /// This property verifies that:
    /// 1. Circular dependencies are detected when they exist
    /// 2. Detected cycles include all affected spec IDs
    /// 3. No false positives (non-circular dependencies are not reported as cycles)
    #[test]
    fn prop_circular_dependency_detection_finds_cycles(
        spec_a_id in arb_spec_id(),
        spec_b_id in arb_spec_id(),
        spec_c_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if spec_a_id != spec_b_id && spec_b_id != spec_c_id && spec_a_id != spec_c_id {
            // Create a circular dependency: A -> B -> C -> A
            // A requires REQ-B, B requires REQ-C, C requires REQ-A
            let mut spec_a = arb_spec_with_tasks(spec_a_id.clone(), vec!["REQ-B".to_string()]);
            spec_a.requirements = vec![arb_requirement("REQ-A".to_string())];

            let mut spec_b = arb_spec_with_tasks(spec_b_id.clone(), vec!["REQ-C".to_string()]);
            spec_b.requirements = vec![arb_requirement("REQ-B".to_string())];

            let mut spec_c = arb_spec_with_tasks(spec_c_id.clone(), vec!["REQ-A".to_string()]);
            spec_c.requirements = vec![arb_requirement("REQ-C".to_string())];

            let specs = vec![spec_a, spec_b, spec_c];

            let cycles = SpecQueryEngine::detect_circular_dependencies(&specs);

            // Property 5.1: Circular dependencies should be detected
            prop_assert!(
                !cycles.is_empty(),
                "Circular dependencies should be detected"
            );

            // Property 5.2: Each cycle should contain at least 2 specs
            for cycle in &cycles {
                prop_assert!(
                    cycle.len() >= 2,
                    "Each cycle should contain at least 2 specs"
                );
            }

            // Property 5.3: All specs in cycles should be from the input set
            let input_ids: Vec<String> = specs.iter().map(|s| s.id.clone()).collect();
            for cycle in &cycles {
                for spec_id in cycle {
                    prop_assert!(
                        input_ids.contains(spec_id),
                        "Cycle should only contain specs from input set"
                    );
                }
            }
        }
    }

    /// Property: For any set of specs without circular dependencies,
    /// the system SHALL NOT report any cycles.
    ///
    /// This property verifies that:
    /// 1. Linear dependency chains are not reported as cycles
    /// 2. Isolated specs are not reported as cycles
    /// 3. No false positives
    #[test]
    fn prop_no_false_positive_cycles(
        spec_a_id in arb_spec_id(),
        spec_b_id in arb_spec_id(),
        spec_c_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if spec_a_id != spec_b_id && spec_b_id != spec_c_id && spec_a_id != spec_c_id {
            // Create a linear dependency: A -> B -> C (no cycle)
            let mut spec_a = arb_spec_with_tasks(spec_a_id.clone(), vec!["REQ-B".to_string()]);
            spec_a.requirements = vec![arb_requirement("REQ-A".to_string())];

            let mut spec_b = arb_spec_with_tasks(spec_b_id.clone(), vec!["REQ-C".to_string()]);
            spec_b.requirements = vec![arb_requirement("REQ-B".to_string())];

            let mut spec_c = arb_spec_with_tasks(spec_c_id.clone(), vec![]);
            spec_c.requirements = vec![arb_requirement("REQ-C".to_string())];

            let specs = vec![spec_a, spec_b, spec_c];

            let cycles = SpecQueryEngine::detect_circular_dependencies(&specs);

            // Property 5.4: No cycles should be detected in linear dependency chain
            prop_assert_eq!(
                cycles.len(),
                0,
                "Linear dependency chain should not be reported as cycles"
            );
        }
    }

    /// Property: For any set of isolated specs (no dependencies),
    /// the system SHALL NOT report any cycles.
    ///
    /// This property verifies that isolated specs don't trigger false positives.
    #[test]
    fn prop_isolated_specs_no_cycles(
        spec_a_id in arb_spec_id(),
        spec_b_id in arb_spec_id(),
        spec_c_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if spec_a_id != spec_b_id && spec_b_id != spec_c_id && spec_a_id != spec_c_id {
            // Create isolated specs with no dependencies
            let spec_a = arb_spec_with_tasks(spec_a_id, vec![]);
            let spec_b = arb_spec_with_tasks(spec_b_id, vec![]);
            let spec_c = arb_spec_with_tasks(spec_c_id, vec![]);

            let specs = vec![spec_a, spec_b, spec_c];

            let cycles = SpecQueryEngine::detect_circular_dependencies(&specs);

            // Property 5.5: No cycles should be detected in isolated specs
            prop_assert_eq!(
                cycles.len(),
                0,
                "Isolated specs should not be reported as cycles"
            );
        }
    }

    /// Property: For any spec with no external dependencies,
    /// the system SHALL NOT report it as part of a cycle.
    ///
    /// This property verifies that specs with only internal references
    /// are not reported as cycles.
    #[test]
    fn prop_internal_references_not_cycles(
        spec_id in arb_spec_id(),
    ) {
        // Create a spec with internal references (task requires own requirement)
        let mut spec = arb_spec_with_tasks(spec_id.clone(), vec!["REQ-A".to_string()]);
        spec.requirements = vec![arb_requirement("REQ-A".to_string())];

        let specs = vec![spec];

        let cycles = SpecQueryEngine::detect_circular_dependencies(&specs);

        // Property 5.6: Internal references should not be reported as cycles
        prop_assert_eq!(
            cycles.len(),
            0,
            "Internal references should not be reported as cycles"
        );
    }

    /// Property: For any two-spec cycle (A -> B -> A),
    /// the system SHALL detect it with both spec IDs.
    ///
    /// This property verifies that two-spec cycles are properly detected.
    #[test]
    fn prop_two_spec_cycle_detected(
        spec_a_id in arb_spec_id(),
        spec_b_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if spec_a_id != spec_b_id {
            // Create a two-spec cycle: A -> B -> A
            let mut spec_a = arb_spec_with_tasks(spec_a_id.clone(), vec!["REQ-B".to_string()]);
            spec_a.requirements = vec![arb_requirement("REQ-A".to_string())];

            let mut spec_b = arb_spec_with_tasks(spec_b_id.clone(), vec!["REQ-A".to_string()]);
            spec_b.requirements = vec![arb_requirement("REQ-B".to_string())];

            let specs = vec![spec_a, spec_b];

            let cycles = SpecQueryEngine::detect_circular_dependencies(&specs);

            // Property 5.8: Two-spec cycle should be detected
            prop_assert!(
                !cycles.is_empty(),
                "Two-spec cycle should be detected"
            );

            // Property 5.9: Cycle should contain both spec IDs
            prop_assert!(
                cycles.iter().any(|cycle| {
                    cycle.contains(&spec_a_id) && cycle.contains(&spec_b_id)
                }),
                "Cycle should contain both spec IDs"
            );
        }
    }

    /// Property: Cycle detection SHALL be deterministic
    ///
    /// This property verifies that running cycle detection multiple times
    /// on the same input produces identical results.
    #[test]
    fn prop_cycle_detection_deterministic(
        spec_a_id in arb_spec_id(),
        spec_b_id in arb_spec_id(),
        spec_c_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if spec_a_id != spec_b_id && spec_b_id != spec_c_id && spec_a_id != spec_c_id {
            // Create a circular dependency
            let mut spec_a = arb_spec_with_tasks(spec_a_id.clone(), vec!["REQ-B".to_string()]);
            spec_a.requirements = vec![arb_requirement("REQ-A".to_string())];

            let mut spec_b = arb_spec_with_tasks(spec_b_id.clone(), vec!["REQ-C".to_string()]);
            spec_b.requirements = vec![arb_requirement("REQ-B".to_string())];

            let mut spec_c = arb_spec_with_tasks(spec_c_id.clone(), vec!["REQ-A".to_string()]);
            spec_c.requirements = vec![arb_requirement("REQ-C".to_string())];

            let specs = vec![spec_a, spec_b, spec_c];

            // Run cycle detection multiple times
            let cycles1 = SpecQueryEngine::detect_circular_dependencies(&specs);
            let cycles2 = SpecQueryEngine::detect_circular_dependencies(&specs);
            let cycles3 = SpecQueryEngine::detect_circular_dependencies(&specs);

            // Property 5.10: Results should be identical across multiple runs
            prop_assert_eq!(
                cycles1.len(),
                cycles2.len(),
                "Cycle detection should be deterministic (run 1 vs 2)"
            );
            prop_assert_eq!(
                cycles2.len(),
                cycles3.len(),
                "Cycle detection should be deterministic (run 2 vs 3)"
            );

            // Property 5.11: Cycles should be in same order
            for (c1, c2) in cycles1.iter().zip(cycles2.iter()) {
                prop_assert_eq!(
                    c1, c2,
                    "Cycles should be identical across runs"
                );
            }
        }
    }

    /// Property: Cycle detection SHALL report all affected specs
    ///
    /// This property verifies that when a cycle is detected, all specs
    /// involved in the cycle are included in the report.
    #[test]
    fn prop_cycle_includes_all_affected_specs(
        spec_a_id in arb_spec_id(),
        spec_b_id in arb_spec_id(),
        spec_c_id in arb_spec_id(),
    ) {
        // Only test if IDs are different
        if spec_a_id != spec_b_id && spec_b_id != spec_c_id && spec_a_id != spec_c_id {
            // Create a three-spec cycle: A -> B -> C -> A
            let mut spec_a = arb_spec_with_tasks(spec_a_id.clone(), vec!["REQ-B".to_string()]);
            spec_a.requirements = vec![arb_requirement("REQ-A".to_string())];

            let mut spec_b = arb_spec_with_tasks(spec_b_id.clone(), vec!["REQ-C".to_string()]);
            spec_b.requirements = vec![arb_requirement("REQ-B".to_string())];

            let mut spec_c = arb_spec_with_tasks(spec_c_id.clone(), vec!["REQ-A".to_string()]);
            spec_c.requirements = vec![arb_requirement("REQ-C".to_string())];

            let specs = vec![spec_a, spec_b, spec_c];

            let cycles = SpecQueryEngine::detect_circular_dependencies(&specs);

            // Property 5.12: At least one cycle should be detected
            prop_assert!(
                !cycles.is_empty(),
                "Cycle should be detected"
            );

            // Property 5.13: Cycle should contain all three spec IDs
            prop_assert!(
                cycles.iter().any(|cycle| {
                    cycle.contains(&spec_a_id)
                        && cycle.contains(&spec_b_id)
                        && cycle.contains(&spec_c_id)
                }),
                "Cycle should contain all affected spec IDs"
            );
        }
    }
}
