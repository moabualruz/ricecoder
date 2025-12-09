//! Property-Based Tests for Issue Manager
//!
//! Tests correctness properties for issue parsing, requirement extraction,
//! and implementation plan generation

use proptest::prelude::*;
use ricecoder_github::{IssueManager, IssueStatus, ParsedRequirement, IssueProgressUpdate};

// Strategy for generating valid issue numbers
fn issue_number_strategy() -> impl Strategy<Value = u32> {
    1u32..=999999u32
}

// Strategy for generating valid GitHub URLs
fn github_url_strategy(issue_num: u32) -> impl Strategy<Value = String> {
    Just(format!(
        "https://github.com/owner/repo/issues/{}",
        issue_num
    ))
}

// Strategy for generating hash format issue references
fn hash_format_strategy(issue_num: u32) -> impl Strategy<Value = String> {
    Just(format!("owner/repo#{}", issue_num))
}

// Strategy for generating simple requirement descriptions
fn requirement_description_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{10,100}"
}

// Strategy for generating acceptance criteria
fn acceptance_criteria_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-zA-Z0-9 ]{5,50}", 0..5)
}

// Strategy for generating priority levels
fn priority_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("HIGH".to_string()),
        Just("MEDIUM".to_string()),
        Just("LOW".to_string()),
    ]
}

// Strategy for generating parsed requirements
fn parsed_requirement_strategy() -> impl Strategy<Value = ParsedRequirement> {
    (
        requirement_description_strategy(),
        acceptance_criteria_strategy(),
        priority_strategy(),
    )
        .prop_map(|(description, criteria, priority)| ParsedRequirement {
            id: "REQ-1".to_string(),
            description,
            acceptance_criteria: criteria,
            priority,
        })
}

// ============================================================================
// Property 6: Issue Input Parsing
// ============================================================================

proptest! {
    /// **Feature: ricecoder-github, Property 6: Issue Input Parsing**
    /// **Validates: Requirements 2.1**
    ///
    /// For any valid issue number, parsing it as a string should return the same number
    #[test]
    fn prop_parse_issue_number_returns_same_number(issue_num in issue_number_strategy()) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let input = issue_num.to_string();
        let result = manager.parse_issue_input(&input);

        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), issue_num);
    }

    /// **Feature: ricecoder-github, Property 6: Issue Input Parsing**
    /// **Validates: Requirements 2.1**
    ///
    /// For any valid GitHub URL with an issue number, parsing should extract the correct number
    #[test]
    fn prop_parse_github_url_extracts_issue_number(issue_num in issue_number_strategy()) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let url = format!(
            "https://github.com/owner/repo/issues/{}",
            issue_num
        );
        let result = manager.parse_issue_input(&url);

        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), issue_num);
    }

    /// **Feature: ricecoder-github, Property 6: Issue Input Parsing**
    /// **Validates: Requirements 2.1**
    ///
    /// For any valid hash format reference, parsing should extract the correct number
    #[test]
    fn prop_parse_hash_format_extracts_issue_number(issue_num in issue_number_strategy()) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let input = format!("owner/repo#{}", issue_num);
        let result = manager.parse_issue_input(&input);

        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), issue_num);
    }

    /// **Feature: ricecoder-github, Property 6: Issue Input Parsing**
    /// **Validates: Requirements 2.1**
    ///
    /// For any invalid input format, parsing should return an error
    #[test]
    fn prop_parse_invalid_input_returns_error(invalid_input in "[^0-9#/]{5,20}") {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.parse_issue_input(&invalid_input);
        prop_assert!(result.is_err());
    }
}

// ============================================================================
// Property 7: Issue Requirement Extraction
// ============================================================================

proptest! {
    /// **Feature: ricecoder-github, Property 7: Issue Requirement Extraction**
    /// **Validates: Requirements 2.2**
    ///
    /// For any non-empty issue body, extraction should produce at least one requirement
    #[test]
    fn prop_extract_requirements_produces_at_least_one(body in "[a-zA-Z0-9 ]{1,100}") {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.extract_requirements(&body);

        prop_assert!(result.is_ok());
        let requirements = result.unwrap();
        prop_assert!(!requirements.is_empty());
    }

    /// **Feature: ricecoder-github, Property 7: Issue Requirement Extraction**
    /// **Validates: Requirements 2.2**
    ///
    /// For an empty issue body, extraction should produce no requirements
    #[test]
    fn prop_extract_requirements_empty_body_produces_empty(body in "") {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.extract_requirements(&body);

        prop_assert!(result.is_ok());
        let requirements = result.unwrap();
        prop_assert_eq!(requirements.len(), 0);
    }

    /// **Feature: ricecoder-github, Property 7: Issue Requirement Extraction**
    /// **Validates: Requirements 2.2**
    ///
    /// For any extracted requirement, the ID should be non-empty
    #[test]
    fn prop_extracted_requirement_has_non_empty_id(body in "[a-zA-Z0-9 ]{1,100}") {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.extract_requirements(&body);

        prop_assert!(result.is_ok());
        let requirements = result.unwrap();
        for req in requirements {
            prop_assert!(!req.id.is_empty());
        }
    }

    /// **Feature: ricecoder-github, Property 7: Issue Requirement Extraction**
    /// **Validates: Requirements 2.2**
    ///
    /// For any extracted requirement, the priority should be one of the valid values
    #[test]
    fn prop_extracted_requirement_has_valid_priority(body in "[a-zA-Z0-9 ]{1,100}") {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.extract_requirements(&body);

        prop_assert!(result.is_ok());
        let requirements = result.unwrap();
        for req in requirements {
            prop_assert!(
                req.priority == "HIGH" || req.priority == "MEDIUM" || req.priority == "LOW"
            );
        }
    }
}

