//! Spec-driven development workflow orchestration

use std::collections::{HashMap, HashSet};

use crate::{
    error::SpecError,
    models::{Spec, Task, TaskStatus},
};

/// Orchestrates spec-driven development workflows
///
/// Manages the relationship between specs and implementation tasks, enabling
/// spec-to-task linking, task completion tracking, and acceptance criteria validation.
#[derive(Debug, Clone)]
pub struct WorkflowOrchestrator {
    /// Mapping of task IDs to their linked requirement IDs
    task_to_requirements: HashMap<String, Vec<String>>,
    /// Mapping of requirement IDs to their linked task IDs
    requirement_to_tasks: HashMap<String, Vec<String>>,
    /// Mapping of task IDs to their completion status
    task_completion: HashMap<String, TaskStatus>,
}

impl WorkflowOrchestrator {
    /// Create a new workflow orchestrator
    pub fn new() -> Self {
        WorkflowOrchestrator {
            task_to_requirements: HashMap::new(),
            requirement_to_tasks: HashMap::new(),
            task_completion: HashMap::new(),
        }
    }

    /// Link a task to acceptance criteria from requirements
    ///
    /// Establishes explicit links between implementation tasks and acceptance criteria
    /// from the requirements document. This enables traceability and validation.
    ///
    /// # Arguments
    /// * `task_id` - The task identifier
    /// * `requirement_ids` - IDs of requirements this task addresses
    ///
    /// # Returns
    /// Ok if linking succeeds, Err if task or requirement IDs are invalid
    pub fn link_task_to_requirements(
        &mut self,
        task_id: String,
        requirement_ids: Vec<String>,
    ) -> Result<(), SpecError> {
        if task_id.is_empty() {
            return Err(SpecError::InvalidFormat(
                "Task ID cannot be empty".to_string(),
            ));
        }

        if requirement_ids.is_empty() {
            return Err(SpecError::InvalidFormat(
                "At least one requirement ID must be provided".to_string(),
            ));
        }

        // Store task-to-requirements mapping
        self.task_to_requirements
            .insert(task_id.clone(), requirement_ids.clone());

        // Store reverse mapping (requirement-to-tasks)
        for req_id in requirement_ids {
            self.requirement_to_tasks
                .entry(req_id)
                .or_default()
                .push(task_id.clone());
        }

        // Initialize task completion status
        self.task_completion
            .entry(task_id)
            .or_insert(TaskStatus::NotStarted);

        Ok(())
    }

