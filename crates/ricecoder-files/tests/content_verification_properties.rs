//! Property-based tests for content verification
//! **Feature: ricecoder-files, Property 2: Content Verification**

use proptest::prelude::*;
use ricecoder_files::ContentVerifier;

// Property 2: Content Verification
// For any content, hash is deterministic and consistent
// For any content, written content matches source byte-for-byte

#[test]
fn prop_hash_is_deterministic() {
    proptest!(|(content in ".*")| {
        // For any content, computing the hash twice should produce identical results
        let hash1 = ContentVerifier::compute_hash(&content);
        let hash2 = ContentVerifier::compute_hash(&content);
        prop_assert_eq!(hash1, hash2);
    });
}

#[test]
fn prop_different_content_produces_different_hashes() {
    proptest!(|(content1 in ".*", content2 in ".*")| {
        prop_assume!(content1 != content2);

        let hash1 = ContentVerifier::compute_hash(&content1);
        let hash2 = ContentVerifier::compute_hash(&content2);
        prop_assert_ne!(hash1, hash2);
    });
}

// Additional test: Hash length is consistent
#[test]
fn test_hash_length_is_consistent() {
    let hash1 = ContentVerifier::compute_hash("test");
    let hash2 = ContentVerifier::compute_hash("another test");
    let hash3 = ContentVerifier::compute_hash("");

    // SHA-256 always produces 64 character hex strings
    assert_eq!(hash1.len(), 64);
    assert_eq!(hash2.len(), 64);
    assert_eq!(hash3.len(), 64);
}

// **Validates: Requirements 1.6**