// ============================================================================
// Property 8: Implementation Plan Generation
// ============================================================================

proptest! {
    /// **Feature: ricecoder-github, Property 8: Implementation Plan Generation**
    /// **Validates: Requirements 2.3**
    ///
    /// For any set of requirements, the generated plan should contain all requirements as tasks
    #[test]
    fn prop_plan_contains_all_requirements(
        requirements in prop::collection::vec(parsed_requirement_strategy(), 1..5)
    ) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.create_implementation_plan(123, requirements.clone());

        prop_assert!(result.is_ok());
        let plan = result.unwrap();
        prop_assert_eq!(plan.tasks.len(), requirements.len());
    }

    /// **Feature: ricecoder-github, Property 8: Implementation Plan Generation**
    /// **Validates: Requirements 2.3**
    ///
    /// For any set of requirements, the generated plan should have positive estimated effort
    #[test]
    fn prop_plan_has_positive_effort(
        requirements in prop::collection::vec(parsed_requirement_strategy(), 1..5)
    ) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.create_implementation_plan(123, requirements);

        prop_assert!(result.is_ok());
        let plan = result.unwrap();
        prop_assert!(plan.estimated_effort > 0);
    }

    /// **Feature: ricecoder-github, Property 8: Implementation Plan Generation**
    /// **Validates: Requirements 2.3**
    ///
    /// For any set of requirements, each task should have a non-empty description
    #[test]
    fn prop_plan_tasks_have_descriptions(
        requirements in prop::collection::vec(parsed_requirement_strategy(), 1..5)
    ) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.create_implementation_plan(123, requirements);

        prop_assert!(result.is_ok());
        let plan = result.unwrap();
        for task in plan.tasks {
            prop_assert!(!task.description.is_empty());
        }
    }

    /// **Feature: ricecoder-github, Property 8: Implementation Plan Generation**
    /// **Validates: Requirements 2.3**
    ///
    /// For any set of requirements, the plan ID should reference the issue number
    #[test]
    fn prop_plan_id_references_issue_number(
        issue_num in 1u32..=999999u32,
        requirements in prop::collection::vec(parsed_requirement_strategy(), 1..5)
    ) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let result = manager.create_implementation_plan(issue_num, requirements);

        prop_assert!(result.is_ok());
        let plan = result.unwrap();
        prop_assert!(plan.id.contains(&issue_num.to_string()));
    }
}

// ============================================================================
// Property 9: Issue Progress Tracking
// ============================================================================

proptest! {
    /// **Feature: ricecoder-github, Property 9: Issue Progress Tracking**
    /// **Validates: Requirements 2.4**
    ///
    /// For any progress percentage, the formatted update should contain the percentage
    #[test]
    fn prop_progress_update_contains_percentage(percentage in 0u32..=100u32) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let update = IssueProgressUpdate {
            issue_number: 123,
            message: "Working on it".to_string(),
            status: IssueStatus::InProgress,
            progress_percentage: percentage,
        };

        let formatted = manager.format_progress_update(&update);
        prop_assert!(formatted.contains(&percentage.to_string()));
    }

    /// **Feature: ricecoder-github, Property 9: Issue Progress Tracking**
    /// **Validates: Requirements 2.4**
    ///
    /// For any progress update, the formatted message should be non-empty
    #[test]
    fn prop_progress_update_is_non_empty(
        percentage in 0u32..=100u32,
        message in "[a-zA-Z0-9 ]{1,100}"
    ) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let update = IssueProgressUpdate {
            issue_number: 123,
            message,
            status: IssueStatus::InProgress,
            progress_percentage: percentage,
        };

        let formatted = manager.format_progress_update(&update);
        prop_assert!(!formatted.is_empty());
    }
}

// ============================================================================
// Property 10: Issue Closure Linking
// ============================================================================

proptest! {
    /// **Feature: ricecoder-github, Property 10: Issue Closure Linking**
    /// **Validates: Requirements 2.5**
    ///
    /// For any PR number, the closure message should contain the PR number
    #[test]
    fn prop_closure_message_contains_pr_number(pr_num in 1u32..=999999u32) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let message = manager.format_pr_closure_message(pr_num, "Fix bug");
        prop_assert!(message.contains(&pr_num.to_string()));
    }

    /// **Feature: ricecoder-github, Property 10: Issue Closure Linking**
    /// **Validates: Requirements 2.5**
    ///
    /// For any PR title, the closure message should contain the title
    #[test]
    fn prop_closure_message_contains_pr_title(
        pr_num in 1u32..=999999u32,
        title in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let message = manager.format_pr_closure_message(pr_num, &title);
        prop_assert!(message.contains(&title));
    }

    /// **Feature: ricecoder-github, Property 10: Issue Closure Linking**
    /// **Validates: Requirements 2.5**
    ///
    /// For any PR, the closure message should be non-empty
    #[test]
    fn prop_closure_message_is_non_empty(
        pr_num in 1u32..=999999u32,
        title in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let manager = IssueManager::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let message = manager.format_pr_closure_message(pr_num, &title);
        prop_assert!(!message.is_empty());
    }
}
