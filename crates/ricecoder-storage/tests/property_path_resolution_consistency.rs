//! Property-based tests for path resolution consistency
//! **Feature: ricecoder-storage, Property 1: Path Resolution Consistency**
//! **Validates: Requirements 4.1, 4.4**

use std::{path::PathBuf, sync::Mutex};

use proptest::prelude::*;
use ricecoder_storage::PathResolver;

// Mutex to serialize environment variable access in tests
lazy_static::lazy_static! {
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

/// Strategy for generating valid directory paths
/// Generates paths that are valid on both Windows and Unix systems
fn valid_path_strategy() -> impl Strategy<Value = String> {
    // Generate simple alphanumeric paths to avoid special character issues
    r"[a-zA-Z0-9_\-]+" // Only alphanumeric, underscore, dash
        .prop_map(|s| format!("/tmp/ricecoder_test_{}", s))
        .prop_filter("Path should be non-empty", |s| !s.is_empty())
}

proptest! {
    /// Property 1: Path Resolution Consistency
    /// For any given environment state, calling PathResolver::resolve_global_path()
    /// multiple times should always return the same path. This ensures that path
    /// resolution is deterministic and consistent across the application.
    #[test]
    fn prop_path_resolution_consistency(path in valid_path_strategy()) {
        // Lock to prevent parallel test interference
        let _lock = ENV_LOCK.lock().unwrap();

        // Set RICECODER_HOME to the generated path
        std::env::set_var("RICECODER_HOME", &path);

        // Resolve the path multiple times
        let resolved1 = PathResolver::resolve_global_path()
            .expect("First resolution should succeed");
        let resolved2 = PathResolver::resolve_global_path()
            .expect("Second resolution should succeed");
        let resolved3 = PathResolver::resolve_global_path()
            .expect("Third resolution should succeed");

        // All resolutions should be identical
        assert_eq!(
            resolved1, resolved2,
            "First and second resolutions should be identical"
        );
        assert_eq!(
            resolved2, resolved3,
            "Second and third resolutions should be identical"
        );
        assert_eq!(
            resolved1, resolved3,
            "First and third resolutions should be identical"
        );

        // All should match the environment variable
        assert_eq!(
            resolved1,
            PathBuf::from(&path),
            "Resolved path should match RICECODER_HOME"
        );

        // Clean up
        std::env::remove_var("RICECODER_HOME");
    }
}
