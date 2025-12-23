//! Metadata-based IndexRepository Implementation
//!
//! Implements the `IndexRepository` trait using the existing `metadata_gating` module.

use crate::application::{AppResult, AppError, IndexRepository, FileIndexEntry as AppFileIndexEntry};
use crate::domain::{FilePath, SearchQuery, SearchResult};
use crate::indexing_optimization::metadata_gating::{MetadataStore, FileIndexEntry as InternalFileIndexEntry};

use std::sync::{Arc, Mutex};
use std::path::PathBuf;

/// Metadata-based implementation of `IndexRepository`
///
/// Wraps the existing `MetadataStore` from the `indexing_optimization` module
/// to provide index metadata operations.
///
/// # Note
/// This is a thin adapter. The actual search functionality would require
/// integration with the lexical search module, which is deferred to later tasks.
pub struct MetadataIndexRepository {
    store: Arc<Mutex<MetadataStore>>,
}

impl MetadataIndexRepository {
    /// Create a new index repository with in-memory storage (temp file for metadata)
    pub fn new() -> Self {
        let temp_path = std::env::temp_dir().join(format!("ricegrep_metadata_{}.json", std::process::id()));
        MetadataIndexRepository {
            store: Arc::new(Mutex::new(MetadataStore::new(temp_path, false))),
        }
    }
    
    /// Create a new index repository with persistent storage
    pub fn with_path(path: PathBuf) -> Self {
        MetadataIndexRepository {
            store: Arc::new(Mutex::new(MetadataStore::new(path, true))),
        }
    }
    
    /// Load existing metadata from disk
    pub fn load(&self) -> AppResult<()> {
        let store = self.store.lock().map_err(|_| AppError::Index {
            operation: "load".to_string(),
            message: "Failed to acquire lock".to_string(),
        })?;
        
        store.load().map_err(|e| AppError::Index {
            operation: "load".to_string(),
            message: e.to_string(),
        })
    }
    
    /// Save metadata to disk
    pub fn save(&self) -> AppResult<()> {
        let store = self.store.lock().map_err(|_| AppError::Index {
            operation: "save".to_string(),
            message: "Failed to acquire lock".to_string(),
        })?;
        
        store.save().map_err(|e| AppError::Index {
            operation: "save".to_string(),
            message: e.to_string(),
        })
    }
    
    /// Get a clone of the Arc for sharing across threads
    pub fn clone_store(&self) -> Arc<Mutex<MetadataStore>> {
        Arc::clone(&self.store)
    }
    
    /// Convert internal FileIndexEntry to application FileIndexEntry
    fn to_app_entry(internal: &InternalFileIndexEntry) -> AppFileIndexEntry {
        AppFileIndexEntry {
            path: internal.path.to_string_lossy().to_string(),
            modified_at: internal.file_mtime,
            size: internal.file_size,
            // Convert u64 hash to string representation if present
            content_hash: internal.content_hash.map(|h| format!("{:016x}", h)),
        }
    }
    
    /// Convert application FileIndexEntry to internal FileIndexEntry
    fn to_internal_entry(app: &AppFileIndexEntry) -> InternalFileIndexEntry {
        InternalFileIndexEntry {
            path: PathBuf::from(&app.path),
            indexed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            file_mtime: app.modified_at,
            file_size: app.size,
            // Parse hex string back to u64 if present
            content_hash: app.content_hash.as_ref().and_then(|s| u64::from_str_radix(s, 16).ok()),
            is_deleted: false,
        }
    }
}

impl Default for MetadataIndexRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexRepository for MetadataIndexRepository {
    fn get_metadata(&self, path: &FilePath) -> Option<AppFileIndexEntry> {
        let store = self.store.lock().ok()?;
        let path_buf = path.as_path().to_path_buf();
        
        store.get(&path_buf).map(|entry| Self::to_app_entry(&entry))
    }
    
