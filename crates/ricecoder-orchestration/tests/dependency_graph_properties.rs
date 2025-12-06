//! Property-based tests for dependency graph consistency
//!
//! **Feature: ricecoder-orchestration, Property 2: Dependency Graph Consistency**
//! **Validates: Requirements 1.2, 3.1**

use proptest::prelude::*;
use ricecoder_orchestration::{DependencyGraph, DependencyType, Project, ProjectDependency, ProjectStatus};
use std::path::PathBuf;

/// Strategy for generating project names
fn project_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,10}".prop_map(|s| s.to_string())
}

/// Strategy for generating unique project names
fn unique_project_names_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        "[a-z][a-z0-9-]{0,10}".prop_map(|s| s.to_string()),
        1..10,
    )
    .prop_map(|mut names| {
        // Make names unique by adding index
        for (i, name) in names.iter_mut().enumerate() {
            *name = format!("{}-{}", name, i);
        }
        names.sort();
        names.dedup();
        names
    })
}

/// Strategy for generating dependency configurations
fn dependency_config_strategy() -> impl Strategy<Value = Vec<(usize, usize)>> {
    prop::collection::vec(
        (0usize..10, 0usize..10),
        0..20,
    )
}

/// Creates a test project
fn create_test_project(name: &str) -> Project {
    Project {
        path: PathBuf::from(format!("/path/to/{}", name)),
        name: name.to_string(),
        project_type: "rust".to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    }
}

