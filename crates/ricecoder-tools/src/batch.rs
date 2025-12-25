//! Batch Tool for Parallel Tool Execution
//!
//! Provides functionality to execute multiple tool invocations in parallel,
//! matching OpenCode's batch tool behavior. All tools execute concurrently
//! and results are collected even if some fail.
//!
//! ## Architecture Note
//! This module was moved from ricecoder-mcp to ricecoder-tools as part of
//! architectural cleanup (ARCH-001). Tool logic belongs in ricecoder-tools,
//! while ricecoder-mcp should contain only protocol/transport code.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use async_trait::async_trait;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

// Import from ricecoder-mcp for the executor types (temporary - should move these too)
use ricecoder_mcp::{
    Error, Result, ToolError, ToolExecutionContext, ToolExecutionResult, ToolExecutor,
};

/// Maximum number of concurrent tool executions (prevents resource exhaustion)
const MAX_CONCURRENT_INVOCATIONS: usize = 10;

/// Default timeout for batch operations (2 minutes)
const DEFAULT_BATCH_TIMEOUT_SECS: u64 = 120;

/// Single tool invocation within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    /// Name of the tool to execute
    pub tool: String,
    /// Input parameters for the tool
    pub input: serde_json::Value,
    /// Optional timeout override for this specific invocation
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

/// Input for batch tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInput {
    /// Array of tool invocations to execute in parallel
    pub invocations: Vec<ToolInvocation>,
    /// Maximum concurrent executions (default: 10)
    #[serde(default)]
    pub max_concurrent: Option<usize>,
    /// Whether to continue execution if some tools fail (default: true)
    #[serde(default = "default_continue_on_failure")]
    pub continue_on_failure: bool,
}

fn default_continue_on_failure() -> bool {
    true
}

/// Result for a single tool invocation within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationResult {
    /// Index of this invocation in the original array
    pub index: usize,
    /// Name of the tool that was executed
    pub tool: String,
    /// Whether the invocation succeeded
    pub success: bool,
    /// Result data (if successful)
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Output from batch tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOutput {
    /// Whether all invocations succeeded
    pub all_succeeded: bool,
    /// Total number of invocations
    pub total_count: usize,
    /// Number of successful invocations
    pub success_count: usize,
    /// Number of failed invocations
    pub failure_count: usize,
    /// Results for each invocation (in original order)
    pub results: Vec<InvocationResult>,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
}

/// Batch tool executor for parallel tool execution
pub struct BatchTool {
    executor: Arc<dyn ToolExecutor>,
    default_timeout: Duration,
}

impl BatchTool {
    /// Create a new batch tool with the given executor
    pub fn new(executor: Arc<dyn ToolExecutor>) -> Self {
        Self {
            executor,
            default_timeout: Duration::from_secs(DEFAULT_BATCH_TIMEOUT_SECS),
        }
    }

    /// Create a new batch tool with custom timeout
    pub fn with_timeout(executor: Arc<dyn ToolExecutor>, timeout: Duration) -> Self {
        Self {
            executor,
            default_timeout: timeout,
        }
    }

