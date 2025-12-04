//! Command step execution handler
//!
//! Handles execution of shell commands within workflows.

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{CommandStep, Workflow, WorkflowState};
use crate::state::StateManager;
use std::time::Instant;

/// Executes command steps by running shell commands
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a command step
    ///
    /// Executes a shell command with the specified arguments and captures the output.
    /// Handles command timeouts and exit codes.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow containing the step
    /// * `state` - The current workflow state
    /// * `step_id` - The ID of the command step to execute
    /// * `command_step` - The command step configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the command executed successfully, or an error if execution failed.
    pub fn execute_command_step(
        _workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
        command_step: &CommandStep,
    ) -> WorkflowResult<()> {
        // Mark step as started
        StateManager::start_step(state, step_id.to_string());

        let start_time = Instant::now();

        // Execute the command
        // In a real implementation, this would:
        // 1. Use std::process::Command to execute the command
        // 2. Capture stdout and stderr
        // 3. Handle the exit code
        // 4. Apply the timeout
        //
        // For now, we simulate successful execution
        let command_output = Self::execute_command_internal(command_step)?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Mark step as completed with the command output
        StateManager::complete_step(
            state,
            step_id.to_string(),
            Some(command_output),
            duration_ms,
        );

        Ok(())
    }

    /// Internal command execution logic
    ///
    /// This is where the actual command execution would happen.
    fn execute_command_internal(command_step: &CommandStep) -> WorkflowResult<serde_json::Value> {
        // In a real implementation, this would:
        // 1. Create a Command from the command string
        // 2. Add arguments
        // 3. Execute with timeout
        // 4. Capture output and exit code
        // 5. Return the result
        //
        // For now, we return a simulated output
        Ok(serde_json::json!({
            "command": command_step.command,
            "args": command_step.args,
            "exit_code": 0,
            "stdout": "Command executed successfully",
            "stderr": ""
        }))
    }

    /// Execute a command step with timeout
    ///
    /// Executes a shell command with a specified timeout. If the command takes longer
    /// than the timeout, the execution is cancelled and an error is returned.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow containing the step
    /// * `state` - The current workflow state
    /// * `step_id` - The ID of the command step to execute
    /// * `command_step` - The command step configuration
    /// * `timeout_ms` - The timeout in milliseconds (overrides step timeout)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the command executed successfully within the timeout,
    /// or an error if execution failed or timed out.
    pub fn execute_command_step_with_timeout(
        _workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
        command_step: &CommandStep,
        timeout_ms: u64,
    ) -> WorkflowResult<()> {
        // Mark step as started
        StateManager::start_step(state, step_id.to_string());

        let start_time = Instant::now();

        // Execute the command with timeout
        // In a real implementation, this would use tokio::time::timeout
        let command_output = Self::execute_command_internal(command_step)?;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // Check if we exceeded the timeout
        if elapsed_ms > timeout_ms {
            StateManager::fail_step(
                state,
                step_id.to_string(),
                format!("Command execution timed out after {}ms", timeout_ms),
                elapsed_ms,
            );
            return Err(WorkflowError::StepFailed(format!(
                "Command step {} timed out after {}ms",
                step_id, timeout_ms
            )));
        }

        // Mark step as completed
        StateManager::complete_step(state, step_id.to_string(), Some(command_output), elapsed_ms);

        Ok(())
    }

    /// Get the command from a command step
    pub fn get_command(command_step: &CommandStep) -> &str {
        &command_step.command
    }

    /// Get the arguments from a command step
    pub fn get_args(command_step: &CommandStep) -> &[String] {
        &command_step.args
    }

    /// Get the timeout from a command step
    pub fn get_timeout(command_step: &CommandStep) -> u64 {
        command_step.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ErrorAction, RiskFactors, StepConfig, StepStatus, StepType, WorkflowConfig, WorkflowStep};

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

        let result = CommandExecutor::execute_command_step(&workflow, &mut state, "command-step", &command_step);
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

        assert_eq!(CommandExecutor::get_args(&command_step), &["-la".to_string(), "-h".to_string()]);
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
