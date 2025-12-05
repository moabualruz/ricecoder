//! Unit tests for CacheManager, ChangeDetector, and CacheStatsTracker
//! Tests cache storage, retrieval, invalidation, and statistics tracking

use ricecoder_research::{
    ArchitecturalIntent, ArchitecturalStyle, CacheManager, CacheStatsTracker, CaseStyle,
    ChangeDetector, DocFormat, DocumentationStyle, FormattingStyle, ImportOrganization, IndentType,
    NamingConventions, ProjectContext, ProjectStructure, ProjectType, StandardsProfile,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Helper function to create a test ProjectContext
fn create_test_context() -> ProjectContext {
    ProjectContext {
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
    }
}

#[test]
fn test_cache_manager_new() {
    let cache = CacheManager::new();
    assert_eq!(cache.entry_count().unwrap(), 0);
}

#[test]
fn test_cache_manager_set_and_get() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root = PathBuf::from("/test/project");
    let file_mtimes = HashMap::new();

    // Set cache
    cache
        .set(&project_root, &context, file_mtimes.clone())
        .unwrap();

    // Get cache
    let retrieved = cache.get(&project_root, &file_mtimes).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().project_type, ProjectType::Library);
}

#[test]
fn test_cache_manager_get_nonexistent() {
    let cache = CacheManager::new();
    let project_root = PathBuf::from("/test/project");
    let file_mtimes = HashMap::new();

    let retrieved = cache.get(&project_root, &file_mtimes).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_cache_manager_invalidate() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root = PathBuf::from("/test/project");
    let file_mtimes = HashMap::new();

    // Set cache
    cache
        .set(&project_root, &context, file_mtimes.clone())
        .unwrap();
    assert_eq!(cache.entry_count().unwrap(), 1);

    // Invalidate
    cache.invalidate(&project_root).unwrap();
    assert_eq!(cache.entry_count().unwrap(), 0);

    // Verify it's gone
    let retrieved = cache.get(&project_root, &file_mtimes).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_cache_manager_clear() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root1 = PathBuf::from("/test/project1");
    let project_root2 = PathBuf::from("/test/project2");
    let file_mtimes = HashMap::new();

    // Set multiple entries
    cache
        .set(&project_root1, &context, file_mtimes.clone())
        .unwrap();
    cache
        .set(&project_root2, &context, file_mtimes.clone())
        .unwrap();
    assert_eq!(cache.entry_count().unwrap(), 2);

    // Clear all
    cache.clear().unwrap();
    assert_eq!(cache.entry_count().unwrap(), 0);
}

#[test]
fn test_cache_manager_is_cached() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root = PathBuf::from("/test/project");
    let file_mtimes = HashMap::new();

    // Not cached initially
    assert!(!cache.is_cached(&project_root, &file_mtimes).unwrap());

    // Set cache
    cache
        .set(&project_root, &context, file_mtimes.clone())
        .unwrap();

    // Now it's cached
    assert!(cache.is_cached(&project_root, &file_mtimes).unwrap());
}

#[test]
fn test_cache_manager_statistics() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root = PathBuf::from("/test/project");
    let file_mtimes = HashMap::new();

    // Initial stats
    let stats = cache.statistics().unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);

    // Set and get (hit)
    cache
        .set(&project_root, &context, file_mtimes.clone())
        .unwrap();
    let _ = cache.get(&project_root, &file_mtimes.clone()).unwrap();

    let stats = cache.statistics().unwrap();
    assert_eq!(stats.hits, 1);

    // Get non-existent (miss)
    let _ = cache.get(&PathBuf::from("/other"), &file_mtimes).unwrap();

    let stats = cache.statistics().unwrap();
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_cache_manager_file_change_detection() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root = PathBuf::from("/test/project");
    let test_file = PathBuf::from("/test/project/src/main.rs");

    // Create initial file mtimes
    let mut file_mtimes = HashMap::new();
    file_mtimes.insert(test_file.clone(), SystemTime::now());

    // Set cache
    cache
        .set(&project_root, &context, file_mtimes.clone())
        .unwrap();
    assert!(cache.is_cached(&project_root, &file_mtimes).unwrap());

    // Simulate file modification
    let mut modified_mtimes = HashMap::new();
    modified_mtimes.insert(
        test_file,
        SystemTime::now() + std::time::Duration::from_secs(1),
    );

    // Cache should be invalidated
    assert!(!cache.is_cached(&project_root, &modified_mtimes).unwrap());
}

#[test]
fn test_cache_manager_file_deletion_detection() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root = PathBuf::from("/test/project");
    let test_file = PathBuf::from("/test/project/src/main.rs");

    // Create initial file mtimes
    let mut file_mtimes = HashMap::new();
    file_mtimes.insert(test_file, SystemTime::now());

    // Set cache
    cache.set(&project_root, &context, file_mtimes).unwrap();

    // Simulate file deletion (empty mtimes)
    let empty_mtimes = HashMap::new();

    // Cache should be invalidated
    assert!(!cache.is_cached(&project_root, &empty_mtimes).unwrap());
}

#[test]
fn test_cache_manager_file_addition_detection() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root = PathBuf::from("/test/project");
    let test_file1 = PathBuf::from("/test/project/src/main.rs");

    // Create initial file mtimes with one file
    let mut file_mtimes = HashMap::new();
    file_mtimes.insert(test_file1, SystemTime::now());

    // Set cache
    cache
        .set(&project_root, &context, file_mtimes.clone())
        .unwrap();

    // Simulate file addition
    let test_file2 = PathBuf::from("/test/project/src/lib.rs");
    let mut new_mtimes = file_mtimes.clone();
    new_mtimes.insert(test_file2, SystemTime::now());

    // Cache should be invalidated
    assert!(!cache.is_cached(&project_root, &new_mtimes).unwrap());
}