    /// Get all requirements linked to a task
    ///
    /// # Arguments
    /// * `task_id` - The task identifier
    ///
    /// # Returns
    /// A vector of requirement IDs linked to this task
    pub fn get_task_requirements(&self, task_id: &str) -> Vec<String> {
        self.task_to_requirements
            .get(task_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all tasks linked to a requirement
    ///
    /// # Arguments
    /// * `requirement_id` - The requirement identifier
    ///
    /// # Returns
    /// A vector of task IDs linked to this requirement
    pub fn get_requirement_tasks(&self, requirement_id: &str) -> Vec<String> {
        self.requirement_to_tasks
            .get(requirement_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Update task completion status
    ///
    /// Tracks the progress of implementation tasks through their lifecycle.
    ///
    /// # Arguments
    /// * `task_id` - The task identifier
    /// * `status` - The new task status
    ///
    /// # Returns
    /// Ok if update succeeds, Err if task is not found
    pub fn update_task_status(
        &mut self,
        task_id: String,
        status: TaskStatus,
    ) -> Result<(), SpecError> {
        if !self.task_completion.contains_key(&task_id) {
            return Err(SpecError::NotFound(format!("Task not found: {}", task_id)));
        }

        self.task_completion.insert(task_id, status);
        Ok(())
    }

    /// Get task completion status
    ///
    /// # Arguments
    /// * `task_id` - The task identifier
    ///
    /// # Returns
    /// The current status of the task, or NotStarted if not found
    pub fn get_task_status(&self, task_id: &str) -> TaskStatus {
        self.task_completion
            .get(task_id)
            .copied()
            .unwrap_or(TaskStatus::NotStarted)
    }

    /// Validate that all tasks have explicit links to acceptance criteria
    ///
    /// Ensures spec-to-task traceability by verifying that every task has
    /// explicit links to at least one requirement.
    ///
    /// # Arguments
    /// * `spec` - The specification to validate
    ///
    /// # Returns
    /// Ok if all tasks are properly linked, Err with list of unlinked tasks
    pub fn validate_task_traceability(&self, spec: &Spec) -> Result<(), SpecError> {
        let mut unlinked_tasks = Vec::new();

        // Collect all task IDs from the spec
        let all_task_ids = self.collect_all_task_ids(&spec.tasks);

        // Check each task for requirement links
        for task_id in all_task_ids {
            if !self.task_to_requirements.contains_key(&task_id) {
                unlinked_tasks.push(task_id);
            }
        }

        if !unlinked_tasks.is_empty() {
            return Err(SpecError::InvalidFormat(format!(
                "Tasks without requirement links: {}",
                unlinked_tasks.join(", ")
            )));
        }

        Ok(())
    }

    /// Validate that all acceptance criteria are addressed by tasks
    ///
    /// Ensures that every acceptance criterion from requirements has at least
    /// one task linked to it.
    ///
    /// # Arguments
    /// * `spec` - The specification to validate
    ///
    /// # Returns
    /// Ok if all acceptance criteria are addressed, Err with list of unaddressed criteria
    pub fn validate_acceptance_criteria_coverage(&self, spec: &Spec) -> Result<(), SpecError> {
        let mut unaddressed_criteria = Vec::new();

        // Check each requirement's acceptance criteria
        for requirement in &spec.requirements {
            for criterion in &requirement.acceptance_criteria {
                let criterion_id = format!("{}.{}", requirement.id, criterion.id);

                // Check if any task is linked to this requirement
                if !self.requirement_to_tasks.contains_key(&requirement.id) {
                    unaddressed_criteria.push(criterion_id);
                }
            }
        }

        if !unaddressed_criteria.is_empty() {
            return Err(SpecError::InvalidFormat(format!(
                "Acceptance criteria without task coverage: {}",
                unaddressed_criteria.join(", ")
            )));
        }

        Ok(())
    }

    /// Get all tasks that are complete
    ///
    /// # Returns
    /// A vector of task IDs that have been marked as complete
    pub fn get_completed_tasks(&self) -> Vec<String> {
        self.task_completion
            .iter()
            .filter(|(_, status)| **status == TaskStatus::Complete)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all tasks that are in progress
    ///
    /// # Returns
    /// A vector of task IDs that are currently in progress
    pub fn get_in_progress_tasks(&self) -> Vec<String> {
        self.task_completion
            .iter()
            .filter(|(_, status)| **status == TaskStatus::InProgress)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all tasks that have not been started
    ///
    /// # Returns
    /// A vector of task IDs that have not been started
    pub fn get_not_started_tasks(&self) -> Vec<String> {
        self.task_completion
            .iter()
            .filter(|(_, status)| **status == TaskStatus::NotStarted)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get overall workflow completion percentage
    ///
    /// Calculates the percentage of tasks that have been completed.
    ///
    /// # Returns
    /// A percentage (0-100) of completed tasks, or 0 if no tasks exist
    pub fn get_completion_percentage(&self) -> f64 {
        if self.task_completion.is_empty() {
            return 0.0;
        }

        let completed = self
            .task_completion
            .values()
            .filter(|status| **status == TaskStatus::Complete)
            .count();

        (completed as f64 / self.task_completion.len() as f64) * 100.0
    }

    /// Collect all task IDs from a hierarchical task list
    #[allow(clippy::only_used_in_recursion)]
    fn collect_all_task_ids(&self, tasks: &[Task]) -> Vec<String> {
        let mut ids = Vec::new();

        for task in tasks {
            ids.push(task.id.clone());
            ids.extend(self.collect_all_task_ids(&task.subtasks));
        }

        ids
    }

    /// Get all linked requirement IDs across all tasks
    ///
    /// # Returns
    /// A set of all requirement IDs that have task links
    pub fn get_all_linked_requirements(&self) -> HashSet<String> {
        self.requirement_to_tasks.keys().cloned().collect()
    }

    /// Get all linked task IDs
    ///
    /// # Returns
    /// A set of all task IDs that have requirement links
    pub fn get_all_linked_tasks(&self) -> HashSet<String> {
        self.task_to_requirements.keys().cloned().collect()
    }

    /// Clear all links and reset the orchestrator
    pub fn reset(&mut self) {
        self.task_to_requirements.clear();
        self.requirement_to_tasks.clear();
        self.task_completion.clear();
    }
}

impl Default for WorkflowOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_task_to_requirements() {
        let mut orchestrator = WorkflowOrchestrator::new();

        let result = orchestrator.link_task_to_requirements(
            "task-1".to_string(),
            vec!["REQ-1".to_string(), "REQ-2".to_string()],
        );

        assert!(result.is_ok());
        assert_eq!(
            orchestrator.get_task_requirements("task-1"),
            vec!["REQ-1".to_string(), "REQ-2".to_string()]
        );
    }

    #[test]
    fn test_link_task_empty_task_id() {
        let mut orchestrator = WorkflowOrchestrator::new();

        let result =
            orchestrator.link_task_to_requirements("".to_string(), vec!["REQ-1".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn test_link_task_empty_requirements() {
        let mut orchestrator = WorkflowOrchestrator::new();

        let result = orchestrator.link_task_to_requirements("task-1".to_string(), vec![]);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_requirement_tasks() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        orchestrator
            .link_task_to_requirements("task-2".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        let tasks = orchestrator.get_requirement_tasks("REQ-1");
        assert_eq!(tasks.len(), 2);
        assert!(tasks.contains(&"task-1".to_string()));
        assert!(tasks.contains(&"task-2".to_string()));
    }

    #[test]
    fn test_update_task_status() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        let result = orchestrator.update_task_status("task-1".to_string(), TaskStatus::InProgress);
        assert!(result.is_ok());
        assert_eq!(
            orchestrator.get_task_status("task-1"),
            TaskStatus::InProgress
        );
    }

    #[test]
    fn test_update_task_status_not_found() {
        let mut orchestrator = WorkflowOrchestrator::new();

        let result =
            orchestrator.update_task_status("nonexistent".to_string(), TaskStatus::Complete);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_completed_tasks() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        orchestrator
            .link_task_to_requirements("task-2".to_string(), vec!["REQ-2".to_string()])
            .unwrap();

        orchestrator
            .update_task_status("task-1".to_string(), TaskStatus::Complete)
            .unwrap();

        let completed = orchestrator.get_completed_tasks();
        assert_eq!(completed.len(), 1);
        assert!(completed.contains(&"task-1".to_string()));
    }

    #[test]
    fn test_get_in_progress_tasks() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        orchestrator
            .update_task_status("task-1".to_string(), TaskStatus::InProgress)
            .unwrap();

        let in_progress = orchestrator.get_in_progress_tasks();
        assert_eq!(in_progress.len(), 1);
        assert!(in_progress.contains(&"task-1".to_string()));
    }

    #[test]
    fn test_get_not_started_tasks() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        let not_started = orchestrator.get_not_started_tasks();
        assert_eq!(not_started.len(), 1);
        assert!(not_started.contains(&"task-1".to_string()));
    }

    #[test]
    fn test_get_completion_percentage() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        orchestrator
            .link_task_to_requirements("task-2".to_string(), vec!["REQ-2".to_string()])
            .unwrap();

        orchestrator
            .update_task_status("task-1".to_string(), TaskStatus::Complete)
            .unwrap();

        let percentage = orchestrator.get_completion_percentage();
        assert_eq!(percentage, 50.0);
    }

    #[test]
    fn test_get_completion_percentage_empty() {
        let orchestrator = WorkflowOrchestrator::new();
        assert_eq!(orchestrator.get_completion_percentage(), 0.0);
    }

    #[test]
    fn test_get_all_linked_requirements() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements(
                "task-1".to_string(),
                vec!["REQ-1".to_string(), "REQ-2".to_string()],
            )
            .unwrap();

        let requirements = orchestrator.get_all_linked_requirements();
        assert_eq!(requirements.len(), 2);
        assert!(requirements.contains("REQ-1"));
        assert!(requirements.contains("REQ-2"));
    }

    #[test]
    fn test_get_all_linked_tasks() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        orchestrator
            .link_task_to_requirements("task-2".to_string(), vec!["REQ-2".to_string()])
            .unwrap();

        let tasks = orchestrator.get_all_linked_tasks();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.contains("task-1"));
        assert!(tasks.contains("task-2"));
    }

    #[test]
    fn test_reset() {
        let mut orchestrator = WorkflowOrchestrator::new();

        orchestrator
            .link_task_to_requirements("task-1".to_string(), vec!["REQ-1".to_string()])
            .unwrap();

        orchestrator.reset();

        assert_eq!(orchestrator.get_all_linked_tasks().len(), 0);
        assert_eq!(orchestrator.get_all_linked_requirements().len(), 0);
    }
}
