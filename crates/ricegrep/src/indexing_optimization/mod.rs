//! Performance optimization module for incremental indexing.
//!
//! This module implements strategies to speed up watch mode through:
//! - Metadata gating (mtime + size checking) 
//! - Event debouncing (batch file changes)
//! - Delta index format (append-only logs)
//! - Hash caching (content hash LRU cache)

pub mod delta_index;
pub mod event_debouncing;
pub mod hash_cache;
pub mod metadata_gating;

pub use delta_index::{
    DeltaLog, IndexDeltaEntry, DeltaOp, DeltaLogStats, DeltaIndexError,
};

pub use event_debouncing::{
    DebounceBuffer, FileChangeEvent, FileChangeKind, DebouncingStats,
};

pub use hash_cache::{
    ContentHash, ContentHashCache, HashCacheError, CacheStats as HashCacheStats,
};

pub use metadata_gating::{
    FileIndexEntry, FileMetadataCache, MetadataGatingError, ChangeReason,
    MetadataStore, MetadataStoreStats, CacheStats,
    FileChangeFilter, FilterResult,
};
