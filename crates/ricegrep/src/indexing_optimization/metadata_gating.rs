//! Metadata gating for incremental indexing.
//!
//! Implements TIER 1 (Fast Path) of the hybrid indexing strategy:
//! - Store mtime + size metadata per indexed file
//! - Before re-indexing, check if mtime/size changed
//! - Skip files with unchanged metadata (<1ms per check)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from metadata gating operations
#[derive(Debug, Error)]
pub enum MetadataGatingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Reasons why a file should be re-indexed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeReason {
    /// File modification time changed
    Mtime { old: u64, new: u64 },
    
    /// File size changed
    Size { old: u64, new: u64 },
    
    /// Both modification time and size changed
    MtimeAndSize {
        old_mtime: u64,
        new_mtime: u64,
        old_size: u64,
        new_size: u64,
    },
    
    /// File was deleted
    Deleted,
}

impl ChangeReason {
    /// Human-readable description of the change
    pub fn description(&self) -> String {
        match self {
            ChangeReason::Mtime { old, new } => {
                format!("mtime changed: {} → {}", old, new)
            }
            ChangeReason::Size { old, new } => {
                format!("size changed: {} → {}", old, new)
            }
            ChangeReason::MtimeAndSize { old_mtime, new_mtime, old_size, new_size } => {
                format!("mtime and size changed: mtime {} → {}, size {} → {}", 
                    old_mtime, new_mtime, old_size, new_size)
            }
            ChangeReason::Deleted => "file deleted".to_string(),
        }
    }
}

/// Metadata for a single indexed file
/// 
/// Used to determine if re-indexing is necessary without re-reading file content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileIndexEntry {
    /// Path to the file
    pub path: PathBuf,
    
    /// Unix timestamp when file was indexed
    pub indexed_at: u64,
    
    /// File's mtime (modification time) in seconds since epoch
    pub file_mtime: u64,
    
    /// File size in bytes
    pub file_size: u64,
    
    /// Optional content hash (fallback for race conditions)
    pub content_hash: Option<u64>,
    
    /// Soft delete flag (doesn't physically remove, just marks obsolete)
    pub is_deleted: bool,
}

impl FileIndexEntry {
    /// Create a new FileIndexEntry from current file metadata
    pub fn from_file(path: PathBuf) -> Result<Self, MetadataGatingError> {
        let metadata = std::fs::metadata(&path)?;
        let modified = metadata.modified()?;
        let mtime = modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| MetadataGatingError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("time error: {}", e)
            )))?
            .as_secs();
        
        Ok(FileIndexEntry {
            path,
            indexed_at: current_timestamp(),
            file_mtime: mtime,
            file_size: metadata.len(),
            content_hash: None,
            is_deleted: false,
        })
    }
    
    /// Check if this entry matches current file metadata (unchanged)
    pub fn matches_current_file(&self) -> Result<bool, MetadataGatingError> {
        let metadata = std::fs::metadata(&self.path)?;
        let modified = metadata.modified()?;
        let current_mtime = modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| MetadataGatingError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("time error: {}", e)
            )))?
            .as_secs();
        
        // Both mtime AND size must match for "unchanged"
        Ok(self.file_mtime == current_mtime && self.file_size == metadata.len())
    }
    
    /// Determine if a file should be re-indexed based on metadata changes
    /// 
    /// Returns a ChangeReason enum describing what changed (or None if unchanged)
    /// This is used to decide whether to re-index a file during watch mode
    pub fn should_reindex(&self) -> Result<Option<ChangeReason>, MetadataGatingError> {
        match std::fs::metadata(&self.path) {
            Ok(metadata) => {
                let modified = metadata.modified()?;
                let current_mtime = modified
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|e| MetadataGatingError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("time error: {}", e)
                    )))?
                    .as_secs();
                
                let current_size = metadata.len();
                
                if self.file_mtime != current_mtime && self.file_size != current_size {
                    // Both changed
                    Ok(Some(ChangeReason::MtimeAndSize {
                        old_mtime: self.file_mtime,
                        new_mtime: current_mtime,
                        old_size: self.file_size,
                        new_size: current_size,
                    }))
                } else if self.file_mtime != current_mtime {
                    // Only mtime changed
                    Ok(Some(ChangeReason::Mtime {
                        old: self.file_mtime,
                        new: current_mtime,
                    }))
                } else if self.file_size != current_size {
                    // Only size changed
                    Ok(Some(ChangeReason::Size {
                        old: self.file_size,
                        new: current_size,
                    }))
                } else {
                    // Nothing changed - no need to re-index
                    Ok(None)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File was deleted
                Ok(Some(ChangeReason::Deleted))
            }
            Err(e) => Err(MetadataGatingError::Io(e)),
        }
    }
}

