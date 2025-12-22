//! Caching layer for semantic analysis and AST parsing
//!
//! This module provides caching mechanisms to improve performance by:
//! - Caching parsed ASTs for unchanged documents
//! - Caching semantic analysis results
//! - Caching symbol indexes
//! - Tracking cache hit rates and performance metrics

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use tracing::debug;

use crate::types::SemanticInfo;

/// Cache entry with timestamp and content hash
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// Cached value
    value: T,
    /// Timestamp when entry was created
    timestamp: u64,
    /// Hash of the input that produced this value
    input_hash: u64,
}

/// Performance metrics for caching
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total cache invalidations
    pub invalidations: u64,
    /// Average analysis time in milliseconds
    pub avg_analysis_time_ms: f64,
}

impl CacheMetrics {
    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Semantic analysis cache
pub struct SemanticCache {
    /// Cache entries by document URI
    entries: Arc<RwLock<HashMap<String, CacheEntry<SemanticInfo>>>>,
    /// Performance metrics
    metrics: Arc<RwLock<CacheMetrics>>,
    /// Maximum cache size in bytes (approximate)
    max_size: usize,
    /// Current cache size in bytes (approximate)
    current_size: Arc<RwLock<usize>>,
}

impl SemanticCache {
    /// Create a new semantic cache with default size (100MB)
    pub fn new() -> Self {
        Self::with_size(100 * 1024 * 1024)
    }

    /// Create a new semantic cache with specified size
    pub fn with_size(max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Get cached semantic information
    pub fn get(&self, uri: &str, input_hash: u64) -> Option<SemanticInfo> {
        let entries = self.entries.read().unwrap();

        if let Some(entry) = entries.get(uri) {
            if entry.input_hash == input_hash {
                debug!("Cache hit for {}", uri);
                let mut metrics = self.metrics.write().unwrap();
                metrics.hits += 1;
                return Some(entry.value.clone());
            }
        }

        let mut metrics = self.metrics.write().unwrap();
        metrics.misses += 1;
        None
    }

    /// Store semantic information in cache
    pub fn put(&self, uri: String, input_hash: u64, value: SemanticInfo) {
        let estimated_size = self.estimate_size(&value);

        // Check if we need to evict entries
        let mut current_size = self.current_size.write().unwrap();
        if *current_size + estimated_size > self.max_size {
            self.evict_oldest();
            *current_size = self.calculate_total_size();
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = CacheEntry {
            value,
            timestamp,
            input_hash,
        };

        let mut entries = self.entries.write().unwrap();
        entries.insert(uri.clone(), entry);
        *current_size += estimated_size;

        debug!("Cached semantic info for {}", uri);
    }

    /// Invalidate cache entry for a document
    pub fn invalidate(&self, uri: &str) {
        let mut entries = self.entries.write().unwrap();
        if entries.remove(uri).is_some() {
            let mut metrics = self.metrics.write().unwrap();
            metrics.invalidations += 1;
            debug!("Invalidated cache for {}", uri);
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
        let mut current_size = self.current_size.write().unwrap();
        *current_size = 0;
        debug!("Cache cleared");
    }

    /// Get cache metrics
    pub fn metrics(&self) -> CacheMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// Estimate size of semantic info in bytes
    fn estimate_size(&self, info: &SemanticInfo) -> usize {
        // Rough estimation: 200 bytes per symbol + 100 bytes per import/definition/reference
        let symbols_size = info.symbols.len() * 200;
        let imports_size = info.imports.len() * 100;
        let definitions_size = info.definitions.len() * 100;
        let references_size = info.references.len() * 100;

        symbols_size + imports_size + definitions_size + references_size
    }

    /// Calculate total cache size
    fn calculate_total_size(&self) -> usize {
        let entries = self.entries.read().unwrap();
        entries
            .values()
            .map(|entry| self.estimate_size(&entry.value))
            .sum()
    }

    /// Evict oldest cache entry
    fn evict_oldest(&self) {
        let mut entries = self.entries.write().unwrap();

        if let Some((oldest_uri, _)) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(uri, entry)| (uri.clone(), entry.timestamp))
        {
            entries.remove(&oldest_uri);
            debug!("Evicted oldest cache entry: {}", oldest_uri);
        }
    }
}

impl Default for SemanticCache {
    fn default() -> Self {
        Self::new()
    }
}

/// AST cache for parsed syntax trees
pub struct AstCache {
    /// Cache entries by document URI
    entries: Arc<RwLock<HashMap<String, CacheEntry<String>>>>,
    /// Performance metrics
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl AstCache {
    /// Create a new AST cache
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get cached AST
    pub fn get(&self, uri: &str, input_hash: u64) -> Option<String> {
        let entries = self.entries.read().unwrap();

        if let Some(entry) = entries.get(uri) {
            if entry.input_hash == input_hash {
                debug!("AST cache hit for {}", uri);
                let mut metrics = self.metrics.write().unwrap();
                metrics.hits += 1;
                return Some(entry.value.clone());
            }
        }

        let mut metrics = self.metrics.write().unwrap();
        metrics.misses += 1;
        None
    }

    /// Store AST in cache
    pub fn put(&self, uri: String, input_hash: u64, value: String) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = CacheEntry {
            value,
            timestamp,
            input_hash,
        };

        let mut entries = self.entries.write().unwrap();
        entries.insert(uri.clone(), entry);

        debug!("Cached AST for {}", uri);
    }

    /// Invalidate cache entry for a document
    pub fn invalidate(&self, uri: &str) {
        let mut entries = self.entries.write().unwrap();
        if entries.remove(uri).is_some() {
            let mut metrics = self.metrics.write().unwrap();
            metrics.invalidations += 1;
            debug!("Invalidated AST cache for {}", uri);
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
        debug!("AST cache cleared");
    }

    /// Get cache metrics
    pub fn metrics(&self) -> CacheMetrics {
        self.metrics.read().unwrap().clone()
    }
}

impl Default for AstCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for symbol index cache entries
type SymbolIndexEntries = Arc<RwLock<HashMap<String, CacheEntry<HashMap<String, usize>>>>>;

/// Symbol index cache
pub struct SymbolIndexCache {
    /// Cache entries by document URI
    entries: SymbolIndexEntries,
    /// Performance metrics
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl SymbolIndexCache {
    /// Create a new symbol index cache
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get cached symbol index
    pub fn get(&self, uri: &str, input_hash: u64) -> Option<HashMap<String, usize>> {
        let entries = self.entries.read().unwrap();

        if let Some(entry) = entries.get(uri) {
            if entry.input_hash == input_hash {
                debug!("Symbol index cache hit for {}", uri);
                let mut metrics = self.metrics.write().unwrap();
                metrics.hits += 1;
                return Some(entry.value.clone());
            }
        }

        let mut metrics = self.metrics.write().unwrap();
        metrics.misses += 1;
        None
    }

    /// Store symbol index in cache
    pub fn put(&self, uri: String, input_hash: u64, value: HashMap<String, usize>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = CacheEntry {
            value,
            timestamp,
            input_hash,
        };

        let mut entries = self.entries.write().unwrap();
        entries.insert(uri.clone(), entry);

        debug!("Cached symbol index for {}", uri);
    }

    /// Invalidate cache entry for a document
    pub fn invalidate(&self, uri: &str) {
        let mut entries = self.entries.write().unwrap();
        if entries.remove(uri).is_some() {
            let mut metrics = self.metrics.write().unwrap();
            metrics.invalidations += 1;
            debug!("Invalidated symbol index cache for {}", uri);
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
        debug!("Symbol index cache cleared");
    }

    /// Get cache metrics
    pub fn metrics(&self) -> CacheMetrics {
        self.metrics.read().unwrap().clone()
    }
}

impl Default for SymbolIndexCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute hash of input string
pub fn hash_input(input: &str) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Position, Range, Symbol, SymbolKind};

    #[test]
    fn test_semantic_cache_hit() {
        let cache = SemanticCache::new();
        let mut info = SemanticInfo::new();
        info.symbols.push(Symbol::new(
            "test".to_string(),
            SymbolKind::Function,
            Range::new(Position::new(0, 0), Position::new(0, 4)),
        ));

        let hash = hash_input("test code");
        cache.put("file://test.rs".to_string(), hash, info.clone());

        let cached = cache.get("file://test.rs", hash);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().symbols.len(), 1);

        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 0);
    }

    #[test]
    fn test_semantic_cache_miss() {
        let cache = SemanticCache::new();
        let info = SemanticInfo::new();

        let hash = hash_input("test code");
        cache.put("file://test.rs".to_string(), hash, info);

        // Try to get with different hash
        let cached = cache.get("file://test.rs", hash + 1);
        assert!(cached.is_none());

        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 1);
    }

