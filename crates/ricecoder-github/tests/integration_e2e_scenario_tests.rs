//! End-to-end scenario tests for GitHub Integration
//!
//! These tests verify complete workflows from start to finish, including:
//! - PR creation and management
//! - PR review and approval
//! - PR merge workflow
//! - And other multi-step scenarios

use ricecoder_github::{
    models::{FileChange, PrStatus},
    PrManager, TaskContext,
};

/// E2E Test: Complete PR creation and merge workflow
///
/// This test simulates the complete workflow of:
/// 1. Creating a PR with implementation
/// 2. Linking to related issues
/// 3. Reviewing and approving the PR
/// 4. Merging the PR
#[test]
fn e2e_pr_creation_and_merge_workflow() {
    // Setup
    let pr_manager = PrManager::new();

    // Step 1: Create PR with implementation
    let issue_number = 42;
    let context = TaskContext::new("Fix authentication login", "Fixed authentication issue")
        .with_issue(issue_number)
        .with_file(FileChange {
            path: "src/auth/mod.rs".to_string(),
            change_type: "modified".to_string(),
            additions: 50,
            deletions: 20,
        })
        .with_file(FileChange {
            path: "tests/auth_tests.rs".to_string(),
            change_type: "added".to_string(),
            additions: 100,
            deletions: 0,
        });

    let options = ricecoder_github::PrOptions::new("fix/auth-login");
    let mut pr = pr_manager
        .create_pr_from_context(context, options)
        .expect("Should create PR");

    // Verify PR links to issue
    assert!(pr.body.contains("Closes #42"), "PR should link to issue");
    assert_eq!(pr.status, PrStatus::Open);
    assert_eq!(pr.files.len(), 2);

    // Step 2: Review code
    let review = ricecoder_github::PrReview::approval("reviewer");
    ricecoder_github::PrOperations::add_review(&mut pr, review).expect("Should add review");
    assert!(pr.body.contains("Approved"));

    // Step 3: Merge PR
    let merge_options = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr, merge_options).expect("Should merge PR");
    assert_eq!(pr.status, PrStatus::Merged);

    // Verify complete workflow
    assert_eq!(pr.title, "Fix authentication login");
    assert_eq!(pr.status, PrStatus::Merged);
    assert!(pr.body.contains("Closes #42"));
}

/// E2E Test: Complete PR review and approval workflow
///
/// This test simulates:
/// 1. Creating a PR with code changes
/// 2. Adding review comments
/// 3. Approving the PR
/// 4. Merging the PR
#[test]
fn e2e_pr_review_and_approval_workflow() {
    // Setup
    let pr_manager = PrManager::new();

    // Step 1: Create PR with code changes
    let context = TaskContext::new("Add new feature", "Implemented new feature")
        .with_file(FileChange {
            path: "src/feature.rs".to_string(),
            change_type: "added".to_string(),
            additions: 150,
            deletions: 0,
        })
        .with_file(FileChange {
            path: "tests/feature_tests.rs".to_string(),
            change_type: "added".to_string(),
            additions: 200,
            deletions: 0,
        });

    let options = ricecoder_github::PrOptions::new("feature/new-feature");
    let mut pr = pr_manager
        .create_pr_from_context(context, options)
        .expect("Should create PR");

    // Step 2: Add review comment
    let comment = ricecoder_github::PrComment::new(
        "Great implementation! Just a few suggestions for improvement.",
        "reviewer",
    );
    ricecoder_github::PrOperations::add_comment(&mut pr, comment).expect("Should add comment");

    // Step 3: Add approval review
    let review = ricecoder_github::PrReview::approval("reviewer");
    ricecoder_github::PrOperations::add_review(&mut pr, review).expect("Should add review");
    assert!(pr.body.contains("Approved"));

    // Step 4: Merge PR
    let merge_options = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr, merge_options).expect("Should merge PR");
    assert_eq!(pr.status, PrStatus::Merged);

    // Verify complete workflow
    assert_eq!(pr.title, "Add new feature");
    assert_eq!(pr.status, PrStatus::Merged);
    assert!(pr.body.contains("Approved"));
}

