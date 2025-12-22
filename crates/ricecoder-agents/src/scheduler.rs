//! Agent scheduler for managing execution order and parallelism

use std::collections::{HashMap, HashSet};

use crate::{
    error::{AgentError, Result},
    models::AgentTask,
};

/// Execution schedule for agents
#[derive(Debug, Clone)]
pub struct ExecutionSchedule {
    /// Ordered list of execution phases
    pub phases: Vec<ExecutionPhase>,
}

/// A phase of execution (can contain parallel tasks)
#[derive(Debug, Clone)]
pub struct ExecutionPhase {
    /// Tasks to execute in parallel in this phase
    pub tasks: Vec<AgentTask>,
}

/// Task dependency information
#[derive(Debug, Clone)]
pub struct TaskDependency {
    /// Task ID
    pub task_id: String,
    /// IDs of tasks this task depends on
    pub depends_on: Vec<String>,
}

/// Directed Acyclic Graph (DAG) for task execution
#[derive(Debug, Clone)]
pub struct TaskDAG {
    /// Map of task ID to its dependencies
    pub dependencies: HashMap<String, Vec<String>>,
    /// Map of task ID to tasks that depend on it
    pub dependents: HashMap<String, Vec<String>>,
    /// All tasks in the DAG
    pub tasks: HashMap<String, AgentTask>,
}

impl TaskDAG {
    /// Create a new empty DAG
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Add a task to the DAG
    pub fn add_task(&mut self, task: AgentTask) {
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);
        self.dependencies.entry(task_id.clone()).or_default();
        self.dependents.entry(task_id).or_default();
    }

    /// Add a dependency between tasks
    pub fn add_dependency(&mut self, task_id: String, depends_on: String) {
        self.dependencies
            .entry(task_id.clone())
            .or_default()
            .push(depends_on.clone());

        self.dependents.entry(depends_on).or_default().push(task_id);
    }

    /// Get tasks with no dependencies (can execute immediately)
    pub fn get_root_tasks(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get tasks that depend on a given task
    pub fn get_dependents(&self, task_id: &str) -> Vec<String> {
        self.dependents.get(task_id).cloned().unwrap_or_default()
    }

    /// Get dependencies for a task
    pub fn get_dependencies(&self, task_id: &str) -> Vec<String> {
        self.dependencies.get(task_id).cloned().unwrap_or_default()
    }
}

impl Default for TaskDAG {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent scheduler for managing execution order and parallelism
pub struct AgentScheduler;

impl AgentScheduler {
    /// Create a new agent scheduler
    pub fn new() -> Self {
        Self
    }

    /// Create an execution schedule from tasks
    pub fn schedule(&self, tasks: &[AgentTask]) -> Result<ExecutionSchedule> {
        // Build a DAG from tasks (currently no explicit dependencies)
        let mut dag = TaskDAG::new();
        for task in tasks {
            dag.add_task(task.clone());
        }

        // Detect circular dependencies
        self.detect_circular_dependencies_in_dag(&dag)?;

        // Create execution phases based on dependencies
        let phases = self.create_execution_phases(&dag)?;

        Ok(ExecutionSchedule { phases })
    }

    /// Resolve task dependencies and create a DAG
    pub fn resolve_dependencies(&self, tasks: &[AgentTask]) -> Result<TaskDAG> {
        let mut dag = TaskDAG::new();

        // Add all tasks to the DAG
        for task in tasks {
            dag.add_task(task.clone());
        }

        // Currently, tasks have no explicit dependencies
        // This can be enhanced to parse dependencies from task options

        Ok(dag)
    }

    /// Detect circular dependencies in the DAG
    pub fn detect_circular_dependencies(&self, tasks: &[AgentTask]) -> Result<()> {
        let dag = self.resolve_dependencies(tasks)?;
        self.detect_circular_dependencies_in_dag(&dag)
    }

    /// Detect circular dependencies in a DAG using DFS
    fn detect_circular_dependencies_in_dag(&self, dag: &TaskDAG) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task_id in dag.tasks.keys() {
            if !visited.contains(task_id) {
                self.dfs_detect_cycle(task_id, dag, &mut visited, &mut rec_stack)?;
            }
        }

        Ok(())
    }

