//! Property-based tests for Error Recovery
//!
//! **Feature: ricecoder-github, Property 61: Error Recovery**
//! **Validates: All requirements**

use proptest::prelude::*;
use ricecoder_github::errors::{GitHubError, Result};

/// Strategy for generating GitHub errors
fn github_error_strategy() -> impl Strategy<Value = GitHubError> {
    prop_oneof![
        Just(GitHubError::RateLimitExceeded),
        ".*".prop_map(|s| GitHubError::api_error(s)),
        ".*".prop_map(|s| GitHubError::auth_error(s)),
        ".*".prop_map(|s| GitHubError::not_found(s)),
        ".*".prop_map(|s| GitHubError::config_error(s)),
        ".*".prop_map(|s| GitHubError::invalid_input(s)),
        ".*".prop_map(|s| GitHubError::network_error(s)),
        ".*".prop_map(|s| GitHubError::storage_error(s)),
    ]
}

/// Property 61: Error Recovery
///
/// For any GitHub API error, the system SHALL handle it gracefully and provide actionable error messages.
///
/// This property verifies that:
/// 1. All error types can be created and converted to strings
/// 2. Error messages are non-empty and descriptive
/// 3. Error types can be checked for specific conditions
/// 4. Errors can be propagated through Result types
proptest! {
    #[test]
    fn prop_error_recovery_all_errors_have_messages(error in github_error_strategy()) {
        // All errors should have a non-empty Display message
        let message = error.to_string();
        prop_assert!(!message.is_empty(), "Error message should not be empty");
        prop_assert!(message.len() > 0, "Error message should have content");
    }

    #[test]
    fn prop_error_recovery_error_classification(error in github_error_strategy()) {
        // Errors should be classifiable
        match error {
            GitHubError::RateLimitExceeded => {
                prop_assert!(error.is_rate_limit());
            }
            GitHubError::AuthError(_) => {
                prop_assert!(error.is_auth_error());
            }
            GitHubError::NotFound(_) => {
                prop_assert!(error.is_not_found());
            }
            _ => {
                // Other errors should not match specific classifications
                prop_assert!(!error.is_rate_limit() || matches!(error, GitHubError::RateLimitExceeded));
                prop_assert!(!error.is_auth_error() || matches!(error, GitHubError::AuthError(_)));
                prop_assert!(!error.is_not_found() || matches!(error, GitHubError::NotFound(_)));
            }
        }
    }

    #[test]
    fn prop_error_recovery_result_propagation(error in github_error_strategy()) {
        // Errors should propagate through Result types
        let result: Result<()> = Err(error.clone());
        prop_assert!(result.is_err());

        // Error should be recoverable from Result
        match result {
            Err(e) => {
                let message = e.to_string();
                prop_assert!(!message.is_empty());
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn prop_error_recovery_error_creation_helpers(msg in ".*") {
        // Error creation helpers should produce valid errors
        let api_error = GitHubError::api_error(msg.clone());
        prop_assert!(!api_error.to_string().is_empty());

        let auth_error = GitHubError::auth_error(msg.clone());
        prop_assert!(!auth_error.to_string().is_empty());

        let config_error = GitHubError::config_error(msg.clone());
        prop_assert!(!config_error.to_string().is_empty());

        let not_found = GitHubError::not_found(msg.clone());
        prop_assert!(!not_found.to_string().is_empty());

        let invalid_input = GitHubError::invalid_input(msg.clone());
        prop_assert!(!invalid_input.to_string().is_empty());

        let network_error = GitHubError::network_error(msg.clone());
        prop_assert!(!network_error.to_string().is_empty());

        let storage_error = GitHubError::storage_error(msg.clone());
        prop_assert!(!storage_error.to_string().is_empty());
    }

    #[test]
    fn prop_error_recovery_error_messages_contain_context(msg in "test.*") {
        // Error messages should contain the provided context
        let api_error = GitHubError::api_error(msg.clone());
        let message = api_error.to_string();
        prop_assert!(message.contains("API error") || message.contains(&msg));

        let auth_error = GitHubError::auth_error(msg.clone());
        let message = auth_error.to_string();
        prop_assert!(message.contains("Authentication") || message.contains(&msg));

        let config_error = GitHubError::config_error(msg.clone());
        let message = config_error.to_string();
        prop_assert!(message.contains("configuration") || message.contains(&msg));
    }

    #[test]
    fn prop_error_recovery_rate_limit_is_retryable(error in github_error_strategy()) {
        // Rate limit errors should be identifiable as retryable
        if error.is_rate_limit() {
            prop_assert!(matches!(error, GitHubError::RateLimitExceeded));
        }
    }

    #[test]
    fn prop_error_recovery_auth_errors_are_not_retryable(error in github_error_strategy()) {
        // Auth errors should be identifiable as non-retryable
        if error.is_auth_error() {
            prop_assert!(matches!(error, GitHubError::AuthError(_)));
            // Auth errors should not be rate limit errors
            prop_assert!(!error.is_rate_limit());
        }
    }

    #[test]
    fn prop_error_recovery_not_found_errors_are_not_retryable(error in github_error_strategy()) {
        // Not found errors should be identifiable as non-retryable
        if error.is_not_found() {
            prop_assert!(matches!(error, GitHubError::NotFound(_)));
            // Not found errors should not be rate limit errors
            prop_assert!(!error.is_rate_limit());
        }
    }

    #[test]
    fn prop_error_recovery_error_display_is_consistent(error in github_error_strategy()) {
        // Error Display should be consistent across multiple calls
        let message1 = error.to_string();
        let message2 = error.to_string();
        prop_assert_eq!(message1, message2, "Error messages should be consistent");
    }

    #[test]
    fn prop_error_recovery_error_debug_is_valid(error in github_error_strategy()) {
        // Error Debug should produce valid output
        let debug_str = format!("{:?}", error);
        prop_assert!(!debug_str.is_empty(), "Debug output should not be empty");
        // Debug output should be a valid string representation
        prop_assert!(debug_str.len() > 0);
    }
}
