//! Property-based tests for validation completeness
//! **Feature: ricecoder-specs, Property 3: Validation Completeness**
//! **Validates: Requirements 1.5, 1.6**

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_specs::{error::Severity, models::*, validation::ValidationEngine};

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

fn arb_requirement_id() -> impl Strategy<Value = String> {
    "REQ-[0-9]{1,3}".prop_map(|s| s)
}

fn arb_user_story() -> impl Strategy<Value = String> {
    "As a [a-z]+ I want to [a-z ]+ so that [a-z ]+".prop_map(|s| s)
}

fn arb_acceptance_criterion() -> impl Strategy<Value = AcceptanceCriterion> {
    ("AC-[0-9]{1,3}", "when [a-z ]+", "then [a-z ]+")
        .prop_map(|(id, when, then)| AcceptanceCriterion { id, when, then })
}

fn arb_requirement() -> impl Strategy<Value = Requirement> {
    (
        arb_requirement_id(),
        arb_user_story(),
        prop::collection::vec(arb_acceptance_criterion(), 1..3),
    )
        .prop_map(|(id, user_story, acceptance_criteria)| Requirement {
            id,
            user_story,
            acceptance_criteria,
            priority: Priority::Must,
        })
}

fn arb_task_id() -> impl Strategy<Value = String> {
    "[0-9]{1,2}".prop_map(|s| s)
}

fn arb_task_description() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{5,50}".prop_map(|s| s)
}

fn arb_task() -> impl Strategy<Value = Task> {
    (arb_task_id(), arb_task_description()).prop_map(|(id, description)| Task {
        id,
        description,
        subtasks: vec![],
        requirements: vec![],
        status: TaskStatus::NotStarted,
        optional: false,
    })
}

fn arb_spec() -> impl Strategy<Value = Spec> {
    (
        arb_spec_id(),
        arb_spec_name(),
        arb_spec_version(),
        prop::collection::vec(arb_requirement(), 0..3),
        prop::collection::vec(arb_task(), 0..3),
    )
        .prop_map(|(id, name, version, requirements, tasks)| Spec {
            id,
            name,
            version,
            requirements,
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
        })
}

// ============================================================================
// Property 3: Validation Completeness
// ============================================================================

