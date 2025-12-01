//! Cache management for RiceCoder storage
//!
//! Provides caching abstractions and file-based cache storage for performance optimization.

pub mod manager;

// Re-export commonly used types
pub use manager::{CacheManager, CacheEntry, CacheInvalidationStrategy};
