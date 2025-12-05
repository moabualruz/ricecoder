//! Workflow state management

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{StepResult, StepStatus, Workflow, WorkflowState, WorkflowStatus};
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;

/// Manages workflow execution state
pub struct StateManager;

impl StateManager {
    /// Create a new workflow state
    pub fn create_state(workflow: &Workflow) -> WorkflowState {
        WorkflowState {
            workflow_id: workflow.id.clone(),
            status: WorkflowStatus::Pending,
            current_step: None,
            completed_steps: Vec::new(),
            step_results: HashMap::new(),
            started_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update workflow state to running
    pub fn start_workflow(state: &mut WorkflowState) {
        state.status = WorkflowStatus::Running;
        state.started_at = Utc::now();
        state.updated_at = Utc::now();
    }

    /// Mark a step as started
    pub fn start_step(state: &mut WorkflowState, step_id: String) {
        state.current_step = Some(step_id.clone());
        state.step_results.insert(
            step_id,
            StepResult {
                status: StepStatus::Running,
                output: None,
                error: None,
                duration_ms: 0,
            },
        );
        state.updated_at = Utc::now();
    }

    /// Mark a step as completed
    pub fn complete_step(
        state: &mut WorkflowState,
        step_id: String,
        output: Option<serde_json::Value>,
        duration_ms: u64,
    ) {
        if let Some(result) = state.step_results.get_mut(&step_id) {
            result.status = StepStatus::Completed;
            result.output = output;
            result.duration_ms = duration_ms;
        }

        state.completed_steps.push(step_id);
        state.updated_at = Utc::now();
    }

    /// Mark a step as failed
    pub fn fail_step(state: &mut WorkflowState, step_id: String, error: String, duration_ms: u64) {
        if let Some(result) = state.step_results.get_mut(&step_id) {
            result.status = StepStatus::Failed;
            result.error = Some(error);
            result.duration_ms = duration_ms;
        }

        state.updated_at = Utc::now();
    }

    /// Mark a step as skipped
    pub fn skip_step(state: &mut WorkflowState, step_id: String) {
        if let Some(result) = state.step_results.get_mut(&step_id) {
            result.status = StepStatus::Skipped;
        }

        state.completed_steps.push(step_id);
        state.updated_at = Utc::now();
    }

    /// Mark workflow as waiting for approval
    pub fn wait_for_approval(state: &mut WorkflowState) {
        state.status = WorkflowStatus::WaitingApproval;
        state.updated_at = Utc::now();
    }

    /// Mark workflow as completed
    pub fn complete_workflow(state: &mut WorkflowState) {
        state.status = WorkflowStatus::Completed;
        state.current_step = None;
        state.updated_at = Utc::now();
    }

    /// Mark workflow as failed
    pub fn fail_workflow(state: &mut WorkflowState) {
        state.status = WorkflowStatus::Failed;
        state.updated_at = Utc::now();
    }

    /// Mark workflow as cancelled
    pub fn cancel_workflow(state: &mut WorkflowState) {
        state.status = WorkflowStatus::Cancelled;
        state.updated_at = Utc::now();
    }

    /// Pause workflow execution at current step
    pub fn pause_workflow(state: &mut WorkflowState) -> WorkflowResult<()> {
        // Can only pause if running or waiting for approval
        if state.status != WorkflowStatus::Running
            && state.status != WorkflowStatus::WaitingApproval
        {
            return Err(WorkflowError::StateError(format!(
                "Cannot pause workflow in {:?} status",
                state.status
            )));
        }

        state.status = WorkflowStatus::Paused;
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Resume workflow execution from paused step
    pub fn resume_workflow(state: &mut WorkflowState) -> WorkflowResult<()> {
        // Can only resume if paused
        if state.status != WorkflowStatus::Paused {
            return Err(WorkflowError::StateError(format!(
                "Cannot resume workflow in {:?} status",
                state.status
            )));
        }

        state.status = WorkflowStatus::Running;
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Check if a step has already been completed
    pub fn is_step_completed(state: &WorkflowState, step_id: &str) -> bool {
        state.completed_steps.contains(&step_id.to_string())
    }

    /// Get the next step to execute (skipping completed steps)
    pub fn get_next_step_to_execute(
        state: &WorkflowState,
        available_steps: &[String],
    ) -> Option<String> {
        available_steps
            .iter()
            .find(|step_id| !Self::is_step_completed(state, step_id))
            .cloned()
    }

    /// Persist workflow state to file (YAML format)
    pub fn persist_state(state: &WorkflowState, path: &Path) -> WorkflowResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                WorkflowError::StateError(format!("Failed to create state directory: {}", e))
            })?;
        }

        let yaml = serde_yaml::to_string(state)
            .map_err(|e| WorkflowError::StateError(format!("Failed to serialize state: {}", e)))?;

        std::fs::write(path, yaml)
            .map_err(|e| WorkflowError::StateError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Persist workflow state to file (JSON format)
    pub fn persist_state_json(state: &WorkflowState, path: &Path) -> WorkflowResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                WorkflowError::StateError(format!("Failed to create state directory: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(state)
            .map_err(|e| WorkflowError::StateError(format!("Failed to serialize state: {}", e)))?;

        std::fs::write(path, json)
            .map_err(|e| WorkflowError::StateError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Load workflow state from file (auto-detects YAML or JSON)
    pub fn load_state(path: &Path) -> WorkflowResult<WorkflowState> {
        if !path.exists() {
            return Err(WorkflowError::StateError(format!(
                "State file not found: {}",
                path.display()
            )));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| WorkflowError::StateError(format!("Failed to read state file: {}", e)))?;

        // Try JSON first
        if let Ok(state) = serde_json::from_str::<WorkflowState>(&content) {
            return Ok(state);
        }

        // Fall back to YAML
        serde_yaml::from_str::<WorkflowState>(&content)
            .map_err(|e| WorkflowError::StateError(format!("Failed to deserialize state: {}", e)))
    }

    /// Validate state integrity
    pub fn validate_state(state: &WorkflowState) -> WorkflowResult<()> {
        // Check that workflow_id is not empty
        if state.workflow_id.is_empty() {
            return Err(WorkflowError::StateError(
                "Workflow ID cannot be empty".to_string(),
            ));
        }

        // Check that all completed steps have results
        for step_id in &state.completed_steps {
            if !state.step_results.contains_key(step_id) {
                return Err(WorkflowError::StateError(format!(
                    "Completed step '{}' has no result",
                    step_id
                )));
            }
        }

        // Check that current step (if any) has a result
        if let Some(current_step) = &state.current_step {
            if !state.step_results.contains_key(current_step) {
                return Err(WorkflowError::StateError(format!(
                    "Current step '{}' has no result",
                    current_step
                )));
            }
        }

        Ok(())
    }

    /// Load state with validation
    pub fn load_state_validated(path: &Path) -> WorkflowResult<WorkflowState> {
        let state = Self::load_state(path)?;
        Self::validate_state(&state)?;
        Ok(state)
    }

    /// Handle corrupted state file gracefully
    pub fn load_state_with_recovery(path: &Path) -> WorkflowResult<WorkflowState> {
        match Self::load_state_validated(path) {
            Ok(state) => Ok(state),
            Err(e) => {
                // Log the error and return a default state
                eprintln!("Warning: Failed to load state file: {}", e);
                Err(e)
            }
        }
    }

    /// Get progress percentage (0-100)
    pub fn get_progress(state: &WorkflowState, total_steps: usize) -> u32 {
        if total_steps == 0 {
            return 0;
        }

        ((state.completed_steps.len() as u32 * 100) / total_steps as u32).min(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ErrorAction, RiskFactors, StepType, WorkflowConfig, WorkflowStep};

    fn create_test_workflow() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                step_type: StepType::Agent(crate::models::AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "test-task".to_string(),
                }),
                config: crate::models::StepConfig {
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
    fn test_create_state() {
        let workflow = create_test_workflow();
        let state = StateManager::create_state(&workflow);

        assert_eq!(state.workflow_id, "test-workflow");
        assert_eq!(state.status, WorkflowStatus::Pending);
        assert!(state.current_step.is_none());
        assert!(state.completed_steps.is_empty());
    }

    #[test]
    fn test_start_workflow() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        StateManager::start_workflow(&mut state);
        assert_eq!(state.status, WorkflowStatus::Running);
    }

    #[test]
    fn test_complete_step() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        StateManager::start_step(&mut state, "step1".to_string());
        StateManager::complete_step(
            &mut state,
            "step1".to_string(),
            Some(serde_json::json!({"result": "success"})),
            100,
        );

        assert!(state.completed_steps.contains(&"step1".to_string()));
        assert_eq!(
            state.step_results.get("step1").unwrap().status,
            StepStatus::Completed
        );
    }

    #[test]
    fn test_get_progress() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        assert_eq!(StateManager::get_progress(&state, 10), 0);

        state.completed_steps.push("step1".to_string());
        assert_eq!(StateManager::get_progress(&state, 10), 10);

        state.completed_steps.push("step2".to_string());
        assert_eq!(StateManager::get_progress(&state, 10), 20);
    }

    #[test]
    fn test_pause_resume_workflow() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        StateManager::start_workflow(&mut state);
        assert_eq!(state.status, WorkflowStatus::Running);

        // Pause the workflow
        let result = StateManager::pause_workflow(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.status, WorkflowStatus::Paused);

        // Resume the workflow
        let result = StateManager::resume_workflow(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.status, WorkflowStatus::Running);
    }

    #[test]
    fn test_pause_non_running_workflow_fails() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        // Try to pause a pending workflow
        let result = StateManager::pause_workflow(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_resume_non_paused_workflow_fails() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        // Try to resume a pending workflow
        let result = StateManager::resume_workflow(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_step_completed() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        assert!(!StateManager::is_step_completed(&state, "step1"));

        state.completed_steps.push("step1".to_string());
        assert!(StateManager::is_step_completed(&state, "step1"));
    }

    #[test]
    fn test_get_next_step_to_execute() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        let available_steps = vec![
            "step1".to_string(),
            "step2".to_string(),
            "step3".to_string(),
        ];

        // First step should be step1
        let next = StateManager::get_next_step_to_execute(&state, &available_steps);
        assert_eq!(next, Some("step1".to_string()));

        // Mark step1 as completed
        state.completed_steps.push("step1".to_string());

        // Next step should be step2
        let next = StateManager::get_next_step_to_execute(&state, &available_steps);
        assert_eq!(next, Some("step2".to_string()));

        // Mark all steps as completed
        state.completed_steps.push("step2".to_string());
        state.completed_steps.push("step3".to_string());

        // No more steps
        let next = StateManager::get_next_step_to_execute(&state, &available_steps);
        assert_eq!(next, None);
    }

    #[test]
    fn test_validate_state_success() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        StateManager::start_step(&mut state, "step1".to_string());
        StateManager::complete_step(&mut state, "step1".to_string(), None, 100);

        let result = StateManager::validate_state(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_state_missing_result() {
        let workflow = create_test_workflow();
        let mut state = StateManager::create_state(&workflow);

        // Add completed step without result
        state.completed_steps.push("step1".to_string());

        let result = StateManager::validate_state(&state);
        assert!(result.is_err());
    }
}
