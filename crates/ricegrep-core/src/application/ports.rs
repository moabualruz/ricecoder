//! Application Ports (Repository Traits)

use crate::domain::{FilePath, SearchQuery, SearchResult, DomainEvent};
use crate::application::errors::{AppResult, AppError, IoOperation};

/// Repository trait for file operations
pub trait FileRepository {
    fn read(&self, path: &FilePath) -> AppResult<String>;
    fn write(&self, path: &FilePath, content: &str) -> AppResult<()>;
    fn exists(&self, path: &FilePath) -> bool;
    fn delete(&self, path: &FilePath) -> AppResult<()>;
    fn ensure_parent_dirs(&self, path: &FilePath) -> AppResult<()>;
}

/// Metadata entry for indexed files
#[derive(Debug, Clone, PartialEq)]
pub struct FileIndexEntry {
    pub path: String,
    pub modified_at: u64,
    pub size: u64,
    pub content_hash: Option<String>,
}

impl FileIndexEntry {
    pub fn new(path: String, modified_at: u64, size: u64) -> Self {
        FileIndexEntry { path, modified_at, size, content_hash: None }
    }
    
    pub fn with_hash(path: String, modified_at: u64, size: u64, hash: String) -> Self {
        FileIndexEntry { path, modified_at, size, content_hash: Some(hash) }
    }
    
    pub fn is_stale(&self, new_modified: u64, new_size: u64) -> bool {
        self.modified_at != new_modified || self.size != new_size
    }
}

/// Repository trait for search index operations
pub trait IndexRepository {
    fn get_metadata(&self, path: &FilePath) -> Option<FileIndexEntry>;
    fn update_metadata(&self, entry: FileIndexEntry) -> AppResult<()>;
    fn remove_metadata(&self, path: &FilePath) -> AppResult<()>;
    fn search(&self, query: &SearchQuery) -> AppResult<Vec<SearchResult>>;
    
    fn needs_reindex(&self, path: &FilePath, modified_at: u64, size: u64) -> bool {
        match self.get_metadata(path) {
            None => true,
            Some(entry) => entry.is_stale(modified_at, size),
        }
    }
}

/// Repository trait for event publishing
pub trait EventPublisher {
    fn publish(&self, event: &DomainEvent);
    fn publish_batch(&self, events: &[DomainEvent]) {
        for event in events { self.publish(event); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_index_entry_is_stale() {
        let entry = FileIndexEntry::new("test.rs".to_string(), 1000, 100);
        assert!(!entry.is_stale(1000, 100));
        assert!(entry.is_stale(2000, 100));
    }
}
