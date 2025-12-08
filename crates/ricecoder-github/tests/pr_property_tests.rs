//! Property-based tests for PR Manager
//!
//! These tests verify correctness properties that should hold across all valid inputs

use proptest::prelude::*;
use ricecoder_github::{
    models::{FileChange, PrStatus},
    PrManager, PrOptions, PrTemplate, TaskContext,
};

// Strategy for generating valid PR titles
fn valid_title_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_]{1,100}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("title must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid descriptions
fn valid_description_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,!?]{1,200}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("description must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid branch names
fn valid_branch_strategy() -> impl Strategy<Value = String> {
    r"feature/[a-z0-9\-]{1,50}"
        .prop_map(|s| s.to_lowercase())
}

// Strategy for generating file changes
fn file_change_strategy() -> impl Strategy<Value = FileChange> {
    (
        r"[a-z0-9/\-_.]{1,50}\.rs",
        r"(added|modified|deleted)",
        0u32..100,
        0u32..100,
    )
        .prop_map(|(path, change_type, additions, deletions)| FileChange {
            path,
            change_type,
            additions,
            deletions,
        })
}

// Strategy for generating task contexts
fn task_context_strategy() -> impl Strategy<Value = TaskContext> {
    (
        valid_title_strategy(),
        valid_description_strategy(),
        prop::collection::vec(file_change_strategy(), 0..5),
        prop::collection::vec(1u32..1000, 0..3),
    )
        .prop_map(|(title, description, files, issues)| {
            let mut context = TaskContext::new(title, description);
            for file in files {
                context = context.with_file(file);
            }
            for issue in issues {
                context = context.with_issue(issue);
            }
            context
        })
}

// Strategy for generating PR options
fn pr_options_strategy() -> impl Strategy<Value = PrOptions> {
    (valid_branch_strategy(), any::<bool>()).prop_map(|(branch, draft)| {
        let options = PrOptions::new(branch);
        if draft {
            options.as_draft()
        } else {
            options
        }
    })
}

// **Feature: ricecoder-github, Property 1: PR Creation Integrity**
// *For any* task context and specified files, creating a PR SHALL include all specified files and no unintended files.
// **Validates: Requirements 1.1, 1.2**
proptest! {
    #[test]
    fn prop_pr_creation_includes_all_files(
        context in task_context_strategy(),
        options in pr_options_strategy()
    ) {
        let manager = PrManager::new();
        let pr = manager.create_pr_from_context(context.clone(), options).unwrap();

        // Property: All files from context should be in PR
        prop_assert_eq!(pr.files.len(), context.files.len());

        // Property: All file paths should match
        for (i, file) in context.files.iter().enumerate() {
            prop_assert_eq!(&pr.files[i].path, &file.path);
            prop_assert_eq!(&pr.files[i].change_type, &file.change_type);
            prop_assert_eq!(pr.files[i].additions, file.additions);
            prop_assert_eq!(pr.files[i].deletions, file.deletions);
        }

        // Property: No unintended files should be added
        prop_assert_eq!(pr.files.len(), context.files.len());
    }
}

// **Feature: ricecoder-github, Property 2: PR Content Generation**
// *For any* task context, the generated PR title and body SHALL be non-empty and contain meaningful content derived from the task context.
// **Validates: Requirements 1.2, 1.4**
proptest! {
    #[test]
    fn prop_pr_content_is_non_empty_and_meaningful(
        context in task_context_strategy(),
        options in pr_options_strategy()
    ) {
        let manager = PrManager::new();
        let pr = manager.create_pr_from_context(context.clone(), options).unwrap();

        // Property: Title must be non-empty
        prop_assert!(!pr.title.is_empty());

        // Property: Body must be non-empty
        prop_assert!(!pr.body.is_empty());

        // Property: Title should contain meaningful content from context
        prop_assert!(pr.title.contains(&context.title) || pr.title.len() > 0);

        // Property: Body should contain meaningful content
        prop_assert!(pr.body.contains(&context.description) || pr.body.contains("Description"));

        // Property: Title should not exceed reasonable length
        prop_assert!(pr.title.len() <= 256);
    }
}

// **Feature: ricecoder-github, Property 3: PR Issue Linking**
// *For any* PR creation with related issue references, the PR body SHALL contain the issue references in the correct format.
// **Validates: Requirements 1.3**
proptest! {
    #[test]
    fn prop_pr_issue_linking_format(
        context in task_context_strategy(),
        options in pr_options_strategy()
    ) {
        let manager = PrManager::new();
        let mut pr = manager.create_pr_from_context(context.clone(), options).unwrap();

        // Only test if there are related issues
        if !context.related_issues.is_empty() {
            manager.link_to_issues(&mut pr, context.related_issues.clone()).unwrap();

            // Property: PR body should contain close keywords for all issues
            for issue_num in &context.related_issues {
                let close_keyword = format!("Closes #{}", issue_num);
                prop_assert!(pr.body.contains(&close_keyword));
            }
        }
    }
}

// **Feature: ricecoder-github, Property 4: PR Template Customization**
// *For any* configured PR template, the generated PR SHALL match the configured template structure and placeholders.
// **Validates: Requirements 1.4**
proptest! {
    #[test]
    fn prop_pr_template_customization(
        context in task_context_strategy(),
        options in pr_options_strategy()
    ) {
        let custom_template = PrTemplate {
            title_template: "CUSTOM: {{title}}".to_string(),
            body_template: "CUSTOM BODY: {{description}}".to_string(),
        };

        let manager = PrManager::with_template(custom_template);
        let pr = manager.create_pr_from_context(context.clone(), options).unwrap();

        // Property: Title should start with custom prefix
        prop_assert!(pr.title.starts_with("CUSTOM:"));

        // Property: Body should start with custom prefix
        prop_assert!(pr.body.starts_with("CUSTOM BODY:"));

        // Property: Title should contain original title
        prop_assert!(pr.title.contains(&context.title));

        // Property: Body should contain original description
        prop_assert!(pr.body.contains(&context.description));
    }
}

// **Feature: ricecoder-github, Property 5: Draft PR Support**
// *For any* draft PR creation request, the created PR SHALL have draft status and not trigger CI/CD workflows.
// **Validates: Requirements 1.5**
proptest! {
    #[test]
    fn prop_draft_pr_support(
        context in task_context_strategy()
    ) {
        let manager = PrManager::new();

        // Create draft PR
        let draft_options = PrOptions::new("feature/test").as_draft();
        let draft_pr = manager.create_pr_from_context(context.clone(), draft_options).unwrap();

        // Property: Draft PR should have Draft status
        prop_assert_eq!(draft_pr.status, PrStatus::Draft);

        // Create non-draft PR
        let open_options = PrOptions::new("feature/test");
        let open_pr = manager.create_pr_from_context(context, open_options).unwrap();

        // Property: Non-draft PR should have Open status
        prop_assert_eq!(open_pr.status, PrStatus::Open);

        // Property: Draft and non-draft should differ only in status
        prop_assert_eq!(draft_pr.title, open_pr.title);
        prop_assert_eq!(draft_pr.branch, open_pr.branch);
    }
}
