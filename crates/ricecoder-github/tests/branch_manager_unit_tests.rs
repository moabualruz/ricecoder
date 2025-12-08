//! Unit tests for Branch Manager
//!
//! These tests verify specific functionality and edge cases

use ricecoder_github::{BranchManager, BranchProtection};

#[tokio::test]
async fn test_create_branch_with_valid_inputs() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager
        .create_branch("feature/new-feature", "main", None)
        .await;

    assert!(result.is_ok());
    let branch = result.unwrap();
    assert_eq!(branch.branch_name, "feature/new-feature");
    assert_eq!(branch.base_branch, "main");
    assert!(branch.success);
}

#[tokio::test]
async fn test_create_branch_with_commit_sha() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager
        .create_branch(
            "feature/test",
            "main",
            Some("abc123def456".to_string()),
        )
        .await;

    assert!(result.is_ok());
    let branch = result.unwrap();
    assert_eq!(branch.commit_sha, "abc123def456");
    assert!(branch.success);
}

#[tokio::test]
async fn test_create_branch_with_empty_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.create_branch("", "main", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_branch_with_empty_base() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager
        .create_branch("feature/test", "", None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_branch_with_invalid_name_starting_slash() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.create_branch("/invalid", "main", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_branch_with_invalid_name_ending_slash() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.create_branch("invalid/", "main", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_branch_with_invalid_name_double_slash() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.create_branch("feature//invalid", "main", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_branch_with_invalid_name_special_chars() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.create_branch("feature@invalid", "main", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_branch_with_valid_names() {
    let manager = BranchManager::new("test_token", "owner", "repo");

    let valid_names = vec![
        "feature/new-feature",
        "bugfix-123",
        "release_v1.0",
        "main",
        "develop",
        "feature/sub/branch",
    ];

    for name in valid_names {
        let result = manager.create_branch(name, "main", None).await;
        assert!(result.is_ok(), "Failed for branch name: {}", name);
    }
}

#[tokio::test]
async fn test_delete_branch_with_valid_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.delete_branch("feature/old-feature").await;

    assert!(result.is_ok());
    let deletion = result.unwrap();
    assert_eq!(deletion.branch_name, "feature/old-feature");
    assert!(deletion.success);
}

#[tokio::test]
async fn test_delete_branch_with_empty_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.delete_branch("").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_branch_main() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.delete_branch("main").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_branch_master() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.delete_branch("master").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_branch_info_with_valid_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.get_branch_info("main").await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.name, "main");
    assert!(!info.is_protected);
}

#[tokio::test]
async fn test_get_branch_info_with_empty_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.get_branch_info("").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_branches() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.list_branches().await;

    assert!(result.is_ok());
    let branches = result.unwrap();
    assert!(branches.contains(&"main".to_string()));
    assert!(branches.contains(&"develop".to_string()));
}

#[tokio::test]
async fn test_protect_branch_with_valid_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let protection = BranchProtection::default();
    let result = manager.protect_branch("main", protection).await;

    assert!(result.is_ok());
    let lifecycle = result.unwrap();
    assert_eq!(lifecycle.branch_name, "main");
    assert_eq!(lifecycle.operation, "protect");
    assert!(lifecycle.success);
}

#[tokio::test]
async fn test_protect_branch_with_empty_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let protection = BranchProtection::default();
    let result = manager.protect_branch("", protection).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_protect_branch_with_custom_settings() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let mut protection = BranchProtection::default();
    protection.required_review_count = 2;
    protection.require_code_owner_reviews = true;

    let result = manager.protect_branch("main", protection).await;

    assert!(result.is_ok());
    let lifecycle = result.unwrap();
    assert!(lifecycle.success);
}

#[tokio::test]
async fn test_unprotect_branch_with_valid_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.unprotect_branch("main").await;

    assert!(result.is_ok());
    let lifecycle = result.unwrap();
    assert_eq!(lifecycle.branch_name, "main");
    assert_eq!(lifecycle.operation, "unprotect");
    assert!(lifecycle.success);
}

#[tokio::test]
async fn test_unprotect_branch_with_empty_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.unprotect_branch("").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_rename_branch_with_valid_names() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager
        .rename_branch("feature/old", "feature/new")
        .await;

    assert!(result.is_ok());
    let lifecycle = result.unwrap();
    assert_eq!(lifecycle.branch_name, "feature/new");
    assert_eq!(lifecycle.operation, "rename");
    assert!(lifecycle.success);
}

#[tokio::test]
async fn test_rename_branch_with_empty_old_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.rename_branch("", "new-name").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_rename_branch_with_empty_new_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.rename_branch("old-name", "").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_rename_branch_with_invalid_new_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.rename_branch("old-name", "/invalid").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_branch_exists_main() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.branch_exists("main").await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_branch_exists_develop() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.branch_exists("develop").await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_branch_exists_nonexistent() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.branch_exists("feature/nonexistent").await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_branch_exists_with_empty_name() {
    let manager = BranchManager::new("test_token", "owner", "repo");
    let result = manager.branch_exists("").await;

    assert!(result.is_err());
}

#[test]
fn test_branch_protection_default() {
    let protection = BranchProtection::default();
    assert!(protection.require_pull_request_reviews);
    assert_eq!(protection.required_review_count, 1);
    assert!(protection.require_status_checks);
    assert!(protection.require_branches_up_to_date);
    assert!(protection.dismiss_stale_reviews);
    assert!(!protection.require_code_owner_reviews);
}

#[test]
fn test_branch_manager_creation() {
    let _manager = BranchManager::new("token123", "owner", "repo");
    // Manager created successfully
}

#[test]
fn test_branch_manager_with_different_repos() {
    let _manager1 = BranchManager::new("token", "owner1", "repo1");
    let _manager2 = BranchManager::new("token", "owner2", "repo2");
    // Managers created successfully
}

#[tokio::test]
async fn test_create_and_delete_branch_lifecycle() {
    let manager = BranchManager::new("test_token", "owner", "repo");

    // Create branch
    let create_result = manager
        .create_branch("feature/test", "main", None)
        .await;
    assert!(create_result.is_ok());

    // Delete branch
    let delete_result = manager.delete_branch("feature/test").await;
    assert!(delete_result.is_ok());
}

#[tokio::test]
async fn test_protect_and_unprotect_branch_lifecycle() {
    let manager = BranchManager::new("test_token", "owner", "repo");

    // Protect branch
    let protect_result = manager
        .protect_branch("main", BranchProtection::default())
        .await;
    assert!(protect_result.is_ok());

    // Unprotect branch
    let unprotect_result = manager.unprotect_branch("main").await;
    assert!(unprotect_result.is_ok());
}

#[tokio::test]
async fn test_multiple_branch_operations() {
    let manager = BranchManager::new("test_token", "owner", "repo");

    // Create multiple branches
    for i in 0..3 {
        let result = manager
            .create_branch(
                format!("feature/test-{}", i),
                "main",
                None,
            )
            .await;
        assert!(result.is_ok());
    }

    // List branches
    let list_result = manager.list_branches().await;
    assert!(list_result.is_ok());
}
