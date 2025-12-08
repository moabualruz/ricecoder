//! Property-based tests for share list completeness
//! **Feature: ricecoder-sharing, Property 3: Share List Completeness**
//! **Validates: Requirements 1.4, 5.1**

use proptest::prelude::*;
use ricecoder_sessions::{SharePermissions, ShareService};

proptest! {
    /// Property 3: Share List Completeness
    /// *For any* set of active shares, the list command SHALL return all active shares with creation date and expiration.
    /// **Validates: Requirements 1.4, 5.1**
    #[test]
    fn prop_share_list_completeness(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 1..10usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares
        let mut created_ids = Vec::new();
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok(), "Failed to generate share link");
            created_ids.push(share.unwrap().id);
        }

        // List all shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok(), "Failed to list shares");
        let shares = shares.unwrap();

        // Should contain all created shares
        prop_assert_eq!(shares.len(), num_shares, "Share count mismatch");

        // All created IDs should be in the list
        for created_id in &created_ids {
            let found = shares.iter().any(|s| &s.id == created_id);
            prop_assert!(found, "Share {} not found in list", created_id);
        }

        // All shares should have creation date
        for share in &shares {
            prop_assert!(
                share.created_at <= chrono::Utc::now(),
                "Creation timestamp should be in the past"
            );
        }
    }
}

proptest! {
    /// Property 3 variant: List includes expiration info
    #[test]
    fn prop_share_list_includes_expiration(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create share with expiration
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(chrono::Duration::hours(1)),
        );
        prop_assert!(share.is_ok(), "Failed to generate share link");

        // List shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok(), "Failed to list shares");
        let shares = shares.unwrap();

        // Should have at least one share with expiration
        let has_expiration = shares.iter().any(|s| s.expires_at.is_some());
        prop_assert!(has_expiration, "No shares with expiration found");

        // Verify expiration is in the future
        for share in shares.iter().filter(|s| s.expires_at.is_some()) {
            let expires_at = share.expires_at.unwrap();
            prop_assert!(
                expires_at > chrono::Utc::now(),
                "Expiration should be in the future"
            );
        }
    }
}

proptest! {
    /// Property 3 variant: List excludes expired shares
    #[test]
    fn prop_share_list_excludes_expired(
        session_id in "[a-z0-9]{1,20}",
    ) {
        use std::thread;
        use std::time::Duration as StdDuration;

        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create an expiring share
        let expiring = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(chrono::Duration::milliseconds(100)),
        );
        prop_assert!(expiring.is_ok());
        let expiring_id = expiring.unwrap().id;

        // Create a non-expiring share
        let permanent = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(permanent.is_ok());
        let permanent_id = permanent.unwrap().id;

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // List shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();

        // Expired share should not be in list
        let found_expired = shares.iter().any(|s| s.id == expiring_id);
        prop_assert!(!found_expired, "Expired share should not be in list");

        // Permanent share should still be in list
        let found_permanent = shares.iter().any(|s| s.id == permanent_id);
        prop_assert!(found_permanent, "Permanent share should be in list");
    }
}

proptest! {
    /// Property 3 variant: List includes all required metadata
    #[test]
    fn prop_share_list_metadata_complete(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 1..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
        }

        // List shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();

        // Verify all shares have complete metadata
        for share in shares {
            // ID should be present and non-empty
            prop_assert!(!share.id.is_empty(), "Share ID should not be empty");

            // Session ID should be present
            prop_assert!(!share.session_id.is_empty(), "Session ID should not be empty");

            // Created at should be present and in the past
            prop_assert!(
                share.created_at <= chrono::Utc::now(),
                "Creation timestamp should be in the past"
            );

            // Permissions should be present
            prop_assert_eq!(share.permissions.read_only, true, "Read-only should be true");
        }
    }
}

proptest! {
    /// Property 3 variant: List returns empty for no shares
    #[test]
    fn prop_share_list_empty(_dummy in Just(())) {
        let service = ShareService::new();

        // List shares when none exist
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();

        // Should be empty
        prop_assert_eq!(shares.len(), 0, "List should be empty when no shares exist");
    }
}

proptest! {
    /// Property 3 variant: List is consistent across multiple calls
    #[test]
    fn prop_share_list_consistency(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 1..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
        }

        // List shares multiple times
        let mut previous_ids: Option<Vec<String>> = None;

        for _ in 0..3 {
            let shares = service.list_shares();
            prop_assert!(shares.is_ok());
            let shares = shares.unwrap();

            let mut current_ids: Vec<String> = shares.iter().map(|s| s.id.clone()).collect();
            current_ids.sort();

            if let Some(mut prev_ids) = previous_ids.take() {
                prev_ids.sort();
                prop_assert_eq!(
                    &current_ids, &prev_ids,
                    "List should be consistent across multiple calls"
                );
            }

            previous_ids = Some(current_ids);
        }
    }
}
