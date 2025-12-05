//! Content verification and integrity checking

use crate::error::FileError;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

/// Verifies file integrity through content comparison and hashing
#[derive(Debug, Clone)]
pub struct ContentVerifier;

impl ContentVerifier {
    /// Creates a new ContentVerifier instance
    pub fn new() -> Self {
        ContentVerifier
    }

    /// Computes SHA-256 hash of content
    ///
    /// # Arguments
    ///
    /// * `content` - The content to hash
    ///
    /// # Returns
    ///
    /// Hexadecimal string representation of the SHA-256 hash
    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verifies that written content matches source byte-for-byte
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the written file
    /// * `expected` - Expected content
    ///
    /// # Returns
    ///
    /// Ok(()) if verification succeeds, error otherwise
    pub async fn verify_write(&self, path: &Path, expected: &str) -> Result<(), FileError> {
        let written = fs::read_to_string(path).await.map_err(|e| {
            FileError::VerificationFailed(format!("Failed to read written file: {}", e))
        })?;

        if written == expected {
            Ok(())
        } else {
            Err(FileError::VerificationFailed(
                "Written content does not match source".to_string(),
            ))
        }
    }

    /// Verifies backup integrity by comparing stored hash with computed hash
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file
    /// * `stored_hash` - Previously stored hash to compare against
    ///
    /// # Returns
    ///
    /// Ok(()) if hashes match (no corruption), error if mismatch or read fails
    pub async fn verify_backup(
        &self,
        backup_path: &Path,
        stored_hash: &str,
    ) -> Result<(), FileError> {
        let backup_content = fs::read_to_string(backup_path)
            .await
            .map_err(|e| FileError::BackupFailed(format!("Failed to read backup file: {}", e)))?;

        let computed_hash = Self::compute_hash(&backup_content);

        if computed_hash == stored_hash {
            Ok(())
        } else {
            Err(FileError::BackupCorrupted)
        }
    }
}

impl Default for ContentVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_deterministic() {
        let content = "test content";
        let hash1 = ContentVerifier::compute_hash(content);
        let hash2 = ContentVerifier::compute_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different_content() {
        let hash1 = ContentVerifier::compute_hash("content1");
        let hash2 = ContentVerifier::compute_hash("content2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_empty_string() {
        let hash = ContentVerifier::compute_hash("");
        // SHA-256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[tokio::test]
    async fn test_verify_write_matching_content() {
        let verifier = ContentVerifier::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = "test content";
        fs::write(&file_path, content).await.unwrap();

        let result = verifier.verify_write(&file_path, content).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_write_mismatched_content() {
        let verifier = ContentVerifier::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "actual content").await.unwrap();

        let result = verifier.verify_write(&file_path, "expected content").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_write_nonexistent_file() {
        let verifier = ContentVerifier::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = verifier.verify_write(&file_path, "content").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_backup_matching_hash() {
        let verifier = ContentVerifier::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.txt");

        let content = "backup content";
        fs::write(&backup_path, content).await.unwrap();
        let stored_hash = ContentVerifier::compute_hash(content);

        let result = verifier.verify_backup(&backup_path, &stored_hash).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_backup_mismatched_hash() {
        let verifier = ContentVerifier::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.txt");

        fs::write(&backup_path, "actual content").await.unwrap();
        let wrong_hash = ContentVerifier::compute_hash("different content");

        let result = verifier.verify_backup(&backup_path, &wrong_hash).await;
        assert!(result.is_err());
        match result {
            Err(FileError::BackupCorrupted) => (),
            _ => panic!("Expected BackupCorrupted error"),
        }
    }

    #[tokio::test]
    async fn test_verify_backup_nonexistent_file() {
        let verifier = ContentVerifier::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_path = temp_dir.path().join("nonexistent.txt");

        let result = verifier.verify_backup(&backup_path, "somehash").await;
        assert!(result.is_err());
    }
}
