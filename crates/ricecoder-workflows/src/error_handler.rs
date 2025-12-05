//! Error handling and recovery for workflow steps

use crate::error::{WorkflowError, WorkflowResult};
#[allow(unused_imports)]
use crate::models::{ErrorAction, RiskFactors, StepStatus, Workflow, WorkflowState};
use crate::state::StateManager;
use std::time::Duration;

/// Handles step execution errors and applies error actions
///
/// Responsible for:
/// - Catching step execution errors
/// - Applying error actions (retry, skip, fail, rollback)
/// - Tracking error history
/// - Managing retry logic with exponential backoff
pub struct ErrorHandler;

/// Error history entry for tracking errors across retries
#[derive(Debug, Clone)]
pub struct ErrorHistoryEntry {
    /// Error message
    pub error: String,
    /// Attempt number (1-indexed)
    pub attempt: usize,
    /// Timestamp of error
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Retry state for tracking retry attempts
#[derive(Debug, Clone)]
pub struct RetryState {
    /// Current attempt number (1-indexed)
    pub attempt: usize,
    /// Maximum attempts allowed
    pub max_attempts: usize,
    /// Delay between retries in milliseconds
    pub delay_ms: u64,
    /// Error history
    pub history: Vec<ErrorHistoryEntry>,
}

impl RetryState {
    /// Create a new retry state
    pub fn new(max_attempts: usize, delay_ms: u64) -> Self {
        Self {
            attempt: 1,
            max_attempts,
            delay_ms,
            history: Vec::new(),
        }
    }

    /// Check if more retries are available
    pub fn can_retry(&self) -> bool {
        self.attempt < self.max_attempts
    }

    /// Get the delay for the next retry with exponential backoff
    ///
    /// Calculates: delay_ms * 2^(attempt - 1)
    pub fn get_backoff_delay(&self) -> Duration {
        let backoff_factor = 2_u64.pow((self.attempt - 1) as u32);
        Duration::from_millis(self.delay_ms * backoff_factor)
    }

    /// Record an error and increment attempt counter
    pub fn record_error(&mut self, error: String) {
        self.history.push(ErrorHistoryEntry {
            error,
            attempt: self.attempt,
            timestamp: chrono::Utc::now(),
        });
        self.attempt += 1;
    }

    /// Get the error history
    pub fn get_history(&self) -> &[ErrorHistoryEntry] {
        &self.history
    }
}

impl ErrorHandler {
    /// Handle a step execution error
    ///
    /// Applies the specified error action and updates the workflow state accordingly.
    /// Returns true if the workflow should continue, false if it should stop.
    pub fn handle_error(
        workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
        error: String,
    ) -> WorkflowResult<bool> {
        // Find the step
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Apply the error action
        match &step.on_error {
            ErrorAction::Fail => {
                // Mark step as failed and stop workflow
                StateManager::fail_step(state, step_id.to_string(), error, 0);
                Ok(false)
            }
            ErrorAction::Retry { .. } => {
                // Retry logic is handled by the caller
                // This just records the error for retry tracking
                StateManager::fail_step(state, step_id.to_string(), error, 0);
                Ok(true) // Signal that retry should be attempted
            }
            ErrorAction::Skip => {
                // Skip the step and continue
                StateManager::skip_step(state, step_id.to_string());
                Ok(true)
            }
            ErrorAction::Rollback => {
                // Rollback is handled separately
                StateManager::fail_step(state, step_id.to_string(), error, 0);
                Ok(false)
            }
        }
    }

    /// Check if a step should be retried
    pub fn should_retry(workflow: &Workflow, step_id: &str) -> bool {
        workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .map(|step| matches!(step.on_error, ErrorAction::Retry { .. }))
            .unwrap_or(false)
    }

    /// Get retry configuration for a step
    pub fn get_retry_config(workflow: &Workflow, step_id: &str) -> Option<(usize, u64)> {
        workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .and_then(|step| match &step.on_error {
                ErrorAction::Retry {
                    max_attempts,
                    delay_ms,
                } => Some((*max_attempts, *delay_ms)),
                _ => None,
            })
    }

    /// Check if a step should be skipped on error
    pub fn should_skip_on_error(workflow: &Workflow, step_id: &str) -> bool {
        workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .map(|step| matches!(step.on_error, ErrorAction::Skip))
            .unwrap_or(false)
    }

    /// Check if a step should rollback on error
    pub fn should_rollback_on_error(workflow: &Workflow, step_id: &str) -> bool {
        workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .map(|step| matches!(step.on_error, ErrorAction::Rollback))
            .unwrap_or(false)
    }

    /// Get the error action for a step
    pub fn get_error_action(workflow: &Workflow, step_id: &str) -> Option<ErrorAction> {
        workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .map(|step| step.on_error.clone())
    }

    /// Capture error details in step result
    ///
    /// Stores error type, message, and stack trace in the step result.
    pub fn capture_error(
        state: &mut WorkflowState,
        step_id: &str,
        error_type: &str,
        error_message: &str,
        stack_trace: Option<&str>,
    ) -> WorkflowResult<()> {
        if let Some(result) = state.step_results.get_mut(step_id) {
            result.error = Some(format!(
                "Type: {}\nMessage: {}\n{}",
                error_type,
                error_message,
                stack_trace.unwrap_or("")
            ));
            result.status = StepStatus::Failed;
        }
        Ok(())
    }

    /// Get error details from a step result
    pub fn get_error_details(state: &WorkflowState, step_id: &str) -> Option<String> {
        state
            .step_results
            .get(step_id)
            .and_then(|result| result.error.clone())
    }

