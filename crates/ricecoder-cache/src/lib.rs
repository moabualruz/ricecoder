//! # RiceCoder Cache
//!
//! Multi-level caching infrastructure for RiceCoder with intelligent invalidation,
//! performance monitoring, and support for memory, disk, and remote caching.
//!
//! ## Features
//!
//! - **Multi-level caching**: Memory, disk, and remote cache support
//! - **Intelligent invalidation**: TTL, file change detection, and custom strategies
//! - **Performance monitoring**: Detailed metrics and statistics
//! - **Async operations**: Full async/await support
//! - **Generic data types**: Support for any serializable data
//! - **Cache hierarchies**: L1/L2/L3 cache configurations

pub mod cache;
pub mod error;
pub mod metrics;
pub mod storage;
pub mod strategy;

pub use cache::{Cache, CacheBuilder, CacheConfig};
pub use error::CacheError;
pub use metrics::{CacheMetrics, CacheStats};
pub use storage::{CacheEntry, CacheStorage, DiskStorage, MemoryStorage};
pub use strategy::{CacheStrategy, FileChangeStrategy, TtlStrategy};

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, CacheError>;
