//! Central execution manager for coordinating plan execution

use crate::error::{ExecutionError, ExecutionResult};
use crate::models::{ExecutionMode, ExecutionPlan, ExecutionState};
use crate::progress_tracker::ProgressTracker;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Central coordinator for execution plan execution
///
/// Manages execution lifecycle (start, pause, resume, cancel) and tracks
/// active executions. Wraps the WorkflowEngine and provides high-level
/// execution plan management.
pub struct ExecutionManager {
    /// Active execution states
    active_executions: HashMap<String, ExecutionState>,
    /// Execution plans
    plans: HashMap<String, ExecutionPlan>,
    /// Progress trackers for active executions
    progress_trackers: HashMap<String, ProgressTracker>,
}

impl Default for ExecutionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionManager {
    /// Create a new execution manager
    pub fn new() -> Self {
        ExecutionManager {
            active_executions: HashMap::new(),
            plans: HashMap::new(),
            progress_trackers: HashMap::new(),
        }
    }

    /// Register an execution plan
    ///
    /// Stores the plan for later execution. Returns the plan ID.
    pub fn register_plan(&mut self, plan: ExecutionPlan) -> ExecutionResult<String> {
        let plan_id = plan.id.clone();
        self.plans.insert(plan_id.clone(), plan);
        Ok(plan_id)
    }

    /// Get a registered plan
    pub fn get_plan(&self, plan_id: &str) -> ExecutionResult<ExecutionPlan> {
        self.plans
            .get(plan_id)
            .cloned()
            .ok_or_else(|| ExecutionError::PlanError(format!("Plan not found: {}", plan_id)))
    }

    /// Start execution of a plan
    ///
    /// Creates a new execution state and begins execution in the specified mode.
    /// Also creates a progress tracker for the execution.
    pub fn start_execution(
        &mut self,
        plan_id: &str,
        mode: ExecutionMode,
    ) -> ExecutionResult<String> {
        let plan = self.get_plan(plan_id)?;

        let execution_id = Uuid::new_v4().to_string();
        let state = ExecutionState {
            execution_id: execution_id.clone(),
            current_step_index: 0,
            completed_steps: Vec::new(),
            mode,
            paused_at: Utc::now(),
        };

        // Create progress tracker for this execution
        let progress_tracker = ProgressTracker::new(&plan);

        self.active_executions.insert(execution_id.clone(), state);
        self.progress_trackers
            .insert(execution_id.clone(), progress_tracker);

        tracing::info!(
            execution_id = %execution_id,
            plan_id = %plan_id,
            mode = ?mode,
            "Execution started"
        );

        Ok(execution_id)
    }

    /// Pause an active execution
    ///
    /// Saves the current execution state for later resumption.
    pub fn pause_execution(&mut self, execution_id: &str) -> ExecutionResult<()> {
        let state = self
            .active_executions
            .get_mut(execution_id)
            .ok_or_else(|| {
                ExecutionError::ValidationError(format!("Execution not found: {}", execution_id))
            })?;

        state.paused_at = Utc::now();

        tracing::info!(
            execution_id = %execution_id,
            "Execution paused"
        );

        Ok(())
    }

    /// Resume a paused execution
    ///
    /// Continues execution from where it was paused.
    pub fn resume_execution(&mut self, execution_id: &str) -> ExecutionResult<()> {
        let state = self
            .active_executions
            .get_mut(execution_id)
            .ok_or_else(|| {
                ExecutionError::ValidationError(format!("Execution not found: {}", execution_id))
            })?;

        // Update pause time to now (for tracking pause duration)
        state.paused_at = Utc::now();

        tracing::info!(
            execution_id = %execution_id,
            "Execution resumed"
        );

        Ok(())
    }

    /// Cancel an active execution
    ///
    /// Stops execution and removes the execution state and progress tracker.
    pub fn cancel_execution(&mut self, execution_id: &str) -> ExecutionResult<()> {
        self.active_executions.remove(execution_id).ok_or_else(|| {
            ExecutionError::ValidationError(format!("Execution not found: {}", execution_id))
        })?;

        // Clean up progress tracker
        self.progress_trackers.remove(execution_id);

        tracing::info!(
            execution_id = %execution_id,
            "Execution cancelled"
        );

        Ok(())
    }

    /// Get the current state of an execution
    pub fn get_execution_state(&self, execution_id: &str) -> ExecutionResult<ExecutionState> {
        self.active_executions
            .get(execution_id)
            .cloned()
            .ok_or_else(|| {
                ExecutionError::ValidationError(format!("Execution not found: {}", execution_id))
            })
    }

    /// Get all active executions
    pub fn get_active_executions(&self) -> Vec<ExecutionState> {
        self.active_executions.values().cloned().collect()
    }

    /// Check if an execution is active
    pub fn is_active(&self, execution_id: &str) -> bool {
        self.active_executions.contains_key(execution_id)
    }

    /// Update execution state (internal use)
    #[allow(dead_code)]
    pub(crate) fn update_execution_state(
        &mut self,
        execution_id: &str,
        state: ExecutionState,
    ) -> ExecutionResult<()> {
        self.active_executions
            .insert(execution_id.to_string(), state);
        Ok(())
    }

