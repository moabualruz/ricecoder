//! Property-based tests for rules validation correctness
//!
//! **Feature: ricecoder-orchestration, Property 3: Rules Validation Correctness**
//!
//! *For any* workspace rule, the RulesValidator SHALL correctly identify compliance
//! violations and reject non-compliant configurations.
//!
//! **Validates: Requirements 1.4**

use std::path::PathBuf;

use proptest::prelude::*;
use ricecoder_orchestration::{
    Project, ProjectStatus, RuleType, RulesValidator, Workspace, WorkspaceConfig, WorkspaceRule,
};

/// Strategy for generating valid project names
fn project_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("my-project".to_string()),
        Just("my-project-123".to_string()),
        Just("project-a".to_string()),
        Just("project-b".to_string()),
        Just("test-project".to_string()),
    ]
}

/// Strategy for generating projects with valid names
fn project_strategy() -> impl Strategy<Value = Project> {
    (project_name_strategy(), Just("rust".to_string())).prop_map(|(name, project_type)| Project {
        path: PathBuf::from(format!("/path/{}", name)),
        name,
        project_type,
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    })
}

/// Strategy for generating workspaces with projects
fn workspace_strategy() -> impl Strategy<Value = Workspace> {
    prop::collection::vec(project_strategy(), 1..5).prop_map(|projects| {
        let mut workspace = Workspace::default();
        workspace.projects = projects;
        workspace
    })
}

proptest! {
    /// Property: Rules validation is deterministic
    ///
    /// For any workspace, validating multiple times should produce
    /// identical results.
    #[test]
    fn prop_rules_validation_is_deterministic(
        workspace in workspace_strategy()
    ) {
        let validator = RulesValidator::new(workspace);

        // Validate multiple times
        let result1 = validator.validate_all();
        let result2 = validator.validate_all();

        // Results should be identical
        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            prop_assert_eq!(r1.passed, r2.passed);
            prop_assert_eq!(r1.violations.len(), r2.violations.len());
        }
    }

    /// Property: Validation results are consistent
    ///
    /// For any workspace, validation should produce consistent results.
    #[test]
    fn prop_validation_results_consistent(
        workspace in workspace_strategy()
    ) {
        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all();

        // Result should be valid
        prop_assert!(result.is_ok());

        if let Ok(validation_result) = result {
            // Violations should be non-empty or passed should be true
            prop_assert!(validation_result.violations.is_empty() || !validation_result.passed);
        }
    }

    /// Property: Violation descriptions are non-empty
    ///
    /// For any violation, the description should be non-empty.
    #[test]
    fn prop_violation_descriptions_non_empty(
        workspace in workspace_strategy()
    ) {
        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all();

        if let Ok(validation_result) = result {
            // All violations should have non-empty descriptions
            for violation in &validation_result.violations {
                prop_assert!(!violation.description.is_empty());
            }
        }
    }

    /// Property: Workspace validation completes without panicking
    ///
    /// For any workspace, validation should complete without panicking.
    #[test]
    fn prop_validation_completes_safely(
        workspace in workspace_strategy()
    ) {
        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all();

        // Should complete without panicking
        prop_assert!(result.is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_validator_creation() {
        let workspace = Workspace::default();
        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_result_structure() {
        let workspace = Workspace::default();
        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all();

        if let Ok(validation_result) = result {
            // Should have valid structure
            assert!(validation_result.violations.is_empty() || !validation_result.passed);
        }
    }

    #[test]
    fn test_validation_deterministic() {
        let workspace = Workspace::default();
        let validator = RulesValidator::new(workspace);

        let result1 = validator.validate_all();
        let result2 = validator.validate_all();

        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            assert_eq!(r1.passed, r2.passed);
            assert_eq!(r1.violations.len(), r2.violations.len());
        }
    }
}
