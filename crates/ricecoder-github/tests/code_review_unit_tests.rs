//! Unit tests for Code Review Agent and Operations

use std::collections::HashMap;

use chrono::Utc;
use ricecoder_github::{
    models::{FileChange, PrStatus},
    ApprovalCondition, CodeQualityIssue, CodeReviewAgent, CodeReviewMetrics, CodeReviewOperations,
    CodeReviewStandards, CodeReviewSuggestion, ConditionalApprovalResult, IssueSeverity, PrReview,
    ReviewState,
};

fn create_test_pr() -> ricecoder_github::models::PullRequest {
    ricecoder_github::models::PullRequest {
        id: 1,
        number: 123,
        title: "Test PR".to_string(),
        body: "This is a test PR".to_string(),
        branch: "feature/test".to_string(),
        base: "main".to_string(),
        status: PrStatus::Open,
        files: vec![FileChange {
            path: "src/main.rs".to_string(),
            change_type: "modified".to_string(),
            additions: 50,
            deletions: 10,
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// Tests for CodeQualityIssue
#[test]
fn test_code_quality_issue_creation() {
    let issue = CodeQualityIssue::new(
        IssueSeverity::Warning,
        "Test issue",
        "This is a test",
        "test.rs",
    );
    assert_eq!(issue.severity, IssueSeverity::Warning);
    assert_eq!(issue.title, "Test issue");
    assert_eq!(issue.file_path, "test.rs");
    assert_eq!(issue.line_number, None);
    assert_eq!(issue.suggested_fix, None);
}

#[test]
fn test_code_quality_issue_with_line_number() {
    let issue = CodeQualityIssue::new(IssueSeverity::Critical, "Test", "Description", "test.rs")
        .with_line_number(42);
    assert_eq!(issue.line_number, Some(42));
}

#[test]
fn test_code_quality_issue_with_suggested_fix() {
    let issue = CodeQualityIssue::new(IssueSeverity::Warning, "Test", "Description", "test.rs")
        .with_suggested_fix("Use better variable name");
    assert_eq!(
        issue.suggested_fix,
        Some("Use better variable name".to_string())
    );
}

// Tests for CodeReviewSuggestion
#[test]
fn test_code_review_suggestion_creation() {
    let suggestion = CodeReviewSuggestion::new("Test", "Body", "test.rs");
    assert_eq!(suggestion.title, "Test");
    assert_eq!(suggestion.body, "Body");
    assert_eq!(suggestion.file_path, "test.rs");
    assert!(!suggestion.is_critical);
}

#[test]
fn test_code_review_suggestion_as_critical() {
    let suggestion = CodeReviewSuggestion::new("Test", "Body", "test.rs").as_critical();
    assert!(suggestion.is_critical);
}

#[test]
fn test_code_review_suggestion_with_line_number() {
    let suggestion = CodeReviewSuggestion::new("Test", "Body", "test.rs").with_line_number(10);
    assert_eq!(suggestion.line_number, Some(10));
}

// Tests for CodeReviewResult
#[test]
fn test_code_review_result_creation() {
    let result = ricecoder_github::CodeReviewResult::new(123);
    assert_eq!(result.pr_number, 123);
    assert_eq!(result.quality_score, 100);
    assert!(result.approved);
    assert!(result.issues.is_empty());
    assert!(result.suggestions.is_empty());
}

#[test]
fn test_code_review_result_with_issue() {
    let issue = CodeQualityIssue::new(IssueSeverity::Critical, "Test", "Description", "test.rs");
    let result = ricecoder_github::CodeReviewResult::new(123).with_issue(issue);
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.critical_issues_count(), 1);
    assert!(result.quality_score < 100);
}

#[test]
fn test_code_review_result_quality_score_calculation() {
    let critical = CodeQualityIssue::new(
        IssueSeverity::Critical,
        "Critical",
        "Description",
        "test.rs",
    );
    let warning =
        CodeQualityIssue::new(IssueSeverity::Warning, "Warning", "Description", "test.rs");
    let result = ricecoder_github::CodeReviewResult::new(123)
        .with_issue(critical)
        .with_issue(warning);
    assert_eq!(result.quality_score, 70); // 100 - 20 - 10
}

#[test]
fn test_code_review_result_has_critical_issues() {
    let critical = CodeQualityIssue::new(
        IssueSeverity::Critical,
        "Critical",
        "Description",
        "test.rs",
    );
    let result = ricecoder_github::CodeReviewResult::new(123).with_issue(critical);
    assert!(result.has_critical_issues());
}

#[test]
fn test_code_review_result_counts() {
    let critical = CodeQualityIssue::new(
        IssueSeverity::Critical,
        "Critical",
        "Description",
        "test.rs",
    );
    let warning =
        CodeQualityIssue::new(IssueSeverity::Warning, "Warning", "Description", "test.rs");
    let info = CodeQualityIssue::new(IssueSeverity::Info, "Info", "Description", "test.rs");
    let result = ricecoder_github::CodeReviewResult::new(123)
        .with_issue(critical)
        .with_issue(warning)
        .with_issue(info);
    assert_eq!(result.critical_issues_count(), 1);
    assert_eq!(result.warnings_count(), 1);
    assert_eq!(result.info_count(), 1);
}

// Tests for CodeReviewStandards
#[test]
fn test_code_review_standards_default() {
    let standards = CodeReviewStandards::default();
    assert_eq!(standards.min_quality_score, 70);
    assert!(standards.require_critical_fixes);
    assert!(!standards.require_warning_fixes);
}

#[test]
fn test_code_review_standards_custom() {
    let standards = CodeReviewStandards::new(80)
        .require_critical_fixes(false)
        .require_warning_fixes(true);
    assert_eq!(standards.min_quality_score, 80);
    assert!(!standards.require_critical_fixes);
    assert!(standards.require_warning_fixes);
}

#[test]
fn test_code_review_standards_with_rule() {
    let standards = CodeReviewStandards::new(70).with_rule("naming", "use_snake_case");
    assert!(standards.custom_rules.contains_key("naming"));
}

// Tests for CodeReviewAgent
#[test]
fn test_code_review_agent_creation() {
    let agent = CodeReviewAgent::new();
    assert_eq!(agent.standards.min_quality_score, 70);
}

#[test]
fn test_code_review_agent_with_standards() {
    let standards = CodeReviewStandards::new(80);
    let agent = CodeReviewAgent::with_standards(standards);
    assert_eq!(agent.standards.min_quality_score, 80);
}

#[test]
fn test_analyze_code_empty_files() {
    let agent = CodeReviewAgent::new();
    let mut pr = create_test_pr();
    pr.files.clear();
    let issues = agent.analyze_code(&pr).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_analyze_code_large_file() {
    let agent = CodeReviewAgent::new();
    let mut pr = create_test_pr();
    pr.files[0].additions = 300;
    pr.files[0].deletions = 300;
    let issues = agent.analyze_code(&pr).unwrap();
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.title.contains("Large file")));
}

#[test]
fn test_analyze_code_empty_body() {
    let agent = CodeReviewAgent::new();
    let mut pr = create_test_pr();
    pr.body.clear();
    let issues = agent.analyze_code(&pr).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.title.contains("Missing PR description")));
}

