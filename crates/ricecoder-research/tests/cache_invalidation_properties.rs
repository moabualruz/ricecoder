//! Property-based tests for cache invalidation correctness
//! **Feature: ricecoder-research, Property 8: Cache Invalidation Correctness**
//! **Validates: Requirements 4.1, 4.2, 4.3**

use proptest::prelude::*;
use ricecoder_research::{
    ArchitecturalIntent, ArchitecturalStyle, CaseStyle, DocFormat, DocumentationStyle,
    FormattingStyle, ImportOrganization, IndentType, NamingConventions, ProjectContext,
    ProjectStructure, ProjectType, SearchStatistics, StandardsProfile,
};
use ricecoder_storage::cache::CacheManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Generate a valid ProjectContext for testing
fn arb_project_context() -> impl Strategy<Value = ProjectContext> {
    Just(ProjectContext {
        project_type: ProjectType::Library,
        languages: vec![],
        frameworks: vec![],
        structure: ProjectStructure {
            root: PathBuf::from("/test"),
            source_dirs: vec![],
            test_dirs: vec![],
            config_files: vec![],
            entry_points: vec![],
        },
        patterns: vec![],
        dependencies: vec![],
        architectural_intent: ArchitecturalIntent {
            style: ArchitecturalStyle::Unknown,
            principles: vec![],
            constraints: vec![],
            decisions: vec![],
        },
        standards: StandardsProfile {
            naming_conventions: NamingConventions {
                function_case: CaseStyle::SnakeCase,
                variable_case: CaseStyle::SnakeCase,
                class_case: CaseStyle::PascalCase,
                constant_case: CaseStyle::UpperCase,
            },
            formatting_style: FormattingStyle {
                indent_size: 4,
                indent_type: IndentType::Spaces,
                line_length: 100,
            },
            import_organization: ImportOrganization {
                order: vec![],
                sort_within_group: true,
            },
            documentation_style: DocumentationStyle {
                format: DocFormat::RustDoc,
                required_for_public: true,
            },
        },
    })
}

