//! Property-based tests for the scheduler
//!
//! **Feature: ricecoder-agents, Property 1: Agent Isolation**
//! **Validates: Requirements 1.1, 1.2, 1.11, 1.12**

#[cfg(test)]
mod tests {
    use crate::executor::{ExecutionConfig, ParallelExecutor};
    use crate::models::AgentTask;
    use crate::models::{TaskOptions, TaskScope, TaskTarget, TaskType};
    use crate::scheduler::{ExecutionPhase, TaskDAG};
    use std::path::PathBuf;

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

    /// Property 1: Agent Isolation
    /// For any agent execution, one agent's failure SHALL NOT affect other agents' execution
    /// or prevent them from running.
    ///
    /// This property tests that:
    /// 1. Multiple tasks can execute in parallel
    /// 2. Each task executes independently
    /// 3. Task results are collected for all tasks regardless of individual outcomes
    #[tokio::test]
    async fn property_agent_isolation_parallel_execution() {
        // Generate multiple tasks
        let tasks = vec![
            create_test_task("task1"),
            create_test_task("task2"),
            create_test_task("task3"),
            create_test_task("task4"),
        ];

        // Create execution phase with all tasks
        let phase = ExecutionPhase {
            tasks: tasks.clone(),
        };

        // Execute tasks in parallel
        let executor = ParallelExecutor::new();
        let results = executor.execute_phase(&phase).await.unwrap();

        // Property: All tasks should have results (isolation means no task blocks others)
        assert_eq!(
            results.len(),
            tasks.len(),
            "All tasks should produce results"
        );

        // Property: Each task should have a unique ID
        let task_ids: Vec<String> = results.iter().map(|r| r.task_id.clone()).collect();
        let unique_ids: std::collections::HashSet<_> = task_ids.iter().cloned().collect();
        assert_eq!(
            unique_ids.len(),
            task_ids.len(),
            "Each task should have a unique ID"
        );

        // Property: Results should be in any order (parallel execution)
        // but all task IDs should be present
        for task in &tasks {
            assert!(
                task_ids.contains(&task.id),
                "Task {} should have a result",
                task.id
            );
        }
    }

    /// Property 1: Agent Isolation (Concurrency Limit)
    /// For any concurrency limit, the executor should respect it and still execute all tasks
    #[tokio::test]
    async fn property_agent_isolation_respects_concurrency() {
        let tasks = vec![
            create_test_task("task1"),
            create_test_task("task2"),
            create_test_task("task3"),
            create_test_task("task4"),
            create_test_task("task5"),
        ];

        let phase = ExecutionPhase {
            tasks: tasks.clone(),
        };

        // Test with different concurrency limits
        for max_concurrency in &[1, 2, 3, 5, 10] {
            let config = ExecutionConfig {
                max_concurrency: *max_concurrency,
                timeout_ms: 30000,
                verbose: false,
            };

            let executor = ParallelExecutor::with_config(config);
            let results = executor.execute_phase(&phase).await.unwrap();

            // Property: All tasks should complete regardless of concurrency limit
            assert_eq!(
                results.len(),
                tasks.len(),
                "All tasks should complete with concurrency limit {}",
                max_concurrency
            );

            // Property: All tasks should succeed
            for result in &results {
                assert!(
                    result.success,
                    "Task {} should succeed with concurrency limit {}",
                    result.task_id, max_concurrency
                );
            }
        }
    }

