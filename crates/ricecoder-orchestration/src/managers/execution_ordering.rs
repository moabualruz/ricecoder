//! Execution ordering and parallelization strategies

use std::collections::{HashMap, HashSet};

use crate::{
    analyzers::DependencyGraph,
    error::{OrchestrationError, Result},
    models::Project,
};

/// Represents a level in the execution hierarchy
#[derive(Debug, Clone)]
pub struct ExecutionLevel {
    /// Projects that can be executed at this level
    pub projects: Vec<String>,

    /// Level number (0 is first)
    pub level: usize,
}

/// Strategy for determining parallelization points
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelizationStrategy {
    /// Execute everything sequentially
    Sequential,

    /// Execute projects at the same level in parallel
    LevelBased,

    /// Execute all projects in parallel (no dependency ordering)
    FullParallel,
}

/// Execution plan for a batch of projects
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Execution levels (each level can be parallelized)
    pub levels: Vec<ExecutionLevel>,

    /// Total number of projects
    pub total_projects: usize,

    /// Parallelization strategy used
    pub strategy: ParallelizationStrategy,

    /// Maximum parallelism possible
    pub max_parallelism: usize,
}

/// Determines execution order and parallelization points
pub struct ExecutionOrderer {
    graph: DependencyGraph,
}

impl ExecutionOrderer {
    /// Creates a new execution orderer
    pub fn new(graph: DependencyGraph) -> Self {
        Self { graph }
    }

    /// Determines the execution order for a set of projects
    pub fn determine_order(&self, projects: &[Project]) -> Result<Vec<String>> {
        // Use level-based planning to determine order
        let plan = self.create_level_based_plan(projects)?;

        // Flatten the levels into a single execution order
        let mut order = Vec::new();
        for level in plan.levels {
            order.extend(level.projects);
        }

        Ok(order)
    }

    /// Creates an execution plan with parallelization points
    pub fn create_execution_plan(
        &self,
        projects: &[Project],
        strategy: ParallelizationStrategy,
    ) -> Result<ExecutionPlan> {
        match strategy {
            ParallelizationStrategy::Sequential => self.create_sequential_plan(projects),
            ParallelizationStrategy::LevelBased => self.create_level_based_plan(projects),
            ParallelizationStrategy::FullParallel => self.create_full_parallel_plan(projects),
        }
    }

    /// Creates a sequential execution plan
    fn create_sequential_plan(&self, projects: &[Project]) -> Result<ExecutionPlan> {
        let order = self.determine_order(projects)?;

        let levels = vec![ExecutionLevel {
            projects: order,
            level: 0,
        }];

        Ok(ExecutionPlan {
            levels,
            total_projects: projects.len(),
            strategy: ParallelizationStrategy::Sequential,
            max_parallelism: 1,
        })
    }

    /// Creates a level-based execution plan (projects at same level can run in parallel)
    fn create_level_based_plan(&self, projects: &[Project]) -> Result<ExecutionPlan> {
        let project_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();

        // Calculate in-degrees for each project
        // In-degree = number of projects this project depends on
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for name in &project_names {
            in_degree.insert(name.clone(), 0);
        }

        // Count how many projects each project depends on (within the batch)
        for name in &project_names {
            let deps = self.graph.get_dependencies(name);
            let batch_deps = deps.iter().filter(|d| project_names.contains(d)).count();
            in_degree.insert(name.clone(), batch_deps);
        }

        // Build levels using Kahn's algorithm
        let mut levels = Vec::new();
        let mut processed = HashSet::new();
        let mut current_in_degree = in_degree.clone();

        while processed.len() < project_names.len() {
            // Find all projects with in-degree 0 (no unprocessed dependencies)
            let current_level: Vec<String> = project_names
                .iter()
                .filter(|name| {
                    !processed.contains(*name)
                        && current_in_degree.get(*name).copied().unwrap_or(0) == 0
                })
                .cloned()
                .collect();

            if current_level.is_empty() {
                return Err(OrchestrationError::CircularDependency(
                    "Circular dependency detected during level-based planning".to_string(),
                ));
            }

            levels.push(ExecutionLevel {
                projects: current_level.clone(),
                level: levels.len(),
            });

            // Mark these projects as processed and update in-degrees
            for project in &current_level {
                processed.insert(project.clone());

                // Reduce in-degree for projects that depend on this one
                let dependents = self.graph.get_dependents(project);
                for dependent in dependents {
                    if project_names.contains(&dependent) && !processed.contains(&dependent) {
                        if let Some(degree) = current_in_degree.get_mut(&dependent) {
                            *degree = degree.saturating_sub(1);
                        }
                    }
                }
            }
        }

        let max_parallelism = levels.iter().map(|l| l.projects.len()).max().unwrap_or(1);

        Ok(ExecutionPlan {
            levels,
            total_projects: projects.len(),
            strategy: ParallelizationStrategy::LevelBased,
            max_parallelism,
        })
    }

