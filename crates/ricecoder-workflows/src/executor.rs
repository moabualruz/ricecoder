//! Step execution orchestration

use crate::agent_executor::AgentExecutor;
use crate::approval::ApprovalGate;
use crate::command_executor::CommandExecutor;
use crate::condition::ConditionEvaluator;
use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{StepResult, StepStatus, StepType, Workflow, WorkflowState};
use crate::parallel_executor::ParallelExecutor;
use crate::state::StateManager;
use std::time::Instant;

/// Orchestrates step execution within workflows
///
/// Handles:
/// - Executing steps in dependency order
/// - Managing step context and results
/// - Handling step dependencies and waiting
pub struct StepExecutor;

impl StepExecutor {
    /// Execute a single step
    ///
    /// Marks the step as running, executes it, and records the result.
    /// Dispatches to the appropriate step type handler (agent, command, condition, parallel, approval).
    pub fn execute_step(
        workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
    ) -> WorkflowResult<()> {
        // Find the step
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Dispatch to the appropriate handler based on step type
        match &step.step_type {
            StepType::Agent(agent_step) => {
                AgentExecutor::execute_agent_step(workflow, state, step_id, agent_step)
            }
            StepType::Command(command_step) => {
                CommandExecutor::execute_command_step(workflow, state, step_id, command_step)
            }
            StepType::Condition(condition_step) => {
                // For condition steps, we execute them and get the next steps
                StateManager::start_step(state, step_id.to_string());
                let start_time = Instant::now();

                let next_steps =
                    ConditionEvaluator::evaluate_condition(workflow, state, condition_step)?;

                let duration_ms = start_time.elapsed().as_millis() as u64;

                StateManager::complete_step(
                    state,
                    step_id.to_string(),
                    Some(serde_json::json!({
                        "next_steps": next_steps,
                        "condition": condition_step.condition,
                    })),
                    duration_ms,
                );

                Ok(())
            }
            StepType::Parallel(parallel_step) => {
                ParallelExecutor::execute_parallel_step(workflow, state, step_id, parallel_step)
            }
            StepType::Approval(approval_step) => {
                // For approval steps, we mark them as completed
                // In a real implementation, this would request approval and wait
                StateManager::start_step(state, step_id.to_string());
                let start_time = Instant::now();

                let duration_ms = start_time.elapsed().as_millis() as u64;

                StateManager::complete_step(
                    state,
                    step_id.to_string(),
                    Some(serde_json::json!({
                        "message": &approval_step.message,
                        "timeout": approval_step.timeout,
                        "default": format!("{:?}", approval_step.default),
                    })),
                    duration_ms,
                );

                Ok(())
            }
        }
    }

    /// Execute steps in dependency order
    ///
    /// Executes all steps in the workflow, respecting dependencies.
    /// Stops on first error unless error handling specifies otherwise.
    pub fn execute_workflow(workflow: &Workflow, state: &mut WorkflowState) -> WorkflowResult<()> {
        // Get execution order
        let order = crate::engine::WorkflowEngine::get_execution_order(workflow)?;

        // Execute each step in order
        for step_id in order {
            // Check if step can be executed
            if !crate::engine::WorkflowEngine::can_execute_step(workflow, state, &step_id)? {
                return Err(WorkflowError::StateError(format!(
                    "Cannot execute step {}: dependencies not met",
                    step_id
                )));
            }

            // Execute the step
            Self::execute_step(workflow, state, &step_id)?;
        }

        Ok(())
    }

    /// Execute the next available step
    ///
    /// Finds and executes the next step that is ready to run.
    /// Returns the ID of the executed step, or None if no steps are ready.
    pub fn execute_next_step(
        workflow: &Workflow,
        state: &mut WorkflowState,
    ) -> WorkflowResult<Option<String>> {
        // Get next executable step
        if let Some(step_id) = crate::engine::WorkflowEngine::get_next_step(workflow, state)? {
            Self::execute_step(workflow, state, &step_id)?;
            Ok(Some(step_id))
        } else {
            Ok(None)
        }
    }

    /// Get step context for execution
    ///
    /// Builds the context needed to execute a step, including:
    /// - Step configuration
    /// - Results from dependent steps
    /// - Workflow parameters
    pub fn get_step_context(
        workflow: &Workflow,
        state: &WorkflowState,
        step_id: &str,
    ) -> WorkflowResult<serde_json::Value> {
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Build context with step config and dependency results
        let mut context = serde_json::json!({
            "step_id": step_id,
            "step_name": &step.name,
            "config": &step.config.config,
            "dependencies": {}
        });

        // Add results from dependent steps
        if let Some(obj) = context.get_mut("dependencies") {
            for dep_id in &step.dependencies {
                if let Some(result) = state.step_results.get(dep_id) {
                    if let Some(output) = &result.output {
                        if let Some(deps_obj) = obj.as_object_mut() {
                            deps_obj.insert(dep_id.clone(), output.clone());
                        }
                    }
                }
            }
        }

        Ok(context)
    }

