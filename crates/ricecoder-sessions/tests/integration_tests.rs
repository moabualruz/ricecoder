//! Integration tests for ricecoder-sessions
//! Tests full workflows combining multiple components
//! **Feature: ricecoder-sessions, Integration Tests**
//! **Validates: Requirements 1.1, 1.2, 2.1, 3.1, 4.1**

use ricecoder_sessions::{
    BackgroundAgent, BackgroundAgentManager, ContextManager, HistoryManager, Message, MessageRole,
    Session, SessionContext, SessionManager, SessionMode, SessionRouter, SessionStore,
    SharePermissions, ShareService,
};
use tempfile::TempDir;

// ============================================================================
// Test Fixtures
// ============================================================================

fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

fn create_test_session(name: &str) -> Session {
    Session::new(name.to_string(), create_test_context())
}

// ============================================================================
// Full Session Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_session_lifecycle_create_switch_close_restore() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();
    let mut manager = SessionManager::new(10);

    // 1. Create a session
    let context = create_test_context();
    let session1 = manager
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();
    let session1_id = session1.id.clone();

    // Verify session is active
    let active = manager.get_active_session().unwrap();
    assert_eq!(active.id, session1_id);

    // 2. Create another session
    let session2 = manager
        .create_session("Session 2".to_string(), context.clone())
        .unwrap();
    let session2_id = session2.id.clone();

    // 3. Switch to session 2
    manager.switch_session(&session2_id).unwrap();
    let active = manager.get_active_session().unwrap();
    assert_eq!(active.id, session2_id);

    // 4. Persist sessions to disk
    store.save(&session1).await.unwrap();
    store.save(&session2).await.unwrap();

    // 5. Verify both sessions are persisted
    assert!(store.exists(&session1_id));
    assert!(store.exists(&session2_id));

    // 6. Load sessions from disk
    let loaded_session1 = store.load(&session1_id).await.unwrap();
    let loaded_session2 = store.load(&session2_id).await.unwrap();

    // 7. Verify loaded sessions match originals
    assert_eq!(loaded_session1.id, session1_id);
    assert_eq!(loaded_session1.name, "Session 1");
    assert_eq!(loaded_session2.id, session2_id);
    assert_eq!(loaded_session2.name, "Session 2");

    // 8. Delete session 1
    manager.delete_session(&session1_id).unwrap();
    store.delete(&session1_id).await.unwrap();

    // 9. Verify session 1 is deleted
    assert!(!store.exists(&session1_id));

    // 10. Verify session 2 is still active
    let active = manager.get_active_session().unwrap();
    assert_eq!(active.id, session2_id);
}

#[tokio::test]
async fn test_session_lifecycle_with_history() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();
    let mut manager = SessionManager::new(10);
    let mut history_manager = HistoryManager::new();

    // 1. Create a session
    let context = create_test_context();
    let mut session = manager
        .create_session("Session with History".to_string(), context)
        .unwrap();
    let session_id = session.id.clone();

    // 2. Add messages to history
    let msg1 = Message::new(MessageRole::User, "Hello, assistant!".to_string());
    let msg2 = Message::new(MessageRole::Assistant, "Hi there!".to_string());
    let msg3 = Message::new(MessageRole::User, "How are you?".to_string());

    history_manager.add_message(msg1.clone());
    history_manager.add_message(msg2.clone());
    history_manager.add_message(msg3.clone());

    // 3. Get history and verify ordering
    let history = history_manager.get_recent_messages(3);
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].role, MessageRole::User);
    assert_eq!(history[1].role, MessageRole::Assistant);
    assert_eq!(history[2].role, MessageRole::User);

    // 4. Update session with history
    session.history = history;

    // 5. Persist session to disk
    store.save(&session).await.unwrap();

    // 6. Load session from disk
    let loaded = store.load(&session_id).await.unwrap();

    // 7. Verify history is preserved
    assert_eq!(loaded.history.len(), 3);
    assert_eq!(loaded.history[0].role, MessageRole::User);
    assert_eq!(loaded.history[1].role, MessageRole::Assistant);
    assert_eq!(loaded.history[2].role, MessageRole::User);
}

// ============================================================================
// Multi-Session Operations Tests
// ============================================================================

