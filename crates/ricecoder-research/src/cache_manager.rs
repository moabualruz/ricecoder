//! Cache management for research analysis results

use crate::error::ResearchError;
use crate::models::ProjectContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Statistics about cache performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Total number of cache invalidations
    pub invalidations: u64,
    /// Current cache size in bytes
    pub size_bytes: u64,
    /// Number of entries in cache
    pub entry_count: usize,
}

impl CacheStatistics {
    /// Calculate hit rate as a percentage (0.0 to 100.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Cached entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// The cached project context
    data: ProjectContext,
    /// When the entry was created
    created_at: SystemTime,
    /// When the entry expires (TTL)
    expires_at: SystemTime,
    /// File modification times when cached
    file_mtimes: HashMap<PathBuf, SystemTime>,
}

impl CacheEntry {
    /// Check if the cache entry has expired
    fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// Check if any tracked files have been modified
    fn has_file_changes(&self, current_mtimes: &HashMap<PathBuf, SystemTime>) -> bool {
        // If file count changed, cache is invalid
        if self.file_mtimes.len() != current_mtimes.len() {
            return true;
        }

        // Check if any tracked file has been modified
        for (path, cached_mtime) in &self.file_mtimes {
            match current_mtimes.get(path) {
                Some(current_mtime) if current_mtime > cached_mtime => return true,
                None => return true, // File was deleted
                _ => {}
            }
        }

        false
    }
}

/// Manages caching of analysis results with TTL and file change detection
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// In-memory cache storage
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStatistics>>,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
}

