//! Write File Use Case
//!
//! Orchestrates file write operations with validation and backup.

use crate::application::{AppResult, AppError, FileRepository, EventPublisher};
use crate::domain::{FilePath, DomainEvent};

/// Request for writing a file
#[derive(Debug, Clone)]
pub struct WriteFileRequest {
    /// Path to write to
    pub file_path: String,
    /// Content to write
    pub content: String,
    /// Whether to create backup of existing file
    pub create_backup: bool,
    /// Whether to fail if file already exists
    pub fail_if_exists: bool,
}

impl WriteFileRequest {
    /// Create a simple write request (overwrites existing)
    pub fn new(file_path: impl Into<String>, content: impl Into<String>) -> Self {
        WriteFileRequest {
            file_path: file_path.into(),
            content: content.into(),
            create_backup: false,
            fail_if_exists: false,
        }
    }
    
    /// Create a write request that backs up existing file
    pub fn with_backup(file_path: impl Into<String>, content: impl Into<String>) -> Self {
        WriteFileRequest {
            file_path: file_path.into(),
            content: content.into(),
            create_backup: true,
            fail_if_exists: false,
        }
    }
    
    /// Create a write request that fails if file exists
    pub fn create_new(file_path: impl Into<String>, content: impl Into<String>) -> Self {
        WriteFileRequest {
            file_path: file_path.into(),
            content: content.into(),
            create_backup: false,
            fail_if_exists: true,
        }
    }
}

/// Response from write operation
#[derive(Debug, Clone)]
pub struct WriteFileResponse {
    /// Path that was written
    pub file_path: String,
    /// Bytes written
    pub bytes_written: usize,
    /// Backup path (if backup was created)
    pub backup_path: Option<String>,
    /// Whether file existed before
    pub existed_before: bool,
}

/// Use case for writing files
///
/// # Example
/// ```ignore
/// let use_case = WriteFileUseCase::new(file_repo, event_publisher);
/// let request = WriteFileRequest::new("output.txt", "Hello World");
/// let response = use_case.execute(request)?;
/// ```
pub struct WriteFileUseCase<F: FileRepository, E: EventPublisher> {
    file_repo: F,
    event_publisher: E,
}

impl<F: FileRepository, E: EventPublisher> WriteFileUseCase<F, E> {
    /// Create a new write file use case
    pub fn new(file_repo: F, event_publisher: E) -> Self {
        WriteFileUseCase {
            file_repo,
            event_publisher,
        }
    }

    /// Execute the write operation
    pub fn execute(&self, request: WriteFileRequest) -> AppResult<WriteFileResponse> {
        // 1. Validate path
        let file_path = FilePath::new(&request.file_path)
            .map_err(|e| AppError::Validation { message: e.to_string() })?;
        
        // 2. Check if file exists
        let existed_before = self.file_repo.exists(&file_path);
        
        // 3. Handle fail_if_exists
        if request.fail_if_exists && existed_before {
            return Err(AppError::Validation {
                message: format!("File already exists: {}", request.file_path),
            });
        }
        
        // 4. Create backup if requested
        let backup_path = if request.create_backup && existed_before {
            let backup_path_str = format!("{}.bak", request.file_path);
            let backup_file_path = FilePath::new(&backup_path_str)
                .map_err(|e| AppError::Validation { message: e.to_string() })?;
            
            // Read existing content and write to backup
            let existing_content = self.file_repo.read(&file_path)?;
            self.file_repo.write(&backup_file_path, &existing_content)?;
            
            Some(backup_path_str)
        } else {
            None
        };
        
        // 5. Write the new content
        let bytes_written = request.content.len();
        self.file_repo.write(&file_path, &request.content)?;
        
        // 6. Publish event (using FileEditExecuted as a proxy for write)
        self.event_publisher.publish(&DomainEvent::FileEditExecuted {
            file_path: request.file_path.clone(),
            pattern: String::new(), // No pattern for direct write
            replacement: format!("[{} bytes]", bytes_written),
            matches_replaced: if existed_before { 1 } else { 0 },
            was_dry_run: false,
        });
        
        Ok(WriteFileResponse {
            file_path: request.file_path,
            bytes_written,
            backup_path,
            existed_before,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::IoOperation;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Test doubles
    struct TestFileRepo {
        files: RefCell<HashMap<String, String>>,
    }

    impl TestFileRepo {
        fn new() -> Self {
            TestFileRepo { files: RefCell::new(HashMap::new()) }
        }
        
        fn with_file(self, path: &str, content: &str) -> Self {
            self.files.borrow_mut().insert(path.to_string(), content.to_string());
            self
        }
        
        fn get_content(&self, path: &str) -> Option<String> {
            self.files.borrow().get(path).cloned()
        }
    }

    impl FileRepository for TestFileRepo {
        fn read(&self, path: &FilePath) -> AppResult<String> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow().get(&path_str).cloned()
                .ok_or_else(|| AppError::Io {
                    operation: IoOperation::Read,
                    path: path_str,
                    source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
                })
        }
        
        fn write(&self, path: &FilePath, content: &str) -> AppResult<()> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow_mut().insert(path_str, content.to_string());
            Ok(())
        }
        
        fn exists(&self, path: &FilePath) -> bool {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow().contains_key(&path_str)
        }
        
        fn delete(&self, path: &FilePath) -> AppResult<()> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow_mut().remove(&path_str);
            Ok(())
        }
        
        fn ensure_parent_dirs(&self, _path: &FilePath) -> AppResult<()> {
            Ok(())
        }
    }

