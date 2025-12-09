//! Property-based tests for share automatic expiration cleanup
//! **Feature: ricecoder-sharing, Property 11: Automatic Expiration Cleanup**
//! **Validates: Requirements 5.4**

use chrono::Duration;
use proptest::prelude::*;
use ricecoder_sessions::{SharePermissions, ShareService};
use std::thread;
use std::time::Duration as StdDuration;

proptest! {
    /// Property 11: Automatic Expiration Cleanup
    /// *For any* share that has expired, it SHALL be automatically cleaned up
    /// and no longer accessible.
    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_automatic_expiration_cleanup(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate an expiring share
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(share.is_ok(), "Failed to generate share link");
        let share_id = share.unwrap().id;

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok(), "Cleanup should succeed");
        let cleaned = cleaned.unwrap();

        // Should have cleaned up at least one share
        prop_assert!(cleaned >= 1, "Should have cleaned up expired shares");

        // Expired share should no longer be accessible
        let retrieved = service.get_share(&share_id);
        prop_assert!(retrieved.is_err(), "Expired share should not be accessible after cleanup");
    }
}

proptest! {
    /// Property 11 variant: Cleanup preserves non-expired shares
    #[test]
    fn prop_cleanup_preserves_active_shares(
        session_id in "[a-z0-9]{1,20}",
    ) {
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
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(expiring.is_ok());

        // Create a non-expiring share
        let permanent = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(permanent.is_ok());
        let permanent_id = permanent.unwrap().id;

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());

        // Non-expiring share should still be accessible
        let retrieved = service.get_share(&permanent_id);
        prop_assert!(retrieved.is_ok(), "Permanent share should still be accessible");
    }
}

proptest! {
    /// Property 11 variant: Cleanup removes all expired shares
    #[test]
    fn prop_cleanup_removes_all_expired(
        session_id in "[a-z0-9]{1,20}",
        num_expired in 1..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple expiring shares
        for _ in 0..num_expired {
            let share = service.generate_share_link(
                &session_id,
                permissions.clone(),
                Some(Duration::milliseconds(100)),
            );
            prop_assert!(share.is_ok());
        }

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());
        let cleaned = cleaned.unwrap();

        // Should have cleaned up all expired shares
        prop_assert_eq!(cleaned, num_expired, "Should have cleaned up all expired shares");

        // List should be empty
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();
        prop_assert_eq!(shares.len(), 0, "List should be empty after cleanup");
    }
}

proptest! {
    /// Property 11 variant: Cleanup returns accurate count
    #[test]
    fn prop_cleanup_count_accurate(
        session_id in "[a-z0-9]{1,20}",
        num_expired in 1..3usize,
        num_permanent in 1..3usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create expiring shares
        for _ in 0..num_expired {
            let share = service.generate_share_link(
                &session_id,
                permissions.clone(),
                Some(Duration::milliseconds(100)),
            );
            prop_assert!(share.is_ok());
        }

        // Create permanent shares
        for _ in 0..num_permanent {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
        }

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());
        let cleaned = cleaned.unwrap();

        // Should have cleaned up only expired shares
        prop_assert_eq!(cleaned, num_expired, "Should have cleaned up only expired shares");

        // List should contain only permanent shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();
        prop_assert_eq!(shares.len(), num_permanent, "List should contain only permanent shares");
    }
}

proptest! {
    /// Property 11 variant: Cleanup can be called multiple times
    #[test]
    fn prop_cleanup_idempotent(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create an expiring share
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(share.is_ok());

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Run cleanup multiple times
        let cleaned_1 = service.cleanup_expired_shares();
        prop_assert!(cleaned_1.is_ok());
        let cleaned_1 = cleaned_1.unwrap();

        let cleaned_2 = service.cleanup_expired_shares();
        prop_assert!(cleaned_2.is_ok());
        let cleaned_2 = cleaned_2.unwrap();

        // First cleanup should have removed the share
        prop_assert!(cleaned_1 >= 1);

        // Second cleanup should remove nothing
        prop_assert_eq!(cleaned_2, 0, "Second cleanup should remove nothing");
    }
}

proptest! {
    /// Property 11 variant: Cleanup doesn't affect non-expired shares
    #[test]
    fn prop_cleanup_preserves_non_expired(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create a non-expiring share
        let permanent = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(permanent.is_ok());
        let permanent_id = permanent.unwrap().id;

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());
        let cleaned = cleaned.unwrap();

        // Should not have cleaned up anything
        prop_assert_eq!(cleaned, 0, "Should not have cleaned up non-expired shares");

        // Permanent share should still be accessible
        let retrieved = service.get_share(&permanent_id);
        prop_assert!(retrieved.is_ok(), "Permanent share should still be accessible");
    }
}

proptest! {
    /// Property 11 variant: Cleanup with mixed expiration times
    #[test]
    fn prop_cleanup_mixed_expiration(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shares with different expiration times
        let share_50ms = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(50)),
        );
        prop_assert!(share_50ms.is_ok());

        let share_150ms = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(150)),
        );
        prop_assert!(share_150ms.is_ok());
        let share_150ms_id = share_150ms.unwrap().id;

        let share_permanent = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share_permanent.is_ok());
        let share_permanent_id = share_permanent.unwrap().id;

        // Wait 100ms
        thread::sleep(StdDuration::from_millis(100));

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());
        let cleaned = cleaned.unwrap();

        // Should have cleaned up only the 50ms share
        prop_assert_eq!(cleaned, 1, "Should have cleaned up only the 50ms share");

        // 150ms and permanent shares should still be accessible
        let retrieved = service.get_share(&share_150ms_id);
        prop_assert!(retrieved.is_ok(), "150ms share should still be accessible");

        let retrieved = service.get_share(&share_permanent_id);
        prop_assert!(retrieved.is_ok(), "Permanent share should still be accessible");
    }
}
