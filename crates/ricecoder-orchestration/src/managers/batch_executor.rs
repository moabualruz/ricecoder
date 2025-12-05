//! Batch execution of operations across multiple projects

use crate::analyzers::DependencyGraph;
use crate::error::{OrchestrationError, Result};
use crate::models::{Operation, Project, Transaction, TransactionState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Trait for operations that can be executed on projects
pub trait ProjectOperation: Send + Sync {
    /// Execute the operation on a project
    fn execute(&self, project: &Project) -> Result<()>;

    /// Rollback the operation on a project
    fn rollback(&self, project: &Project) -> Result<()>;

    /// Get a description of the operation
    fn description(&self) -> String;
}

/// Configuration for batch execution
#[derive(Debug, Clone)]
pub struct BatchExecutionConfig {
    /// Whether to execute projects in parallel where safe
    pub parallel: bool,

    /// Maximum number of concurrent operations
    pub max_concurrent: usize,

    /// Whether to stop on first error
    pub fail_fast: bool,

    /// Whether to automatically rollback on failure
    pub auto_rollback: bool,
}

impl Default for BatchExecutionConfig {
    fn default() -> Self {
        Self {
            parallel: false,
            max_concurrent: 4,
            fail_fast: true,
            auto_rollback: true,
        }
    }
}

/// Result of a batch execution
#[derive(Debug, Clone)]
pub struct BatchExecutionResult {
    /// Transaction ID
    pub transaction_id: String,

    /// Projects that were executed successfully
    pub successful_projects: Vec<String>,

    /// Projects that failed
    pub failed_projects: Vec<(String, String)>,

    /// Final transaction state
    pub final_state: TransactionState,

    /// Execution order
    pub execution_order: Vec<String>,
}

/// Executes operations across multiple projects in dependency order
pub struct BatchExecutor {
    /// Dependency graph for ordering
    graph: Arc<Mutex<DependencyGraph>>,

    /// Configuration for execution
    config: BatchExecutionConfig,

    /// Transaction history
    transactions: Arc<Mutex<HashMap<String, Transaction>>>,
}

impl BatchExecutor {
    /// Creates a new batch executor
    pub fn new(graph: DependencyGraph, config: BatchExecutionConfig) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
            config,
            transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a new batch executor with default configuration
    pub fn with_graph(graph: DependencyGraph) -> Self {
        Self::new(graph, BatchExecutionConfig::default())
    }

    /// Executes an operation on all specified projects
    pub async fn execute_batch(
        &self,
        projects: Vec<Project>,
        operation: Arc<dyn ProjectOperation>,
    ) -> Result<BatchExecutionResult> {
        let transaction_id = uuid::Uuid::new_v4().to_string();

        // Get the dependency graph
        let graph = self.graph.lock().await;

        // Determine execution order
        let execution_order = self.determine_execution_order(&graph, &projects)?;

        // Filter to only projects that are in the execution order
        let projects_map: HashMap<String, Project> = projects
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();

        let mut successful_projects = Vec::new();
        let mut failed_projects = Vec::new();
        let mut executed_operations = Vec::new();

        // Execute projects in order
        for project_name in &execution_order {
            if let Some(project) = projects_map.get(project_name) {
                match operation.execute(project) {
                    Ok(()) => {
                        successful_projects.push(project_name.clone());
                        executed_operations.push(Operation {
                            id: uuid::Uuid::new_v4().to_string(),
                            project: project_name.clone(),
                            operation_type: "batch_operation".to_string(),
                            data: serde_json::json!({}),
                        });
                    }
                    Err(e) => {
                        failed_projects.push((project_name.clone(), e.to_string()));

                        if self.config.fail_fast {
                            // Rollback if configured
                            if self.config.auto_rollback {
                                self.rollback_operations(&projects_map, &executed_operations, &operation)
                                    .await?;
                            }

                            return Ok(BatchExecutionResult {
                                transaction_id,
                                successful_projects,
                                failed_projects,
                                final_state: TransactionState::RolledBack,
                                execution_order,
                            });
                        }
                    }
                }
            }
        }

        // Determine final state
        let final_state = if failed_projects.is_empty() {
            TransactionState::Committed
        } else {
            TransactionState::RolledBack
        };

        // Store transaction
        let transaction = Transaction {
            id: transaction_id.clone(),
            operations: executed_operations,
            state: final_state,
        };

        self.transactions
            .lock()
            .await
            .insert(transaction_id.clone(), transaction);

        Ok(BatchExecutionResult {
            transaction_id,
            successful_projects,
            failed_projects,
            final_state,
            execution_order,
        })
    }

    /// Determines the execution order based on dependencies
    fn determine_execution_order(
        &self,
        graph: &DependencyGraph,
        projects: &[Project],
    ) -> Result<Vec<String>> {
        // Use ExecutionOrderer to determine the order
        let orderer = crate::managers::ExecutionOrderer::new(graph.clone());
        orderer.determine_order(projects)
    }

    /// Rolls back executed operations
    async fn rollback_operations(
        &self,
        projects_map: &HashMap<String, Project>,
        executed_operations: &[Operation],
        operation: &Arc<dyn ProjectOperation>,
    ) -> Result<()> {
        // Rollback in reverse order
        for op in executed_operations.iter().rev() {
            if let Some(project) = projects_map.get(&op.project) {
                operation.rollback(project)?;
            }
        }

        Ok(())
    }

    /// Gets a transaction by ID
    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Option<Transaction>> {
        Ok(self.transactions.lock().await.get(transaction_id).cloned())
    }

    /// Gets all transactions
    pub async fn get_all_transactions(&self) -> Result<Vec<Transaction>> {
        Ok(self
            .transactions
            .lock()
            .await
            .values()
            .cloned()
            .collect())
    }

    /// Clears all transactions
    pub async fn clear_transactions(&self) -> Result<()> {
        self.transactions.lock().await.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DependencyType, ProjectStatus};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
        should_fail: bool,
    }

    impl ProjectOperation for TestOperation {
        fn execute(&self, _project: &Project) -> Result<()> {
            self.executed.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err(OrchestrationError::BatchExecutionFailed(
                    "Test failure".to_string(),
                ))
            } else {
                Ok(())
            }
        }

        fn rollback(&self, _project: &Project) -> Result<()> {
            Ok(())
        }

        fn description(&self) -> String {
            "Test operation".to_string()
        }
    }

    #[tokio::test]
    async fn test_batch_executor_creation() {
        let graph = DependencyGraph::new(false);
        let executor = BatchExecutor::with_graph(graph);

        assert_eq!(executor.config.parallel, false);
        assert_eq!(executor.config.max_concurrent, 4);
    }

    #[tokio::test]
    async fn test_execute_single_project() {
        let mut graph = DependencyGraph::new(false);
        let project = create_test_project("project-a");
        graph.add_project(project.clone()).unwrap();

        let executor = BatchExecutor::with_graph(graph);
        let operation = Arc::new(TestOperation {
            executed: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
        });

        let result = executor
            .execute_batch(vec![project], operation.clone())
            .await
            .unwrap();

        assert_eq!(result.successful_projects.len(), 1);
        assert_eq!(result.failed_projects.len(), 0);
        assert_eq!(result.final_state, TransactionState::Committed);
        assert_eq!(operation.executed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_execute_multiple_projects_in_order() {
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

        let executor = BatchExecutor::with_graph(graph);
        let operation = Arc::new(TestOperation {
            executed: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
        });

        let result = executor
            .execute_batch(
                vec![project_a, project_b, project_c],
                operation.clone(),
            )
            .await
            .unwrap();

        assert_eq!(result.successful_projects.len(), 3);
        assert_eq!(result.failed_projects.len(), 0);
        assert_eq!(result.execution_order.len(), 3);

        // Verify order: A should come before B, B before C
        let a_idx = result.execution_order.iter().position(|x| x == "project-a").unwrap();
        let b_idx = result.execution_order.iter().position(|x| x == "project-b").unwrap();
        let c_idx = result.execution_order.iter().position(|x| x == "project-c").unwrap();

        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[tokio::test]
    async fn test_execute_with_failure_fail_fast() {
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
            should_fail: true,
        });

        let result = executor
            .execute_batch(vec![project_a, project_b], operation.clone())
            .await
            .unwrap();

        assert_eq!(result.successful_projects.len(), 0);
        assert_eq!(result.failed_projects.len(), 1);
        assert_eq!(result.final_state, TransactionState::RolledBack);
    }

    #[tokio::test]
    async fn test_transaction_storage() {
        let graph = DependencyGraph::new(false);
        let project = create_test_project("project-a");
        let mut graph = graph;
        graph.add_project(project.clone()).unwrap();

        let executor = BatchExecutor::with_graph(graph);
        let operation = Arc::new(TestOperation {
            executed: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
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
        assert_eq!(txn.state, TransactionState::Committed);
    }

    #[tokio::test]
    async fn test_get_all_transactions() {
        let graph = DependencyGraph::new(false);
        let project = create_test_project("project-a");
        let mut graph = graph;
        graph.add_project(project.clone()).unwrap();

        let executor = BatchExecutor::with_graph(graph);
        let operation = Arc::new(TestOperation {
            executed: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
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
    async fn test_clear_transactions() {
        let graph = DependencyGraph::new(false);
        let project = create_test_project("project-a");
        let mut graph = graph;
        graph.add_project(project.clone()).unwrap();

        let executor = BatchExecutor::with_graph(graph);
        let operation = Arc::new(TestOperation {
            executed: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
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
}
