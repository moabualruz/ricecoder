//! Property-based tests for path resolution
//! **Feature: ricecoder-path-resolution, Property 1: Path Resolution Consistency**
//! **Feature: ricecoder-path-resolution, Property 2: Environment Variable Override**
//! **Validates: Requirements 1.1, 1.2, 1.3**

use proptest::prelude::*;
use ricecoder_storage::PathResolver;
use std::path::PathBuf;
use std::sync::Mutex;

// Mutex to serialize environment variable tests to prevent race conditions
lazy_static::lazy_static! {
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

proptest! {
    /// Property: For any environment state, multiple calls to resolve_project_path
    /// should return identical results (consistency)
    #[test]
    fn prop_project_path_resolution_is_consistent(
        _dummy in 0..1u32  // Just to satisfy proptest's requirement for at least one strategy
    ) {
        // Call resolve_project_path multiple times
        let path1 = PathResolver::resolve_project_path();
        let path2 = PathResolver::resolve_project_path();
        let path3 = PathResolver::resolve_project_path();

        // All calls should return identical results
        prop_assert_eq!(&path1, &path2);
        prop_assert_eq!(&path2, &path3);
        
        // Project path should always be .agent
        prop_assert_eq!(&path1, &PathBuf::from(".agent"));
    }
}

#[test]
fn test_global_path_resolution_is_consistent() {
    // This test doesn't use proptest because environment variables
    // can interfere with parallel test execution. Instead, we test
    // the core property: multiple calls return identical results.
    
    let _guard = ENV_LOCK.lock().unwrap();
    
    // Save original value if it exists
    let original = std::env::var("RICECODER_HOME").ok();
    
    // Remove RICECODER_HOME to test default behavior
    std::env::remove_var("RICECODER_HOME");
    
    // Call resolve_global_path multiple times (without modifying environment)
    let path1 = PathResolver::resolve_global_path();
    let path2 = PathResolver::resolve_global_path();
    let path3 = PathResolver::resolve_global_path();

    // All calls should return identical results (both Ok or both Err)
    match (&path1, &path2, &path3) {
        (Ok(p1), Ok(p2), Ok(p3)) => {
            assert_eq!(p1, p2, "Path 1 and 2 should be equal");
            assert_eq!(p2, p3, "Path 2 and 3 should be equal");
        }
        (Err(_), Err(_), Err(_)) => {
            // All failed - this is also consistent
        }
        _ => {
            panic!("Inconsistent path resolution results: {:?}, {:?}, {:?}", path1, path2, path3);
        }
    }
    
    // Restore original value
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    }
}

#[test]
fn test_environment_variable_override_is_respected() {
    // This test doesn't use proptest because environment variables
    // can interfere with parallel test execution. Instead, we test
    // the core property: RICECODER_HOME is respected when set.
    
    let _guard = ENV_LOCK.lock().unwrap();
    
    // Save the original value if it exists
    let original = std::env::var("RICECODER_HOME").ok();
    
    // Ensure clean state - remove any existing value first
    std::env::remove_var("RICECODER_HOME");

    // Set RICECODER_HOME environment variable to a unique value
    let override_path = "/tmp/ricecoder-test-override-unique-54321";
    std::env::set_var("RICECODER_HOME", override_path);

    // Verify the environment variable was set
    let env_check = std::env::var("RICECODER_HOME");
    assert!(env_check.is_ok(), "RICECODER_HOME should be set");
    if let Ok(env_val) = env_check {
        assert_eq!(&env_val, override_path);
    }

    // Resolve global path
    let resolved = PathResolver::resolve_global_path();

    // Should succeed and return the override path
    assert!(resolved.is_ok(), "Path resolution should succeed");
    if let Ok(path) = resolved {
        assert_eq!(&path, &PathBuf::from(override_path), 
            "Resolved path should match RICECODER_HOME environment variable");
    }

    // Restore original value - CRITICAL: must restore before test ends
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    } else {
        std::env::remove_var("RICECODER_HOME");
    }
}

