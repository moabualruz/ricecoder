//! Property-based tests for session list completeness
//!
//! **Feature: ricecoder-sessions, Property 6: Session List Completeness**
//! **Validates: Requirements 1.3**
//!
//! Property: *For any* set of active sessions, the rendered session list SHALL display
//! all active sessions with their current status.

use proptest::prelude::*;
use ricecoder_tui::SessionIntegration;
use ricecoder_sessions::SessionMode;

/// Generate a valid session name
fn session_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]{1,50}".prop_map(|s| s.to_string())
}

/// Generate a valid session context
fn session_context_strategy() -> impl Strategy<Value = ricecoder_sessions::SessionContext> {
    (
        r"[a-z]{3,10}".prop_map(|s| s.to_string()),
        r"[a-z0-9\-]{3,20}".prop_map(|s| s.to_string()),
    )
        .prop_map(|(provider, model)| {
            ricecoder_sessions::SessionContext::new(provider, model, SessionMode::Chat)
        })
}

/// Generate a list of sessions to create
fn sessions_strategy() -> impl Strategy<Value = Vec<(String, ricecoder_sessions::SessionContext)>> {
    prop::collection::vec(
        (session_name_strategy(), session_context_strategy()),
        1..=10,
    )
}

proptest! {
    /// Property test: Session list completeness
    ///
    /// For any set of sessions created in the integration, the widget should display
    /// all of them with their correct status.
    #[test]
    fn prop_session_list_completeness(
        sessions in sessions_strategy()
    ) {
        let mut integration = SessionIntegration::new(20);

        // Create all sessions
        let mut created_ids = Vec::new();
        for (name, context) in sessions.iter() {
            if let Ok(id) = integration.create_session(name.clone(), context.clone()) {
                created_ids.push(id);
            }
        }

        // Verify that all created sessions appear in the widget
        let widget_session_ids: Vec<String> = integration
            .widget()
            .session_ids()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // All created sessions should be in the widget
        for created_id in &created_ids {
            prop_assert!(
                widget_session_ids.contains(created_id),
                "Session {} not found in widget",
                created_id
            );
        }

        // The widget should have the same number of sessions as we created
        prop_assert_eq!(
            widget_session_ids.len(),
            created_ids.len(),
            "Widget session count mismatch"
        );

        // All sessions in the widget should have a status
        for session in integration.widget().session_ids() {
            if let Some(tui_session) = integration.widget().get_session(session) {
                // Status should be set (not uninitialized)
                prop_assert!(
                    tui_session.status != ricecoder_tui::SessionStatus::Error,
                    "Session {} has error status",
                    session
                );
            }
        }
    }

    /// Property test: Session status display
    ///
    /// For any active session, the widget should display it with Active status.
    #[test]
    fn prop_active_session_status_display(
        name in session_name_strategy(),
        context in session_context_strategy()
    ) {
        let mut integration = SessionIntegration::new(10);

        let session_id = integration
            .create_session(name, context)
            .expect("Failed to create session");

        // The active session should be displayed with Active status
        if let Some(tui_session) = integration.widget().get_session(&session_id) {
            prop_assert_eq!(
                tui_session.status,
                ricecoder_tui::SessionStatus::Active,
                "Active session should have Active status"
            );
        } else {
            prop_assert!(false, "Active session not found in widget");
        }
    }

    /// Property test: Session list reflects all sessions
    ///
    /// For any number of sessions created, the widget session count should match
    /// the manager session count.
    #[test]
    fn prop_widget_reflects_all_sessions(
        sessions in sessions_strategy()
    ) {
        let mut integration = SessionIntegration::new(20);

        // Create all sessions
        for (name, context) in sessions.iter() {
            let _ = integration.create_session(name.clone(), context.clone());
        }

        // Widget session count should match manager session count
        prop_assert_eq!(
            integration.widget().session_count(),
            integration.session_count(),
            "Widget and manager session counts don't match"
        );
    }

    /// Property test: Session names are preserved
    ///
    /// For any session created with a specific name, the widget should display
    /// that exact name.
    #[test]
    fn prop_session_names_preserved(
        sessions in sessions_strategy()
    ) {
        let mut integration = SessionIntegration::new(20);

        // Create all sessions and track their names
        let mut session_names = Vec::new();
        for (name, context) in sessions.iter() {
            if let Ok(_id) = integration.create_session(name.clone(), context.clone()) {
                session_names.push(name.clone());
            }
        }

        // Get widget session names
        let widget_names: Vec<String> = integration
            .widget()
            .session_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // All created session names should be in the widget
        for name in &session_names {
            prop_assert!(
                widget_names.contains(name),
                "Session name '{}' not found in widget",
                name
            );
        }
    }
}