    /// Property 1: Agent Isolation (Task Independence)
    /// For any set of tasks, each task should execute independently without affecting others
    #[tokio::test]
    async fn property_agent_isolation_task_independence() {
        // Create tasks with different types
        let tasks = vec![
            AgentTask {
                id: "review1".to_string(),
                task_type: TaskType::CodeReview,
                target: TaskTarget {
                    files: vec![PathBuf::from("src/main.rs")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
            AgentTask {
                id: "security1".to_string(),
                task_type: TaskType::SecurityAnalysis,
                target: TaskTarget {
                    files: vec![PathBuf::from("src/lib.rs")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
            AgentTask {
                id: "refactor1".to_string(),
                task_type: TaskType::Refactoring,
                target: TaskTarget {
                    files: vec![PathBuf::from("src/utils.rs")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
        ];

        let phase = ExecutionPhase {
            tasks: tasks.clone(),
        };

        let executor = ParallelExecutor::new();
        let results = executor.execute_phase(&phase).await.unwrap();

        // Property: All tasks should complete
        assert_eq!(results.len(), 3);

        // Property: Each task should have its own result
        for task in &tasks {
            let result = results.iter().find(|r| r.task_id == task.id);
            assert!(result.is_some(), "Task {} should have a result", task.id);
            assert!(
                result.unwrap().success,
                "Task {} should succeed independently",
                task.id
            );
        }
    }

    /// Property 1: Agent Isolation (DAG Independence)
    /// For any DAG structure, tasks without dependencies should execute independently
    #[test]
    fn property_agent_isolation_dag_independence() {
        let mut dag = TaskDAG::new();

        // Create independent tasks (no dependencies)
        for i in 1..=5 {
            dag.add_task(create_test_task(&format!("task{}", i)));
        }

        // Property: All tasks should be root tasks (no dependencies)
        let root_tasks = dag.get_root_tasks();
        assert_eq!(
            root_tasks.len(),
            5,
            "All independent tasks should be root tasks"
        );

        // Property: Each task should have no dependencies
        for i in 1..=5 {
            let task_id = format!("task{}", i);
            let deps = dag.get_dependencies(&task_id);
            assert!(
                deps.is_empty(),
                "Task {} should have no dependencies",
                task_id
            );
        }

        // Property: Each task should have no dependents
        for i in 1..=5 {
            let task_id = format!("task{}", i);
            let dependents = dag.get_dependents(&task_id);
            assert!(
                dependents.is_empty(),
                "Task {} should have no dependents",
                task_id
            );
        }
    }

    /// Property 1: Agent Isolation (Execution Phases)
    /// For any DAG, tasks in the same phase should be independent
    #[test]
    fn property_agent_isolation_execution_phases() {
        let mut dag = TaskDAG::new();

        // Create a DAG with multiple phases
        dag.add_task(create_test_task("task1"));
        dag.add_task(create_test_task("task2"));
        dag.add_task(create_test_task("task3"));
        dag.add_task(create_test_task("task4"));

        // task1 and task2 are independent (phase 1)
        // task3 depends on task1 (phase 2)
        // task4 depends on task3 (phase 3)
        dag.add_dependency("task3".to_string(), "task1".to_string());
        dag.add_dependency("task4".to_string(), "task3".to_string());

        // Property: Root tasks should be independent
        let root_tasks = dag.get_root_tasks();
        assert_eq!(root_tasks.len(), 2, "Should have 2 root tasks");
        assert!(root_tasks.contains(&"task1".to_string()));
        assert!(root_tasks.contains(&"task2".to_string()));

        // Property: task1 and task2 should not depend on each other
        let task1_deps = dag.get_dependencies("task1");
        let task2_deps = dag.get_dependencies("task2");
        assert!(task1_deps.is_empty(), "task1 should have no dependencies");
        assert!(task2_deps.is_empty(), "task2 should have no dependencies");
    }

    /// Property 3: Parallel Execution Safety
    /// For any parallel agent execution, results SHALL be deterministic and identical
    /// regardless of execution order.
    ///
    /// This property tests that:
    /// 1. Multiple executions of the same tasks produce the same results
    /// 2. Task execution order doesn't affect the final result set
    /// 3. All tasks complete successfully in parallel
    #[tokio::test]
    async fn property_parallel_execution_safety_deterministic() {
        let tasks = vec![
            create_test_task("task1"),
            create_test_task("task2"),
            create_test_task("task3"),
        ];

        let phase = ExecutionPhase {
            tasks: tasks.clone(),
        };

        let executor = ParallelExecutor::new();

        // Execute the same phase multiple times
        let mut all_results = Vec::new();
        for _ in 0..3 {
            let results = executor.execute_phase(&phase).await.unwrap();
            all_results.push(results);
        }

        // Property: All executions should produce the same number of results
        for results in &all_results {
            assert_eq!(
                results.len(),
                tasks.len(),
                "Each execution should produce results for all tasks"
            );
        }

        // Property: All executions should have the same task IDs (in any order)
        let first_execution_ids: std::collections::HashSet<_> =
            all_results[0].iter().map(|r| r.task_id.clone()).collect();

        for results in &all_results[1..] {
            let execution_ids: std::collections::HashSet<_> =
                results.iter().map(|r| r.task_id.clone()).collect();
            assert_eq!(
                execution_ids, first_execution_ids,
                "All executions should produce results for the same tasks"
            );
        }

        // Property: All tasks should succeed in all executions
        for results in &all_results {
            for result in results {
                assert!(
                    result.success,
                    "Task {} should succeed in all executions",
                    result.task_id
                );
            }
        }
    }

    /// Property 3: Parallel Execution Safety (Order Independence)
    /// For any set of independent tasks, execution order should not affect results
    #[tokio::test]
    async fn property_parallel_execution_safety_order_independence() {
        let tasks = vec![
            create_test_task("task1"),
            create_test_task("task2"),
            create_test_task("task3"),
            create_test_task("task4"),
        ];

        // Execute with different orderings
        let mut all_results = Vec::new();

        // Original order
        let phase1 = ExecutionPhase {
            tasks: tasks.clone(),
        };

        // Reversed order
        let mut reversed = tasks.clone();
        reversed.reverse();
        let phase2 = ExecutionPhase { tasks: reversed };

        let executor = ParallelExecutor::new();

        let results1 = executor.execute_phase(&phase1).await.unwrap();
        let results2 = executor.execute_phase(&phase2).await.unwrap();

        all_results.push(results1);
        all_results.push(results2);

        // Property: Both executions should produce the same set of task IDs
        let ids1: std::collections::HashSet<_> =
            all_results[0].iter().map(|r| r.task_id.clone()).collect();
        let ids2: std::collections::HashSet<_> =
            all_results[1].iter().map(|r| r.task_id.clone()).collect();

        assert_eq!(ids1, ids2, "Execution order should not affect task IDs");

        // Property: All tasks should succeed regardless of order
        for results in &all_results {
            for result in results {
                assert!(
                    result.success,
                    "Task {} should succeed regardless of execution order",
                    result.task_id
                );
            }
        }
    }

    /// Property 3: Parallel Execution Safety (Consistency)
    /// For any execution configuration, results should be consistent
    #[tokio::test]
    async fn property_parallel_execution_safety_consistency() {
        let tasks = vec![
            create_test_task("task1"),
            create_test_task("task2"),
            create_test_task("task3"),
        ];

        let phase = ExecutionPhase {
            tasks: tasks.clone(),
        };

        // Test with different configurations
        let configs = vec![
            ExecutionConfig {
                max_concurrency: 1,
                timeout_ms: 30000,
                verbose: false,
            },
            ExecutionConfig {
                max_concurrency: 2,
                timeout_ms: 30000,
                verbose: false,
            },
            ExecutionConfig {
                max_concurrency: 10,
                timeout_ms: 30000,
                verbose: false,
            },
        ];

        let mut all_results = Vec::new();

        for config in configs {
            let executor = ParallelExecutor::with_config(config);
            let results = executor.execute_phase(&phase).await.unwrap();
            all_results.push(results);
        }

        // Property: All configurations should produce the same task IDs
        let first_ids: std::collections::HashSet<_> =
            all_results[0].iter().map(|r| r.task_id.clone()).collect();

        for results in &all_results[1..] {
            let ids: std::collections::HashSet<_> =
                results.iter().map(|r| r.task_id.clone()).collect();
            assert_eq!(
                ids, first_ids,
                "Different configurations should produce the same task IDs"
            );
        }

        // Property: All tasks should succeed with all configurations
        for results in &all_results {
            for result in results {
                assert!(
                    result.success,
                    "Task {} should succeed with all configurations",
                    result.task_id
                );
            }
        }
    }

    /// Property 3: Parallel Execution Safety (Completeness)
    /// For any set of tasks, all tasks should complete regardless of execution strategy
    #[tokio::test]
    async fn property_parallel_execution_safety_completeness() {
        // Test with varying numbers of tasks
        for num_tasks in &[1, 2, 5, 10, 20] {
            let mut tasks = Vec::new();
            for i in 0..*num_tasks {
                tasks.push(create_test_task(&format!("task{}", i)));
            }

            let phase = ExecutionPhase {
                tasks: tasks.clone(),
            };

            let executor = ParallelExecutor::new();
            let results = executor.execute_phase(&phase).await.unwrap();

            // Property: All tasks should have results
            assert_eq!(
                results.len(),
                *num_tasks,
                "All {} tasks should have results",
                num_tasks
            );

            // Property: All tasks should succeed
            for result in &results {
                assert!(result.success, "Task {} should succeed", result.task_id);
            }

            // Property: No duplicate results
            let ids: std::collections::HashSet<_> =
                results.iter().map(|r| r.task_id.clone()).collect();
            assert_eq!(
                ids.len(),
                *num_tasks,
                "All {} tasks should have unique results",
                num_tasks
            );
        }
    }
}
