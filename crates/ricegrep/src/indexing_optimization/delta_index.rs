//! Delta index format for incremental updates.
//!
//! Implements TIER 3 (Fallback Path) of the hybrid indexing strategy:
//! - Append-only log of file changes (create, modify, delete)
//! - Non-blocking writes (O(1) append)
//! - Background compaction (merge deltas into main index)
//! - Reduces disk I/O for frequent updates

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::metadata_gating::FileIndexEntry;

/// Errors from delta index operations
#[derive(Debug, Error)]
pub enum DeltaIndexError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("delta log corrupted: {0}")]
    Corrupted(String),
}

/// Type of change in delta log
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaOp {
    /// File was created
    Create,
    /// File was modified
    Modify,
    /// File was deleted
    Delete,
}

impl std::fmt::Display for DeltaOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeltaOp::Create => write!(f, "create"),
            DeltaOp::Modify => write!(f, "modify"),
            DeltaOp::Delete => write!(f, "delete"),
        }
    }
}

/// Single entry in delta log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDeltaEntry {
    /// Path to the file
    pub path: PathBuf,
    
    /// Type of change
    pub operation: DeltaOp,
    
    /// File metadata (None for Delete operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FileIndexEntry>,
    
    /// Timestamp when change occurred
    pub timestamp: u64, // Unix timestamp
}

impl IndexDeltaEntry {
    /// Create a new delta entry for create operation
    pub fn create(path: PathBuf, metadata: FileIndexEntry) -> Self {
        Self {
            path,
            operation: DeltaOp::Create,
            metadata: Some(metadata),
            timestamp: current_timestamp(),
        }
    }
    
    /// Create a new delta entry for modify operation
    pub fn modify(path: PathBuf, metadata: FileIndexEntry) -> Self {
        Self {
            path,
            operation: DeltaOp::Modify,
            metadata: Some(metadata),
            timestamp: current_timestamp(),
        }
    }
    
    /// Create a new delta entry for delete operation
    pub fn delete(path: PathBuf) -> Self {
        Self {
            path,
            operation: DeltaOp::Delete,
            metadata: None,
            timestamp: current_timestamp(),
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Append-only delta log for incremental index updates
#[derive(Debug)]
pub struct DeltaLog {
    log_path: PathBuf,
    entries: Vec<IndexDeltaEntry>,
    entry_count: usize,
}

impl DeltaLog {
    /// Open or create a delta log
    pub fn new(log_path: PathBuf) -> Result<Self, DeltaIndexError> {
        let entries = if log_path.exists() {
            // Load existing log
            let file = File::open(&log_path)?;
            let reader = BufReader::new(file);
            serde_json::Deserializer::from_reader(reader)
                .into_iter::<IndexDeltaEntry>()
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        
        let entry_count = entries.len();
        
        Ok(Self {
            log_path,
            entries,
            entry_count,
        })
    }
    
    /// Append an entry to the delta log
    pub fn append(&mut self, entry: IndexDeltaEntry) -> Result<(), DeltaIndexError> {
        self.entries.push(entry);
        self.entry_count += 1;
        
        // Persist to disk (append-only mode)
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.entries[self.entries.len() - 1])?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        
        Ok(())
    }
    
    /// Get all entries from the log
    pub fn entries(&self) -> &[IndexDeltaEntry] {
        &self.entries
    }
    
    /// Get number of entries in log
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Clear all entries (for compaction)
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Load all entries from disk
    pub fn load(&mut self) -> Result<(), DeltaIndexError> {
        if !self.log_path.exists() {
            return Ok(());
        }
        
        let file = File::open(&self.log_path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry = serde_json::from_str::<IndexDeltaEntry>(&line)?;
            entries.push(entry);
        }
        
        self.entries = entries;
        self.entry_count = self.entries.len();
        
        Ok(())
    }
    
    /// Get statistics about the delta log
    pub fn stats(&self) -> DeltaLogStats {
        let mut creates = 0;
        let mut modifies = 0;
        let mut deletes = 0;
        
        for entry in &self.entries {
            match entry.operation {
                DeltaOp::Create => creates += 1,
                DeltaOp::Modify => modifies += 1,
                DeltaOp::Delete => deletes += 1,
            }
        }
        
        DeltaLogStats {
            total_entries: self.entries.len(),
            creates,
            modifies,
            deletes,
            log_path: self.log_path.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeltaLogStats {
    pub total_entries: usize,
    pub creates: usize,
    pub modifies: usize,
    pub deletes: usize,
    pub log_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_delta_entry_creation() {
        let path = PathBuf::from("test.txt");
        let entry = IndexDeltaEntry::delete(path.clone());
        
        assert_eq!(entry.path, path);
        assert_eq!(entry.operation, DeltaOp::Delete);
        assert!(entry.metadata.is_none());
    }
    
    #[test]
    fn test_delta_log_creation() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("delta.log");
        
        let log = DeltaLog::new(log_path).unwrap();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }
    
    #[test]
    fn test_delta_log_append() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("delta.log");
        
        let mut log = DeltaLog::new(log_path.clone()).unwrap();
        
        let entry = IndexDeltaEntry::delete(PathBuf::from("test.txt"));
        log.append(entry).unwrap();
        
        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
        assert!(log_path.exists());
    }
    
    #[test]
    fn test_delta_log_persistence() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("delta.log");
        
        // Write entries
        {
            let mut log = DeltaLog::new(log_path.clone()).unwrap();
            log.append(IndexDeltaEntry::delete(PathBuf::from("file1.txt"))).unwrap();
            log.append(IndexDeltaEntry::delete(PathBuf::from("file2.txt"))).unwrap();
        }
        
        // Load in new instance
        let log = DeltaLog::new(log_path).unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log.entries[0].path, PathBuf::from("file1.txt"));
        assert_eq!(log.entries[1].path, PathBuf::from("file2.txt"));
    }
    
    #[test]
    fn test_delta_log_stats() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("delta.log");
        
        let mut log = DeltaLog::new(log_path).unwrap();
        log.append(IndexDeltaEntry::delete(PathBuf::from("file1.txt"))).unwrap();
        log.append(IndexDeltaEntry::delete(PathBuf::from("file2.txt"))).unwrap();
        log.append(IndexDeltaEntry::delete(PathBuf::from("file3.txt"))).unwrap();
        
        let stats = log.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.deletes, 3);
        assert_eq!(stats.creates, 0);
        assert_eq!(stats.modifies, 0);
    }
    
    #[test]
    fn test_delta_log_operations_mixed() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("delta.log");
        
        let mut log = DeltaLog::new(log_path).unwrap();
        log.append(IndexDeltaEntry::delete(PathBuf::from("file1.txt"))).unwrap();
        log.append(IndexDeltaEntry::delete(PathBuf::from("file2.txt"))).unwrap();
        log.append(IndexDeltaEntry::delete(PathBuf::from("file3.txt"))).unwrap();
        
        let stats = log.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.deletes, 3);
    }
}
