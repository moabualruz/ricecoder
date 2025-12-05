//! Property-based tests for backup integrity
//! **Feature: ricecoder-files, Property 4: Backup Integrity**

use proptest::prelude::*;
use ricecoder_files::ContentVerifier;

// Property 4: Backup Integrity
// For any backup, computing the hash of the backup file SHALL match the stored hash, detecting any corruption
// For any backup, modifying the backup produces a different hash

#[test]
fn prop_hash_consistency_for_same_content() {
    proptest!(|(content in ".*")| {
        // For any content, the hash should be consistent
        let hash1 = ContentVerifier::compute_hash(&content);
        let hash2 = ContentVerifier::compute_hash(&content);
        prop_assert_eq!(hash1, hash2);
    });
}

#[test]
fn prop_different_content_different_hash() {
    proptest!(|(content1 in ".*", content2 in ".*")| {
        prop_assume!(content1 != content2);

        let hash1 = ContentVerifier::compute_hash(&content1);
        let hash2 = ContentVerifier::compute_hash(&content2);
        prop_assert_ne!(hash1, hash2);
    });
}

// Additional test: Backup corruption detection
#[test]
fn test_backup_corruption_detection() {
    let original_content = "important backup data";
    let original_hash = ContentVerifier::compute_hash(original_content);

    // Simulate corruption by modifying content
    let corrupted_content = "important backup data CORRUPTED";
    let corrupted_hash = ContentVerifier::compute_hash(corrupted_content);

    // Hashes should be different
    assert_ne!(original_hash, corrupted_hash);
}

// **Validates: Requirements 4.4**
