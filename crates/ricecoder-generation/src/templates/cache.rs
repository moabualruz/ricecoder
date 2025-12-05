//! Template caching for improved performance
//!
//! Provides in-memory caching of parsed templates with file change detection
//! and cache statistics.

use crate::templates::error::TemplateError;
use crate::templates::parser::ParsedTemplate;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Statistics about the template cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of templates in cache
    pub total_templates: usize,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Total size of cached templates in bytes
    pub total_size_bytes: usize,
}

impl CacheStats {
    /// Calculate cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Cached template entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Parsed template
    template: ParsedTemplate,
    /// File path (if loaded from file)
    file_path: Option<PathBuf>,
    /// Last modified time of the file
    last_modified: Option<u64>,
    /// Size of the template content in bytes
    size_bytes: usize,
}

/// Template cache with file change detection
pub struct TemplateCache {
    /// Cache storage
    cache: HashMap<String, CacheEntry>,
    /// Cache statistics
    stats: CacheStats,
}

impl TemplateCache {
    /// Create a new template cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: CacheStats {
                total_templates: 0,
                hits: 0,
                misses: 0,
                total_size_bytes: 0,
            },
        }
    }

    /// Get a cached template by key
    ///
    /// # Arguments
    /// * `key` - Cache key (usually template name or path)
    ///
    /// # Returns
    /// Cached template if found and valid, None otherwise
    pub fn get(&mut self, key: &str) -> Option<ParsedTemplate> {
        if let Some(entry) = self.cache.get(key) {
            // Check if file has been modified (if it's file-backed)
            if let Some(file_path) = &entry.file_path {
                if let Ok(modified_time) = self.get_file_modified_time(file_path) {
                    if Some(modified_time) != entry.last_modified {
                        // File has been modified, invalidate cache entry
                        self.cache.remove(key);
                        self.stats.misses += 1;
                        return None;
                    }
                }
            }

            self.stats.hits += 1;
            Some(entry.template.clone())
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a template into the cache
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `template` - Parsed template to cache
    pub fn insert(&mut self, key: String, template: ParsedTemplate) {
        self.insert_with_file(key, template, None);
    }

    /// Insert a template into the cache with file path for change detection
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `template` - Parsed template to cache
    /// * `file_path` - Path to the template file (for change detection)
    pub fn insert_with_file(
        &mut self,
        key: String,
        template: ParsedTemplate,
        file_path: Option<PathBuf>,
    ) {
        let size_bytes = template.elements.len() * 8; // Rough estimate
        let last_modified = file_path
            .as_ref()
            .and_then(|p| self.get_file_modified_time(p).ok());

        let entry = CacheEntry {
            template,
            file_path,
            last_modified,
            size_bytes,
        };

        if !self.cache.contains_key(&key) {
            self.stats.total_templates += 1;
            self.stats.total_size_bytes += size_bytes;
        }

        self.cache.insert(key, entry);
    }

    /// Remove a template from the cache
    pub fn remove(&mut self, key: &str) -> Option<ParsedTemplate> {
        if let Some(entry) = self.cache.remove(key) {
            self.stats.total_templates = self.stats.total_templates.saturating_sub(1);
            self.stats.total_size_bytes =
                self.stats.total_size_bytes.saturating_sub(entry.size_bytes);
            Some(entry.template)
        } else {
            None
        }
    }

    /// Clear all cached templates
    pub fn clear(&mut self) {
        self.cache.clear();
        self.stats.total_templates = 0;
        self.stats.total_size_bytes = 0;
    }

    /// Invalidate cache entries for a specific file
    pub fn invalidate_file(&mut self, file_path: &Path) {
        let keys_to_remove: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, entry)| entry.file_path.as_ref().is_some_and(|p| p == file_path))
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            self.remove(&key);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.clone()
    }

    /// Check if a key exists in the cache
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Get the number of cached templates
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get file modification time as Unix timestamp
    fn get_file_modified_time(&self, path: &Path) -> Result<u64, TemplateError> {
        let metadata = std::fs::metadata(path).map_err(TemplateError::IoError)?;

        let modified = metadata.modified().map_err(TemplateError::IoError)?;

        let duration = modified.duration_since(UNIX_EPOCH).map_err(|_| {
            TemplateError::RenderError("Invalid file modification time".to_string())
        })?;

        Ok(duration.as_secs())
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::parser::TemplateElement;

    fn create_test_template() -> ParsedTemplate {
        ParsedTemplate {
            elements: vec![TemplateElement::Text("test".to_string())],
            placeholders: vec![],
            placeholder_names: Default::default(),
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache = TemplateCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test".to_string(), template.clone());
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get("test");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = TemplateCache::new();
        let retrieved = cache.get("nonexistent");
        assert!(retrieved.is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_cache_hit() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test".to_string(), template);
        let _ = cache.get("test");
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test".to_string(), template);
        assert_eq!(cache.len(), 1);

        let removed = cache.remove("test");
        assert!(removed.is_some());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test1".to_string(), template.clone());
        cache.insert("test2".to_string(), template);

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_contains() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test".to_string(), template);
        assert!(cache.contains("test"));
        assert!(!cache.contains("nonexistent"));
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test".to_string(), template);

        // 2 hits, 1 miss
        let _ = cache.get("test");
        let _ = cache.get("test");
        let _ = cache.get("nonexistent");

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 66.66).abs() < 1.0); // Approximately 66.66%
    }

    #[test]
    fn test_cache_multiple_templates() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test1".to_string(), template.clone());
        cache.insert("test2".to_string(), template.clone());
        cache.insert("test3".to_string(), template);

        assert_eq!(cache.len(), 3);
        assert!(cache.contains("test1"));
        assert!(cache.contains("test2"));
        assert!(cache.contains("test3"));
    }

    #[test]
    fn test_cache_stats_total_templates() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();

        cache.insert("test1".to_string(), template.clone());
        cache.insert("test2".to_string(), template);

        let stats = cache.stats();
        assert_eq!(stats.total_templates, 2);
    }

    #[test]
    fn test_cache_stats_zero_hit_rate() {
        let cache = TemplateCache::new();
        let stats = cache.stats();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_insert_with_file() {
        let mut cache = TemplateCache::new();
        let template = create_test_template();
        let path = PathBuf::from("test.tmpl");

        cache.insert_with_file("test".to_string(), template, Some(path.clone()));
        assert!(cache.contains("test"));
    }
}