    /// Creates a full parallel execution plan (no dependency ordering)
    fn create_full_parallel_plan(&self, projects: &[Project]) -> Result<ExecutionPlan> {
        let project_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();

        let levels = vec![ExecutionLevel {
            projects: project_names.clone(),
            level: 0,
        }];

        Ok(ExecutionPlan {
            levels,
            total_projects: projects.len(),
            strategy: ParallelizationStrategy::FullParallel,
            max_parallelism: projects.len(),
        })
    }

    /// Determines safe parallelization points
    pub fn find_parallelization_points(&self, projects: &[Project]) -> Result<Vec<Vec<String>>> {
        let plan = self.create_level_based_plan(projects)?;

        Ok(plan
            .levels
            .into_iter()
            .map(|level| level.projects)
            .collect())
    }

    /// Handles execution failures and determines rollback strategy
    pub fn plan_rollback(
        &self,
        failed_project: &str,
        execution_order: &[String],
    ) -> Result<Vec<String>> {
        // Find all projects that depend on the failed project
        let dependents = self.graph.get_dependents(failed_project);

        // Filter to only those that were executed after the failed project
        let failed_idx = execution_order
            .iter()
            .position(|p| p == failed_project)
            .ok_or_else(|| {
                OrchestrationError::BatchExecutionFailed(format!(
                    "Project {} not found in execution order",
                    failed_project
                ))
            })?;

        let projects_to_rollback: Vec<String> = execution_order
            .iter()
            .enumerate()
            .filter(|(idx, name)| {
                *idx > failed_idx
                    && (dependents.contains(name)
                        || self.is_transitive_dependent(name, failed_project))
            })
            .map(|(_, name)| name.clone())
            .collect();

        // Reverse the order for rollback
        let mut rollback_order = projects_to_rollback;
        rollback_order.reverse();

        Ok(rollback_order)
    }

