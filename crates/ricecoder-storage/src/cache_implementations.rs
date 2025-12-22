//! Concrete caching implementations for ricecoder
//!
//! This module provides ready-to-use caching implementations for common operations:
//! - Configuration caching
//! - Specification caching
//! - Provider response caching
//! - Project analysis caching

use std::{
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use serde::Serialize;
use tracing::{debug, info};

use crate::CacheManager;

/// Cache statistics tracker
#[derive(Debug, Clone)]
pub struct CacheStats {
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl CacheStats {
    /// Create new cache statistics tracker
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get cache hit rate (0.0 to 1.0)
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

    /// Get statistics tuple (hits, misses, hit_rate)
    pub fn stats(&self) -> (u64, u64, f64) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let rate = self.hit_rate();
        (hits, misses, rate)
    }

    /// Log cache statistics
    pub fn log_stats(&self, name: &str) {
        let (hits, misses, rate) = self.stats();
        info!(
            "{} cache statistics: {} hits, {} misses, {:.2}% hit rate",
            name,
            hits,
            misses,
            rate * 100.0
        );
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration caching wrapper
pub struct ConfigCache {
    cache: CacheManager,
    stats: CacheStats,
}

impl ConfigCache {
    /// Create new configuration cache
    pub fn new(cache_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
            stats: CacheStats::new(),
        })
    }

    /// Get cached configuration or load from file
    pub fn get_config<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        path: &Path,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let cache_key = format!("config_{}", path.display());

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key)? {
            debug!("Configuration cache hit: {}", path.display());
            self.stats.record_hit();
            return Ok(serde_json::from_str(&cached)?);
        }

        debug!("Configuration cache miss: {}", path.display());
        self.stats.record_miss();

        // Load and parse configuration
        let content = std::fs::read_to_string(path)?;
        let config: T = serde_yaml::from_str(&content)?;

        // Cache for 1 hour (3600 seconds)
        let json = serde_json::to_string(&config)?;
        self.cache.set(
            &cache_key,
            json,
            crate::CacheInvalidationStrategy::Ttl(3600),
        )?;

        Ok(config)
    }

    /// Invalidate configuration cache
    pub fn invalidate_config(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let cache_key = format!("config_{}", path.display());
        self.cache.invalidate(&cache_key)?;
        debug!("Configuration cache invalidated: {}", path.display());
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
}

/// Specification caching wrapper
pub struct SpecCache {
    cache: CacheManager,
    stats: CacheStats,
}

impl SpecCache {
    /// Create new specification cache
    pub fn new(cache_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
            stats: CacheStats::new(),
        })
    }

    /// Get cached specification or load from file
    pub fn get_spec<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        path: &Path,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let cache_key = format!("spec_{}", path.display());

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key)? {
            debug!("Specification cache hit: {}", path.display());
            self.stats.record_hit();
            return Ok(serde_json::from_str(&cached)?);
        }

        debug!("Specification cache miss: {}", path.display());
        self.stats.record_miss();

        // Load and parse specification
        let content = std::fs::read_to_string(path)?;
        let spec: T = serde_yaml::from_str(&content)?;

        // Cache for 1 hour (3600 seconds)
        let json = serde_json::to_string(&spec)?;
        self.cache.set(
            &cache_key,
            json,
            crate::CacheInvalidationStrategy::Ttl(3600),
        )?;

        Ok(spec)
    }

    /// Invalidate specification cache
    pub fn invalidate_spec(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let cache_key = format!("spec_{}", path.display());
        self.cache.invalidate(&cache_key)?;
        debug!("Specification cache invalidated: {}", path.display());
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
}

/// Provider response caching wrapper
pub struct ProviderCache {
    cache: CacheManager,
    stats: CacheStats,
}

impl ProviderCache {
    /// Create new provider response cache
    pub fn new(cache_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
            stats: CacheStats::new(),
        })
    }

    /// Get cached provider response
    pub fn get_response(
        &self,
        provider: &str,
        model: &str,
        prompt: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let cache_key = self.make_cache_key(provider, model, prompt);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key)? {
            debug!("Provider response cache hit: {}/{}", provider, model);
            self.stats.record_hit();
            return Ok(Some(cached));
        }

        debug!("Provider response cache miss: {}/{}", provider, model);
        self.stats.record_miss();
        Ok(None)
    }

    /// Cache provider response
    pub fn cache_response(
        &self,
        provider: &str,
        model: &str,
        prompt: &str,
        response: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cache_key = self.make_cache_key(provider, model, prompt);

        // Cache for 24 hours (86400 seconds)
        self.cache.set(
            &cache_key,
            response.to_string(),
            crate::CacheInvalidationStrategy::Ttl(86400),
        )?;

        debug!("Provider response cached: {}/{}", provider, model);
        Ok(())
    }

    /// Make cache key from provider, model, and prompt
    fn make_cache_key(&self, provider: &str, model: &str, prompt: &str) -> String {
        // Use simple hash of prompt to avoid long keys
        // Calculate a simple hash by summing byte values
        let hash = prompt
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

        format!("provider_{}_{}_{}", provider, model, hash)
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
}