    /// Mark a step as failed with error details
    pub fn fail_step(
        state: &mut WorkflowState,
        step_id: &str,
        error: String,
    ) -> WorkflowResult<()> {
        let duration_ms = state
            .step_results
            .get(step_id)
            .map(|r| r.duration_ms)
            .unwrap_or(0);

        StateManager::fail_step(state, step_id.to_string(), error, duration_ms);
        Ok(())
    }

    /// Mark a step as skipped
    pub fn skip_step(state: &mut WorkflowState, step_id: &str) -> WorkflowResult<()> {
        StateManager::skip_step(state, step_id.to_string());
        Ok(())
    }

    /// Get the status of a step
    pub fn get_step_status(state: &WorkflowState, step_id: &str) -> Option<StepStatus> {
        state.step_results.get(step_id).map(|r| r.status)
    }

    /// Check if a step has completed successfully
    pub fn is_step_completed(state: &WorkflowState, step_id: &str) -> bool {
        state
            .step_results
            .get(step_id)
            .map(|r| r.status == StepStatus::Completed)
            .unwrap_or(false)
    }

    /// Check if a step has failed
    pub fn is_step_failed(state: &WorkflowState, step_id: &str) -> bool {
        state
            .step_results
            .get(step_id)
            .map(|r| r.status == StepStatus::Failed)
            .unwrap_or(false)
    }

    /// Get step result
    pub fn get_step_result(state: &WorkflowState, step_id: &str) -> Option<StepResult> {
        state.step_results.get(step_id).cloned()
    }

    /// Execute a condition step and return the next steps to execute
    ///
    /// Evaluates the condition and returns the list of step IDs that should be executed
    /// based on the condition result (then_steps or else_steps).
    pub fn execute_condition_step(
        workflow: &Workflow,
        state: &mut WorkflowState,
        step_id: &str,
    ) -> WorkflowResult<Vec<String>> {
        // Find the step
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Extract the condition step
        let condition_step = match &step.step_type {
            StepType::Condition(cs) => cs,
            _ => {
                return Err(WorkflowError::Invalid(format!(
                    "Step {} is not a condition step",
                    step_id
                )))
            }
        };

        // Mark step as started
        StateManager::start_step(state, step_id.to_string());

        // Record execution time
        let start_time = Instant::now();

        // Evaluate the condition
        let next_steps = ConditionEvaluator::evaluate_condition(workflow, state, condition_step)?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Mark step as completed with the next steps as output
        StateManager::complete_step(
            state,
            step_id.to_string(),
            Some(serde_json::json!({
                "next_steps": next_steps,
                "condition": condition_step.condition,
            })),
            duration_ms,
        );

        Ok(next_steps)
    }

    /// Get the next steps to execute after a condition step
    ///
    /// Returns the list of step IDs that should be executed based on the condition result.
    pub fn get_condition_next_steps(
        workflow: &Workflow,
        state: &WorkflowState,
        step_id: &str,
    ) -> WorkflowResult<Vec<String>> {
        // Find the step
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Extract the condition step
        let condition_step = match &step.step_type {
            StepType::Condition(cs) => cs,
            _ => {
                return Err(WorkflowError::Invalid(format!(
                    "Step {} is not a condition step",
                    step_id
                )))
            }
        };

        // Evaluate the condition
        ConditionEvaluator::evaluate_condition(workflow, state, condition_step)
    }

    /// Check if a step requires approval
    ///
    /// Returns true if the step has approval_required set to true.
    pub fn requires_approval(workflow: &Workflow, step_id: &str) -> WorkflowResult<bool> {
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        Ok(step.approval_required)
    }

    /// Request approval for a step
    ///
    /// Creates an approval request for the step. The step will not execute
    /// until the approval is granted.
    pub fn request_step_approval(
        approval_gate: &mut ApprovalGate,
        workflow: &Workflow,
        step_id: &str,
    ) -> WorkflowResult<String> {
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        let message = format!("Approval required for step: {}", step.name);
        let timeout_ms = 3600000; // 1 hour default timeout

        approval_gate.request_approval(step_id.to_string(), message, timeout_ms)
    }