#[test]
fn test_validate_standards_no_files() {
    let agent = CodeReviewAgent::new();
    let mut pr = create_test_pr();
    pr.files.clear();
    let issues = agent.validate_standards(&pr).unwrap();
    assert!(issues.iter().any(|i| i.severity == IssueSeverity::Critical));
}

#[test]
fn test_validate_standards_invalid_branch() {
    let agent = CodeReviewAgent::new();
    let mut pr = create_test_pr();
    pr.branch = "invalid-branch".to_string();
    let issues = agent.validate_standards(&pr).unwrap();
    assert!(issues.iter().any(|i| i.title.contains("Invalid branch")));
}

#[test]
fn test_is_valid_branch_name() {
    let agent = CodeReviewAgent::new();
    assert!(agent.is_valid_branch_name("feature/test"));
    assert!(agent.is_valid_branch_name("bugfix/test"));
    assert!(agent.is_valid_branch_name("hotfix/test"));
    assert!(agent.is_valid_branch_name("main"));
    assert!(!agent.is_valid_branch_name("invalid"));
}

#[test]
fn test_generate_suggestions() {
    let agent = CodeReviewAgent::new();
    let issues = vec![CodeQualityIssue::new(
        IssueSeverity::Warning,
        "Test",
        "Description",
        "test.rs",
    )];
    let suggestions = agent.generate_suggestions(&issues).unwrap();
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].title, "Test");
}