#[test]
fn test_change_detector_new() {
    let detector = ChangeDetector::new();
    assert_eq!(detector.tracked_file_count(), 0);
}

#[test]
fn test_change_detector_skip_patterns() {
    let detector = ChangeDetector::new();

    // Should skip hidden files
    assert!(detector.should_skip(std::path::Path::new(".hidden")));
    assert!(detector.should_skip(std::path::Path::new(".git")));

    // Should skip common directories
    assert!(detector.should_skip(std::path::Path::new("node_modules")));
    assert!(detector.should_skip(std::path::Path::new("target")));
    assert!(detector.should_skip(std::path::Path::new("__pycache__")));

    // Should not skip regular files
    assert!(!detector.should_skip(std::path::Path::new("src")));
    assert!(!detector.should_skip(std::path::Path::new("main.rs")));
}

#[test]
fn test_change_detector_clear() {
    let mut detector = ChangeDetector::new();
    // We can't directly insert into file_mtimes since it's private,
    // but we can verify that clear() works by checking tracked_file_count
    assert_eq!(detector.tracked_file_count(), 0);

    detector.clear();
    assert_eq!(detector.tracked_file_count(), 0);
}

#[test]
fn test_cache_stats_tracker_new() {
    let tracker = CacheStatsTracker::new();
    let stats = tracker.get_stats().unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.invalidations, 0);
}

#[test]
fn test_cache_stats_tracker_record_hit() {
    let tracker = CacheStatsTracker::new();
    tracker.record_hit(1.5);

    let stats = tracker.get_stats().unwrap();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_cache_stats_tracker_record_miss() {
    let tracker = CacheStatsTracker::new();
    tracker.record_miss();

    let stats = tracker.get_stats().unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_cache_stats_tracker_record_store() {
    let tracker = CacheStatsTracker::new();
    tracker.record_store(2.0, 1024);

    let stats = tracker.get_stats().unwrap();
    assert_eq!(stats.size_bytes, 1024);
}

#[test]
fn test_cache_stats_tracker_record_invalidation() {
    let tracker = CacheStatsTracker::new();
    tracker.record_invalidation();

    let stats = tracker.get_stats().unwrap();
    assert_eq!(stats.invalidations, 1);
}

#[test]
fn test_cache_stats_tracker_set_entry_count() {
    let tracker = CacheStatsTracker::new();
    tracker.set_entry_count(42);

    let stats = tracker.get_stats().unwrap();
    assert_eq!(stats.entry_count, 42);
}

#[test]
fn test_cache_stats_tracker_reset() {
    let tracker = CacheStatsTracker::new();
    tracker.record_hit(1.5);
    tracker.record_miss();
    tracker.record_invalidation();

    let stats_before = tracker.get_stats().unwrap();
    assert!(stats_before.hits > 0 || stats_before.misses > 0);

    tracker.reset();

    let stats_after = tracker.get_stats().unwrap();
    assert_eq!(stats_after.hits, 0);
    assert_eq!(stats_after.misses, 0);
    assert_eq!(stats_after.invalidations, 0);
}

#[test]
fn test_cache_stats_tracker_hit_rate() {
    let tracker = CacheStatsTracker::new();
    tracker.record_hit(1.5);
    tracker.record_hit(1.5);
    tracker.record_miss();

    let stats = tracker.get_stats().unwrap();
    let hit_rate = stats.hit_rate();
    assert!((hit_rate - 66.66).abs() < 1.0); // Approximately 66.66%
}

#[test]
fn test_cache_stats_tracker_efficiency_score() {
    let tracker = CacheStatsTracker::new();
    tracker.record_hit(1.5);
    tracker.record_hit(1.5);
    tracker.record_hit(1.5);
    tracker.record_miss();

    let stats = tracker.get_stats().unwrap();
    let efficiency = stats.efficiency_score();
    assert!(efficiency > 0.0);
    assert!(efficiency <= 100.0);
}

#[test]
fn test_cache_stats_tracker_summary() {
    let tracker = CacheStatsTracker::new();
    tracker.record_hit(1.5);
    tracker.record_miss();

    let summary = tracker.summary();
    assert!(summary.contains("Cache Statistics"));
    assert!(summary.contains("Hits: 1"));
    assert!(summary.contains("Misses: 1"));
}

#[test]
fn test_cache_manager_multiple_projects() {
    let cache = CacheManager::new();
    let context = create_test_context();
    let project_root1 = PathBuf::from("/test/project1");
    let project_root2 = PathBuf::from("/test/project2");
    let file_mtimes = HashMap::new();

    // Set cache for both projects
    cache
        .set(&project_root1, &context, file_mtimes.clone())
        .unwrap();
    cache
        .set(&project_root2, &context, file_mtimes.clone())
        .unwrap();

    assert_eq!(cache.entry_count().unwrap(), 2);

    // Invalidate one project
    cache.invalidate(&project_root1).unwrap();
    assert_eq!(cache.entry_count().unwrap(), 1);

    // Other project should still be cached
    assert!(cache.is_cached(&project_root2, &file_mtimes).unwrap());
}

#[test]
fn test_cache_manager_default_ttl() {
    let cache = CacheManager::new();
    assert_eq!(cache.default_ttl, std::time::Duration::from_secs(3600));
}

#[test]
fn test_cache_manager_custom_ttl() {
    let ttl = std::time::Duration::from_secs(300);
    let cache = CacheManager::with_ttl(ttl);
    assert_eq!(cache.default_ttl, ttl);
}
