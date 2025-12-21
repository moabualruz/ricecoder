//! Unit tests for BatchExecutor
//! Tests execution with various dependency graphs, error handling, and rollback

use ricecoder_orchestration::{
    BatchExecutionConfig, BatchExecutor, DependencyGraph, DependencyType, OrchestrationError,
    Project, ProjectDependency, ProjectOperation, ProjectStatus, Result,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn create_test_project(name: &str) -> Project {
    Project {
        path: PathBuf::from(format!("/path/to/{}", name)),
        name: name.to_string(),
        project_type: "rust".to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    }
}

struct TestOperation {
    executed: Arc<AtomicUsize>,
    should_fail_on: Option<String>,
}

impl ProjectOperation for TestOperation {
    fn execute(&self, project: &Project) -> Result<()> {
        self.executed.fetch_add(1, Ordering::SeqCst);

        if let Some(ref fail_on) = self.should_fail_on {
            if &project.name == fail_on {
                return Err(OrchestrationError::BatchExecutionFailed(format!(
                    "Intentional failure on {}",
                    project.name
                )));
            }
        }

        Ok(())
    }

    fn rollback(&self, _project: &Project) -> Result<()> {
        Ok(())
    }

    fn description(&self) -> String {
        "Test operation".to_string()
    }
}

#[tokio::test]
async fn test_batch_executor_single_project() {
    let mut graph = DependencyGraph::new(false);
    let project = create_test_project("project-a");
    graph.add_project(project.clone()).unwrap();

    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    let result = executor
        .execute_batch(vec![project], operation.clone())
        .await
        .unwrap();

    assert_eq!(result.successful_projects.len(), 1);
    assert_eq!(result.failed_projects.len(), 0);
    assert_eq!(operation.executed.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_batch_executor_multiple_projects_linear() {
    let mut graph = DependencyGraph::new(false);
    let project_a = create_test_project("project-a");
    let project_b = create_test_project("project-b");
    let project_c = create_test_project("project-c");

    graph.add_project(project_a.clone()).unwrap();
    graph.add_project(project_b.clone()).unwrap();
    graph.add_project(project_c.clone()).unwrap();

    // A -> B -> C
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

    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    let result = executor
        .execute_batch(vec![project_a, project_b, project_c], operation.clone())
        .await
        .unwrap();

    assert_eq!(result.successful_projects.len(), 3);
    assert_eq!(result.failed_projects.len(), 0);
    assert_eq!(operation.executed.load(Ordering::SeqCst), 3);

    // Verify execution order
    let a_idx = result
        .execution_order
        .iter()
        .position(|x| x == "project-a")
        .unwrap();
    let b_idx = result
        .execution_order
        .iter()
        .position(|x| x == "project-b")
        .unwrap();
    let c_idx = result
        .execution_order
        .iter()
        .position(|x| x == "project-c")
        .unwrap();

    assert!(a_idx < b_idx);
    assert!(b_idx < c_idx);
}

#[tokio::test]
async fn test_batch_executor_error_handling_fail_fast() {
    let mut graph = DependencyGraph::new(false);
    let project_a = create_test_project("project-a");
    let project_b = create_test_project("project-b");

    graph.add_project(project_a.clone()).unwrap();
    graph.add_project(project_b.clone()).unwrap();

    let executor = BatchExecutor::new(
        graph,
        BatchExecutionConfig {
            parallel: false,
            max_concurrent: 4,
            fail_fast: true,
            auto_rollback: true,
        },
    );

    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: Some("project-a".to_string()),
    });

    let result = executor
        .execute_batch(vec![project_a, project_b], operation.clone())
        .await
        .unwrap();

    assert_eq!(result.successful_projects.len(), 0);
    assert_eq!(result.failed_projects.len(), 1);
    assert_eq!(result.failed_projects[0].0, "project-a");
}

#[tokio::test]
async fn test_batch_executor_error_handling_continue_on_error() {
    let mut graph = DependencyGraph::new(false);
    let project_a = create_test_project("project-a");
    let project_b = create_test_project("project-b");

    graph.add_project(project_a.clone()).unwrap();
    graph.add_project(project_b.clone()).unwrap();

    let executor = BatchExecutor::new(
        graph,
        BatchExecutionConfig {
            parallel: false,
            max_concurrent: 4,
            fail_fast: false,
            auto_rollback: false,
        },
    );

    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: Some("project-a".to_string()),
    });

    let result = executor
        .execute_batch(vec![project_a, project_b], operation.clone())
        .await
        .unwrap();

    // With fail_fast=false, both projects should be attempted
    assert_eq!(operation.executed.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_batch_executor_transaction_storage() {
    let mut graph = DependencyGraph::new(false);
    let project = create_test_project("project-a");
    graph.add_project(project.clone()).unwrap();

    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    let result = executor
        .execute_batch(vec![project], operation)
        .await
        .unwrap();

    let transaction = executor
        .get_transaction(&result.transaction_id)
        .await
        .unwrap();

    assert!(transaction.is_some());
    let txn = transaction.unwrap();
    assert_eq!(txn.id, result.transaction_id);
    assert_eq!(
        txn.state,
        ricecoder_orchestration::TransactionState::Committed
    );
}

#[tokio::test]
async fn test_batch_executor_get_all_transactions() {
    let mut graph = DependencyGraph::new(false);
    let project = create_test_project("project-a");
    graph.add_project(project.clone()).unwrap();

    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    executor
        .execute_batch(vec![project.clone()], operation.clone())
        .await
        .unwrap();

    executor
        .execute_batch(vec![project], operation)
        .await
        .unwrap();

    let transactions = executor.get_all_transactions().await.unwrap();
    assert_eq!(transactions.len(), 2);
}

#[tokio::test]
async fn test_batch_executor_clear_transactions() {
    let mut graph = DependencyGraph::new(false);
    let project = create_test_project("project-a");
    graph.add_project(project.clone()).unwrap();

    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    executor
        .execute_batch(vec![project], operation)
        .await
        .unwrap();

    let transactions = executor.get_all_transactions().await.unwrap();
    assert_eq!(transactions.len(), 1);

    executor.clear_transactions().await.unwrap();

    let transactions = executor.get_all_transactions().await.unwrap();
    assert_eq!(transactions.len(), 0);
}

#[tokio::test]
async fn test_batch_executor_parallel_safe_projects() {
    let mut graph = DependencyGraph::new(false);
    let project_a = create_test_project("project-a");
    let project_b = create_test_project("project-b");
    let project_c = create_test_project("project-c");

    graph.add_project(project_a.clone()).unwrap();
    graph.add_project(project_b.clone()).unwrap();
    graph.add_project(project_c.clone()).unwrap();

    // B -> A, C -> A (B and C can run in parallel)
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

    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    let result = executor
        .execute_batch(vec![project_a, project_b, project_c], operation.clone())
        .await
        .unwrap();

    assert_eq!(result.successful_projects.len(), 3);
    assert_eq!(result.failed_projects.len(), 0);

    // Verify A executes before B and C
    let a_idx = result
        .execution_order
        .iter()
        .position(|x| x == "project-a")
        .unwrap();
    let b_idx = result
        .execution_order
        .iter()
        .position(|x| x == "project-b")
        .unwrap();
    let c_idx = result
        .execution_order
        .iter()
        .position(|x| x == "project-c")
        .unwrap();

    assert!(a_idx < b_idx);
    assert!(a_idx < c_idx);
}

#[tokio::test]
async fn test_batch_executor_empty_project_list() {
    let graph = DependencyGraph::new(false);
    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    let result = executor.execute_batch(vec![], operation).await.unwrap();

    assert_eq!(result.successful_projects.len(), 0);
    assert_eq!(result.failed_projects.len(), 0);
}

#[tokio::test]
async fn test_batch_executor_transaction_state_committed() {
    let mut graph = DependencyGraph::new(false);
    let project = create_test_project("project-a");
    graph.add_project(project.clone()).unwrap();

    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: None,
    });

    let result = executor
        .execute_batch(vec![project], operation)
        .await
        .unwrap();

    assert_eq!(
        result.final_state,
        ricecoder_orchestration::TransactionState::Committed
    );
}

#[tokio::test]
async fn test_batch_executor_transaction_state_rolled_back() {
    let mut graph = DependencyGraph::new(false);
    let project = create_test_project("project-a");
    graph.add_project(project.clone()).unwrap();

    let executor = BatchExecutor::new(
        graph,
        BatchExecutionConfig {
            parallel: false,
            max_concurrent: 4,
            fail_fast: true,
            auto_rollback: true,
        },
    );

    let operation = Arc::new(TestOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        should_fail_on: Some("project-a".to_string()),
    });

    let result = executor
        .execute_batch(vec![project], operation)
        .await
        .unwrap();

    assert_eq!(
        result.final_state,
        ricecoder_orchestration::TransactionState::RolledBack
    );
}
