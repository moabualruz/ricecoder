//! Property-based tests for share expiration enforcement
//! **Feature: ricecoder-sharing, Property 7: Expiration Enforcement**
//! **Validates: Requirements 3.1, 3.2**

use chrono::Duration;
use proptest::prelude::*;
use ricecoder_sessions::{SharePermissions, ShareService};
use std::thread;
use std::time::Duration as StdDuration;

proptest! {
    /// Property 7: Expiration Enforcement
    /// *For any* share with expiration time set, accessing the share after expiration
    /// SHALL return an error (ShareExpired).
    /// **Validates: Requirements 3.1, 3.2**
    #[test]
    fn prop_expiration_enforcement(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link that expires in 100ms
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(share.is_ok(), "Failed to generate share link");
        let share = share.unwrap();

        // Should be accessible immediately
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok(), "Share should be accessible before expiration");

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Should be expired now
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_err(), "Share should be expired after expiration time");
    }
}

proptest! {
    /// Property 7 variant: Non-expiring shares remain accessible
    #[test]
    fn prop_no_expiration_always_accessible(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link with no expiration
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok(), "Failed to generate share link");
        let share = share.unwrap();

        // Should be accessible
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok(), "Share should be accessible");

        // Wait a bit
        thread::sleep(StdDuration::from_millis(100));

        // Should still be accessible
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok(), "Non-expiring share should remain accessible");
    }
}

proptest! {
    /// Property 7 variant: Expiration time is respected
    #[test]
    fn prop_expiration_time_respected(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link that expires in 200ms
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(200)),
        );
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Should be accessible at 50ms
        thread::sleep(StdDuration::from_millis(50));
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok(), "Share should be accessible before expiration");

        // Should be accessible at 100ms
        thread::sleep(StdDuration::from_millis(50));
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok(), "Share should still be accessible");

        // Should be expired at 250ms
        thread::sleep(StdDuration::from_millis(150));
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_err(), "Share should be expired after expiration time");
    }
}

proptest! {
    /// Property 7 variant: Expired shares don't appear in list
    #[test]
    fn prop_expired_shares_not_in_list(
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
        let expiring_id = expiring.unwrap().id;

        // Create a non-expiring share
        let permanent = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(permanent.is_ok());
        let permanent_id = permanent.unwrap().id;

        // Both should be in list initially
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();
        prop_assert_eq!(shares.len(), 2, "Both shares should be in list initially");

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // List shares again
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();

        // Expired share should not be in list
        let found_expired = shares.iter().any(|s| s.id == expiring_id);
        prop_assert!(!found_expired, "Expired share should not be in list");

        // Permanent share should still be in list
        let found_permanent = shares.iter().any(|s| s.id == permanent_id);
        prop_assert!(found_permanent, "Permanent share should still be in list");
    }
}

proptest! {
    /// Property 7 variant: Expiration metadata is correct
    #[test]
    fn prop_expiration_metadata_correct(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link with expiration
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::hours(1)),
        );
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Verify expiration metadata
        prop_assert!(share.expires_at.is_some(), "Expiration should be set");
        let expires_at = share.expires_at.unwrap();
        let now = chrono::Utc::now();

        // Expiration should be in the future
        prop_assert!(
            expires_at > now,
            "Expiration should be in the future"
        );

        // Expiration should be approximately 1 hour from now
        let duration = expires_at - now;
        prop_assert!(
            duration.num_minutes() >= 59 && duration.num_minutes() <= 61,
            "Expiration should be approximately 1 hour from now"
        );
    }
}

proptest! {
    /// Property 7 variant: Cleanup removes expired shares
    #[test]
    fn prop_cleanup_removes_expired(
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
        prop_assert!(cleaned.is_ok(), "Cleanup should succeed");
        let cleaned = cleaned.unwrap();

        // Should have cleaned up at least one share
        prop_assert!(cleaned >= 1, "Should have cleaned up expired shares");

        // Non-expiring share should still be accessible
        let retrieved = service.get_share(&permanent_id);
        prop_assert!(retrieved.is_ok(), "Permanent share should still be accessible");
    }
}

proptest! {
    /// Property 7 variant: Multiple shares with different expiration times
    #[test]
    fn prop_multiple_expiration_times(
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
        let share_50ms_id = share_50ms.unwrap().id;

        let share_150ms = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(150)),
        );
        prop_assert!(share_150ms.is_ok());
        let share_150ms_id = share_150ms.unwrap().id;

        // Wait 100ms
        thread::sleep(StdDuration::from_millis(100));

        // 50ms share should be expired
        let retrieved = service.get_share(&share_50ms_id);
        prop_assert!(retrieved.is_err(), "50ms share should be expired");

        // 150ms share should still be accessible
        let retrieved = service.get_share(&share_150ms_id);
        prop_assert!(retrieved.is_ok(), "150ms share should still be accessible");

        // Wait another 100ms
        thread::sleep(StdDuration::from_millis(100));

        // 150ms share should now be expired
        let retrieved = service.get_share(&share_150ms_id);
        prop_assert!(retrieved.is_err(), "150ms share should now be expired");
    }
}
