# RiceCoder Cache

Multi-level caching infrastructure for RiceCoder with intelligent invalidation, performance monitoring, and support for memory, disk, and remote caching.

## Features

- **Multi-level caching**: L1 (memory), L2 (disk), L3 (remote) cache hierarchy
- **Intelligent invalidation**: TTL, file change detection, and custom strategies
- **Performance monitoring**: Detailed metrics and statistics tracking
- **Async operations**: Full async/await support with tokio
- **Generic data types**: Support for any serializable data
- **Builder pattern**: Flexible cache construction and configuration
- **Strategy pattern**: Pluggable invalidation strategies

## Usage

### Basic Cache Usage

```rust
use ricecoder_cache::{Cache, CacheBuilder};
use ricecoder_cache::storage::MemoryStorage;
use std::sync::Arc;

// Create a simple memory cache
let storage = Arc::new(MemoryStorage::new());
let cache = Cache::new(storage);

// Store and retrieve data
cache.set("user:123", "John Doe", None).await?;
let name: Option<String> = cache.get("user:123").await?;
assert_eq!(name, Some("John Doe".to_string()));
```

### Multi-Level Cache with TTL

```rust
use ricecoder_cache::{CacheConfig, CacheBuilder};
use ricecoder_cache::storage::{MemoryStorage, DiskStorage};
use ricecoder_cache::strategy::TtlStrategy;
use std::time::Duration;

// Configure cache with TTL strategy
let config = CacheConfig {
    default_ttl: Some(Duration::from_secs(3600)), // 1 hour
    max_entries: Some(1000),
    enable_metrics: true,
    ..Default::default()
};

let cache = CacheBuilder::new()
    .config(config)
    .primary_storage(Arc::new(MemoryStorage::new()))
    .secondary_storage(Arc::new(DiskStorage::new("./cache")))
    .strategy(Arc::new(TtlStrategy::with_hours(1)))
    .build()?;

// Cache automatically handles TTL expiration
```

### File Change Detection

```rust
use ricecoder_cache::strategy::{FileChangeStrategy, CompositeStrategy};

// Create strategy that invalidates on file changes
let mut file_strategy = FileChangeStrategy::new();
file_strategy.monitor_file("src/main.rs")?;
file_strategy.monitor_file("Cargo.toml")?;

// Combine with TTL strategy
let composite_strategy = CompositeStrategy::new()
    .add_strategy(TtlStrategy::with_hours(24))
    .add_strategy(file_strategy);

let cache = CacheBuilder::new()
    .primary_storage(Arc::new(MemoryStorage::new()))
    .strategy(Arc::new(composite_strategy))
    .build()?;
```

### Performance Monitoring

```rust
// Get cache statistics
let stats = cache.stats();
println!("Hit rate: {:.2}%", stats.hit_rate());
println!("Total operations: {}", stats.hits + stats.misses);
println!("Avg retrieval time: {:.2}ms", stats.avg_retrieval_time_ms);

// Get detailed metrics
let metrics = cache.metrics();
println!("{}", metrics.summary());
```

## Cache Strategies

### Built-in Strategies

- **TTL Strategy**: Time-based expiration
- **File Change Strategy**: Invalidates when monitored files change
- **LRU Strategy**: Least recently used eviction
- **Size Limit Strategy**: Evicts when cache size exceeds limit
- **Composite Strategy**: Combines multiple strategies

### Custom Strategies

```rust
use ricecoder_cache::strategy::CacheStrategy;
use async_trait::async_trait;

#[derive(Debug)]
struct CustomStrategy;

#[async_trait]
impl CacheStrategy for CustomStrategy {
    async fn should_invalidate(
        &self,
        key: &str,
        created_at: std::time::SystemTime,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // Custom invalidation logic
        Ok(key.starts_with("temp:"))
    }

    fn name(&self) -> &str {
        "custom"
    }
}
```

## Storage Backends

### Memory Storage
Fast in-memory storage using HashMap.

### Disk Storage
Persistent disk storage with file-based entries.

### Custom Storage
Implement the `CacheStorage` trait for custom backends:

```rust
use ricecoder_cache::storage::CacheStorage;
use async_trait::async_trait;

#[derive(Debug)]
struct RedisStorage {
    // Redis connection
}

#[async_trait]
impl CacheStorage for RedisStorage {
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        entry: CacheEntry<T>,
    ) -> Result<()> {
        // Implement Redis storage
        Ok(())
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<CacheEntry<T>>> {
        // Implement Redis retrieval
        Ok(None)
    }

    // ... implement other methods
}
```

## Configuration

```rust
use ricecoder_cache::CacheConfig;
use std::time::Duration;

let config = CacheConfig {
    default_ttl: Some(Duration::from_secs(1800)), // 30 minutes
    max_entries: Some(5000),
    max_size_bytes: Some(100 * 1024 * 1024), // 100MB
    enable_metrics: true,
};
```

## Performance Benchmarks

Run benchmarks with:

```bash
cargo bench
```

Benchmarks include:
- Cache hit/miss performance
- Multi-level cache promotion
- Strategy evaluation overhead
- Storage backend performance

## Testing

```bash
cargo test
```

Tests cover:
- Basic cache operations
- Multi-level caching
- Strategy implementations
- Metrics collection
- Storage backends
- Edge cases and error conditions

## Architecture

The cache follows a hierarchical design:

```
┌─────────────────┐
│   Application   │
└─────────────────┘
         │
    ┌────────────┐
    │   Cache    │ ← Multi-level cache manager
    └────────────┘
         │
    ┌────┼────┐
    │    │    │
┌─────┐ ┌─────┐ ┌─────┐
│  L1 │ │  L2 │ │  L3 │ ← Storage backends
│ Mem │ │Disk │ │Remote│
└─────┘ └─────┘ └─────┘
         │
    ┌────────────┐
    │ Strategies │ ← Invalidation strategies
    └────────────┘
         │
    ┌────────────┐
    │  Metrics   │ ← Performance monitoring
    └────────────┘
```

## Dependencies

- `tokio`: Async runtime
- `serde`: Serialization
- `async-trait`: Async trait support
- `thiserror`: Error handling

## License

MIT OR Apache-2.0