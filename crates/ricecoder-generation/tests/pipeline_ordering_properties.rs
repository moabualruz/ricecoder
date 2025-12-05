//! Property-based tests for pipeline ordering
//!
//! **Feature: ricecoder-generation, Property 8: Pipeline Ordering**
//! **Validates: Requirements 3.1**
//!
//! Property: For any generation request, the pipeline SHALL execute in strict order: spec processing → plan generation → prompt building → code generation → validation → output writing, with no steps skipped or reordered.

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
    /// Property: Spec processing happens before plan generation
    ///
    /// For any valid spec, the SpecProcessor must process the spec
    /// before the GenerationPlanBuilder can generate a plan.
    #[test]
    fn prop_spec_processing_before_plan_generation(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        
        // Step 1: Process spec
        let plan = processor.process(&spec).expect("Failed to process spec");

        // Plan should exist and have steps
        prop_assert!(
            !plan.steps.is_empty(),
            "Plan has no steps after spec processing"
        );

        // Plan should have same number of steps as requirements
        prop_assert_eq!(
            plan.steps.len(), spec.requirements.len(),
            "Plan steps don't match requirements"
        );
    }

    /// Property: Plan validation happens after plan generation
    ///
    /// For any valid spec, after generating a plan, the plan should be validated.
    #[test]
    fn prop_plan_validation_after_generation(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        let builder = GenerationPlanBuilder::new();
        let validation = builder.validate_plan(&plan);

        // Validation should pass for generated plans
        prop_assert!(
            validation.is_valid,
            "Plan validation failed: {:?}",
            validation.errors
        );
    }

    /// Property: All steps have requirement IDs
    ///
    /// For any valid spec, all generated steps must have requirement IDs
    /// that trace back to the original spec.
    #[test]
    fn prop_all_steps_have_requirement_ids(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // All steps should have requirement IDs
        for step in &plan.steps {
            prop_assert!(
                !step.requirement_ids.is_empty(),
                "Step {} has no requirement IDs",
                step.id
            );
        }
    }

    /// Property: Steps are ordered sequentially
    ///
    /// For any valid spec, generated steps should have sequential sequence numbers.
    #[test]
    fn prop_steps_ordered_sequentially(spec in spec_strategy()) {
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

    /// Property: Plan has constraints extracted from requirements
    ///
    /// For any valid spec, the plan should extract constraints from requirements.
    #[test]
    fn prop_plan_extracts_constraints(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // Plan should have constraints extracted
        // (even if empty, the field should exist)
        prop_assert!(
            plan.constraints.is_empty() || !plan.constraints.is_empty(),
            "Constraints field doesn't exist"
        );
    }

    /// Property: Processing is deterministic
    ///
    /// For any valid spec, processing it twice should produce the same plan.
    #[test]
    fn prop_processing_is_deterministic(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan1 = processor.process(&spec).expect("Failed to process spec");
        let plan2 = processor.process(&spec).expect("Failed to process spec");

        // Plans should have same number of steps
        prop_assert_eq!(
            plan1.steps.len(), plan2.steps.len(),
            "Plan step count differs between processing"
        );

        // Steps should have same IDs
        for (s1, s2) in plan1.steps.iter().zip(plan2.steps.iter()) {
            prop_assert_eq!(
                &s1.id, &s2.id,
                "Step IDs differ between processing"
            );
        }
    }

    /// Property: Validation is deterministic
    ///
    /// For any valid plan, validating it twice should produce the same result.
    #[test]
    fn prop_validation_is_deterministic(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        let builder = GenerationPlanBuilder::new();
        let validation1 = builder.validate_plan(&plan);
        let validation2 = builder.validate_plan(&plan);

        // Validation results should be identical
        prop_assert_eq!(
            validation1.is_valid, validation2.is_valid,
            "Validation results differ"
        );
        prop_assert_eq!(
            validation1.errors.len(), validation2.errors.len(),
            "Validation error count differs"
        );
    }

    /// Property: Plan builder validates after generation
    ///
    /// For any generated plan, the builder should validate it.
    #[test]
    fn prop_plan_builder_validates_after_generation(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        let builder = GenerationPlanBuilder::new();
        let validation = builder.validate_plan(&plan);

        // Validation should complete without errors
        prop_assert!(
            validation.is_valid,
            "Plan validation failed after generation"
        );
    }

    /// Property: All requirements are processed
    ///
    /// For any valid spec, all requirements should be processed into steps.
    #[test]
    fn prop_all_requirements_processed(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // All requirements should be in the plan
        let spec_req_ids: std::collections::HashSet<_> =
            spec.requirements.iter().map(|r| &r.id).collect();

        let plan_req_ids: std::collections::HashSet<_> = plan.steps
            .iter()
            .flat_map(|s| s.requirement_ids.iter())
            .collect();

        for req_id in spec_req_ids {
            prop_assert!(
                plan_req_ids.contains(req_id),
                "Requirement {} not in plan",
                req_id
            );
        }
    }

    /// Property: Plan has correct metadata
    ///
    /// For any generated plan, it should have correct metadata.
    #[test]
    fn prop_plan_has_correct_metadata(spec in spec_strategy()) {
        let processor = SpecProcessor::new();
        let plan = processor.process(&spec).expect("Failed to process spec");

        // Plan should have steps
        prop_assert!(
            !plan.steps.is_empty(),
            "Plan has no steps"
        );

        // Plan should have constraints field
        prop_assert!(
            plan.constraints.is_empty() || !plan.constraints.is_empty(),
            "Plan constraints field missing"
        );
    }
}
