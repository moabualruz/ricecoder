//! Property-based tests for batch execution order
//! **Feature: ricecoder-orchestration, Property 3: Batch Execution Order**
//! **Validates: Requirements 2.1**

use proptest::prelude::*;
use ricecoder_orchestration::{
    BatchExecutor, DependencyGraph, DependencyType, ExecutionOrderer, ParallelizationStrategy,
    Project, ProjectDependency, ProjectStatus,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

/// Strategy for generating random projects
fn project_name_strategy() -> impl Strategy<Value = String> {
    "project-[a-z]{1,10}"
}

/// Strategy for generating random project counts
fn project_count_strategy() -> impl Strategy<Value = usize> {
    1..20usize
}

/// Strategy for generating random dependency configurations
fn dependency_probability_strategy() -> impl Strategy<Value = f64> {
    0.0..0.5f64
}

/// Creates a test project with the given name
fn create_test_project(name: &str) -> Project {
    Project {
        path: PathBuf::from(format!("/path/to/{}", name)),
        name: name.to_string(),
        project_type: "rust".to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    }
}

/// Generates a random dependency graph with projects
fn generate_dependency_graph(
    project_count: usize,
    dependency_probability: f64,
) -> (DependencyGraph, Vec<Project>) {
    let mut graph = DependencyGraph::new(false);
    let mut projects = Vec::new();

    // Create projects
    for i in 0..project_count {
        let project = create_test_project(&format!("project-{}", i));
        graph.add_project(project.clone()).unwrap();
        projects.push(project);
    }

    // Add random dependencies (avoiding cycles)
    // We add dependencies from higher-indexed projects to lower-indexed ones
    // This ensures no cycles since we always go from higher to lower
    for i in 0..project_count {
        for j in 0..i {
            if rand::random::<f64>() < dependency_probability {
                let dep = ProjectDependency {
                    from: format!("project-{}", i),
                    to: format!("project-{}", j),
                    dependency_type: DependencyType::Direct,
                    version_constraint: "^0.1.0".to_string(),
                };

                // This should never fail since we're going from higher to lower indices
                let _ = graph.add_dependency(dep);
            }
        }
    }

    (graph, projects)
}

proptest! {
    /// Property: For any batch operation on projects with dependencies,
    /// the level-based execution plan SHALL respect dependency order such that
    /// all dependencies are processed before dependent projects.
    #[test]
    fn prop_batch_execution_respects_dependency_order(
        project_count in project_count_strategy(),
        dependency_probability in dependency_probability_strategy(),
    ) {
        // Generate random dependency graph
        let (graph, projects) = generate_dependency_graph(project_count, dependency_probability);

        // Create execution orderer
        let orderer = ExecutionOrderer::new(graph.clone());

        // Create level-based execution plan
        let plan = orderer
            .create_execution_plan(&projects, ParallelizationStrategy::LevelBased)
            .unwrap();

        // Flatten the levels into a single execution order
        let mut order = Vec::new();
        for level in &plan.levels {
            order.extend(level.projects.clone());
        }

        // Verify all projects are included
        prop_assert_eq!(order.len(), projects.len(), "Not all projects were included in execution order");

        // Verify no duplicates
        let unique_projects: HashSet<_> = order.iter().cloned().collect();
        prop_assert_eq!(unique_projects.len(), order.len(), "Duplicate projects in execution order");

        // Verify dependency order is respected
        // For each project, all projects it depends on must be executed before it
        for i in 0..order.len() {
            let current_project = &order[i];
            let dependencies = graph.get_dependencies(current_project);

            for dep in dependencies {
                // Find the position of the dependency in the execution order
                if let Some(dep_idx) = order.iter().position(|p| p == &dep) {
                    // If current_project depends on dep, then dep must execute BEFORE current
                    // So dep_idx < i
                    prop_assert!(
                        dep_idx < i,
                        "Project {} depends on {} but {} is executed after {}",
                        current_project,
                        dep,
                        dep,
                        current_project
                    );
                }
            }
        }
    }

    /// Property: For any batch operation, all projects in the batch
    /// SHALL be executed exactly once.
    #[test]
    fn prop_batch_execution_includes_all_projects(
        project_count in project_count_strategy(),
        dependency_probability in dependency_probability_strategy(),
    ) {
        let (graph, projects) = generate_dependency_graph(project_count, dependency_probability);
        let orderer = ExecutionOrderer::new(graph);

        let order = orderer.determine_order(&projects).unwrap();

        // All projects should be in the execution order
        let project_names: HashSet<_> = projects.iter().map(|p| p.name.clone()).collect();
        let executed_names: HashSet<_> = order.iter().cloned().collect();

        prop_assert_eq!(
            project_names, executed_names,
            "Not all projects were executed"
        );

        // No project should be executed more than once
        let mut seen = HashSet::new();
        for project in &order {
            prop_assert!(
                seen.insert(project),
                "Project {} was executed more than once",
                project
            );
        }
    }

    /// Property: For any batch operation with level-based parallelization,
    /// projects at the same level can be executed in parallel without
    /// violating dependencies.
    #[test]
    fn prop_level_based_parallelization_is_safe(
        project_count in project_count_strategy(),
        dependency_probability in dependency_probability_strategy(),
    ) {
        let (graph, projects) = generate_dependency_graph(project_count, dependency_probability);
        let orderer = ExecutionOrderer::new(graph.clone());

        let plan = orderer
            .create_execution_plan(&projects, ParallelizationStrategy::LevelBased)
            .unwrap();

        // Verify that projects at the same level don't depend on each other
        for level in &plan.levels {
            for i in 0..level.projects.len() {
                for j in (i + 1)..level.projects.len() {
                    let project_a = &level.projects[i];
                    let project_b = &level.projects[j];

                    // A should not depend on B
                    let deps_a = graph.get_dependencies(project_a);
                    prop_assert!(
                        !deps_a.contains(project_b),
                        "Project {} at level {} depends on {} at same level",
                        project_a,
                        level.level,
                        project_b
                    );

                    // B should not depend on A
                    let deps_b = graph.get_dependencies(project_b);
                    prop_assert!(
                        !deps_b.contains(project_a),
                        "Project {} at level {} depends on {} at same level",
                        project_b,
                        level.level,
                        project_a
                    );
                }
            }
        }
    }

    /// Property: For any batch operation, the execution plan SHALL
    /// include all projects exactly once across all levels.
    #[test]
    fn prop_execution_plan_includes_all_projects(
        project_count in project_count_strategy(),
        dependency_probability in dependency_probability_strategy(),
    ) {
        let (graph, projects) = generate_dependency_graph(project_count, dependency_probability);
        let orderer = ExecutionOrderer::new(graph);

        let plan = orderer
            .create_execution_plan(&projects, ParallelizationStrategy::LevelBased)
            .unwrap();

        // Collect all projects from all levels
        let mut all_executed = Vec::new();
        for level in &plan.levels {
            all_executed.extend(level.projects.clone());
        }

        // Should have same count as input
        prop_assert_eq!(
            all_executed.len(),
            projects.len(),
            "Execution plan doesn't include all projects"
        );

        // No duplicates
        let unique: HashSet<_> = all_executed.iter().cloned().collect();
        prop_assert_eq!(
            unique.len(),
            all_executed.len(),
            "Execution plan has duplicate projects"
        );
    }

    /// Property: For any batch operation, the maximum parallelism
    /// SHALL be at least 1 and at most the number of projects.
    #[test]
    fn prop_max_parallelism_is_valid(
        project_count in project_count_strategy(),
        dependency_probability in dependency_probability_strategy(),
    ) {
        let (graph, projects) = generate_dependency_graph(project_count, dependency_probability);
        let orderer = ExecutionOrderer::new(graph);

        let plan = orderer
            .create_execution_plan(&projects, ParallelizationStrategy::LevelBased)
            .unwrap();

        prop_assert!(plan.max_parallelism >= 1, "Max parallelism is less than 1");
        prop_assert!(
            plan.max_parallelism <= projects.len(),
            "Max parallelism exceeds project count"
        );
    }

    /// Property: For any batch operation, sequential execution
    /// SHALL result in a single level containing all projects.
    #[test]
    fn prop_sequential_plan_has_single_level(
        project_count in project_count_strategy(),
        dependency_probability in dependency_probability_strategy(),
    ) {
        let (graph, projects) = generate_dependency_graph(project_count, dependency_probability);
        let orderer = ExecutionOrderer::new(graph);

        let plan = orderer
            .create_execution_plan(&projects, ParallelizationStrategy::Sequential)
            .unwrap();

        prop_assert_eq!(plan.levels.len(), 1, "Sequential plan should have exactly 1 level");
        prop_assert_eq!(
            plan.levels[0].projects.len(),
            projects.len(),
            "Sequential plan level should contain all projects"
        );
        prop_assert_eq!(plan.max_parallelism, 1, "Sequential plan max parallelism should be 1");
    }

    /// Property: For any batch operation, full parallel execution
    /// SHALL result in a single level containing all projects.
    #[test]
    fn prop_full_parallel_plan_has_single_level(
        project_count in project_count_strategy(),
        dependency_probability in dependency_probability_strategy(),
    ) {
        let (graph, projects) = generate_dependency_graph(project_count, dependency_probability);
        let orderer = ExecutionOrderer::new(graph);

        let plan = orderer
            .create_execution_plan(&projects, ParallelizationStrategy::FullParallel)
            .unwrap();

        prop_assert_eq!(plan.levels.len(), 1, "Full parallel plan should have exactly 1 level");
        prop_assert_eq!(
            plan.levels[0].projects.len(),
            projects.len(),
            "Full parallel plan level should contain all projects"
        );
        prop_assert_eq!(
            plan.max_parallelism,
            projects.len(),
            "Full parallel plan max parallelism should equal project count"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_order_with_simple_chain() {
        let mut graph = DependencyGraph::new(false);
        let projects = vec![
            create_test_project("project-a"),
            create_test_project("project-b"),
            create_test_project("project-c"),
        ];

        for project in &projects {
            graph.add_project(project.clone()).unwrap();
        }

        // B -> A, C -> B (C depends on B, B depends on A)
        graph
            .add_dependency(ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-c".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let order = orderer.determine_order(&projects).unwrap();

        assert_eq!(order.len(), 3);
        let a_idx = order.iter().position(|x| x == "project-a").unwrap();
        let b_idx = order.iter().position(|x| x == "project-b").unwrap();
        let c_idx = order.iter().position(|x| x == "project-c").unwrap();

        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn test_execution_order_with_parallel_branches() {
        let mut graph = DependencyGraph::new(false);
        let projects = vec![
            create_test_project("project-a"),
            create_test_project("project-b"),
            create_test_project("project-c"),
        ];

        for project in &projects {
            graph.add_project(project.clone()).unwrap();
        }

        // B -> A, C -> A (B and C depend on A, so A executes first)
        graph
            .add_dependency(ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-c".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let orderer = ExecutionOrderer::new(graph);
        let plan = orderer
            .create_execution_plan(&projects, ParallelizationStrategy::LevelBased)
            .unwrap();

        assert_eq!(plan.levels.len(), 2);
        assert_eq!(plan.levels[0].projects.len(), 1); // A
        assert_eq!(plan.levels[1].projects.len(), 2); // B and C
    }
}
