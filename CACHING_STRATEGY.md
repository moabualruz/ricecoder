# Caching Strategy for RiceCoder

**Status**: Phase 4 - Performance Optimization (Task 24.2)

**Date**: December 5, 2025

**Purpose**: Document caching strategies and implementation patterns for performance optimization

---

## Overview

RiceCoder uses a file-based cache manager (`ricecoder_storage::CacheManager`) to cache expensive computations and I/O operations. This document describes caching strategies and implementation patterns.

---

## Cache Manager API

The `CacheManager` provides a simple key-value cache with TTL and manual invalidation support.

### Basic Usage

```rust
use ricecoder_storage::{CacheManager, CacheInvalidationStrategy};

// Create cache manager
let cache = CacheManager::new("/path/to/cache")?;

// Set a cached value with TTL (3600 seconds = 1 hour)
cache.set(
    "config_key",
    config_json,
    CacheInvalidationStrategy::Ttl(3600),
)?;

// Get a cached value
if let Some(cached_config) = cache.get("config_key")? {
    // Use cached value
} else {
    // Compute and cache
}

// Invalidate a cached value
cache.invalidate("config_key")?;

// Check if key exists and is not expired
if cache.exists("config_key")? {
    // Use cached value
}

// Clear all cache
cache.clear()?;

// Clean up expired entries
let cleaned = cache.cleanup_expired()?;
```

---

## Caching Strategies

### 1. Configuration Caching

**What**: Cache parsed configuration files

**Why**: Configuration parsing is expensive (YAML/JSON parsing, validation)

**TTL**: 3600 seconds (1 hour) - reload on file change or after 1 hour

**Implementation**:

```rust
use ricecoder_storage::{CacheManager, CacheInvalidationStrategy};
use std::path::Path;

pub struct ConfigCache {
    cache: CacheManager,
}

impl ConfigCache {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
        })
    }

    pub fn get_config(&self, config_path: &Path) -> Result<Config> {
        let cache_key = format!("config_{}", config_path.display());

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key)? {
            return Ok(serde_json::from_str(&cached)?);
        }

        // Load and parse config
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;

        // Cache for 1 hour
        let json = serde_json::to_string(&config)?;
        self.cache.set(
            &cache_key,
            json,
            CacheInvalidationStrategy::Ttl(3600),
        )?;

        Ok(config)
    }

    pub fn invalidate_config(&self, config_path: &Path) -> Result<()> {
        let cache_key = format!("config_{}", config_path.display());
        self.cache.invalidate(&cache_key)?;
        Ok(())
    }
}
```

### 2. Provider Response Caching

**What**: Cache AI provider responses

**Why**: Avoid redundant API calls for same prompts

**TTL**: 86400 seconds (24 hours) - responses are stable

**Implementation**:

```rust
use ricecoder_storage::{CacheManager, CacheInvalidationStrategy};
use sha2::{Sha256, Digest};

pub struct ProviderCache {
    cache: CacheManager,
}

impl ProviderCache {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
        })
    }

    pub async fn get_response(
        &self,
        provider: &str,
        model: &str,
        prompt: &str,
    ) -> Result<Option<String>> {
        let cache_key = self.make_cache_key(provider, model, prompt);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key)? {
            tracing::debug!("Cache hit for provider response");
            return Ok(Some(cached));
        }

        Ok(None)
    }

    pub fn cache_response(
        &self,
        provider: &str,
        model: &str,
        prompt: &str,
        response: &str,
    ) -> Result<()> {
        let cache_key = self.make_cache_key(provider, model, prompt);

        // Cache for 24 hours
        self.cache.set(
            &cache_key,
            response.to_string(),
            CacheInvalidationStrategy::Ttl(86400),
        )?;

        Ok(())
    }

    fn make_cache_key(&self, provider: &str, model: &str, prompt: &str) -> String {
        // Use hash of prompt to avoid long keys
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        format!("provider_{}_{}_{}",provider, model, hash)
    }
}
```

### 3. Spec Parsing Caching

**What**: Cache parsed specification files

**Why**: Spec parsing is expensive (YAML parsing, validation, context building)

**TTL**: 3600 seconds (1 hour) - reload on file change