#[tokio::test]
async fn test_multi_session_operations_create_and_switch() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    // Create 3 sessions
    let session1 = manager
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();
    let session1_id = session1.id.clone();

    let session2 = manager
        .create_session("Session 2".to_string(), context.clone())
        .unwrap();
    let session2_id = session2.id.clone();

    let session3 = manager
        .create_session("Session 3".to_string(), context.clone())
        .unwrap();
    let session3_id = session3.id.clone();

    // Verify all sessions exist
    assert_eq!(manager.list_sessions().len(), 3);

    // Switch between sessions
    manager.switch_session(&session2_id).unwrap();
    assert_eq!(manager.get_active_session().unwrap().id, session2_id);

    manager.switch_session(&session3_id).unwrap();
    assert_eq!(manager.get_active_session().unwrap().id, session3_id);

    manager.switch_session(&session1_id).unwrap();
    assert_eq!(manager.get_active_session().unwrap().id, session1_id);

    // Delete a session
    manager.delete_session(&session2_id).unwrap();
    assert_eq!(manager.list_sessions().len(), 2);

    // Verify remaining sessions
    let sessions = manager.list_sessions();
    let ids: Vec<String> = sessions.iter().map(|s| s.id.clone()).collect();
    assert!(ids.contains(&session1_id));
    assert!(ids.contains(&session3_id));
    assert!(!ids.contains(&session2_id));
}

#[tokio::test]
async fn test_multi_session_context_isolation() {
    // Create separate context managers for each session to test isolation
    let mut context_manager1 = ContextManager::new();
    let mut context_manager2 = ContextManager::new();

    // Set context for session 1
    let mut context1 = create_test_context();
    context1.project_path = Some("/project1".to_string());
    context_manager1.set_context(context1.clone());

    // Set context for session 2
    let mut context2 = create_test_context();
    context2.project_path = Some("/project2".to_string());
    context_manager2.set_context(context2.clone());

    // Verify contexts are isolated
    let retrieved1 = context_manager1.get_context().unwrap();
    let retrieved2 = context_manager2.get_context().unwrap();

    assert_eq!(retrieved1.project_path, Some("/project1".to_string()));
    assert_eq!(retrieved2.project_path, Some("/project2".to_string()));

    // Modify context 1
    let mut modified_context1 = retrieved1;
    modified_context1.project_path = Some("/project1_modified".to_string());
    context_manager1.set_context(modified_context1);

    // Verify context 2 is unchanged
    let retrieved2_again = context_manager2.get_context().unwrap();
    assert_eq!(retrieved2_again.project_path, Some("/project2".to_string()));
}

#[tokio::test]
async fn test_multi_session_persistence_and_restore() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    // Create and persist multiple sessions
    let session1 = create_test_session("Session 1");
    let session2 = create_test_session("Session 2");
    let session3 = create_test_session("Session 3");

    let session1_id = session1.id.clone();
    let session2_id = session2.id.clone();
    let session3_id = session3.id.clone();

    store.save(&session1).await.unwrap();
    store.save(&session2).await.unwrap();
    store.save(&session3).await.unwrap();

    // List all sessions
    let sessions = store.list().await.unwrap();
    assert_eq!(sessions.len(), 3);

    // Load each session and verify
    let loaded1 = store.load(&session1_id).await.unwrap();
    let loaded2 = store.load(&session2_id).await.unwrap();
    let loaded3 = store.load(&session3_id).await.unwrap();

    assert_eq!(loaded1.name, "Session 1");
    assert_eq!(loaded2.name, "Session 2");
    assert_eq!(loaded3.name, "Session 3");
}

// ============================================================================
// Session Sharing Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_session_sharing_workflow_generate_and_access() {
    let share_service = ShareService::new();
    let session = create_test_session("Shared Session");
    let session_id = session.id.clone();

    // 1. Generate a share link
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let share = share_service
        .generate_share_link(&session_id, permissions.clone(), None)
        .unwrap();

    let share_id = share.id.clone();

    // 2. Verify share was created
    assert_eq!(share.session_id, session_id);
    assert_eq!(share.permissions.read_only, true);
    assert_eq!(share.permissions.include_history, true);
    assert_eq!(share.permissions.include_context, true);

    // 3. Retrieve the share
    let retrieved_share = share_service.get_share(&share_id).unwrap();
    assert_eq!(retrieved_share.id, share_id);
    assert_eq!(retrieved_share.session_id, session_id);

    // 4. Create a shared session view
    let shared_view = share_service.create_shared_session_view(&session, &permissions);
    assert_eq!(shared_view.id, session.id);
}

