//! Rollback capability for workflow recovery

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{Workflow, WorkflowState};
use crate::state::StateManager;
use std::collections::HashMap;

/// Manages rollback steps and recovery for failed workflows
///
/// Responsible for:
/// - Tracking rollback steps for each executed step
/// - Executing rollback steps in reverse order on failure
/// - Restoring workflow state after rollback
pub struct RollbackManager;

/// Rollback plan for a workflow
#[derive(Debug, Clone)]
pub struct RollbackPlan {
    /// Mapping of step ID to its rollback steps
    pub rollback_steps: HashMap<String, Vec<String>>,
    /// Order of steps that were executed
    pub execution_order: Vec<String>,
}

impl RollbackPlan {
    /// Create a new rollback plan
    pub fn new() -> Self {
        Self {
            rollback_steps: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    /// Add a rollback step for a given step
    pub fn add_rollback_step(&mut self, step_id: String, rollback_step: String) {
        self.rollback_steps
            .entry(step_id)
            .or_default()
            .push(rollback_step);
    }

    /// Record that a step was executed
    pub fn record_execution(&mut self, step_id: String) {
        self.execution_order.push(step_id);
    }

    /// Get the rollback steps in reverse execution order
    pub fn get_rollback_order(&self) -> Vec<String> {
        let mut rollback_order = Vec::new();

        // Iterate through execution order in reverse
        for step_id in self.execution_order.iter().rev() {
            if let Some(rollback_steps) = self.rollback_steps.get(step_id) {
                rollback_order.extend(rollback_steps.clone());
            }
        }

        rollback_order
    }
}

impl Default for RollbackPlan {
    fn default() -> Self {
        Self::new()
    }
}

impl RollbackManager {
    /// Create a rollback plan for a workflow
    pub fn create_rollback_plan(workflow: &Workflow) -> RollbackPlan {
        let mut plan = RollbackPlan::new();

        // For each step, check if it has rollback steps defined
        for step in &workflow.steps {
            // In a real implementation, rollback steps would be defined in the workflow
            // For now, we just create an empty plan
            plan.record_execution(step.id.clone());
        }

        plan
    }

    /// Execute rollback steps in reverse order
    ///
    /// Executes all rollback steps that were defined for the failed step,
    /// in reverse order of execution.
    pub fn execute_rollback(
        workflow: &Workflow,
        state: &mut WorkflowState,
        rollback_plan: &RollbackPlan,
    ) -> WorkflowResult<()> {
        let rollback_order = rollback_plan.get_rollback_order();

        // Execute rollback steps in order
        for rollback_step_id in rollback_order {
            // Find the rollback step in the workflow
            let _step = workflow
                .steps
                .iter()
                .find(|s| s.id == rollback_step_id)
                .ok_or_else(|| {
                    WorkflowError::NotFound(format!("Rollback step not found: {}", rollback_step_id))
                })?;

            // Mark rollback step as started
            StateManager::start_step(state, rollback_step_id.clone());

            // In a real implementation, this would execute the rollback step
            // For now, we just mark it as completed
            StateManager::complete_step(
                state,
                rollback_step_id,
                Some(serde_json::json!({"rollback": true})),
                0,
            );
        }

        Ok(())
    }

    /// Restore workflow state after rollback
    ///
    /// Clears the completed steps and step results to restore the workflow
    /// to a state where it can be resumed or restarted.
    pub fn restore_state(state: &mut WorkflowState) -> WorkflowResult<()> {
        // Clear completed steps
        state.completed_steps.clear();

        // Clear step results
        state.step_results.clear();

        // Reset current step
        state.current_step = None;

        Ok(())
    }

    /// Check if a step has rollback steps defined
    pub fn has_rollback_steps(rollback_plan: &RollbackPlan, step_id: &str) -> bool {
        rollback_plan
            .rollback_steps
            .get(step_id)
            .map(|steps| !steps.is_empty())
            .unwrap_or(false)
    }

    /// Get rollback steps for a specific step
    pub fn get_rollback_steps(rollback_plan: &RollbackPlan, step_id: &str) -> Vec<String> {
        rollback_plan
            .rollback_steps
            .get(step_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Add a rollback step to the plan
    pub fn add_rollback_step(
        rollback_plan: &mut RollbackPlan,
        step_id: String,
        rollback_step: String,
    ) {
        rollback_plan.add_rollback_step(step_id, rollback_step);
    }

    /// Record step execution in the rollback plan
    pub fn record_step_execution(rollback_plan: &mut RollbackPlan, step_id: String) {
        rollback_plan.record_execution(step_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentStep, ErrorAction, StepConfig, StepType, WorkflowConfig, WorkflowStep, RiskFactors};

    fn create_simple_workflow() -> Workflow {
        Workflow {
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
                        config: serde_json::json!({"param": "value"}),
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
                        config: serde_json::json!({"param": "value"}),
                    },
                    dependencies: vec!["step1".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail, risk_score: None, risk_factors: RiskFactors::default(),
                },
            ],
            config: WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        }
    }

    #[test]
    fn test_rollback_plan_creation() {
        let plan = RollbackPlan::new();
        assert!(plan.rollback_steps.is_empty());
        assert!(plan.execution_order.is_empty());
    }

    #[test]
    fn test_add_rollback_step() {
        let mut plan = RollbackPlan::new();
        plan.add_rollback_step("step1".to_string(), "rollback1".to_string());

        assert!(plan.rollback_steps.contains_key("step1"));
        assert_eq!(
            plan.rollback_steps.get("step1").unwrap(),
            &vec!["rollback1".to_string()]
        );
    }

    #[test]
    fn test_record_execution() {
        let mut plan = RollbackPlan::new();
        plan.record_execution("step1".to_string());
        plan.record_execution("step2".to_string());

        assert_eq!(plan.execution_order, vec!["step1".to_string(), "step2".to_string()]);
    }

    #[test]
    fn test_get_rollback_order() {
        let mut plan = RollbackPlan::new();

        plan.add_rollback_step("step1".to_string(), "rollback1".to_string());
        plan.add_rollback_step("step2".to_string(), "rollback2".to_string());

        plan.record_execution("step1".to_string());
        plan.record_execution("step2".to_string());

        let rollback_order = plan.get_rollback_order();

        // Should be in reverse order: step2's rollback first, then step1's
        assert_eq!(
            rollback_order,
            vec!["rollback2".to_string(), "rollback1".to_string()]
        );
    }

    #[test]
    fn test_create_rollback_plan() {
        let workflow = create_simple_workflow();
        let plan = RollbackManager::create_rollback_plan(&workflow);

        assert_eq!(plan.execution_order.len(), 2);
        assert_eq!(plan.execution_order[0], "step1");
        assert_eq!(plan.execution_order[1], "step2");
    }

    #[test]
    fn test_restore_state() {
        let workflow = create_simple_workflow();
        let mut state = StateManager::create_state(&workflow);

        // Add some data to state
        state.completed_steps.push("step1".to_string());
        state.current_step = Some("step2".to_string());

        let result = RollbackManager::restore_state(&mut state);
        assert!(result.is_ok());

        assert!(state.completed_steps.is_empty());
        assert!(state.step_results.is_empty());
        assert!(state.current_step.is_none());
    }

    #[test]
    fn test_has_rollback_steps() {
        let mut plan = RollbackPlan::new();
        plan.add_rollback_step("step1".to_string(), "rollback1".to_string());

        assert!(RollbackManager::has_rollback_steps(&plan, "step1"));
        assert!(!RollbackManager::has_rollback_steps(&plan, "step2"));
    }

    #[test]
    fn test_get_rollback_steps() {
        let mut plan = RollbackPlan::new();
        plan.add_rollback_step("step1".to_string(), "rollback1".to_string());
        plan.add_rollback_step("step1".to_string(), "rollback2".to_string());

        let steps = RollbackManager::get_rollback_steps(&plan, "step1");
        assert_eq!(steps, vec!["rollback1".to_string(), "rollback2".to_string()]);
    }

    #[test]
    fn test_add_rollback_step_to_plan() {
        let mut plan = RollbackPlan::new();
        RollbackManager::add_rollback_step(&mut plan, "step1".to_string(), "rollback1".to_string());

        assert!(RollbackManager::has_rollback_steps(&plan, "step1"));
    }

    #[test]
    fn test_record_step_execution() {
        let mut plan = RollbackPlan::new();
        RollbackManager::record_step_execution(&mut plan, "step1".to_string());

        assert_eq!(plan.execution_order, vec!["step1".to_string()]);
    }
}


