//! Unit Tests for Issue Manager and Issue Operations
//!
//! Tests specific examples and edge cases for issue parsing, tracking, and updates

use ricecoder_github::{
    IssueManager, IssueOperations, IssueStatus, Issue, IssueProgressUpdate,
};

// ============================================================================
// IssueManager Unit Tests
// ============================================================================

#[test]
fn test_parse_issue_input_with_plain_number() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    assert_eq!(manager.parse_issue_input("123").unwrap(), 123);
    assert_eq!(manager.parse_issue_input("999999").unwrap(), 999999);
    assert_eq!(manager.parse_issue_input("1").unwrap(), 1);
}

#[test]
fn test_parse_issue_input_with_github_url() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let url = "https://github.com/owner/repo/issues/456";
    assert_eq!(manager.parse_issue_input(url).unwrap(), 456);
}

#[test]
fn test_parse_issue_input_with_hash_format() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let input = "owner/repo#789";
    assert_eq!(manager.parse_issue_input(input).unwrap(), 789);
}

#[test]
fn test_parse_issue_input_invalid_format() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    assert!(manager.parse_issue_input("invalid").is_err());
    assert!(manager.parse_issue_input("abc123").is_err());
    assert!(manager.parse_issue_input("").is_err());
}

#[test]
fn test_extract_requirements_from_simple_body() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let body = "Implement user authentication";
    let requirements = manager.extract_requirements(body).unwrap();

    assert_eq!(requirements.len(), 1);
    assert_eq!(requirements[0].description, "Implement user authentication");
    assert_eq!(requirements[0].priority, "MEDIUM");
}

#[test]
fn test_extract_requirements_from_empty_body() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let body = "";
    let requirements = manager.extract_requirements(body).unwrap();

    assert_eq!(requirements.len(), 0);
}

#[test]
fn test_create_implementation_plan_single_requirement() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let requirements = vec![ricecoder_github::ParsedRequirement {
        id: "REQ-1".to_string(),
        description: "Implement feature X".to_string(),
        acceptance_criteria: vec!["Criterion 1".to_string()],
        priority: "HIGH".to_string(),
    }];

    let plan = manager.create_implementation_plan(123, requirements).unwrap();

    assert_eq!(plan.issue_number, 123);
    assert_eq!(plan.tasks.len(), 1);
    assert!(plan.estimated_effort > 0);
    assert!(plan.id.contains("123"));
}

#[test]
fn test_create_implementation_plan_multiple_requirements() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let requirements = vec![
        ricecoder_github::ParsedRequirement {
            id: "REQ-1".to_string(),
            description: "Implement feature X".to_string(),
            acceptance_criteria: vec![],
            priority: "HIGH".to_string(),
        },
        ricecoder_github::ParsedRequirement {
            id: "REQ-2".to_string(),
            description: "Add tests".to_string(),
            acceptance_criteria: vec![],
            priority: "MEDIUM".to_string(),
        },
        ricecoder_github::ParsedRequirement {
            id: "REQ-3".to_string(),
            description: "Documentation".to_string(),
            acceptance_criteria: vec![],
            priority: "LOW".to_string(),
        },
    ];

    let plan = manager.create_implementation_plan(456, requirements).unwrap();

    assert_eq!(plan.tasks.len(), 3);
    assert!(plan.estimated_effort > 0);
    // HIGH (8) + MEDIUM (5) + LOW (3) = 16
    assert_eq!(plan.estimated_effort, 16);
}

#[test]
fn test_format_progress_update_with_percentage() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let update = IssueProgressUpdate {
        issue_number: 123,
        message: "Working on implementation".to_string(),
        status: IssueStatus::InProgress,
        progress_percentage: 50,
    };

    let formatted = manager.format_progress_update(&update);

    assert!(formatted.contains("Progress Update"));
    assert!(formatted.contains("50%"));
    assert!(formatted.contains("Working on implementation"));
}

#[test]
fn test_format_progress_update_zero_percent() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let update = IssueProgressUpdate {
        issue_number: 123,
        message: "Just started".to_string(),
        status: IssueStatus::Open,
        progress_percentage: 0,
    };

    let formatted = manager.format_progress_update(&update);

    assert!(formatted.contains("0%"));
}

