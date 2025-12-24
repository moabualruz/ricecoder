//! Application Services Container (Dependency Injection)
//!
//! Provides a centralized container for wiring up use cases with their dependencies.
//! This enables:
//! - Easy testing with mock implementations
//! - Clean separation of construction from use
//! - Consistent service configuration

use crate::application::{
    FileRepository, IndexRepository, EventPublisher,
    use_cases::{EditFileUseCase, SearchFilesUseCase, WriteFileUseCase},
};

/// Application services container
///
/// Holds all configured use cases, ready to be invoked.
/// Use the builder pattern via `AppServicesBuilder` to construct.
///
/// # Example
/// ```ignore
/// use ricegrep_core::application::AppServicesBuilder;
///
/// let services = AppServicesBuilder::new()
///     .with_file_repo(my_file_repo)
///     .with_index_repo(my_index_repo)
///     .with_event_publisher(my_event_publisher)
///     .build();
///
/// let edit_result = services.edit_file().execute(request);
/// ```
pub struct AppServices<F, I, E>
where
    F: FileRepository,
    I: IndexRepository,
    E: EventPublisher,
{
    edit_file: EditFileUseCase<F, E>,
    search_files: SearchFilesUseCase<I, E>,
    write_file: WriteFileUseCase<F, E>,
}

impl<F, I, E> AppServices<F, I, E>
where
    F: FileRepository + Clone,
    I: IndexRepository,
    E: EventPublisher + Clone,
{
    /// Create a new services container with all dependencies
    pub fn new(file_repo: F, index_repo: I, event_publisher: E) -> Self {
        AppServices {
            edit_file: EditFileUseCase::new(file_repo.clone(), event_publisher.clone()),
            search_files: SearchFilesUseCase::new(index_repo, event_publisher.clone()),
            write_file: WriteFileUseCase::new(file_repo, event_publisher),
        }
    }

    /// Get the edit file use case
    pub fn edit_file(&self) -> &EditFileUseCase<F, E> {
        &self.edit_file
    }

    /// Get the search files use case
    pub fn search_files(&self) -> &SearchFilesUseCase<I, E> {
        &self.search_files
    }

    /// Get the write file use case
    pub fn write_file(&self) -> &WriteFileUseCase<F, E> {
        &self.write_file
    }
}

/// Builder for constructing AppServices
///
/// Allows step-by-step configuration of all dependencies.
pub struct AppServicesBuilder<F, I, E> {
    file_repo: Option<F>,
    index_repo: Option<I>,
    event_publisher: Option<E>,
}

impl<F, I, E> Default for AppServicesBuilder<F, I, E>
where
    F: FileRepository + Clone,
    I: IndexRepository,
    E: EventPublisher + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<F, I, E> AppServicesBuilder<F, I, E>
