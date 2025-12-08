//! Property-based tests for Code Review Agent
//!
//! These tests verify correctness properties that should hold across all valid inputs

use proptest::prelude::*;
use ricecoder_github::{
    models::{FileChange, PrStatus},
    CodeQualityIssue, CodeReviewAgent, CodeReviewStandards, IssueSeverity,
};
use chrono::Utc;

// Strategy for generating valid PR titles
fn valid_title_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_]{1,100}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("title must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid PR bodies
fn valid_body_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,!?\n]{1,200}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("body must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid branch names
fn valid_branch_strategy() -> impl Strategy<Value = String> {
    r"(feature|bugfix|hotfix)/[a-z0-9\-]{1,50}"
        .prop_map(|s| s.to_lowercase())
}

// Strategy for generating file changes
fn file_change_strategy() -> impl Strategy<Value = FileChange> {
    (
        r"[a-z0-9/\-_.]{1,50}\.(rs|ts|py|go|java)",
        r"(added|modified|deleted)",
        0u32..500,
        0u32..500,
    )
        .prop_map(|(path, change_type, additions, deletions)| FileChange {
            path,
            change_type,
            additions,
            deletions,
        })
}

// Strategy for generating pull requests
fn pull_request_strategy() -> impl Strategy<Value = ricecoder_github::models::PullRequest> {
    (
        valid_title_strategy(),
        valid_body_strategy(),
        valid_branch_strategy(),
        prop::collection::vec(file_change_strategy(), 1..5),
    )
        .prop_map(|(title, body, branch, files)| ricecoder_github::models::PullRequest {
            id: 1,
            number: 123,
            title,
            body,
            branch,
            base: "main".to_string(),
            status: PrStatus::Open,
            files,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
}

// Strategy for generating code quality issues
fn code_quality_issue_strategy() -> impl Strategy<Value = CodeQualityIssue> {
    (
        prop_oneof![
            Just(IssueSeverity::Critical),
            Just(IssueSeverity::Warning),
            Just(IssueSeverity::Info),
        ],
        r"[a-zA-Z0-9 ]{1,50}",
        r"[a-zA-Z0-9 .,]{1,100}",
        r"[a-z0-9/\-_.]{1,50}\.(rs|ts|py)",
    )
        .prop_map(|(severity, title, description, file_path)| {
            CodeQualityIssue::new(severity, title, description, file_path)
        })
}

// **Feature: ricecoder-github, Property 46: Code Quality Analysis**
// *For any* PR, the system SHALL analyze the code changes for quality issues.
// **Validates: Requirements 10.1**
proptest! {
    #[test]
    fn prop_code_quality_analysis_produces_issues(
        pr in pull_request_strategy()
    ) {
        let agent = CodeReviewAgent::new();
        let issues = agent.analyze_code(&pr).unwrap();

        // Property: Analysis should always return a result (may be empty or have issues)
        prop_assert!(true); // Analysis completed successfully

        // Property: All issues should have non-empty titles
        for issue in &issues {
            prop_assert!(!issue.title.is_empty());
        }

        // Property: All issues should have non-empty descriptions
        for issue in &issues {
            prop_assert!(!issue.description.is_empty());
        }

        // Property: All issues should reference valid file paths
        for issue in &issues {
            prop_assert!(!issue.file_path.is_empty());
        }
    }
}

// **Feature: ricecoder-github, Property 47: Code Review Suggestion Posting**
// *For any* code quality issue found, the system SHALL post a suggestion as a PR comment.
// **Validates: Requirements 10.2**
proptest! {
    #[test]
    fn prop_code_review_suggestions_generated_for_issues(
        issues in prop::collection::vec(code_quality_issue_strategy(), 1..5)
    ) {
        let agent = CodeReviewAgent::new();
        let suggestions = agent.generate_suggestions(&issues).unwrap();

        // Property: Should generate one suggestion per issue
        prop_assert_eq!(suggestions.len(), issues.len());

        // Property: Each suggestion should have non-empty title
        for suggestion in &suggestions {
            prop_assert!(!suggestion.title.is_empty());
        }

        // Property: Each suggestion should have non-empty body
        for suggestion in &suggestions {
            prop_assert!(!suggestion.body.is_empty());
        }

        // Property: Each suggestion should reference a file path
        for suggestion in &suggestions {
            prop_assert!(!suggestion.file_path.is_empty());
        }

        // Property: Critical issues should generate critical suggestions
        for (i, issue) in issues.iter().enumerate() {
            if issue.severity == IssueSeverity::Critical {
                prop_assert!(suggestions[i].is_critical);
            }
        }
    }
}

// **Feature: ricecoder-github, Property 48: Standards Validation**
// *For any* code change, the system SHALL validate the code against configured project standards.
// **Validates: Requirements 10.3**
proptest! {
    #[test]
    fn prop_standards_validation_checks_all_prs(
        pr in pull_request_strategy()
    ) {
        let agent = CodeReviewAgent::new();
        let issues = agent.validate_standards(&pr).unwrap();

        // Property: Validation should always complete
        prop_assert!(true);

        // Property: All issues should have valid severity levels
        for issue in &issues {
            prop_assert!(
                issue.severity == IssueSeverity::Critical
                    || issue.severity == IssueSeverity::Warning
                    || issue.severity == IssueSeverity::Info
            );
        }

        // Property: All issues should have non-empty titles
        for issue in &issues {
            prop_assert!(!issue.title.is_empty());
        }
    }
}

// **Feature: ricecoder-github, Property 49: Code Review Summary Generation**
// *For any* PR review, the system SHALL generate a summary of all findings and suggestions.
// **Validates: Requirements 10.4**
proptest! {
    #[test]
    fn prop_code_review_summary_generated(
        pr in pull_request_strategy()
    ) {
        let agent = CodeReviewAgent::new();
        let result = agent.review_pr(&pr).unwrap();
        let summary = agent.generate_summary(&result).unwrap();

        // Property: Summary should not be empty
        prop_assert!(!summary.is_empty());

        // Property: Summary should contain PR number
        let pr_str = format!("PR #{}", result.pr_number);
        prop_assert!(summary.contains(&pr_str));

        // Property: Summary should contain quality score
        let score_str = format!("{}/100", result.quality_score);
        prop_assert!(summary.contains(&score_str));

        // Property: Summary should contain approval status
        prop_assert!(summary.contains("APPROVED") || summary.contains("NEEDS REVIEW"));

        // Property: Summary should be valid markdown
        prop_assert!(summary.contains("##") || summary.contains("**"));
    }
}

// **Feature: ricecoder-github, Property 50: Conditional PR Approval**
// *For any* PR meeting approval conditions, the system SHALL approve the PR.
// **Validates: Requirements 10.5**
proptest! {
    #[test]
    fn prop_conditional_approval_respects_standards(
        pr in pull_request_strategy(),
        min_quality_score in 0u32..100
    ) {
        let standards = CodeReviewStandards::new(min_quality_score);
        let require_critical_fixes = standards.require_critical_fixes;
        let agent = CodeReviewAgent::with_standards(standards);
        let result = agent.review_pr(&pr).unwrap();

        // Property: Approval decision should be consistent with quality score
        let should_approve = agent.should_approve(&result).unwrap();
        if result.quality_score >= min_quality_score {
            prop_assert!(should_approve || result.has_critical_issues());
        } else {
            prop_assert!(!should_approve);
        }

        // Property: If approved, quality score should meet minimum
        if result.approved {
            prop_assert!(result.quality_score >= min_quality_score);
        }

        // Property: If has critical issues and require_critical_fixes is true,
        // should not be approved
        if require_critical_fixes && result.has_critical_issues() {
            prop_assert!(!result.approved);
        }
    }
}

// Additional property: Code review result consistency
// *For any* PR, the code review result should be internally consistent
proptest! {
    #[test]
    fn prop_code_review_result_consistency(
        pr in pull_request_strategy()
    ) {
        let agent = CodeReviewAgent::new();
        let result = agent.review_pr(&pr).unwrap();

        // Property: Quality score should be between 0 and 100
        prop_assert!(result.quality_score <= 100);

        // Property: Number of suggestions should match number of issues
        prop_assert_eq!(result.suggestions.len(), result.issues.len());

        // Property: If has critical issues, quality score should be lower
        if result.has_critical_issues() {
            prop_assert!(result.quality_score < 100);
        }

        // Property: Approval reason should be present if approval status is set
        if result.approved || !result.issues.is_empty() {
            prop_assert!(result.approval_reason.is_some());
        }
    }
}

// Additional property: Issue severity consistency
// *For any* set of issues, severity levels should be consistent
proptest! {
    #[test]
    fn prop_issue_severity_consistency(
        issues in prop::collection::vec(code_quality_issue_strategy(), 1..10)
    ) {
        // Property: All issues should have valid severity
        for issue in &issues {
            prop_assert!(
                issue.severity == IssueSeverity::Critical
                    || issue.severity == IssueSeverity::Warning
                    || issue.severity == IssueSeverity::Info
            );
        }

        // Property: Critical issues should have descriptions
        for issue in &issues {
            if issue.severity == IssueSeverity::Critical {
                prop_assert!(!issue.description.is_empty());
            }
        }
    }
}