    struct TestEventPublisher {
        events: RefCell<Vec<DomainEvent>>,
    }

    impl TestEventPublisher {
        fn new() -> Self {
            TestEventPublisher { events: RefCell::new(Vec::new()) }
        }
        
        fn event_count(&self) -> usize {
            self.events.borrow().len()
        }
    }

    impl EventPublisher for TestEventPublisher {
        fn publish(&self, event: &DomainEvent) {
            self.events.borrow_mut().push(event.clone());
        }
    }

    #[test]
    fn test_write_new_file() {
        let file_repo = TestFileRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = WriteFileUseCase::new(file_repo, event_pub);
        let request = WriteFileRequest::new("new.txt", "hello world");
        
        let response = use_case.execute(request).unwrap();
        
        assert_eq!(response.file_path, "new.txt");
        assert_eq!(response.bytes_written, 11);
        assert!(!response.existed_before);
        assert!(response.backup_path.is_none());
        
        // Verify file was written
        let content = use_case.file_repo.get_content("new.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_write_overwrite_existing() {
        let file_repo = TestFileRepo::new()
            .with_file("existing.txt", "old content");
        let event_pub = TestEventPublisher::new();
        
        let use_case = WriteFileUseCase::new(file_repo, event_pub);
        let request = WriteFileRequest::new("existing.txt", "new content");
        
        let response = use_case.execute(request).unwrap();
        
        assert!(response.existed_before);
        assert!(response.backup_path.is_none());
        
        let content = use_case.file_repo.get_content("existing.txt").unwrap();
        assert_eq!(content, "new content");
    }

    #[test]
    fn test_write_with_backup() {
        let file_repo = TestFileRepo::new()
            .with_file("backup.txt", "original content");
        let event_pub = TestEventPublisher::new();
        
        let use_case = WriteFileUseCase::new(file_repo, event_pub);
        let request = WriteFileRequest::with_backup("backup.txt", "updated content");
        
        let response = use_case.execute(request).unwrap();
        
        assert!(response.existed_before);
        assert_eq!(response.backup_path, Some("backup.txt.bak".to_string()));
        
        // Verify backup was created
        let backup = use_case.file_repo.get_content("backup.txt.bak").unwrap();
        assert_eq!(backup, "original content");
        
        // Verify new content was written
        let new_content = use_case.file_repo.get_content("backup.txt").unwrap();
        assert_eq!(new_content, "updated content");
    }

    #[test]
    fn test_write_fail_if_exists() {
        let file_repo = TestFileRepo::new()
            .with_file("existing.txt", "content");
        let event_pub = TestEventPublisher::new();
        
        let use_case = WriteFileUseCase::new(file_repo, event_pub);
        let request = WriteFileRequest::create_new("existing.txt", "new content");
        
        let result = use_case.execute(request);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[test]
    fn test_write_create_new_success() {
        let file_repo = TestFileRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = WriteFileUseCase::new(file_repo, event_pub);
        let request = WriteFileRequest::create_new("new.txt", "content");
        
        let response = use_case.execute(request).unwrap();
        
        assert!(!response.existed_before);
    }

    #[test]
    fn test_write_publishes_event() {
        let file_repo = TestFileRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = WriteFileUseCase::new(file_repo, event_pub);
        let request = WriteFileRequest::new("test.txt", "content");
        
        use_case.execute(request).unwrap();
        
        assert_eq!(use_case.event_publisher.event_count(), 1);
    }

    #[test]
    fn test_write_invalid_path() {
        let file_repo = TestFileRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = WriteFileUseCase::new(file_repo, event_pub);
        let request = WriteFileRequest::new("", "content");
        
        let result = use_case.execute(request);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }
}
