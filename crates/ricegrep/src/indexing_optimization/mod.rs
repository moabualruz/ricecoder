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

pub use delta_index::{DeltaIndexError, DeltaLog, DeltaLogStats, DeltaOp, IndexDeltaEntry};

pub use event_debouncing::{DebounceBuffer, DebouncingStats, FileChangeEvent, FileChangeKind};

pub use hash_cache::{CacheStats as HashCacheStats, ContentHash, ContentHashCache, HashCacheError};

pub use metadata_gating::{
    CacheStats, ChangeReason, FileChangeFilter, FileIndexEntry, FileMetadataCache, FilterResult,
    MetadataGatingError, MetadataStore, MetadataStoreStats,
};
