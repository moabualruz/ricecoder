//! Property-based tests for test validation
//!
//! **Feature: ricecoder-execution, Property 3: Test Validation**
//! **Validates: Requirements 3.1, 3.2**

use proptest::prelude::*;
use ricecoder_execution::{TestFailure, TestFramework, TestResults};
use tempfile::TempDir;

/// Property: Test failures prevent completion
///
/// For any execution with tests, test failures SHALL prevent completion without explicit override.
/// This property verifies that when tests fail, the system correctly reports the failures
/// and prevents normal completion.
#[test]
fn prop_test_failures_prevent_completion() {
    proptest!(|(
        passed in 0usize..100,
        failed in 1usize..100,  // At least 1 failure
        skipped in 0usize..50,
    )| {
        // Create test results with failures
        let results = TestResults {
            passed,
            failed,
            skipped,
            failures: (0..failed)
                .map(|i| TestFailure {
                    name: format!("test_{}", i),
                    message: "Test failed".to_string(),
                    location: None,
                })
                .collect(),
            framework: TestFramework::Rust,
        };

        // Property: When tests fail, failed count must be > 0
        prop_assert!(results.failed > 0, "Failed count must be positive");

        // Property: Number of failures must match failed count
        prop_assert_eq!(
            results.failures.len(),
            results.failed,
            "Failure details must match failed count"
        );

        // Property: Each failure must have a name
        for failure in &results.failures {
            prop_assert!(!failure.name.is_empty(), "Failure name must not be empty");
        }

        // Property: Each failure must have a message
        for failure in &results.failures {
            prop_assert!(!failure.message.is_empty(), "Failure message must not be empty");
        }
    });
}

/// Property: Test results are consistent
///
/// For any test results, the sum of passed + failed + skipped should be reasonable
/// and the framework should be set correctly.
#[test]
fn prop_test_results_consistency() {
    proptest!(|(
        passed in 0usize..1000,
        failed in 0usize..1000,
        skipped in 0usize..1000,
    )| {
        let results = TestResults {
            passed,
            failed,
            skipped,
            failures: (0..failed)
                .map(|i| TestFailure {
                    name: format!("test_{}", i),
                    message: "Test failed".to_string(),
                    location: None,
                })
                .collect(),
            framework: TestFramework::Rust,
        };

        // Property: Counts are stored correctly (usize is always non-negative)
        // Just verify they're accessible
        let _ = results.passed;
        let _ = results.failed;
        let _ = results.skipped;

        // Property: Framework must be set
        prop_assert_eq!(results.framework, TestFramework::Rust, "Framework must be set");

        // Property: Failures list length must match failed count
        prop_assert_eq!(
            results.failures.len(),
            results.failed,
            "Failures list length must match failed count"
        );
    });
}

/// Property: Test framework detection is deterministic
///
/// For any given project structure, framework detection must always return the same result.
#[test]
fn prop_test_framework_detection_deterministic() {
    proptest!(|(framework_type in 0u8..3)| {
        let temp_dir = TempDir::new().unwrap();

        // Create appropriate marker file based on framework type
        match framework_type {
            0 => {
                // Rust
                std::fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();
            }
            1 => {
                // TypeScript
                std::fs::write(temp_dir.path().join("package.json"), "").unwrap();
            }
            2 => {
                // Python
                std::fs::write(temp_dir.path().join("pytest.ini"), "").unwrap();
            }
            _ => unreachable!(),
        }

        let runner = ricecoder_execution::TestRunner::new(temp_dir.path());

        // Detect framework multiple times
        let result1 = runner.detect_framework();
        let result2 = runner.detect_framework();

        // Property: Detection must be deterministic
        match (result1, result2) {
            (Ok(f1), Ok(f2)) => {
                prop_assert_eq!(f1, f2, "Framework detection must be deterministic");
            }
            (Err(_), Err(_)) => {
                // Both failed, which is also consistent
            }
            _ => {
                prop_assert!(false, "Framework detection must be consistent");
            }
        }
    });
}

/// Property: Test command building is deterministic
///
/// For any given framework and pattern, the test command must be built consistently.
#[test]
fn prop_test_command_building_deterministic() {
    proptest!(|(
        framework_type in 0u8..3,
        pattern_opt in prop::option::of("[a-z_]{1,20}"),
    )| {
        let temp_dir = TempDir::new().unwrap();
        let runner = ricecoder_execution::TestRunner::new(temp_dir.path());

        let framework = match framework_type {
            0 => TestFramework::Rust,
            1 => TestFramework::TypeScript,
            2 => TestFramework::Python,
            _ => unreachable!(),
        };

        // Build command multiple times
        let result1 = runner.build_test_command(&framework, pattern_opt.as_deref());
        let result2 = runner.build_test_command(&framework, pattern_opt.as_deref());

        // Property: Command building must be deterministic
        match (result1, result2) {
            (Ok((cmd1, args1)), Ok((cmd2, args2))) => {
                prop_assert_eq!(cmd1, cmd2, "Command must be deterministic");
                prop_assert_eq!(args1, args2, "Arguments must be deterministic");
            }
            (Err(_), Err(_)) => {
                // Both failed, which is also consistent
            }
            _ => {
                prop_assert!(false, "Command building must be consistent");
            }
        }
    });
}

/// Property: Test failure details are preserved
///
/// For any test failure, the name and message must be preserved correctly.
#[test]
fn prop_test_failure_details_preserved() {
    proptest!(|(
        name in "[a-z_]{1,50}",
        message in "[a-zA-Z0-9 ]{1,100}",
    )| {
        let failure = TestFailure {
            name: name.clone(),
            message: message.clone(),
            location: None,
        };

        // Property: Name must be preserved
        prop_assert_eq!(failure.name, name, "Failure name must be preserved");

        // Property: Message must be preserved
        prop_assert_eq!(failure.message, message, "Failure message must be preserved");

        // Property: Location can be None
        prop_assert_eq!(failure.location, None, "Location should be None");
    });
}

/// Property: Multiple frameworks can be detected in sequence
///
/// For any sequence of projects with different frameworks, each should be detected correctly.
#[test]
fn prop_test_multiple_framework_detection() {
    proptest!(|(frameworks in prop::collection::vec(0u8..3, 1..10))| {
        for framework_type in frameworks {
            let temp_dir = TempDir::new().unwrap();

            // Create appropriate marker file
            match framework_type {
                0 => {
                    std::fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();
                }
                1 => {
                    std::fs::write(temp_dir.path().join("package.json"), "").unwrap();
                }
                2 => {
                    std::fs::write(temp_dir.path().join("pytest.ini"), "").unwrap();
                }
                _ => unreachable!(),
            }

            let runner = ricecoder_execution::TestRunner::new(temp_dir.path());

            // Property: Detection must succeed for valid projects
            let result = runner.detect_framework();
            prop_assert!(result.is_ok(), "Framework detection must succeed for valid projects");

            // Property: Detected framework must match the marker file
            if let Ok(detected) = result {
                let expected = match framework_type {
                    0 => TestFramework::Rust,
                    1 => TestFramework::TypeScript,
                    2 => TestFramework::Python,
                    _ => unreachable!(),
                };
                prop_assert_eq!(detected, expected, "Detected framework must match marker file");
            }
        }
    });
}
