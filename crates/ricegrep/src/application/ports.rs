//! Application Ports (Repository Traits)
//!
//! Traits that define the interfaces between the application layer and infrastructure.
//! Infrastructure adapters implement these traits to provide concrete implementations.
//!
//! # Design Principles (from Oracle Architectural Guidance)
//! - Traits live in application layer, NOT domain
//! - No async initially (can add later for performance)
//! - Infrastructure implements these traits
//! - Domain types flow through without modification

use crate::domain::{FilePath, SearchQuery, SearchResult, SearchMatch};
use crate::application::errors::{AppResult, AppError, IoOperation};

/// Repository trait for file operations
///
/// Provides an abstraction over filesystem operations, allowing for:
/// - Testing with mock implementations
/// - Alternative storage backends (memory, network, etc.)
/// - Consistent error handling across the codebase
///
/// # Example Implementation
/// ```ignore
/// struct FsFileRepository;
/// 
/// impl FileRepository for FsFileRepository {
///     fn read(&self, path: &FilePath) -> AppResult<String> {
///         std::fs::read_to_string(path.as_path())
///             .map_err(|e| AppError::Io {
///                 operation: IoOperation::Read,
///                 path: path.as_path().to_string_lossy().to_string(),
///                 source: e,
///             })
///     }
///     // ...
/// }
/// ```
pub trait FileRepository {
    /// Read file contents as string
    ///
    /// # Errors
    /// Returns `AppError::Io` if the file cannot be read
    fn read(&self, path: &FilePath) -> AppResult<String>;
    
    /// Write content to file
    ///
    /// # Errors
    /// Returns `AppError::Io` if the file cannot be written
    fn write(&self, path: &FilePath, content: &str) -> AppResult<()>;
    
    /// Check if file exists
    fn exists(&self, path: &FilePath) -> bool;
    
    /// Delete a file
    ///
    /// # Errors
    /// Returns `AppError::Io` if the file cannot be deleted
    fn delete(&self, path: &FilePath) -> AppResult<()>;
    
    /// Create parent directories for a path if they don't exist
    ///
    /// # Errors
    /// Returns `AppError::Io` if directories cannot be created
    fn ensure_parent_dirs(&self, path: &FilePath) -> AppResult<()>;
}

/// Metadata entry for indexed files
///
/// Represents cached metadata about a file in the search index.
/// Used for incremental updates and metadata gating (REQ-PERF-001).
#[derive(Debug, Clone, PartialEq)]
pub struct FileIndexEntry {
    /// Path to the indexed file
    pub path: String,
    /// Last modified timestamp (Unix epoch seconds)
    pub modified_at: u64,
    /// File size in bytes
    pub size: u64,
    /// Content hash for change detection
    pub content_hash: Option<String>,
}

impl FileIndexEntry {
    /// Create a new file index entry
    pub fn new(path: String, modified_at: u64, size: u64) -> Self {
        FileIndexEntry {
            path,
            modified_at,
            size,
            content_hash: None,
        }
    }
    
    /// Create entry with content hash
    pub fn with_hash(path: String, modified_at: u64, size: u64, hash: String) -> Self {
        FileIndexEntry {
            path,
            modified_at,
            size,
            content_hash: Some(hash),
        }
    }
    
    /// Check if this entry is stale compared to new metadata
    pub fn is_stale(&self, new_modified: u64, new_size: u64) -> bool {
        self.modified_at != new_modified || self.size != new_size
    }
}

/// Repository trait for search index operations
///
/// Provides an abstraction over the search index, allowing for:
/// - Testing with mock implementations
/// - Alternative indexing backends
/// - Consistent metadata gating logic
///
/// # REQ-PERF-001: Metadata Gating
/// The `get_metadata` and `is_stale` methods support the metadata gating
/// optimization that avoids re-indexing unchanged files.
pub trait IndexRepository {
    /// Get metadata for a file path
    ///
    /// Returns `None` if the file is not in the index
    fn get_metadata(&self, path: &FilePath) -> Option<FileIndexEntry>;
    
    /// Update metadata for a file
    ///
    /// # Errors
    /// Returns `AppError::Index` if the update fails
    fn update_metadata(&self, entry: FileIndexEntry) -> AppResult<()>;
    
    /// Remove metadata for a file
    ///
    /// # Errors
    /// Returns `AppError::Index` if the removal fails
    fn remove_metadata(&self, path: &FilePath) -> AppResult<()>;
    
    /// Search the index with a query
    ///
    /// # Errors
    /// Returns `AppError::Search` if the search fails
    fn search(&self, query: &SearchQuery) -> AppResult<Vec<SearchResult>>;
    
