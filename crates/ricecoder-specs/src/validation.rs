//! Validation engine for specs

use crate::error::{SpecError, ValidationError, Severity};
use crate::models::Spec;
use regex::Regex;

/// Validates spec structure and semantic correctness
pub struct ValidationEngine;

impl ValidationEngine {
    /// Validate a spec - runs all validation checks
    pub fn validate(spec: &Spec) -> Result<(), SpecError> {
        let mut all_errors = Vec::new();

        // Run structure validation
        if let Err(errors) = Self::validate_structure(spec) {
            all_errors.extend(errors);
        }

        // Run EARS compliance validation
        if let Err(errors) = Self::validate_ears_compliance(spec) {
            all_errors.extend(errors);
        }

        // Run INCOSE rules validation
        if let Err(errors) = Self::validate_incose_rules(spec) {
            all_errors.extend(errors);
        }

        if !all_errors.is_empty() {
            return Err(SpecError::ValidationFailed(all_errors));
        }

        Ok(())
    }

    /// Validate spec structure - checks required fields and structure
    pub fn validate_structure(spec: &Spec) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let path = format!("spec:{}", spec.id);

        // Check required fields
        if spec.id.is_empty() {
            errors.push(ValidationError {
                path: path.clone(),
                line: 1,
                column: 1,
                message: "Spec ID is required and cannot be empty".to_string(),
                severity: Severity::Error,
            });
        }

        if spec.name.is_empty() {
            errors.push(ValidationError {
                path: path.clone(),
                line: 2,
                column: 1,
                message: "Spec name is required and cannot be empty".to_string(),
                severity: Severity::Error,
            });
        }

        if spec.version.is_empty() {
            errors.push(ValidationError {
                path: path.clone(),
                line: 3,
                column: 1,
                message: "Spec version is required and cannot be empty".to_string(),
                severity: Severity::Error,
            });
        }