/// Project analysis caching wrapper
pub struct ProjectAnalysisCache {
    cache: CacheManager,
    stats: CacheStats,
}

impl ProjectAnalysisCache {
    /// Create new project analysis cache
    pub fn new(cache_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cache: CacheManager::new(cache_dir)?,
            stats: CacheStats::new(),
        })
    }

    /// Get cached project analysis
    pub fn get_analysis<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        project_path: &Path,
    ) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let cache_key = format!("analysis_{}", project_path.display());

        if let Some(cached) = self.cache.get(&cache_key)? {
            debug!("Project analysis cache hit: {}", project_path.display());
            self.stats.record_hit();
            return Ok(Some(serde_json::from_str(&cached)?));
        }

        debug!("Project analysis cache miss: {}", project_path.display());
        self.stats.record_miss();
        Ok(None)
    }

    /// Cache project analysis
    pub fn cache_analysis<T: serde::Serialize>(
        &self,
        project_path: &Path,
        analysis: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cache_key = format!("analysis_{}", project_path.display());

        // Cache for 1 hour (3600 seconds)
        let json = serde_json::to_string(analysis)?;
        self.cache.set(
            &cache_key,
            json,
            crate::CacheInvalidationStrategy::Ttl(3600),
        )?;

        debug!("Project analysis cached: {}", project_path.display());
        Ok(())
    }

    /// Invalidate project analysis cache
    pub fn invalidate_analysis(
        &self,
        project_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cache_key = format!("analysis_{}", project_path.display());
        self.cache.invalidate(&cache_key)?;
        debug!(
            "Project analysis cache invalidated: {}",
            project_path.display()
        );
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats::new();

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        let (hits, misses, rate) = stats.stats();
        assert_eq!(hits, 2);
        assert_eq!(misses, 1);
        assert!((rate - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_cache_stats_reset() {
        let stats = CacheStats::new();

        stats.record_hit();
        stats.record_miss();
        stats.reset();

        let (hits, misses, _) = stats.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
    }

    #[test]
    fn test_config_cache() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().join("cache");
        std::fs::create_dir(&cache_dir)?;

        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "key: value")?;

        let cache = ConfigCache::new(&cache_dir)?;

        // First access: miss
        let _: serde_json::Value = cache.get_config(&config_path)?;
        assert_eq!(cache.stats().stats().1, 1); // 1 miss

        // Second access: hit
        let _: serde_json::Value = cache.get_config(&config_path)?;
        assert_eq!(cache.stats().stats().0, 1); // 1 hit

        Ok(())
    }

    #[test]
    fn test_spec_cache() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().join("cache");
        std::fs::create_dir(&cache_dir)?;

        let spec_path = temp_dir.path().join("spec.yaml");
        std::fs::write(&spec_path, "name: test")?;

        let cache = SpecCache::new(&cache_dir)?;

        // First access: miss
        let _: serde_json::Value = cache.get_spec(&spec_path)?;
        assert_eq!(cache.stats().stats().1, 1); // 1 miss

        // Second access: hit
        let _: serde_json::Value = cache.get_spec(&spec_path)?;
        assert_eq!(cache.stats().stats().0, 1); // 1 hit

        Ok(())
    }

    #[test]
    fn test_provider_cache() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().join("cache");
        std::fs::create_dir(&cache_dir)?;

        let cache = ProviderCache::new(&cache_dir)?;

        // First access: miss
        let result = cache.get_response("openai", "gpt-4", "hello")?;
        assert!(result.is_none());
        assert_eq!(cache.stats().stats().1, 1); // 1 miss

        // Cache response
        cache.cache_response("openai", "gpt-4", "hello", "world")?;

        // Second access: hit
        let result = cache.get_response("openai", "gpt-4", "hello")?;
        assert_eq!(result, Some("world".to_string()));
        assert_eq!(cache.stats().stats().0, 1); // 1 hit

        Ok(())
    }
}