    /// Check if a step has been approved
    ///
    /// Returns true if the approval request has been approved.
    /// Returns error if the request is not found or timed out.
    pub fn is_step_approved(
        approval_gate: &ApprovalGate,
        request_id: &str,
    ) -> WorkflowResult<bool> {
        approval_gate.is_approved(request_id)
    }

    /// Check if a step has been rejected
    ///
    /// Returns true if the approval request has been rejected.
    /// Returns error if the request is not found or timed out.
    pub fn is_step_rejected(
        approval_gate: &ApprovalGate,
        request_id: &str,
    ) -> WorkflowResult<bool> {
        approval_gate.is_rejected(request_id)
    }

    /// Check if approval is still pending
    ///
    /// Returns true if the approval request is still pending.
    pub fn is_approval_pending(
        approval_gate: &ApprovalGate,
        request_id: &str,
    ) -> WorkflowResult<bool> {
        approval_gate.is_pending(request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AgentStep, ErrorAction, RiskFactors, StepConfig, StepType, WorkflowConfig, WorkflowStep,
    };

    fn create_simple_workflow() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![WorkflowStep {
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
    fn test_execute_step() {
        let workflow = create_simple_workflow();
        let mut state = StateManager::create_state(&workflow);

        let result = StepExecutor::execute_step(&workflow, &mut state, "step1");
        assert!(result.is_ok());

        assert!(StepExecutor::is_step_completed(&state, "step1"));
        assert!(state.completed_steps.contains(&"step1".to_string()));
    }

    #[test]
    fn test_execute_next_step() {
        let workflow = create_simple_workflow();
        let mut state = StateManager::create_state(&workflow);

        let result = StepExecutor::execute_next_step(&workflow, &mut state);
        assert!(result.is_ok());

        let executed = result.unwrap();
        assert_eq!(executed, Some("step1".to_string()));
        assert!(StepExecutor::is_step_completed(&state, "step1"));
    }

    #[test]
    fn test_get_step_context() {
        let workflow = create_simple_workflow();
        let state = StateManager::create_state(&workflow);

        let context = StepExecutor::get_step_context(&workflow, &state, "step1");
        assert!(context.is_ok());

        let context = context.unwrap();
        assert_eq!(context["step_id"], "step1");
        assert_eq!(context["step_name"], "Step 1");
    }

    #[test]
    fn test_fail_step() {
        let workflow = create_simple_workflow();
        let mut state = StateManager::create_state(&workflow);

        StateManager::start_step(&mut state, "step1".to_string());
        let result = StepExecutor::fail_step(&mut state, "step1", "Test error".to_string());
        assert!(result.is_ok());

        assert!(StepExecutor::is_step_failed(&state, "step1"));
    }

    #[test]
    fn test_skip_step() {
        let workflow = create_simple_workflow();
        let mut state = StateManager::create_state(&workflow);

        // First start the step to create a result entry
        StateManager::start_step(&mut state, "step1".to_string());

        let result = StepExecutor::skip_step(&mut state, "step1");
        assert!(result.is_ok());

        let status = StepExecutor::get_step_status(&state, "step1");
        assert_eq!(status, Some(StepStatus::Skipped));
    }

    #[test]
    fn test_get_step_result() {
        let workflow = create_simple_workflow();
        let mut state = StateManager::create_state(&workflow);

        StateManager::start_step(&mut state, "step1".to_string());
        StateManager::complete_step(
            &mut state,
            "step1".to_string(),
            Some(serde_json::json!({"result": "success"})),
            100,
        );

        let result = StepExecutor::get_step_result(&state, "step1");
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.status, StepStatus::Completed);
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_execute_condition_step_then_branch() {
        use crate::models::{ConditionStep, StepStatus};

        let workflow = Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![
                WorkflowStep {
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
                },
                WorkflowStep {
                    id: "condition".to_string(),
                    name: "Condition".to_string(),
                    step_type: StepType::Condition(ConditionStep {
                        condition: "step1.output.count > 5".to_string(),
                        then_steps: vec!["step2".to_string()],
                        else_steps: vec!["step3".to_string()],
                    }),
                    config: crate::models::StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["step1".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
            ],
            config: crate::models::WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        };

        let mut state = StateManager::create_state(&workflow);

        // Add step1 result with count > 5
        state.step_results.insert(
            "step1".to_string(),
            StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 10})),
                error: None,
                duration_ms: 100,
            },
        );
        state.completed_steps.push("step1".to_string());

        let result = StepExecutor::execute_condition_step(&workflow, &mut state, "condition");
        assert!(result.is_ok());

        let next_steps = result.unwrap();
        assert_eq!(next_steps, vec!["step2".to_string()]);
        assert!(StepExecutor::is_step_completed(&state, "condition"));
    }

