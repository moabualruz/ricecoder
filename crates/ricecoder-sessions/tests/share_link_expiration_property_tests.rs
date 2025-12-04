//! Property-based tests for share link expiration
//! **Feature: ricecoder-sessions, Property 10: Share Link Expiration**
//! **Validates: Requirements 3.5**

use proptest::prelude::*;
use ricecoder_sessions::{ShareService, SharePermissions};
use chrono::Duration;
use std::thread;
use std::time::Duration as StdDuration;

proptest! {
    /// Property 10: Share Link Expiration
    /// *For any* share link with an expiration timestamp, accessing the link after expiration
    /// SHALL return an error indicating the link has expired.
    /// **Validates: Requirements 3.5**
    #[test]
    fn prop_share_link_expiration(
        session_id in ".*",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link that expires in 1 second
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Should be accessible immediately
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Should be expired now
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_err());
        if let Err(e) = retrieved {
            prop_assert!(e.to_string().contains("expired"));
        }
    }

    /// Property 10 variant: Non-expiring shares remain accessible
    #[test]
    fn prop_share_link_no_expiration(
        session_id in ".*",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link with no expiration
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Should be accessible
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());

        // Wait a bit
        thread::sleep(StdDuration::from_millis(100));

        // Should still be accessible
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());
    }

    /// Property 10 variant: Cleanup removes expired shares
    #[test]
    fn prop_share_cleanup_expired(
        session_id in ".*",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate multiple shares with different expiration times
        let share_1 = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(50)),
        );
        prop_assert!(share_1.is_ok());

        let share_2 = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share_2.is_ok());

        // Wait for first share to expire
        thread::sleep(StdDuration::from_millis(100));

        // Cleanup expired shares
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());
        let cleaned = cleaned.unwrap();

        // Should have cleaned up at least one share
        prop_assert!(cleaned >= 1);

        // Non-expiring share should still be accessible
        let retrieved = service.get_share(&share_2.unwrap().id);
        prop_assert!(retrieved.is_ok());
    }
}
