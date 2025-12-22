//! Dependency resolution for workflow steps

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    error::{WorkflowError, WorkflowResult},
    models::Workflow,
};

/// Resolves step dependencies and builds execution order
///
/// Handles:
/// - Building execution order from dependency graph
/// - Detecting and reporting circular dependencies
/// - Waiting for dependencies before executing step
pub struct DependencyResolver;

impl DependencyResolver {
    /// Build execution order from dependency graph
    ///
    /// Uses topological sort to determine the order in which steps should execute
    /// based on their dependencies. Returns error if circular dependencies are detected.
    pub fn resolve_execution_order(workflow: &Workflow) -> WorkflowResult<Vec<String>> {
        Self::topological_sort(workflow)
    }

    /// Perform topological sort on workflow steps
    ///
    /// Builds a valid execution order where all dependencies are satisfied
    /// before a step is executed.
    fn topological_sort(workflow: &Workflow) -> WorkflowResult<Vec<String>> {
        let mut order = Vec::new();
        let mut completed = HashSet::new();
        let mut queue = VecDeque::new();

        // Find all steps with no dependencies
        for step in &workflow.steps {
            if step.dependencies.is_empty() {
                queue.push_back(step.id.clone());
            }
        }

        // Build step map for quick lookup
        let step_map: HashMap<_, _> = workflow.steps.iter().map(|s| (&s.id, s)).collect();

        // Process queue
        while let Some(step_id) = queue.pop_front() {
            if completed.contains(&step_id) {
                continue;
            }

            // Check if all dependencies are completed
            if let Some(step) = step_map.get(&step_id) {
                let all_deps_completed =
                    step.dependencies.iter().all(|dep| completed.contains(dep));

                if all_deps_completed {
                    order.push(step_id.clone());
                    completed.insert(step_id.clone());

                    // Add steps that depend on this one
                    for other_step in &workflow.steps {
                        if other_step.dependencies.contains(&step_id)
                            && !completed.contains(&other_step.id)
                        {
                            queue.push_back(other_step.id.clone());
                        }
                    }
                } else {
                    // Re-queue if dependencies not met
                    queue.push_back(step_id);
                }
            }
        }

        if order.len() != workflow.steps.len() {
            return Err(WorkflowError::Invalid(
                "Could not determine execution order for all steps".to_string(),
            ));
        }

        Ok(order)
    }

    /// Detect circular dependencies in workflow
    ///
    /// Uses depth-first search to detect cycles in the dependency graph.
    /// Returns error if any circular dependency is found.
    pub fn detect_circular_dependencies(workflow: &Workflow) -> WorkflowResult<()> {
        let step_map: HashMap<&String, &crate::models::WorkflowStep> =
            workflow.steps.iter().map(|s| (&s.id, s)).collect();

        // For each step, perform DFS to detect cycles
        for start_step in &workflow.steps {
            let mut visited = HashSet::new();
            let mut rec_stack = HashSet::new();

            Self::dfs_detect_cycle(&step_map, &start_step.id, &mut visited, &mut rec_stack)?;
        }

        Ok(())
    }

