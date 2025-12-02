//! Property-based tests for query result consistency
//! **Feature: ricecoder-specs, Property 6: Query Result Consistency**
//! **Validates: Requirements 2.1, 2.2**

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

fn arb_spec_name() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,50}".prop_map(|s| s)
}

fn arb_spec_status() -> impl Strategy<Value = SpecStatus> {
    prop_oneof![
        Just(SpecStatus::Draft),
        Just(SpecStatus::InReview),
        Just(SpecStatus::Approved),
        Just(SpecStatus::Archived),
    ]
}

fn arb_spec_phase() -> impl Strategy<Value = SpecPhase> {
    prop_oneof![
        Just(SpecPhase::Discovery),
        Just(SpecPhase::Requirements),
        Just(SpecPhase::Design),
        Just(SpecPhase::Tasks),
        Just(SpecPhase::Execution),
    ]
}

fn arb_priority() -> impl Strategy<Value = Priority> {
    prop_oneof![
        Just(Priority::Must),
        Just(Priority::Should),
        Just(Priority::Could),
    ]
}

fn arb_spec(
    id: String,
    name: String,
    status: SpecStatus,
    phase: SpecPhase,
    priority: Priority,
) -> Spec {
    Spec {
        id,
        name,
        version: "1.0.0".to_string(),
        requirements: vec![Requirement {
            id: "REQ-1".to_string(),
            user_story: "Test story".to_string(),
            acceptance_criteria: vec![],
            priority,
        }],
        design: None,
        tasks: vec![],
        metadata: SpecMetadata {
            author: Some("Test Author".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            phase,
            status,
        },
        inheritance: None,
    }
}

// ============================================================================
// Property 6: Query Result Consistency
// ============================================================================

proptest! {
    /// Property: For any query with filters, the system SHALL return only specs
    /// matching all filter criteria, and results SHALL be consistent across
    /// multiple executions with identical input.
    ///
    /// This property verifies that:
    /// 1. Query results are deterministic
    /// 2. Multiple executions with same input produce identical results
    /// 3. Results are consistent in order and content
    #[test]
    fn prop_query_consistency_multiple_executions(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name, status, phase, priority);
        let specs = vec![spec];

        let query = SpecQuery {
            status: Some(status),
            phase: Some(phase),
            ..Default::default()
        };

        // Execute query multiple times
        let results1 = SpecQueryEngine::query(&specs, &query);
        let results2 = SpecQueryEngine::query(&specs, &query);
        let results3 = SpecQueryEngine::query(&specs, &query);

        // Property 6.1: All executions should return same number of results
        prop_assert_eq!(
            results1.len(),
            results2.len(),
            "Query results should be consistent (run 1 vs 2)"
        );
        prop_assert_eq!(
            results2.len(),
            results3.len(),
            "Query results should be consistent (run 2 vs 3)"
        );

        // Property 6.2: Results should be in same order
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            prop_assert_eq!(
                &r1.id, &r2.id,
                "Query results should be in same order"
            );
        }

        // Property 6.3: Results should have identical content
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            prop_assert_eq!(
                &r1.name, &r2.name,
                "Query results should have identical content"
            );
            prop_assert_eq!(
                r1.metadata.status, r2.metadata.status,
                "Query results should have identical status"
            );
        }
    }

    /// Property: For any query with status filter, the system SHALL return
    /// only specs with matching status.
    ///
    /// This property verifies that status filtering works correctly.
    #[test]
    fn prop_query_status_filter_correctness(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name, status, phase, priority);
        let specs = vec![spec];

        let query = SpecQuery {
            status: Some(status),
            ..Default::default()
        };

        let results = SpecQueryEngine::query(&specs, &query);

        // Property 6.4: All results should match the status filter
        for result in &results {
            prop_assert_eq!(
                result.metadata.status,
                status,
                "All results should match status filter"
            );
        }
    }

    /// Property: For any query with phase filter, the system SHALL return
    /// only specs with matching phase.
    ///
    /// This property verifies that phase filtering works correctly.
    #[test]
    fn prop_query_phase_filter_correctness(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name, status, phase, priority);
        let specs = vec![spec];

        let query = SpecQuery {
            phase: Some(phase),
            ..Default::default()
        };

        let results = SpecQueryEngine::query(&specs, &query);

        // Property 6.5: All results should match the phase filter
        for result in &results {
            prop_assert_eq!(
                result.metadata.phase,
                phase,
                "All results should match phase filter"
            );
        }
    }

    /// Property: For any query with name filter, the system SHALL return
    /// only specs with matching name.
    ///
    /// This property verifies that name filtering works correctly.
    #[test]
    fn prop_query_name_filter_correctness(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name.clone(), status, phase, priority);
        let specs = vec![spec];

        let query = SpecQuery {
            name: Some(spec_name.clone()),
            ..Default::default()
        };

        let results = SpecQueryEngine::query(&specs, &query);

        // Property 6.6: All results should match the name filter
        for result in &results {
            prop_assert!(
                result.name.to_lowercase().contains(&spec_name.to_lowercase())
                    || result.id.to_lowercase().contains(&spec_name.to_lowercase()),
                "All results should match name filter"
            );
        }
    }

    /// Property: For any query with multiple filters, the system SHALL return
    /// only specs matching ALL filter criteria.
    ///
    /// This property verifies that multiple filters are applied with AND logic.
    #[test]
    fn prop_query_multiple_filters_and_logic(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name.clone(), status, phase, priority);
        let specs = vec![spec];

        let query = SpecQuery {
            status: Some(status),
            phase: Some(phase),
            name: Some(spec_name.clone()),
            ..Default::default()
        };

        let results = SpecQueryEngine::query(&specs, &query);

        // Property 6.7: All results should match ALL filters
        for result in &results {
            prop_assert_eq!(
                result.metadata.status,
                status,
                "Result should match status filter"
            );
            prop_assert_eq!(
                result.metadata.phase,
                phase,
                "Result should match phase filter"
            );
            prop_assert!(
                result.name.to_lowercase().contains(&spec_name.to_lowercase())
                    || result.id.to_lowercase().contains(&spec_name.to_lowercase()),
                "Result should match name filter"
            );
        }
    }

    /// Property: For any query with no filters, the system SHALL return
    /// all specs.
    ///
    /// This property verifies that empty queries return all specs.
    #[test]
    fn prop_query_no_filters_returns_all(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name, status, phase, priority);
        let specs = vec![spec];

        let query = SpecQuery::default();

        let results = SpecQueryEngine::query(&specs, &query);

        // Property 6.8: Empty query should return all specs
        prop_assert_eq!(
            results.len(),
            specs.len(),
            "Empty query should return all specs"
        );
    }

    /// Property: For any query that matches no specs, the system SHALL return
    /// an empty result set consistently.
    ///
    /// This property verifies that non-matching queries return empty results.
    #[test]
    fn prop_query_no_matches_returns_empty(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name, status, phase, priority);
        let specs = vec![spec];

        // Create a query that won't match
        let non_matching_status = match status {
            SpecStatus::Draft => SpecStatus::Approved,
            SpecStatus::Approved => SpecStatus::Draft,
            SpecStatus::InReview => SpecStatus::Archived,
            SpecStatus::Archived => SpecStatus::InReview,
        };

        let query = SpecQuery {
            status: Some(non_matching_status),
            ..Default::default()
        };

        let results1 = SpecQueryEngine::query(&specs, &query);
        let results2 = SpecQueryEngine::query(&specs, &query);

        // Property 6.9: Non-matching query should return empty results
        prop_assert_eq!(
            results1.len(),
            0,
            "Non-matching query should return empty results"
        );

        // Property 6.10: Empty results should be consistent
        prop_assert_eq!(
            results1.len(),
            results2.len(),
            "Empty results should be consistent across executions"
        );
    }

    /// Property: Query results SHALL not be modified by the order of specs
    /// in the input.
    ///
    /// This property verifies that query results are independent of input order.
    #[test]
    fn prop_query_results_independent_of_input_order(
        spec_id_1 in arb_spec_id(),
        spec_id_2 in arb_spec_id(),
        spec_name_1 in arb_spec_name(),
        spec_name_2 in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        // Only test if IDs are different
        if spec_id_1 != spec_id_2 {
            let spec1 = arb_spec(spec_id_1.clone(), spec_name_1, status, phase, priority);
            let spec2 = arb_spec(spec_id_2.clone(), spec_name_2, status, phase, priority);

            let query = SpecQuery {
                status: Some(status),
                ..Default::default()
            };

            // Query with different input orders
            let results1 = SpecQueryEngine::query(&[spec1.clone(), spec2.clone()], &query);
            let results2 = SpecQueryEngine::query(&[spec2, spec1], &query);

            // Property 6.11: Results should contain same specs regardless of input order
            prop_assert_eq!(
                results1.len(),
                results2.len(),
                "Results should be independent of input order"
            );

            // Property 6.12: Results should contain same IDs
            let ids1: std::collections::HashSet<_> = results1.iter().map(|s| s.id.clone()).collect();
            let ids2: std::collections::HashSet<_> = results2.iter().map(|s| s.id.clone()).collect();
            prop_assert_eq!(
                ids1, ids2,
                "Results should contain same specs regardless of input order"
            );
        }
    }

    /// Property: Query results SHALL be deterministic across different
    /// execution environments.
    ///
    /// This property verifies that queries produce identical results
    /// when run multiple times in sequence.
    #[test]
    fn prop_query_deterministic_across_runs(
        spec_id in arb_spec_id(),
        spec_name in arb_spec_name(),
        status in arb_spec_status(),
        phase in arb_spec_phase(),
        priority in arb_priority(),
    ) {
        let spec = arb_spec(spec_id, spec_name, status, phase, priority);
        let specs = vec![spec];

        let query = SpecQuery {
            status: Some(status),
            phase: Some(phase),
            ..Default::default()
        };

        // Run query 5 times
        let mut all_results = vec![];
        for _ in 0..5 {
            all_results.push(SpecQueryEngine::query(&specs, &query));
        }

        // Property 6.13: All runs should produce identical results
        for i in 1..all_results.len() {
            prop_assert_eq!(
                all_results[0].len(),
                all_results[i].len(),
                "Query should be deterministic across runs"
            );

            for (r0, ri) in all_results[0].iter().zip(all_results[i].iter()) {
                prop_assert_eq!(
                    &r0.id, &ri.id,
                    "Query results should be identical across runs"
                );
            }
        }
    }
}