where
    F: FileRepository + Clone,
    I: IndexRepository,
    E: EventPublisher + Clone,
{
    /// Create a new builder
    pub fn new() -> Self {
        AppServicesBuilder {
            file_repo: None,
            index_repo: None,
            event_publisher: None,
        }
    }

    /// Set the file repository
    pub fn with_file_repo(mut self, repo: F) -> Self {
        self.file_repo = Some(repo);
        self
    }

    /// Set the index repository
    pub fn with_index_repo(mut self, repo: I) -> Self {
        self.index_repo = Some(repo);
        self
    }

    /// Set the event publisher
    pub fn with_event_publisher(mut self, publisher: E) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    /// Build the services container
    ///
    /// # Panics
    /// Panics if any dependency is not set
    pub fn build(self) -> AppServices<F, I, E> {
        AppServices::new(
            self.file_repo.expect("FileRepository not set"),
            self.index_repo.expect("IndexRepository not set"),
            self.event_publisher.expect("EventPublisher not set"),
        )
    }

    /// Try to build the services container
    ///
    /// Returns None if any dependency is not set
    pub fn try_build(self) -> Option<AppServices<F, I, E>> {
        Some(AppServices::new(
            self.file_repo?,
            self.index_repo?,
            self.event_publisher?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{AppResult, FileIndexEntry};
    use crate::domain::{FilePath, SearchQuery, SearchResult, DomainEvent};
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Test implementations
    #[derive(Clone)]
    struct MockFileRepo {
        files: std::sync::Arc<RefCell<HashMap<String, String>>>,
    }

    impl MockFileRepo {
        fn new() -> Self {
            MockFileRepo {
                files: std::sync::Arc::new(RefCell::new(HashMap::new())),
            }
        }
        
        fn with_file(self, path: &str, content: &str) -> Self {
            self.files.borrow_mut().insert(path.to_string(), content.to_string());
            self
        }
    }

    impl FileRepository for MockFileRepo {
        fn read(&self, path: &FilePath) -> AppResult<String> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow().get(&path_str).cloned()
                .ok_or_else(|| crate::application::AppError::Io {
                    operation: crate::application::IoOperation::Read,
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

    struct MockIndexRepo;

    impl IndexRepository for MockIndexRepo {
        fn get_metadata(&self, _path: &FilePath) -> Option<FileIndexEntry> {
            None
        }
        
        fn update_metadata(&self, _entry: FileIndexEntry) -> AppResult<()> {
            Ok(())
        }
        
        fn remove_metadata(&self, _path: &FilePath) -> AppResult<()> {
            Ok(())
        }
        
        fn search(&self, _query: &SearchQuery) -> AppResult<Vec<SearchResult>> {
            Ok(vec![])
        }
    }

    #[derive(Clone)]
    struct MockEventPublisher {
        events: std::sync::Arc<RefCell<Vec<DomainEvent>>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            MockEventPublisher {
                events: std::sync::Arc::new(RefCell::new(Vec::new())),
            }
        }
        
        fn event_count(&self) -> usize {
            self.events.borrow().len()
        }
    }

    impl EventPublisher for MockEventPublisher {
        fn publish(&self, event: &DomainEvent) {
            self.events.borrow_mut().push(event.clone());
        }
    }

    #[test]
    fn test_app_services_construction() {
        let file_repo = MockFileRepo::new().with_file("test.rs", "fn main() {}");
        let index_repo = MockIndexRepo;
        let event_pub = MockEventPublisher::new();
        
        let services = AppServices::new(file_repo, index_repo, event_pub);
        
        // Services should be accessible
        let _ = services.edit_file();
        let _ = services.search_files();
        let _ = services.write_file();
    }

    #[test]
    fn test_app_services_builder() {
        let file_repo = MockFileRepo::new();
        let index_repo = MockIndexRepo;
        let event_pub = MockEventPublisher::new();
        
        let services = AppServicesBuilder::new()
            .with_file_repo(file_repo)
            .with_index_repo(index_repo)
            .with_event_publisher(event_pub)
            .build();
        
        let _ = services.edit_file();
    }

    #[test]
    fn test_app_services_try_build_success() {
        let file_repo = MockFileRepo::new();
        let index_repo = MockIndexRepo;
        let event_pub = MockEventPublisher::new();
        
        let result = AppServicesBuilder::new()
            .with_file_repo(file_repo)
            .with_index_repo(index_repo)
            .with_event_publisher(event_pub)
            .try_build();
        
        assert!(result.is_some());
    }

    #[test]
    fn test_app_services_try_build_missing() {
        let result: Option<AppServices<MockFileRepo, MockIndexRepo, MockEventPublisher>> = 
            AppServicesBuilder::new()
                .with_file_repo(MockFileRepo::new())
                // Missing index_repo and event_publisher
                .try_build();
        
        assert!(result.is_none());
    }

    #[test]
    fn test_app_services_edit_file_use_case() {
        use crate::application::use_cases::EditFileRequest;
        
        let file_repo = MockFileRepo::new().with_file("test.rs", "hello world");
        let index_repo = MockIndexRepo;
        let event_pub = MockEventPublisher::new();
        
        let services = AppServices::new(file_repo, index_repo, event_pub.clone());
        
        let request = EditFileRequest {
            file_path: "test.rs".to_string(),
            pattern: "hello".to_string(),
            replacement: "goodbye".to_string(),
            is_regex: false,
            dry_run: true,
        };
        
        let result = services.edit_file().execute(request);
        
        assert!(result.is_ok());
        assert!(event_pub.event_count() > 0);
    }

    #[test]
    fn test_app_services_write_file_use_case() {
        use crate::application::use_cases::WriteFileRequest;
        
        let file_repo = MockFileRepo::new();
        let index_repo = MockIndexRepo;
        let event_pub = MockEventPublisher::new();
        
        let services = AppServices::new(file_repo, index_repo, event_pub);
        
        let request = WriteFileRequest::new("output.txt", "test content");
        
        let result = services.write_file().execute(request);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.bytes_written, 12);
    }

    #[test]
    fn test_app_services_search_use_case() {
        use crate::application::use_cases::SearchFilesRequest;
        
        let file_repo = MockFileRepo::new();
        let index_repo = MockIndexRepo;
        let event_pub = MockEventPublisher::new();
        
        let services = AppServices::new(file_repo, index_repo, event_pub);
        
        let request = SearchFilesRequest::literal("TODO");
        
        let result = services.search_files().execute(request);
        
        assert!(result.is_ok());
        // Empty results from mock
        assert!(result.unwrap().results.is_empty());
    }
}
