//! Property-based tests for background agent isolation
//! **Feature: ricecoder-sessions, Property 15: Background Agent Isolation**
//! **Validates: Requirements 4.5**

use ricecoder_sessions::{AgentStatus, BackgroundAgent, BackgroundAgentManager};

/// Property: For any two background agents running concurrently, the execution
/// context of one agent SHALL NOT affect the execution context of the other agent.
///
/// This property tests that:
/// 1. Agents maintain independent state
/// 2. Cancelling one agent doesn't affect others
/// 3. Each agent's status is independent
#[tokio::test]
async fn prop_background_agent_isolation() {
    for num_agents in 2..=5 {
        let manager = BackgroundAgentManager::new();

        // Start multiple agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(format!("agent_{}", i), Some(format!("task_{}", i)));
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // All agents should be running independently
        for agent_id in &agent_ids {
            let status = manager.get_agent_status(agent_id).await.unwrap();
            assert_eq!(
                status,
                AgentStatus::Running,
                "All agents should be running independently"
            );
        }

        // Cancel the first agent
        if !agent_ids.is_empty() {
            manager.cancel_agent(&agent_ids[0]).await.unwrap();

            // First agent should be cancelled
            let status = manager.get_agent_status(&agent_ids[0]).await.unwrap();
            assert_eq!(
                status,
                AgentStatus::Cancelled,
                "First agent should be cancelled"
            );

            // Other agents should still be running
            for i in 1..agent_ids.len() {
                let status = manager.get_agent_status(&agent_ids[i]).await.unwrap();
                assert_eq!(
                    status,
                    AgentStatus::Running,
                    "Other agents should not be affected by cancellation of first agent"
                );
            }
        }
    }
}

/// Property: Each agent SHALL maintain its own task and metadata independently.
#[tokio::test]
async fn prop_agent_metadata_isolation() {
    for num_agents in 2..=5 {
        let manager = BackgroundAgentManager::new();

        // Start agents with different tasks
        let mut agent_metadata = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(
                format!("agent_type_{}", i),
                Some(format!("unique_task_{}", i)),
            );
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_metadata.push((
                agent_id,
                format!("agent_type_{}", i),
                format!("unique_task_{}", i),
            ));
        }

        // Each agent should have its own metadata
        let agents = manager.list_agents().await;
        assert_eq!(agents.len(), num_agents, "All agents should be present");

        // Verify each agent has unique metadata
        for (expected_id, expected_type, expected_task) in agent_metadata {
            let agent = agents
                .iter()
                .find(|a| a.id == expected_id)
                .expect("Agent should exist");
            assert_eq!(agent.agent_type, expected_type, "Agent type should match");
            assert_eq!(agent.task, Some(expected_task), "Agent task should match");
        }
    }
}

/// Property: Cancelling one agent SHALL NOT affect the completion events of
/// other agents.
#[tokio::test]
async fn prop_agent_events_isolation() {
    for num_agents in 2..=3 {
        let manager = BackgroundAgentManager::new();

        // Start agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(format!("agent_{}", i), None);
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Cancel first agent
        if !agent_ids.is_empty() {
            manager.cancel_agent(&agent_ids[0]).await.unwrap();
        }

        // Wait for other agents to complete
        for i in 1..agent_ids.len() {
            manager.wait_for_agent(&agent_ids[i]).await.unwrap();
        }

        // Give time for events
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Get events
        let events = manager.get_completion_events().await;

        // Should have events for all agents
        assert!(
            events.len() >= num_agents,
            "Should have events for all agents"
        );

        // Each agent should have its own event
        for agent_id in &agent_ids {
            let agent_events: Vec<_> = events.iter().filter(|e| e.agent_id == *agent_id).collect();
            assert!(
                !agent_events.is_empty(),
                "Agent {} should have its own event",
                agent_id
            );
        }
    }
}

/// Property: Multiple managers SHALL maintain independent agent contexts.
#[tokio::test]
async fn prop_manager_isolation() {
    for num_managers in 2..=3 {
        for agents_per_manager in 1..=3 {
            let mut managers = Vec::new();
            let mut all_agent_ids = Vec::new();

            // Create multiple managers with agents
            for m in 0..num_managers {
                let manager = BackgroundAgentManager::new();
                let mut manager_agent_ids = Vec::new();

                for a in 0..agents_per_manager {
                    let agent = BackgroundAgent::new(format!("manager_{}_agent_{}", m, a), None);
                    let agent_id = manager.start_agent(agent).await.unwrap();
                    manager_agent_ids.push(agent_id);
                }

                all_agent_ids.push(manager_agent_ids);
                managers.push(manager);
            }

            // Each manager should have its own agents
            for (m, manager) in managers.iter().enumerate() {
                let agents = manager.list_agents().await;
                assert_eq!(
                    agents.len(),
                    agents_per_manager,
                    "Manager {} should have its own agents",
                    m
                );
            }

            // Cancelling in one manager shouldn't affect others
            if !managers.is_empty() && !all_agent_ids[0].is_empty() {
                managers[0]
                    .cancel_agent(&all_agent_ids[0][0])
                    .await
                    .unwrap();

                // First manager's agent should be cancelled
                let status = managers[0]
                    .get_agent_status(&all_agent_ids[0][0])
                    .await
                    .unwrap();
                assert_eq!(status, AgentStatus::Cancelled);

                // Other managers' agents should be unaffected
                for m in 1..managers.len() {
                    for agent_id in &all_agent_ids[m] {
                        let status = managers[m].get_agent_status(agent_id).await.unwrap();
                        assert_eq!(
                            status,
                            AgentStatus::Running,
                            "Manager {} agents should not be affected",
                            m
                        );
                    }
                }
            }
        }
    }
}
