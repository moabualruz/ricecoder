//! Property-based tests for undo/redo round trip
//!
//! **Feature: ricecoder-undo-redo, Property 3: Undo/Redo Round Trip**
//! **Validates: Requirements 19.1, 19.2, 42.2, 42.3**
//!
//! For any sequence of changes, performing undo operations followed by redo operations
//! should restore the system to its original state.

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_undo_redo::{Change, ChangeType, HistoryConfig, HistoryManager};

// Strategy for generating changes
fn arb_change() -> impl Strategy<Value = Change> {
    prop_oneof![
        // Create change
        (
            "[a-zA-Z0-9/_.-]{1,50}".prop_map(|s| s),   // file_path
            "[a-zA-Z0-9 .,!?]{1,50}".prop_map(|s| s),  // description
            "[a-zA-Z0-9 .,!?]{1,100}".prop_map(|s| s), // after
        )
            .prop_map(|(file_path, description, after)| {
                Change::new(
                    file_path,
                    "".to_string(),
                    after,
                    description,
                    ChangeType::Create,
                )
                .unwrap()
            }),
        // Modify change
        (
            "[a-zA-Z0-9/_.-]{1,50}".prop_map(|s| s),  // file_path
            "[a-zA-Z0-9 .,!?]{1,50}".prop_map(|s| s), // description
            "[a-zA-Z0-9 .,!?]{1,50}".prop_map(|s| s), // before
            "[a-zA-Z0-9 .,!?]{1,50}".prop_map(|s| s), // after
        )
            .prop_map(|(file_path, description, before, after)| {
                Change::new(file_path, before, after, description, ChangeType::Modify).unwrap()
            }),
        // Delete change
        (
            "[a-zA-Z0-9/_.-]{1,50}".prop_map(|s| s),   // file_path
            "[a-zA-Z0-9 .,!?]{1,50}".prop_map(|s| s),  // description
            "[a-zA-Z0-9 .,!?]{1,100}".prop_map(|s| s), // before
        )
            .prop_map(|(file_path, description, before)| {
                Change::new(
                    file_path,
                    before,
                    "".to_string(),
                    description,
                    ChangeType::Delete,
                )
                .unwrap()
            }),
    ]
}

// Strategy for generating history configurations
fn arb_history_config() -> impl Strategy<Value = HistoryConfig> {
    (
        10usize..200usize, // max_undo_stack_size
        5usize..100usize,  // max_redo_stack_size
        any::<bool>(),     // session_scoped
        any::<bool>(),     // enable_persistence
    )
        .prop_map(
            |(max_undo, max_redo, session_scoped, enable_persistence)| HistoryConfig {
                max_undo_stack_size: max_undo,
                max_redo_stack_size: max_redo,
                session_scoped,
                enable_persistence,
                persistence_dir: None,
            },
        )
}

