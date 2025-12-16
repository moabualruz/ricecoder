//! Comprehensive session state management unit tests
//! **Feature: ricecoder-sessions, Unit Tests: State Management**
//! **Validates: Requirements 1.5, 1.6, 1.7**

use ricecoder_sessions::{
    BackgroundAgent, BackgroundAgentManager, ContextManager, HistoryManager, Message, MessageRole,
    Session, SessionContext, SessionManager, SessionMode, SessionStatus, SessionStore,
};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

fn create_test_session(name: &str) -> Session {
    Session::new(name.to_string(), create_test_context())
}

#[test]
fn test_session_manager_initialization() {
    let manager = SessionManager::new(10);

    assert_eq!(manager.session_count(), 0);
    assert!(manager.get_active_session().is_none());
    assert!(manager.list_sessions().is_empty());
}

#[test]
fn test_session_manager_create_single_session() {
    let mut manager = SessionManager::new(10);
    let context = create_test_context();

    let session = manager.create_session("Test Session".to_string(), context).unwrap();

    assert_eq!(manager.session_count(), 1);
    assert_eq!(manager.get_active_session().unwrap().id, session.id);
    assert_eq!(manager.list_sessions().len(), 1);
}

#[test]
fn test_session_manager_multiple_sessions() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session1 = manager.create_session("Session 1".to_string(), context.clone()).unwrap();
    let session2 = manager.create_session("Session 2".to_string(), context.clone()).unwrap();
    let session3 = manager.create_session("Session 3".to_string(), context).unwrap();

    assert_eq!(manager.session_count(), 3);

    // Last created should be active
    assert_eq!(manager.get_active_session().unwrap().id, session3.id);

    // All sessions should be in the list
    let sessions = manager.list_sessions();
    assert_eq!(sessions.len(), 3);
    let ids: std::collections::HashSet<String> = sessions.iter().map(|s| s.id.clone()).collect();
    assert!(ids.contains(&session1.id));
    assert!(ids.contains(&session2.id));
    assert!(ids.contains(&session3.id));
}

#[test]
fn test_session_manager_switch_sessions() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session1 = manager.create_session("Session 1".to_string(), context.clone()).unwrap();
    let session2 = manager.create_session("Session 2".to_string(), context.clone()).unwrap();
    let session3 = manager.create_session("Session 3".to_string(), context).unwrap();

    // Initially session3 should be active
    assert_eq!(manager.get_active_session().unwrap().id, session3.id);

    // Switch to session1
    manager.switch_session(&session1.id).unwrap();
    assert_eq!(manager.get_active_session().unwrap().id, session1.id);

    // Switch to session2
    manager.switch_session(&session2.id).unwrap();
    assert_eq!(manager.get_active_session().unwrap().id, session2.id);

    // Switch back to session3
    manager.switch_session(&session3.id).unwrap();
    assert_eq!(manager.get_active_session().unwrap().id, session3.id);
}

#[test]
fn test_session_manager_delete_sessions() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session1 = manager.create_session("Session 1".to_string(), context.clone()).unwrap();
    let session2 = manager.create_session("Session 2".to_string(), context.clone()).unwrap();
    let session3 = manager.create_session("Session 3".to_string(), context).unwrap();

    assert_eq!(manager.session_count(), 3);

    // Delete session2
    manager.delete_session(&session2.id).unwrap();
    assert_eq!(manager.session_count(), 2);
    assert!(manager.get_session(&session2.id).is_err());

    // Active session should now be session3 (last one)
    assert_eq!(manager.get_active_session().unwrap().id, session3.id);

    // Delete active session
    manager.delete_session(&session3.id).unwrap();
    assert_eq!(manager.session_count(), 1);
    assert!(manager.get_session(&session3.id).is_err());

    // Active session should now be session1
    assert_eq!(manager.get_active_session().unwrap().id, session1.id);
}

#[test]
fn test_session_manager_limit_enforcement() {
    let mut manager = SessionManager::new(2);
    let context = create_test_context();

    // Create up to the limit
    manager.create_session("Session 1".to_string(), context.clone()).unwrap();
    manager.create_session("Session 2".to_string(), context.clone()).unwrap();
    assert_eq!(manager.session_count(), 2);

    // Attempt to create beyond limit should fail
    let result = manager.create_session("Session 3".to_string(), context);
    assert!(result.is_err());
    assert_eq!(manager.session_count(), 2);
}

