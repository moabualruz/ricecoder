//! Integration tests for GitHub API integration with real API
//!
//! These tests verify that the GitHub Integration module correctly interacts with
//! the GitHub API and that all manager interactions work together properly.
//!
//! Note: These tests use mock/test data and do not require actual GitHub API access.
//! For real API testing, set GITHUB_TOKEN environment variable.

use ricecoder_github::{
    models::{FileChange, PrStatus},
    PrManager, TaskContext,
};

/// Test that PrManager can be initialized
#[test]
fn test_pr_manager_initialization() {
    let manager = PrManager::new();
    // Manager should be created successfully
    let context = TaskContext::new("Test", "Test description");
    let options = ricecoder_github::PrOptions::new("feature/test");
    let result = manager.create_pr_from_context(context, options);
    assert!(result.is_ok());
}

/// Test that PrManager can create and manage PRs
#[test]
fn test_pr_manager_create_and_manage() {
    let pr_manager = PrManager::new();

    // Create a PR
    let context = TaskContext::new("Fix bug", "Fixed a critical bug");
    let options = ricecoder_github::PrOptions::new("feature/fix-bug");
    let pr = pr_manager.create_pr_from_context(context, options).unwrap();

    // Verify PR was created
    assert_eq!(pr.title, "Fix bug");
    assert_eq!(pr.status, PrStatus::Open);
    assert_eq!(pr.branch, "feature/fix-bug");
}

/// Test PR creation with issue linking
#[test]
fn test_pr_with_issue_linking() {
    let pr_manager = PrManager::new();

    // Create PR with issue reference
    let context = TaskContext::new("Fix parser", "Implemented parser fix").with_issue(123);
    let options = ricecoder_github::PrOptions::new("feature/fix-parser");
    let mut pr = pr_manager.create_pr_from_context(context, options).unwrap();

    // Verify PR links to issue
    assert!(pr.body.contains("Closes #123"));

    // Simulate PR merge
    let update_options = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr, update_options).unwrap();
    assert_eq!(pr.status, PrStatus::Merged);
}

/// Test PR creation with multiple files
#[test]
fn test_pr_with_multiple_files() {
    let pr_manager = PrManager::new();

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
    let options = ricecoder_github::PrOptions::new("feature/add-tests");

    let pr = pr_manager.create_pr_from_context(context, options).unwrap();

    assert_eq!(pr.files.len(), 2);
    assert_eq!(pr.files[0].path, "src/main.rs");
    assert_eq!(pr.files[1].path, "tests/test.rs");
}

/// Test PR creation with draft status
#[test]
fn test_pr_draft_creation() {
    let pr_manager = PrManager::new();

    let context = TaskContext::new("WIP: New feature", "Work in progress");
    let options = ricecoder_github::PrOptions::new("feature/new-feature").as_draft();

    let pr = pr_manager.create_pr_from_context(context, options).unwrap();

    assert_eq!(pr.status, PrStatus::Draft);
}

/// Test PR review and approval
#[test]
fn test_pr_review_and_approval() {
    let pr_manager = PrManager::new();

    let context = TaskContext::new("Add tests", "Added comprehensive tests");
    let options = ricecoder_github::PrOptions::new("feature/tests");
    let mut pr = pr_manager.create_pr_from_context(context, options).unwrap();

    // Add approval review
    let review = ricecoder_github::PrReview::approval("reviewer");
    ricecoder_github::PrOperations::add_review(&mut pr, review).unwrap();
    assert!(pr.body.contains("Approved"));
}

/// Test PR with custom template
#[test]
fn test_pr_with_custom_template() {
    let template = ricecoder_github::PrTemplate {
        title_template: "[FEATURE] {{title}}".to_string(),
        body_template: "## Changes\n{{description}}\n\n## Files\n{{files_summary}}".to_string(),
    };

    let manager = PrManager::with_template(template);
    let context = TaskContext::new("Add new feature", "This adds a new feature");
    let options = ricecoder_github::PrOptions::new("feature/new");

    let pr = manager.create_pr_from_context(context, options).unwrap();

    assert!(pr.title.starts_with("[FEATURE]"));
    assert!(pr.body.contains("## Changes"));
    assert!(pr.body.contains("## Files"));
}

/// Test PR validation
#[test]
fn test_pr_validation_empty_title() {
    let pr_manager = PrManager::new();
    let context = TaskContext::new("", "Description");
    let options = ricecoder_github::PrOptions::new("feature/test");

    let result = pr_manager.create_pr_from_context(context, options);
    assert!(result.is_err());
}

