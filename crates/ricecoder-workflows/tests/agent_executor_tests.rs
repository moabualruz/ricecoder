use ricecoder_workflows::*;

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