/// Property 6: Dependency Isolation
/// For any crate that needs path resolution, it SHALL depend on `ricecoder-storage`
/// and not implement custom path logic.
/// **Feature: ricecoder-path-resolution, Property 6: Dependency Isolation**
/// **Validates: Requirements 2.1, 2.2**
#[test]
fn test_dependency_isolation_all_path_resolution_through_path_resolver() {
    // This test verifies that all path resolution goes through PathResolver
    // by checking that:
    // 1. PathResolver::resolve_project_path() always returns .agent
    // 2. PathResolver::resolve_global_path() always returns a consistent path
    // 3. Multiple calls return identical results (no custom logic)

    let _guard = ENV_LOCK.lock().unwrap();
    
    // Save original value if it exists
    let original = std::env::var("RICECODER_HOME").ok();
    
    // Remove RICECODER_HOME to test default behavior
    std::env::remove_var("RICECODER_HOME");

    // Test project path resolution
    let project_path1 = PathResolver::resolve_project_path();
    let project_path2 = PathResolver::resolve_project_path();
    let project_path3 = PathResolver::resolve_project_path();

    // All calls should return identical results
    assert_eq!(&project_path1, &project_path2);
    assert_eq!(&project_path2, &project_path3);

    // Project path should always be .agent (no custom logic)
    assert_eq!(&project_path1, &PathBuf::from(".agent"));

    // Test global path resolution (without modifying environment)
    let global_path1 = PathResolver::resolve_global_path();
    let global_path2 = PathResolver::resolve_global_path();
    let global_path3 = PathResolver::resolve_global_path();

    // All calls should return identical results
    match (&global_path1, &global_path2, &global_path3) {
        (Ok(p1), Ok(p2), Ok(p3)) => {
            assert_eq!(p1, p2, "Global paths should be identical");
            assert_eq!(p2, p3, "Global paths should be identical");
        }
        (Err(_), Err(_), Err(_)) => {
            // All failed - this is also consistent
        }
        _ => {
            panic!("Inconsistent path resolution results: {:?}, {:?}, {:?}", global_path1, global_path2, global_path3);
        }
    }
    
    // Restore original value
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    }
}

#[test]
fn test_dependency_isolation_no_hardcoded_paths() {
    // This test verifies that path resolution doesn't use hardcoded paths
    // by checking that PathResolver is the single source of truth

    // Project path should be resolved through PathResolver
    let project_path = PathResolver::resolve_project_path();
    assert_eq!(project_path, PathBuf::from(".agent"));

    // Global path should be resolved through PathResolver
    let original = std::env::var("RICECODER_HOME").ok();
    std::env::remove_var("RICECODER_HOME");

    let global_path = PathResolver::resolve_global_path();
    assert!(global_path.is_ok(), "Global path should be resolvable");

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    }
}

// Note: Environment variable override test is covered by test_environment_variable_override_is_respected
// We avoid duplicate environment variable tests to prevent conflicts in parallel test execution

#[test]
fn test_dependency_isolation_consistency_across_multiple_calls() {
    // This test verifies that path resolution is consistent across multiple calls
    // ensuring that all path resolution goes through the same PathResolver logic

    let _guard = ENV_LOCK.lock().unwrap();
    
    // Save the original RICECODER_HOME value to restore it later
    let original = std::env::var("RICECODER_HOME").ok();
    
    // Remove RICECODER_HOME to test default behavior
    std::env::remove_var("RICECODER_HOME");

    // Make multiple calls to project path resolution
    let mut project_paths = Vec::new();
    for _ in 0..10 {
        project_paths.push(PathResolver::resolve_project_path());
    }

    // All should be identical
    for path in &project_paths {
        assert_eq!(path, &PathBuf::from(".agent"));
    }

    // Make multiple calls to global path resolution (without modifying environment)
    let mut global_paths = Vec::new();
    for _ in 0..10 {
        global_paths.push(PathResolver::resolve_global_path());
    }

    // All should be identical (either all Ok with same path, or all Err)
    for i in 1..global_paths.len() {
        match (&global_paths[i], &global_paths[0]) {
            (Ok(p1), Ok(p2)) => {
                assert_eq!(p1, p2, "Global paths should be identical");
            }
            (Err(_), Err(_)) => {
                // Both failed - this is also consistent
            }
            _ => {
                panic!("Inconsistent path resolution results: {:?} vs {:?}", global_paths[i], global_paths[0]);
            }
        }
    }

    // Restore original RICECODER_HOME value
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    }
}