**Implementation**:

```rust
use ricecoder_storage::{CacheManager, CacheInvalidationStrategy};

pub struct SpecCache {
    cache: CacheManager,
}

impl SpecCache {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
        })
    }

    pub fn get_spec(&self, spec_path: &Path) -> Result<Spec> {
        let cache_key = format!("spec_{}", spec_path.display());

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key)? {
            return Ok(serde_json::from_str(&cached)?);
        }

        // Parse spec
        let spec = parse_spec_file(spec_path)?;

        // Cache for 1 hour
        let json = serde_json::to_string(&spec)?;
        self.cache.set(
            &cache_key,
            json,
            CacheInvalidationStrategy::Ttl(3600),
        )?;

        Ok(spec)
    }

    pub fn invalidate_spec(&self, spec_path: &Path) -> Result<()> {
        let cache_key = format!("spec_{}", spec_path.display());
        self.cache.invalidate(&cache_key)?;
        Ok(())
    }
}
```

### 4. Project Analysis Caching

**What**: Cache project structure analysis results

**Why**: Project analysis is expensive (file tree traversal, dependency parsing)

**TTL**: 3600 seconds (1 hour) - reload on file change

**Implementation**:

```rust
use ricecoder_storage::{CacheManager, CacheInvalidationStrategy};

pub struct ProjectAnalysisCache {
    cache: CacheManager,
}

impl ProjectAnalysisCache {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
        })
    }

    pub fn get_analysis(&self, project_path: &Path) -> Result<Option<ProjectAnalysis>> {
        let cache_key = format!("analysis_{}", project_path.display());

        if let Some(cached) = self.cache.get(&cache_key)? {
            return Ok(Some(serde_json::from_str(&cached)?));
        }

        Ok(None)
    }

    pub fn cache_analysis(
        &self,
        project_path: &Path,
        analysis: &ProjectAnalysis,
    ) -> Result<()> {
        let cache_key = format!("analysis_{}", project_path.display());

        // Cache for 1 hour
        let json = serde_json::to_string(analysis)?;
        self.cache.set(
            &cache_key,
            json,
            CacheInvalidationStrategy::Ttl(3600),
        )?;

        Ok(())
    }

    pub fn invalidate_analysis(&self, project_path: &Path) -> Result<()> {
        let cache_key = format!("analysis_{}", project_path.display());
        self.cache.invalidate(&cache_key)?;
        Ok(())
    }
}
```

---

## Cache Statistics and Monitoring

### Cache Hit Rate Tracking

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct CacheStats {
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn stats(&self) -> (u64, u64, f64) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let rate = self.hit_rate();
        (hits, misses, rate)
    }
}
```

### Logging Cache Operations

```rust
use tracing::{debug, info};

pub fn log_cache_stats(stats: &CacheStats) {
    let (hits, misses, rate) = stats.stats();
    info!(
        "Cache statistics: {} hits, {} misses, {:.2}% hit rate",
        hits,
        misses,
        rate * 100.0
    );
}
```

---

## Cache Invalidation Strategies

### 1. Time-Based (TTL)

**When**: Data changes infrequently (configs, specs, analysis)

**TTL Values**:
- Configuration: 3600 seconds (1 hour)
- Specs: 3600 seconds (1 hour)
- Project analysis: 3600 seconds (1 hour)
- Provider responses: 86400 seconds (24 hours)

**Pros**: Automatic cleanup, simple to implement

**Cons**: Stale data possible, requires TTL tuning

### 2. Manual Invalidation

**When**: Data changes on demand (user edits, file changes)

**Implementation**:

```rust
// Invalidate on file change
pub fn on_file_changed(&self, path: &Path) {
    let cache_key = format!("spec_{}", path.display());
    let _ = self.cache.invalidate(&cache_key);
}

// Invalidate on user action
pub fn on_config_updated(&self) {
    let _ = self.cache.invalidate("config_global");
    let _ = self.cache.invalidate("config_project");
}
```

**Pros**: Always fresh data, no stale data

**Cons**: Requires explicit invalidation, more complex

### 3. Hybrid Approach

**When**: Combine TTL and manual invalidation

**Implementation**:

```rust
pub struct HybridCache {
    cache: CacheManager,
}

