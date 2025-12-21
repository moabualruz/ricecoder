//! Property-based tests for share context filtering
//! **Feature: ricecoder-sharing, Property 6: Context Filtering**
//! **Validates: Requirements 3.5, 4.3**

use proptest::prelude::*;
use ricecoder_sessions::{Session, SessionContext, SessionMode, SharePermissions, ShareService};

fn arb_session_context() -> impl Strategy<Value = SessionContext> {
    (
        "[a-z0-9]{1,20}",
        "[a-z0-9]{1,20}",
        prop_oneof![
            Just(SessionMode::Chat),
            Just(SessionMode::Code),
            Just(SessionMode::Vibe)
        ],
    )
        .prop_map(|(provider, model, mode)| SessionContext::new(provider, model, mode))
}

fn arb_session_with_context() -> impl Strategy<Value = Session> {
    (
        "[a-z0-9]{1,20}",
        arb_session_context(),
        1..5usize,
        1..3usize,
    )
        .prop_map(|(name, context, num_files, num_custom)| {
            let mut session = Session::new(name, context);

            // Add context files
            for i in 0..num_files {
                session.context.files.push(format!("file{}.rs", i));
            }

            // Add custom context
            for i in 0..num_custom {
                session.context.custom.insert(
                    format!("key{}", i),
                    serde_json::json!(format!("value{}", i)),
                );
            }

            session
        })
}

proptest! {
    /// Property 6: Context Filtering
    /// *For any* shared session with include_context=false, the returned session SHALL have empty context files and custom data.
    /// **Validates: Requirements 3.5, 4.3**
    #[test]
    fn prop_context_filtering(
        session in arb_session_with_context(),
        include_context in any::<bool>(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context,
        };

        let original_files_len = session.context.files.len();
        let original_custom_len = session.context.custom.len();

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify context filtering
        if include_context {
            prop_assert_eq!(
                shared_view.context.files.len(),
                original_files_len,
                "Context files should be included when include_context=true"
            );
            prop_assert_eq!(
                shared_view.context.custom.len(),
                original_custom_len,
                "Custom context should be included when include_context=true"
            );
        } else {
            prop_assert_eq!(
                shared_view.context.files.len(),
                0,
                "Context files should be empty when include_context=false"
            );
            prop_assert_eq!(
                shared_view.context.custom.len(),
                0,
                "Custom context should be empty when include_context=false"
            );
        }

        // Provider and model should always be included
        prop_assert_eq!(
            &shared_view.context.provider, &session.context.provider,
            "Provider should always be included"
        );
        prop_assert_eq!(
            &shared_view.context.model, &session.context.model,
            "Model should always be included"
        );
    }
}

proptest! {
    /// Property 6 variant: Context content is preserved when included
    #[test]
    fn prop_context_content_preserved(
        session in arb_session_with_context(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify context content is preserved
        prop_assert_eq!(
            shared_view.context.files, session.context.files,
            "Context files should be preserved"
        );
        prop_assert_eq!(
            shared_view.context.custom, session.context.custom,
            "Custom context should be preserved"
        );
    }
}

proptest! {
    /// Property 6 variant: Context is completely empty when excluded
    #[test]
    fn prop_context_completely_empty_when_excluded(
        session in arb_session_with_context(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: false,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify context is completely empty
        prop_assert_eq!(
            shared_view.context.files.len(),
            0,
            "Context files should be completely empty"
        );
        prop_assert_eq!(
            shared_view.context.custom.len(),
            0,
            "Custom context should be completely empty"
        );
        prop_assert!(
            shared_view.context.files.is_empty(),
            "Context files should be empty"
        );
        prop_assert!(
            shared_view.context.custom.is_empty(),
            "Custom context should be empty"
        );
    }
}

proptest! {
    /// Property 6 variant: Context filtering doesn't affect other session data
    #[test]
    fn prop_context_filtering_isolation(
        session in arb_session_with_context(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: false,
        };

        let original_name = session.name.clone();
        let original_provider = session.context.provider.clone();
        let original_model = session.context.model.clone();

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify other data is preserved
        prop_assert_eq!(
            &shared_view.name, &original_name,
            "Session name should be preserved"
        );
        prop_assert_eq!(
            &shared_view.context.provider, &original_provider,
            "Provider should be preserved"
        );
        prop_assert_eq!(
            &shared_view.context.model, &original_model,
            "Model should be preserved"
        );
    }
}

proptest! {
    /// Property 6 variant: Context filtering works with empty context
    #[test]
    fn prop_context_filtering_empty_context(
        session_name in "[a-z0-9]{1,20}",
        context in arb_session_context(),
    ) {
        let service = ShareService::new();
        let session = Session::new(session_name, context);

        // Session has no context files or custom data
        prop_assert_eq!(session.context.files.len(), 0);
        prop_assert_eq!(session.context.custom.len(), 0);

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify context is still empty
        prop_assert_eq!(
            shared_view.context.files.len(),
            0,
            "Context files should remain empty"
        );
        prop_assert_eq!(
            shared_view.context.custom.len(),
            0,
            "Custom context should remain empty"
        );
    }
}

proptest! {
    /// Property 6 variant: Context filtering is consistent across multiple calls
    #[test]
    fn prop_context_filtering_consistency(
        session in arb_session_with_context(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: false,
        };

        // Create shared view multiple times
        for _ in 0..3 {
            let shared_view = service.create_shared_session_view(&session, &permissions);

            // Verify context is always empty
            prop_assert_eq!(
                shared_view.context.files.len(),
                0,
                "Context files should be consistently empty"
            );
            prop_assert_eq!(
                shared_view.context.custom.len(),
                0,
                "Custom context should be consistently empty"
            );
        }
    }
}

proptest! {
    /// Property 6 variant: Context filtering with history filtering
    #[test]
    fn prop_context_filtering_with_history_filtering(
        session in arb_session_with_context(),
        include_history in any::<bool>(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history,
            include_context: false,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify context is always empty regardless of history filtering
        prop_assert_eq!(
            shared_view.context.files.len(),
            0,
            "Context should be empty when include_context=false"
        );
        prop_assert_eq!(
            shared_view.context.custom.len(),
            0,
            "Custom context should be empty when include_context=false"
        );

        // Verify history filtering is independent
        if include_history {
            prop_assert!(
                !shared_view.history.is_empty() || session.history.is_empty(),
                "History should be included when include_history=true"
            );
        } else {
            prop_assert_eq!(
                shared_view.history.len(),
                0,
                "History should be empty when include_history=false"
            );
        }
    }
}

proptest! {
    /// Property 6 variant: Provider and model are always preserved
    #[test]
    fn prop_provider_model_always_preserved(
        session in arb_session_with_context(),
    ) {
        let service = ShareService::new();

        // Test with all permission combinations
        for include_context in &[true, false] {
            let permissions = SharePermissions {
                read_only: true,
                include_history: true,
                include_context: *include_context,
            };

            let shared_view = service.create_shared_session_view(&session, &permissions);

            // Provider and model should always be preserved
            prop_assert_eq!(
                &shared_view.context.provider, &session.context.provider,
                "Provider should always be preserved"
            );
            prop_assert_eq!(
                &shared_view.context.model, &session.context.model,
                "Model should always be preserved"
            );
        }
    }
}
