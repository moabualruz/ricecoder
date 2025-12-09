//! Unit tests for PR Manager and PR Operations
//!
//! These tests verify specific examples, edge cases, and error conditions

use ricecoder_github::{
    models::{FileChange, PrStatus},
    PrComment, PrManager, PrOperations, PrOptions, PrReview, PrTemplate, PrUpdateOptions,
    ProgressUpdate, TaskContext,
};

#[test]
fn test_pr_manager_create_simple_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Fix critical bug", "Fixed a critical bug in the parser");
    let options = PrOptions::new("feature/fix-parser");

    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert_eq!(pr.title, "Fix critical bug");
    assert_eq!(pr.branch, "feature/fix-parser");
    assert_eq!(pr.base, "main");
    assert_eq!(pr.status, PrStatus::Open);
    assert!(!pr.body.is_empty());
}

#[test]
fn test_pr_manager_create_draft_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("WIP: New feature", "Work in progress");
    let options = PrOptions::new("feature/new-feature").as_draft();

    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert_eq!(pr.status, PrStatus::Draft);
}

#[test]
fn test_pr_manager_with_files() {
    let manager = PrManager::new();
    let file1 = FileChange {
        path: "src/main.rs".to_string(),
        change_type: "modified".to_string(),
        additions: 10,
        deletions: 5,
    };
    let file2 = FileChange {
        path: "tests/test.rs".to_string(),
        change_type: "added".to_string(),
        additions: 20,
        deletions: 0,
    };

    let context = TaskContext::new("Add tests", "Added comprehensive tests")
        .with_file(file1)
        .with_file(file2);
    let options = PrOptions::new("feature/add-tests");

    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert_eq!(pr.files.len(), 2);
    assert_eq!(pr.files[0].path, "src/main.rs");
    assert_eq!(pr.files[1].path, "tests/test.rs");
}

#[test]
fn test_pr_manager_with_related_issues() {
    let manager = PrManager::new();
    let context = TaskContext::new("Fix issue", "Fixed the reported issue")
        .with_issue(123)
        .with_issue(456);
    let options = PrOptions::new("feature/fix");

    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(pr.body.contains("Closes #123"));
    assert!(pr.body.contains("Closes #456"));
}

#[test]
fn test_pr_manager_custom_template() {
    let template = PrTemplate {
        title_template: "[FEATURE] {{title}}".to_string(),
        body_template: "## Changes\n{{description}}\n\n## Files\n{{files_summary}}".to_string(),
    };

    let manager = PrManager::with_template(template);
    let context = TaskContext::new("Add new feature", "This adds a new feature");
    let options = PrOptions::new("feature/new");

    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(pr.title.starts_with("[FEATURE]"));
    assert!(pr.body.contains("## Changes"));
    assert!(pr.body.contains("## Files"));
}

#[test]
fn test_pr_manager_validation_empty_title() {
    let manager = PrManager::new();
    let context = TaskContext::new("", "Description");
    let options = PrOptions::new("feature/test");

    let result = manager.create_pr_from_context(context, options);
    assert!(result.is_err());
}

#[test]
fn test_pr_manager_validation_same_branch() {
    let manager = PrManager::new();
    let context = TaskContext::new("Test", "Description");
    let options = PrOptions::new("main").with_base_branch("main");

    let result = manager.create_pr_from_context(context, options);
    assert!(result.is_err());
}

#[test]
fn test_pr_manager_validation_title_too_long() {
    let manager = PrManager::new();
    let long_title = "a".repeat(300);
    let context = TaskContext::new(long_title, "Description");
    let options = PrOptions::new("feature/test");

    let result = manager.create_pr_from_context(context, options);
    assert!(result.is_err());
}

#[test]
fn test_pr_operations_update_title() {
    let manager = PrManager::new();
    let context = TaskContext::new("Old Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let update_options = PrUpdateOptions::new().with_title("New Title");
    PrOperations::update_pr(&mut pr, update_options).unwrap();

    assert_eq!(pr.title, "New Title");
}

#[test]
fn test_pr_operations_update_body() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Old Body");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let update_options = PrUpdateOptions::new().with_body("New Body");
    PrOperations::update_pr(&mut pr, update_options).unwrap();

    assert_eq!(pr.body, "New Body");
}

