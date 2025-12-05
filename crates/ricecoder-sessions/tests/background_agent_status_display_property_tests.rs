//! Property-based tests for background agent status display
//! **Feature: ricecoder-sessions, Property 12: Background Agent Status Display**
//! **Validates: Requirements 4.2**

use ricecoder_sessions::{AgentStatus, BackgroundAgent, BackgroundAgentManager};

/// Property: For any running background agent, the system SHALL display its status
/// (Running, Completed, Failed, Cancelled) in the session UI.
///
/// This property tests that:
/// 1. Agent status is always retrievable
/// 2. Status reflects the actual agent state
/// 3. Status transitions are visible
#[tokio::test]
async fn prop_background_agent_status_display() {
    for num_agents in 1..=5 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(format!("agent_{}", i), Some(format!("task_{}", i)));
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // All agents should have retrievable status
        for agent_id in &agent_ids {
            let status = manager.get_agent_status(agent_id).await;
            assert!(
                status.is_ok(),
                "Status should be retrievable for agent {}",
                agent_id
            );
            assert_eq!(
                status.unwrap(),
                AgentStatus::Running,
                "Status should be Running immediately after start"
            );
        }

        // Wait for agents to complete
        for agent_id in &agent_ids {
            manager.wait_for_agent(agent_id).await.unwrap();
        }

        // All agents should show Completed status
        for agent_id in &agent_ids {
            let status = manager.get_agent_status(agent_id).await.unwrap();
            assert_eq!(
                status,
                AgentStatus::Completed,
                "Status should be Completed after agent finishes"
            );
        }
    }
}

/// Property: When an agent is cancelled, its status SHALL immediately reflect
/// the Cancelled state.
#[tokio::test]
async fn prop_agent_status_after_cancellation() {
    for num_agents in 1..=5 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(format!("agent_{}", i), None);
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Cancel all agents
        for agent_id in &agent_ids {
            manager.cancel_agent(agent_id).await.unwrap();
        }

        // All agents should show Cancelled status
        for agent_id in &agent_ids {
            let status = manager.get_agent_status(agent_id).await.unwrap();
            assert_eq!(
                status,
                AgentStatus::Cancelled,
                "Status should be Cancelled after cancellation"
            );
        }
    }
}

/// Property: The list of agents returned by list_agents() SHALL include all
/// agents with their current status.
#[tokio::test]
async fn prop_agent_list_completeness() {
    for num_agents in 1..=5 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(format!("agent_{}", i), Some(format!("task_{}", i)));
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Get list of agents
        let agents = manager.list_agents().await;

        // List should contain all agents
        assert_eq!(agents.len(), num_agents, "List should contain all agents");

        // All agents in list should have valid status
        for agent in agents {
            assert!(
                matches!(
                    agent.status,
                    AgentStatus::Running
                        | AgentStatus::Completed
                        | AgentStatus::Failed
                        | AgentStatus::Cancelled
                ),
                "Agent status should be one of the valid states"
            );
        }
    }
}
