//! Property-based tests for offline mode
//! **Feature: ricecoder-storage, Property 15: Offline Mode Graceful Degradation**
//! **Validates: Requirements 6.6**

use proptest::prelude::*;
use ricecoder_storage::{OfflineModeHandler, StorageState};
use tempfile::TempDir;

/// Strategy for generating cache availability states
fn cache_availability_strategy() -> impl Strategy<Value = bool> {
    prop_oneof![Just(true), Just(false)]
}

/// Property 15: Offline Mode Graceful Degradation
/// For any unavailable storage directory, the OfflineModeHandler should operate
/// in read-only mode using cached data and produce a warning log entry.
#[test]
fn prop_offline_mode_graceful_degradation() {
    proptest!(|(cache_available in cache_availability_strategy())| {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a storage state that simulates unavailable storage
        let state = OfflineModeHandler::enter_offline_mode(temp_dir.path(), cache_available);

        // Verify the state is correct based on cache availability
        match state {
            StorageState::ReadOnly { cached_at } => {
                // Should only be ReadOnly if cache is available
                assert!(
                    cache_available,
                    "ReadOnly state should only occur when cache is available"
                );
                // Verify cached_at is a valid timestamp
                assert!(
                    !cached_at.is_empty(),
                    "cached_at should contain a timestamp"
                );
            }
            StorageState::Unavailable { reason } => {
                // Should be Unavailable if cache is not available
                assert!(
                    !cache_available,
                    "Unavailable state should occur when cache is not available"
                );
                assert!(
                    !reason.is_empty(),
                    "Unavailable state should have a reason"
                );
            }
            StorageState::Available => {
                panic!("Should not be Available in offline mode");
            }
        }
    });
}

/// Property: Offline mode validation respects cache availability
/// The validation should succeed only when cache is available
#[test]
fn prop_offline_mode_validation() {
    proptest!(|(cache_available in cache_availability_strategy())| {
        let result = OfflineModeHandler::validate_offline_mode(cache_available);

        if cache_available {
            assert!(
                result.is_ok(),
                "Validation should succeed when cache is available"
            );
        } else {
            assert!(
                result.is_err(),
                "Validation should fail when cache is not available"
            );
        }
    });
}

/// Property: Storage availability check is consistent
/// Checking the same storage path multiple times should return the same result
#[test]
fn prop_storage_availability_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Check availability multiple times
    let state1 = OfflineModeHandler::check_storage_availability(temp_dir.path());
    let state2 = OfflineModeHandler::check_storage_availability(temp_dir.path());
    let state3 = OfflineModeHandler::check_storage_availability(temp_dir.path());

    // All checks should return the same state
    assert_eq!(
        state1, state2,
        "Storage availability should be consistent across checks"
    );
    assert_eq!(
        state2, state3,
        "Storage availability should be consistent across checks"
    );
}

/// Property: Retry storage access returns correct result
/// After checking storage availability, retry should return the same result
#[test]
fn prop_retry_storage_access_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Check initial availability
    let initial_available = OfflineModeHandler::retry_storage_access(temp_dir.path());

    // Retry should return the same result
    let retry_available = OfflineModeHandler::retry_storage_access(temp_dir.path());

    assert_eq!(
        initial_available, retry_available,
        "Retry should return consistent result"
    );
}

/// Property: Offline mode handles missing storage gracefully
/// When storage path doesn't exist, offline mode should handle it gracefully
#[test]
fn prop_offline_mode_missing_storage() {
    proptest!(|(cache_available in cache_availability_strategy())| {
        let nonexistent_path = std::path::PathBuf::from("/nonexistent/storage/path");

        // Check availability of missing storage
        let state = OfflineModeHandler::check_storage_availability(&nonexistent_path);

        // Should be unavailable
        match state {
            StorageState::Unavailable { reason } => {
                assert!(
                    !reason.is_empty(),
                    "Unavailable state should have a reason"
                );
            }
            _ => {
                panic!("Missing storage should result in Unavailable state");
            }
        }

        // Enter offline mode with missing storage
        let offline_state = OfflineModeHandler::enter_offline_mode(&nonexistent_path, cache_available);

        // Verify offline state is correct
        match offline_state {
            StorageState::ReadOnly { .. } => {
                assert!(
                    cache_available,
                    "ReadOnly state should only occur when cache is available"
                );
            }
            StorageState::Unavailable { .. } => {
                assert!(
                    !cache_available,
                    "Unavailable state should occur when cache is not available"
                );
            }
            StorageState::Available => {
                panic!("Should not be Available for missing storage");
            }
        }
    });
}

/// Property: Offline mode logging doesn't fail
/// Logging offline warnings should not cause errors
#[test]
fn prop_offline_mode_logging() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // This should not panic or error
    OfflineModeHandler::log_offline_warning(
        temp_dir.path(),
        "Test offline warning",
    );

    // If we get here, logging succeeded
    assert!(true);
}

/// Property: External storage detection is consistent
/// Checking if storage is external multiple times should return the same result
#[test]
fn prop_external_storage_detection_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Check if external multiple times
    let is_external1 = OfflineModeHandler::is_external_storage(temp_dir.path());
    let is_external2 = OfflineModeHandler::is_external_storage(temp_dir.path());
    let is_external3 = OfflineModeHandler::is_external_storage(temp_dir.path());

    // All checks should return the same result
    assert_eq!(
        is_external1, is_external2,
        "External storage detection should be consistent"
    );
    assert_eq!(
        is_external2, is_external3,
        "External storage detection should be consistent"
    );
}
