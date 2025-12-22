// Property-based tests for path resolution
// **Feature: ricecoder-path-resolution, Property 3: Project Path Correctness**
// **Validates: Requirements 1.4**

use std::path::PathBuf;

use proptest::prelude::*;
use ricecoder_storage::PathResolver;

/// Property 3: Project Path Correctness
/// For any project, the resolved project path SHALL be `.agent/` relative to the project root.
#[test]
fn prop_project_path_is_agent_directory() {
    proptest!(|(_i in 0..100)| {
        // Run the test multiple times to ensure consistency
        let project_path = PathResolver::resolve_project_path();

        // Project path should be .agent
        prop_assert_eq!(project_path, PathBuf::from(".agent"));
    });
}

/// Test that project path is always consistent
#[test]
fn test_project_path_consistency() {
    let path1 = PathResolver::resolve_project_path();
    let path2 = PathResolver::resolve_project_path();
    let path3 = PathResolver::resolve_project_path();

    assert_eq!(path1, path2);
    assert_eq!(path2, path3);
    assert_eq!(path1, PathBuf::from(".agent"));
}

/// Test that project path is always .agent
#[test]
fn test_project_path_always_agent() {
    let project_path = PathResolver::resolve_project_path();

    // Should be exactly .agent
    assert_eq!(project_path.to_str().unwrap(), ".agent");

    // Should have exactly one component
    let components: Vec<_> = project_path.components().collect();
    assert_eq!(components.len(), 1);
}

/// Test that project path can be joined with subdirectories
#[test]
fn test_project_path_can_join_subdirectories() {
    let project_path = PathResolver::resolve_project_path();

    // Should be able to join specs
    let specs_path = project_path.join("specs");
    assert_eq!(specs_path, PathBuf::from(".agent").join("specs"));

    // Should be able to join steering
    let steering_path = project_path.join("steering");
    assert_eq!(steering_path, PathBuf::from(".agent").join("steering"));

    // Should be able to join config
    let config_path = project_path.join("config");
    assert_eq!(config_path, PathBuf::from(".agent").join("config"));
}

/// Property 4: Global Path Correctness
/// For any system without `RICECODER_HOME` set, the resolved global path SHALL be either `~/.ricecoder/` or `~/Documents/.ricecoder/`.
#[test]
fn prop_global_path_correctness() {
    proptest!(|(_i in 0..100)| {
        // Ensure RICECODER_HOME is not set for this test
        std::env::remove_var("RICECODER_HOME");

        let global_path = PathResolver::resolve_global_path()
            .expect("Should resolve global path");

        // Global path should end with .ricecoder
        let path_str = global_path.to_str().expect("Path should be valid UTF-8");
        prop_assert!(
            path_str.ends_with(".ricecoder") || path_str.ends_with(".ricecoder/"),
            "Global path should end with .ricecoder, got: {}",
            path_str
        );
    });
}

/// Test that global path ends with .ricecoder
#[test]
fn test_global_path_ends_with_ricecoder() {
    // Ensure RICECODER_HOME is not set
    std::env::remove_var("RICECODER_HOME");

    let global_path = PathResolver::resolve_global_path().expect("Should resolve global path");

    let path_str = global_path.to_str().expect("Path should be valid UTF-8");
    assert!(
        path_str.ends_with(".ricecoder") || path_str.ends_with(".ricecoder/"),
        "Global path should end with .ricecoder, got: {}",
        path_str
    );
}

/// Test that global path respects RICECODER_HOME environment variable
#[test]
fn test_global_path_respects_ricecoder_home() {
    let test_path = "/tmp/test-ricecoder";
    std::env::set_var("RICECODER_HOME", test_path);

    let global_path = PathResolver::resolve_global_path().expect("Should resolve global path");

    assert_eq!(global_path, PathBuf::from(test_path));

    // Clean up
    std::env::remove_var("RICECODER_HOME");
}

/// Test that global path is consistent across multiple calls
#[test]
fn test_global_path_consistency() {
    // Ensure RICECODER_HOME is not set
    std::env::remove_var("RICECODER_HOME");

    let path1 = PathResolver::resolve_global_path().expect("Should resolve global path");
    let path2 = PathResolver::resolve_global_path().expect("Should resolve global path");
    let path3 = PathResolver::resolve_global_path().expect("Should resolve global path");

    assert_eq!(path1, path2);
    assert_eq!(path2, path3);
}
