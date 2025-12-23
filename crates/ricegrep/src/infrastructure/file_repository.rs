//! Filesystem-based FileRepository Implementation
//!
//! Implements the `FileRepository` trait using `std::fs` for real file I/O.

use std::fs;
use std::path::Path;

use crate::application::{AppResult, AppError, IoOperation, FileRepository};
use crate::domain::FilePath;

/// Filesystem-based implementation of `FileRepository`
///
/// Provides real file I/O operations using `std::fs`.
/// All errors are mapped to `AppError::Io` with appropriate context.
///
/// # Example
/// ```ignore
/// use ricegrep::infrastructure::FsFileRepository;
/// use ricegrep::application::FileRepository;
/// use ricegrep::domain::FilePath;
///
/// let repo = FsFileRepository::new();
/// let path = FilePath::new("src/main.rs").unwrap();
/// let content = repo.read(&path)?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct FsFileRepository;

impl FsFileRepository {
    /// Create a new filesystem repository
    pub fn new() -> Self {
        FsFileRepository
    }
    
    /// Helper to convert FilePath to &Path
    fn as_path(file_path: &FilePath) -> &Path {
        file_path.as_path()
    }
    
    /// Helper to create IoError with context
    fn io_error(operation: IoOperation, path: &FilePath, source: std::io::Error) -> AppError {
        AppError::Io {
            operation,
            path: path.as_path().to_string_lossy().to_string(),
            source,
        }
    }
}

impl FileRepository for FsFileRepository {
    fn read(&self, path: &FilePath) -> AppResult<String> {
        fs::read_to_string(Self::as_path(path))
            .map_err(|e| Self::io_error(IoOperation::Read, path, e))
    }
    
    fn write(&self, path: &FilePath, content: &str) -> AppResult<()> {
        // Ensure parent directories exist
        self.ensure_parent_dirs(path)?;
        
        fs::write(Self::as_path(path), content)
            .map_err(|e| Self::io_error(IoOperation::Write, path, e))
    }
    
    fn exists(&self, path: &FilePath) -> bool {
        Self::as_path(path).exists()
    }
    
    fn delete(&self, path: &FilePath) -> AppResult<()> {
        if self.exists(path) {
            fs::remove_file(Self::as_path(path))
                .map_err(|e| Self::io_error(IoOperation::Delete, path, e))
        } else {
            Ok(()) // Deleting non-existent file is a no-op
        }
    }
    
    fn ensure_parent_dirs(&self, path: &FilePath) -> AppResult<()> {
        if let Some(parent) = Self::as_path(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| Self::io_error(IoOperation::Create, path, e))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, FsFileRepository) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo = FsFileRepository::new();
        (temp_dir, repo)
    }

    #[test]
    fn test_write_and_read() {
        let (temp_dir, repo) = setup();
        let file_path = temp_dir.path().join("test.txt");
        let path = FilePath::new(file_path.to_str().unwrap()).unwrap();
        
        repo.write(&path, "hello world").unwrap();
        let content = repo.read(&path).unwrap();
        
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_read_nonexistent() {
        let (temp_dir, repo) = setup();
        let file_path = temp_dir.path().join("nonexistent.txt");
        let path = FilePath::new(file_path.to_str().unwrap()).unwrap();
        
        let result = repo.read(&path);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Io { operation: IoOperation::Read, .. }));
    }

    #[test]
    fn test_exists() {
        let (temp_dir, repo) = setup();
        let file_path = temp_dir.path().join("exists.txt");
        let path = FilePath::new(file_path.to_str().unwrap()).unwrap();
        
        assert!(!repo.exists(&path));
        
        repo.write(&path, "content").unwrap();
        
        assert!(repo.exists(&path));
    }

    #[test]
    fn test_delete() {
        let (temp_dir, repo) = setup();
        let file_path = temp_dir.path().join("to_delete.txt");
        let path = FilePath::new(file_path.to_str().unwrap()).unwrap();
        
        repo.write(&path, "content").unwrap();
        assert!(repo.exists(&path));
        
        repo.delete(&path).unwrap();
        assert!(!repo.exists(&path));
    }

    #[test]
    fn test_delete_nonexistent_is_ok() {
        let (temp_dir, repo) = setup();
        let file_path = temp_dir.path().join("nonexistent.txt");
        let path = FilePath::new(file_path.to_str().unwrap()).unwrap();
        
        // Should not error
        let result = repo.delete(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_parent_dirs() {
        let (temp_dir, repo) = setup();
        let file_path = temp_dir.path().join("nested/deep/file.txt");
        let path = FilePath::new(file_path.to_str().unwrap()).unwrap();
        
        // Parent doesn't exist yet
        assert!(!temp_dir.path().join("nested/deep").exists());
        
        repo.write(&path, "content").unwrap();
        
        // Parent now exists
        assert!(temp_dir.path().join("nested/deep").exists());
        assert!(repo.exists(&path));
    }

    #[test]
    fn test_overwrite_existing() {
        let (temp_dir, repo) = setup();
        let file_path = temp_dir.path().join("overwrite.txt");
        let path = FilePath::new(file_path.to_str().unwrap()).unwrap();
        
        repo.write(&path, "original").unwrap();
        assert_eq!(repo.read(&path).unwrap(), "original");
        
        repo.write(&path, "updated").unwrap();
        assert_eq!(repo.read(&path).unwrap(), "updated");
    }
}