#[test]
fn test_pr_operations_update_state() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    assert_eq!(pr.status, PrStatus::Open);

    let update_options = PrUpdateOptions::new().with_state(PrStatus::Closed);
    PrOperations::update_pr(&mut pr, update_options).unwrap();

    assert_eq!(pr.status, PrStatus::Closed);
}

#[test]
fn test_pr_operations_add_comment() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let original_body = pr.body.clone();
    let comment = PrComment::new("This is a great PR!", "reviewer1");
    PrOperations::add_comment(&mut pr, comment).unwrap();

    assert!(pr.body.len() > original_body.len());
    assert!(pr.body.contains("This is a great PR!"));
    assert!(pr.body.contains("reviewer1"));
}

#[test]
fn test_pr_operations_add_progress_update() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let update = ProgressUpdate::new("Implementation", "In Progress")
        .with_progress(50)
        .with_description("Currently working on the core logic");

    PrOperations::add_progress_update(&mut pr, update).unwrap();

    assert!(pr.body.contains("Implementation"));
    assert!(pr.body.contains("In Progress"));
    assert!(pr.body.contains("50%"));
    assert!(pr.body.contains("Currently working on the core logic"));
}

#[test]
fn test_pr_operations_add_approval_review() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let review = PrReview::approval("reviewer1");
    PrOperations::add_review(&mut pr, review).unwrap();

    assert!(pr.body.contains("reviewer1"));
    assert!(pr.body.contains("Approved"));
}

#[test]
fn test_pr_operations_add_changes_requested_review() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let review = PrReview::changes_requested("reviewer1", "Please add more tests");
    PrOperations::add_review(&mut pr, review).unwrap();

    assert!(pr.body.contains("reviewer1"));
    assert!(pr.body.contains("Changes Requested"));
    assert!(pr.body.contains("Please add more tests"));
}

#[test]
fn test_pr_operations_validate_update_options_empty_title() {
    let options = PrUpdateOptions::new().with_title("");
    let result = PrOperations::validate_update_options(&options);
    assert!(result.is_err());
}

#[test]
fn test_pr_operations_validate_update_options_title_too_long() {
    let long_title = "a".repeat(300);
    let options = PrUpdateOptions::new().with_title(long_title);
    let result = PrOperations::validate_update_options(&options);
    assert!(result.is_err());
}

#[test]
fn test_pr_operations_validate_comment_empty_body() {
    let comment = PrComment::new("", "user1");
    let result = PrOperations::validate_comment(&comment);
    assert!(result.is_err());
}

#[test]
fn test_pr_operations_validate_review_empty_body_with_changes_requested() {
    let review = PrReview::changes_requested("reviewer1", "");
    let result = PrOperations::validate_review(&review);
    assert!(result.is_err());
}

#[test]
fn test_pr_operations_validate_review_empty_body_with_approval() {
    let review = PrReview::approval("reviewer1");
    let result = PrOperations::validate_review(&review);
    assert!(result.is_ok());
}

#[test]
fn test_pr_operations_can_approve_open_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(PrOperations::can_approve(&pr));
}

#[test]
fn test_pr_operations_can_approve_draft_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test").as_draft();
    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(PrOperations::can_approve(&pr));
}

#[test]
fn test_pr_operations_cannot_approve_merged_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let update_options = PrUpdateOptions::new().with_state(PrStatus::Merged);
    PrOperations::update_pr(&mut pr, update_options).unwrap();

    assert!(!PrOperations::can_approve(&pr));
}

#[test]
fn test_pr_operations_can_merge_open_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(PrOperations::can_merge(&pr));
}

#[test]
fn test_pr_operations_cannot_merge_draft_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test").as_draft();
    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(!PrOperations::can_merge(&pr));
}

#[test]
fn test_pr_operations_can_close_open_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(PrOperations::can_close(&pr));
}

#[test]
fn test_pr_operations_cannot_close_merged_pr() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let update_options = PrUpdateOptions::new().with_state(PrStatus::Merged);
    PrOperations::update_pr(&mut pr, update_options).unwrap();

    assert!(!PrOperations::can_close(&pr));
}