#[test]
fn test_generate_summary() {
    let agent = CodeReviewAgent::new();
    let result = ricecoder_github::CodeReviewResult::new(123);
    let summary = agent.generate_summary(&result).unwrap();
    assert!(summary.contains("Code Review Summary"));
    assert!(summary.contains("PR #123"));
    assert!(summary.contains("APPROVED"));
}

#[test]
fn test_should_approve_high_quality() {
    let agent = CodeReviewAgent::new();
    let result = ricecoder_github::CodeReviewResult::new(123).set_approved(true, None);
    assert!(agent.should_approve(&result).unwrap());
}

#[test]
fn test_should_approve_low_quality() {
    let agent = CodeReviewAgent::new();
    let mut result = ricecoder_github::CodeReviewResult::new(123);
    result.quality_score = 50;
    assert!(!agent.should_approve(&result).unwrap());
}

#[test]
fn test_should_approve_with_critical_issues() {
    let standards = CodeReviewStandards::default().require_critical_fixes(true);
    let agent = CodeReviewAgent::with_standards(standards);
    let issue = CodeQualityIssue::new(
        IssueSeverity::Critical,
        "Critical",
        "Description",
        "test.rs",
    );
    let result = ricecoder_github::CodeReviewResult::new(123).with_issue(issue);
    assert!(!agent.should_approve(&result).unwrap());
}

#[test]
fn test_review_pr_complete() {
    let agent = CodeReviewAgent::new();
    let pr = create_test_pr();
    let result = agent.review_pr(&pr).unwrap();
    assert_eq!(result.pr_number, 123);
    assert!(result.quality_score > 0);
}

#[test]
fn test_review_pr_with_issues() {
    let agent = CodeReviewAgent::new();
    let mut pr = create_test_pr();
    pr.body.clear();
    pr.branch = "invalid".to_string();
    let result = agent.review_pr(&pr).unwrap();
    assert!(!result.issues.is_empty());
}

// Tests for CodeReviewOperations
#[test]
fn test_code_review_operations_creation() {
    let _ops = CodeReviewOperations::new();
}

#[test]
fn test_post_suggestion_comment() {
    let ops = CodeReviewOperations::new();
    let comment = ops
        .post_suggestion_comment(123, "src/main.rs", Some(42), "Use better variable name")
        .unwrap();
    assert!(comment.contains("Code Review Suggestion"));
    assert!(comment.contains("src/main.rs"));
    assert!(comment.contains("42"));
}

#[test]
fn test_generate_summary_report() {
    let ops = CodeReviewOperations::new();
    let report = ops.generate_summary_report(123, 85, 2, 3, true).unwrap();
    assert!(report.contains("PR #123"));
    assert!(report.contains("85/100"));
    assert!(report.contains("APPROVED"));
}

#[test]
fn test_track_metrics() {
    let ops = CodeReviewOperations::new();
    let mut metrics = CodeReviewMetrics::new();
    ops.track_metrics(&mut metrics, true, 85, 30).unwrap();
    assert_eq!(metrics.total_prs_reviewed, 1);
}

#[test]
fn test_evaluate_conditional_approval() {
    let ops = CodeReviewOperations::new();
    let mut conditions = HashMap::new();
    conditions.insert("Quality".to_string(), true);
    conditions.insert("Tests".to_string(), true);
    let result = ops.evaluate_conditional_approval(123, conditions).unwrap();
    assert!(result.approved);
}

