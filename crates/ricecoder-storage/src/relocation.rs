//! Storage relocation functionality
//!
//! Provides functionality to move global storage to a new location
//! and update configuration pointers.

use crate::error::{IoOperation, StorageError, StorageResult};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Marker file that stores the current global storage path
const STORAGE_PATH_MARKER: &str = ".ricecoder_storage_path";

/// Relocation service for moving global storage
pub struct RelocationService;

impl RelocationService {
    /// Relocate storage from one location to another
    ///
    /// # Arguments
    ///
    /// * `from` - Current storage location
    /// * `to` - New storage location
    ///
    /// # Errors
    ///
    /// Returns error if relocation fails
    pub fn relocate(from: &Path, to: &Path) -> StorageResult<()> {
        debug!(
            "Starting relocation from {} to {}",
            from.display(),
            to.display()
        );

        // Validate source exists
        if !from.exists() {
            return Err(StorageError::relocation_error(
                from.to_path_buf(),
                to.to_path_buf(),
                "Source directory does not exist",
            ));
        }

        // Validate target doesn't exist or is empty
        if to.exists() {
            if to.is_dir() {
                let entries = fs::read_dir(to)
                    .map_err(|e| StorageError::io_error(to.to_path_buf(), IoOperation::Read, e))?;

                if entries.count() > 0 {
                    return Err(StorageError::relocation_error(
                        from.to_path_buf(),
                        to.to_path_buf(),
                        "Target directory is not empty",
                    ));
                }
            } else {
                return Err(StorageError::relocation_error(
                    from.to_path_buf(),
                    to.to_path_buf(),
                    "Target path exists and is not a directory",
                ));
            }
        }

        // Create target parent directory if needed
        if let Some(parent) = to.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    StorageError::directory_creation_failed(parent.to_path_buf(), e)
                })?;
            }
        }

        // Copy all data from source to target
        Self::copy_dir_recursive(from, to)?;

        // Verify data integrity by checking file count
        let source_count = Self::count_files(from)?;
        let target_count = Self::count_files(to)?;

        if source_count != target_count {
            // Cleanup target on failure
            let _ = fs::remove_dir_all(to);
            return Err(StorageError::relocation_error(
                from.to_path_buf(),
                to.to_path_buf(),
                format!(
                    "Data integrity check failed: {} files in source, {} in target",
                    source_count, target_count
                ),
            ));
        }

        // Update configuration pointer
        Self::update_storage_path_marker(to)?;

        info!(
            "Successfully relocated storage from {} to {}",
            from.display(),
            to.display()
        );

        Ok(())
    }

    /// Get the stored storage path from marker file
    ///
    /// # Arguments
    ///
    /// * `marker_dir` - Directory containing the marker file
    ///
    /// # Returns
    ///
    /// Returns the stored path if marker exists, None otherwise
    pub fn get_stored_path(marker_dir: &Path) -> StorageResult<Option<PathBuf>> {
        let marker_path = marker_dir.join(STORAGE_PATH_MARKER);

        if !marker_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&marker_path)
            .map_err(|e| StorageError::io_error(marker_path.clone(), IoOperation::Read, e))?;

        let path = PathBuf::from(content.trim());
        debug!("Read stored storage path: {}", path.display());

        Ok(Some(path))
    }

    /// Copy directory recursively
    fn copy_dir_recursive(src: &Path, dst: &Path) -> StorageResult<()> {
        fs::create_dir_all(dst)
            .map_err(|e| StorageError::directory_creation_failed(dst.to_path_buf(), e))?;

        for entry in fs::read_dir(src)
            .map_err(|e| StorageError::io_error(src.to_path_buf(), IoOperation::Read, e))?
        {
            let entry = entry
                .map_err(|e| StorageError::io_error(src.to_path_buf(), IoOperation::Read, e))?;

            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dst.join(&file_name);

            if path.is_dir() {
                Self::copy_dir_recursive(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)
                    .map_err(|e| StorageError::io_error(path.clone(), IoOperation::Read, e))?;
            }
        }

        Ok(())
    }

    /// Count files in directory recursively
    fn count_files(dir: &Path) -> StorageResult<usize> {
        let mut count = 0;

        for entry in fs::read_dir(dir)
            .map_err(|e| StorageError::io_error(dir.to_path_buf(), IoOperation::Read, e))?
        {
            let entry = entry
                .map_err(|e| StorageError::io_error(dir.to_path_buf(), IoOperation::Read, e))?;

            let path = entry.path();

            if path.is_dir() {
                count += Self::count_files(&path)?;
            } else {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Update the storage path marker file
    fn update_storage_path_marker(storage_path: &Path) -> StorageResult<()> {
        // Get the home directory to store the marker
        let home = dirs::home_dir().ok_or_else(|| {
            StorageError::path_resolution_error("Could not determine home directory")
        })?;

        let marker_path = home.join(STORAGE_PATH_MARKER);

        let path_str = storage_path.to_str().ok_or_else(|| {
            StorageError::path_resolution_error("Could not convert path to string")
        })?;

        fs::write(&marker_path, path_str)
            .map_err(|e| StorageError::io_error(marker_path.clone(), IoOperation::Write, e))?;

        debug!("Updated storage path marker: {}", marker_path.display());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_relocation_success() -> StorageResult<()> {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create some test files in source
        fs::write(source_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(source_dir.path().join("file2.txt"), "content2").unwrap();

        let source_path = source_dir.path().to_path_buf();
        let target_path = target_dir.path().join("new_storage");

        // Perform relocation
        RelocationService::relocate(&source_path, &target_path)?;

        // Verify files were copied
        assert!(target_path.join("file1.txt").exists());
        assert!(target_path.join("file2.txt").exists());

        let content1 = fs::read_to_string(target_path.join("file1.txt")).unwrap();
        assert_eq!(content1, "content1");

        Ok(())
    }

    #[test]
    fn test_relocation_with_subdirs() -> StorageResult<()> {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create nested structure
        fs::create_dir(source_dir.path().join("subdir")).unwrap();
        fs::write(source_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(source_dir.path().join("subdir/file2.txt"), "content2").unwrap();

        let source_path = source_dir.path().to_path_buf();
        let target_path = target_dir.path().join("new_storage");

        // Perform relocation
        RelocationService::relocate(&source_path, &target_path)?;

        // Verify structure was copied
        assert!(target_path.join("file1.txt").exists());
        assert!(target_path.join("subdir/file2.txt").exists());

        Ok(())
    }

    #[test]
    fn test_relocation_source_not_exists() {
        let source_path = PathBuf::from("/nonexistent/source");
        let target_path = PathBuf::from("/nonexistent/target");

        let result = RelocationService::relocate(&source_path, &target_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_relocation_target_not_empty() -> StorageResult<()> {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create file in source
        fs::write(source_dir.path().join("file1.txt"), "content1").unwrap();

        // Create file in target to make it non-empty
        fs::write(target_dir.path().join("existing.txt"), "existing").unwrap();

        let source_path = source_dir.path().to_path_buf();
        let target_path = target_dir.path().to_path_buf();

        let result = RelocationService::relocate(&source_path, &target_path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_count_files() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();

        // Create test structure
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        fs::write(temp_dir.path().join("subdir/file3.txt"), "content3").unwrap();

        let count = RelocationService::count_files(temp_dir.path())?;
        assert_eq!(count, 3);

        Ok(())
    }
}