#[test]
fn test_pr_manager_link_to_issues() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    manager.link_to_issues(&mut pr, vec![123, 456]).unwrap();

    assert!(pr.body.contains("Closes #123"));
    assert!(pr.body.contains("Closes #456"));
}

#[test]
fn test_pr_manager_link_to_empty_issues() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let original_body = pr.body.clone();
    manager.link_to_issues(&mut pr, vec![]).unwrap();

    assert_eq!(pr.body, original_body);
}

#[test]
fn test_pr_manager_validate_pr_content() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(manager.validate_pr_content(&pr).is_ok());
}

#[test]
fn test_pr_manager_validate_pr_content_empty_title() {
    use ricecoder_github::models::PullRequest;

    let manager = PrManager::new();
    let pr = PullRequest {
        id: 0,
        number: 0,
        title: String::new(),
        body: "Body".to_string(),
        branch: "feature/test".to_string(),
        base: "main".to_string(),
        status: PrStatus::Open,
        files: Vec::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert!(manager.validate_pr_content(&pr).is_err());
}

#[test]
fn test_pr_manager_validate_pr_content_empty_body() {
    use ricecoder_github::models::PullRequest;

    let manager = PrManager::new();
    let pr = PullRequest {
        id: 0,
        number: 0,
        title: "Title".to_string(),
        body: String::new(),
        branch: "feature/test".to_string(),
        base: "main".to_string(),
        status: PrStatus::Open,
        files: Vec::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert!(manager.validate_pr_content(&pr).is_err());
}

#[test]
fn test_pr_operations_multiple_updates() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    // First update
    let update1 = PrUpdateOptions::new().with_title("Updated Title");
    PrOperations::update_pr(&mut pr, update1).unwrap();
    assert_eq!(pr.title, "Updated Title");

    // Second update
    let update2 = PrUpdateOptions::new().with_body("Updated Body");
    PrOperations::update_pr(&mut pr, update2).unwrap();
    assert_eq!(pr.body, "Updated Body");
    assert_eq!(pr.title, "Updated Title"); // Title should remain unchanged
}

#[test]
fn test_pr_operations_multiple_comments() {
    let manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = PrOptions::new("feature/test");
    let mut pr = manager.create_pr_from_context(context, options).unwrap();

    let comment1 = PrComment::new("First comment", "user1");
    PrOperations::add_comment(&mut pr, comment1).unwrap();

    let comment2 = PrComment::new("Second comment", "user2");
    PrOperations::add_comment(&mut pr, comment2).unwrap();

    assert!(pr.body.contains("First comment"));
    assert!(pr.body.contains("Second comment"));
    assert!(pr.body.contains("user1"));
    assert!(pr.body.contains("user2"));
}

#[test]
fn test_progress_update_format_with_metadata() {
    let update = ProgressUpdate::new("Task", "In Progress")
        .with_progress(75)
        .with_description("Almost done")
        .with_metadata("estimated_time", "2 hours")
        .with_metadata("blockers", "None");

    let comment = update.format_as_comment();

    assert!(comment.contains("Task"));
    assert!(comment.contains("In Progress"));
    assert!(comment.contains("75%"));
    assert!(comment.contains("Almost done"));
    assert!(comment.contains("estimated_time"));
    assert!(comment.contains("2 hours"));
    assert!(comment.contains("blockers"));
    assert!(comment.contains("None"));
}

#[test]
fn test_pr_options_builder_pattern() {
    let options = PrOptions::new("feature/test")
        .with_base_branch("develop")
        .as_draft();

    assert_eq!(options.branch, "feature/test");
    assert_eq!(options.base_branch, "develop");
    assert!(options.draft);
}

#[test]
fn test_task_context_builder_pattern() {
    let context = TaskContext::new("Title", "Description")
        .with_issue(123)
        .with_issue(456)
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

    assert_eq!(context.related_issues.len(), 2);
    assert_eq!(context.metadata.len(), 2);
    assert_eq!(context.metadata.get("key1"), Some(&"value1".to_string()));
}