#[tokio::test]
async fn test_session_sharing_workflow_import_shared_session() {
    let share_service = ShareService::new();
    let original_session = create_test_session("Original Session");
    let original_id = original_session.id.clone();

    // 1. Generate a share link
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let share = share_service
        .generate_share_link(&original_id, permissions.clone(), None)
        .unwrap();

    // 2. Create a shared view
    let shared_view = share_service.create_shared_session_view(&original_session, &permissions);

    // 3. Import the shared session
    let imported = share_service
        .import_shared_session(&share.id, &shared_view)
        .unwrap();

    // 4. Verify imported session is a new session with same data
    assert_ne!(imported.id, original_id);
    assert_eq!(imported.name, original_session.name);
    assert_eq!(imported.context.provider, original_session.context.provider);
    assert_eq!(imported.context.model, original_session.context.model);
}

#[tokio::test]
async fn test_session_sharing_workflow_with_privacy_settings() {
    let share_service = ShareService::new();
    let mut session = create_test_session("Session with Data");
    let session_id = session.id.clone();

    // Add some data to the session
    session.context.project_path = Some("/project".to_string());
    session.context.files.push("file1.rs".to_string());
    session
        .history
        .push(Message::new(MessageRole::User, "Test message".to_string()));

    // 1. Create share with history but no context
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: false,
    };

    let _share = share_service
        .generate_share_link(&session_id, permissions.clone(), None)
        .unwrap();

    // 2. Create shared view with these permissions
    let shared_view = share_service.create_shared_session_view(&session, &permissions);

    // 3. Verify context files are excluded
    assert_eq!(shared_view.context.files.len(), 0);
    // Note: project_path is not cleared by include_context=false, only files and custom data
    assert_eq!(shared_view.context.custom.len(), 0);

    // 4. Verify history is included
    assert_eq!(shared_view.history.len(), 1);

    // 5. Create share with context but no history
    let permissions2 = SharePermissions {
        read_only: true,
        include_history: false,
        include_context: true,
    };

    let shared_view2 = share_service.create_shared_session_view(&session, &permissions2);

    // 6. Verify history is excluded
    assert_eq!(shared_view2.history.len(), 0);

    // 7. Verify context is included
    assert_eq!(shared_view2.context.files.len(), 1);
    assert_eq!(
        shared_view2.context.project_path,
        Some("/project".to_string())
    );
}

#[tokio::test]
async fn test_session_sharing_workflow_share_expiration() {
    use chrono::Duration;

    let share_service = ShareService::new();
    let session = create_test_session("Session");
    let session_id = session.id.clone();

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // 1. Create a share with expiration
    let share = share_service
        .generate_share_link(&session_id, permissions, Some(Duration::seconds(1)))
        .unwrap();

    let share_id = share.id.clone();

    // 2. Verify share is accessible immediately
    let retrieved = share_service.get_share(&share_id).unwrap();
    assert_eq!(retrieved.id, share_id);

    // 3. Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 4. Verify share is expired
    let result = share_service.get_share(&share_id);
    assert!(result.is_err());
}

// ============================================================================
// Background Agent Execution Tests
// ============================================================================

#[tokio::test]
async fn test_background_agent_execution_and_monitoring() {
    let manager = BackgroundAgentManager::new();

    // 1. Create a background agent
    let agent = BackgroundAgent::new("analysis".to_string(), Some("test_agent".to_string()));
    let agent_id = agent.id.clone();

    // 2. Start the agent
    let started_id = manager.start_agent(agent).await.unwrap();
    assert_eq!(started_id, agent_id);

    // 3. Check initial status
    let status = manager.get_agent_status(&agent_id).await.unwrap();
    assert_eq!(status, ricecoder_sessions::AgentStatus::Running);

    // 4. Wait for agent to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 5. Check final status
    let final_status = manager.get_agent_status(&agent_id).await.unwrap();
    assert_eq!(final_status, ricecoder_sessions::AgentStatus::Completed);

    // 6. Verify completion event was emitted
    let events = manager.get_completion_events().await;
    assert!(!events.is_empty());
    assert_eq!(events[0].agent_id, agent_id);
    assert_eq!(events[0].status, ricecoder_sessions::AgentStatus::Completed);
}