#[test]
fn test_session_manager_get_session_by_id() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session1 = manager.create_session("Session 1".to_string(), context.clone()).unwrap();
    let session2 = manager.create_session("Session 2".to_string(), context).unwrap();

    // Get existing sessions
    let retrieved1 = manager.get_session(&session1.id).unwrap();
    let retrieved2 = manager.get_session(&session2.id).unwrap();

    assert_eq!(retrieved1.id, session1.id);
    assert_eq!(retrieved1.name, "Session 1");
    assert_eq!(retrieved2.id, session2.id);
    assert_eq!(retrieved2.name, "Session 2");

    // Get non-existent session
    assert!(manager.get_session("nonexistent").is_err());
}

#[test]
fn test_session_manager_close_session() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session1 = manager.create_session("Session 1".to_string(), context.clone()).unwrap();
    let session2 = manager.create_session("Session 2".to_string(), context).unwrap();

    assert_eq!(manager.session_count(), 2);

    // Close session1
    manager.close_session(&session1.id).unwrap();
    assert_eq!(manager.session_count(), 1);
    assert!(manager.get_session(&session1.id).is_err());

    // Active session should still be session2
    assert_eq!(manager.get_active_session().unwrap().id, session2.id);
}

#[test]
fn test_session_manager_switch_to_nonexistent_session() {
    let mut manager = SessionManager::new(5);

    // Attempt to switch to non-existent session
    let result = manager.switch_session("nonexistent");
    assert!(result.is_err());
    assert!(manager.get_active_session().is_none());
}

#[test]
fn test_session_manager_delete_nonexistent_session() {
    let mut manager = SessionManager::new(5);

    // Attempt to delete non-existent session
    let result = manager.delete_session("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_session_manager_state_consistency_after_operations() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    // Create sessions
    let session1 = manager.create_session("Session 1".to_string(), context.clone()).unwrap();
    let session2 = manager.create_session("Session 2".to_string(), context.clone()).unwrap();

    // Verify initial state
    assert_eq!(manager.session_count(), 2);
    assert_eq!(manager.get_active_session().unwrap().id, session2.id);

    // Switch and verify
    manager.switch_session(&session1.id).unwrap();
    assert_eq!(manager.get_active_session().unwrap().id, session1.id);
    assert_eq!(manager.session_count(), 2);

    // Delete and verify
    manager.delete_session(&session2.id).unwrap();
    assert_eq!(manager.session_count(), 1);
    assert_eq!(manager.get_active_session().unwrap().id, session1.id);

    // Create new session and verify
    let session3 = manager.create_session("Session 3".to_string(), context).unwrap();
    assert_eq!(manager.session_count(), 2);
    assert_eq!(manager.get_active_session().unwrap().id, session3.id);
}

#[test]
fn test_context_manager_isolation() {
    let mut context_manager1 = ContextManager::new();
    let mut context_manager2 = ContextManager::new();

    let mut context1 = create_test_context();
    context1.project_path = Some("/project1".to_string());
    context1.files.push("file1.rs".to_string());

    let mut context2 = create_test_context();
    context2.project_path = Some("/project2".to_string());
    context2.files.push("file2.rs".to_string());

    context_manager1.set_context(context1);
    context_manager2.set_context(context2);

    // Verify isolation
    let retrieved1 = context_manager1.get_context().unwrap();
    let retrieved2 = context_manager2.get_context().unwrap();

    assert_eq!(retrieved1.project_path, Some("/project1".to_string()));
    assert_eq!(retrieved2.project_path, Some("/project2".to_string()));
    assert_eq!(retrieved1.files, vec!["file1.rs".to_string()]);
    assert_eq!(retrieved2.files, vec!["file2.rs".to_string()]);
}

#[test]
fn test_context_manager_empty_initial_state() {
    let context_manager = ContextManager::new();

    // Initially should have no context
    assert!(context_manager.get_context().is_none());
}

#[test]
fn test_context_manager_context_updates() {
    let mut context_manager = ContextManager::new();
    let mut context = create_test_context();

    context.project_path = Some("/initial".to_string());
    context_manager.set_context(context);

    // Update context
    let mut updated_context = context_manager.get_context().unwrap();
    updated_context.project_path = Some("/updated".to_string());
    updated_context.files.push("new_file.rs".to_string());
    context_manager.set_context(updated_context);

    // Verify update
    let final_context = context_manager.get_context().unwrap();
    assert_eq!(final_context.project_path, Some("/updated".to_string()));
    assert_eq!(final_context.files, vec!["new_file.rs".to_string()]);
}

#[test]
fn test_history_manager_message_ordering() {
    let mut history_manager = HistoryManager::new();

    let msg1 = Message::new(MessageRole::User, "First message".to_string());
    let msg2 = Message::new(MessageRole::Assistant, "Second message".to_string());
    let msg3 = Message::new(MessageRole::User, "Third message".to_string());

    history_manager.add_message(msg1);
    history_manager.add_message(msg2);
    history_manager.add_message(msg3);

    let recent = history_manager.get_recent_messages(3);
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].content(), "First message");
    assert_eq!(recent[1].content(), "Second message");
    assert_eq!(recent[2].content(), "Third message");
}

