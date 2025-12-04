//! Property-based tests for background agent async execution
//! **Feature: ricecoder-sessions, Property 11: Background Agent Async Execution**
//! **Validates: Requirements 4.1**

use ricecoder_sessions::{BackgroundAgentManager, BackgroundAgent, AgentStatus};

/// Property: For any background agent, starting the agent SHALL NOT block the active session
/// from processing messages.
///
/// This property tests that:
/// 1. Starting an agent returns immediately
/// 2. The agent runs asynchronously
/// 3. Multiple agents can run concurrently
#[tokio::test]
async fn prop_background_agent_async_execution() {
    for num_agents in 1..=10 {
        let manager = BackgroundAgentManager::new();
        let start_time = std::time::Instant::now();

        // Start multiple agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let agent = BackgroundAgent::new(
                format!("agent_{}", i),
                Some(format!("task_{}", i)),
            );
            let agent_id = manager.start_agent(agent).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Starting agents should be fast (not blocking)
        let elapsed = start_time.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "Starting {} agents took {:?}ms, should be fast",
            num_agents,
            elapsed.as_millis()
        );

        // All agents should be running
        for agent_id in &agent_ids {
            let status = manager.get_agent_status(agent_id).await.unwrap();
            assert_eq!(
                status,
                AgentStatus::Running,
                "Agent should be running immediately after start"
            );
        }

        // Wait for all agents to complete
        for agent_id in &agent_ids {
            let final_status = manager.wait_for_agent(agent_id).await.unwrap();
            assert_eq!(
                final_status,
                AgentStatus::Completed,
                "Agent should eventually complete"
            );
        }
    }
}

/// Property: Starting multiple background agents concurrently should not interfere
/// with each other's execution.
#[tokio::test]
async fn prop_concurrent_agent_execution() {
    for num_agents in 1..=5 {
        let manager = BackgroundAgentManager::new();

        // Start all agents concurrently
        let mut handles = Vec::new();
        for i in 0..num_agents {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                let agent = BackgroundAgent::new(
                    format!("agent_{}", i),
                    Some(format!("task_{}", i)),
                );
                mgr.start_agent(agent).await
            });
            handles.push(handle);
        }

        // Wait for all starts to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // All should succeed
        assert_eq!(results.len(), num_agents);
        for result in &results {
            assert!(result.is_ok(), "All agent starts should succeed");
        }

        // All agents should be running
        let agents = manager.list_agents().await;
        assert_eq!(agents.len(), num_agents, "All agents should be created");

        for agent in agents {
            assert_eq!(
                agent.status,
                AgentStatus::Running,
                "All agents should be running"
            );
        }
    }
}