#[tokio::test]
async fn test_background_agent_multiple_concurrent_execution() {
    let manager = BackgroundAgentManager::new();

    // 1. Start multiple agents concurrently
    let agent1 = BackgroundAgent::new("analysis".to_string(), Some("agent1".to_string()));
    let agent2 = BackgroundAgent::new("generation".to_string(), Some("agent2".to_string()));
    let agent3 = BackgroundAgent::new("validation".to_string(), Some("agent3".to_string()));

    let agent1_id = agent1.id.clone();
    let agent2_id = agent2.id.clone();
    let agent3_id = agent3.id.clone();

    manager.start_agent(agent1).await.unwrap();
    manager.start_agent(agent2).await.unwrap();
    manager.start_agent(agent3).await.unwrap();

    // 2. Verify all agents are running
    let status1 = manager.get_agent_status(&agent1_id).await.unwrap();
    let status2 = manager.get_agent_status(&agent2_id).await.unwrap();
    let status3 = manager.get_agent_status(&agent3_id).await.unwrap();

    assert_eq!(status1, ricecoder_sessions::AgentStatus::Running);
    assert_eq!(status2, ricecoder_sessions::AgentStatus::Running);
    assert_eq!(status3, ricecoder_sessions::AgentStatus::Running);

    // 3. Wait for all agents to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // 4. Verify all agents completed
    let final_status1 = manager.get_agent_status(&agent1_id).await.unwrap();
    let final_status2 = manager.get_agent_status(&agent2_id).await.unwrap();
    let final_status3 = manager.get_agent_status(&agent3_id).await.unwrap();

    assert_eq!(final_status1, ricecoder_sessions::AgentStatus::Completed);
    assert_eq!(final_status2, ricecoder_sessions::AgentStatus::Completed);
    assert_eq!(final_status3, ricecoder_sessions::AgentStatus::Completed);

    // 5. Verify completion events for all agents
    let events = manager.get_completion_events().await;
    assert_eq!(events.len(), 3);
}

#[tokio::test]
async fn test_background_agent_isolation() {
    let manager = BackgroundAgentManager::new();

    // 1. Start two agents
    let agent1 = BackgroundAgent::new("analysis".to_string(), Some("agent1".to_string()));
    let agent2 = BackgroundAgent::new("generation".to_string(), Some("agent2".to_string()));

    let agent1_id = agent1.id.clone();
    let agent2_id = agent2.id.clone();

    manager.start_agent(agent1).await.unwrap();
    manager.start_agent(agent2).await.unwrap();

    // 2. Verify both agents are running independently
    let status1 = manager.get_agent_status(&agent1_id).await.unwrap();
    let status2 = manager.get_agent_status(&agent2_id).await.unwrap();

    assert_eq!(status1, ricecoder_sessions::AgentStatus::Running);
    assert_eq!(status2, ricecoder_sessions::AgentStatus::Running);

    // 3. Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // 4. Verify both agents completed independently
    let final_status1 = manager.get_agent_status(&agent1_id).await.unwrap();
    let final_status2 = manager.get_agent_status(&agent2_id).await.unwrap();

    assert_eq!(final_status1, ricecoder_sessions::AgentStatus::Completed);
    assert_eq!(final_status2, ricecoder_sessions::AgentStatus::Completed);

    // 5. Verify each agent has its own completion event
    let events = manager.get_completion_events().await;
    let agent1_events: Vec<_> = events.iter().filter(|e| e.agent_id == agent1_id).collect();
    let agent2_events: Vec<_> = events.iter().filter(|e| e.agent_id == agent2_id).collect();

    assert_eq!(agent1_events.len(), 1);
    assert_eq!(agent2_events.len(), 1);
}

// ============================================================================
// Message Routing Tests
// ============================================================================

