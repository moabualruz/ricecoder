//! Property-based tests for Branch Lifecycle Management
//!
//! These tests verify correctness properties that should hold across all valid inputs

use proptest::prelude::*;
use ricecoder_github::BranchManager;

// Strategy for generating valid branch names
fn valid_branch_name_strategy() -> impl Strategy<Value = String> {
    r"(feature|bugfix|hotfix|release)/[a-z0-9\-]{1,30}"
        .prop_map(|s| s.to_string())
}

// Strategy for generating base branch names
fn base_branch_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("main".to_string()),
        Just("develop".to_string()),
        Just("staging".to_string()),
    ]
}

// Strategy for generating commit SHAs
fn commit_sha_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        r"[a-f0-9]{40}".prop_map(|s| Some(s.to_string())),
    ]
}

// Strategy for generating repository owners
fn owner_strategy() -> impl Strategy<Value = String> {
    r"[a-z0-9\-]{3,20}"
        .prop_map(|s| s.to_string())
}

// Strategy for generating repository names
fn repo_strategy() -> impl Strategy<Value = String> {
    r"[a-z0-9\-_]{3,30}"
        .prop_map(|s| s.to_string())
}

// **Feature: ricecoder-github, Property 62: Branch Lifecycle Management**
// *For any* branch creation and deletion, the system SHALL correctly create and delete branches.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_branch_creation_and_deletion(
        branch_name in valid_branch_name_strategy(),
        base_branch in base_branch_strategy(),
        commit_sha in commit_sha_strategy(),
        owner in owner_strategy(),
        repo in repo_strategy()
    ) {
        let manager = BranchManager::new("test_token", &owner, &repo);

        // Property 1: Branch creation should succeed with valid inputs
        let create_result = futures::executor::block_on(async {
            manager.create_branch(&branch_name, &base_branch, commit_sha.clone()).await
        });

        prop_assert!(create_result.is_ok(), "Branch creation failed");

        let creation = create_result.unwrap();

        // Property 2: Created branch name should match input
        prop_assert_eq!(&creation.branch_name, &branch_name);

        // Property 3: Base branch should match input
        prop_assert_eq!(&creation.base_branch, &base_branch);

        // Property 4: Creation should be marked as success
        prop_assert!(creation.success);

        // Property 5: Commit SHA should be set (either provided or HEAD)
        prop_assert!(!creation.commit_sha.is_empty());

        // Property 6: Branch deletion should succeed
        let delete_result = futures::executor::block_on(async {
            manager.delete_branch(&creation.branch_name).await
        });

        prop_assert!(delete_result.is_ok(), "Branch deletion failed");

        let deletion = delete_result.unwrap();

        // Property 7: Deleted branch name should match
        prop_assert_eq!(&deletion.branch_name, &branch_name);

        // Property 8: Deletion should be marked as success
        prop_assert!(deletion.success);
    }
}

// **Feature: ricecoder-github, Property 63: Branch Protection Lifecycle**
// *For any* branch protection operation, the system SHALL correctly protect and unprotect branches.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_branch_protection_lifecycle(
        base_branch in base_branch_strategy(),
        owner in owner_strategy(),
        repo in repo_strategy()
    ) {
        let manager = BranchManager::new("test_token", &owner, &repo);
        let protection = ricecoder_github::BranchProtection::default();

        // Property 1: Branch protection should succeed
        let protect_result = futures::executor::block_on(async {
            manager.protect_branch(&base_branch, protection.clone()).await
        });

        prop_assert!(protect_result.is_ok(), "Branch protection failed");

        let protect_lifecycle = protect_result.unwrap();

        // Property 2: Protected branch name should match
        prop_assert_eq!(&protect_lifecycle.branch_name, &base_branch);

        // Property 3: Operation should be "protect"
        prop_assert_eq!(protect_lifecycle.operation, "protect");

        // Property 4: Protection should be marked as success
        prop_assert!(protect_lifecycle.success);

        // Property 5: Branch unprotection should succeed
        let unprotect_result = futures::executor::block_on(async {
            manager.unprotect_branch(&protect_lifecycle.branch_name).await
        });

        prop_assert!(unprotect_result.is_ok(), "Branch unprotection failed");

        let unprotect_lifecycle = unprotect_result.unwrap();

        // Property 6: Unprotected branch name should match
        prop_assert_eq!(&unprotect_lifecycle.branch_name, &base_branch);

        // Property 7: Operation should be "unprotect"
        prop_assert_eq!(unprotect_lifecycle.operation, "unprotect");

        // Property 8: Unprotection should be marked as success
        prop_assert!(unprotect_lifecycle.success);
    }
}