/// LRU cache for file metadata
/// 
/// Avoids repeated metadata reads for unchanged files
/// Stores mtime, size, and optional hash for triple-validation
#[derive(Debug)]
pub struct FileMetadataCache {
    cache: HashMap<PathBuf, CachedMetadata>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
struct CachedMetadata {
    mtime: u64,
    size: u64,
    hash: Option<u64>,
    cached_at: u64,
}

impl FileMetadataCache {
    /// Create new cache with given size limit
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries: max_entries.max(1),
        }
    }
    
    /// Add entry to cache
    pub fn add(&mut self, path: PathBuf, mtime: u64, size: u64, hash: Option<u64>) {
        if self.cache.len() >= self.max_entries {
            // Simple eviction: remove oldest entry
            if let Some(oldest) = self.cache.iter()
                .min_by_key(|(_, m)| m.cached_at)
                .map(|(p, _)| p.clone())
            {
                self.cache.remove(&oldest);
            }
        }
        
        self.cache.insert(path, CachedMetadata {
            mtime,
            size,
            hash,
            cached_at: current_timestamp(),
        });
    }
    
    /// Get cached metadata with triple-validation
    /// Returns Some((hash)) if cache hit, None if miss or validation failed
    pub fn get_if_valid(&self, path: &Path, current_mtime: u64, current_size: u64) -> Option<Option<u64>> {
        self.cache.get(path).and_then(|cached| {
            // Triple validation: mtime + size + recent timestamp
            if cached.mtime == current_mtime && 
               cached.size == current_size &&
               is_recent(cached.cached_at, 60) {  // 60 second window
                Some(cached.hash)
            } else {
                None
            }
        })
    }
    
    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            capacity: self.max_entries,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub capacity: usize,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_recent(timestamp: u64, threshold_secs: u64) -> bool {
    let now = current_timestamp();
    now.saturating_sub(timestamp) < threshold_secs
}

/// Stores and retrieves file metadata for incremental indexing
/// 
/// Persists metadata to disk (JSON format) to survives process restarts
/// Provides thread-safe access via mutex
#[derive(Debug)]
pub struct MetadataStore {
    entries: std::sync::Mutex<HashMap<PathBuf, FileIndexEntry>>,
    storage_path: PathBuf,
    auto_save: bool,
}

impl MetadataStore {
    /// Create new metadata store
    /// 
    /// # Arguments
    /// * `storage_path` - Path to JSON file where metadata will be persisted
    /// * `auto_save` - Whether to auto-save to disk after each modification
    pub fn new(storage_path: PathBuf, auto_save: bool) -> Self {
        Self {
            entries: std::sync::Mutex::new(HashMap::new()),
            storage_path,
            auto_save,
        }
    }
    
    /// Load metadata from disk
    pub fn load(&self) -> Result<(), MetadataGatingError> {
        if !self.storage_path.exists() {
            // No existing metadata file - start fresh
            return Ok(());
        }
        
        let content = std::fs::read_to_string(&self.storage_path)?;
        let entries: HashMap<PathBuf, FileIndexEntry> = serde_json::from_str(&content)
            .map_err(|e| MetadataGatingError::Serialization(e.to_string()))?;
        
        let mut store = self.entries.lock().unwrap();
        *store = entries;
        
        Ok(())
    }
    
