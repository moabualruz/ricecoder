//! Integration tests for git operations
//!
//! Tests git status checking, auto-commit workflow, diff review, and commit message generation.

use ricecoder_files::git::GitIntegration;
use ricecoder_files::models::FileOperation;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

/// Test git status checking with modified files
#[tokio::test]
async fn test_git_status_detects_modified_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a git repository
    let repo = git2::Repository::init(repo_path).unwrap();

    // Create and commit an initial file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "initial content").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("test.txt")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Modify the file
    fs::write(&file_path, "modified content").await.unwrap();

    // Check git status
    let status = GitIntegration::check_status(repo_path).unwrap();

    // Verify modified file is detected
    assert!(!status.modified.is_empty());
    assert!(status
        .modified
        .iter()
        .any(|p| p.to_string_lossy().contains("test.txt")));
}

/// Test git status with untracked files
#[tokio::test]
async fn test_git_status_detects_untracked_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a git repository
    let repo = git2::Repository::init(repo_path).unwrap();

    // Create an initial commit
    let file_path = repo_path.join("initial.txt");
    fs::write(&file_path, "initial").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("initial.txt")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Create an untracked file
    let untracked_path = repo_path.join("untracked.txt");
    fs::write(&untracked_path, "untracked content")
        .await
        .unwrap();

    // Check git status
    let status = GitIntegration::check_status(repo_path).unwrap();

    // Verify untracked file is detected
    assert!(!status.untracked.is_empty());
    assert!(status
        .untracked
        .iter()
        .any(|p| p.to_string_lossy().contains("untracked.txt")));
}

/// Test git status with staged files
#[tokio::test]
async fn test_git_status_detects_staged_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a git repository
    let repo = git2::Repository::init(repo_path).unwrap();

    // Create an initial commit
    let file_path = repo_path.join("initial.txt");
    fs::write(&file_path, "initial").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("initial.txt")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Create a new file and stage it
    let new_file = repo_path.join("new.txt");
    fs::write(&new_file, "new content").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("new.txt")).unwrap();
    index.write().unwrap();

    // Check git status
    let status = GitIntegration::check_status(repo_path).unwrap();

    // Verify staged file is detected
    assert!(!status.staged.is_empty());
    assert!(status
        .staged
        .iter()
        .any(|p| p.to_string_lossy().contains("new.txt")));
}

/// Test getting current branch name
#[tokio::test]
async fn test_get_current_branch() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a git repository
    let repo = git2::Repository::init(repo_path).unwrap();

    // Create an initial commit to establish a branch
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "content").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("test.txt")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Get current branch
    let branch = GitIntegration::get_current_branch(repo_path).unwrap();

    // Verify branch name is returned (usually "master" or "main")
    assert!(!branch.is_empty());
}

/// Test git status on clean repository
#[tokio::test]
async fn test_git_status_clean_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a git repository
    let repo = git2::Repository::init(repo_path).unwrap();

    // Create an initial commit
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "content").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("test.txt")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Check git status on clean repository
    let status = GitIntegration::check_status(repo_path).unwrap();

    // Verify no changes are detected
    assert!(status.modified.is_empty());
    assert!(status.staged.is_empty());
    assert!(status.untracked.is_empty());
}

/// Test commit message generation from file operations
#[test]
fn test_commit_message_generation_single_file() {
    let operations = vec![FileOperation {
        path: PathBuf::from("src/main.rs"),
        operation: ricecoder_files::models::OperationType::Create,
        content: Some("fn main() {}".to_string()),
        backup_path: None,
        content_hash: Some("hash".to_string()),
    }];

    let message = GitIntegration::generate_commit_message(&operations);

    // Verify message contains file information
    assert!(!message.is_empty());
    assert!(message.contains("main.rs") || message.contains("1 file"));
}

/// Test commit message generation with multiple files
#[test]
fn test_commit_message_generation_multiple_files() {
    let operations = vec![
        FileOperation {
            path: PathBuf::from("src/main.rs"),
            operation: ricecoder_files::models::OperationType::Create,
            content: Some("fn main() {}".to_string()),
            backup_path: None,
            content_hash: Some("hash1".to_string()),
        },
        FileOperation {
            path: PathBuf::from("src/lib.rs"),
            operation: ricecoder_files::models::OperationType::Create,
            content: Some("pub fn lib() {}".to_string()),
            backup_path: None,
            content_hash: Some("hash2".to_string()),
        },
        FileOperation {
            path: PathBuf::from("Cargo.toml"),
            operation: ricecoder_files::models::OperationType::Update,
            content: Some("[package]".to_string()),
            backup_path: None,
            content_hash: Some("hash3".to_string()),
        },
    ];

    let message = GitIntegration::generate_commit_message(&operations);

    // Verify message contains file count
    assert!(!message.is_empty());
    assert!(
        message.contains("Create 1 file(s)")
            || message.contains("Update 1 file(s)")
            || message.contains("file(s)")
    );
}

/// Test commit message generation with different operation types
#[test]
fn test_commit_message_includes_operation_types() {
    let operations = vec![
        FileOperation {
            path: PathBuf::from("new.rs"),
            operation: ricecoder_files::models::OperationType::Create,
            content: Some("new".to_string()),
            backup_path: None,
            content_hash: Some("hash1".to_string()),
        },
        FileOperation {
            path: PathBuf::from("modified.rs"),
            operation: ricecoder_files::models::OperationType::Update,
            content: Some("modified".to_string()),
            backup_path: None,
            content_hash: Some("hash2".to_string()),
        },
    ];

    let message = GitIntegration::generate_commit_message(&operations);

    // Verify message is generated
    assert!(!message.is_empty());
}

/// Test git status with multiple file types
#[tokio::test]
async fn test_git_status_with_multiple_file_types() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a git repository
    let repo = git2::Repository::init(repo_path).unwrap();

    // Create and commit initial files
    let file1 = repo_path.join("file1.txt");
    let file2 = repo_path.join("file2.txt");
    fs::write(&file1, "content1").await.unwrap();
    fs::write(&file2, "content2").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("file1.txt")).unwrap();
    index.add_path(std::path::Path::new("file2.txt")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Modify file1, create untracked file3, and stage file2 modification
    fs::write(&file1, "modified1").await.unwrap();
    fs::write(&file2, "modified2").await.unwrap();
    let file3 = repo_path.join("file3.txt");
    fs::write(&file3, "untracked").await.unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("file2.txt")).unwrap();
    index.write().unwrap();

    // Check git status
    let status = GitIntegration::check_status(repo_path).unwrap();

    // Verify all file types are detected
    assert!(!status.modified.is_empty()); // file1
    assert!(!status.staged.is_empty()); // file2
    assert!(!status.untracked.is_empty()); // file3
}
