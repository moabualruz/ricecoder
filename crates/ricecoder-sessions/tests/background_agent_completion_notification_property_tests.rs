//! Property-based tests for background agent completion notification
//! **Feature: ricecoder-sessions, Property 13: Background Agent Completion Notification**
//! **Validates: Requirements 4.3**

use ricecoder_sessions::{BackgroundAgentManager, BackgroundAgent, AgentStatus};

/// Property: For any background agent that completes, the system SHALL emit a
/// notification event.
///
/// This property tests that:
/// 1. Completion events are emitted when agents complete
/// 2. Events contain the correct agent ID and status
/// 3. Events are retrievable from the manager
#[tokio::test]
async fn prop_background_agent_completion_notification() {
    for num_agents in 1..=5 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(
                format!("agent_{}", i),
                Some(format!("task_{}", i)),
            );
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Wait for all agents to complete
        for agent_id in &agent_ids {
            manager.wait_for_agent(agent_id).await.unwrap();
        }

        // Give a bit of time for events to be emitted
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Get completion events
        let events = manager.get_completion_events().await;

        // Should have events for all completed agents
        assert!(
            events.len() >= num_agents,
            "Should have completion events for all agents"
        );

        // All events should be for completed agents
        for event in &events {
            assert_eq!(
                event.status,
                AgentStatus::Completed,
                "Completion event should have Completed status"
            );
            assert!(
                agent_ids.contains(&event.agent_id),
                "Event should be for one of the started agents"
            );
            assert!(
                event.message.is_some(),
                "Completion event should have a message"
            );
        }
    }
}

/// Property: When an agent is cancelled, a cancellation notification event
/// SHALL be emitted.
#[tokio::test]
async fn prop_agent_cancellation_notification() {
    for num_agents in 1..=5 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(
                format!("agent_{}", i),
                None,
            );
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Cancel all agents
        for agent_id in &agent_ids {
            manager.cancel_agent(agent_id).await.unwrap();
        }

        // Get completion events
        let events = manager.get_completion_events().await;

        // Should have cancellation events
        assert!(
            events.len() >= num_agents,
            "Should have cancellation events for all cancelled agents"
        );

        // All events should be cancellations
        for event in &events {
            assert_eq!(
                event.status,
                AgentStatus::Cancelled,
                "Cancellation event should have Cancelled status"
            );
            assert!(
                agent_ids.contains(&event.agent_id),
                "Event should be for one of the started agents"
            );
        }
    }
}

/// Property: Completion events SHALL be clearable and subsequent completions
/// SHALL generate new events.
#[tokio::test]
async fn prop_completion_events_clearable() {
    for num_agents in 1..=3 {
        let manager = BackgroundAgentManager::new();

        // Start and complete first batch of agents
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(
                format!("agent_batch1_{}", i),
                None,
            );
            manager.start_agent(agent).await.unwrap();
        }

        // Wait for completion
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Get events
        let events_before = manager.get_completion_events().await;
        assert!(
            events_before.len() >= num_agents,
            "Should have events after first batch"
        );

        // Clear events
        manager.clear_completion_events().await;

        // Events should be cleared
        let events_after_clear = manager.get_completion_events().await;
        assert_eq!(
            events_after_clear.len(),
            0,
            "Events should be cleared"
        );

        // Start second batch
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(
                format!("agent_batch2_{}", i),
                None,
            );
            manager.start_agent(agent).await.unwrap();
        }

        // Wait for completion
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Should have new events
        let events_after_second_batch = manager.get_completion_events().await;
        assert!(
            events_after_second_batch.len() >= num_agents,
            "Should have new events after second batch"
        );
    }
}