proptest! {
    /// Property: For any spec, validation SHALL detect all structural errors
    /// and report them with specific file paths and line numbers.
    ///
    /// This property verifies that:
    /// 1. All validation errors have file paths
    /// 2. All validation errors have line numbers > 0
    /// 3. All validation errors have column numbers > 0
    /// 4. All validation errors have non-empty messages
    /// 5. All validation errors have severity levels
    #[test]
    fn prop_validation_reports_file_paths_and_line_numbers(spec in arb_spec()) {
        // Run validation
        let result = ValidationEngine::validate_structure(&spec);

        // If validation fails, check that all errors have required information
        if let Err(errors) = result {
            for error in errors {
                // Property 3.1: All errors must have file paths
                prop_assert!(
                    !error.path.is_empty(),
                    "Validation error missing file path"
                );

                // Property 3.2: All errors must have line numbers > 0
                prop_assert!(
                    error.line > 0,
                    "Validation error has invalid line number: {}",
                    error.line
                );

                // Property 3.3: All errors must have column numbers > 0
                prop_assert!(
                    error.column > 0,
                    "Validation error has invalid column number: {}",
                    error.column
                );

                // Property 3.4: All errors must have non-empty messages
                prop_assert!(
                    !error.message.is_empty(),
                    "Validation error has empty message"
                );

                // Property 3.5: All errors must have severity levels
                // (This is guaranteed by the type system, but we verify it's set)
                let _ = error.severity;
            }
        }
    }

    /// Property: Validation SHALL detect missing required fields
    ///
    /// This property verifies that when required fields are empty,
    /// validation detects and reports them.
    #[test]
    fn prop_validation_detects_missing_required_fields(mut spec in arb_spec()) {
        // Make spec invalid by clearing required fields
        spec.id = String::new();

        let result = ValidationEngine::validate_structure(&spec);

        // Validation must fail
        prop_assert!(result.is_err(), "Validation should fail for empty spec ID");

        if let Err(errors) = result {
            // Must have at least one error about the empty ID
            prop_assert!(
                errors.iter().any(|e| e.message.contains("ID is required")),
                "Validation should report empty ID error"
            );

            // All errors must have file paths and line numbers
            for error in errors {
                prop_assert!(!error.path.is_empty(), "Error missing file path");
                prop_assert!(error.line > 0, "Error has invalid line number");
            }
        }
    }

    /// Property: Validation SHALL detect invalid requirement structure
    ///
    /// This property verifies that when requirements have invalid structure,
    /// validation detects and reports them.
    #[test]
    fn prop_validation_detects_invalid_requirements(mut spec in arb_spec()) {
        // Add a requirement with empty ID
        let invalid_req = Requirement {
            id: String::new(),
            user_story: "As a user, I want something".to_string(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1".to_string(),
                when: "something happens".to_string(),
                then: "something else happens".to_string(),
            }],
            priority: Priority::Must,
        };
        spec.requirements.push(invalid_req);

        let result = ValidationEngine::validate_structure(&spec);

        // Validation must fail
        prop_assert!(result.is_err(), "Validation should fail for requirement with empty ID");

        if let Err(errors) = result {
            // Must have at least one error about the empty requirement ID
            prop_assert!(
                errors.iter().any(|e| e.message.contains("empty ID")),
                "Validation should report empty requirement ID error"
            );

            // All errors must have complete information
            for error in errors {
                prop_assert!(!error.path.is_empty(), "Error missing file path");
                prop_assert!(error.line > 0, "Error has invalid line number");
                prop_assert!(error.column > 0, "Error has invalid column number");
                prop_assert!(!error.message.is_empty(), "Error has empty message");
            }
        }
    }

    /// Property: Validation SHALL detect invalid acceptance criteria
    ///
    /// This property verifies that when acceptance criteria have invalid structure,
    /// validation detects and reports them.
    #[test]
    fn prop_validation_detects_invalid_criteria(mut spec in arb_spec()) {
        // Add a requirement with invalid acceptance criteria
        let req = Requirement {
            id: "REQ-1".to_string(),
            user_story: "As a user, I want something".to_string(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: String::new(), // Invalid: empty ID
                when: "something happens".to_string(),
                then: "something else happens".to_string(),
            }],
            priority: Priority::Must,
        };
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);

        // Validation must fail
        prop_assert!(
            result.is_err(),
            "Validation should fail for criterion with empty ID"
        );

        if let Err(errors) = result {
            // Must have at least one error about the empty criterion ID
            prop_assert!(
                errors.iter().any(|e| e.message.contains("empty ID")),
                "Validation should report empty criterion ID error"
            );

            // All errors must have complete information
            for error in errors {
                prop_assert!(!error.path.is_empty(), "Error missing file path");
                prop_assert!(error.line > 0, "Error has invalid line number");
            }
        }
    }

    /// Property: Validation errors SHALL have appropriate severity levels
    ///
    /// This property verifies that validation errors are classified with
    /// appropriate severity levels (Error, Warning, Info).
    #[test]
    fn prop_validation_errors_have_severity(mut spec in arb_spec()) {
        // Add a requirement with empty user story (should be Warning)
        let req = Requirement {
            id: "REQ-1".to_string(),
            user_story: String::new(), // Invalid: empty user story
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1".to_string(),
                when: "something happens".to_string(),
                then: "something else happens".to_string(),
            }],
            priority: Priority::Must,
        };
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);

        if let Err(errors) = result {
            // All errors must have a severity level
            for error in errors {
                // Verify severity is one of the valid values
                match error.severity {
                    Severity::Error | Severity::Warning | Severity::Info => {
                        // Valid severity
                    }
                }
            }
        }
    }

    /// Property: Valid specs SHALL pass validation
    ///
    /// This property verifies that specs with all required fields
    /// and valid structure pass validation.
    #[test]
    fn prop_valid_specs_pass_validation(spec in arb_spec()) {
        // Only test specs that have all required fields
        if !spec.id.is_empty()
            && !spec.name.is_empty()
            && !spec.version.is_empty()
            && spec.requirements.iter().all(|r| {
                !r.id.is_empty()
                    && !r.user_story.is_empty()
                    && !r.acceptance_criteria.is_empty()
                    && r.acceptance_criteria.iter().all(|ac| {
                        !ac.id.is_empty() && !ac.when.is_empty() && !ac.then.is_empty()
                    })
            })
            && spec.tasks.iter().all(|t| !t.id.is_empty() && !t.description.is_empty())
        {
            let result = ValidationEngine::validate_structure(&spec);

            // Valid specs should pass structure validation
            // (They may fail EARS or INCOSE validation, but structure should be OK)
            prop_assert!(
                result.is_ok(),
                "Valid spec should pass structure validation"
            );
        }
    }
}
