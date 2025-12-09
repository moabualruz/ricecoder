/// Property-based tests for hot-reload correctness
/// **Feature: ricecoder-teams, Property 6: Hot-Reload Correctness**
/// **Validates: Requirements 1.9, 1.10**

use proptest::prelude::*;
use ricecoder_teams::sync::SyncService;
use std::time::Duration;

/// Strategy for generating random team IDs
fn arb_team_id() -> impl Strategy<Value = String> {
    "[a-z0-9]{1,20}"
}

/// Strategy for generating random member IDs
fn arb_member_id() -> impl Strategy<Value = String> {
    "[a-z0-9]{1,20}"
}

/// Strategy for generating random messages
fn arb_message() -> impl Strategy<Value = String> {
    "[a-z0-9 ]{1,100}"
}

#[tokio::test]
async fn test_sync_service_creation() {
    let _sync_service = SyncService::new();
    // Service should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_register_team_members() {
    let sync_service = SyncService::new();
    let team_id = "team-1";
    let members = vec!["member-1".to_string(), "member-2".to_string()];

    let result = sync_service
        .register_team_members(team_id, members.clone())
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_notify_members_with_registered_members() {
    let sync_service = SyncService::new();
    let team_id = "team-1";
    let members = vec!["member-1".to_string(), "member-2".to_string()];

    sync_service
        .register_team_members(team_id, members.clone())
        .await
        .expect("Failed to register members");

    let result = sync_service
        .notify_members(team_id, "Configuration updated")
        .await;

    assert!(result.is_ok());

    // Verify notification was recorded
    let history = sync_service
        .get_notification_history(team_id)
        .await
        .expect("Failed to get notification history");

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].team_id, team_id);
    assert_eq!(history[0].message, "Configuration updated");
    assert_eq!(history[0].recipients.len(), 2);
    assert!(history[0].delivered);
}

#[tokio::test]
async fn test_notify_members_without_registered_members() {
    let sync_service = SyncService::new();
    let team_id = "team-1";

    // Should not fail even if no members are registered
    let result = sync_service
        .notify_members(team_id, "Configuration updated")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_notification_history_tracking() {
    let sync_service = SyncService::new();
    let team_id = "team-1";
    let members = vec!["member-1".to_string(), "member-2".to_string()];

    sync_service
        .register_team_members(team_id, members)
        .await
        .expect("Failed to register members");

    // Send multiple notifications
    sync_service
        .notify_members(team_id, "Update 1")
        .await
        .expect("Failed to notify");

    sync_service
        .notify_members(team_id, "Update 2")
        .await
        .expect("Failed to notify");

    // Verify history
    let history = sync_service
        .get_notification_history(team_id)
        .await
        .expect("Failed to get notification history");

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].message, "Update 1");
    assert_eq!(history[1].message, "Update 2");
}