impl CacheManager {
    /// Create a new cache manager with default TTL of 1 hour
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(3600))
    }

    /// Create a new cache manager with custom TTL
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStatistics {
                hits: 0,
                misses: 0,
                invalidations: 0,
                size_bytes: 0,
                entry_count: 0,
            })),
            default_ttl: ttl,
        }
    }

    /// Get a cached project context if valid
    pub fn get(
        &self,
        project_root: &Path,
        file_mtimes: &HashMap<PathBuf, SystemTime>,
    ) -> Result<Option<ProjectContext>, ResearchError> {
        let cache = self.cache.read().map_err(|e| {
            ResearchError::CacheError {
                operation: "read".to_string(),
                reason: format!("Failed to acquire read lock: {}", e),
            }
        })?;

        if let Some(entry) = cache.get(project_root) {
            // Check if entry has expired
            if entry.is_expired() {
                drop(cache);
                self.invalidate(project_root)?;
                let mut stats = self.stats.write().map_err(|e| {
                    ResearchError::CacheError {
                        operation: "write".to_string(),
                        reason: format!("Failed to acquire write lock: {}", e),
                    }
                })?;
                stats.misses += 1;
                return Ok(None);
            }

            // Check if any tracked files have changed
            if entry.has_file_changes(file_mtimes) {
                drop(cache);
                self.invalidate(project_root)?;
                let mut stats = self.stats.write().map_err(|e| {
                    ResearchError::CacheError {
                        operation: "write".to_string(),
                        reason: format!("Failed to acquire write lock: {}", e),
                    }
                })?;
                stats.misses += 1;
                stats.invalidations += 1;
                return Ok(None);
            }

            // Cache hit
            let mut stats = self.stats.write().map_err(|e| {
                ResearchError::CacheError {
                    operation: "write".to_string(),
                    reason: format!("Failed to acquire write lock: {}", e),
                }
            })?;
            stats.hits += 1;

            Ok(Some(entry.data.clone()))
        } else {
            let mut stats = self.stats.write().map_err(|e| {
                ResearchError::CacheError {
                    operation: "write".to_string(),
                    reason: format!("Failed to acquire write lock: {}", e),
                }
            })?;
            stats.misses += 1;
            Ok(None)
        }
    }

    /// Store a project context in the cache
    pub fn set(
        &self,
        project_root: &Path,
        context: &ProjectContext,
        file_mtimes: HashMap<PathBuf, SystemTime>,
    ) -> Result<(), ResearchError> {
        let now = SystemTime::now();
        let expires_at = now + self.default_ttl;

        let entry = CacheEntry {
            data: context.clone(),
            created_at: now,
            expires_at,
            file_mtimes,
        };

        let mut cache = self.cache.write().map_err(|e| {
            ResearchError::CacheError {
                operation: "write".to_string(),
                reason: format!("Failed to acquire write lock: {}", e),
            }
        })?;

        cache.insert(project_root.to_path_buf(), entry);

        // Update statistics
        let mut stats = self.stats.write().map_err(|e| {
            ResearchError::CacheError {
                operation: "write".to_string(),
                reason: format!("Failed to acquire write lock: {}", e),
            }
        })?;
        stats.entry_count = cache.len();

        Ok(())
    }

    /// Invalidate cache for a specific project
    pub fn invalidate(&self, project_root: &Path) -> Result<(), ResearchError> {
        let mut cache = self.cache.write().map_err(|e| {
            ResearchError::CacheError {
                operation: "write".to_string(),
                reason: format!("Failed to acquire write lock: {}", e),
            }
        })?;

        if cache.remove(project_root).is_some() {
            let mut stats = self.stats.write().map_err(|e| {
                ResearchError::CacheError {
                    operation: "write".to_string(),
                    reason: format!("Failed to acquire write lock: {}", e),
                }
            })?;
            stats.invalidations += 1;
            stats.entry_count = cache.len();
        }

        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&self) -> Result<(), ResearchError> {
        let mut cache = self.cache.write().map_err(|e| {
            ResearchError::CacheError {
                operation: "write".to_string(),
                reason: format!("Failed to acquire write lock: {}", e),
            }
        })?;

        let cleared_count = cache.len();
        cache.clear();

        let mut stats = self.stats.write().map_err(|e| {
            ResearchError::CacheError {
                operation: "write".to_string(),
                reason: format!("Failed to acquire write lock: {}", e),
            }
        })?;
        stats.invalidations += cleared_count as u64;
        stats.entry_count = 0;

        Ok(())
    }

    /// Get current cache statistics
    pub fn statistics(&self) -> Result<CacheStatistics, ResearchError> {
        let stats = self.stats.read().map_err(|e| {
            ResearchError::CacheError {
                operation: "read".to_string(),
                reason: format!("Failed to acquire read lock: {}", e),
            }
        })?;

        Ok(stats.clone())
    }

    /// Check if a project is cached and valid
    pub fn is_cached(
        &self,
        project_root: &Path,
        file_mtimes: &HashMap<PathBuf, SystemTime>,
    ) -> Result<bool, ResearchError> {
        let cache = self.cache.read().map_err(|e| {
            ResearchError::CacheError {
                operation: "read".to_string(),
                reason: format!("Failed to acquire read lock: {}", e),
            }
        })?;

        if let Some(entry) = cache.get(project_root) {
            Ok(!entry.is_expired() && !entry.has_file_changes(file_mtimes))
        } else {
            Ok(false)
        }
    }

    /// Get cache entry count
    pub fn entry_count(&self) -> Result<usize, ResearchError> {
        let cache = self.cache.read().map_err(|e| {
            ResearchError::CacheError {
                operation: "read".to_string(),
                reason: format!("Failed to acquire read lock: {}", e),
            }
        })?;

        Ok(cache.len())
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_creation() {
        let manager = CacheManager::new();
        assert_eq!(manager.default_ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_cache_manager_with_custom_ttl() {
        let ttl = Duration::from_secs(300);
        let manager = CacheManager::with_ttl(ttl);
        assert_eq!(manager.default_ttl, ttl);
    }

    #[test]
    fn test_cache_statistics_hit_rate_zero() {
        let stats = CacheStatistics {
            hits: 0,
            misses: 0,
            invalidations: 0,
            size_bytes: 0,
            entry_count: 0,
        };
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_statistics_hit_rate_calculation() {
        let stats = CacheStatistics {
            hits: 75,
            misses: 25,
            invalidations: 0,
            size_bytes: 0,
            entry_count: 0,
        };
        assert_eq!(stats.hit_rate(), 75.0);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let now = SystemTime::now();
        let entry = CacheEntry {
            data: ProjectContext {
                project_type: crate::models::ProjectType::Library,
                languages: vec![],
                frameworks: vec![],
                structure: crate::models::ProjectStructure {
                    root: PathBuf::from("/test"),
                    source_dirs: vec![],
                    test_dirs: vec![],
                    config_files: vec![],
                    entry_points: vec![],
                },
                patterns: vec![],
                dependencies: vec![],
                architectural_intent: crate::models::ArchitecturalIntent {
                    style: crate::models::ArchitecturalStyle::Unknown,
                    principles: vec![],
                    constraints: vec![],
                    decisions: vec![],
                },
                standards: crate::models::StandardsProfile {
                    naming_conventions: crate::models::NamingConventions {
                        function_case: crate::models::CaseStyle::SnakeCase,
                        variable_case: crate::models::CaseStyle::SnakeCase,
                        class_case: crate::models::CaseStyle::PascalCase,
                        constant_case: crate::models::CaseStyle::UpperCase,
                    },
                    formatting_style: crate::models::FormattingStyle {
                        indent_size: 4,
                        indent_type: crate::models::IndentType::Spaces,
                        line_length: 100,
                    },
                    import_organization: crate::models::ImportOrganization {
                        order: vec![],
                        sort_within_group: true,
                    },
                    documentation_style: crate::models::DocumentationStyle {
                        format: crate::models::DocFormat::RustDoc,
                        required_for_public: true,
                    },
                },
            },
            created_at: now,
            expires_at: now - Duration::from_secs(1), // Already expired
            file_mtimes: HashMap::new(),
        };

        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_not_expired() {
        let now = SystemTime::now();
        let entry = CacheEntry {
            data: ProjectContext {
                project_type: crate::models::ProjectType::Library,
                languages: vec![],
                frameworks: vec![],
                structure: crate::models::ProjectStructure {
                    root: PathBuf::from("/test"),
                    source_dirs: vec![],
                    test_dirs: vec![],
                    config_files: vec![],
                    entry_points: vec![],
                },
                patterns: vec![],
                dependencies: vec![],
                architectural_intent: crate::models::ArchitecturalIntent {
                    style: crate::models::ArchitecturalStyle::Unknown,
                    principles: vec![],
                    constraints: vec![],
                    decisions: vec![],
                },
                standards: crate::models::StandardsProfile {
                    naming_conventions: crate::models::NamingConventions {
                        function_case: crate::models::CaseStyle::SnakeCase,
                        variable_case: crate::models::CaseStyle::SnakeCase,
                        class_case: crate::models::CaseStyle::PascalCase,
                        constant_case: crate::models::CaseStyle::UpperCase,
                    },
                    formatting_style: crate::models::FormattingStyle {
                        indent_size: 4,
                        indent_type: crate::models::IndentType::Spaces,
                        line_length: 100,
                    },
                    import_organization: crate::models::ImportOrganization {
                        order: vec![],
                        sort_within_group: true,
                    },
                    documentation_style: crate::models::DocumentationStyle {
                        format: crate::models::DocFormat::RustDoc,
                        required_for_public: true,
                    },
                },
            },
            created_at: now,
            expires_at: now + Duration::from_secs(3600), // Expires in 1 hour
            file_mtimes: HashMap::new(),
        };

        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_file_changes_detection() {
        let now = SystemTime::now();
        let mut cached_mtimes = HashMap::new();
        cached_mtimes.insert(PathBuf::from("/test/file1.rs"), now);

        let entry = CacheEntry {
            data: ProjectContext {
                project_type: crate::models::ProjectType::Library,
                languages: vec![],
                frameworks: vec![],
                structure: crate::models::ProjectStructure {
                    root: PathBuf::from("/test"),
                    source_dirs: vec![],
                    test_dirs: vec![],
                    config_files: vec![],
                    entry_points: vec![],
                },
                patterns: vec![],
                dependencies: vec![],
                architectural_intent: crate::models::ArchitecturalIntent {
                    style: crate::models::ArchitecturalStyle::Unknown,
                    principles: vec![],
                    constraints: vec![],
                    decisions: vec![],
                },
                standards: crate::models::StandardsProfile {
                    naming_conventions: crate::models::NamingConventions {
                        function_case: crate::models::CaseStyle::SnakeCase,
                        variable_case: crate::models::CaseStyle::SnakeCase,
                        class_case: crate::models::CaseStyle::PascalCase,
                        constant_case: crate::models::CaseStyle::UpperCase,
                    },
                    formatting_style: crate::models::FormattingStyle {
                        indent_size: 4,
                        indent_type: crate::models::IndentType::Spaces,
                        line_length: 100,
                    },
                    import_organization: crate::models::ImportOrganization {
                        order: vec![],
                        sort_within_group: true,
                    },
                    documentation_style: crate::models::DocumentationStyle {
                        format: crate::models::DocFormat::RustDoc,
                        required_for_public: true,
                    },
                },
            },
            created_at: now,
            expires_at: now + Duration::from_secs(3600),
            file_mtimes: cached_mtimes,
        };

        // Test: file was modified
        let mut current_mtimes = HashMap::new();
        current_mtimes.insert(PathBuf::from("/test/file1.rs"), now + Duration::from_secs(1));
        assert!(entry.has_file_changes(&current_mtimes));

        // Test: file was deleted
        let current_mtimes_empty = HashMap::new();
        assert!(entry.has_file_changes(&current_mtimes_empty));

        // Test: no changes
        let mut current_mtimes_same = HashMap::new();
        current_mtimes_same.insert(PathBuf::from("/test/file1.rs"), now);
        assert!(!entry.has_file_changes(&current_mtimes_same));
    }
}