    #[test]
    fn test_execute_condition_step_else_branch() {
        use crate::models::{ConditionStep, StepStatus};

        let workflow = Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![
                WorkflowStep {
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
                },
                WorkflowStep {
                    id: "condition".to_string(),
                    name: "Condition".to_string(),
                    step_type: StepType::Condition(ConditionStep {
                        condition: "step1.output.count > 5".to_string(),
                        then_steps: vec!["step2".to_string()],
                        else_steps: vec!["step3".to_string()],
                    }),
                    config: crate::models::StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["step1".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
            ],
            config: crate::models::WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        };

        let mut state = StateManager::create_state(&workflow);

        // Add step1 result with count <= 5
        state.step_results.insert(
            "step1".to_string(),
            StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 3})),
                error: None,
                duration_ms: 100,
            },
        );
        state.completed_steps.push("step1".to_string());

        let result = StepExecutor::execute_condition_step(&workflow, &mut state, "condition");
        assert!(result.is_ok());

        let next_steps = result.unwrap();
        assert_eq!(next_steps, vec!["step3".to_string()]);
        assert!(StepExecutor::is_step_completed(&state, "condition"));
    }

    #[test]
    fn test_get_condition_next_steps() {
        use crate::models::{ConditionStep, StepStatus};

        let workflow = Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![
                WorkflowStep {
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
                },
                WorkflowStep {
                    id: "condition".to_string(),
                    name: "Condition".to_string(),
                    step_type: StepType::Condition(ConditionStep {
                        condition: "step1.output.status == 'success'".to_string(),
                        then_steps: vec!["step2".to_string()],
                        else_steps: vec!["step3".to_string()],
                    }),
                    config: crate::models::StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["step1".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
            ],
            config: crate::models::WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        };

        let mut state = StateManager::create_state(&workflow);

        // Add step1 result with status = 'success'
        state.step_results.insert(
            "step1".to_string(),
            StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"status": "success"})),
                error: None,
                duration_ms: 100,
            },
        );

        let result = StepExecutor::get_condition_next_steps(&workflow, &state, "condition");
        assert!(result.is_ok());

        let next_steps = result.unwrap();
        assert_eq!(next_steps, vec!["step2".to_string()]);
    }

    #[test]
    fn test_requires_approval() {
        let workflow = Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![
                WorkflowStep {
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
                    approval_required: true,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
                WorkflowStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
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
                },
            ],
            config: crate::models::WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        };

        assert!(StepExecutor::requires_approval(&workflow, "step1").unwrap());
        assert!(!StepExecutor::requires_approval(&workflow, "step2").unwrap());
    }

    #[test]
    fn test_request_step_approval() {
        use crate::approval::ApprovalGate;

        let workflow = create_simple_workflow();
        let mut approval_gate = ApprovalGate::new();

        let request_id =
            StepExecutor::request_step_approval(&mut approval_gate, &workflow, "step1").unwrap();

        assert!(!request_id.is_empty());
        assert!(StepExecutor::is_approval_pending(&approval_gate, &request_id).unwrap());
    }

    #[test]
    fn test_is_step_approved() {
        use crate::approval::ApprovalGate;

        let workflow = create_simple_workflow();
        let mut approval_gate = ApprovalGate::new();

        let request_id =
            StepExecutor::request_step_approval(&mut approval_gate, &workflow, "step1").unwrap();

        // Initially not approved
        assert!(!StepExecutor::is_step_approved(&approval_gate, &request_id).unwrap());

        // Approve it
        approval_gate.approve(&request_id, None).unwrap();

        // Now it should be approved
        assert!(StepExecutor::is_step_approved(&approval_gate, &request_id).unwrap());
    }

    #[test]
    fn test_is_step_rejected() {
        use crate::approval::ApprovalGate;

        let workflow = create_simple_workflow();
        let mut approval_gate = ApprovalGate::new();

        let request_id =
            StepExecutor::request_step_approval(&mut approval_gate, &workflow, "step1").unwrap();

        // Initially not rejected
        assert!(!StepExecutor::is_step_rejected(&approval_gate, &request_id).unwrap());

        // Reject it
        approval_gate.reject(&request_id, None).unwrap();

        // Now it should be rejected
        assert!(StepExecutor::is_step_rejected(&approval_gate, &request_id).unwrap());
    }
}