impl HybridCache {
    pub fn get_with_invalidation(
        &self,
        key: &str,
        file_path: &Path,
    ) -> Result<Option<String>> {
        // Check if file was modified since cache creation
        let metadata = std::fs::metadata(file_path)?;
        let modified = metadata.modified()?;

        // If file was modified, invalidate cache
        if let Ok(cached) = self.cache.get(key) {
            if cached.is_some() {
                // Check file modification time
                // If newer than cache, invalidate
                let _ = self.cache.invalidate(key);
                return Ok(None);
            }
        }

        self.cache.get(key)
    }
}
```

---

## Cache Cleanup

### Periodic Cleanup

```rust
use tokio::time::{interval, Duration};

pub async fn start_cache_cleanup(cache: Arc<CacheManager>) {
    let mut interval = interval(Duration::from_secs(3600)); // Every hour

    loop {
        interval.tick().await;

        match cache.cleanup_expired() {
            Ok(cleaned) => {
                tracing::info!("Cleaned up {} expired cache entries", cleaned);
            }
            Err(e) => {
                tracing::warn!("Failed to cleanup cache: {}", e);
            }
        }
    }
}
```

### Manual Cleanup

```rust
// Clear all cache
cache.clear()?;

// Clean up only expired entries
let cleaned = cache.cleanup_expired()?;
```

---

## Performance Impact

### Expected Improvements

| Operation | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Config loading | 500ms | 50ms | 10x |
| Spec parsing | 1000ms | 100ms | 10x |
| Project analysis | 2000ms | 200ms | 10x |
| Provider response | 5000ms | 50ms | 100x |

### Cache Size Estimates

| Cache Type | Typical Size | Max Size |
|-----------|--------------|----------|
| Configuration | 10KB | 100KB |
| Specs | 50KB | 500KB |
| Project analysis | 100KB | 1MB |
| Provider responses | 500KB | 5MB |

---

## Best Practices

1. **Use appropriate TTLs**: Balance freshness vs. performance
2. **Monitor cache hit rates**: Adjust TTLs based on actual usage
3. **Clean up expired entries**: Run periodic cleanup to save disk space
4. **Invalidate on changes**: Manually invalidate when data changes
5. **Log cache operations**: Track cache performance for optimization
6. **Test cache behavior**: Ensure correctness with caching enabled

---

## Testing Cache Behavior

### Unit Tests

```rust
#[test]
fn test_cache_hit_rate() -> Result<()> {
    let cache = CacheManager::new(temp_dir.path())?;
    let stats = CacheStats::new();

    // Populate cache
    cache.set("key1", "data1".to_string(), CacheInvalidationStrategy::Manual)?;

    // First access: miss
    if cache.get("key1")?.is_none() {
        stats.record_miss();
    } else {
        stats.record_hit();
    }

    // Second access: hit
    if cache.get("key1")?.is_none() {
        stats.record_miss();
    } else {
        stats.record_hit();
    }

    assert_eq!(stats.hit_rate(), 0.5); // 50% hit rate

    Ok(())
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_cache_with_file_changes() -> Result<()> {
    let cache = CacheManager::new(temp_dir.path())?;
    let config_path = temp_dir.path().join("config.yaml");

    // Write initial config
    std::fs::write(&config_path, "key: value1")?;

    // Cache config
    let config1 = load_and_cache_config(&cache, &config_path)?;
    assert_eq!(config1.key, "value1");

    // Update config file
    std::fs::write(&config_path, "key: value2")?;

    // Invalidate cache
    cache.invalidate("config")?;

    // Load updated config
    let config2 = load_and_cache_config(&cache, &config_path)?;
    assert_eq!(config2.key, "value2");

    Ok(())
}
```

---

## References

- [Cache Manager API](../crates/ricecoder-storage/src/cache/manager.rs)
- [Performance Profiling Guide](./PERFORMANCE_PROFILING_GUIDE.md)
- [Caching Best Practices](https://en.wikipedia.org/wiki/Cache_(computing))

---

*Last updated: December 5, 2025*