/// E2E Test: Multiple PR workflow
///
/// This test simulates:
/// 1. Creating multiple PRs
/// 2. Managing each PR independently
/// 3. Merging all PRs
#[test]
fn e2e_multiple_pr_workflow() {
    // Setup
    let pr_manager = PrManager::new();

    // Step 1: Create first PR
    let context1 = TaskContext::new("Feature 1", "First feature");
    let options1 = ricecoder_github::PrOptions::new("feature/1");
    let mut pr1 = pr_manager
        .create_pr_from_context(context1, options1)
        .expect("Should create PR 1");

    // Step 2: Create second PR
    let context2 = TaskContext::new("Feature 2", "Second feature");
    let options2 = ricecoder_github::PrOptions::new("feature/2");
    let mut pr2 = pr_manager
        .create_pr_from_context(context2, options2)
        .expect("Should create PR 2");

    // Step 3: Approve both PRs
    let review1 = ricecoder_github::PrReview::approval("reviewer");
    ricecoder_github::PrOperations::add_review(&mut pr1, review1)
        .expect("Should add review to PR 1");

    let review2 = ricecoder_github::PrReview::approval("reviewer");
    ricecoder_github::PrOperations::add_review(&mut pr2, review2)
        .expect("Should add review to PR 2");

    // Step 4: Merge both PRs
    let merge_options1 = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr1, merge_options1).expect("Should merge PR 1");

    let merge_options2 = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr2, merge_options2).expect("Should merge PR 2");

    // Verify complete workflow
    assert_eq!(pr1.title, "Feature 1");
    assert_eq!(pr1.status, PrStatus::Merged);
    assert_eq!(pr2.title, "Feature 2");
    assert_eq!(pr2.status, PrStatus::Merged);
}

/// E2E Test: PR with progress tracking
///
/// This test simulates:
/// 1. Creating a PR
/// 2. Adding progress updates
/// 3. Adding multiple comments
/// 4. Merging the PR
#[test]
fn e2e_pr_with_progress_tracking() {
    // Setup
    let pr_manager = PrManager::new();

    // Step 1: Create PR
    let context = TaskContext::new("Implement feature", "Implementing new feature");
    let options = ricecoder_github::PrOptions::new("feature/implementation");
    let mut pr = pr_manager
        .create_pr_from_context(context, options)
        .expect("Should create PR");

    // Step 2: Add progress update
    let progress = ricecoder_github::ProgressUpdate::new("Implementation", "In Progress")
        .with_progress(50)
        .with_description("Halfway through implementation");

    ricecoder_github::PrOperations::add_progress_update(&mut pr, progress)
        .expect("Should add progress update");

    // Step 3: Add comment
    let comment = ricecoder_github::PrComment::new("Looking good so far!", "reviewer");
    ricecoder_github::PrOperations::add_comment(&mut pr, comment).expect("Should add comment");

    // Step 4: Add another progress update
    let progress2 = ricecoder_github::ProgressUpdate::new("Implementation", "Complete")
        .with_progress(100)
        .with_description("Implementation complete, ready for review");

    ricecoder_github::PrOperations::add_progress_update(&mut pr, progress2)
        .expect("Should add second progress update");

    // Step 5: Approve and merge
    let review = ricecoder_github::PrReview::approval("reviewer");
    ricecoder_github::PrOperations::add_review(&mut pr, review).expect("Should add review");

    let merge_options = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr, merge_options).expect("Should merge PR");

    // Verify complete workflow
    assert_eq!(pr.status, PrStatus::Merged);
    assert!(pr.body.contains("50%"));
    assert!(pr.body.contains("100%"));
    assert!(pr.body.contains("Looking good so far!"));
}

/// E2E Test: PR with error recovery
///
/// This test simulates:
/// 1. Attempting invalid PR creation
/// 2. Recovering with valid PR
/// 3. Managing the PR successfully
#[test]
fn e2e_pr_with_error_recovery() {
    // Setup
    let pr_manager = PrManager::new();

    // Step 1: Try to create invalid PR
    let invalid_context = TaskContext::new("", "");
    let options1 = ricecoder_github::PrOptions::new("feature/test");
    let result = pr_manager.create_pr_from_context(invalid_context, options1);
    assert!(result.is_err(), "Should reject invalid PR");

    // Step 2: Recover with valid PR
    let valid_context = TaskContext::new("Valid title", "Valid description");
    let options2 = ricecoder_github::PrOptions::new("feature/test");
    let mut pr = pr_manager
        .create_pr_from_context(valid_context, options2)
        .expect("Should create valid PR");

    // Step 3: Manage the PR
    let review = ricecoder_github::PrReview::approval("reviewer");
    ricecoder_github::PrOperations::add_review(&mut pr, review).expect("Should add review");

    let merge_options = ricecoder_github::PrUpdateOptions::new().with_state(PrStatus::Merged);
    ricecoder_github::PrOperations::update_pr(&mut pr, merge_options).expect("Should merge PR");

    // Verify complete workflow
    assert_eq!(pr.title, "Valid title");
    assert_eq!(pr.status, PrStatus::Merged);
}