    /// Execute a batch of tool invocations in parallel
    pub async fn execute_batch(
        &self,
        input: &BatchInput,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<BatchOutput> {
        let start_time = Instant::now();

        // Validate input
        if input.invocations.is_empty() {
            return Ok(BatchOutput {
                all_succeeded: true,
                total_count: 0,
                success_count: 0,
                failure_count: 0,
                results: Vec::new(),
                total_execution_time_ms: 0,
            });
        }

        let max_concurrent = input
            .max_concurrent
            .unwrap_or(MAX_CONCURRENT_INVOCATIONS)
            .min(MAX_CONCURRENT_INVOCATIONS);

        info!(
            "Executing batch with {} invocations (max concurrent: {})",
            input.invocations.len(),
            max_concurrent
        );

        // Execute in chunks to respect concurrency limit
        let mut all_results: Vec<InvocationResult> = Vec::with_capacity(input.invocations.len());

        for chunk in input.invocations.chunks(max_concurrent) {
            let chunk_results = self
                .execute_chunk(chunk, &user_id, &session_id, input.continue_on_failure)
                .await;

            // Calculate offset for this chunk
            let offset = all_results.len();

            for (i, result) in chunk_results.into_iter().enumerate() {
                let mut result = result;
                result.index = offset + i;
                all_results.push(result);
            }

            // If not continuing on failure and we have a failure, stop
            if !input.continue_on_failure && all_results.iter().any(|r| !r.success) {
                break;
            }
        }

        let success_count = all_results.iter().filter(|r| r.success).count();
        let failure_count = all_results.len() - success_count;

        let output = BatchOutput {
            all_succeeded: failure_count == 0,
            total_count: all_results.len(),
            success_count,
            failure_count,
            results: all_results,
            total_execution_time_ms: start_time.elapsed().as_millis() as u64,
        };

        if output.all_succeeded {
            info!(
                "Batch completed successfully: {} invocations in {}ms",
                output.total_count, output.total_execution_time_ms
            );
        } else {
            warn!(
                "Batch completed with failures: {}/{} succeeded in {}ms",
                output.success_count, output.total_count, output.total_execution_time_ms
            );
        }

        Ok(output)
    }

    /// Execute a chunk of invocations in parallel
    async fn execute_chunk(
        &self,
        chunk: &[ToolInvocation],
        user_id: &Option<String>,
        session_id: &Option<String>,
        _continue_on_failure: bool,
    ) -> Vec<InvocationResult> {
        let futures: Vec<_> = chunk
            .iter()
            .enumerate()
            .map(|(idx, invocation)| {
                let executor = Arc::clone(&self.executor);
                let tool_name = invocation.tool.clone();
                let input = invocation.input.clone();
                let timeout = invocation
                    .timeout_seconds
                    .map(Duration::from_secs)
                    .unwrap_or(self.default_timeout);
                let user_id = user_id.clone();
                let session_id = session_id.clone();

                async move {
                    let start = Instant::now();

                    // Convert input Value to HashMap
                    let parameters: HashMap<String, serde_json::Value> = match input.as_object() {
                        Some(obj) => obj.clone().into_iter().collect(),
                        None => {
                            // If input is not an object, wrap it
                            let mut map = HashMap::new();
                            map.insert("input".to_string(), input);
                            map
                        }
                    };

                    let context = ToolExecutionContext {
                        tool_name: tool_name.clone(),
                        parameters,
                        user_id,
                        session_id,
                        timeout,
                        metadata: HashMap::new(),
                    };

                    // Execute with timeout
                    let result = tokio::time::timeout(timeout, executor.execute(&context)).await;

                    let execution_time_ms = start.elapsed().as_millis() as u64;

                    match result {
                        Ok(Ok(exec_result)) => InvocationResult {
                            index: idx,
                            tool: tool_name,
                            success: exec_result.success,
                            result: exec_result.result,
                            error: exec_result.error.map(|e| e.error),
                            execution_time_ms,
                        },
                        Ok(Err(e)) => {
                            error!("Tool {} failed: {}", tool_name, e);
                            InvocationResult {
                                index: idx,
                                tool: tool_name,
                                success: false,
                                result: None,
                                error: Some(e.to_string()),
                                execution_time_ms,
                            }
                        }
                        Err(_) => {
                            error!("Tool {} timed out after {:?}", tool_name, timeout);
                            InvocationResult {
                                index: idx,
                                tool: tool_name,
                                success: false,
                                result: None,
                                error: Some(format!("Execution timed out after {:?}", timeout)),
                                execution_time_ms,
                            }
                        }
                    }
                }
            })
            .collect();

        // Execute all futures in parallel
        join_all(futures).await
    }

    /// Execute batch with timeout enforcement
    pub async fn execute_batch_with_timeout(
        &self,
        input: &BatchInput,
        user_id: Option<String>,
        session_id: Option<String>,
        overall_timeout: Duration,
    ) -> Result<BatchOutput> {
        match tokio::time::timeout(
            overall_timeout,
            self.execute_batch(input, user_id, session_id),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(Error::ExecutionError(format!(
                "Batch execution exceeded overall timeout of {:?}",
                overall_timeout
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_mcp::ToolMetadata;
    use std::time::SystemTime;

    /// Mock executor for testing
    struct MockExecutor {
        /// Results to return for each tool
        results: HashMap<String, Result<ToolExecutionResult>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                results: HashMap::new(),
            }
        }

        fn with_success(mut self, tool_name: &str, result: serde_json::Value) -> Self {
            self.results.insert(
                tool_name.to_string(),
                Ok(ToolExecutionResult {
                    tool_name: tool_name.to_string(),
                    success: true,
                    result: Some(result),
                    error: None,
                    execution_time_ms: 10,
                    timestamp: SystemTime::now(),
                    metadata: HashMap::new(),
                }),
            );
            self
        }

        fn with_failure(mut self, tool_name: &str, error: &str) -> Self {
            self.results.insert(
                tool_name.to_string(),
                Ok(ToolExecutionResult {
                    tool_name: tool_name.to_string(),
                    success: false,
                    result: None,
                    error: Some(ToolError {
                        tool_id: tool_name.to_string(),
                        error: error.to_string(),
                        error_type: "ExecutionError".to_string(),
                        context: None,
                    }),
                    execution_time_ms: 5,
                    timestamp: SystemTime::now(),
                    metadata: HashMap::new(),
                }),
            );
            self
        }
    }

    #[async_trait]
    impl ToolExecutor for MockExecutor {
        async fn execute(&self, context: &ToolExecutionContext) -> Result<ToolExecutionResult> {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(5)).await;

            match self.results.get(&context.tool_name) {
                Some(Ok(result)) => Ok(ToolExecutionResult {
                    tool_name: result.tool_name.clone(),
                    success: result.success,
                    result: result.result.clone(),
                    error: result.error.clone(),
                    execution_time_ms: result.execution_time_ms,
                    timestamp: result.timestamp,
                    metadata: result.metadata.clone(),
                }),
                Some(Err(e)) => Err(Error::ExecutionError(e.to_string())),
                None => Err(Error::ToolNotFound(format!(
                    "Tool not found: {}",
                    context.tool_name
                ))),
            }
        }

        async fn validate_parameters(
            &self,
            _tool_name: &str,
            _parameters: &HashMap<String, serde_json::Value>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_tool_metadata(&self, _tool_name: &str) -> Result<Option<ToolMetadata>> {
            Ok(None)
        }

        async fn list_tools(&self) -> Result<Vec<ToolMetadata>> {
            Ok(vec![])
        }

        fn generate_cache_key(&self, context: &ToolExecutionContext) -> String {
            format!("{}:{:?}", context.tool_name, context.parameters)
        }

        fn is_cache_result_valid(
            &self,
            _cached_result: &ToolExecutionResult,
            _context: &ToolExecutionContext,
        ) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn test_empty_batch() {
        let executor = Arc::new(MockExecutor::new());
        let batch_tool = BatchTool::new(executor);

        let input = BatchInput {
            invocations: vec![],
            max_concurrent: None,
            continue_on_failure: true,
        };

        let result = batch_tool.execute_batch(&input, None, None).await.unwrap();
        assert!(result.all_succeeded);
        assert_eq!(result.total_count, 0);
        assert_eq!(result.success_count, 0);
        assert_eq!(result.failure_count, 0);
    }

    #[tokio::test]
    async fn test_single_invocation_success() {
        let executor = Arc::new(
            MockExecutor::new().with_success("tool1", serde_json::json!({"status": "ok"})),
        );
        let batch_tool = BatchTool::new(executor);

        let input = BatchInput {
            invocations: vec![ToolInvocation {
                tool: "tool1".to_string(),
                input: serde_json::json!({}),
                timeout_seconds: None,
            }],
            max_concurrent: None,
            continue_on_failure: true,
        };

        let result = batch_tool.execute_batch(&input, None, None).await.unwrap();
        assert!(result.all_succeeded);
        assert_eq!(result.total_count, 1);
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 0);
        assert!(result.results[0].success);
    }

    #[tokio::test]
    async fn test_multiple_invocations_parallel() {
        let executor = Arc::new(
            MockExecutor::new()
                .with_success("tool1", serde_json::json!({"id": 1}))
                .with_success("tool2", serde_json::json!({"id": 2}))
                .with_success("tool3", serde_json::json!({"id": 3})),
        );
        let batch_tool = BatchTool::new(executor);

        let input = BatchInput {
            invocations: vec![
                ToolInvocation {
                    tool: "tool1".to_string(),
                    input: serde_json::json!({}),
                    timeout_seconds: None,
                },
                ToolInvocation {
                    tool: "tool2".to_string(),
                    input: serde_json::json!({}),
                    timeout_seconds: None,
                },
                ToolInvocation {
                    tool: "tool3".to_string(),
                    input: serde_json::json!({}),
                    timeout_seconds: None,
                },
            ],
            max_concurrent: None,
            continue_on_failure: true,
        };

        let result = batch_tool.execute_batch(&input, None, None).await.unwrap();
        assert!(result.all_succeeded);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.success_count, 3);
        assert_eq!(result.failure_count, 0);
    }

    #[tokio::test]
    async fn test_partial_failure_continue() {
        let executor = Arc::new(
            MockExecutor::new()
                .with_success("tool1", serde_json::json!({"id": 1}))
                .with_failure("tool2", "Something went wrong")
                .with_success("tool3", serde_json::json!({"id": 3})),
        );
        let batch_tool = BatchTool::new(executor);

        let input = BatchInput {
            invocations: vec![
                ToolInvocation {
                    tool: "tool1".to_string(),
                    input: serde_json::json!({}),
                    timeout_seconds: None,
                },
                ToolInvocation {
                    tool: "tool2".to_string(),
                    input: serde_json::json!({}),
                    timeout_seconds: None,
                },
                ToolInvocation {
                    tool: "tool3".to_string(),
                    input: serde_json::json!({}),
                    timeout_seconds: None,
                },
            ],
            max_concurrent: None,
            continue_on_failure: true,
        };

        let result = batch_tool.execute_batch(&input, None, None).await.unwrap();
        assert!(!result.all_succeeded);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failure_count, 1);

        // Results should be in order
        assert!(result.results[0].success);
        assert!(!result.results[1].success);
        assert!(result.results[2].success);
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let executor = Arc::new(MockExecutor::new());
        let batch_tool = BatchTool::new(executor);

        let input = BatchInput {
            invocations: vec![ToolInvocation {
                tool: "nonexistent".to_string(),
                input: serde_json::json!({}),
                timeout_seconds: None,
            }],
            max_concurrent: None,
            continue_on_failure: true,
        };

        let result = batch_tool.execute_batch(&input, None, None).await.unwrap();
        assert!(!result.all_succeeded);
        assert_eq!(result.failure_count, 1);
        assert!(result.results[0].error.is_some());
    }

    #[tokio::test]
    async fn test_concurrency_limit() {
        let executor = Arc::new(
            MockExecutor::new().with_success("tool", serde_json::json!({"ok": true})),
        );
        let batch_tool = BatchTool::new(executor);

        // Create 15 invocations but limit to 5 concurrent
        let input = BatchInput {
            invocations: (0..15)
                .map(|_| ToolInvocation {
                    tool: "tool".to_string(),
                    input: serde_json::json!({}),
                    timeout_seconds: None,
                })
                .collect(),
            max_concurrent: Some(5),
            continue_on_failure: true,
        };

        let result = batch_tool.execute_batch(&input, None, None).await.unwrap();
        // All should still complete even with chunking
        assert_eq!(result.total_count, 15);
        assert!(result.all_succeeded);
    }
}
