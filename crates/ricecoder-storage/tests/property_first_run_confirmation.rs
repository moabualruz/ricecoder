//! Property-based tests for first-run confirmation
//! **Feature: ricecoder-storage, Property 14: First-Run Storage Confirmation**
//! **Validates: Requirements 6.4**

use proptest::prelude::*;
use ricecoder_storage::FirstRunHandler;
use tempfile::TempDir;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 14: First-Run Storage Confirmation
    /// For any first-time initialization, the FirstRunHandler should correctly
    /// detect that it's the first run before the marker file is created,
    /// and should correctly detect that it's not the first run after the marker is created.
    #[test]
    fn prop_first_run_detection(_dummy in Just(())) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Initially should be first run (no marker file)
        let is_first_before = FirstRunHandler::is_first_run(&path)
            .expect("Failed to check first run");
        assert!(
            is_first_before,
            "Should detect first run when marker doesn't exist"
        );

        // Mark as complete
        FirstRunHandler::mark_first_run_complete(&path)
            .expect("Failed to mark first run complete");

        // Should no longer be first run
        let is_first_after = FirstRunHandler::is_first_run(&path)
            .expect("Failed to check first run");
        assert!(
            !is_first_after,
            "Should not detect first run after marker is created"
        );
    }

    /// Property: First-run marker file is created correctly
    /// After marking first run as complete, the marker file should exist
    #[test]
    fn prop_first_run_marker_creation(_dummy in Just(())) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Mark as complete
        FirstRunHandler::mark_first_run_complete(&path)
            .expect("Failed to mark first run complete");

        // Verify marker file exists
        let marker_path = path.join(".ricecoder-initialized");
        assert!(
            marker_path.exists(),
            "Marker file should exist after marking first run complete"
        );
    }

    /// Property: First-run detection is idempotent
    /// Checking first-run status multiple times should always return the same result
    #[test]
    fn prop_first_run_detection_idempotent(_dummy in Just(())) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Check multiple times before marking
        let check1 = FirstRunHandler::is_first_run(&path)
            .expect("First check failed");
        let check2 = FirstRunHandler::is_first_run(&path)
            .expect("Second check failed");
        let check3 = FirstRunHandler::is_first_run(&path)
            .expect("Third check failed");

        assert_eq!(check1, check2, "First and second checks should match");
        assert_eq!(check2, check3, "Second and third checks should match");

        // Mark as complete
        FirstRunHandler::mark_first_run_complete(&path)
            .expect("Failed to mark first run complete");

        // Check multiple times after marking
        let check4 = FirstRunHandler::is_first_run(&path)
            .expect("Fourth check failed");
        let check5 = FirstRunHandler::is_first_run(&path)
            .expect("Fifth check failed");
        let check6 = FirstRunHandler::is_first_run(&path)
            .expect("Sixth check failed");

        assert_eq!(check4, check5, "Fourth and fifth checks should match");
        assert_eq!(check5, check6, "Fifth and sixth checks should match");

        // All after-marking checks should be false
        assert!(!check4, "After marking, should not be first run");
        assert!(!check5, "After marking, should not be first run");
        assert!(!check6, "After marking, should not be first run");
    }

    /// Property: Suggested path is always valid
    /// The suggested path should always be resolvable and contain .ricecoder
    #[test]
    fn prop_suggested_path_valid(_dummy in Just(())) {
        let path = FirstRunHandler::get_suggested_path()
            .expect("Failed to get suggested path");

        // Path should contain .ricecoder
        assert!(
            path.to_string_lossy().contains(".ricecoder"),
            "Suggested path should contain .ricecoder"
        );

        // Path should be absolute or relative (not empty)
        assert!(
            !path.as_os_str().is_empty(),
            "Suggested path should not be empty"
        );
    }

    /// Property: Multiple first-run completions are safe
    /// Marking first run as complete multiple times should not cause errors
    #[test]
    fn prop_multiple_first_run_completions(_dummy in Just(())) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Mark as complete multiple times
        FirstRunHandler::mark_first_run_complete(&path)
            .expect("First mark failed");
        FirstRunHandler::mark_first_run_complete(&path)
            .expect("Second mark failed");
        FirstRunHandler::mark_first_run_complete(&path)
            .expect("Third mark failed");

        // Should still not be first run
        let is_first = FirstRunHandler::is_first_run(&path)
            .expect("Failed to check first run");
        assert!(!is_first, "Should not be first run after multiple marks");
    }
}