    /// Check if a file needs re-indexing based on metadata
    ///
    /// This is the core of metadata gating (REQ-PERF-001):
    /// - If file is not in index → needs indexing
    /// - If file metadata changed → needs re-indexing
    /// - If file is unchanged → skip indexing
    fn needs_reindex(&self, path: &FilePath, modified_at: u64, size: u64) -> bool {
        match self.get_metadata(path) {
            None => true, // Not in index
            Some(entry) => entry.is_stale(modified_at, size),
        }
    }
}

/// Repository trait for event publishing
///
/// Allows the application layer to publish domain events to external systems
/// (logging, metrics, event stores, etc.) without coupling to specific implementations.
pub trait EventPublisher {
    /// Publish a domain event
    ///
    /// Events are fire-and-forget; failures are logged but don't fail operations.
    fn publish(&self, event: &crate::domain::DomainEvent);
    
    /// Publish multiple events in batch
    fn publish_batch(&self, events: &[crate::domain::DomainEvent]) {
        for event in events {
            self.publish(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::cell::RefCell;

    /// Mock file repository for testing
    struct MockFileRepository {
        files: RefCell<HashMap<String, String>>,
    }

    impl MockFileRepository {
        fn new() -> Self {
            MockFileRepository {
                files: RefCell::new(HashMap::new()),
            }
        }
        
        fn with_file(self, path: &str, content: &str) -> Self {
            self.files.borrow_mut().insert(path.to_string(), content.to_string());
            self
        }
    }

    impl FileRepository for MockFileRepository {
        fn read(&self, path: &FilePath) -> AppResult<String> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow().get(&path_str)
                .cloned()
                .ok_or_else(|| AppError::Io {
                    operation: IoOperation::Read,
                    path: path_str,
                    source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
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
            Ok(()) // No-op for mock
        }
    }

    /// Mock index repository for testing
    struct MockIndexRepository {
        entries: RefCell<HashMap<String, FileIndexEntry>>,
    }

    impl MockIndexRepository {
        fn new() -> Self {
            MockIndexRepository {
                entries: RefCell::new(HashMap::new()),
            }
        }
    }

    impl IndexRepository for MockIndexRepository {
        fn get_metadata(&self, path: &FilePath) -> Option<FileIndexEntry> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.entries.borrow().get(&path_str).cloned()
        }
        
        fn update_metadata(&self, entry: FileIndexEntry) -> AppResult<()> {
            self.entries.borrow_mut().insert(entry.path.clone(), entry);
            Ok(())
        }
        
        fn remove_metadata(&self, path: &FilePath) -> AppResult<()> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.entries.borrow_mut().remove(&path_str);
            Ok(())
        }
        
        fn search(&self, _query: &SearchQuery) -> AppResult<Vec<SearchResult>> {
            // Mock returns empty results
            Ok(vec![])
        }
    }

    /// Mock event publisher for testing
    struct MockEventPublisher {
        events: RefCell<Vec<String>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            MockEventPublisher {
                events: RefCell::new(Vec::new()),
            }
        }
        
        fn event_count(&self) -> usize {
            self.events.borrow().len()
        }
    }

    impl EventPublisher for MockEventPublisher {
        fn publish(&self, event: &crate::domain::DomainEvent) {
            self.events.borrow_mut().push(format!("{:?}", event));
        }
    }

    // FileRepository Tests
    
    #[test]
    fn test_file_repository_read_existing() {
        let repo = MockFileRepository::new()
            .with_file("test.rs", "fn main() {}");
        
        let path = FilePath::new("test.rs").unwrap();
        let content = repo.read(&path).unwrap();
        
        assert_eq!(content, "fn main() {}");
    }
    
    #[test]
    fn test_file_repository_read_not_found() {
        let repo = MockFileRepository::new();
        
        let path = FilePath::new("missing.rs").unwrap();
        let result = repo.read(&path);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Io { operation: IoOperation::Read, .. }));
    }
    
    #[test]
    fn test_file_repository_write_and_read() {
        let repo = MockFileRepository::new();
        let path = FilePath::new("new.rs").unwrap();
        
        repo.write(&path, "hello world").unwrap();
        let content = repo.read(&path).unwrap();
        
        assert_eq!(content, "hello world");
    }
    
    #[test]
    fn test_file_repository_exists() {
        let repo = MockFileRepository::new()
            .with_file("exists.rs", "content");
        
        let existing = FilePath::new("exists.rs").unwrap();
        let missing = FilePath::new("missing.rs").unwrap();
        
        assert!(repo.exists(&existing));
        assert!(!repo.exists(&missing));
    }
    
    #[test]
    fn test_file_repository_delete() {
        let repo = MockFileRepository::new()
            .with_file("to_delete.rs", "content");
        
        let path = FilePath::new("to_delete.rs").unwrap();
        assert!(repo.exists(&path));
        
        repo.delete(&path).unwrap();
        assert!(!repo.exists(&path));
    }

    // IndexRepository Tests
    
    #[test]
    fn test_index_repository_get_metadata_missing() {
        let repo = MockIndexRepository::new();
        let path = FilePath::new("missing.rs").unwrap();
        
        assert!(repo.get_metadata(&path).is_none());
    }
    
    #[test]
    fn test_index_repository_update_and_get() {
        let repo = MockIndexRepository::new();
        let path = FilePath::new("test.rs").unwrap();
        
        let entry = FileIndexEntry::new("test.rs".to_string(), 1000, 100);
        repo.update_metadata(entry.clone()).unwrap();
        
        let retrieved = repo.get_metadata(&path).unwrap();
        assert_eq!(retrieved, entry);
    }
    
    #[test]
    fn test_index_repository_needs_reindex_not_indexed() {
        let repo = MockIndexRepository::new();
        let path = FilePath::new("new.rs").unwrap();
        
        assert!(repo.needs_reindex(&path, 1000, 100));
    }
    
    #[test]
    fn test_index_repository_needs_reindex_unchanged() {
        let repo = MockIndexRepository::new();
        let path = FilePath::new("unchanged.rs").unwrap();
        
        let entry = FileIndexEntry::new("unchanged.rs".to_string(), 1000, 100);
        repo.update_metadata(entry).unwrap();
        
        assert!(!repo.needs_reindex(&path, 1000, 100));
    }
    
    #[test]
    fn test_index_repository_needs_reindex_modified() {
        let repo = MockIndexRepository::new();
        let path = FilePath::new("modified.rs").unwrap();
        
        let entry = FileIndexEntry::new("modified.rs".to_string(), 1000, 100);
        repo.update_metadata(entry).unwrap();
        
        // Changed modification time
        assert!(repo.needs_reindex(&path, 2000, 100));
        // Changed size
        assert!(repo.needs_reindex(&path, 1000, 200));
    }
    
    #[test]
    fn test_index_repository_remove_metadata() {
        let repo = MockIndexRepository::new();
        let path = FilePath::new("to_remove.rs").unwrap();
        
        let entry = FileIndexEntry::new("to_remove.rs".to_string(), 1000, 100);
        repo.update_metadata(entry).unwrap();
        
        assert!(repo.get_metadata(&path).is_some());
        
        repo.remove_metadata(&path).unwrap();
        assert!(repo.get_metadata(&path).is_none());
    }

    // FileIndexEntry Tests
    
    #[test]
    fn test_file_index_entry_new() {
        let entry = FileIndexEntry::new("test.rs".to_string(), 1000, 100);
        
        assert_eq!(entry.path, "test.rs");
        assert_eq!(entry.modified_at, 1000);
        assert_eq!(entry.size, 100);
        assert!(entry.content_hash.is_none());
    }
    
    #[test]
    fn test_file_index_entry_with_hash() {
        let entry = FileIndexEntry::with_hash(
            "test.rs".to_string(),
            1000,
            100,
            "abc123".to_string(),
        );
        
        assert_eq!(entry.content_hash, Some("abc123".to_string()));
    }
    
    #[test]
    fn test_file_index_entry_is_stale() {
        let entry = FileIndexEntry::new("test.rs".to_string(), 1000, 100);
        
        // Same metadata = not stale
        assert!(!entry.is_stale(1000, 100));
        
        // Different time = stale
        assert!(entry.is_stale(2000, 100));
        
        // Different size = stale
        assert!(entry.is_stale(1000, 200));
    }

    // EventPublisher Tests
    
    #[test]
    fn test_event_publisher_publish() {
        let publisher = MockEventPublisher::new();
        
        let event = crate::domain::DomainEvent::SearchExecuted {
            file_path: "test.rs".to_string(),
            matches_found: 5,
        };
        
        publisher.publish(&event);
        assert_eq!(publisher.event_count(), 1);
    }
    
    #[test]
    fn test_event_publisher_publish_batch() {
        let publisher = MockEventPublisher::new();
        
        let events = vec![
            crate::domain::DomainEvent::SearchExecuted {
                file_path: "test1.rs".to_string(),
                matches_found: 1,
            },
            crate::domain::DomainEvent::SearchExecuted {
                file_path: "test2.rs".to_string(),
                matches_found: 2,
            },
        ];
        
        publisher.publish_batch(&events);
        assert_eq!(publisher.event_count(), 2);
    }
}