#[test]
fn test_evaluate_conditional_approval_not_met() {
    let ops = CodeReviewOperations::new();
    let mut conditions = HashMap::new();
    conditions.insert("Quality".to_string(), true);
    conditions.insert("Tests".to_string(), false);
    let result = ops.evaluate_conditional_approval(123, conditions).unwrap();
    assert!(!result.approved);
}

#[test]
fn test_generate_approval_checklist() {
    let ops = CodeReviewOperations::new();
    let conditions = vec![
        ApprovalCondition::new("Quality", "Quality check", true),
        ApprovalCondition::new("Tests", "Test coverage", false),
    ];
    let checklist = ops.generate_approval_checklist(123, &conditions).unwrap();
    assert!(checklist.contains("PR #123"));
    assert!(checklist.contains("✅"));
    assert!(checklist.contains("❌"));
}

// Tests for PrReview
#[test]
fn test_pr_review_creation() {
    let review = PrReview::new("reviewer", ReviewState::Approved, "Looks good!");
    assert_eq!(review.reviewer, "reviewer");
    assert_eq!(review.state, ReviewState::Approved);
    assert_eq!(review.body, "Looks good!");
}

#[test]
fn test_pr_review_approval() {
    let review = PrReview::approval("reviewer");
    assert_eq!(review.reviewer, "reviewer");
    assert_eq!(review.state, ReviewState::Approved);
}

#[test]
fn test_pr_review_changes_requested() {
    let review = PrReview::changes_requested("reviewer", "Please fix this");
    assert_eq!(review.reviewer, "reviewer");
    assert_eq!(review.state, ReviewState::ChangesRequested);
    assert_eq!(review.body, "Please fix this");
}

// Tests for CodeReviewMetrics
#[test]
fn test_code_review_metrics_default() {
    let metrics = CodeReviewMetrics::default();
    assert_eq!(metrics.total_prs_reviewed, 0);
    assert_eq!(metrics.prs_approved, 0);
}

#[test]
fn test_code_review_metrics_update() {
    let mut metrics = CodeReviewMetrics::new();
    metrics.update_with_review(true, 85, 30);
    assert_eq!(metrics.total_prs_reviewed, 1);
    assert_eq!(metrics.prs_approved, 1);
    assert_eq!(metrics.average_quality_score, 85.0);
}

#[test]
fn test_code_review_metrics_approval_rate() {
    let mut metrics = CodeReviewMetrics::new();
    metrics.update_with_review(true, 85, 30);
    metrics.update_with_review(false, 60, 45);
    assert_eq!(metrics.approval_rate(), 50.0);
}

// Tests for ApprovalCondition
#[test]
fn test_approval_condition_creation() {
    let condition = ApprovalCondition::new("Test", "Test condition", true);
    assert_eq!(condition.name, "Test");
    assert!(condition.is_met);
}

// Tests for ConditionalApprovalResult
#[test]
fn test_conditional_approval_result_creation() {
    let result = ConditionalApprovalResult::new(123);
    assert_eq!(result.pr_number, 123);
    assert!(result.conditions.is_empty());
}

#[test]
fn test_conditional_approval_result_with_conditions() {
    let condition1 = ApprovalCondition::new("Test1", "Description1", true);
    let condition2 = ApprovalCondition::new("Test2", "Description2", false);
    let result = ConditionalApprovalResult::new(123)
        .with_condition(condition1)
        .with_condition(condition2);
    assert_eq!(result.conditions.len(), 2);
}

#[test]
fn test_conditional_approval_result_all_conditions_met() {
    let condition = ApprovalCondition::new("Test", "Description", true);
    let result = ConditionalApprovalResult::new(123).with_condition(condition);
    assert!(result.all_conditions_met());
}

#[test]
fn test_conditional_approval_result_unmet_conditions() {
    let condition1 = ApprovalCondition::new("Test1", "Description1", true);
    let condition2 = ApprovalCondition::new("Test2", "Description2", false);
    let result = ConditionalApprovalResult::new(123)
        .with_condition(condition1)
        .with_condition(condition2);
    assert_eq!(result.unmet_conditions().len(), 1);
}
