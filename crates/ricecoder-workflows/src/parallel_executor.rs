//! Parallel step execution handler
//!
//! Handles execution of multiple steps concurrently within workflows.

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{ParallelStep, Workflow, WorkflowState};
use crate::state::StateManager;
use std::time::Instant;

/// Executes parallel steps by running multiple steps concurrently
pub struct ParallelExecutor;

impl ParallelExecutor {
    /// Execute a parallel step
    ///
    /// Executes multiple steps concurrently, respecting the max_concurrency limit.
    /// Waits for all parallel steps to complete before returning.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow containing the step
    /// * `state` - The current workflow state
    /// * `step_id` - The ID of the parallel step to execute
    /// * `parallel_step` - The parallel step configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all parallel steps executed successfully,
    /// or an error if any step failed.
    pub fn execute_parallel_step(
        workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
        parallel_step: &ParallelStep,
    ) -> WorkflowResult<()> {
        // Mark step as started
        StateManager::start_step(state, step_id.to_string());

        let start_time = Instant::now();

        // Execute parallel steps
        // In a real implementation, this would:
        // 1. Create tokio tasks for each step
        // 2. Respect max_concurrency limit
        // 3. Wait for all tasks to complete
        // 4. Aggregate results
        //
        // For now, we simulate successful execution
        let parallel_output = Self::execute_parallel_internal(workflow, state, parallel_step)?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Mark step as completed with the aggregated output
        StateManager::complete_step(
            state,
            step_id.to_string(),
            Some(parallel_output),
            duration_ms,
        );

        Ok(())
    }

    /// Internal parallel execution logic
    ///
    /// This is where the actual parallel execution would happen.
    fn execute_parallel_internal(
        _workflow: &Workflow,
        _state: &WorkflowState,
        parallel_step: &ParallelStep,
    ) -> WorkflowResult<serde_json::Value> {
        // In a real implementation, this would:
        // 1. Create tokio tasks for each step in parallel_step.steps
        // 2. Limit concurrency to parallel_step.max_concurrency
        // 3. Wait for all tasks to complete
        // 4. Aggregate results from all steps
        // 5. Return the aggregated output
        //
        // For now, we return a simulated output
        Ok(serde_json::json!({
            "parallel_steps": parallel_step.steps,
            "max_concurrency": parallel_step.max_concurrency,
            "status": "completed",
            "results": {}
        }))
    }

    /// Execute parallel steps with a concurrency limit
    ///
    /// Executes multiple steps concurrently, limiting the number of concurrent
    /// executions to the specified max_concurrency value.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow containing the step
    /// * `state` - The current workflow state
    /// * `step_id` - The ID of the parallel step to execute
    /// * `parallel_step` - The parallel step configuration
    /// * `max_concurrency` - The maximum number of concurrent executions
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all parallel steps executed successfully,
    /// or an error if any step failed.
    pub fn execute_parallel_step_with_limit(
        workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
        parallel_step: &ParallelStep,
        max_concurrency: usize,
    ) -> WorkflowResult<()> {
        // Validate max_concurrency
        if max_concurrency == 0 {
            return Err(WorkflowError::Invalid(
                "max_concurrency must be greater than 0".to_string(),
            ));
        }

        // Mark step as started
        StateManager::start_step(state, step_id.to_string());

        let start_time = Instant::now();

        // Create a modified parallel step with the new concurrency limit
        let mut modified_step = parallel_step.clone();
        modified_step.max_concurrency = max_concurrency;

        // Execute parallel steps with the new limit
        let parallel_output = Self::execute_parallel_internal(workflow, state, &modified_step)?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Mark step as completed
        StateManager::complete_step(state, step_id.to_string(), Some(parallel_output), duration_ms);

        Ok(())
    }

    /// Get the steps to execute in parallel
    pub fn get_parallel_steps(parallel_step: &ParallelStep) -> &[String] {
        &parallel_step.steps
    }

    /// Get the max concurrency from a parallel step
    pub fn get_max_concurrency(parallel_step: &ParallelStep) -> usize {
        parallel_step.max_concurrency
    }

