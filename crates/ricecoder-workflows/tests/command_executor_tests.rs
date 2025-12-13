use ricecoder_workflows::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ErrorAction, RiskFactors, StepConfig, StepStatus, StepType, WorkflowConfig, WorkflowStep,
    };

    fn create_workflow_with_command_step() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![WorkflowStep {
                id: "command-step".to_string(),
                name: "Command Step".to_string(),
                step_type: StepType::Command(CommandStep {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                    timeout: 5000,
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
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
    fn test_execute_command_step() {
        let workflow = create_workflow_with_command_step();
        let mut state = StateManager::create_state(&workflow);
        let command_step = CommandStep {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            timeout: 5000,
        };

        let result = CommandExecutor::execute_command_step(
            &workflow,
            &mut state,
            "command-step",
            &command_step,
        );
        assert!(result.is_ok());

        // Verify step is marked as completed
        let step_result = state.step_results.get("command-step");
        assert!(step_result.is_some());
        assert_eq!(step_result.unwrap().status, StepStatus::Completed);
    }

    #[test]
    fn test_execute_command_step_with_timeout() {
        let workflow = create_workflow_with_command_step();
        let mut state = StateManager::create_state(&workflow);
        let command_step = CommandStep {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            timeout: 5000,
        };

        let result = CommandExecutor::execute_command_step_with_timeout(
            &workflow,
            &mut state,
            "command-step",
            &command_step,
            10000, // 10 second timeout
        );
        assert!(result.is_ok());

        // Verify step is marked as completed
        let step_result = state.step_results.get("command-step");
        assert!(step_result.is_some());
        assert_eq!(step_result.unwrap().status, StepStatus::Completed);
    }

    #[test]
    fn test_get_command() {
        let command_step = CommandStep {
            command: "ls".to_string(),
            args: vec!["-la".to_string()],
            timeout: 5000,
        };

        assert_eq!(CommandExecutor::get_command(&command_step), "ls");
    }

    #[test]
    fn test_get_args() {
        let command_step = CommandStep {
            command: "ls".to_string(),
            args: vec!["-la".to_string(), "-h".to_string()],
            timeout: 5000,
        };

        assert_eq!(
            CommandExecutor::get_args(&command_step),
            &["-la".to_string(), "-h".to_string()]
        );
    }

    #[test]
    fn test_get_timeout() {
        let command_step = CommandStep {
            command: "ls".to_string(),
            args: vec![],
            timeout: 3000,
        };

        assert_eq!(CommandExecutor::get_timeout(&command_step), 3000);
    }
}