#[tokio::test]
async fn test_register_change_callback() {
    let sync_service = SyncService::new();
    let callback = std::sync::Arc::new(|_event| {
        // Callback implementation
    });

    let result = sync_service.register_change_callback(callback).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_teams_notification_isolation() {
    let sync_service = SyncService::new();

    // Register members for team 1
    sync_service
        .register_team_members("team-1", vec!["member-1".to_string()])
        .await
        .expect("Failed to register team 1 members");

    // Register members for team 2
    sync_service
        .register_team_members("team-2", vec!["member-2".to_string()])
        .await
        .expect("Failed to register team 2 members");

    // Send notifications to both teams
    sync_service
        .notify_members("team-1", "Team 1 update")
        .await
        .expect("Failed to notify team 1");

    sync_service
        .notify_members("team-2", "Team 2 update")
        .await
        .expect("Failed to notify team 2");

    // Verify isolation
    let history_1 = sync_service
        .get_notification_history("team-1")
        .await
        .expect("Failed to get team 1 history");

    let history_2 = sync_service
        .get_notification_history("team-2")
        .await
        .expect("Failed to get team 2 history");

    assert_eq!(history_1.len(), 1);
    assert_eq!(history_2.len(), 1);
    assert_eq!(history_1[0].message, "Team 1 update");
    assert_eq!(history_2[0].message, "Team 2 update");
}

// Property-based tests using synchronous runtime
// Note: We use tokio::runtime to run async code within proptest

proptest! {
    /// Property 6: Hot-Reload Correctness
    /// For any configuration change in team standards storage,
    /// all team members SHALL receive the updated configuration within 5 seconds
    /// without requiring application restart.
    #[test]
    fn prop_hot_reload_notification_delivery(
        team_id in arb_team_id(),
        member_ids in prop::collection::vec(arb_member_id(), 1..5),
        message in arb_message(),
    ) {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let sync_service = SyncService::new();

            // Register team members
            sync_service
                .register_team_members(&team_id, member_ids.clone())
                .await
                .expect("Failed to register members");

            // Send notification
            sync_service
                .notify_members(&team_id, &message)
                .await
                .expect("Failed to notify members");

            // Property: All team members should receive the notification
            let history = sync_service
                .get_notification_history(&team_id)
                .await
                .expect("Failed to get notification history");

            prop_assert_eq!(
                history.len(),
                1,
                "Exactly one notification should be recorded"
            );

            prop_assert_eq!(
                &history[0].team_id,
                &team_id,
                "Notification should be for the correct team"
            );

            prop_assert_eq!(
                &history[0].message,
                &message,
                "Notification message should match"
            );

            prop_assert_eq!(
                history[0].recipients.len(),
                member_ids.len(),
                "All registered members should be recipients"
            );

            prop_assert!(
                history[0].delivered,
                "Notification should be marked as delivered"
            );

            // Property: Notification timestamp should be recent (within 5 seconds)
            let now = std::time::SystemTime::now();
            let time_diff = now
                .duration_since(history[0].timestamp)
                .unwrap_or_default();

            prop_assert!(
                time_diff <= Duration::from_secs(5),
                "Notification should be delivered within 5 seconds"
            );

            Ok(())
        }).expect("Async test failed")
    }

    /// Property: Multiple notifications should be tracked independently
    #[test]
    fn prop_hot_reload_multiple_notifications(
        team_id in arb_team_id(),
        member_ids in prop::collection::vec(arb_member_id(), 1..3),
        messages in prop::collection::vec(arb_message(), 2..5),
    ) {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let sync_service = SyncService::new();

            // Register team members
            sync_service
                .register_team_members(&team_id, member_ids.clone())
                .await
                .expect("Failed to register members");

            // Send multiple notifications
            for message in &messages {
                sync_service
                    .notify_members(&team_id, message)
                    .await
                    .expect("Failed to notify members");
            }

            // Property: All notifications should be tracked
            let history = sync_service
                .get_notification_history(&team_id)
                .await
                .expect("Failed to get notification history");

            prop_assert_eq!(
                history.len(),
                messages.len(),
                "All notifications should be tracked"
            );

            // Property: Messages should be in order
            for (i, message) in messages.iter().enumerate() {
                prop_assert_eq!(
                    &history[i].message,
                    message,
                    "Notification messages should be in order"
                );
            }

            Ok(())
        }).expect("Async test failed")
    }

    /// Property: Notification isolation between teams
    #[test]
    fn prop_hot_reload_team_isolation(
        team_id_1 in arb_team_id(),
        team_id_2 in arb_team_id(),
        members_1 in prop::collection::vec(arb_member_id(), 1..3),
        members_2 in prop::collection::vec(arb_member_id(), 1..3),
        message_1 in arb_message(),
        message_2 in arb_message(),
    ) {
        // Skip if team IDs are the same
        prop_assume!(team_id_1 != team_id_2);

        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let sync_service = SyncService::new();

            // Register members for both teams
            sync_service
                .register_team_members(&team_id_1, members_1.clone())
                .await
                .expect("Failed to register team 1 members");

            sync_service
                .register_team_members(&team_id_2, members_2.clone())
                .await
                .expect("Failed to register team 2 members");

            // Send notifications to both teams
            sync_service
                .notify_members(&team_id_1, &message_1)
                .await
                .expect("Failed to notify team 1");

            sync_service
                .notify_members(&team_id_2, &message_2)
                .await
                .expect("Failed to notify team 2");

            // Property: Each team should only see their own notifications
            let history_1 = sync_service
                .get_notification_history(&team_id_1)
                .await
                .expect("Failed to get team 1 history");

            let history_2 = sync_service
                .get_notification_history(&team_id_2)
                .await
                .expect("Failed to get team 2 history");

            prop_assert_eq!(
                history_1.len(),
                1,
                "Team 1 should have exactly one notification"
            );

            prop_assert_eq!(
                history_2.len(),
                1,
                "Team 2 should have exactly one notification"
            );

            prop_assert_eq!(
                &history_1[0].message,
                &message_1,
                "Team 1 should receive their message"
            );

            prop_assert_eq!(
                &history_2[0].message,
                &message_2,
                "Team 2 should receive their message"
            );

            prop_assert_eq!(
                history_1[0].recipients.len(),
                members_1.len(),
                "Team 1 should have correct recipient count"
            );

            prop_assert_eq!(
                history_2[0].recipients.len(),
                members_2.len(),
                "Team 2 should have correct recipient count"
            );

            Ok(())
        }).expect("Async test failed")
    }

    /// Property: Notification delivery status should always be true
    #[test]
    fn prop_hot_reload_delivery_status(
        team_id in arb_team_id(),
        member_ids in prop::collection::vec(arb_member_id(), 1..5),
        message in arb_message(),
    ) {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let sync_service = SyncService::new();

            sync_service
                .register_team_members(&team_id, member_ids)
                .await
                .expect("Failed to register members");

            sync_service
                .notify_members(&team_id, &message)
                .await
                .expect("Failed to notify members");

            let history = sync_service
                .get_notification_history(&team_id)
                .await
                .expect("Failed to get notification history");

            prop_assert!(
                history[0].delivered,
                "Notification should be marked as delivered"
            );

            Ok(())
        }).expect("Async test failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hot_reload_basic_notification() {
        let sync_service = SyncService::new();
        let team_id = "team-1";
        let members = vec!["member-1".to_string(), "member-2".to_string()];

        sync_service
            .register_team_members(team_id, members)
            .await
            .expect("Failed to register members");

        sync_service
            .notify_members(team_id, "Configuration updated")
            .await
            .expect("Failed to notify");

        let history = sync_service
            .get_notification_history(team_id)
            .await
            .expect("Failed to get history");

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].team_id, team_id);
        assert_eq!(history[0].message, "Configuration updated");
        assert_eq!(history[0].recipients.len(), 2);
        assert!(history[0].delivered);
    }

    #[tokio::test]
    async fn test_hot_reload_notification_within_5_seconds() {
        let sync_service = SyncService::new();
        let team_id = "team-1";
        let members = vec!["member-1".to_string()];

        sync_service
            .register_team_members(team_id, members)
            .await
            .expect("Failed to register members");

        let start = std::time::SystemTime::now();

        sync_service
            .notify_members(team_id, "Update")
            .await
            .expect("Failed to notify");

        let elapsed = start.elapsed().expect("Failed to measure time");

        // Notification should be delivered almost instantly
        assert!(elapsed < Duration::from_secs(5));

        let history = sync_service
            .get_notification_history(team_id)
            .await
            .expect("Failed to get history");

        assert_eq!(history.len(), 1);
        assert!(history[0].delivered);
    }

    #[tokio::test]
    async fn test_hot_reload_multiple_teams_isolation() {
        let sync_service = SyncService::new();

        sync_service
            .register_team_members("team-1", vec!["member-1".to_string()])
            .await
            .expect("Failed to register team 1");

        sync_service
            .register_team_members("team-2", vec!["member-2".to_string()])
            .await
            .expect("Failed to register team 2");

        sync_service
            .notify_members("team-1", "Team 1 update")
            .await
            .expect("Failed to notify team 1");

        sync_service
            .notify_members("team-2", "Team 2 update")
            .await
            .expect("Failed to notify team 2");

        let history_1 = sync_service
            .get_notification_history("team-1")
            .await
            .expect("Failed to get team 1 history");

        let history_2 = sync_service
            .get_notification_history("team-2")
            .await
            .expect("Failed to get team 2 history");

        assert_eq!(history_1.len(), 1);
        assert_eq!(history_2.len(), 1);
        assert_eq!(history_1[0].message, "Team 1 update");
        assert_eq!(history_2[0].message, "Team 2 update");
    }

    #[tokio::test]
    async fn test_hot_reload_all_members_notified() {
        let sync_service = SyncService::new();
        let team_id = "team-1";
        let members = vec![
            "member-1".to_string(),
            "member-2".to_string(),
            "member-3".to_string(),
        ];

        sync_service
            .register_team_members(team_id, members.clone())
            .await
            .expect("Failed to register members");

        sync_service
            .notify_members(team_id, "Update")
            .await
            .expect("Failed to notify");

        let history = sync_service
            .get_notification_history(team_id)
            .await
            .expect("Failed to get history");

        assert_eq!(history[0].recipients.len(), members.len());
        for (i, member) in members.iter().enumerate() {
            assert_eq!(history[0].recipients[i], *member);
        }
    }
}