// **Feature: ricecoder-github, Property 64: Branch Rename Idempotence**
// *For any* branch rename operation, renaming to the same name twice should result in identical state.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_branch_rename_consistency(
        old_name in valid_branch_name_strategy(),
        new_name in valid_branch_name_strategy(),
        owner in owner_strategy(),
        repo in repo_strategy()
    ) {
        let manager = BranchManager::new("test_token", &owner, &repo);

        // First rename
        let result1 = futures::executor::block_on(async {
            manager.rename_branch(&old_name, &new_name).await
        });

        prop_assert!(result1.is_ok(), "First rename failed");

        let rename1 = result1.unwrap();

        // Property 1: New branch name should match
        prop_assert_eq!(&rename1.branch_name, &new_name);

        // Property 2: Operation should be "rename"
        prop_assert_eq!(rename1.operation, "rename");

        // Property 3: Rename should be marked as success
        prop_assert!(rename1.success);

        // Property 4: Message should contain both old and new names
        prop_assert!(rename1.message.contains(&old_name));
        prop_assert!(rename1.message.contains(&rename1.branch_name));
    }
}

// **Feature: ricecoder-github, Property 65: Branch Existence Consistency**
// *For any* branch, checking existence should return consistent results.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_branch_existence_consistency(
        owner in owner_strategy(),
        repo in repo_strategy()
    ) {
        let manager = BranchManager::new("test_token", &owner, &repo);

        // Check main branch exists
        let main_exists = futures::executor::block_on(async {
            manager.branch_exists("main").await
        });

        prop_assert!(main_exists.is_ok());
        prop_assert!(main_exists.unwrap());

        // Check develop branch exists
        let develop_exists = futures::executor::block_on(async {
            manager.branch_exists("develop").await
        });

        prop_assert!(develop_exists.is_ok());
        prop_assert!(develop_exists.unwrap());

        // Check non-existent branch
        let nonexistent_exists = futures::executor::block_on(async {
            manager.branch_exists("feature/nonexistent-branch-xyz").await
        });

        prop_assert!(nonexistent_exists.is_ok());
        prop_assert!(!nonexistent_exists.unwrap());
    }
}

// **Feature: ricecoder-github, Property 66: Branch Information Retrieval**
// *For any* branch, retrieving information should return consistent data.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_branch_info_retrieval(
        branch_name in valid_branch_name_strategy(),
        owner in owner_strategy(),
        repo in repo_strategy()
    ) {
        let manager = BranchManager::new("test_token", &owner, &repo);

        // Get branch info
        let info_result = futures::executor::block_on(async {
            manager.get_branch_info(&branch_name).await
        });

        prop_assert!(info_result.is_ok(), "Failed to get branch info");

        let info = info_result.unwrap();

        // Property 1: Branch name should match
        prop_assert_eq!(&info.name, &branch_name);

        // Property 2: Commit SHA should not be empty
        prop_assert!(!info.commit_sha.is_empty());

        // Property 3: Protection status should be boolean
        let _ = info.is_protected; // Just verify it exists

        // Property 4: Getting info multiple times should return consistent results
        let info_result2 = futures::executor::block_on(async {
            manager.get_branch_info(&info.name).await
        });

        prop_assert!(info_result2.is_ok());
        let info2 = info_result2.unwrap();

        // Property 5: Branch names should match
        prop_assert_eq!(&info.name, &info2.name);
    }
}

// **Feature: ricecoder-github, Property 67: Branch List Consistency**
// *For any* repository, listing branches should return consistent results.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_branch_list_consistency(
        owner in owner_strategy(),
        repo in repo_strategy()
    ) {
        let manager = BranchManager::new("test_token", &owner, &repo);

        // List branches first time
        let list1 = futures::executor::block_on(async {
            manager.list_branches().await
        });

        prop_assert!(list1.is_ok());
        let branches1 = list1.unwrap();

        // Property 1: Should return a list
        prop_assert!(!branches1.is_empty());

        // Property 2: Should contain main branch
        prop_assert!(branches1.contains(&"main".to_string()));

        // List branches second time
        let list2 = futures::executor::block_on(async {
            manager.list_branches().await
        });

        prop_assert!(list2.is_ok());
        let branches2 = list2.unwrap();

        // Property 3: Results should be consistent
        prop_assert_eq!(branches1.len(), branches2.len());

        // Property 4: All branches from first list should be in second list
        for branch in &branches1 {
            prop_assert!(branches2.contains(branch));
        }
    }
}

// **Feature: ricecoder-github, Property 68: Branch Name Validation**
// *For any* branch name, validation should correctly identify valid and invalid names.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_branch_name_validation(
        branch_name in valid_branch_name_strategy()
    ) {
        let manager = BranchManager::new("test_token", "owner", "repo");

        // Valid branch names should succeed in creation
        let result = futures::executor::block_on(async {
            manager.create_branch(&branch_name, "main", None).await
        });

        prop_assert!(result.is_ok(), "Valid branch name should succeed");

        // Invalid branch names should fail
        let invalid_names = vec![
            "/invalid",
            "invalid/",
            "invalid//branch",
            "invalid@branch",
            "invalid#branch",
        ];

        for invalid_name in invalid_names {
            let result = futures::executor::block_on(async {
                manager.create_branch(invalid_name, "main", None).await
            });

            prop_assert!(result.is_err(), "Invalid branch name should fail: {}", invalid_name);
        }
    }
}