    /// Save metadata to disk
    pub fn save(&self) -> Result<(), MetadataGatingError> {
        let store = self.entries.lock().unwrap();
        let content = serde_json::to_string_pretty(&*store)
            .map_err(|e| MetadataGatingError::Serialization(e.to_string()))?;
        
        std::fs::write(&self.storage_path, content)?;
        Ok(())
    }
    
    /// Store metadata for a file
    pub fn store(&self, entry: FileIndexEntry) -> Result<(), MetadataGatingError> {
        {
            let mut store = self.entries.lock().unwrap();
            store.insert(entry.path.clone(), entry);
        }
        
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// Get metadata for a file
    pub fn get(&self, path: &Path) -> Option<FileIndexEntry> {
        let store = self.entries.lock().unwrap();
        store.get(path).cloned()
    }
    
    /// Remove metadata for a file
    pub fn remove(&self, path: &Path) -> Result<Option<FileIndexEntry>, MetadataGatingError> {
        let entry = {
            let mut store = self.entries.lock().unwrap();
            store.remove(path)
        };
        
        if self.auto_save && entry.is_some() {
            self.save()?;
        }
        
        Ok(entry)
    }
    
    /// Check if file needs re-indexing based on stored metadata
    /// 
    /// Returns Some(ChangeReason) if re-indexing needed, None if unchanged
    pub fn check_reindex(&self, path: &Path) -> Result<Option<ChangeReason>, MetadataGatingError> {
        if let Some(stored_entry) = self.get(path) {
            stored_entry.should_reindex()
        } else {
            // No stored metadata - file is new
            Ok(Some(ChangeReason::Deleted)) // Use Deleted to indicate "unknown state"
        }
    }
    
    /// Get all stored metadata entries
    pub fn all_entries(&self) -> Vec<FileIndexEntry> {
        let store = self.entries.lock().unwrap();
        store.values().cloned().collect()
    }
    
    /// Clear all metadata
    pub fn clear(&self) -> Result<(), MetadataGatingError> {
        {
            let mut store = self.entries.lock().unwrap();
            store.clear();
        }
        
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
/// Get metadata store statistics
pub fn stats(&self) -> MetadataStoreStats {
        let store = self.entries.lock().unwrap();
        let total_size: u64 = store.values().map(|e| e.file_size).sum();
        
        MetadataStoreStats {
            entry_count: store.len(),
            total_file_size: total_size,
            storage_path: self.storage_path.clone(),
        }
    }
}

/// Filter files to determine which ones actually need re-indexing
/// 
/// Applies metadata gating to skip unchanged files based on stored metadata
#[derive(Debug)]
pub struct FileChangeFilter {
    store: MetadataStore,
}

impl FileChangeFilter {
    /// Create new file change filter
    pub fn new(metadata_store: MetadataStore) -> Self {
        Self {
            store: metadata_store,
        }
    }
    
    /// Filter files: returns only those that need re-indexing
    /// 
    /// Also tracks skipped files and reasons why
    pub fn filter_changes(&self, changed_files: &[PathBuf]) -> FilterResult {
        let mut files_to_reindex = Vec::new();
        let mut skipped_files = Vec::new();
        
        for file_path in changed_files {
            match self.store.check_reindex(file_path) {
                Ok(Some(reason)) => {
                    files_to_reindex.push(file_path.clone());
                }
                Ok(None) => {
                    skipped_files.push((file_path.clone(), "unchanged metadata".to_string()));
                }
                Err(e) => {
                    // On error, re-index to be safe
                    files_to_reindex.push(file_path.clone());
                    tracing::debug!("Error checking metadata for {}: {}, re-indexing anyway", file_path.display(), e);
                }
            }
        }
        
        FilterResult {
            files_to_reindex,
            skipped_files,
        }
    }
    
    /// Update stored metadata for a file after successful re-indexing
    pub fn update_metadata(&self, file_path: &Path) -> Result<(), MetadataGatingError> {
        let entry = FileIndexEntry::from_file(file_path.to_path_buf())?;
        self.store.store(entry)?;
        Ok(())
    }
    
    /// Update metadata for multiple files
    pub fn update_metadata_batch(&self, files: &[PathBuf]) -> (usize, usize) {
        let mut success_count = 0;
        let mut error_count = 0;
        
        for file_path in files {
            match self.update_metadata(file_path) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    tracing::debug!("Failed to update metadata for {}: {}", file_path.display(), e);
                }
            }
        }
        
        (success_count, error_count)
    }
}