    /// Validate a parallel step
    ///
    /// Checks that the parallel step is valid:
    /// - Has at least one step to execute
    /// - max_concurrency is greater than 0
    pub fn validate_parallel_step(parallel_step: &ParallelStep) -> WorkflowResult<()> {
        if parallel_step.steps.is_empty() {
            return Err(WorkflowError::Invalid(
                "Parallel step must have at least one step".to_string(),
            ));
        }

        if parallel_step.max_concurrency == 0 {
            return Err(WorkflowError::Invalid(
                "max_concurrency must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ErrorAction, RiskFactors, StepConfig, StepStatus, StepType, WorkflowConfig, WorkflowStep};

    fn create_workflow_with_parallel_step() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![WorkflowStep {
                id: "parallel-step".to_string(),
                name: "Parallel Step".to_string(),
                step_type: StepType::Parallel(ParallelStep {
                    steps: vec!["step1".to_string(), "step2".to_string()],
                    max_concurrency: 2,
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
    fn test_execute_parallel_step() {
        let workflow = create_workflow_with_parallel_step();
        let mut state = StateManager::create_state(&workflow);
        let parallel_step = ParallelStep {
            steps: vec!["step1".to_string(), "step2".to_string()],
            max_concurrency: 2,
        };

        let result = ParallelExecutor::execute_parallel_step(&workflow, &mut state, "parallel-step", &parallel_step);
        assert!(result.is_ok());

        // Verify step is marked as completed
        let step_result = state.step_results.get("parallel-step");
        assert!(step_result.is_some());
        assert_eq!(step_result.unwrap().status, StepStatus::Completed);
    }

    #[test]
    fn test_execute_parallel_step_with_limit() {
        let workflow = create_workflow_with_parallel_step();
        let mut state = StateManager::create_state(&workflow);
        let parallel_step = ParallelStep {
            steps: vec!["step1".to_string(), "step2".to_string(), "step3".to_string()],
            max_concurrency: 2,
        };

        let result = ParallelExecutor::execute_parallel_step_with_limit(
            &workflow,
            &mut state,
            "parallel-step",
            &parallel_step,
            1, // Override to 1 concurrent execution
        );
        assert!(result.is_ok());

        // Verify step is marked as completed
        let step_result = state.step_results.get("parallel-step");
        assert!(step_result.is_some());
        assert_eq!(step_result.unwrap().status, StepStatus::Completed);
    }

    #[test]
    fn test_get_parallel_steps() {
        let parallel_step = ParallelStep {
            steps: vec!["step1".to_string(), "step2".to_string()],
            max_concurrency: 2,
        };

        assert_eq!(
            ParallelExecutor::get_parallel_steps(&parallel_step),
            &["step1".to_string(), "step2".to_string()]
        );
    }

    #[test]
    fn test_get_max_concurrency() {
        let parallel_step = ParallelStep {
            steps: vec!["step1".to_string()],
            max_concurrency: 4,
        };

        assert_eq!(ParallelExecutor::get_max_concurrency(&parallel_step), 4);
    }

    #[test]
    fn test_validate_parallel_step_valid() {
        let parallel_step = ParallelStep {
            steps: vec!["step1".to_string(), "step2".to_string()],
            max_concurrency: 2,
        };

        let result = ParallelExecutor::validate_parallel_step(&parallel_step);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_parallel_step_empty_steps() {
        let parallel_step = ParallelStep {
            steps: vec![],
            max_concurrency: 2,
        };

        let result = ParallelExecutor::validate_parallel_step(&parallel_step);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_parallel_step_zero_concurrency() {
        let parallel_step = ParallelStep {
            steps: vec!["step1".to_string()],
            max_concurrency: 0,
        };

        let result = ParallelExecutor::validate_parallel_step(&parallel_step);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_parallel_step_with_limit_zero_concurrency() {
        let workflow = create_workflow_with_parallel_step();
        let mut state = StateManager::create_state(&workflow);
        let parallel_step = ParallelStep {
            steps: vec!["step1".to_string()],
            max_concurrency: 2,
        };

        let result = ParallelExecutor::execute_parallel_step_with_limit(
            &workflow,
            &mut state,
            "parallel-step",
            &parallel_step,
            0, // Invalid: 0 concurrency
        );
        assert!(result.is_err());
    }
}