#[tokio::test]
async fn test_message_routing_to_active_session() {
    let mut router = SessionRouter::new();
    let context = create_test_context();

    // 1. Create two sessions
    let session1 = router
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();
    let session1_id = session1.id.clone();

    let session2 = router
        .create_session("Session 2".to_string(), context.clone())
        .unwrap();
    let session2_id = session2.id.clone();

    // 2. Route message to active session (should be session1)
    let routed_id = router
        .route_to_active_session("Hello from session 1")
        .unwrap();
    assert_eq!(routed_id, session1_id);

    // 3. Switch to session 2
    router.switch_session(&session2_id).unwrap();

    // 4. Route message to active session (should be session2)
    let routed_id = router
        .route_to_active_session("Hello from session 2")
        .unwrap();
    assert_eq!(routed_id, session2_id);

    // 5. Switch back to session 1
    router.switch_session(&session1_id).unwrap();

    // 6. Route message to active session (should be session1)
    let routed_id = router
        .route_to_active_session("Hello again from session 1")
        .unwrap();
    assert_eq!(routed_id, session1_id);
}

#[tokio::test]
async fn test_message_routing_prevents_cross_session_leakage() {
    let mut router = SessionRouter::new();
    let context = create_test_context();

    // 1. Create two sessions
    let session1 = router
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();
    let session1_id = session1.id.clone();

    let session2 = router
        .create_session("Session 2".to_string(), context.clone())
        .unwrap();
    let session2_id = session2.id.clone();

    // 2. Add messages to session 1
    router
        .route_to_active_session("Message 1 for session 1")
        .unwrap();
    router
        .route_to_active_session("Message 2 for session 1")
        .unwrap();

    // 3. Switch to session 2
    router.switch_session(&session2_id).unwrap();

    // 4. Add messages to session 2
    router
        .route_to_active_session("Message 1 for session 2")
        .unwrap();

    // 5. Get both sessions and verify message isolation
    let session1_data = router.get_session(&session1_id).unwrap();
    let session2_data = router.get_session(&session2_id).unwrap();

    // Session 1 should have 2 messages
    assert_eq!(session1_data.history.len(), 2);
    assert!(session1_data.history[0].content().contains("session 1"));
    assert!(session1_data.history[1].content().contains("session 1"));

    // Session 2 should have 1 message
    assert_eq!(session2_data.history.len(), 1);
    assert!(session2_data.history[0].content().contains("session 2"));
}

// ============================================================================
// Complex Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_complex_workflow_full_session_with_sharing_and_agents() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();
    let mut manager = SessionManager::new(10);
    let share_service = ShareService::new();
    let agent_manager = BackgroundAgentManager::new();

    // 1. Create a session
    let context = create_test_context();
    let mut session = manager
        .create_session("Complex Workflow Session".to_string(), context)
        .unwrap();
    let session_id = session.id.clone();

    // 2. Add some data to the session
    session.context.project_path = Some("/project".to_string());
    session.history.push(Message::new(
        MessageRole::User,
        "Analyze this code".to_string(),
    ));

    // 3. Persist the session
    store.save(&session).await.unwrap();

    // 4. Create a share for the session
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let share = share_service
        .generate_share_link(&session_id, permissions.clone(), None)
        .unwrap();

    // 5. Create a shared view
    let shared_view = share_service.create_shared_session_view(&session, &permissions);

    // 6. Import the shared session
    let imported_session = share_service
        .import_shared_session(&share.id, &shared_view)
        .unwrap();

    // 7. Start a background agent for the original session
    let agent = BackgroundAgent::new("analysis".to_string(), Some("analysis_agent".to_string()));
    let agent_id = agent.id.clone();

    agent_manager.start_agent(agent).await.unwrap();

    // 8. Wait for agent to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 9. Verify agent completed
    let agent_status = agent_manager.get_agent_status(&agent_id).await.unwrap();
    assert_eq!(agent_status, ricecoder_sessions::AgentStatus::Completed);

    // 10. Verify all components worked together
    assert!(store.exists(&session_id));
    assert_ne!(imported_session.id, session_id);
    assert_eq!(imported_session.name, session.name);
    assert_eq!(imported_session.history.len(), session.history.len());
}