#[derive(Debug)]
pub struct FilterResult {
    pub files_to_reindex: Vec<PathBuf>,
    pub skipped_files: Vec<(PathBuf, String)>,
}

impl FilterResult {
    pub fn reindex_count(&self) -> usize {
        self.files_to_reindex.len()
    }
    
    pub fn skipped_count(&self) -> usize {
        self.skipped_files.len()
    }
    
    pub fn total_count(&self) -> usize {
        self.reindex_count() + self.skipped_count()
    }
}

#[derive(Debug, Clone)]
pub struct MetadataStoreStats {
    pub entry_count: usize,
    pub total_file_size: u64,
    pub storage_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_file_index_entry_creation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();
        
        let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        assert_eq!(entry.path, file_path);
        assert_eq!(entry.file_size, 12); // "test content" = 12 bytes
        assert!(!entry.is_deleted);
    }
    
    #[test]
    fn test_unchanged_file_detection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        assert!(entry.matches_current_file().unwrap());
        
        // Modify file - should no longer match
        fs::write(&file_path, "modified content").unwrap();
        assert!(!entry.matches_current_file().unwrap());
    }
    
    #[test]
    fn test_metadata_cache() {
        let mut cache = FileMetadataCache::new(5);
        
        let path = PathBuf::from("/tmp/test.txt");
        cache.add(path.clone(), 1000, 512, Some(12345));
        
        // Cache hit with matching metadata
        assert_eq!(cache.get_if_valid(&path, 1000, 512), Some(Some(12345)));
        
        // Cache miss: mtime changed
        assert_eq!(cache.get_if_valid(&path, 1001, 512), None);
        
        // Cache miss: size changed
        assert_eq!(cache.get_if_valid(&path, 1000, 513), None);
    }
    
    #[test]
    fn test_cache_eviction() {
        let mut cache = FileMetadataCache::new(2);
        
        cache.add(PathBuf::from("/tmp/file1.txt"), 1000, 100, None);
        cache.add(PathBuf::from("/tmp/file2.txt"), 2000, 200, None);
        assert_eq!(cache.stats().entries, 2);
        
        // Adding third entry should evict oldest
        cache.add(PathBuf::from("/tmp/file3.txt"), 3000, 300, None);
        assert_eq!(cache.stats().entries, 2);
    }
    
    #[test]
    fn test_should_reindex_no_change() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        
        // File unchanged - should return None
        let result = entry.should_reindex().unwrap();
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_should_reindex_mtime_change() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let mut entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        let original_mtime = entry.file_mtime;
        
        // Simulate time passing and file modification
        entry.file_mtime = original_mtime - 10;
        
        let result = entry.should_reindex().unwrap();
        assert!(matches!(result, Some(ChangeReason::Mtime { .. })));
    }
    
    #[test]
    fn test_should_reindex_size_change() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let mut entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        let original_size = entry.file_size;
        
        // Simulate size change
        entry.file_size = original_size + 100;
        
        let result = entry.should_reindex().unwrap();
        assert!(matches!(result, Some(ChangeReason::Size { .. })));
    }
    
    #[test]
    fn test_should_reindex_both_change() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let mut entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        
        // Simulate both changes
        entry.file_mtime -= 5;
        entry.file_size += 50;
        
        let result = entry.should_reindex().unwrap();
        assert!(matches!(result, Some(ChangeReason::MtimeAndSize { .. })));
    }
    
    #[test]
    fn test_should_reindex_deleted_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        
        // Delete the file
        fs::remove_file(&file_path).unwrap();
        
        let result = entry.should_reindex().unwrap();
        assert_eq!(result, Some(ChangeReason::Deleted));
    }
    
    #[test]
    fn test_change_reason_description() {
        let mtime_change = ChangeReason::Mtime { old: 100, new: 200 };
        assert!(mtime_change.description().contains("mtime changed"));
        
        let size_change = ChangeReason::Size { old: 512, new: 1024 };
        assert!(size_change.description().contains("size changed"));
        
        let deleted = ChangeReason::Deleted;
        assert_eq!(deleted.description(), "file deleted");
    }
    
    #[test]
    fn test_metadata_store_creation() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("metadata.json");
        
        let store = MetadataStore::new(store_path.clone(), false);
        assert_eq!(store.stats().entry_count, 0);
    }
    
    #[test]
    fn test_metadata_store_store_and_get() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();
        
        let store_path = dir.path().join("metadata.json");
        let store = MetadataStore::new(store_path, false);
        
        let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        store.store(entry.clone()).unwrap();
        
        let retrieved = store.get(&file_path).unwrap();
        assert_eq!(retrieved, entry);
    }
    
    #[test]
    fn test_metadata_store_persistence() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let store_path = dir.path().join("metadata.json");
        
        // Store metadata
        {
            let store = MetadataStore::new(store_path.clone(), true);
            let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
            store.store(entry).unwrap();
        }
        
        // Verify file was saved
        assert!(store_path.exists());
        
        // Load in new store instance
        let new_store = MetadataStore::new(store_path, false);
        new_store.load().unwrap();
        
        let retrieved = new_store.get(&file_path);
        assert!(retrieved.is_some());
    }
    
    #[test]
    fn test_metadata_store_remove() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();
        
        let store_path = dir.path().join("metadata.json");
        let store = MetadataStore::new(store_path, false);
        
        let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        store.store(entry).unwrap();
        
        assert!(store.get(&file_path).is_some());
        
        store.remove(&file_path).unwrap();
        
        assert!(store.get(&file_path).is_none());
    }
    
    #[test]
    fn test_metadata_store_check_reindex() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();
        
        let store_path = dir.path().join("metadata.json");
        let store = MetadataStore::new(store_path, false);
        
        let entry = FileIndexEntry::from_file(file_path.clone()).unwrap();
        store.store(entry).unwrap();
        
        // File unchanged - should return None
        let result = store.check_reindex(&file_path).unwrap();
        assert_eq!(result, None);
        
        // Modify file
        fs::write(&file_path, "modified").unwrap();
        
        // Should detect change
        let result = store.check_reindex(&file_path).unwrap();
        assert!(result.is_some());
    }
    
    #[test]
    fn test_file_change_filter_basic() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        
        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();
        
        let store_path = dir.path().join("metadata.json");
        let store = MetadataStore::new(store_path, false);
        let filter = FileChangeFilter::new(store);
        
        // Store metadata for file1 (unchanged)
        filter.update_metadata(&file1).unwrap();
        
        // Filter changes: file1 unchanged, file2 new
        let result = filter.filter_changes(&[file1.clone(), file2.clone()]);
        
        assert_eq!(result.reindex_count(), 1); // file2 needs reindex (new)
        assert_eq!(result.skipped_count(), 1);  // file1 skipped (unchanged)
    }
    
    #[test]
    fn test_file_change_filter_update_metadata() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        
        let store_path = dir.path().join("metadata.json");
        let store = MetadataStore::new(store_path, false);
        let filter = FileChangeFilter::new(store);
        
        // Update metadata
        filter.update_metadata(&file_path).unwrap();
        
        // File should now be marked unchanged
        let result = filter.filter_changes(&[file_path.clone()]);
        assert_eq!(result.skipped_count(), 1);
    }
    
    #[test]
    fn test_filter_result_stats() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        
        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();
        
        let store_path = dir.path().join("metadata.json");
        let store = MetadataStore::new(store_path, false);
        let filter = FileChangeFilter::new(store);
        
        filter.update_metadata(&file1).unwrap();
        
        let result = filter.filter_changes(&[file1, file2]);
        
        assert_eq!(result.total_count(), 2);
        assert_eq!(result.reindex_count(), 1);
        assert_eq!(result.skipped_count(), 1);
    }
}
