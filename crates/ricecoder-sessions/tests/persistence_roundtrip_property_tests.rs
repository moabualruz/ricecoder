//! Property-based tests for session persistence round-trip
//! **Feature: ricecoder-sessions, Property 1: Session Persistence Round-Trip**
//! **Validates: Requirements 2.1, 2.2**

use proptest::prelude::*;
use ricecoder_sessions::{
    Message, MessageRole, Session, SessionContext, SessionMode, SessionStore,
};
use tempfile::TempDir;

/// Strategy for generating valid session contexts
fn session_context_strategy() -> impl Strategy<Value = SessionContext> {
    (
        prop::string::string_regex("[a-z]+").unwrap(),
        prop::string::string_regex("[a-z0-9-]+").unwrap(),
    )
        .prop_map(|(provider, model)| SessionContext::new(provider, model, SessionMode::Chat))
}

/// Strategy for generating valid messages
fn message_strategy() -> impl Strategy<Value = Message> {
    (
        prop::string::string_regex("[a-zA-Z0-9 .,!?]{1,100}").unwrap(),
        prop::bool::ANY,
    )
        .prop_map(|(content, is_user)| {
            let role = if is_user {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };
            Message::new(role, content)
        })
}

/// Strategy for generating valid sessions
fn session_strategy() -> impl Strategy<Value = Session> {
    (
        prop::string::string_regex("[a-zA-Z0-9 ]{1,50}").unwrap(),
        session_context_strategy(),
        prop::collection::vec(message_strategy(), 0..5),
    )
        .prop_map(|(name, context, messages)| {
            let mut session = Session::new(name, context);
            session.history = messages;
            session
        })
}

/// Property: For any session, saving to disk and loading SHALL produce an equivalent
/// session with identical context, history, and metadata.
///
/// This property tests that:
/// 1. A session can be saved to disk
/// 2. The session can be loaded back from disk
/// 3. The loaded session is equivalent to the original
/// 4. All fields (context, history, metadata) are preserved
#[test]
fn prop_session_persistence_roundtrip() {
    proptest!(|(session in session_strategy())| {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let sessions_dir = temp_dir.path().join("sessions");
        let archive_dir = temp_dir.path().join("archive");

        // Create a session store with temporary directories
        let store = SessionStore::with_dirs(sessions_dir, archive_dir)
            .expect("Failed to create SessionStore");

        // Save the session
        let save_result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(store.save(&session));
        prop_assert!(save_result.is_ok(), "Failed to save session");

        // Load the session back
        let load_result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(store.load(&session.id));
        prop_assert!(load_result.is_ok(), "Failed to load session");

        let loaded_session = load_result.unwrap();

        // Verify all fields are preserved
        prop_assert_eq!(loaded_session.id, session.id, "Session ID mismatch");
        prop_assert_eq!(loaded_session.name, session.name, "Session name mismatch");
        prop_assert_eq!(loaded_session.status, session.status, "Session status mismatch");

        // Verify context is preserved
        prop_assert_eq!(
            loaded_session.context.provider, session.context.provider,
            "Provider mismatch"
        );
        prop_assert_eq!(
            loaded_session.context.model, session.context.model,
            "Model mismatch"
        );
        prop_assert_eq!(
            loaded_session.context.mode, session.context.mode,
            "Mode mismatch"
        );

        // Verify history is preserved
        prop_assert_eq!(
            loaded_session.history.len(),
            session.history.len(),
            "History length mismatch"
        );

        for (i, (original_msg, loaded_msg)) in session
            .history
            .iter()
            .zip(loaded_session.history.iter())
            .enumerate()
        {
            prop_assert_eq!(
                &loaded_msg.id, &original_msg.id,
                "Message {} ID mismatch",
                i
            );
            prop_assert_eq!(
                loaded_msg.role, original_msg.role,
                "Message {} role mismatch",
                i
            );
            prop_assert_eq!(
                &loaded_msg.content(), &original_msg.content(),
                "Message {} content mismatch",
                i
            );
        }
    });
}

/// Property: For any session, the session file SHALL exist after saving and
/// SHALL be valid JSON.
#[test]
fn prop_session_file_is_valid_json() {
    proptest!(|(session in session_strategy())| {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let sessions_dir = temp_dir.path().join("sessions");
        let archive_dir = temp_dir.path().join("archive");

        let store = SessionStore::with_dirs(sessions_dir.clone(), archive_dir)
            .expect("Failed to create SessionStore");

        // Save the session
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(store.save(&session))
            .expect("Failed to save session");

        // Verify the file exists
        let session_file = sessions_dir.join(format!("{}.json", session.id));
        prop_assert!(session_file.exists(), "Session file does not exist");

        // Verify the file contains valid JSON
        let file_content = std::fs::read_to_string(&session_file)
            .expect("Failed to read session file");
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&file_content);
        prop_assert!(parsed.is_ok(), "Session file is not valid JSON");
    });
}

/// Property: For any session, saving multiple times SHALL result in the same
/// file content (deterministic serialization).
#[test]
fn prop_session_persistence_is_deterministic() {
    proptest!(|(session in session_strategy())| {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let sessions_dir = temp_dir.path().join("sessions");
        let archive_dir = temp_dir.path().join("archive");

        let store = SessionStore::with_dirs(sessions_dir.clone(), archive_dir)
            .expect("Failed to create SessionStore");

        // Save the session
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(store.save(&session))
            .expect("Failed to save session");

        let session_file = sessions_dir.join(format!("{}.json", session.id));
        let first_content = std::fs::read_to_string(&session_file)
            .expect("Failed to read session file");

        // Delete and save again
        std::fs::remove_file(&session_file).expect("Failed to delete session file");

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(store.save(&session))
            .expect("Failed to save session again");

        let second_content = std::fs::read_to_string(&session_file)
            .expect("Failed to read session file");

        // Content should be identical
        prop_assert_eq!(
            first_content, second_content,
            "Session serialization is not deterministic"
        );
    });
}

/// Property: For any session, the session file SHALL be readable and contain
/// all session data.
#[test]
fn prop_session_file_contains_all_data() {
    proptest!(|(session in session_strategy())| {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let sessions_dir = temp_dir.path().join("sessions");
        let archive_dir = temp_dir.path().join("archive");

        let store = SessionStore::with_dirs(sessions_dir.clone(), archive_dir)
            .expect("Failed to create SessionStore");

        // Save the session
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(store.save(&session))
            .expect("Failed to save session");

        let session_file = sessions_dir.join(format!("{}.json", session.id));
        let file_content = std::fs::read_to_string(&session_file)
            .expect("Failed to read session file");

        // Verify the file contains key fields
        prop_assert!(
            file_content.contains(&session.id),
            "File does not contain session ID"
        );
        prop_assert!(
            file_content.contains(&session.name),
            "File does not contain session name"
        );
        prop_assert!(
            file_content.contains(&session.context.provider),
            "File does not contain provider"
        );
        prop_assert!(
            file_content.contains(&session.context.model),
            "File does not contain model"
        );
    });
}