    /// Check if a step has an error
    pub fn has_error(state: &WorkflowState, step_id: &str) -> bool {
        state
            .step_results
            .get(step_id)
            .map(|result| result.error.is_some())
            .unwrap_or(false)
    }

    /// Get all errors in the workflow
    pub fn get_all_errors(state: &WorkflowState) -> Vec<(String, String)> {
        state
            .step_results
            .iter()
            .filter_map(|(step_id, result)| {
                result
                    .error
                    .as_ref()
                    .map(|error| (step_id.clone(), error.clone()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentStep, StepConfig, StepType, WorkflowConfig, WorkflowStep};

    fn create_workflow_with_error_action(error_action: ErrorAction) -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],steps: vec![WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "test-task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({"param": "value"}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: error_action,
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
    fn test_retry_state_creation() {
        let retry_state = RetryState::new(3, 100);
        assert_eq!(retry_state.attempt, 1);
        assert_eq!(retry_state.max_attempts, 3);
        assert_eq!(retry_state.delay_ms, 100);
        assert!(retry_state.can_retry());
    }

    #[test]
    fn test_retry_state_exponential_backoff() {
        let mut retry_state = RetryState::new(3, 100);

        // First retry: 100ms * 2^0 = 100ms
        let delay1 = retry_state.get_backoff_delay();
        assert_eq!(delay1.as_millis(), 100);

        retry_state.record_error("Error 1".to_string());

        // Second retry: 100ms * 2^1 = 200ms
        let delay2 = retry_state.get_backoff_delay();
        assert_eq!(delay2.as_millis(), 200);

        retry_state.record_error("Error 2".to_string());

        // Third retry: 100ms * 2^2 = 400ms
        let delay3 = retry_state.get_backoff_delay();
        assert_eq!(delay3.as_millis(), 400);

        retry_state.record_error("Error 3".to_string());

        // No more retries
        assert!(!retry_state.can_retry());
    }

    #[test]
    fn test_retry_state_error_history() {
        let mut retry_state = RetryState::new(3, 100);

        retry_state.record_error("Error 1".to_string());
        retry_state.record_error("Error 2".to_string());

        let history = retry_state.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].error, "Error 1");
        assert_eq!(history[0].attempt, 1);
        assert_eq!(history[1].error, "Error 2");
        assert_eq!(history[1].attempt, 2);
    }

    #[test]
    fn test_should_retry() {
        let workflow = create_workflow_with_error_action(ErrorAction::Retry {
            max_attempts: 3,
            delay_ms: 100,
        });

        assert!(ErrorHandler::should_retry(&workflow, "step1"));
    }

    #[test]
    fn test_should_not_retry_on_fail() {
        let workflow = create_workflow_with_error_action(ErrorAction::Fail);

        assert!(!ErrorHandler::should_retry(&workflow, "step1"));
    }

    #[test]
    fn test_should_skip_on_error() {
        let workflow = create_workflow_with_error_action(ErrorAction::Skip);

        assert!(ErrorHandler::should_skip_on_error(&workflow, "step1"));
    }

    #[test]
    fn test_should_rollback_on_error() {
        let workflow = create_workflow_with_error_action(ErrorAction::Rollback);

        assert!(ErrorHandler::should_rollback_on_error(&workflow, "step1"));
    }

    #[test]
    fn test_get_retry_config() {
        let workflow = create_workflow_with_error_action(ErrorAction::Retry {
            max_attempts: 3,
            delay_ms: 100,
        });

        let config = ErrorHandler::get_retry_config(&workflow, "step1");
        assert_eq!(config, Some((3, 100)));
    }

    #[test]
    fn test_get_error_action() {
        let workflow = create_workflow_with_error_action(ErrorAction::Skip);

        let action = ErrorHandler::get_error_action(&workflow, "step1");
        assert!(matches!(action, Some(ErrorAction::Skip)));
    }

    #[test]
    fn test_capture_error() {
        let workflow = create_workflow_with_error_action(ErrorAction::Fail);
        let mut state = StateManager::create_state(&workflow);

        StateManager::start_step(&mut state, "step1".to_string());

        let result = ErrorHandler::capture_error(
            &mut state,
            "step1",
            "RuntimeError",
            "Something went wrong",
            Some("at line 42"),
        );

        assert!(result.is_ok());
        assert!(ErrorHandler::has_error(&state, "step1"));

        let error = ErrorHandler::get_error_details(&state, "step1");
        assert!(error.is_some());
        let error_str = error.unwrap();
        assert!(error_str.contains("RuntimeError"));
        assert!(error_str.contains("Something went wrong"));
        assert!(error_str.contains("at line 42"));
    }

    #[test]
    fn test_get_all_errors() {
        let workflow = Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],steps: vec![
                WorkflowStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec![],
                    approval_required: false,
                    on_error: ErrorAction::Fail, risk_score: None, risk_factors: RiskFactors::default(),
                },
                WorkflowStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec![],
                    approval_required: false,
                    on_error: ErrorAction::Fail, risk_score: None, risk_factors: RiskFactors::default(),
                },
            ],
            config: WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        };

        let mut state = StateManager::create_state(&workflow);

        StateManager::start_step(&mut state, "step1".to_string());
        StateManager::start_step(&mut state, "step2".to_string());

        ErrorHandler::capture_error(&mut state, "step1", "Error1", "Message1", None).ok();
        ErrorHandler::capture_error(&mut state, "step2", "Error2", "Message2", None).ok();

        let errors = ErrorHandler::get_all_errors(&state);
        assert_eq!(errors.len(), 2);
    }
}



