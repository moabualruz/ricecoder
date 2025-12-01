//! Property-based tests for RICECODER_HOME override
//! **Feature: ricecoder-storage, Property 10: RICECODER_HOME Override**
//! **Validates: Requirements 6.2**

use proptest::prelude::*;
use ricecoder_storage::PathResolver;
use std::path::PathBuf;
use std::sync::Mutex;

// Mutex to serialize environment variable access in tests
lazy_static::lazy_static! {
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

/// Strategy for generating valid directory paths
/// Generates paths that are valid on both Windows and Unix systems
fn valid_path_strategy() -> impl Strategy<Value = String> {
    // Generate simple alphanumeric paths to avoid special character issues
    r"[a-zA-Z0-9_\-]+" // Only alphanumeric, underscore, dash
        .prop_map(|s| format!("/tmp/ricecoder_{}", s))
        .prop_filter("Path should be non-empty", |s| !s.is_empty())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 10: RICECODER_HOME Override
    /// For any valid directory path set in the RICECODER_HOME environment variable,
    /// the PathResolver should use that path as the global storage location
    /// instead of the default.
    #[test]
    fn prop_ricecoder_home_override(path in valid_path_strategy()) {
        // Lock to prevent parallel test interference
        let _lock = ENV_LOCK.lock().unwrap();

        // Set RICECODER_HOME to the generated path
        std::env::set_var("RICECODER_HOME", &path);

        // Resolve the global path
        let resolved = PathResolver::resolve_global_path()
            .expect("Should resolve path with RICECODER_HOME set");

        // Verify the resolved path matches the environment variable
        assert_eq!(
            resolved,
            PathBuf::from(&path),
            "Resolved path should match RICECODER_HOME environment variable"
        );

        // Clean up
        std::env::remove_var("RICECODER_HOME");
    }

    /// Property: RICECODER_HOME takes precedence over default paths
    /// When RICECODER_HOME is set, it should always be used regardless of
    /// whether Documents folder exists
    #[test]
    fn prop_ricecoder_home_precedence(path in valid_path_strategy()) {
        // Lock to prevent parallel test interference
        let _lock = ENV_LOCK.lock().unwrap();

        // Set RICECODER_HOME
        std::env::set_var("RICECODER_HOME", &path);

        // Resolve path multiple times to ensure consistency
        let resolved1 = PathResolver::resolve_global_path()
            .expect("First resolution should succeed");
        let resolved2 = PathResolver::resolve_global_path()
            .expect("Second resolution should succeed");

        // Both resolutions should be identical
        assert_eq!(
            resolved1, resolved2,
            "Multiple resolutions should produce identical results"
        );

        // Both should match the environment variable
        assert_eq!(
            resolved1,
            PathBuf::from(&path),
            "Should use RICECODER_HOME value"
        );

        // Clean up
        std::env::remove_var("RICECODER_HOME");
    }
}