    /// Checks if a project is transitively dependent on another
    fn is_transitive_dependent(&self, dependent: &str, project: &str) -> bool {
        self.graph.can_reach(dependent, project)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::models::{DependencyType, ProjectStatus};

    fn create_test_project(name: &str) -> Project {
        Project {
            path: PathBuf::from(format!("/path/to/{}", name)),
            name: name.to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        }
    }

    #[test]
    fn test_sequential_plan() {
        let mut graph = DependencyGraph::new(false);
        let project_a = create_test_project("project-a");
        let project_b = create_test_project("project-b");

        graph.add_project(project_a.clone()).unwrap();
        graph.add_project(project_b.clone()).unwrap();

        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let plan = orderer
            .create_execution_plan(&[project_a, project_b], ParallelizationStrategy::Sequential)
            .unwrap();

        assert_eq!(plan.levels.len(), 1);
        assert_eq!(plan.max_parallelism, 1);
        assert_eq!(plan.strategy, ParallelizationStrategy::Sequential);
    }

    #[test]
    fn test_level_based_plan() {
        let mut graph = DependencyGraph::new(false);
        let project_a = create_test_project("project-a");
        let project_b = create_test_project("project-b");
        let project_c = create_test_project("project-c");

        graph.add_project(project_a.clone()).unwrap();
        graph.add_project(project_b.clone()).unwrap();
        graph.add_project(project_c.clone()).unwrap();

        // B -> A, C -> A (B and C depend on A, so A executes first)
        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-c".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let plan = orderer
            .create_execution_plan(
                &[project_a, project_b, project_c],
                ParallelizationStrategy::LevelBased,
            )
            .unwrap();

        assert_eq!(plan.levels.len(), 2);
        assert_eq!(plan.levels[0].projects.len(), 1); // A
        assert_eq!(plan.levels[1].projects.len(), 2); // B and C
        assert_eq!(plan.max_parallelism, 2);
    }

    #[test]
    fn test_full_parallel_plan() {
        let mut graph = DependencyGraph::new(false);
        let project_a = create_test_project("project-a");
        let project_b = create_test_project("project-b");

        graph.add_project(project_a.clone()).unwrap();
        graph.add_project(project_b.clone()).unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let plan = orderer
            .create_execution_plan(
                &[project_a, project_b],
                ParallelizationStrategy::FullParallel,
            )
            .unwrap();

        assert_eq!(plan.levels.len(), 1);
        assert_eq!(plan.levels[0].projects.len(), 2);
        assert_eq!(plan.max_parallelism, 2);
    }

    #[test]
    fn test_determine_order() {
        let mut graph = DependencyGraph::new(false);
        let project_a = create_test_project("project-a");
        let project_b = create_test_project("project-b");
        let project_c = create_test_project("project-c");

        graph.add_project(project_a.clone()).unwrap();
        graph.add_project(project_b.clone()).unwrap();
        graph.add_project(project_c.clone()).unwrap();

        // B -> A, C -> B (C depends on B, B depends on A)
        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-c".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let order = orderer
            .determine_order(&[project_a, project_b, project_c])
            .unwrap();

        assert_eq!(order.len(), 3);
        let a_idx = order.iter().position(|x| x == "project-a").unwrap();
        let b_idx = order.iter().position(|x| x == "project-b").unwrap();
        let c_idx = order.iter().position(|x| x == "project-c").unwrap();

        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn test_find_parallelization_points() {
        let mut graph = DependencyGraph::new(false);
        let project_a = create_test_project("project-a");
        let project_b = create_test_project("project-b");
        let project_c = create_test_project("project-c");

        graph.add_project(project_a.clone()).unwrap();
        graph.add_project(project_b.clone()).unwrap();
        graph.add_project(project_c.clone()).unwrap();

        // B -> A, C -> A (B and C depend on A)
        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-c".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let points = orderer
            .find_parallelization_points(&[project_a, project_b, project_c])
            .unwrap();

        assert_eq!(points.len(), 2);
        assert_eq!(points[0].len(), 1); // A
        assert_eq!(points[1].len(), 2); // B and C
    }

    #[test]
    fn test_plan_rollback() {
        let mut graph = DependencyGraph::new(false);
        let project_a = create_test_project("project-a");
        let project_b = create_test_project("project-b");
        let project_c = create_test_project("project-c");

        graph.add_project(project_a.clone()).unwrap();
        graph.add_project(project_b.clone()).unwrap();
        graph.add_project(project_c.clone()).unwrap();

        // B -> A, C -> B (C depends on B, B depends on A)
        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(crate::models::ProjectDependency {
                from: "project-c".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let execution_order = vec![
            "project-a".to_string(),
            "project-b".to_string(),
            "project-c".to_string(),
        ];

        let rollback_order = orderer
            .plan_rollback("project-b", &execution_order)
            .unwrap();

        // Should rollback C first (it depends on B)
        assert_eq!(rollback_order.len(), 1);
        assert_eq!(rollback_order[0], "project-c");
    }
}
