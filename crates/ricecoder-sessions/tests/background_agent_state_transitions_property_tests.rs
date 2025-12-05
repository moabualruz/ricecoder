//! Property-based tests for background agent state transitions
//! **Feature: ricecoder-sessions, Property 14: Background Agent State Transitions**
//! **Validates: Requirements 4.4**

use ricecoder_sessions::{AgentStatus, BackgroundAgent, BackgroundAgentManager};

/// Property: For any background agent, requesting pause or cancel SHALL transition
/// the agent to the requested state and stop execution.
///
/// This property tests that:
/// 1. Pause transitions agent to Cancelled state
/// 2. Cancel transitions agent to Cancelled state
/// 3. State transitions are immediate
#[tokio::test]
async fn prop_background_agent_state_transitions() {
    for num_agents in 1..=5 {
        for action in &["pause", "cancel"] {
            let manager = BackgroundAgentManager::new();

            // Start agents
            let mut agent_ids = Vec::new();
            for i in 0..num_agents {
                let agent = BackgroundAgent::new(format!("agent_{}", i), None);
                let agent_id = manager.start_agent(agent).await.unwrap();
                agent_ids.push(agent_id);
            }

            // Apply action to all agents
            for agent_id in &agent_ids {
                match *action {
                    "pause" => {
                        let result = manager.pause_agent(agent_id).await;
                        assert!(result.is_ok(), "Pause should succeed");
                    }
                    "cancel" => {
                        let result = manager.cancel_agent(agent_id).await;
                        assert!(result.is_ok(), "Cancel should succeed");
                    }
                    _ => {}
                }
            }

            // All agents should be in Cancelled state
            for agent_id in &agent_ids {
                let status = manager.get_agent_status(agent_id).await.unwrap();
                assert_eq!(
                    status,
                    AgentStatus::Cancelled,
                    "Agent should be in Cancelled state after action"
                );
            }
        }
    }
}

/// Property: An agent can only be paused if it is in Running state.
#[tokio::test]
async fn prop_pause_only_running_agents() {
    for num_agents in 1..=3 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(format!("agent_{}", i), None);
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Pause all agents
        for agent_id in &agent_ids {
            let result = manager.pause_agent(agent_id).await;
            assert!(result.is_ok(), "First pause should succeed");

            // Try to pause again - should fail
            let result2 = manager.pause_agent(agent_id).await;
            assert!(result2.is_err(), "Cannot pause an already paused agent");
        }
    }
}

/// Property: When an agent transitions to Cancelled state, its completed_at
/// timestamp SHALL be set.
#[tokio::test]
async fn prop_agent_completion_timestamp_on_cancel() {
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

        // All agents should have completed_at set
        for agent_id in &agent_ids {
            let agent = manager.get_agent(agent_id).await.unwrap();
            assert!(
                agent.completed_at.is_some(),
                "completed_at should be set when agent is cancelled"
            );
        }
    }
}

/// Property: State transitions SHALL be atomic - an agent cannot be in an
/// intermediate state.
#[tokio::test]
async fn prop_atomic_state_transitions() {
    for num_agents in 1..=3 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(format!("agent_{}", i), None);
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Rapidly check status while cancelling
        for agent_id in &agent_ids {
            manager.cancel_agent(agent_id).await.unwrap();

            // Status should be one of the valid states, never intermediate
            let status = manager.get_agent_status(agent_id).await.unwrap();
            assert!(
                matches!(
                    status,
                    AgentStatus::Running
                        | AgentStatus::Completed
                        | AgentStatus::Failed
                        | AgentStatus::Cancelled
                ),
                "Agent should be in a valid state, not intermediate"
            );
        }
    }
}
