//! Property-based tests for share session deletion invalidation
//! **Feature: ricecoder-sharing, Property 10: Session Deletion Invalidation**
//! **Validates: Requirements 5.5**

use proptest::prelude::*;
use ricecoder_sessions::{SharePermissions, ShareService};

proptest! {
    /// Property 10: Session Deletion Invalidation
    /// *For any* session that is deleted, all associated shares SHALL become invalid
    /// (return error on access).
    /// **Validates: Requirements 5.5**
    #[test]
    fn prop_session_deletion_invalidation(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 1..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares for a session
        let mut share_ids = Vec::new();
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok(), "Failed to generate share link");
            share_ids.push(share.unwrap().id);
        }

        // All shares should be accessible
        for share_id in &share_ids {
            let retrieved = service.get_share(share_id);
            prop_assert!(retrieved.is_ok(), "Share should be accessible before deletion");
        }

        // Invalidate all shares for the session (simulating session deletion)
        let invalidated = service.invalidate_session_shares(&session_id);
        prop_assert!(invalidated.is_ok(), "Failed to invalidate session shares");
        let invalidated = invalidated.unwrap();
        prop_assert_eq!(invalidated, num_shares, "Should have invalidated all shares");

        // All shares should now be inaccessible
        for share_id in &share_ids {
            let retrieved = service.get_share(share_id);
            prop_assert!(retrieved.is_err(), "Share should be inaccessible after deletion");
        }
    }
}

proptest! {
    /// Property 10 variant: Invalidation only affects specified session
    #[test]
    fn prop_session_deletion_isolation(
        session_id_1 in "[a-z0-9]{1,20}",
        session_id_2 in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shares for two different sessions
        let share_1 = service.generate_share_link(&session_id_1, permissions.clone(), None);
        let share_2 = service.generate_share_link(&session_id_2, permissions.clone(), None);

        prop_assert!(share_1.is_ok(), "Failed to generate share 1");
        prop_assert!(share_2.is_ok(), "Failed to generate share 2");

        let share_1_id = share_1.unwrap().id;
        let share_2_id = share_2.unwrap().id;

        // Invalidate shares for session 1
        let invalidated = service.invalidate_session_shares(&session_id_1);
        prop_assert!(invalidated.is_ok(), "Failed to invalidate session 1 shares");

        // Share 1 should be inaccessible
        let retrieved = service.get_share(&share_1_id);
        prop_assert!(retrieved.is_err(), "Share 1 should be inaccessible");

        // Share 2 should still be accessible
        let retrieved = service.get_share(&share_2_id);
        prop_assert!(retrieved.is_ok(), "Share 2 should still be accessible");
    }
}

proptest! {
    /// Property 10 variant: Invalidated shares don't appear in list
    #[test]
    fn prop_invalidated_shares_not_in_list(
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

        // Verify all shares are in list
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();
        prop_assert_eq!(shares.len(), num_shares, "All shares should be in list");

        // Invalidate all shares for the session
        let invalidated = service.invalidate_session_shares(&session_id);
        prop_assert!(invalidated.is_ok());

        // List should be empty
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();
        prop_assert_eq!(shares.len(), 0, "List should be empty after invalidation");
    }
}

proptest! {
    /// Property 10 variant: Invalidation count is accurate
    #[test]
    fn prop_invalidation_count_accurate(
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

        // Invalidate and check count
        let invalidated = service.invalidate_session_shares(&session_id);
        prop_assert!(invalidated.is_ok());
        let invalidated = invalidated.unwrap();

        // Count should match number of shares created
        prop_assert_eq!(invalidated, num_shares, "Invalidation count should match share count");
    }
}

proptest! {
    /// Property 10 variant: Invalidating non-existent session returns 0
    #[test]
    fn prop_invalidate_nonexistent_session(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();

        // Try to invalidate a session that doesn't have any shares
        let invalidated = service.invalidate_session_shares(&session_id);
        prop_assert!(invalidated.is_ok(), "Invalidation should succeed");
        let invalidated = invalidated.unwrap();

        // Should return 0 since no shares were invalidated
        prop_assert_eq!(invalidated, 0, "Should return 0 for non-existent session");
    }
}

proptest! {
    /// Property 10 variant: Multiple invalidations work correctly
    #[test]
    fn prop_multiple_invalidations(
        session_id_1 in "[a-z0-9]{1,20}",
        session_id_2 in "[a-z0-9]{1,20}",
        num_shares_1 in 1..3usize,
        num_shares_2 in 1..3usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shares for session 1
        for _ in 0..num_shares_1 {
            let share = service.generate_share_link(&session_id_1, permissions.clone(), None);
            prop_assert!(share.is_ok());
        }

        // Create shares for session 2
        for _ in 0..num_shares_2 {
            let share = service.generate_share_link(&session_id_2, permissions.clone(), None);
            prop_assert!(share.is_ok());
        }

        // Invalidate session 1
        let invalidated_1 = service.invalidate_session_shares(&session_id_1);
        prop_assert!(invalidated_1.is_ok());
        prop_assert_eq!(invalidated_1.unwrap(), num_shares_1);

        // Invalidate session 2
        let invalidated_2 = service.invalidate_session_shares(&session_id_2);
        prop_assert!(invalidated_2.is_ok());
        prop_assert_eq!(invalidated_2.unwrap(), num_shares_2);

        // List should be empty
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();
        prop_assert_eq!(shares.len(), 0, "List should be empty after all invalidations");
    }
}

proptest! {
    /// Property 10 variant: Invalidation is immediate
    #[test]
    fn prop_invalidation_immediate(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create a share
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share_id = share.unwrap().id;

        // Verify it's accessible
        let retrieved = service.get_share(&share_id);
        prop_assert!(retrieved.is_ok());

        // Invalidate the session
        let invalidated = service.invalidate_session_shares(&session_id);
        prop_assert!(invalidated.is_ok());

        // Immediately try to access - should fail
        let retrieved = service.get_share(&share_id);
        prop_assert!(retrieved.is_err(), "Share should be inaccessible immediately after invalidation");
    }
}