    /// DFS helper to detect cycles
    #[allow(clippy::only_used_in_recursion)]
    fn dfs_detect_cycle(
        &self,
        task_id: &str,
        dag: &TaskDAG,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<()> {
        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        let dependencies = dag.get_dependencies(task_id);
        for dep_id in dependencies {
            if !visited.contains(&dep_id) {
                self.dfs_detect_cycle(&dep_id, dag, visited, rec_stack)?;
            } else if rec_stack.contains(&dep_id) {
                return Err(AgentError::invalid_input(format!(
                    "Circular dependency detected: {} -> {}",
                    task_id, dep_id
                )));
            }
        }

        rec_stack.remove(task_id);
        Ok(())
    }

    /// Create execution phases from a DAG
    fn create_execution_phases(&self, dag: &TaskDAG) -> Result<Vec<ExecutionPhase>> {
        let mut phases = Vec::new();
        let mut completed = HashSet::new();
        let mut remaining: HashSet<String> = dag.tasks.keys().cloned().collect();

        while !remaining.is_empty() {
            // Find tasks that can execute in this phase (all dependencies completed)
            let mut phase_tasks = Vec::new();

            for task_id in remaining.iter() {
                let dependencies = dag.get_dependencies(task_id);
                if dependencies.iter().all(|dep| completed.contains(dep)) {
                    phase_tasks.push(task_id.clone());
                }
            }

            if phase_tasks.is_empty() {
                // This shouldn't happen if circular dependency detection worked
                return Err(AgentError::invalid_input(
                    "Unable to create execution phases: no executable tasks found".to_string(),
                ));
            }

            // Create phase with tasks that can execute in parallel
            let phase = ExecutionPhase {
                tasks: phase_tasks
                    .iter()
                    .filter_map(|id| dag.tasks.get(id).cloned())
                    .collect(),
            };

            phases.push(phase);

            // Mark tasks as completed
            for task_id in phase_tasks {
                completed.insert(task_id.clone());
                remaining.remove(&task_id);
            }
        }

        Ok(phases)
    }
}

impl Default for AgentScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::models::{TaskOptions, TaskScope, TaskTarget, TaskType};

    fn create_test_task(id: &str) -> AgentTask {
        AgentTask {
            id: id.to_string(),
            task_type: TaskType::CodeReview,
            target: TaskTarget {
                files: vec![PathBuf::from("test.rs")],
                scope: TaskScope::File,
            },
            options: TaskOptions::default(),
        }
    }

    #[test]
    fn test_schedule_single_task() {
        let scheduler = AgentScheduler::new();
        let tasks = vec![create_test_task("task1")];

        let schedule = scheduler.schedule(&tasks).unwrap();
        assert_eq!(schedule.phases.len(), 1);
        assert_eq!(schedule.phases[0].tasks.len(), 1);
        assert_eq!(schedule.phases[0].tasks[0].id, "task1");
    }

    #[test]
    fn test_schedule_multiple_tasks() {
        let scheduler = AgentScheduler::new();
        let tasks = vec![
            create_test_task("task1"),
            create_test_task("task2"),
            create_test_task("task3"),
        ];

        let schedule = scheduler.schedule(&tasks).unwrap();
        assert_eq!(schedule.phases.len(), 1);
        assert_eq!(schedule.phases[0].tasks.len(), 3);
    }

    #[test]
    fn test_resolve_dependencies() {
        let scheduler = AgentScheduler::new();
        let tasks = vec![create_test_task("task1"), create_test_task("task2")];

        let dag = scheduler.resolve_dependencies(&tasks).unwrap();
        assert_eq!(dag.tasks.len(), 2);
        assert!(dag.tasks.contains_key("task1"));
        assert!(dag.tasks.contains_key("task2"));
    }

    #[test]
    fn test_detect_circular_dependencies() {
        let scheduler = AgentScheduler::new();
        let tasks = vec![create_test_task("task1")];

        let result = scheduler.detect_circular_dependencies(&tasks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_task_dag_add_task() {
        let mut dag = TaskDAG::new();
        let task = create_test_task("task1");

        dag.add_task(task.clone());

        assert_eq!(dag.tasks.len(), 1);
        assert!(dag.tasks.contains_key("task1"));
        assert!(dag.dependencies.contains_key("task1"));
        assert!(dag.dependents.contains_key("task1"));
    }

    #[test]
    fn test_task_dag_add_dependency() {
        let mut dag = TaskDAG::new();
        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));

        dag.add_dependency("task2".to_string(), "task1".to_string());

        assert_eq!(dag.get_dependencies("task2"), vec!["task1"]);
        assert_eq!(dag.get_dependents("task1"), vec!["task2"]);
    }

