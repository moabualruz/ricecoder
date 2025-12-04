//! Property-based tests for share link uniqueness
//! **Feature: ricecoder-sessions, Property 7: Share Link Uniqueness**
//! **Validates: Requirements 3.1**

use proptest::prelude::*;
use ricecoder_sessions::{ShareService, SharePermissions};

proptest! {
    /// Property 7: Share Link Uniqueness
    /// *For any* two share requests, the generated share links SHALL have unique IDs.
    /// **Validates: Requirements 3.1**
    #[test]
    fn prop_share_link_uniqueness(
        session_id_1 in ".*",
        session_id_2 in ".*",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate two share links
        let share_1 = service.generate_share_link(&session_id_1, permissions.clone(), None);
        let share_2 = service.generate_share_link(&session_id_2, permissions.clone(), None);

        // Both should succeed
        prop_assert!(share_1.is_ok());
        prop_assert!(share_2.is_ok());

        let share_1 = share_1.unwrap();
        let share_2 = share_2.unwrap();

        // Share IDs should be unique
        prop_assert_ne!(share_1.id, share_2.id);
    }

    /// Property 7 variant: Multiple shares for the same session should have unique IDs
    #[test]
    fn prop_share_link_uniqueness_same_session(
        session_id in ".*",
        num_shares in 2..10usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let mut share_ids = Vec::new();

        // Generate multiple shares for the same session
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
            share_ids.push(share.unwrap().id);
        }

        // All share IDs should be unique
        let mut sorted_ids = share_ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        prop_assert_eq!(sorted_ids.len(), share_ids.len());
    }
}
