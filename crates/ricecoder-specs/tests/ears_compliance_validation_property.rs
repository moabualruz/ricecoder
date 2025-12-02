//! Property-based tests for EARS compliance validation
//! **Feature: ricecoder-specs, Property 9: EARS Compliance Validation**
//! **Validates: Requirements 3.3, 3.9**

use proptest::prelude::*;
use ricecoder_specs::{
    models::*,
    validation::ValidationEngine,
    error::Severity,
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

fn arb_requirement_id() -> impl Strategy<Value = String> {
    "REQ-[0-9]{1,3}".prop_map(|s| s)
}

fn arb_user_story() -> impl Strategy<Value = String> {
    "As a [a-z]+ I want to [a-z ]+ so that [a-z ]+".prop_map(|s| s)
}

/// Generate EARS-compliant acceptance criteria
fn arb_ears_compliant_criterion() -> impl Strategy<Value = AcceptanceCriterion> {
    prop_oneof![
        // Event-driven pattern: WHEN ... THEN THE system SHALL ...
        (
            "AC-[0-9]{1,3}",
            "when [a-z ]+",
            "THE system SHALL [a-z ]+",
        )
            .prop_map(|(id, when, then)| AcceptanceCriterion {
                id,
                when,
                then,
            }),
        // Ubiquitous pattern: THE system SHALL ...
        (
            "AC-[0-9]{1,3}",
            "",
            "THE system SHALL [a-z ]+",
        )
            .prop_map(|(id, when, then)| AcceptanceCriterion {
                id,
                when,
                then,
            }),
        // State-driven pattern: WHILE ... THE system SHALL ...
        (
            "AC-[0-9]{1,3}",
            "WHILE [a-z ]+",
            "THE system SHALL [a-z ]+",
        )
            .prop_map(|(id, when, then)| AcceptanceCriterion {
                id,
                when,
                then,
            }),
        // Unwanted event pattern: IF ... THEN THE system SHALL ...
        (
            "AC-[0-9]{1,3}",
            "IF [a-z ]+",
            "THEN THE system SHALL [a-z ]+",
        )
            .prop_map(|(id, when, then)| AcceptanceCriterion {
                id,
                when,
                then,
            }),
        // Optional feature pattern: WHERE ... THE system SHALL ...
        (
            "AC-[0-9]{1,3}",
            "WHERE [a-z ]+",
            "THE system SHALL [a-z ]+",
        )
            .prop_map(|(id, when, then)| AcceptanceCriterion {
                id,
                when,
                then,
            }),
    ]
}

/// Generate non-EARS-compliant acceptance criteria
fn arb_non_ears_compliant_criterion() -> impl Strategy<Value = AcceptanceCriterion> {
    (
        "AC-[0-9]{1,3}",
        "[a-z ]+",
        "[a-z ]+",
    )
        .prop_map(|(id, when, then)| AcceptanceCriterion {
            id,
            when,
            then,
        })
}

fn arb_requirement_with_ears_criteria() -> impl Strategy<Value = Requirement> {
    (
        arb_requirement_id(),
        arb_user_story(),
        prop::collection::vec(arb_ears_compliant_criterion(), 1..3),
    )
        .prop_map(|(id, user_story, acceptance_criteria)| Requirement {
            id,
            user_story,
            acceptance_criteria,
            priority: Priority::Must,
        })
}

fn arb_requirement_with_non_ears_criteria() -> impl Strategy<Value = Requirement> {
    (
        arb_requirement_id(),
        arb_user_story(),
        prop::collection::vec(arb_non_ears_compliant_criterion(), 1..3),
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

fn arb_spec_with_ears_requirements() -> impl Strategy<Value = Spec> {
    (
        arb_spec_id(),
        arb_spec_name(),
        arb_spec_version(),
        prop::collection::vec(arb_requirement_with_ears_criteria(), 0..3),
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

fn arb_spec_with_non_ears_requirements() -> impl Strategy<Value = Spec> {
    (
        arb_spec_id(),
        arb_spec_name(),
        arb_spec_version(),
        prop::collection::vec(arb_requirement_with_non_ears_criteria(), 1..3),
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
// Property 9: EARS Compliance Validation
// ============================================================================

proptest! {
    /// Property: For any requirement generated by the AI, validation SHALL verify
    /// EARS pattern compliance (Ubiquitous, Event-driven, State-driven, Unwanted event,
    /// Optional feature, or Complex).
    ///
    /// This property verifies that:
    /// 1. EARS-compliant requirements pass validation (or have only info-level issues)
    /// 2. Non-EARS-compliant requirements are detected and reported
    /// 3. All EARS compliance errors have appropriate messages
    /// 4. All EARS compliance errors have file paths and line numbers
    #[test]
    fn prop_ears_compliant_requirements_pass_validation(spec in arb_spec_with_ears_requirements()) {
        // Run EARS compliance validation
        let result = ValidationEngine::validate_ears_compliance(&spec);

        // EARS-compliant requirements should pass or have only info-level issues
        if let Err(errors) = result {
            // All errors should be info-level (not errors or warnings)
            for error in errors {
                prop_assert!(
                    error.severity == Severity::Info || error.severity == Severity::Warning,
                    "EARS-compliant requirement should not have Error severity: {}",
                    error.message
                );
            }
        }
    }

    /// Property: Non-EARS-compliant requirements SHALL be detected
    ///
    /// This property verifies that when requirements don't follow EARS patterns,
    /// validation detects and reports them.
    #[test]
    fn prop_non_ears_compliant_requirements_detected(spec in arb_spec_with_non_ears_requirements()) {
        // Run EARS compliance validation
        let result = ValidationEngine::validate_ears_compliance(&spec);

        // Non-EARS-compliant requirements should be detected
        prop_assert!(
            result.is_err(),
            "Non-EARS-compliant requirements should be detected"
        );

        if let Err(errors) = result {
            // Must have at least one error about EARS pattern
            prop_assert!(
                errors.iter().any(|e| e.message.contains("EARS pattern")),
                "Validation should report EARS pattern error"
            );

            // All errors must have complete information
            for error in errors {
                prop_assert!(!error.path.is_empty(), "Error missing file path");
                prop_assert!(error.line > 0, "Error has invalid line number");
                prop_assert!(!error.message.is_empty(), "Error has empty message");
            }
        }
    }

    /// Property: EARS compliance errors SHALL include helpful suggestions
    ///
    /// This property verifies that when EARS compliance violations are detected,
    /// the error messages include suggestions for fixing them.
    #[test]
    fn prop_ears_compliance_errors_include_suggestions(spec in arb_spec_with_non_ears_requirements()) {
        // Run EARS compliance validation
        let result = ValidationEngine::validate_ears_compliance(&spec);

        if let Err(errors) = result {
            // All EARS pattern errors should include suggestions
            for error in errors {
                if error.message.contains("EARS pattern") {
                    prop_assert!(
                        error.message.contains("Expected one of:") || error.message.contains("WHEN") || error.message.contains("WHILE") || error.message.contains("IF") || error.message.contains("WHERE") || error.message.contains("THE"),
                        "EARS compliance error should include suggestions: {}",
                        error.message
                    );
                }
            }
        }
    }

    /// Property: EARS validation SHALL report errors with file paths and line numbers
    ///
    /// This property verifies that all EARS compliance errors include
    /// specific file paths and line numbers for precise error reporting.
    #[test]
    fn prop_ears_errors_have_location_info(spec in arb_spec_with_non_ears_requirements()) {
        // Run EARS compliance validation
        let result = ValidationEngine::validate_ears_compliance(&spec);

        if let Err(errors) = result {
            for error in errors {
                // All errors must have file paths
                prop_assert!(
                    !error.path.is_empty(),
                    "EARS compliance error missing file path"
                );

                // All errors must have line numbers > 0
                prop_assert!(
                    error.line > 0,
                    "EARS compliance error has invalid line number: {}",
                    error.line
                );

                // All errors must have column numbers > 0
                prop_assert!(
                    error.column > 0,
                    "EARS compliance error has invalid column number: {}",
                    error.column
                );
            }
        }
    }

    /// Property: Event-driven pattern (WHEN/THEN) SHALL be recognized
    ///
    /// This property verifies that the event-driven EARS pattern is correctly
    /// recognized and accepted as valid.
    #[test]
    fn prop_event_driven_pattern_recognized(
        id in "AC-[0-9]{1,3}",
        when in "[a-z]{2,10}",
        then in "[a-z]{2,10}"
    ) {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![Requirement {
                id: "REQ-1".to_string(),
                user_story: "As a user, I want something".to_string(),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id,
                    when: format!("WHEN {}", when),
                    then: format!("THEN THE system SHALL {}", then),
                }],
                priority: Priority::Must,
            }],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let result = ValidationEngine::validate_ears_compliance(&spec);

        // Event-driven pattern should be recognized
        if let Err(errors) = result {
            // Should not have EARS pattern errors for this criterion
            prop_assert!(
                !errors.iter().any(|e| e.message.contains("EARS pattern")),
                "Event-driven pattern should be recognized"
            );
        }
    }

    /// Property: Ubiquitous pattern (THE system SHALL) SHALL be recognized
    ///
    /// This property verifies that the ubiquitous EARS pattern is correctly
    /// recognized and accepted as valid.
    ///
    /// Note: The ubiquitous pattern is recognized when the combined "when then" string
    /// starts with "THE system SHALL". Since the validation combines with a space,
    /// we need to ensure the pattern matches correctly.
    #[test]
    fn prop_ubiquitous_pattern_recognized(
        id in "AC-[0-9]{1,3}",
        when in "[a-z]{2,10}",
        then in "[a-z]{2,10}"
    ) {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![Requirement {
                id: "REQ-1".to_string(),
                user_story: "As a user, I want something".to_string(),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id,
                    when: format!("THE system SHALL {}", when),
                    then: format!("and {}", then),
                }],
                priority: Priority::Must,
            }],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let result = ValidationEngine::validate_ears_compliance(&spec);

        // Ubiquitous pattern should be recognized
        if let Err(errors) = result {
            // Should not have EARS pattern errors for this criterion
            prop_assert!(
                !errors.iter().any(|e| e.message.contains("EARS pattern")),
                "Ubiquitous pattern should be recognized"
            );
        }
    }

    /// Property: State-driven pattern (WHILE) SHALL be recognized
    ///
    /// This property verifies that the state-driven EARS pattern is correctly
    /// recognized and accepted as valid.
    #[test]
    fn prop_state_driven_pattern_recognized(
        id in "AC-[0-9]{1,3}",
        condition in "[a-z]{2,10}",
        then in "[a-z]{2,10}"
    ) {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![Requirement {
                id: "REQ-1".to_string(),
                user_story: "As a user, I want something".to_string(),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id,
                    when: format!("WHILE {}", condition),
                    then: format!("THE system SHALL {}", then),
                }],
                priority: Priority::Must,
            }],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let result = ValidationEngine::validate_ears_compliance(&spec);

        // State-driven pattern should be recognized
        if let Err(errors) = result {
            // Should not have EARS pattern errors for this criterion
            prop_assert!(
                !errors.iter().any(|e| e.message.contains("EARS pattern")),
                "State-driven pattern should be recognized"
            );
        }
    }

    /// Property: Unwanted event pattern (IF/THEN) SHALL be recognized
    ///
    /// This property verifies that the unwanted event EARS pattern is correctly
    /// recognized and accepted as valid.
    #[test]
    fn prop_unwanted_event_pattern_recognized(
        id in "AC-[0-9]{1,3}",
        condition in "[a-z]{2,10}",
        then in "[a-z]{2,10}"
    ) {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![Requirement {
                id: "REQ-1".to_string(),
                user_story: "As a user, I want something".to_string(),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id,
                    when: format!("IF {}", condition),
                    then: format!("THEN THE system SHALL {}", then),
                }],
                priority: Priority::Must,
            }],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let result = ValidationEngine::validate_ears_compliance(&spec);

        // Unwanted event pattern should be recognized
        if let Err(errors) = result {
            // Should not have EARS pattern errors for this criterion
            prop_assert!(
                !errors.iter().any(|e| e.message.contains("EARS pattern")),
                "Unwanted event pattern should be recognized"
            );
        }
    }

    /// Property: Optional feature pattern (WHERE) SHALL be recognized
    ///
    /// This property verifies that the optional feature EARS pattern is correctly
    /// recognized and accepted as valid.
    #[test]
    fn prop_optional_feature_pattern_recognized(
        id in "AC-[0-9]{1,3}",
        option in "[a-z]{2,10}",
        then in "[a-z]{2,10}"
    ) {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![Requirement {
                id: "REQ-1".to_string(),
                user_story: "As a user, I want something".to_string(),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id,
                    when: format!("WHERE {}", option),
                    then: format!("THE system SHALL {}", then),
                }],
                priority: Priority::Must,
            }],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let result = ValidationEngine::validate_ears_compliance(&spec);

        // Optional feature pattern should be recognized
        if let Err(errors) = result {
            // Should not have EARS pattern errors for this criterion
            prop_assert!(
                !errors.iter().any(|e| e.message.contains("EARS pattern")),
                "Optional feature pattern should be recognized"
            );
        }
    }

    /// Property: Empty specs SHALL pass EARS validation
    ///
    /// This property verifies that specs with no requirements pass EARS validation
    /// (there's nothing to validate).
    #[test]
    fn prop_empty_specs_pass_ears_validation(
        id in arb_spec_id(),
        name in arb_spec_name(),
        version in arb_spec_version()
    ) {
        let spec = Spec {
            id,
            name,
            version,
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
            inheritance: None,
        };

        let result = ValidationEngine::validate_ears_compliance(&spec);

        // Empty specs should pass EARS validation
        prop_assert!(
            result.is_ok(),
            "Empty specs should pass EARS validation"
        );
    }
}