    /// Depth-first search to detect cycles
    fn dfs_detect_cycle(
        step_map: &HashMap<&String, &crate::models::WorkflowStep>,
        step_id: &String,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> WorkflowResult<()> {
        visited.insert(step_id.clone());
        rec_stack.insert(step_id.clone());

        if let Some(step) = step_map.get(step_id) {
            for dep in &step.dependencies {
                if !visited.contains(dep) {
                    Self::dfs_detect_cycle(step_map, dep, visited, rec_stack)?;
                } else if rec_stack.contains(dep) {
                    return Err(WorkflowError::Invalid(format!(
                        "Circular dependency detected: {} -> {}",
                        step_id, dep
                    )));
                }
            }
        }

        rec_stack.remove(step_id);
        Ok(())
    }

    /// Get all dependencies for a step (transitive closure)
    ///
    /// Returns all steps that must be completed before the given step can execute,
    /// including transitive dependencies.
    pub fn get_all_dependencies(
        workflow: &Workflow,
        step_id: &str,
    ) -> WorkflowResult<HashSet<String>> {
        let mut all_deps = HashSet::new();
        let mut queue = VecDeque::new();

        // Find the step
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Add direct dependencies to queue
        for dep in &step.dependencies {
            queue.push_back(dep.clone());
        }

        // Build step map
        let step_map: HashMap<_, _> = workflow.steps.iter().map(|s| (&s.id, s)).collect();

        // Process queue to find transitive dependencies
        while let Some(dep_id) = queue.pop_front() {
            if all_deps.contains(&dep_id) {
                continue;
            }

            all_deps.insert(dep_id.clone());

            // Add dependencies of this dependency
            if let Some(dep_step) = step_map.get(&dep_id) {
                for transitive_dep in &dep_step.dependencies {
                    if !all_deps.contains(transitive_dep) {
                        queue.push_back(transitive_dep.clone());
                    }
                }
            }
        }

        Ok(all_deps)
    }

    /// Get all steps that depend on a given step (reverse dependencies)
    ///
    /// Returns all steps that have the given step as a dependency (direct or transitive).
    pub fn get_dependent_steps(
        workflow: &Workflow,
        step_id: &str,
    ) -> WorkflowResult<HashSet<String>> {
        let mut dependents = HashSet::new();

        // Find all steps that directly depend on this step
        for step in &workflow.steps {
            if step.dependencies.contains(&step_id.to_string()) {
                dependents.insert(step.id.clone());

                // Recursively find steps that depend on these steps
                if let Ok(transitive) = Self::get_dependent_steps(workflow, &step.id) {
                    dependents.extend(transitive);
                }
            }
        }

        Ok(dependents)
    }

    /// Check if a step can be executed
    ///
    /// A step can be executed if all its dependencies have been completed.
    pub fn can_execute_step(
        workflow: &Workflow,
        completed_steps: &[String],
        step_id: &str,
    ) -> WorkflowResult<bool> {
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Check if all dependencies are completed
        for dep in &step.dependencies {
            if !completed_steps.contains(dep) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get steps that are ready to execute
    ///
    /// Returns all steps whose dependencies are satisfied and haven't been executed yet.
    pub fn get_ready_steps(
        workflow: &Workflow,
        completed_steps: &[String],
        in_progress_steps: &[String],
    ) -> WorkflowResult<Vec<String>> {
        let mut ready = Vec::new();

        for step in &workflow.steps {
            // Skip if already completed or in progress
            if completed_steps.contains(&step.id) || in_progress_steps.contains(&step.id) {
                continue;
            }

            // Check if all dependencies are completed
            if Self::can_execute_step(workflow, completed_steps, &step.id)? {
                ready.push(step.id.clone());
            }
        }

        Ok(ready)
    }

    /// Validate dependency graph
    ///
    /// Checks for:
    /// - Missing dependencies (references to non-existent steps)
    /// - Circular dependencies
    /// - Duplicate step IDs
    pub fn validate_dependencies(workflow: &Workflow) -> WorkflowResult<()> {
        // Check for duplicate step IDs
        let mut step_ids = HashSet::new();
        for step in &workflow.steps {
            if !step_ids.insert(&step.id) {
                return Err(WorkflowError::Invalid(format!(
                    "Duplicate step id: {}",
                    step.id
                )));
            }
        }

        // Check for missing dependencies
        for step in &workflow.steps {
            for dep in &step.dependencies {
                if !step_ids.contains(dep) {
                    return Err(WorkflowError::Invalid(format!(
                        "Step {} depends on non-existent step {}",
                        step.id, dep
                    )));
                }
            }
        }

        // Check for circular dependencies
        Self::detect_circular_dependencies(workflow)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AgentStep, ErrorAction, RiskFactors, StepConfig, StepType, WorkflowConfig, WorkflowStep,
    };

    fn create_workflow_with_deps() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![
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
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
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
                    dependencies: vec!["step1".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
                WorkflowStep {
                    id: "step3".to_string(),
                    name: "Step 3".to_string(),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["step1".to_string(), "step2".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
            ],
            config: WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        }
    }

    #[test]
    fn test_resolve_execution_order() {
        let workflow = create_workflow_with_deps();
        let order = DependencyResolver::resolve_execution_order(&workflow).unwrap();

        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "step1");
        assert_eq!(order[1], "step2");
        assert_eq!(order[2], "step3");
    }

    #[test]
    fn test_detect_circular_dependency() {
        let mut workflow = create_workflow_with_deps();
        // Create a circular dependency: step1 -> step2 -> step1
        workflow.steps[0].dependencies.push("step2".to_string());

        let result = DependencyResolver::detect_circular_dependencies(&workflow);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_dependencies() {
        let workflow = create_workflow_with_deps();
        let deps = DependencyResolver::get_all_dependencies(&workflow, "step3").unwrap();

        assert_eq!(deps.len(), 2);
        assert!(deps.contains("step1"));
        assert!(deps.contains("step2"));
    }

    #[test]
    fn test_get_dependent_steps() {
        let workflow = create_workflow_with_deps();
        let dependents = DependencyResolver::get_dependent_steps(&workflow, "step1").unwrap();

        assert!(dependents.contains("step2"));
        assert!(dependents.contains("step3"));
    }

    #[test]
    fn test_can_execute_step() {
        let workflow = create_workflow_with_deps();

        // step1 can execute (no dependencies)
        assert!(DependencyResolver::can_execute_step(&workflow, &[], "step1").unwrap());

        // step2 cannot execute (depends on step1)
        assert!(!DependencyResolver::can_execute_step(&workflow, &[], "step2").unwrap());

        // step2 can execute after step1 is completed
        assert!(
            DependencyResolver::can_execute_step(&workflow, &["step1".to_string()], "step2")
                .unwrap()
        );
    }

    #[test]
    fn test_get_ready_steps() {
        let workflow = create_workflow_with_deps();

        // Initially, only step1 is ready
        let ready = DependencyResolver::get_ready_steps(&workflow, &[], &[]).unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], "step1");

        // After step1 completes, step2 is ready
        let ready =
            DependencyResolver::get_ready_steps(&workflow, &["step1".to_string()], &[]).unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], "step2");

        // After step1 and step2 complete, step3 is ready
        let ready = DependencyResolver::get_ready_steps(
            &workflow,
            &["step1".to_string(), "step2".to_string()],
            &[],
        )
        .unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], "step3");
    }

    #[test]
    fn test_validate_dependencies() {
        let workflow = create_workflow_with_deps();
        let result = DependencyResolver::validate_dependencies(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_missing_dependency() {
        let mut workflow = create_workflow_with_deps();
        workflow.steps[1]
            .dependencies
            .push("non-existent".to_string());

        let result = DependencyResolver::validate_dependencies(&workflow);
        assert!(result.is_err());
    }
}