    #[test]
    fn test_semantic_cache_invalidation() {
        let cache = SemanticCache::new();
        let info = SemanticInfo::new();

        let hash = hash_input("test code");
        cache.put("file://test.rs".to_string(), hash, info);

        cache.invalidate("file://test.rs");

        let cached = cache.get("file://test.rs", hash);
        assert!(cached.is_none());

        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 1);
    }

    #[test]
    fn test_ast_cache() {
        let cache = AstCache::new();
        let ast = "fn main() {}".to_string();

        let hash = hash_input("fn main() {}");
        cache.put("file://test.rs".to_string(), hash, ast.clone());

        let cached = cache.get("file://test.rs", hash);
        assert_eq!(cached, Some(ast));
    }

    #[test]
    fn test_symbol_index_cache() {
        let cache = SymbolIndexCache::new();
        let mut index = HashMap::new();
        index.insert("main".to_string(), 0);

        let hash = hash_input("fn main() {}");
        cache.put("file://test.rs".to_string(), hash, index.clone());

        let cached = cache.get("file://test.rs", hash);
        assert_eq!(cached, Some(index));
    }

    #[test]
    fn test_cache_metrics_hit_rate() {
        let metrics = CacheMetrics {
            hits: 80,
            misses: 20,
            invalidations: 0,
            avg_analysis_time_ms: 0.0,
        };

        assert_eq!(metrics.hit_rate(), 80.0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = SemanticCache::new();
        let info = SemanticInfo::new();

        let hash = hash_input("test code");
        cache.put("file://test.rs".to_string(), hash, info);

        cache.clear();

        let cached = cache.get("file://test.rs", hash);
        assert!(cached.is_none());
    }
}