/// Test PR update operations
#[test]
fn test_pr_update_operations() {
    let pr_manager = PrManager::new();
    let context = TaskContext::new("Old Title", "Description");
    let options = ricecoder_github::PrOptions::new("feature/test");
    let mut pr = pr_manager.create_pr_from_context(context, options).unwrap();

    let update_options = ricecoder_github::PrUpdateOptions::new().with_title("New Title");
    ricecoder_github::PrOperations::update_pr(&mut pr, update_options).unwrap();

    assert_eq!(pr.title, "New Title");
}

/// Test PR comment operations
#[test]
fn test_pr_comment_operations() {
    let pr_manager = PrManager::new();
    let context = TaskContext::new("Title", "Description");
    let options = ricecoder_github::PrOptions::new("feature/test");
    let mut pr = pr_manager.create_pr_from_context(context, options).unwrap();

    let comment = ricecoder_github::PrComment::new("This is a great PR!", "reviewer1");
    ricecoder_github::PrOperations::add_comment(&mut pr, comment).unwrap();

    assert!(pr.body.contains("This is a great PR!"));
    assert!(pr.body.contains("reviewer1"));
}

/// Test error recovery in PR operations
#[test]
fn test_error_recovery_in_pr_operations() {
    let pr_manager = PrManager::new();

    // Test invalid PR creation
    let invalid_context = TaskContext::new("", "");
    let options1 = ricecoder_github::PrOptions::new("feature/test");
    let result = pr_manager.create_pr_from_context(invalid_context, options1);
    assert!(result.is_err()); // Should reject invalid input

    // Test recovery with valid data
    let valid_context = TaskContext::new("Valid title", "Valid description");
    let options2 = ricecoder_github::PrOptions::new("feature/test");
    let result = pr_manager.create_pr_from_context(valid_context, options2);
    assert!(result.is_ok()); // Should succeed with valid input
}

/// Test multiple PR operations
#[test]
fn test_multiple_pr_operations() {
    let pr_manager = PrManager::new();

    // Create multiple PRs
    let context1 = TaskContext::new("Feature 1", "First feature");
    let options1 = ricecoder_github::PrOptions::new("feature/1");
    let pr1 = pr_manager
        .create_pr_from_context(context1, options1)
        .unwrap();

    let context2 = TaskContext::new("Feature 2", "Second feature");
    let options2 = ricecoder_github::PrOptions::new("feature/2");
    let pr2 = pr_manager
        .create_pr_from_context(context2, options2)
        .unwrap();

    // Verify both PRs were created
    assert_eq!(pr1.title, "Feature 1");
    assert_eq!(pr2.title, "Feature 2");
    assert_ne!(pr1.branch, pr2.branch);
}

/// Test PR state consistency
#[test]
fn test_pr_state_consistency() {
    let pr_manager = PrManager::new();

    // Create PR
    let context = TaskContext::new("Test", "Description");
    let options = ricecoder_github::PrOptions::new("feature/test");
    let mut pr = pr_manager.create_pr_from_context(context, options).unwrap();

    // Update PR multiple times
    let update1 = ricecoder_github::PrUpdateOptions::new().with_title("Updated 1");
    ricecoder_github::PrOperations::update_pr(&mut pr, update1).unwrap();

    let update2 = ricecoder_github::PrUpdateOptions::new().with_body("Updated body");
    ricecoder_github::PrOperations::update_pr(&mut pr, update2).unwrap();

    // Verify state is consistent
    assert_eq!(pr.title, "Updated 1");
    assert_eq!(pr.body, "Updated body");
    assert_eq!(pr.branch, "feature/test");
    assert_eq!(pr.status, PrStatus::Open);
}

/// Test PR merge workflow
#[test]
fn test_pr_merge_workflow() {
    let pr_manager = PrManager::new();

    let context = TaskContext::new("Fix bug", "Fixed a critical bug");
    let options = ricecoder_github::PrOptions::new("feature/fix");
    let mut pr = pr_manager.create_pr_from_context(context, options).unwrap();

    // Verify initial state
    assert_eq!(pr.status, PrStatus::Open);

    // Merge PR
    let merge_options = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr, merge_options).unwrap();

    // Verify merged state
    assert_eq!(pr.status, PrStatus::Merged);
}

/// Test PR with progress updates
#[test]
fn test_pr_with_progress_updates() {
    let pr_manager = PrManager::new();

    let context = TaskContext::new("Title", "Description");
    let options = ricecoder_github::PrOptions::new("feature/test");
    let mut pr = pr_manager.create_pr_from_context(context, options).unwrap();

    let update = ricecoder_github::ProgressUpdate::new("Implementation", "In Progress")
        .with_progress(50)
        .with_description("Currently working on the core logic");

    ricecoder_github::PrOperations::add_progress_update(&mut pr, update).unwrap();

    assert!(pr.body.contains("Implementation"));
    assert!(pr.body.contains("In Progress"));
    assert!(pr.body.contains("50%"));
}