#[test]
fn test_format_progress_update_complete() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let update = IssueProgressUpdate {
        issue_number: 123,
        message: "All done!".to_string(),
        status: IssueStatus::Closed,
        progress_percentage: 100,
    };

    let formatted = manager.format_progress_update(&update);

    assert!(formatted.contains("100%"));
}

#[test]
fn test_format_pr_closure_message() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let message = manager.format_pr_closure_message(42, "Fix authentication bug");

    assert!(message.contains("#42"));
    assert!(message.contains("Fix authentication bug"));
    assert!(message.contains("Closes"));
}

#[test]
fn test_validate_issue_valid() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let issue = Issue {
        id: 1,
        number: 123,
        title: "Test Issue".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert!(manager.validate_issue(&issue).is_ok());
}

#[test]
fn test_validate_issue_empty_title() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let issue = Issue {
        id: 1,
        number: 123,
        title: "".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert!(manager.validate_issue(&issue).is_err());
}

#[test]
fn test_validate_issue_empty_body() {
    let manager = IssueManager::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let issue = Issue {
        id: 1,
        number: 123,
        title: "Test Issue".to_string(),
        body: "".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert!(manager.validate_issue(&issue).is_err());
}

// ============================================================================
// IssueOperations Unit Tests
// ============================================================================

#[test]
fn test_create_comment() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let comment = ops.create_comment("Test comment".to_string());

    assert_eq!(comment.body, "Test comment");
    assert_eq!(comment.id, None);
}

#[test]
fn test_format_progress_comment() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let comment = ops.format_progress_comment("Step 1", 5, 2, "Working on implementation");

    assert!(comment.body.contains("Progress Update"));
    assert!(comment.body.contains("Step 1"));
    assert!(comment.body.contains("2/5"));
    assert!(comment.body.contains("40%"));
}

#[test]
fn test_format_status_change_comment() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let comment = ops.format_status_change_comment(IssueStatus::Open, IssueStatus::InProgress);

    assert!(comment.body.contains("Status Changed"));
    assert!(comment.body.contains("InProgress"));
}

#[test]
fn test_format_pr_link_comment() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let pr_link = ricecoder_github::PrLink {
        pr_number: 42,
        pr_title: "Implement feature".to_string(),
        link_type: "closes".to_string(),
    };

    let comment = ops.format_pr_link_comment(&pr_link);

    assert!(comment.body.contains("PR Linked"));
    assert!(comment.body.contains("#42"));
    assert!(comment.body.contains("Implement feature"));
}

#[test]
fn test_create_pr_closure_link() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let link = ops.create_pr_closure_link(42, "Implement feature".to_string());

    assert_eq!(link.pr_number, 42);
    assert_eq!(link.pr_title, "Implement feature");
    assert_eq!(link.link_type, "closes");
}

#[test]
fn test_create_pr_relation_link() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let link = ops.create_pr_relation_link(42, "Related work".to_string());

    assert_eq!(link.pr_number, 42);
    assert_eq!(link.link_type, "relates to");
}

#[test]
fn test_format_closure_message() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let message = ops.format_closure_message(123);

    assert!(message.contains("Closes #123"));
}

#[test]
fn test_validate_comment_valid() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let comment = ricecoder_github::IssueComment {
        body: "Valid comment".to_string(),
        id: None,
    };

    assert!(ops.validate_comment(&comment).is_ok());
}

#[test]
fn test_validate_comment_empty() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let comment = ricecoder_github::IssueComment {
        body: "".to_string(),
        id: None,
    };

    assert!(ops.validate_comment(&comment).is_err());
}

#[test]
fn test_extract_issue_number_from_closure() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let message = "Closes #123";
    assert_eq!(ops.extract_issue_number_from_closure(message).unwrap(), 123);
}

#[test]
fn test_extract_issue_number_case_insensitive() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let message = "closes #456";
    assert_eq!(ops.extract_issue_number_from_closure(message).unwrap(), 456);
}

#[test]
fn test_extract_issue_number_invalid() {
    let ops = IssueOperations::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let message = "No issue here";
    assert!(ops.extract_issue_number_from_closure(message).is_err());
}