    fn update_metadata(&self, entry: AppFileIndexEntry) -> AppResult<()> {
        let store = self.store.lock().map_err(|_| AppError::Index {
            operation: "update".to_string(),
            message: "Failed to acquire lock".to_string(),
        })?;
        
        let internal_entry = Self::to_internal_entry(&entry);
        
        store.store(internal_entry).map_err(|e| AppError::Index {
            operation: "update".to_string(),
            message: e.to_string(),
        })
    }
    
    fn remove_metadata(&self, path: &FilePath) -> AppResult<()> {
        let store = self.store.lock().map_err(|_| AppError::Index {
            operation: "remove".to_string(),
            message: "Failed to acquire lock".to_string(),
        })?;
        
        let path_buf = path.as_path().to_path_buf();
        store.remove(&path_buf).map_err(|e| AppError::Index {
            operation: "remove".to_string(),
            message: e.to_string(),
        })?;
        Ok(())
    }
    
    fn search(&self, _query: &SearchQuery) -> AppResult<Vec<SearchResult>> {
        // TODO: Integrate with lexical search module in future task
        // For now, return empty results as this is infrastructure scaffolding
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_repository() {
        let repo = MetadataIndexRepository::new();
        let path = FilePath::new("test.rs").unwrap();
        
        // Should return None for non-existent entry
        assert!(repo.get_metadata(&path).is_none());
    }

    #[test]
    fn test_update_and_get_metadata() {
        let repo = MetadataIndexRepository::new();
        let path = FilePath::new("test.rs").unwrap();
        
        let entry = AppFileIndexEntry::new("test.rs".to_string(), 1000, 100);
        repo.update_metadata(entry.clone()).unwrap();
        
        let retrieved = repo.get_metadata(&path).unwrap();
        assert_eq!(retrieved.path, "test.rs");
        assert_eq!(retrieved.modified_at, 1000);
        assert_eq!(retrieved.size, 100);
    }

    #[test]
    fn test_remove_metadata() {
        let repo = MetadataIndexRepository::new();
        let path = FilePath::new("test.rs").unwrap();
        
        let entry = AppFileIndexEntry::new("test.rs".to_string(), 1000, 100);
        repo.update_metadata(entry).unwrap();
        
        assert!(repo.get_metadata(&path).is_some());
        
        repo.remove_metadata(&path).unwrap();
        
        assert!(repo.get_metadata(&path).is_none());
    }

    #[test]
    fn test_needs_reindex_not_indexed() {
        let repo = MetadataIndexRepository::new();
        let path = FilePath::new("new.rs").unwrap();
        
        assert!(repo.needs_reindex(&path, 1000, 100));
    }

    #[test]
    fn test_needs_reindex_unchanged() {
        let repo = MetadataIndexRepository::new();
        let path = FilePath::new("unchanged.rs").unwrap();
        
        let entry = AppFileIndexEntry::new("unchanged.rs".to_string(), 1000, 100);
        repo.update_metadata(entry).unwrap();
        
        assert!(!repo.needs_reindex(&path, 1000, 100));
    }

    #[test]
    fn test_needs_reindex_changed() {
        let repo = MetadataIndexRepository::new();
        let path = FilePath::new("changed.rs").unwrap();
        
        let entry = AppFileIndexEntry::new("changed.rs".to_string(), 1000, 100);
        repo.update_metadata(entry).unwrap();
        
        // Changed mtime
        assert!(repo.needs_reindex(&path, 2000, 100));
        // Changed size
        assert!(repo.needs_reindex(&path, 1000, 200));
    }

    #[test]
    fn test_search_returns_empty() {
        let repo = MetadataIndexRepository::new();
        let query = SearchQuery::new("test", false, false, false).unwrap();
        
        let results = repo.search(&query).unwrap();
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_hash_conversion() {
        let repo = MetadataIndexRepository::new();
        let path = FilePath::new("hashed.rs").unwrap();
        
        let entry = AppFileIndexEntry::with_hash(
            "hashed.rs".to_string(),
            1000,
            100,
            "deadbeef12345678".to_string(),
        );
        repo.update_metadata(entry).unwrap();
        
        let retrieved = repo.get_metadata(&path).unwrap();
        assert_eq!(retrieved.content_hash, Some("deadbeef12345678".to_string()));
    }
}