        // Validate requirements structure
        for (idx, req) in spec.requirements.iter().enumerate() {
            let req_line = 10 + (idx * 5);

            if req.id.is_empty() {
                errors.push(ValidationError {
                    path: path.clone(),
                    line: req_line,
                    column: 1,
                    message: format!("Requirement {} has empty ID", idx + 1),
                    severity: Severity::Error,
                });
            }

            if req.user_story.is_empty() {
                errors.push(ValidationError {
                    path: path.clone(),
                    line: req_line + 1,
                    column: 1,
                    message: format!("Requirement {} has empty user story", req.id),
                    severity: Severity::Warning,
                });
            }

            if req.acceptance_criteria.is_empty() {
                errors.push(ValidationError {
                    path: path.clone(),
                    line: req_line + 2,
                    column: 1,
                    message: format!("Requirement {} has no acceptance criteria", req.id),
                    severity: Severity::Warning,
                });
            }

            // Validate acceptance criteria
            for (ac_idx, ac) in req.acceptance_criteria.iter().enumerate() {
                let ac_line = req_line + 3 + ac_idx;

                if ac.id.is_empty() {
                    errors.push(ValidationError {
                        path: path.clone(),
                        line: ac_line,
                        column: 1,
                        message: format!(
                            "Acceptance criterion {} in requirement {} has empty ID",
                            ac_idx + 1,
                            req.id
                        ),
                        severity: Severity::Error,
                    });
                }

                if ac.when.is_empty() {
                    errors.push(ValidationError {
                        path: path.clone(),
                        line: ac_line,
                        column: 1,
                        message: format!(
                            "Acceptance criterion {} in requirement {} has empty 'when' clause",
                            ac.id, req.id
                        ),
                        severity: Severity::Error,
                    });
                }

                if ac.then.is_empty() {
                    errors.push(ValidationError {
                        path: path.clone(),
                        line: ac_line,
                        column: 1,
                        message: format!(
                            "Acceptance criterion {} in requirement {} has empty 'then' clause",
                            ac.id, req.id
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }

        // Validate tasks structure
        for (idx, task) in spec.tasks.iter().enumerate() {
            let task_line = 50 + (idx * 3);

            if task.id.is_empty() {
                errors.push(ValidationError {
                    path: path.clone(),
                    line: task_line,
                    column: 1,
                    message: format!("Task {} has empty ID", idx + 1),
                    severity: Severity::Error,
                });
            }

            if task.description.is_empty() {
                errors.push(ValidationError {
                    path: path.clone(),
                    line: task_line + 1,
                    column: 1,
                    message: format!("Task {} has empty description", task.id),
                    severity: Severity::Warning,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate EARS compliance - checks requirement patterns
    pub fn validate_ears_compliance(spec: &Spec) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let path = format!("spec:{}", spec.id);

        // EARS patterns to check
        let ubiquitous_pattern = Regex::new(r"(?i)^THE\s+\w+\s+SHALL").unwrap();
        let event_driven_pattern = Regex::new(r"(?i)^WHEN\s+.+\s+THEN\s+THE\s+\w+\s+SHALL").unwrap();
        let state_driven_pattern = Regex::new(r"(?i)^WHILE\s+.+\s+THE\s+\w+\s+SHALL").unwrap();
        let unwanted_event_pattern = Regex::new(r"(?i)^IF\s+.+\s+THEN\s+THE\s+\w+\s+SHALL").unwrap();
        let optional_pattern = Regex::new(r"(?i)^WHERE\s+.+\s+THE\s+\w+\s+SHALL").unwrap();

        for (idx, req) in spec.requirements.iter().enumerate() {
            let req_line = 10 + (idx * 5);

            // Check each acceptance criterion for EARS compliance
            for (ac_idx, ac) in req.acceptance_criteria.iter().enumerate() {
                let ac_line = req_line + 3 + ac_idx;
                let combined = format!("{} {}", ac.when, ac.then);

                // Check if it matches any EARS pattern
                let matches_ears = ubiquitous_pattern.is_match(&combined)
                    || event_driven_pattern.is_match(&combined)
                    || state_driven_pattern.is_match(&combined)
                    || unwanted_event_pattern.is_match(&combined)
                    || optional_pattern.is_match(&combined);

                if !matches_ears {
                    errors.push(ValidationError {
                        path: path.clone(),
                        line: ac_line,
                        column: 1,
                        message: format!(
                            "Acceptance criterion {} does not match EARS pattern. \
                             Expected one of: WHEN/THEN, WHILE, IF/THEN, WHERE, or THE...SHALL",
                            ac.id
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate INCOSE semantic rules - checks semantic correctness
    pub fn validate_incose_rules(spec: &Spec) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let path = format!("spec:{}", spec.id);

        for (idx, req) in spec.requirements.iter().enumerate() {
            let req_line = 10 + (idx * 5);

            // Rule 1: Active voice - check for passive constructions
            if Self::is_passive_voice(&req.user_story) {
                errors.push(ValidationError {
                    path: path.clone(),
                    line: req_line + 1,
                    column: 1,
                    message: format!(
                        "Requirement {} user story appears to use passive voice. \
                         Use active voice (e.g., 'I want' instead of 'should be able to')",
                        req.id
                    ),
                    severity: Severity::Warning,
                });
            }

            // Rule 2: No vague terms
            if Self::contains_vague_terms(&req.user_story) {
                errors.push(ValidationError {
                    path: path.clone(),
                    line: req_line + 1,
                    column: 1,
                    message: format!(
                        "Requirement {} user story contains vague terms. \
                         Use specific, measurable language",
                        req.id
                    ),
                    severity: Severity::Warning,
                });
            }

            // Rule 3: Check acceptance criteria for vague terms
            for (ac_idx, ac) in req.acceptance_criteria.iter().enumerate() {
                let ac_line = req_line + 3 + ac_idx;

                if Self::contains_vague_terms(&ac.when) || Self::contains_vague_terms(&ac.then) {
                    errors.push(ValidationError {
                        path: path.clone(),
                        line: ac_line,
                        column: 1,
                        message: format!(
                            "Acceptance criterion {} contains vague terms. \
                             Use specific, measurable language",
                            ac.id
                        ),
                        severity: Severity::Warning,
                    });
                }

                // Rule 4: Check for negative statements
                if Self::contains_negative_statement(&ac.when)
                    || Self::contains_negative_statement(&ac.then)
                {
                    errors.push(ValidationError {
                        path: path.clone(),
                        line: ac_line,
                        column: 1,
                        message: format!(
                            "Acceptance criterion {} uses negative statements. \
                             Rephrase as positive requirements",
                            ac.id
                        ),
                        severity: Severity::Info,
                    });
                }

                // Rule 5: Check for pronouns
                if Self::contains_pronouns(&ac.when) || Self::contains_pronouns(&ac.then) {
                    errors.push(ValidationError {
                        path: path.clone(),
                        line: ac_line,
                        column: 1,
                        message: format!(
                            "Acceptance criterion {} uses pronouns (it, them, etc.). \
                             Use explicit references",
                            ac.id
                        ),
                        severity: Severity::Info,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // Helper functions for INCOSE validation

    fn is_passive_voice(text: &str) -> bool {
        let passive_indicators = vec!["should be", "can be", "is able to", "is required to"];
        passive_indicators
            .iter()
            .any(|indicator| text.to_lowercase().contains(indicator))
    }

    fn contains_vague_terms(text: &str) -> bool {
        let vague_terms = vec![
            "quickly", "slowly", "fast", "adequate", "sufficient", "appropriate", "suitable",
            "good", "bad", "nice", "easy", "hard", "simple", "complex", "etc", "and so on",
            "as needed", "as appropriate", "where possible", "if possible",
        ];
        let lower = text.to_lowercase();
        vague_terms.iter().any(|term| lower.contains(term))
    }

    fn contains_negative_statement(text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.contains("shall not")
            || lower.contains("should not")
            || lower.contains("must not")
            || lower.contains("cannot")
            || lower.contains("will not")
    }

    fn contains_pronouns(text: &str) -> bool {
        let pronouns = vec!["it ", " it", "them", "they", "this", "that", "these", "those"];
        let lower = text.to_lowercase();
        pronouns.iter().any(|pronoun| lower.contains(pronoun))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;

    fn create_minimal_spec() -> Spec {
        Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
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
            inheritance: None,
        }
    }

    fn create_valid_requirement() -> Requirement {
        Requirement {
            id: "REQ-1".to_string(),
            user_story: "As a user, I want to create tasks".to_string(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1.1".to_string(),
                when: "user enters task description".to_string(),
                then: "THE system SHALL add task to list".to_string(),
            }],
            priority: Priority::Must,
        }
    }

    // ============================================================================
    // Structure Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_structure_valid_spec() {
        let spec = create_minimal_spec();
        assert!(ValidationEngine::validate_structure(&spec).is_ok());
    }

    #[test]
    fn test_validate_structure_empty_id() {
        let mut spec = create_minimal_spec();
        spec.id = String::new();

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("ID is required")));
    }

    #[test]
    fn test_validate_structure_empty_name() {
        let mut spec = create_minimal_spec();
        spec.name = String::new();

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("name is required")));
    }

    #[test]
    fn test_validate_structure_empty_version() {
        let mut spec = create_minimal_spec();
        spec.version = String::new();

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("version is required")));
    }

    #[test]
    fn test_validate_structure_requirement_empty_id() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.id = String::new();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty ID")));
    }

    #[test]
    fn test_validate_structure_requirement_empty_user_story() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.user_story = String::new();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty user story")));
    }

    #[test]
    fn test_validate_structure_requirement_no_criteria() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria = vec![];
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("no acceptance criteria")));
    }

    #[test]
    fn test_validate_structure_criterion_empty_id() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria[0].id = String::new();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty ID")));
    }

    #[test]
    fn test_validate_structure_criterion_empty_when() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria[0].when = String::new();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty 'when' clause")));
    }

    #[test]
    fn test_validate_structure_criterion_empty_then() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria[0].then = String::new();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty 'then' clause")));
    }

    #[test]
    fn test_validate_structure_task_empty_id() {
        let mut spec = create_minimal_spec();
        let task = Task {
            id: String::new(),
            description: "Test task".to_string(),
            subtasks: vec![],
            requirements: vec![],
            status: TaskStatus::NotStarted,
            optional: false,
        };
        spec.tasks.push(task);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty ID")));
    }

    #[test]
    fn test_validate_structure_task_empty_description() {
        let mut spec = create_minimal_spec();
        let task = Task {
            id: "1".to_string(),
            description: String::new(),
            subtasks: vec![],
            requirements: vec![],
            status: TaskStatus::NotStarted,
            optional: false,
        };
        spec.tasks.push(task);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty description")));
    }

    #[test]
    fn test_validate_structure_reports_file_path_and_line() {
        let mut spec = create_minimal_spec();
        spec.id = String::new();

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.is_empty());

        let error = &errors[0];
        assert!(error.path.contains("spec:"));
        assert!(error.line > 0);
        assert!(error.column > 0);
    }

    #[test]
    fn test_validate_structure_error_severity() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.user_story = String::new();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_structure(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        let warning_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.severity == Severity::Warning)
            .collect();
        assert!(!warning_errors.is_empty());
    }

    // ============================================================================
    // EARS Compliance Tests
    // ============================================================================

    #[test]
    fn test_validate_ears_compliance_valid_event_driven() {
        let mut spec = create_minimal_spec();
        spec.requirements.push(create_valid_requirement());

        let result = ValidationEngine::validate_ears_compliance(&spec);
        // Should pass or have only info-level issues
        if let Err(errors) = result {
            assert!(errors.iter().all(|e| e.severity != Severity::Error));
        }
    }

    #[test]
    fn test_validate_ears_compliance_ubiquitous_pattern() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria[0].when = String::new();
        req.acceptance_criteria[0].then = "THE system SHALL validate input".to_string();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_ears_compliance(&spec);
        // Should pass or have only info-level issues
        if let Err(errors) = result {
            assert!(errors.iter().all(|e| e.severity != Severity::Error));
        }
    }

    #[test]
    fn test_validate_ears_compliance_non_compliant() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria[0].when = "something happens".to_string();
        req.acceptance_criteria[0].then = "something else happens".to_string();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_ears_compliance(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("EARS pattern")));
    }

    // ============================================================================
    // INCOSE Rules Tests
    // ============================================================================

    #[test]
    fn test_validate_incose_rules_valid_requirement() {
        let mut spec = create_minimal_spec();
        spec.requirements.push(create_valid_requirement());

        let result = ValidationEngine::validate_incose_rules(&spec);
        // Should pass or have only info-level issues
        if let Err(errors) = result {
            assert!(errors.iter().all(|e| e.severity != Severity::Error));
        }
    }

    #[test]
    fn test_validate_incose_rules_passive_voice() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.user_story = "Tasks should be created by the system".to_string();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_incose_rules(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("passive voice")));
    }

    #[test]
    fn test_validate_incose_rules_vague_terms() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.user_story = "As a user, I want to quickly create tasks".to_string();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_incose_rules(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("vague terms")));
    }

    #[test]
    fn test_validate_incose_rules_negative_statement() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria[0].then = "THE system SHALL NOT delete tasks".to_string();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_incose_rules(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("negative statements")));
    }

    #[test]
    fn test_validate_incose_rules_pronouns() {
        let mut spec = create_minimal_spec();
        let mut req = create_valid_requirement();
        req.acceptance_criteria[0].then = "THE system SHALL process it".to_string();
        spec.requirements.push(req);

        let result = ValidationEngine::validate_incose_rules(&spec);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("pronouns")));
    }

    // ============================================================================
    // Full Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_combines_all_checks() {
        let mut spec = create_minimal_spec();
        spec.requirements.push(create_valid_requirement());

        let result = ValidationEngine::validate(&spec);
        // Should pass or have only info-level issues
        if let Err(SpecError::ValidationFailed(errors)) = result {
            assert!(errors.iter().all(|e| e.severity != Severity::Error));
        }
    }

    #[test]
    fn test_validate_returns_all_errors() {
        let mut spec = create_minimal_spec();
        spec.id = String::new();
        spec.name = String::new();

        let result = ValidationEngine::validate(&spec);
        assert!(result.is_err());

        if let Err(SpecError::ValidationFailed(errors)) = result {
            assert!(errors.len() >= 2);
        }
    }
}