#[test]
fn test_history_manager_empty_history() {
    let history_manager = HistoryManager::new();

    let recent = history_manager.get_recent_messages(10);
    assert!(recent.is_empty());
}

#[test]
fn test_history_manager_limit_messages() {
    let mut history_manager = HistoryManager::new();

    // Add more messages than we'll retrieve
    for i in 0..10 {
        let msg = Message::new(MessageRole::User, format!("Message {}", i));
        history_manager.add_message(msg);
    }

    // Retrieve only last 5
    let recent = history_manager.get_recent_messages(5);
    assert_eq!(recent.len(), 5);
    assert_eq!(recent[0].content(), "Message 5");
    assert_eq!(recent[4].content(), "Message 9");
}

#[test]
fn test_background_agent_manager_initialization() {
    let agent_manager = BackgroundAgentManager::new();

    // Initially should have no agents
    assert!(agent_manager.list_agents().is_empty());
}

#[test]
fn test_background_agent_manager_agent_lifecycle() {
    let mut agent_manager = BackgroundAgentManager::new();

    // Create agent
    let agent_id = agent_manager.create_agent("test_agent".to_string(), Some("Testing".to_string())).unwrap();

    // Verify agent exists
    let agents = agent_manager.list_agents();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].id, agent_id);

    // Update agent status
    agent_manager.update_agent_status(&agent_id, ricecoder_sessions::AgentStatus::Completed).unwrap();

    // Verify status update
    let updated_agents = agent_manager.list_agents();
    assert_eq!(updated_agents[0].status, ricecoder_sessions::AgentStatus::Completed);

    // Remove agent
    agent_manager.remove_agent(&agent_id).unwrap();
    assert!(agent_manager.list_agents().is_empty());
}

#[test]
fn test_background_agent_manager_concurrent_operations() {
    let agent_manager = Arc::new(Mutex::new(BackgroundAgentManager::new()));
    let mut handles = vec![];

    // Spawn multiple threads creating agents
    for i in 0..5 {
        let manager_clone = Arc::clone(&agent_manager);
        let handle = std::thread::spawn(move || {
            let mut manager = tokio::runtime::Runtime::new().unwrap().block_on(manager_clone.lock());
            let agent_id = manager.create_agent(format!("agent_{}", i), Some(format!("Task {}", i))).unwrap();
            agent_id
        });
        handles.push(handle);
    }

    // Wait for all threads and collect agent IDs
    let mut agent_ids = vec![];
    for handle in handles {
        agent_ids.push(handle.join().unwrap());
    }

    // Verify all agents were created
    let manager = tokio::runtime::Runtime::new().unwrap().block_on(agent_manager.lock());
    assert_eq!(manager.list_agents().len(), 5);

    // Verify all agent IDs are unique
    let mut unique_ids = std::collections::HashSet::new();
    for id in &agent_ids {
        assert!(unique_ids.insert(id.clone()));
    }
}