proptest! {
    /// Property: Cache stores and retrieves data correctly
    /// For any cached analysis, retrieving it should return the same data
    #[test]
    fn prop_cache_stores_and_retrieves_correctly(context in arb_project_context()) {
        let cache = CacheManager::new();
        let project_root = PathBuf::from("/test/project");
        let file_mtimes = HashMap::new();

        // Store the context
        cache.set(&project_root, &context, file_mtimes.clone()).expect("Failed to set cache");

        // Retrieve the context
        let retrieved = cache.get(&project_root, &file_mtimes).expect("Failed to get cache");

        // Verify it matches
        prop_assert!(retrieved.is_some(), "Cache should contain the stored context");
        let retrieved_context = retrieved.unwrap();
        prop_assert_eq!(retrieved_context.project_type, context.project_type);
    }

    /// Property: Cache invalidation removes entries
    /// For any cached analysis, invalidating it should remove the entry
    #[test]
    fn prop_cache_invalidation_removes_entries(context in arb_project_context()) {
        let cache = CacheManager::new();
        let project_root = PathBuf::from("/test/project");
        let file_mtimes = HashMap::new();

        // Store the context
        cache.set(&project_root, &context, file_mtimes.clone()).expect("Failed to set cache");

        // Verify it's cached
        let is_cached = cache.is_cached(&project_root, &file_mtimes.clone()).expect("Failed to check cache");
        prop_assert!(is_cached, "Context should be cached");

        // Invalidate the cache
        cache.invalidate(&project_root).expect("Failed to invalidate cache");

        // Verify it's no longer cached
        let is_cached_after = cache.is_cached(&project_root, &file_mtimes).expect("Failed to check cache");
        prop_assert!(!is_cached_after, "Context should not be cached after invalidation");
    }

    /// Property: Cache detects file modifications
    /// For any cached analysis, modifying tracked files should invalidate the cache
    #[test]
    fn prop_cache_detects_file_modifications(context in arb_project_context()) {
        let cache = CacheManager::new();
        let project_root = PathBuf::from("/test/project");
        let test_file = PathBuf::from("/test/project/src/main.rs");

        // Create initial file mtimes
        let mut file_mtimes = HashMap::new();
        file_mtimes.insert(test_file.clone(), SystemTime::now());

        // Store the context
        cache.set(&project_root, &context, file_mtimes.clone()).expect("Failed to set cache");

        // Verify it's cached
        let is_cached = cache.is_cached(&project_root, &file_mtimes.clone()).expect("Failed to check cache");
        prop_assert!(is_cached, "Context should be cached");

        // Simulate file modification by updating mtime
        let mut modified_mtimes = HashMap::new();
        modified_mtimes.insert(test_file, SystemTime::now() + std::time::Duration::from_secs(1));

        // Verify cache is invalidated due to file change
        let is_cached_after = cache.is_cached(&project_root, &modified_mtimes).expect("Failed to check cache");
        prop_assert!(!is_cached_after, "Cache should be invalidated when files are modified");
    }

    /// Property: Cache detects file deletions
    /// For any cached analysis with tracked files, deleting files should invalidate the cache
    #[test]
    fn prop_cache_detects_file_deletions(context in arb_project_context()) {
        let cache = CacheManager::new();
        let project_root = PathBuf::from("/test/project");
        let test_file = PathBuf::from("/test/project/src/main.rs");

        // Create initial file mtimes
        let mut file_mtimes = HashMap::new();
        file_mtimes.insert(test_file, SystemTime::now());

        // Store the context
        cache.set(&project_root, &context, file_mtimes.clone()).expect("Failed to set cache");

        // Verify it's cached
        let is_cached = cache.is_cached(&project_root, &file_mtimes).expect("Failed to check cache");
        prop_assert!(is_cached, "Context should be cached");

        // Simulate file deletion by providing empty mtimes
        let empty_mtimes = HashMap::new();

        // Verify cache is invalidated due to file deletion
        let is_cached_after = cache.is_cached(&project_root, &empty_mtimes).expect("Failed to check cache");
        prop_assert!(!is_cached_after, "Cache should be invalidated when files are deleted");
    }

    /// Property: Cache detects file additions
    /// For any cached analysis, adding new files should invalidate the cache
    #[test]
    fn prop_cache_detects_file_additions(context in arb_project_context()) {
        let cache = CacheManager::new();
        let project_root = PathBuf::from("/test/project");
        let test_file1 = PathBuf::from("/test/project/src/main.rs");

        // Create initial file mtimes with one file
        let mut file_mtimes = HashMap::new();
        file_mtimes.insert(test_file1, SystemTime::now());

        // Store the context
        cache.set(&project_root, &context, file_mtimes.clone()).expect("Failed to set cache");

        // Verify it's cached
        let is_cached = cache.is_cached(&project_root, &file_mtimes.clone()).expect("Failed to check cache");
        prop_assert!(is_cached, "Context should be cached");

        // Simulate file addition by adding a new file to mtimes
        let test_file2 = PathBuf::from("/test/project/src/lib.rs");
        let mut new_mtimes = file_mtimes.clone();
        new_mtimes.insert(test_file2, SystemTime::now());

        // Verify cache is invalidated due to file addition
        let is_cached_after = cache.is_cached(&project_root, &new_mtimes).expect("Failed to check cache");
        prop_assert!(!is_cached_after, "Cache should be invalidated when files are added");
    }

    /// Property: Cache statistics are accurate
    /// For any sequence of cache operations, statistics should reflect the operations
    #[test]
    fn prop_cache_statistics_accuracy(context in arb_project_context()) {
        let cache = CacheManager::new();
        let project_root = PathBuf::from("/test/project");
        let file_mtimes = HashMap::new();

        // Initial statistics should show no operations
        let initial_stats = cache.statistics().expect("Failed to get statistics");
        prop_assert_eq!(initial_stats.hits, 0);
        prop_assert_eq!(initial_stats.misses, 0);

        // Store the context
        cache.set(&project_root, &context, file_mtimes.clone()).expect("Failed to set cache");

        // First retrieval should be a hit
        let _ = cache.get(&project_root, &file_mtimes.clone()).expect("Failed to get cache");
        let stats_after_hit = cache.statistics().expect("Failed to get statistics");
        prop_assert_eq!(stats_after_hit.hits, 1);

        // Invalidate and try to retrieve (should be a miss)
        cache.invalidate(&project_root).expect("Failed to invalidate cache");
        let _ = cache.get(&project_root, &file_mtimes).expect("Failed to get cache");
        let stats_after_miss = cache.statistics().expect("Failed to get statistics");
        prop_assert_eq!(stats_after_miss.misses, 1);
        prop_assert_eq!(stats_after_miss.invalidations, 1);
    }

    /// Property: Cache clear removes all entries
    /// For any cached entries, clearing the cache should remove all of them
    #[test]
    fn prop_cache_clear_removes_all_entries(context in arb_project_context()) {
        let cache = CacheManager::new();
        let project_root1 = PathBuf::from("/test/project1");
        let project_root2 = PathBuf::from("/test/project2");
        let file_mtimes = HashMap::new();

        // Store multiple contexts
        cache.set(&project_root1, &context, file_mtimes.clone()).expect("Failed to set cache");
        cache.set(&project_root2, &context, file_mtimes.clone()).expect("Failed to set cache");

        // Verify both are cached
        let count_before = cache.entry_count().expect("Failed to get entry count");
        prop_assert_eq!(count_before, 2);

        // Clear the cache
        cache.clear().expect("Failed to clear cache");

        // Verify all entries are removed
        let count_after = cache.entry_count().expect("Failed to get entry count");
        prop_assert_eq!(count_after, 0);

        // Verify entries are no longer retrievable
        let retrieved1 = cache.get(&project_root1, &file_mtimes.clone()).expect("Failed to get cache");
        let retrieved2 = cache.get(&project_root2, &file_mtimes).expect("Failed to get cache");
        prop_assert!(retrieved1.is_none());
        prop_assert!(retrieved2.is_none());
    }

    /// Property: Cache hit rate calculation is correct
    /// For any sequence of hits and misses, hit rate should be calculated correctly
    #[test]
    fn prop_cache_hit_rate_calculation(hits in 0u64..1000, misses in 0u64..1000) {
        let stats = SearchStatistics {
            hits,
            misses,
            invalidations: 0,
            size_bytes: 0,
            entry_count: 0,
        };

        let hit_rate = stats.hit_rate();
        let total = hits + misses;

        if total == 0 {
            prop_assert_eq!(hit_rate, 0.0);
        } else {
            let expected = (hits as f64 / total as f64) * 100.0;
            prop_assert!((hit_rate - expected).abs() < 0.01);
        }
    }
}
