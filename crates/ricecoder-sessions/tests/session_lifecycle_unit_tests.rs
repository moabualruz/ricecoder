//! Comprehensive session lifecycle unit tests
//! **Feature: ricecoder-sessions, Unit Tests: Session Lifecycle**
//! **Validates: Requirements 1.1, 1.2, 1.3, 1.4**

use std::sync::Arc;

use ricecoder_sessions::{
    BackgroundAgent, BackgroundAgentManager, ContextManager, HistoryManager, Message, MessageRole,
    Session, SessionContext, SessionManager, SessionMode, SessionStatus, SessionStore,
};
use tempfile::TempDir;
use tokio::sync::Mutex;

fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

fn create_test_session(name: &str) -> Session {
    Session::new(name.to_string(), create_test_context())
}

#[test]
fn test_session_creation_initial_state() {
    let context = create_test_context();
    let session = Session::new("Test Session".to_string(), context.clone());

    // Verify initial state
    assert!(!session.id.is_empty());
    assert_eq!(session.name, "Test Session");
    assert_eq!(session.status, SessionStatus::Active);
    assert_eq!(session.context.provider, context.provider);
    assert_eq!(session.context.model, context.model);
    assert_eq!(session.context.mode, context.mode);
    assert!(session.history.is_empty());
    assert!(session.background_agents.is_empty());

    // Verify timestamps are set
    assert!(session.created_at <= session.updated_at);
}

#[test]
fn test_session_status_transitions() {
    let mut session = create_test_session("Test Session");

    // Initial status should be Active
    assert_eq!(session.status, SessionStatus::Active);

    // Test transition to Paused
    session.status = SessionStatus::Paused;
    assert_eq!(session.status, SessionStatus::Paused);

    // Test transition to Archived
    session.status = SessionStatus::Archived;
    assert_eq!(session.status, SessionStatus::Archived);

    // Test transition back to Active
    session.status = SessionStatus::Active;
    assert_eq!(session.status, SessionStatus::Active);
}

#[test]
fn test_session_context_isolation() {
    let mut context1 = create_test_context();
    context1.project_path = Some("/project1".to_string());
    context1.files.push("file1.rs".to_string());

    let mut context2 = create_test_context();
    context2.project_path = Some("/project2".to_string());
    context2.files.push("file2.rs".to_string());

    let session1 = Session::new("Session 1".to_string(), context1);
    let session2 = Session::new("Session 2".to_string(), context2);

    // Verify contexts are isolated
    assert_eq!(session1.context.project_path, Some("/project1".to_string()));
    assert_eq!(session2.context.project_path, Some("/project2".to_string()));
    assert_eq!(session1.context.files, vec!["file1.rs".to_string()]);
    assert_eq!(session2.context.files, vec!["file2.rs".to_string()]);
}

#[test]
fn test_session_history_management() {
    let mut session = create_test_session("Test Session");

    // Initially empty
    assert!(session.history.is_empty());

    // Add messages
    let msg1 = Message::new(MessageRole::User, "Hello".to_string());
    let msg2 = Message::new(MessageRole::Assistant, "Hi there!".to_string());

    session.history.push(msg1.clone());
    session.history.push(msg2.clone());

    // Verify messages are stored
    assert_eq!(session.history.len(), 2);
    assert_eq!(session.history[0].role, MessageRole::User);
    assert_eq!(session.history[1].role, MessageRole::Assistant);
    assert_eq!(session.history[0].content(), "Hello");
    assert_eq!(session.history[1].content(), "Hi there!");
}

#[test]
fn test_session_background_agents() {
    let mut session = create_test_session("Test Session");

    // Initially empty
    assert!(session.background_agents.is_empty());

    // Add background agents
    let agent1 = BackgroundAgent::new(
        "code_review".to_string(),
        Some("Reviewing code".to_string()),
    );
    let agent2 = BackgroundAgent::new("diff_analysis".to_string(), None);

    session.background_agents.push(agent1);
    session.background_agents.push(agent2);

    // Verify agents are stored
    assert_eq!(session.background_agents.len(), 2);
    assert_eq!(session.background_agents[0].agent_type, "code_review");
    assert_eq!(session.background_agents[1].agent_type, "diff_analysis");
    assert_eq!(
        session.background_agents[0].task,
        Some("Reviewing code".to_string())
    );
    assert_eq!(session.background_agents[1].task, None);
}

#[test]
fn test_session_timestamps_update() {
    let mut session = create_test_session("Test Session");
    let original_created = session.created_at;
    let original_updated = session.updated_at;

    // Simulate time passing and update
    std::thread::sleep(std::time::Duration::from_millis(1));
    session.updated_at = chrono::Utc::now();

    // Created timestamp should remain the same
    assert_eq!(session.created_at, original_created);
    // Updated timestamp should be newer
    assert!(session.updated_at > original_updated);
}

