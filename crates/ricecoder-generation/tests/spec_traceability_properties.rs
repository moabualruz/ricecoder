//! Property-based tests for spec traceability
//!
//! **Feature: ricecoder-generation, Property 1: Spec Traceability**
//! **Validates: Requirements 1.1, 1.2**
//!
//! Property: For any generated code, all code elements SHALL trace back to at least one requirement in the spec.

use proptest::prelude::*;
use ricecoder_generation::{SpecProcessor, GenerationPlanBuilder};
use ricecoder_specs::models::{
    Spec, Requirement, AcceptanceCriterion, Priority, SpecMetadata, SpecPhase, SpecStatus,
};
use chrono::Utc;

/// Strategy for generating valid requirement IDs
fn requirement_id_strategy() -> impl Strategy<Value = String> {
    r"req-[a-z0-9]{1,10}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid user stories
fn user_story_strategy() -> impl Strategy<Value = String> {
    r"As a [a-z]{3,10}, I want [a-z]{3,20}, so that [a-z]{3,20}"
        .prop_map(|s| s.to_string())
}

/// Strategy for generating acceptance criteria
fn acceptance_criteria_strategy() -> impl Strategy<Value = AcceptanceCriterion> {
    (
        r"ac-[a-z0-9]{1,10}",
        r"WHEN [a-z]{3,20}",
        r"THEN [a-z]{3,20}",
    )
        .prop_map(|(id, when, then)| AcceptanceCriterion {
            id: id.to_string(),
            when: when.to_string(),
            then: then.to_string(),
        })
}

/// Strategy for generating requirements
fn requirement_strategy() -> impl Strategy<Value = Requirement> {
    (
        requirement_id_strategy(),
        user_story_strategy(),
        prop::collection::vec(acceptance_criteria_strategy(), 1..3),
        prop_oneof![
            Just(Priority::Must),
            Just(Priority::Should),
            Just(Priority::Could),
        ],
    )
        .prop_map(|(id, user_story, acceptance_criteria, priority)| Requirement {
            id,
            user_story,
            acceptance_criteria,
            priority,
        })
}

/// Strategy for generating specs
fn spec_strategy() -> impl Strategy<Value = Spec> {
    (
        r"spec-[a-z0-9]{1,10}",
        r"[A-Z][a-z]{3,20}",
        r"[0-9]\.[0-9]\.[0-9]",
        prop::collection::vec(requirement_strategy(), 1..5),
    )
        .prop_map(|(id, name, version, mut requirements)| {
            // Ensure requirement IDs are unique
            for (idx, req) in requirements.iter_mut().enumerate() {
                req.id = format!("req-{}", idx);
            }
            Spec {
                id,
                name,
                version,
                requirements,
                design: None,
                tasks: vec![],
                metadata: SpecMetadata {
                    author: Some("Test".to_string()),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    phase: SpecPhase::Requirements,
                    status: SpecStatus::Approved,
                },
                inheritance: None,
            }
        })
}

proptest! {
    /// Property: All generation steps trace back to spec requirements
    ///
    /// For any valid spec, when we process it into a generation plan,
    /// every generation step must have at least one requirement ID that
    /// exists in the original spec.
    #[test]
    fn prop_all_steps_trace_to_requirements(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // Collect all requirement IDs from the spec
        let spec_requirement_ids: std::collections::HashSet<_> =
            spec.requirements.iter().map(|r| &r.id).collect();

        // For each step in the plan, verify it traces back to a spec requirement
        for step in &plan.steps {
            // Each step must have at least one requirement ID
            prop_assert!(
                !step.requirement_ids.is_empty(),
                "Step {} has no requirement IDs",
                step.id
            );

            // Each requirement ID in the step must exist in the spec
            for req_id in &step.requirement_ids {
                prop_assert!(
                    spec_requirement_ids.contains(req_id),
                    "Step {} references non-existent requirement: {}",
                    step.id,
                    req_id
                );
            }
        }
    }

    /// Property: All acceptance criteria are preserved in generation steps
    ///
    /// For any valid spec, when we process it into a generation plan,
    /// every acceptance criterion from the spec must appear in at least
    /// one generation step.
    #[test]
    fn prop_all_acceptance_criteria_preserved(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // Collect all acceptance criteria from the spec
        let spec_criteria: std::collections::HashSet<_> = spec
            .requirements
            .iter()
            .flat_map(|r| r.acceptance_criteria.iter().map(|ac| &ac.id))
            .collect();

        // Collect all acceptance criteria from the plan
        let plan_criteria: std::collections::HashSet<_> = plan
            .steps
            .iter()
            .flat_map(|s| s.acceptance_criteria.iter().map(|ac| &ac.id))
            .collect();

        // All spec criteria must be in the plan
        for criterion_id in spec_criteria {
            prop_assert!(
                plan_criteria.contains(criterion_id),
                "Acceptance criterion {} not found in generation plan",
                criterion_id
            );
        }
    }

    /// Property: Generation plan has correct number of steps
    ///
    /// For any valid spec with N requirements, the generation plan
    /// must have exactly N steps (one per requirement).
    #[test]
    fn prop_plan_has_one_step_per_requirement(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        prop_assert_eq!(
            plan.steps.len(),
            spec.requirements.len(),
            "Plan should have one step per requirement"
        );
    }

    /// Property: All steps have valid sequence numbers
    ///
    /// For any valid spec, when we process it into a generation plan,
    /// all steps must have unique, sequential sequence numbers starting from 0.
    #[test]
    fn prop_steps_have_valid_sequences(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        let mut sequences: Vec<_> = plan.steps.iter().map(|s| s.sequence).collect();
        sequences.sort();

        // Sequences should be 0, 1, 2, ..., n-1
        for (idx, &seq) in sequences.iter().enumerate() {
            prop_assert_eq!(
                seq, idx,
                "Sequence {} at position {} is not sequential",
                seq, idx
            );
        }
    }

    /// Property: Generation plan builder validates correctly
    ///
    /// For any valid spec, when we process it and validate the plan,
    /// the validation should pass (no errors).
    #[test]
    fn prop_valid_spec_produces_valid_plan(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        let builder = GenerationPlanBuilder::new();
        let validation = builder.validate_plan(&plan);

        prop_assert!(
            validation.is_valid,
            "Plan validation failed: {:?}",
            validation.errors
        );
    }

    /// Property: All constraints are extracted from requirements
    ///
    /// For any valid spec with requirements that mention constraints,
    /// the generation plan must extract those constraints.
    #[test]
    fn prop_constraints_extracted_from_requirements(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // Count how many acceptance criteria mention constraints
        let constraint_mentions = spec
            .requirements
            .iter()
            .flat_map(|r| &r.acceptance_criteria)
            .filter(|ac| {
                let text = ac.then.to_lowercase();
                text.contains("naming")
                    || text.contains("doc comment")
                    || text.contains("documentation")
                    || text.contains("error handling")
                    || text.contains("error type")
                    || text.contains("unit test")
                    || text.contains("quality")
                    || text.contains("standard")
            })
            .count();

        // If there are constraint mentions, we should have extracted constraints
        if constraint_mentions > 0 {
            prop_assert!(
                !plan.constraints.is_empty(),
                "No constraints extracted despite {} mentions in requirements",
                constraint_mentions
            );
        }
    }

    /// Property: Step IDs are unique
    ///
    /// For any valid spec, when we process it into a generation plan,
    /// all step IDs must be unique.
    #[test]
    fn prop_step_ids_are_unique(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        let mut ids: Vec<_> = plan.steps.iter().map(|s| &s.id).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();

        prop_assert_eq!(
            ids.len(),
            original_len,
            "Step IDs are not unique"
        );
    }

    /// Property: Priority is preserved from requirements to steps
    ///
    /// For any valid spec, when we process it into a generation plan,
    /// each step must have the same priority as its corresponding requirement.
    #[test]
    fn prop_priority_preserved_in_steps(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // Create a map of requirement IDs to priorities
        let req_priorities: std::collections::HashMap<_, _> = spec
            .requirements
            .iter()
            .map(|r| (r.id.clone(), r.priority))
            .collect();

        // Check that each step has the correct priority
        for step in &plan.steps {
            for req_id in &step.requirement_ids {
                if let Some(&expected_priority) = req_priorities.get(req_id) {
                    prop_assert_eq!(
                        step.priority, expected_priority,
                        "Step {} has priority {:?} but requirement {} has priority {:?}",
                        step.id, step.priority, req_id, expected_priority
                    );
                }
            }
        }
    }
}
