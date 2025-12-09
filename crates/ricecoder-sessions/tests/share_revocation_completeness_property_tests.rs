//! Property-based tests for share revocation completeness
//! **Feature: ricecoder-sharing, Property 4: Revocation Completeness**
//! **Validates: Requirements 1.5, 5.3**

use proptest::prelude::*;
use ricecoder_sessions::{SharePermissions, ShareService};

proptest! {
    /// Property 4: Revocation Completeness
    /// *For any* revoked share, subsequent access attempts SHALL return an error (ShareRevoked).
    /// **Validates: Requirements 1.5, 5.3**
    #[test]
    fn prop_revocation_completeness(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok(), "Failed to generate share link");
        let share = share.unwrap();

        // Should be accessible before revocation
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok(), "Share should be accessible before revocation");

        // Revoke the share
        let revoked = service.revoke_share(&share.id);
        prop_assert!(revoked.is_ok(), "Failed to revoke share");

        // Should not be accessible after revocation
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_err(), "Share should not be accessible after revocation");
    }
}

proptest! {
    /// Property 4 variant: Revoked share no longer appears in list
    #[test]
    fn prop_revocation_removes_from_list(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 2..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares
        let mut share_ids = Vec::new();
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok(), "Failed to generate share link");
            share_ids.push(share.unwrap().id);
        }

        // Revoke the first share
        let revoked = service.revoke_share(&share_ids[0]);
        prop_assert!(revoked.is_ok(), "Failed to revoke share");

        // List shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok(), "Failed to list shares");
        let shares = shares.unwrap();

        // Revoked share should not be in list
        let found = shares.iter().any(|s| s.id == share_ids[0]);
        prop_assert!(!found, "Revoked share should not be in list");

        // Other shares should still be in list
        for i in 1..share_ids.len() {
            let found = shares.iter().any(|s| s.id == share_ids[i]);
            prop_assert!(found, "Non-revoked share should be in list");
        }
    }
}

proptest! {
    /// Property 4 variant: Multiple revocations work correctly
    #[test]
    fn prop_multiple_revocations(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 2..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares
        let mut share_ids = Vec::new();
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
            share_ids.push(share.unwrap().id);
        }

        // Revoke all shares
        for share_id in &share_ids {
            let revoked = service.revoke_share(share_id);
            prop_assert!(revoked.is_ok(), "Failed to revoke share");
        }

        // All shares should be inaccessible
        for share_id in &share_ids {
            let retrieved = service.get_share(share_id);
            prop_assert!(retrieved.is_err(), "Revoked share should not be accessible");
        }

        // List should be empty
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();
        prop_assert_eq!(shares.len(), 0, "List should be empty after revoking all shares");
    }
}

proptest! {
    /// Property 4 variant: Revoking non-existent share fails
    #[test]
    fn prop_revoke_nonexistent_fails(
        fake_share_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
    ) {
        let service = ShareService::new();

        // Try to revoke a non-existent share
        let revoked = service.revoke_share(&fake_share_id);
        prop_assert!(revoked.is_err(), "Revoking non-existent share should fail");
    }
}

proptest! {
    /// Property 4 variant: Revoking same share twice fails
    #[test]
    fn prop_revoke_twice_fails(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share_id = share.unwrap().id;

        // Revoke the share
        let revoked = service.revoke_share(&share_id);
        prop_assert!(revoked.is_ok(), "First revocation should succeed");

        // Try to revoke again
        let revoked_again = service.revoke_share(&share_id);
        prop_assert!(revoked_again.is_err(), "Second revocation should fail");
    }
}

proptest! {
    /// Property 4 variant: Revocation is immediate
    #[test]
    fn prop_revocation_immediate(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share_id = share.unwrap().id;

        // Verify it's accessible
        let retrieved = service.get_share(&share_id);
        prop_assert!(retrieved.is_ok());

        // Revoke the share
        let revoked = service.revoke_share(&share_id);
        prop_assert!(revoked.is_ok());

        // Immediately try to access - should fail
        let retrieved = service.get_share(&share_id);
        prop_assert!(retrieved.is_err(), "Share should be inaccessible immediately after revocation");
    }
}

proptest! {
    /// Property 4 variant: Revocation doesn't affect other shares
    #[test]
    fn prop_revocation_isolation(
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

        prop_assert!(share_1.is_ok());
        prop_assert!(share_2.is_ok());

        let share_1_id = share_1.unwrap().id;
        let share_2_id = share_2.unwrap().id;

        // Revoke share 1
        let revoked = service.revoke_share(&share_1_id);
        prop_assert!(revoked.is_ok());

        // Share 1 should be inaccessible
        let retrieved = service.get_share(&share_1_id);
        prop_assert!(retrieved.is_err());

        // Share 2 should still be accessible
        let retrieved = service.get_share(&share_2_id);
        prop_assert!(retrieved.is_ok(), "Revoking one share should not affect others");
    }
}