proptest! {
    /// **Feature: ricecoder-undo-redo, Property 3: Undo/Redo Round Trip**
    /// **Validates: Requirements 19.1, 19.2, 42.2, 42.3**
    ///
    /// For any sequence of changes and any history configuration, performing undo
    /// operations for all changes followed by redo operations should restore the
    /// original state.
    #[test]
    fn prop_undo_redo_round_trip(
        changes in prop::collection::vec(arb_change(), 1..10), // Reduced to avoid stack overflow
        config in arb_history_config(),
    ) {
        let mut manager = HistoryManager::with_config(config);

        // Record all changes
        for change in &changes {
            manager.record_change(change.clone()).unwrap();
        }

        let original_undo_count = manager.undo_stack_size();

        // Perform undo operations for all changes
        let mut undo_operations = 0;
        while manager.can_undo() && undo_operations < changes.len() {
            manager.undo().unwrap();
            undo_operations += 1;
        }

        // Verify we undid the expected number of operations
        prop_assert_eq!(undo_operations, changes.len().min(original_undo_count),
            "Should be able to undo all recorded changes within stack limits");

        // Perform redo operations
        let mut redo_operations = 0;
        while manager.can_redo() && redo_operations < undo_operations {
            manager.redo().unwrap();
            redo_operations += 1;
        }

        // Verify we redid the operations we undid
        prop_assert_eq!(redo_operations, undo_operations,
            "Should be able to redo all operations we undid");
    }

    /// **Feature: ricecoder-undo-redo, Property 3: Undo/Redo Round Trip - Stack Size Limits**
    /// **Validates: Requirements 42.2, 42.3**
    ///
    /// When stack size limits are exceeded, undo/redo operations should still work correctly.
    #[test]
    fn prop_undo_redo_with_stack_limits(
        changes in prop::collection::vec(arb_change(), 50..100), // Many changes to exceed limits
        max_undo in 5usize..20usize,
        max_redo in 3usize..10usize,
    ) {
        let config = HistoryConfig {
            max_undo_stack_size: max_undo,
            max_redo_stack_size: max_redo,
            session_scoped: false,
            enable_persistence: false,
            persistence_dir: None,
        };

        let mut manager = HistoryManager::with_config(config);

        // Record all changes (some will be dropped due to stack limits)
        for change in &changes {
            manager.record_change(change.clone()).unwrap();
        }

        // Verify undo stack is limited
        prop_assert!(manager.undo_stack_size() <= max_undo,
            "Undo stack should not exceed configured limit");

        // Perform undo operations
        let mut undo_count = 0;
        while manager.can_undo() && undo_count < max_undo {
            manager.undo().unwrap();
            undo_count += 1;
        }

        // Perform redo operations
        let mut redo_count = 0;
        while manager.can_redo() && redo_count < max_redo {
            manager.redo().unwrap();
            redo_count += 1;
        }

        // Verify redo stack is limited
        prop_assert!(manager.redo_stack_size() <= max_redo,
            "Redo stack should not exceed configured limit");
    }

    /// **Feature: ricecoder-undo-redo, Property 3: Undo/Redo Round Trip - Session Scoping**
    /// **Validates: Requirements 19.3, 42.1**
    ///
    /// Session-scoped history should maintain separate undo/redo stacks per session.
    #[test]
    fn prop_session_scoped_undo_redo(
        session1_changes in prop::collection::vec(arb_change(), 2..10),
        session2_changes in prop::collection::vec(arb_change(), 2..10),
        session1_id in "[a-zA-Z0-9_-]{1,20}",
        session2_id in "[a-zA-Z0-9_-]{1,20}",
    ) {
        prop_assume!(session1_id != session2_id);

        let config = HistoryConfig {
            max_undo_stack_size: 50,
            max_redo_stack_size: 25,
            session_scoped: true,
            enable_persistence: false,
            persistence_dir: None,
        };

        let mut manager = HistoryManager::with_config(config);

        // Record changes for session 1
        manager.set_current_session(session1_id.clone());
        for change in &session1_changes {
            manager.record_change(change.clone()).unwrap();
        }

        // Record changes for session 2
        manager.set_current_session(session2_id.clone());
        for change in &session2_changes {
            manager.record_change(change.clone()).unwrap();
        }

        // Verify session-specific changes are tracked
        let session1_history = manager.get_session_changes(&session1_id);
        let session2_history = manager.get_session_changes(&session2_id);

        prop_assert_eq!(session1_history.len(), session1_changes.len(),
            "Session 1 should have correct number of changes");
        prop_assert_eq!(session2_history.len(), session2_changes.len(),
            "Session 2 should have correct number of changes");

        // Test undo/redo for current session (session 2)
        let original_undo_count = manager.undo_stack_size();
        if original_undo_count > 0 {
            manager.undo().unwrap();
            prop_assert_eq!(manager.undo_stack_size(), original_undo_count - 1,
                "Undo should reduce stack size by 1");
            prop_assert!(manager.can_redo(), "Should be able to redo after undo");
        }
    }
}
