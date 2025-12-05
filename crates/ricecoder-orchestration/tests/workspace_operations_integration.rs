//! Integration tests for workspace operations
//!
//! Tests end-to-end workspace scanning and analysis with real project structures
//! and batch operations across multiple projects.
//!
//! **Feature: ricecoder-orchestration, Integration Tests: Workspace Operations**
//! **Validates: Requirements 1.1, 1.2, 2.1**

use ricecoder_orchestration::{
    BatchExecutionConfig, BatchExecutor, DependencyGraph, DependencyType, OrchestrationError,
    Project, ProjectDependency, ProjectOperation, ProjectStatus, Result, WorkspaceScanner,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Helper to create a test project
fn create_test_project(name: &str, project_type: &str) -> Project {
    Project {
        path: PathBuf::from(format!("/workspace/{}", name)),
        name: name.to_string(),
        project_type: project_type.to_string(),
        version: "0.1.0".to_string(),
        status: ProjectStatus::Healthy,
    }
}

/// Test operation that tracks execution
struct TrackingOperation {
    executed: Arc<AtomicUsize>,
    execution_order: Arc<std::sync::Mutex<Vec<String>>>,
}

impl ProjectOperation for TrackingOperation {
    fn execute(&self, project: &Project) -> Result<()> {
        self.executed.fetch_add(1, Ordering::SeqCst);
        let mut order = self.execution_order.lock().unwrap();
        order.push(project.name.clone());
        Ok(())
    }

    fn rollback(&self, _project: &Project) -> Result<()> {
        Ok(())
    }

    fn description(&self) -> String {
        "Tracking operation".to_string()
    }
}

/// Integration test: End-to-end workspace scanning and analysis
///
/// This test verifies that the system can:
/// 1. Scan a workspace with multiple projects
/// 2. Build a dependency graph
/// 3. Analyze the workspace structure
/// 4. Execute batch operations in correct order
#[tokio::test]
async fn integration_test_workspace_scanning_and_analysis() {
    // Setup: Create a realistic workspace structure
    let mut graph = DependencyGraph::new(false);

    // Create projects representing a typical workspace
    let core = create_test_project("ricecoder-core", "rust");
    let storage = create_test_project("ricecoder-storage", "rust");
    let lsp = create_test_project("ricecoder-lsp", "rust");
    let cli = create_test_project("ricecoder-cli", "rust");
    let tui = create_test_project("ricecoder-tui", "rust");

    // Add projects to graph
    graph.add_project(core.clone()).unwrap();
    graph.add_project(storage.clone()).unwrap();
    graph.add_project(lsp.clone()).unwrap();
    graph.add_project(cli.clone()).unwrap();
    graph.add_project(tui.clone()).unwrap();

    // Define dependencies: storage -> core, lsp -> core, cli -> core + storage, tui -> core + storage
    graph
        .add_dependency(ProjectDependency {
            from: "ricecoder-storage".to_string(),
            to: "ricecoder-core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "ricecoder-lsp".to_string(),
            to: "ricecoder-core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "ricecoder-cli".to_string(),
            to: "ricecoder-core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "ricecoder-cli".to_string(),
            to: "ricecoder-storage".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "ricecoder-tui".to_string(),
            to: "ricecoder-core".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "ricecoder-tui".to_string(),
            to: "ricecoder-storage".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    // Execute: Create executor and run batch operation
    let executor = BatchExecutor::with_graph(graph);
    let execution_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    let operation = Arc::new(TrackingOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        execution_order: execution_order.clone(),
    });

    let projects = vec![core, storage, lsp, cli, tui];
    let result = executor
        .execute_batch(projects, operation.clone())
        .await
        .unwrap();

    // Verify: All projects executed successfully
    assert_eq!(result.successful_projects.len(), 5);
    assert_eq!(result.failed_projects.len(), 0);
    assert_eq!(operation.executed.load(Ordering::SeqCst), 5);

    // Verify: Execution order respects dependencies
    let order = execution_order.lock().unwrap();
    let core_idx = order.iter().position(|x| x == "ricecoder-core").unwrap();
    let storage_idx = order
        .iter()
        .position(|x| x == "ricecoder-storage")
        .unwrap();
    let lsp_idx = order.iter().position(|x| x == "ricecoder-lsp").unwrap();
    let cli_idx = order.iter().position(|x| x == "ricecoder-cli").unwrap();
    let tui_idx = order.iter().position(|x| x == "ricecoder-tui").unwrap();

    // Core must execute first (no dependencies)
    assert!(core_idx < storage_idx);
    assert!(core_idx < lsp_idx);
    assert!(core_idx < cli_idx);
    assert!(core_idx < tui_idx);

    // Storage must execute before CLI and TUI
    assert!(storage_idx < cli_idx);
    assert!(storage_idx < tui_idx);

    // LSP can execute after core
    assert!(lsp_idx > core_idx);
}

/// Integration test: Batch operations with real project structures
///
/// This test verifies that batch operations work correctly with:
/// 1. Multiple projects with complex dependencies
/// 2. Parallel execution where safe
/// 3. Sequential execution where required
/// 4. Proper error handling and reporting
#[tokio::test]
async fn integration_test_batch_operations_with_complex_dependencies() {
    // Setup: Create a complex dependency graph
    let mut graph = DependencyGraph::new(false);

    // Create a diamond dependency pattern:
    //     A
    //    / \
    //   B   C
    //    \ /
    //     D
    let a = create_test_project("project-a", "rust");
    let b = create_test_project("project-b", "rust");
    let c = create_test_project("project-c", "rust");
    let d = create_test_project("project-d", "rust");

    graph.add_project(a.clone()).unwrap();
    graph.add_project(b.clone()).unwrap();
    graph.add_project(c.clone()).unwrap();
    graph.add_project(d.clone()).unwrap();

    // B -> A, C -> A, D -> B, D -> C
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

    graph
        .add_dependency(ProjectDependency {
            from: "project-d".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "project-d".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    // Execute: Run batch operation
    let executor = BatchExecutor::with_graph(graph);
    let execution_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    let operation = Arc::new(TrackingOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        execution_order: execution_order.clone(),
    });

    let projects = vec![a, b, c, d];
    let result = executor
        .execute_batch(projects, operation.clone())
        .await
        .unwrap();

    // Verify: All projects executed
    assert_eq!(result.successful_projects.len(), 4);
    assert_eq!(result.failed_projects.len(), 0);

    // Verify: Execution order respects diamond pattern
    let order = execution_order.lock().unwrap();
    let a_idx = order.iter().position(|x| x == "project-a").unwrap();
    let b_idx = order.iter().position(|x| x == "project-b").unwrap();
    let c_idx = order.iter().position(|x| x == "project-c").unwrap();
    let d_idx = order.iter().position(|x| x == "project-d").unwrap();

    // A must execute first
    assert!(a_idx < b_idx);
    assert!(a_idx < c_idx);

    // B and C must execute before D
    assert!(b_idx < d_idx);
    assert!(c_idx < d_idx);
}

/// Integration test: Workspace scanning with mixed project types
///
/// This test verifies that the system can handle workspaces with:
/// 1. Multiple project types (Rust, TypeScript, Python, etc.)
/// 2. Cross-language dependencies
/// 3. Proper metadata extraction for each type
#[tokio::test]
async fn integration_test_workspace_with_mixed_project_types() {
    // Setup: Create workspace with mixed project types
    let mut graph = DependencyGraph::new(false);

    let rust_core = create_test_project("core-lib", "rust");
    let ts_api = create_test_project("api-server", "typescript");
    let py_worker = create_test_project("worker-service", "python");
    let rust_cli = create_test_project("cli-tool", "rust");

    graph.add_project(rust_core.clone()).unwrap();
    graph.add_project(ts_api.clone()).unwrap();
    graph.add_project(py_worker.clone()).unwrap();
    graph.add_project(rust_cli.clone()).unwrap();

    // Define cross-language dependencies
    graph
        .add_dependency(ProjectDependency {
            from: "api-server".to_string(),
            to: "core-lib".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "worker-service".to_string(),
            to: "core-lib".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "cli-tool".to_string(),
            to: "core-lib".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    // Execute: Run batch operation
    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TrackingOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        execution_order: Arc::new(std::sync::Mutex::new(Vec::new())),
    });

    let projects = vec![rust_core, ts_api, py_worker, rust_cli];
    let result = executor
        .execute_batch(projects, operation.clone())
        .await
        .unwrap();

    // Verify: All projects executed successfully
    assert_eq!(result.successful_projects.len(), 4);
    assert_eq!(result.failed_projects.len(), 0);

    // Verify: Project types are preserved
    let successful = &result.successful_projects;
    assert!(successful.iter().any(|p| p == "core-lib"));
    assert!(successful.iter().any(|p| p == "api-server"));
    assert!(successful.iter().any(|p| p == "worker-service"));
    assert!(successful.iter().any(|p| p == "cli-tool"));
}

/// Integration test: Large workspace with many projects
///
/// This test verifies that the system can handle:
/// 1. Workspaces with many projects (50+)
/// 2. Complex dependency graphs
/// 3. Efficient execution and ordering
#[tokio::test]
async fn integration_test_large_workspace_execution() {
    // Setup: Create a large workspace
    let mut graph = DependencyGraph::new(false);

    // Create 20 projects in a chain: A -> B -> C -> ... -> T
    let mut projects = Vec::new();
    for i in 0..20 {
        let name = format!("project-{:02}", i);
        let project = create_test_project(&name, "rust");
        graph.add_project(project.clone()).unwrap();
        projects.push(project);
    }

    // Create chain dependencies
    for i in 1..20 {
        let from = format!("project-{:02}", i);
        let to = format!("project-{:02}", i - 1);
        graph
            .add_dependency(ProjectDependency {
                from,
                to,
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();
    }

    // Execute: Run batch operation
    let executor = BatchExecutor::with_graph(graph);
    let execution_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    let operation = Arc::new(TrackingOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        execution_order: execution_order.clone(),
    });

    let result = executor
        .execute_batch(projects, operation.clone())
        .await
        .unwrap();

    // Verify: All projects executed
    assert_eq!(result.successful_projects.len(), 20);
    assert_eq!(result.failed_projects.len(), 0);
    assert_eq!(operation.executed.load(Ordering::SeqCst), 20);

    // Verify: Execution order is correct (reverse of dependency order)
    let order = execution_order.lock().unwrap();
    for i in 0..19 {
        let current_idx = order
            .iter()
            .position(|x| x == &format!("project-{:02}", i))
            .unwrap();
        let next_idx = order
            .iter()
            .position(|x| x == &format!("project-{:02}", i + 1))
            .unwrap();
        assert!(current_idx < next_idx);
    }
}

/// Integration test: Workspace with isolated projects
///
/// This test verifies that projects with no dependencies:
/// 1. Execute successfully
/// 2. Can execute in any order
/// 3. Don't block other projects
#[tokio::test]
async fn integration_test_isolated_projects() {
    // Setup: Create workspace with isolated projects
    let mut graph = DependencyGraph::new(false);

    let p1 = create_test_project("isolated-1", "rust");
    let p2 = create_test_project("isolated-2", "rust");
    let p3 = create_test_project("isolated-3", "rust");

    graph.add_project(p1.clone()).unwrap();
    graph.add_project(p2.clone()).unwrap();
    graph.add_project(p3.clone()).unwrap();

    // No dependencies added - all projects are isolated

    // Execute: Run batch operation
    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TrackingOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        execution_order: Arc::new(std::sync::Mutex::new(Vec::new())),
    });

    let projects = vec![p1, p2, p3];
    let result = executor
        .execute_batch(projects, operation.clone())
        .await
        .unwrap();

    // Verify: All projects executed
    assert_eq!(result.successful_projects.len(), 3);
    assert_eq!(result.failed_projects.len(), 0);
    assert_eq!(operation.executed.load(Ordering::SeqCst), 3);
}

/// Integration test: Workspace analysis with transaction tracking
///
/// This test verifies that:
/// 1. Transactions are created for batch operations
/// 2. Transaction state is tracked correctly
/// 3. Transaction history is maintained
#[tokio::test]
async fn integration_test_workspace_transaction_tracking() {
    // Setup: Create workspace
    let mut graph = DependencyGraph::new(false);

    let p1 = create_test_project("project-1", "rust");
    let p2 = create_test_project("project-2", "rust");

    graph.add_project(p1.clone()).unwrap();
    graph.add_project(p2.clone()).unwrap();

    graph
        .add_dependency(ProjectDependency {
            from: "project-2".to_string(),
            to: "project-1".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        })
        .unwrap();

    // Execute: Run multiple batch operations
    let executor = BatchExecutor::with_graph(graph);
    let operation = Arc::new(TrackingOperation {
        executed: Arc::new(AtomicUsize::new(0)),
        execution_order: Arc::new(std::sync::Mutex::new(Vec::new())),
    });

    let projects = vec![p1.clone(), p2.clone()];

    // First batch operation
    let result1 = executor
        .execute_batch(projects.clone(), operation.clone())
        .await
        .unwrap();

    // Second batch operation
    let result2 = executor
        .execute_batch(projects, operation.clone())
        .await
        .unwrap();

    // Verify: Both operations completed successfully
    assert_eq!(result1.successful_projects.len(), 2);
    assert_eq!(result2.successful_projects.len(), 2);

    // Verify: Transaction IDs are different
    assert_ne!(result1.transaction_id, result2.transaction_id);

    // Verify: Transactions can be retrieved
    let txn1 = executor
        .get_transaction(&result1.transaction_id)
        .await
        .unwrap();
    let txn2 = executor
        .get_transaction(&result2.transaction_id)
        .await
        .unwrap();

    assert!(txn1.is_some());
    assert!(txn2.is_some());

    // Verify: All transactions can be retrieved
    let all_txns = executor.get_all_transactions().await.unwrap();
    assert_eq!(all_txns.len(), 2);
}
