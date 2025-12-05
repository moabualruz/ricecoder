//! Property-based tests for share privacy settings enforcement
//! **Feature: ricecoder-sessions, Property 9: Share Privacy Settings Enforcement**
//! **Validates: Requirements 3.4**

use proptest::prelude::*;
use ricecoder_sessions::{
    Message, MessageRole, Session, SessionContext, SessionMode, SharePermissions, ShareService,
};

fn arb_session_context() -> impl Strategy<Value = SessionContext> {
    (
        ".*",
        ".*",
        prop_oneof![
            Just(SessionMode::Chat),
            Just(SessionMode::Code),
            Just(SessionMode::Vibe)
        ],
    )
        .prop_map(|(provider, model, mode)| SessionContext::new(provider, model, mode))
}

fn arb_session() -> impl Strategy<Value = Session> {
    (".*", arb_session_context()).prop_map(|(name, context)| Session::new(name, context))
}

proptest! {
    /// Property 9: Share Privacy Settings Enforcement
    /// *For any* share with privacy settings, accessing the share SHALL respect the configured
    /// permissions (read-only, include_history, include_context).
    /// **Validates: Requirements 3.4**
    #[test]
    fn prop_share_privacy_settings_enforcement(
        mut session in arb_session(),
        include_history in any::<bool>(),
        include_context in any::<bool>(),
    ) {
        let service = ShareService::new();

        // Add messages and context
        session.history.push(Message::new(MessageRole::User, "Test message".to_string()));
        session.context.files.push("test.rs".to_string());
        session.context.custom.insert("key".to_string(), serde_json::json!("value"));

        let permissions = SharePermissions {
            read_only: true,
            include_history,
            include_context,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify history is included/excluded based on permissions
        if include_history {
            prop_assert_eq!(shared_view.history.len(), session.history.len());
        } else {
            prop_assert_eq!(shared_view.history.len(), 0);
        }

        // Verify context is included/excluded based on permissions
        if include_context {
            prop_assert_eq!(shared_view.context.files.len(), session.context.files.len());
            prop_assert_eq!(shared_view.context.custom.len(), session.context.custom.len());
        } else {
            prop_assert_eq!(shared_view.context.files.len(), 0);
            prop_assert_eq!(shared_view.context.custom.len(), 0);
        }

        // Provider and model should always be included
        prop_assert_eq!(shared_view.context.provider, session.context.provider);
        prop_assert_eq!(shared_view.context.model, session.context.model);
    }

    /// Property 9 variant: All permission combinations work correctly
    #[test]
    fn prop_share_privacy_all_combinations(
        mut session in arb_session(),
    ) {
        let service = ShareService::new();

        // Add messages and context
        session.history.push(Message::new(MessageRole::User, "Test".to_string()));
        session.context.files.push("test.rs".to_string());

        // Test all 8 combinations of permissions
        for read_only in &[true, false] {
            for include_history in &[true, false] {
                for include_context in &[true, false] {
                    let permissions = SharePermissions {
                        read_only: *read_only,
                        include_history: *include_history,
                        include_context: *include_context,
                    };

                    let shared_view = service.create_shared_session_view(&session, &permissions);

                    // Verify history filtering
                    if *include_history {
                        prop_assert_eq!(shared_view.history.len(), 1);
                    } else {
                        prop_assert_eq!(shared_view.history.len(), 0);
                    }

                    // Verify context filtering
                    if *include_context {
                        prop_assert_eq!(shared_view.context.files.len(), 1);
                    } else {
                        prop_assert_eq!(shared_view.context.files.len(), 0);
                    }
                }
            }
        }
    }
}