    /// Get the progress tracker for an execution
    ///
    /// Returns a mutable reference to the progress tracker for the given execution.
    pub fn get_progress_tracker_mut(
        &mut self,
        execution_id: &str,
    ) -> ExecutionResult<&mut ProgressTracker> {
        self.progress_trackers.get_mut(execution_id).ok_or_else(|| {
            ExecutionError::ValidationError(format!(
                "Progress tracker not found for execution: {}",
                execution_id
            ))
        })
    }

    /// Get the progress tracker for an execution (read-only)
    ///
    /// Returns a reference to the progress tracker for the given execution.
    pub fn get_progress_tracker(&self, execution_id: &str) -> ExecutionResult<&ProgressTracker> {
        self.progress_trackers.get(execution_id).ok_or_else(|| {
            ExecutionError::ValidationError(format!(
                "Progress tracker not found for execution: {}",
                execution_id
            ))
        })
    }

    /// Register a progress callback for an execution
    ///
    /// Callbacks are called whenever progress is updated during execution.
    pub fn on_progress<F>(&mut self, execution_id: &str, callback: F) -> ExecutionResult<()>
    where
        F: Fn(crate::progress_tracker::ProgressUpdate) + Send + Sync + 'static,
    {
        let tracker = self.get_progress_tracker_mut(execution_id)?;
        tracker.on_progress(callback);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manager() {
        let manager = ExecutionManager::new();
        assert_eq!(manager.active_executions.len(), 0);
        assert_eq!(manager.plans.len(), 0);
    }

    #[test]
    fn test_register_plan() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);

        let plan_id = manager.register_plan(plan.clone()).unwrap();
        assert_eq!(plan_id, plan.id);
        assert!(manager.get_plan(&plan_id).is_ok());
    }

    #[test]
    fn test_start_execution() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);
        let plan_id = manager.register_plan(plan).unwrap();

        let execution_id = manager
            .start_execution(&plan_id, ExecutionMode::Automatic)
            .unwrap();

        assert!(manager.is_active(&execution_id));
    }

    #[test]
    fn test_pause_resume_execution() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);
        let plan_id = manager.register_plan(plan).unwrap();

        let execution_id = manager
            .start_execution(&plan_id, ExecutionMode::Automatic)
            .unwrap();

        manager.pause_execution(&execution_id).unwrap();
        assert!(manager.is_active(&execution_id));

        manager.resume_execution(&execution_id).unwrap();
        assert!(manager.is_active(&execution_id));
    }

    #[test]
    fn test_cancel_execution() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);
        let plan_id = manager.register_plan(plan).unwrap();

        let execution_id = manager
            .start_execution(&plan_id, ExecutionMode::Automatic)
            .unwrap();

        assert!(manager.is_active(&execution_id));
        manager.cancel_execution(&execution_id).unwrap();
        assert!(!manager.is_active(&execution_id));
    }

    #[test]
    fn test_get_execution_state() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);
        let plan_id = manager.register_plan(plan).unwrap();

        let execution_id = manager
            .start_execution(&plan_id, ExecutionMode::StepByStep)
            .unwrap();

        let state = manager.get_execution_state(&execution_id).unwrap();
        assert_eq!(state.execution_id, execution_id);
        assert_eq!(state.mode, ExecutionMode::StepByStep);
        assert_eq!(state.current_step_index, 0);
    }

    #[test]
    fn test_get_active_executions() {
        let mut manager = ExecutionManager::new();
        let plan1 = ExecutionPlan::new("test1".to_string(), vec![]);
        let plan2 = ExecutionPlan::new("test2".to_string(), vec![]);

        let plan_id1 = manager.register_plan(plan1).unwrap();
        let plan_id2 = manager.register_plan(plan2).unwrap();

        let _exec_id1 = manager
            .start_execution(&plan_id1, ExecutionMode::Automatic)
            .unwrap();
        let _exec_id2 = manager
            .start_execution(&plan_id2, ExecutionMode::DryRun)
            .unwrap();

        let active = manager.get_active_executions();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_nonexistent_plan() {
        let manager = ExecutionManager::new();
        assert!(manager.get_plan("nonexistent").is_err());
    }

    #[test]
    fn test_nonexistent_execution() {
        let manager = ExecutionManager::new();
        assert!(manager.get_execution_state("nonexistent").is_err());
    }

    #[test]
    fn test_progress_tracker_created() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);
        let plan_id = manager.register_plan(plan).unwrap();

        let execution_id = manager
            .start_execution(&plan_id, ExecutionMode::Automatic)
            .unwrap();

        let tracker = manager.get_progress_tracker(&execution_id);
        assert!(tracker.is_ok());
    }

    #[test]
    fn test_progress_tracker_cleanup() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);
        let plan_id = manager.register_plan(plan).unwrap();

        let execution_id = manager
            .start_execution(&plan_id, ExecutionMode::Automatic)
            .unwrap();

        assert!(manager.get_progress_tracker(&execution_id).is_ok());

        manager.cancel_execution(&execution_id).unwrap();

        assert!(manager.get_progress_tracker(&execution_id).is_err());
    }

    #[test]
    fn test_progress_callback_registration() {
        let mut manager = ExecutionManager::new();
        let plan = ExecutionPlan::new("test".to_string(), vec![]);
        let plan_id = manager.register_plan(plan).unwrap();

        let execution_id = manager
            .start_execution(&plan_id, ExecutionMode::Automatic)
            .unwrap();

        let result = manager.on_progress(&execution_id, |_progress| {
            // Callback
        });

        assert!(result.is_ok());
    }
}