proptest! {
    /// Property 2: Dependency Graph Consistency
    ///
    /// For any set of projects with declared dependencies, the dependency graph
    /// SHALL accurately reflect all direct and transitive relationships without
    /// cycles (unless explicitly allowed).
    ///
    /// **Validates: Requirements 1.2, 3.1**
    #[test]
    fn prop_dependency_graph_reflects_all_relationships(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut graph = DependencyGraph::new(false);

        // Add all projects to the graph
        for name in &project_names {
            let project = create_test_project(name);
            graph.add_project(project).expect("failed to add project");
        }

        // Add valid dependencies (filter out invalid indices and self-loops)
        let mut added_deps = Vec::new();
        for (from_idx, to_idx) in dependency_indices {
            if from_idx < project_names.len() && to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[from_idx];
                let to = &project_names[to_idx];

                // Skip if this would create a cycle (for non-cycle-allowing graph)
                if graph.can_reach(to, from) {
                    continue;
                }

                let dep = ProjectDependency {
                    from: from.clone(),
                    to: to.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: "^0.1.0".to_string(),
                };

                if graph.add_dependency(dep.clone()).is_ok() {
                    added_deps.push(dep);
                }
            }
        }

        // Verify all added dependencies are reflected in the graph
        for dep in &added_deps {
            prop_assert!(
                graph.has_dependency(&dep.from, &dep.to),
                "Dependency from {} to {} not found in graph",
                dep.from,
                dep.to
            );

            let deps = graph.get_dependencies(&dep.from);
            prop_assert!(
                deps.contains(&dep.to),
                "Dependency {} not in dependencies of {}",
                dep.to,
                dep.from
            );
        }

        // Verify graph consistency
        prop_assert!(graph.validate().is_ok(), "Graph validation failed");
    }

    /// Property: No cycles in acyclic graph
    ///
    /// For any dependency graph created with allow_cycles=false, there should be
    /// no cycles in the graph.
    #[test]
    fn prop_no_cycles_in_acyclic_graph(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut graph = DependencyGraph::new(false);

        // Add all projects
        for name in &project_names {
            let project = create_test_project(name);
            graph.add_project(project).expect("failed to add project");
        }

        // Add dependencies (cycles will be rejected)
        for (from_idx, to_idx) in dependency_indices {
            if from_idx < project_names.len() && to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[from_idx];
                let to = &project_names[to_idx];

                let dep = ProjectDependency {
                    from: from.clone(),
                    to: to.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: "^0.1.0".to_string(),
                };

                // Ignore errors (cycles will be rejected)
                let _ = graph.add_dependency(dep);
            }
        }

        // Verify no cycles exist
        prop_assert!(graph.detect_cycles().is_ok(), "Cycles detected in acyclic graph");
    }

    /// Property: Topological sort respects dependencies
    ///
    /// For any valid dependency graph, the topological sort should order projects
    /// such that all dependencies come before dependents.
    #[test]
    fn prop_topological_sort_respects_dependencies(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut graph = DependencyGraph::new(false);

        // Add all projects
        for name in &project_names {
            let project = create_test_project(name);
            graph.add_project(project).expect("failed to add project");
        }

        // Add dependencies
        for (from_idx, to_idx) in dependency_indices {
            if from_idx < project_names.len() && to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[from_idx];
                let to = &project_names[to_idx];

                let dep = ProjectDependency {
                    from: from.clone(),
                    to: to.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: "^0.1.0".to_string(),
                };

                let _ = graph.add_dependency(dep);
            }
        }

        // Get topological sort
        if let Ok(sorted) = graph.topological_sort() {
            // Verify all projects are in the sort
            prop_assert_eq!(sorted.len(), project_names.len(), "Not all projects in topological sort");

            // Create position map
            let mut positions = std::collections::HashMap::new();
            for (i, name) in sorted.iter().enumerate() {
                positions.insert(name.clone(), i);
            }

            // Verify dependencies come before dependents
            for dep in graph.get_all_dependencies() {
                let from_pos = positions.get(&dep.from).expect("from project not in sort");
                let to_pos = positions.get(&dep.to).expect("to project not in sort");
                prop_assert!(
                    from_pos < to_pos,
                    "Dependency not respected: {} should come before {}",
                    dep.from,
                    dep.to
                );
            }
        }
    }

    /// Property: Reachability is transitive
    ///
    /// For any three projects A, B, C where A can reach B and B can reach C,
    /// A should be able to reach C.
    #[test]
    fn prop_reachability_is_transitive(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(project_names.len() >= 3);

        let mut graph = DependencyGraph::new(false);

        // Add projects
        for name in &project_names {
            let project = create_test_project(name);
            graph.add_project(project).expect("failed to add project");
        }

        // Create a simple chain: A -> B -> C
        let a = &project_names[0];
        let b = &project_names[1];
        let c = &project_names[2];

        let dep_ab = ProjectDependency {
            from: a.clone(),
            to: b.clone(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        let dep_bc = ProjectDependency {
            from: b.clone(),
            to: c.clone(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        graph.add_dependency(dep_ab).expect("failed to add A->B");
        graph.add_dependency(dep_bc).expect("failed to add B->C");

        // Verify transitivity
        prop_assert!(graph.can_reach(a, b), "A should reach B");
        prop_assert!(graph.can_reach(b, c), "B should reach C");
        prop_assert!(graph.can_reach(a, c), "A should reach C (transitivity)");
        prop_assert!(!graph.can_reach(c, a), "C should not reach A");
    }

    /// Property: Dependency count matches unique added dependencies
    ///
    /// For any graph, the dependency count should equal the number of unique
    /// successfully added dependencies (duplicates are not added twice).
    #[test]
    fn prop_dependency_count_matches_added(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut graph = DependencyGraph::new(false);

        // Add all projects
        for name in &project_names {
            let project = create_test_project(name);
            graph.add_project(project).expect("failed to add project");
        }

        // Add dependencies and track unique ones
        let mut unique_deps = std::collections::HashSet::new();
        for (from_idx, to_idx) in dependency_indices {
            if from_idx < project_names.len() && to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[from_idx];
                let to = &project_names[to_idx];

                let dep = ProjectDependency {
                    from: from.clone(),
                    to: to.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: "^0.1.0".to_string(),
                };

                if graph.add_dependency(dep.clone()).is_ok() {
                    unique_deps.insert((from.clone(), to.clone()));
                }
            }
        }

        // Verify count matches unique dependencies
        prop_assert_eq!(
            graph.dependency_count(),
            unique_deps.len(),
            "Dependency count mismatch"
        );
    }

    /// Property: Removing a dependency makes it unreachable
    ///
    /// For any dependency A -> B, after removing it, A should not be able to
    /// reach B (unless there's another path).
    #[test]
    fn prop_removing_dependency_affects_reachability(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(project_names.len() >= 2);

        let mut graph = DependencyGraph::new(false);

        // Add projects
        for name in &project_names {
            let project = create_test_project(name);
            graph.add_project(project).expect("failed to add project");
        }

        let a = &project_names[0];
        let b = &project_names[1];

        // Add dependency A -> B
        let dep = ProjectDependency {
            from: a.clone(),
            to: b.clone(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        graph.add_dependency(dep).expect("failed to add dependency");
        prop_assert!(graph.can_reach(a, b), "A should reach B before removal");

        // Remove the dependency
        graph.remove_dependency(a, b).expect("failed to remove dependency");

        // Verify reachability is affected
        prop_assert!(!graph.can_reach(a, b), "A should not reach B after removal");
    }

    /// Property: Graph validation catches invalid dependencies
    ///
    /// For any graph with dependencies to non-existent projects, validation
    /// should fail.
    #[test]
    fn prop_validation_catches_invalid_dependencies(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut graph = DependencyGraph::new(false);

        // Add only one project
        let project = create_test_project(&project_names[0]);
        graph.add_project(project).expect("failed to add project");

        // Try to add dependency to non-existent project
        let dep = ProjectDependency {
            from: project_names[0].clone(),
            to: "non-existent-project".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        // This should fail
        prop_assert!(graph.add_dependency(dep).is_err(), "Should reject dependency to non-existent project");
    }

    /// Property: Adjacency list is consistent with dependencies
    ///
    /// For any graph, the adjacency list should accurately represent all
    /// direct dependencies.
    #[test]
    fn prop_adjacency_list_consistent(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut graph = DependencyGraph::new(false);

        // Add all projects
        for name in &project_names {
            let project = create_test_project(name);
            graph.add_project(project).expect("failed to add project");
        }

        // Add dependencies
        for (from_idx, to_idx) in dependency_indices {
            if from_idx < project_names.len() && to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[from_idx];
                let to = &project_names[to_idx];

                let dep = ProjectDependency {
                    from: from.clone(),
                    to: to.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: "^0.1.0".to_string(),
                };

                let _ = graph.add_dependency(dep);
            }
        }

        // Get adjacency list
        let adj_list = graph.get_adjacency_list();

        // Verify consistency
        for dep in graph.get_all_dependencies() {
            let deps = adj_list.get(&dep.from).expect("from project not in adjacency list");
            prop_assert!(
                deps.contains(&dep.to),
                "Dependency {} -> {} not in adjacency list",
                dep.from,
                dep.to
            );
        }
    }
}
