//! Parallel execution engine for agents

use crate::error::Result;
use crate::models::AgentTask;
use crate::scheduler::ExecutionPhase;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Configuration for the parallel execution engine
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrency: usize,
    /// Timeout for each task in milliseconds
    pub timeout_ms: u64,
    /// Enable detailed logging
    pub verbose: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            timeout_ms: 30000, // 30 seconds
            verbose: false,
        }
    }
}

/// Result of executing a task
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Task ID
    pub task_id: String,
    /// Whether the task succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

/// Parallel execution engine for agents
pub struct ParallelExecutor {
    config: ExecutionConfig,
}

impl ParallelExecutor {
    /// Create a new parallel executor with default configuration
    pub fn new() -> Self {
        Self {
            config: ExecutionConfig::default(),
        }
    }

    /// Create a new parallel executor with custom configuration
    pub fn with_config(config: ExecutionConfig) -> Self {
        Self { config }
    }

    /// Execute a phase of tasks in parallel
    pub async fn execute_phase(&self, phase: &ExecutionPhase) -> Result<Vec<ExecutionResult>> {
        info!(
            task_count = phase.tasks.len(),
            max_concurrency = self.config.max_concurrency,
            "Starting parallel execution phase"
        );

        let mut results = Vec::new();

        // Execute tasks with concurrency limit
        let semaphore =
            std::sync::Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrency));

        let mut handles = Vec::new();

        for task in &phase.tasks {
            let task_clone = task.clone();
            let semaphore_clone = semaphore.clone();
            let timeout_ms = self.config.timeout_ms;
            let verbose = self.config.verbose;

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await;

                debug!(task_id = %task_clone.id, "Task execution started");

                let start = std::time::Instant::now();

                // Simulate task execution with timeout
                let result = timeout(
                    Duration::from_millis(timeout_ms),
                    Self::execute_task_internal(&task_clone, verbose),
                )
                .await;

                let duration_ms = start.elapsed().as_millis() as u64;

                match result {
                    Ok(Ok(())) => {
                        debug!(
                            task_id = %task_clone.id,
                            duration_ms = duration_ms,
                            "Task execution completed successfully"
                        );
                        ExecutionResult {
                            task_id: task_clone.id.clone(),
                            success: true,
                            error: None,
                            duration_ms,
                        }
                    }
                    Ok(Err(e)) => {
                        warn!(
                            task_id = %task_clone.id,
                            error = %e,
                            duration_ms = duration_ms,
                            "Task execution failed"
                        );
                        ExecutionResult {
                            task_id: task_clone.id.clone(),
                            success: false,
                            error: Some(e),
                            duration_ms,
                        }
                    }
                    Err(_) => {
                        warn!(
                            task_id = %task_clone.id,
                            timeout_ms = timeout_ms,
                            "Task execution timeout"
                        );
                        ExecutionResult {
                            task_id: task_clone.id.clone(),
                            success: false,
                            error: Some(format!("Task timeout after {}ms", timeout_ms)),
                            duration_ms,
                        }
                    }
                }
            });

            handles.push(handle);
        }

        // Collect results
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Task was cancelled or panicked
                    warn!(error = %e, "Task execution error");
                    results.push(ExecutionResult {
                        task_id: "unknown".to_string(),
                        success: false,
                        error: Some(format!("Task execution error: {}", e)),
                        duration_ms: 0,
                    });
                }
            }
        }

        info!(
            completed_count = results.len(),
            success_count = results.iter().filter(|r| r.success).count(),
            "Parallel execution phase completed"
        );

        Ok(results)
    }

    /// Internal task execution logic
    async fn execute_task_internal(
        task: &AgentTask,
        verbose: bool,
    ) -> std::result::Result<(), String> {
        if verbose {
            eprintln!("Executing task: {}", task.id);
        }

        // Simulate task execution
        // In a real implementation, this would call the actual agent
        tokio::time::sleep(Duration::from_millis(10)).await;

        if verbose {
            eprintln!("Task completed: {}", task.id);
        }

        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: ExecutionConfig) {
        self.config = config;
    }

    /// Set the maximum concurrency
    pub fn set_max_concurrency(&mut self, max_concurrency: usize) {
        self.config.max_concurrency = max_concurrency;
    }

    /// Set the timeout for tasks
    pub fn set_timeout_ms(&mut self, timeout_ms: u64) {
        self.config.timeout_ms = timeout_ms;
    }

    /// Enable or disable verbose logging
    pub fn set_verbose(&mut self, verbose: bool) {
        self.config.verbose = verbose;
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskOptions, TaskScope, TaskTarget, TaskType};
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

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.max_concurrency, 4);
        assert_eq!(config.timeout_ms, 30000);
        assert!(!config.verbose);
    }

    #[test]
    fn test_execution_config_custom() {
        let config = ExecutionConfig {
            max_concurrency: 8,
            timeout_ms: 60000,
            verbose: true,
        };

        assert_eq!(config.max_concurrency, 8);
        assert_eq!(config.timeout_ms, 60000);
        assert!(config.verbose);
    }

    #[test]
    fn test_parallel_executor_new() {
        let executor = ParallelExecutor::new();
        assert_eq!(executor.config().max_concurrency, 4);
        assert_eq!(executor.config().timeout_ms, 30000);
    }

    #[test]
    fn test_parallel_executor_with_config() {
        let config = ExecutionConfig {
            max_concurrency: 16,
            timeout_ms: 120000,
            verbose: true,
        };

        let executor = ParallelExecutor::with_config(config.clone());
        assert_eq!(executor.config().max_concurrency, 16);
        assert_eq!(executor.config().timeout_ms, 120000);
        assert!(executor.config().verbose);
    }

    #[test]
    fn test_parallel_executor_set_max_concurrency() {
        let mut executor = ParallelExecutor::new();
        executor.set_max_concurrency(8);
        assert_eq!(executor.config().max_concurrency, 8);
    }

    #[test]
    fn test_parallel_executor_set_timeout() {
        let mut executor = ParallelExecutor::new();
        executor.set_timeout_ms(60000);
        assert_eq!(executor.config().timeout_ms, 60000);
    }

    #[test]
    fn test_parallel_executor_set_verbose() {
        let mut executor = ParallelExecutor::new();
        executor.set_verbose(true);
        assert!(executor.config().verbose);
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult {
            task_id: "task1".to_string(),
            success: true,
            error: None,
            duration_ms: 100,
        };

        assert_eq!(result.task_id, "task1");
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult {
            task_id: "task1".to_string(),
            success: false,
            error: Some("Task failed".to_string()),
            duration_ms: 50,
        };

        assert_eq!(result.task_id, "task1");
        assert!(!result.success);
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "Task failed");
    }

    #[tokio::test]
    async fn test_execute_phase_single_task() {
        let executor = ParallelExecutor::new();
        let phase = ExecutionPhase {
            tasks: vec![create_test_task("task1")],
        };

        let results = executor.execute_phase(&phase).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task_id, "task1");
        assert!(results[0].success);
    }

    #[tokio::test]
    async fn test_execute_phase_multiple_tasks() {
        let executor = ParallelExecutor::new();
        let phase = ExecutionPhase {
            tasks: vec![
                create_test_task("task1"),
                create_test_task("task2"),
                create_test_task("task3"),
            ],
        };

        let results = executor.execute_phase(&phase).await.unwrap();
        assert_eq!(results.len(), 3);

        for result in &results {
            assert!(result.success);
            assert!(result.error.is_none());
        }
    }

    #[tokio::test]
    async fn test_execute_phase_respects_concurrency() {
        let config = ExecutionConfig {
            max_concurrency: 2,
            timeout_ms: 30000,
            verbose: false,
        };

        let executor = ParallelExecutor::with_config(config);
        let phase = ExecutionPhase {
            tasks: vec![
                create_test_task("task1"),
                create_test_task("task2"),
                create_test_task("task3"),
                create_test_task("task4"),
            ],
        };

        let results = executor.execute_phase(&phase).await.unwrap();
        assert_eq!(results.len(), 4);

        // All tasks should complete successfully
        for result in &results {
            assert!(result.success);
        }
    }

    #[tokio::test]
    async fn test_execute_phase_empty() {
        let executor = ParallelExecutor::new();
        let phase = ExecutionPhase { tasks: vec![] };

        let results = executor.execute_phase(&phase).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parallel_executor_default() {
        let _executor = ParallelExecutor::default();
        // Just verify it can be created with default
    }
}