#[test]
fn test_session_unique_ids() {
    let session1 = create_test_session("Session 1");
    let session2 = create_test_session("Session 2");

    // IDs should be unique
    assert_ne!(session1.id, session2.id);

    // Both should be valid UUIDs
    assert!(uuid::Uuid::parse_str(&session1.id).is_ok());
    assert!(uuid::Uuid::parse_str(&session2.id).is_ok());
}

#[test]
fn test_session_context_custom_data() {
    let mut session = create_test_session("Test Session");

    // Add custom context data
    session
        .context
        .custom
        .insert("workspace".to_string(), serde_json::json!("dev"));
    session
        .context
        .custom
        .insert("priority".to_string(), serde_json::json!(1));

    // Verify custom data is stored
    assert_eq!(session.context.custom.len(), 2);
    assert_eq!(
        session.context.custom.get("workspace"),
        Some(&serde_json::json!("dev"))
    );
    assert_eq!(
        session.context.custom.get("priority"),
        Some(&serde_json::json!(1))
    );
}

#[test]
fn test_session_mode_variations() {
    let chat_session = Session::new(
        "Chat".to_string(),
        SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat),
    );
    let code_session = Session::new(
        "Code".to_string(),
        SessionContext::new(
            "anthropic".to_string(),
            "claude-3".to_string(),
            SessionMode::Code,
        ),
    );
    let vibe_session = Session::new(
        "Vibe".to_string(),
        SessionContext::new(
            "openai".to_string(),
            "gpt-3.5".to_string(),
            SessionMode::Vibe,
        ),
    );

    assert_eq!(chat_session.context.mode, SessionMode::Chat);
    assert_eq!(code_session.context.mode, SessionMode::Code);
    assert_eq!(vibe_session.context.mode, SessionMode::Vibe);
}

#[test]
fn test_session_with_complex_history() {
    let mut session = create_test_session("Complex Session");

    // Add various types of messages
    let user_msg = Message::new(MessageRole::User, "Please analyze this code".to_string());
    let mut assistant_msg = Message::new(
        MessageRole::Assistant,
        "I'll help you analyze it".to_string(),
    );
    assistant_msg.add_code(
        "rust".to_string(),
        "fn main() { println!(\"Hello\"); }".to_string(),
    );
    assistant_msg.add_reasoning("This is a simple Rust program".to_string());

    let system_msg = Message::new(MessageRole::System, "Session initialized".to_string());

    session.history.push(user_msg);
    session.history.push(assistant_msg);
    session.history.push(system_msg);

    // Verify complex history
    assert_eq!(session.history.len(), 3);
    assert_eq!(session.history[0].role, MessageRole::User);
    assert_eq!(session.history[1].role, MessageRole::Assistant);
    assert_eq!(session.history[2].role, MessageRole::System);

    // Verify message parts in assistant message
    assert!(session.history[1].parts.len() > 1); // Should have text, code, and reasoning
}

#[test]
fn test_session_background_agent_status_transitions() {
    let mut session = create_test_session("Agent Session");
    let mut agent = BackgroundAgent::new("test_agent".to_string(), Some("Testing".to_string()));

    // Initial status should be Running
    assert_eq!(agent.status, ricecoder_sessions::AgentStatus::Running);

    // Test status transitions
    agent.status = ricecoder_sessions::AgentStatus::Completed;
    assert_eq!(agent.status, ricecoder_sessions::AgentStatus::Completed);

    agent.status = ricecoder_sessions::AgentStatus::Failed;
    assert_eq!(agent.status, ricecoder_sessions::AgentStatus::Failed);

    agent.status = ricecoder_sessions::AgentStatus::Cancelled;
    assert_eq!(agent.status, ricecoder_sessions::AgentStatus::Cancelled);

    session.background_agents.push(agent);
    assert_eq!(
        session.background_agents[0].status,
        ricecoder_sessions::AgentStatus::Cancelled
    );
}

#[test]
fn test_session_context_file_tracking() {
    let mut session = create_test_session("File Session");

    // Add files to context
    session.context.files.push("src/main.rs".to_string());
    session.context.files.push("src/lib.rs".to_string());
    session.context.files.push("Cargo.toml".to_string());

    // Verify files are tracked
    assert_eq!(session.context.files.len(), 3);
    assert!(session.context.files.contains(&"src/main.rs".to_string()));
    assert!(session.context.files.contains(&"src/lib.rs".to_string()));
    assert!(session.context.files.contains(&"Cargo.toml".to_string()));
}

#[test]
fn test_session_immutability_through_operations() {
    let session = create_test_session("Immutable Test");

    // Store original values
    let original_id = session.id.clone();
    let original_name = session.name.clone();
    let original_status = session.status;
    let original_context = session.context.clone();

    // Operations shouldn't modify the original session
    // (This is more of a design validation - in practice we'd use Arc<Mutex<>> for shared state)

    assert_eq!(session.id, original_id);
    assert_eq!(session.name, original_name);
    assert_eq!(session.status, original_status);
    assert_eq!(session.context.provider, original_context.provider);
}
