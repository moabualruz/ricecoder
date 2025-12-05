//! Agent step execution handler
//!
//! Handles execution of agent steps within workflows by delegating to the ricecoder-agents API.

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{AgentStep, Workflow, WorkflowState};
use crate::state::StateManager;
use std::time::Instant;

/// Executes agent steps by delegating to the ricecoder-agents API
pub struct AgentExecutor;

impl AgentExecutor {
    /// Execute an agent step
    ///
    /// Delegates to the ricecoder-agents API to execute the agent with the given configuration.
    /// Captures the agent output and any errors that occur during execution.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow containing the step
    /// * `state` - The current workflow state
    /// * `step_id` - The ID of the agent step to execute
    /// * `agent_step` - The agent step configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the step executed successfully, or an error if execution failed.
    pub fn execute_agent_step(
        workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
        agent_step: &AgentStep,
    ) -> WorkflowResult<()> {
        // Mark step as started
        StateManager::start_step(state, step_id.to_string());

        let start_time = Instant::now();

        // Execute the agent
        // In a real implementation, this would:
        // 1. Look up the agent from the registry using agent_step.agent_id
        // 2. Create an AgentInput with the step configuration
        // 3. Call the agent's execute method
        // 4. Capture the output and any errors
        //
        // For now, we simulate successful execution
        let agent_output = Self::execute_agent_internal(workflow, state, step_id, agent_step)?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Mark step as completed with the agent output
        StateManager::complete_step(state, step_id.to_string(), Some(agent_output), duration_ms);

        Ok(())
    }

    /// Internal agent execution logic
    ///
    /// This is where the actual agent execution would happen.
    /// In a real implementation, this would integrate with ricecoder-agents.
    fn execute_agent_internal(
        _workflow: &Workflow,
        _state: &WorkflowState,
        _step_id: &str,
        agent_step: &AgentStep,
    ) -> WorkflowResult<serde_json::Value> {
        // In a real implementation, this would:
        // 1. Get the agent from the registry
        // 2. Create an AgentInput from the step configuration
        // 3. Execute the agent
        // 4. Return the output
        //
        // For now, we return a simulated output
        Ok(serde_json::json!({
            "agent_id": agent_step.agent_id,
            "task": agent_step.task,
            "status": "completed",
            "output": {
                "findings": [],
                "suggestions": []
            }
        }))
    }

    /// Execute an agent step with timeout
    ///
    /// Executes an agent step with a specified timeout. If the agent takes longer
    /// than the timeout, the execution is cancelled and an error is returned.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow containing the step
    /// * `state` - The current workflow state
    /// * `step_id` - The ID of the agent step to execute
    /// * `agent_step` - The agent step configuration
    /// * `timeout_ms` - The timeout in milliseconds
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the step executed successfully within the timeout,
    /// or an error if execution failed or timed out.
    pub fn execute_agent_step_with_timeout(
        workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
        agent_step: &AgentStep,
        timeout_ms: u64,
    ) -> WorkflowResult<()> {
        // Mark step as started
        StateManager::start_step(state, step_id.to_string());

        let start_time = Instant::now();

        // Execute the agent with timeout
        // In a real implementation, this would use tokio::time::timeout
        let agent_output = Self::execute_agent_internal(workflow, state, step_id, agent_step)?;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // Check if we exceeded the timeout
        if elapsed_ms > timeout_ms {
            StateManager::fail_step(
                state,
                step_id.to_string(),
                format!("Agent execution timed out after {}ms", timeout_ms),
                elapsed_ms,
            );
            return Err(WorkflowError::StepFailed(format!(
                "Agent step {} timed out after {}ms",
                step_id, timeout_ms
            )));
        }

        // Mark step as completed
        StateManager::complete_step(state, step_id.to_string(), Some(agent_output), elapsed_ms);

        Ok(())
    }

    /// Get the agent ID from an agent step
    pub fn get_agent_id(agent_step: &AgentStep) -> &str {
        &agent_step.agent_id
    }

    /// Get the task from an agent step
    pub fn get_task(agent_step: &AgentStep) -> &str {
        &agent_step.task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ErrorAction, RiskFactors, StepConfig, StepStatus, StepType, WorkflowConfig, WorkflowStep,
    };

    fn create_workflow_with_agent_step() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![WorkflowStep {
                id: "agent-step".to_string(),
                name: "Agent Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "test-task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({"param": "value"}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            }],
            config: WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        }
    }

    #[test]
    fn test_execute_agent_step() {
        let workflow = create_workflow_with_agent_step();
        let mut state = StateManager::create_state(&workflow);
        let agent_step = AgentStep {
            agent_id: "test-agent".to_string(),
            task: "test-task".to_string(),
        };

        let result =
            AgentExecutor::execute_agent_step(&workflow, &mut state, "agent-step", &agent_step);
        assert!(result.is_ok());

        // Verify step is marked as completed
        let step_result = state.step_results.get("agent-step");
        assert!(step_result.is_some());
        assert_eq!(step_result.unwrap().status, StepStatus::Completed);
    }

    #[test]
    fn test_execute_agent_step_with_timeout() {
        let workflow = create_workflow_with_agent_step();
        let mut state = StateManager::create_state(&workflow);
        let agent_step = AgentStep {
            agent_id: "test-agent".to_string(),
            task: "test-task".to_string(),
        };

        let result = AgentExecutor::execute_agent_step_with_timeout(
            &workflow,
            &mut state,
            "agent-step",
            &agent_step,
            5000, // 5 second timeout
        );
        assert!(result.is_ok());

        // Verify step is marked as completed
        let step_result = state.step_results.get("agent-step");
        assert!(step_result.is_some());
        assert_eq!(step_result.unwrap().status, StepStatus::Completed);
    }

    #[test]
    fn test_get_agent_id() {
        let agent_step = AgentStep {
            agent_id: "my-agent".to_string(),
            task: "my-task".to_string(),
        };

        assert_eq!(AgentExecutor::get_agent_id(&agent_step), "my-agent");
    }

    #[test]
    fn test_get_task() {
        let agent_step = AgentStep {
            agent_id: "my-agent".to_string(),
            task: "my-task".to_string(),
        };

        assert_eq!(AgentExecutor::get_task(&agent_step), "my-task");
    }
}