    #[test]
    fn test_task_dag_get_root_tasks() {
        let mut dag = TaskDAG::new();
        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));
        dag.add_task(create_test_task("task3"));

        dag.add_dependency("task2".to_string(), "task1".to_string());
        dag.add_dependency("task3".to_string(), "task1".to_string());

        let root_tasks = dag.get_root_tasks();
        assert_eq!(root_tasks.len(), 1);
        assert_eq!(root_tasks[0], "task1");
    }

    #[test]
    fn test_task_dag_multiple_root_tasks() {
        let mut dag = TaskDAG::new();
        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));
        dag.add_task(create_test_task("task3"));

        dag.add_dependency("task3".to_string(), "task1".to_string());

        let root_tasks = dag.get_root_tasks();
        assert_eq!(root_tasks.len(), 2);
        assert!(root_tasks.contains(&"task1".to_string()));
        assert!(root_tasks.contains(&"task2".to_string()));
    }

    #[test]
    fn test_create_execution_phases_linear_dependency() {
        let scheduler = AgentScheduler::new();
        let mut dag = TaskDAG::new();

        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));
        dag.add_task(create_test_task("task3"));

        dag.add_dependency("task2".to_string(), "task1".to_string());
        dag.add_dependency("task3".to_string(), "task2".to_string());

        let phases = scheduler.create_execution_phases(&dag).unwrap();

        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0].tasks.len(), 1);
        assert_eq!(phases[0].tasks[0].id, "task1");
        assert_eq!(phases[1].tasks.len(), 1);
        assert_eq!(phases[1].tasks[0].id, "task2");
        assert_eq!(phases[2].tasks.len(), 1);
        assert_eq!(phases[2].tasks[0].id, "task3");
    }

    #[test]
    fn test_create_execution_phases_parallel_tasks() {
        let scheduler = AgentScheduler::new();
        let mut dag = TaskDAG::new();

        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));
        dag.add_task(create_test_task("task3"));

        dag.add_dependency("task3".to_string(), "task1".to_string());
        dag.add_dependency("task3".to_string(), "task2".to_string());

        let phases = scheduler.create_execution_phases(&dag).unwrap();

        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].tasks.len(), 2);
        assert_eq!(phases[1].tasks.len(), 1);
        assert_eq!(phases[1].tasks[0].id, "task3");
    }

    #[test]
    fn test_detect_circular_dependency_simple() {
        let scheduler = AgentScheduler::new();
        let mut dag = TaskDAG::new();

        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));

        dag.add_dependency("task1".to_string(), "task2".to_string());
        dag.add_dependency("task2".to_string(), "task1".to_string());

        let result = scheduler.detect_circular_dependencies_in_dag(&dag);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_detect_circular_dependency_self_loop() {
        let scheduler = AgentScheduler::new();
        let mut dag = TaskDAG::new();

        dag.add_task(create_test_task("task1"));
        dag.add_dependency("task1".to_string(), "task1".to_string());

        let result = scheduler.detect_circular_dependencies_in_dag(&dag);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_circular_dependency_complex() {
        let scheduler = AgentScheduler::new();
        let mut dag = TaskDAG::new();

        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));
        dag.add_task(create_test_task("task3"));
        dag.add_task(create_test_task("task4"));

        dag.add_dependency("task2".to_string(), "task1".to_string());
        dag.add_dependency("task3".to_string(), "task2".to_string());
        dag.add_dependency("task1".to_string(), "task3".to_string()); // Creates cycle

        let result = scheduler.detect_circular_dependencies_in_dag(&dag);
        assert!(result.is_err());
    }

    #[test]
    fn test_schedule_with_no_tasks() {
        let scheduler = AgentScheduler::new();
        let tasks: Vec<AgentTask> = vec![];

        let schedule = scheduler.schedule(&tasks).unwrap();
        assert_eq!(schedule.phases.len(), 0);
    }

    #[test]
    fn test_task_dag_default() {
        let dag = TaskDAG::default();
        assert!(dag.tasks.is_empty());
        assert!(dag.dependencies.is_empty());
        assert!(dag.dependents.is_empty());
    }

    #[test]
    fn test_scheduler_default() {
        let _scheduler = AgentScheduler::default();
        // Just verify it can be created with default
    }
}
