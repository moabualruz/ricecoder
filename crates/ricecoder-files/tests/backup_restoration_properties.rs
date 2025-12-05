//! Property-based tests for backup restoration
//! **Feature: ricecoder-files, Property 3: Backup Restoration**

use proptest::prelude::*;
use ricecoder_files::{BackupManager, ContentVerifier};
use tempfile::TempDir;
use tokio::fs;

// Property 3: Backup Restoration
// For any backed-up file, restoring from backup SHALL produce content identical to the original
// For any backed-up file, backup metadata is preserved correctly

fn arb_content() -> impl Strategy<Value = String> {
    ".*"
}

#[test]
fn prop_restored_content_matches_original() {
    proptest!(|(content in arb_content())| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let backup_dir = temp_dir.path().join("backups");
            let original_path = temp_dir.path().join("original.txt");
            let restored_path = temp_dir.path().join("restored.txt");

            // Write original file
            fs::write(&original_path, &content).await.unwrap();

            // Create backup
            let manager = BackupManager::new(backup_dir, 10);
            let metadata = manager.create_backup(&original_path).await.unwrap();

            // Restore from backup
            manager
                .restore_from_backup(&metadata.backup_path, &restored_path)
                .await
                .unwrap();

            // Read restored content
            let restored_content = fs::read_to_string(&restored_path).await.unwrap();

            // Verify restored content matches original exactly
            restored_content == content
        });
        prop_assert!(result);
    });
}

#[test]
fn prop_backup_metadata_preserved() {
    proptest!(|(content in arb_content())| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let backup_dir = temp_dir.path().join("backups");
            let original_path = temp_dir.path().join("original.txt");

            // Write original file
            fs::write(&original_path, &content).await.unwrap();

            // Create backup
            let manager = BackupManager::new(backup_dir, 10);
            let metadata = manager.create_backup(&original_path).await.unwrap();

            // Verify metadata fields are set correctly
            let path_matches = metadata.original_path == original_path;
            let backup_exists = metadata.backup_path.exists();

            // Verify content hash matches
            let expected_hash = ContentVerifier::compute_hash(&content);
            let hash_matches = metadata.content_hash == expected_hash;

            path_matches && backup_exists && hash_matches
        });
        prop_assert!(result);
    });
}

#[test]
fn prop_backup_hash_matches_content() {
    proptest!(|(content in arb_content())| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let backup_dir = temp_dir.path().join("backups");
            let original_path = temp_dir.path().join("original.txt");

            // Write original file
            fs::write(&original_path, &content).await.unwrap();

            // Create backup
            let manager = BackupManager::new(backup_dir, 10);
            let metadata = manager.create_backup(&original_path).await.unwrap();

            // Read backup content and compute hash
            let backup_content = fs::read_to_string(&metadata.backup_path)
                .await
                .unwrap();
            let computed_hash = ContentVerifier::compute_hash(&backup_content);

            // Verify stored hash matches computed hash
            metadata.content_hash == computed_hash
        });
        prop_assert!(result);
    });
}

// **Validates: Requirements 4.1, 4.3**
