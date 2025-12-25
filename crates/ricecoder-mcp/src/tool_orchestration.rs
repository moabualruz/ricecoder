//! MCP Tool Orchestration
//!
//! Tool chaining, execution pipelines, result caching, and performance monitoring
//! for complex multi-tool workflows.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use ricecoder_cache::{
    storage::CacheEntry, Cache as ExternalCache, CacheConfig as ExternalCacheConfig,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::{
    error::{Error, Result, ToolError},
    metadata::ToolMetadata,
    tool_execution::{ToolExecutionContext, ToolExecutionResult, ToolExecutor},
};

/// Tool execution step in a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub depends_on: Vec<String>, // Step IDs this step depends on
    pub timeout_seconds: Option<u64>,
    pub retry_count: u32,
    pub step_id: String,
}

/// Tool execution pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPipeline {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<PipelineStep>,
    pub max_execution_time_seconds: u64,
    pub created_at: SystemTime,
}

/// Pipeline execution context
#[derive(Debug, Clone)]
pub struct PipelineExecutionContext {
    pub pipeline_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

/// Pipeline execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineExecutionResult {
    pub pipeline_id: String,
    pub success: bool,
    pub step_results: HashMap<String, ToolExecutionResult>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
    pub completed_at: SystemTime,
}

/// Tool orchestration engine
pub struct ToolOrchestrator {
    executor: Arc<dyn ToolExecutor>,
    cache: Arc<ExternalCache>,
    pipelines: Arc<RwLock<HashMap<String, ToolPipeline>>>,
    execution_stats: Arc<RwLock<HashMap<String, PipelineStats>>>,
}

impl ToolOrchestrator {
    /// Create a new tool orchestrator
    pub fn new(executor: Arc<dyn ToolExecutor>) -> Self {
        let cache_config = ExternalCacheConfig {
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
            max_entries: Some(1000),
            enable_metrics: true,
            ..Default::default()
        };

        // Create a simple memory cache for now
        let cache_storage = Arc::new(ricecoder_cache::storage::MemoryStorage::new());
        let cache = Arc::new(ExternalCache::with_config(cache_storage, cache_config));

        Self {
            executor,
            cache,
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new tool orchestrator with custom cache
    pub fn with_cache(executor: Arc<dyn ToolExecutor>, cache: Arc<ExternalCache>) -> Self {
        Self {
            executor,
            cache,
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool pipeline
    pub async fn register_pipeline(&self, pipeline: ToolPipeline) -> Result<()> {
        // Validate pipeline
        self.validate_pipeline(&pipeline).await?;

        let mut pipelines = self.pipelines.write().await;
        pipelines.insert(pipeline.id.clone(), pipeline);
        Ok(())
    }

    /// Execute a tool pipeline
    pub async fn execute_pipeline(
        &self,
        context: &PipelineExecutionContext,
    ) -> Result<PipelineExecutionResult> {
        let start_time = SystemTime::now();

        // Get pipeline
        let pipeline = {
            let pipelines = self.pipelines.read().await;
            pipelines
                .get(&context.pipeline_id)
                .cloned()
                .ok_or_else(|| {
                    Error::ValidationError(format!("Pipeline not found: {}", context.pipeline_id))
                })?
        };

        info!("Executing pipeline: {} ({})", pipeline.name, pipeline.id);

        // Check cache first
        if let Some(cached_result) = self.check_pipeline_cache(&context).await? {
            info!("Pipeline result served from cache: {}", context.pipeline_id);
            return Ok(cached_result);
        }

        // Execute pipeline steps
        let step_results = self.execute_pipeline_steps(&pipeline, context).await?;

        let execution_time_ms = SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        let success = step_results.values().all(|r| r.success);
        let error = if success {
            None
        } else {
            Some("One or more pipeline steps failed".to_string())
        };

        let result = PipelineExecutionResult {
            pipeline_id: context.pipeline_id.clone(),
            success,
            step_results,
            execution_time_ms,
            error,
            completed_at: SystemTime::now(),
        };

        // Cache successful results
        if success {
            self.cache_pipeline_result(&context, &result).await?;
        }

        // Update statistics
        self.update_pipeline_stats(&context.pipeline_id, success, execution_time_ms)
            .await?;

        info!(
            "Pipeline execution completed: {} (success: {}, time: {}ms)",
            context.pipeline_id, success, execution_time_ms
        );

        Ok(result)
    }

    /// Execute individual tool
    pub async fn execute_tool(
        &self,
        context: &ToolExecutionContext,
    ) -> Result<ToolExecutionResult> {
        // Check cache first
        let cache_key = format!(
            "tool_{}_{}",
            context.tool_name,
            serde_json::to_string(&context.parameters)?
        );

        if let Ok(Some(cached_result)) = self.cache.get::<ToolExecutionResult>(&cache_key).await {
            debug!("Tool result served from cache: {}", context.tool_name);
            return Ok(cached_result);
        }

        // Execute tool
        let result = self.executor.execute(context).await?;

        // Cache successful results
        if result.success {
            let _ = self.cache.set(&cache_key, result.clone(), None).await;
        }

        Ok(result)
    }

    /// Get pipeline statistics
    pub async fn get_pipeline_stats(&self, pipeline_id: &str) -> Option<PipelineStats> {
        let stats = self.execution_stats.read().await;
        stats.get(pipeline_id).cloned()
    }

    /// List all registered pipelines
    pub async fn list_pipelines(&self) -> Vec<ToolPipeline> {
        let pipelines = self.pipelines.read().await;
        pipelines.values().cloned().collect()
    }

    /// Validate pipeline configuration
    async fn validate_pipeline(&self, pipeline: &ToolPipeline) -> Result<()> {
        if pipeline.steps.is_empty() {
            return Err(Error::ValidationError(
                "Pipeline must have at least one step".to_string(),
            ));
        }

        // Check for duplicate step IDs
        let mut step_ids = HashSet::new();
        for step in &pipeline.steps {
            if !step_ids.insert(&step.step_id) {
                return Err(Error::ValidationError(format!(
                    "Duplicate step ID: {}",
                    step.step_id
                )));
            }
        }

        // Validate dependencies
        let step_id_set: HashSet<&String> = pipeline.steps.iter().map(|s| &s.step_id).collect();
        for step in &pipeline.steps {
            for dep in &step.depends_on {
                if !step_id_set.contains(dep) {
                    return Err(Error::ValidationError(format!(
                        "Step '{}' depends on unknown step '{}'",
                        step.step_id, dep
                    )));
                }
            }
        }

        // Validate that tools exist and parameters are correct
        for step in &pipeline.steps {
            // Check if tool exists
            match self.executor.get_tool_metadata(&step.tool_name).await {
                Ok(Some(metadata)) => {
                    // Validate parameters against metadata
                    for (param_name, param_value) in &step.parameters {
                        let param_meta = metadata.parameters.iter().find(|p| &p.name == param_name);
                        match param_meta {
                            Some(param_meta) => {
                                // Basic type validation
                                if param_meta.required && param_value.is_null() {
                                    return Err(Error::ParameterValidationError(format!(
                                        "Required parameter '{}' for tool '{}' is null",
                                        param_name, step.tool_name
                                    )));
                                }
                                // Could add more sophisticated validation here
                            }
                            None => {
                                return Err(Error::ParameterValidationError(format!(
                                    "Unknown parameter '{}' for tool '{}'",
                                    param_name, step.tool_name
                                )));
                            }
                        }
                    }

                    // Check for missing required parameters
                    for param_meta in &metadata.parameters {
                        if param_meta.required && !step.parameters.contains_key(&param_meta.name) {
                            return Err(Error::ParameterValidationError(format!(
                                "Missing required parameter '{}' for tool '{}'",
                                param_meta.name, step.tool_name
                            )));
                        }
                    }
                }
                Ok(None) => {
                    return Err(Error::ToolNotFound(format!(
                        "Tool '{}' not found in pipeline step '{}'",
                        step.tool_name, step.step_id
                    )));
                }
                Err(e) => {
                    return Err(Error::ValidationError(format!(
                        "Failed to validate tool '{}' in pipeline step '{}': {}",
                        step.tool_name, step.step_id, e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Execute pipeline steps in dependency order
    async fn execute_pipeline_steps(
        &self,
        pipeline: &ToolPipeline,
        context: &PipelineExecutionContext,
    ) -> Result<HashMap<String, ToolExecutionResult>> {
        let mut results = HashMap::new();
        let mut completed_steps = HashSet::new();
        let mut pending_steps: VecDeque<&PipelineStep> = pipeline.steps.iter().collect();

        let max_time = Duration::from_secs(pipeline.max_execution_time_seconds);
        let start_time = SystemTime::now();

        while !pending_steps.is_empty() {
            // Check timeout
            if SystemTime::now()
                .duration_since(start_time)
                .unwrap_or(max_time)
                > max_time
            {
                return Err(Error::TimeoutError(pipeline.max_execution_time_seconds));
            }

            let mut executed_any = false;

            // Find steps that can be executed (all dependencies satisfied)
            let mut i = 0;
            while i < pending_steps.len() {
                let step = pending_steps[i];

                // Check if all dependencies are satisfied
                let deps_satisfied = step
                    .depends_on
                    .iter()
                    .all(|dep| completed_steps.contains(dep));

                if deps_satisfied {
                    // Execute the step
                    let step_result = self.execute_pipeline_step(step, context).await?;
                    results.insert(step.step_id.clone(), step_result);
                    completed_steps.insert(&step.step_id);
                    pending_steps.remove(i);
                    executed_any = true;
                } else {
                    i += 1;
                }
            }

            // If no steps were executed, we have a circular dependency or unsatisfiable dependencies
            if !executed_any && !pending_steps.is_empty() {
                return Err(Error::ValidationError(
                    "Pipeline has circular or unsatisfiable dependencies".to_string(),
                ));
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(results)
    }

    /// Execute a single pipeline step
    async fn execute_pipeline_step(
        &self,
        step: &PipelineStep,
        context: &PipelineExecutionContext,
    ) -> Result<ToolExecutionResult> {
        // Merge pipeline parameters with step parameters
        let mut parameters = context.parameters.clone();
        for (key, value) in &step.parameters {
            parameters.insert(key.clone(), value.clone());
        }

        // Resolve parameter references from previous steps
        self.resolve_parameter_references(&mut parameters, context)?;

        let tool_context = ToolExecutionContext {
            tool_name: step.tool_name.clone(),
            parameters,
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            timeout: Duration::from_secs(step.timeout_seconds.unwrap_or(30)),
            metadata: context.metadata.clone(),
        };

        // Execute with retries
        let mut last_error = None;
        for attempt in 0..=step.retry_count {
            match self.execute_tool(&tool_context).await {
                Ok(result) if result.success => return Ok(result),
                Ok(result) => {
                    last_error = result.error;
                    if attempt < step.retry_count {
                        warn!(
                            "Tool execution failed (attempt {}), retrying: {}",
                            attempt + 1,
                            step.tool_name
                        );
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
                Err(e) => {
                    last_error = Some(ToolError::new(
                        step.tool_name.clone(),
                        format!("Execution error: {}", e),
                        "execution_error".to_string(),
                    ));
                    if attempt < step.retry_count {
                        warn!(
                            "Tool execution error (attempt {}), retrying: {}",
                            attempt + 1,
                            e
                        );
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }

        // Return the last error if all retries failed
        Err(Error::ExecutionError(format!(
            "Tool '{}' failed after {} attempts: {:?}",
            step.tool_name,
            step.retry_count + 1,
            last_error
        )))
    }

    /// Resolve parameter references from previous step results
    fn resolve_parameter_references(
        &self,
        parameters: &mut HashMap<String, serde_json::Value>,
        context: &PipelineExecutionContext,
    ) -> Result<()> {
        // Simple parameter reference resolution
        // Format: ${step_id.output_field} or ${step_id}
        for value in parameters.values_mut() {
            if let serde_json::Value::String(ref mut s) = value {
                if s.starts_with("${") && s.ends_with("}") {
                    let reference = &s[2..s.len() - 1];
                    // For now, just leave references unresolved
                    // In a full implementation, this would look up values from previous step results
                    warn!("Parameter reference '{}' not resolved", reference);
                }
            }
        }
        Ok(())
    }

    /// Check pipeline cache
    async fn check_pipeline_cache(
        &self,
        context: &PipelineExecutionContext,
    ) -> Result<Option<PipelineExecutionResult>> {
        let cache_key = format!(
            "pipeline_{}_{}",
            context.pipeline_id,
            serde_json::to_string(&context.parameters)?
        );

        if let Ok(Some(cached_result)) = self.cache.get::<PipelineExecutionResult>(&cache_key).await
        {
            // Check if cache is still valid (not older than pipeline definition)
            if let Some(pipeline) = self.pipelines.read().await.get(&context.pipeline_id) {
                if cached_result.completed_at > pipeline.created_at {
                    return Ok(Some(cached_result));
                }
            }
        }

        Ok(None)
    }

    /// Cache pipeline result
    async fn cache_pipeline_result(
        &self,
        context: &PipelineExecutionContext,
        result: &PipelineExecutionResult,
    ) -> Result<()> {
        let cache_key = format!(
            "pipeline_{}_{}",
            context.pipeline_id,
            serde_json::to_string(&context.parameters)?
        );
        let _ = self.cache.set(&cache_key, result.clone(), None).await;
        Ok(())
    }

    /// Update pipeline execution statistics
    async fn update_pipeline_stats(
        &self,
        pipeline_id: &str,
        success: bool,
        execution_time_ms: u64,
    ) -> Result<()> {
        let mut stats = self.execution_stats.write().await;
        let pipeline_stats =
            stats
                .entry(pipeline_id.to_string())
                .or_insert_with(|| PipelineStats {
                    pipeline_id: pipeline_id.to_string(),
                    total_executions: 0,
                    successful_executions: 0,
                    failed_executions: 0,
                    total_execution_time_ms: 0,
                    average_execution_time_ms: 0.0,
                    last_execution: None,
                });

        pipeline_stats.total_executions += 1;
        pipeline_stats.total_execution_time_ms += execution_time_ms;
        pipeline_stats.average_execution_time_ms =
            pipeline_stats.total_execution_time_ms as f64 / pipeline_stats.total_executions as f64;
        pipeline_stats.last_execution = Some(SystemTime::now());

        if success {
            pipeline_stats.successful_executions += 1;
        } else {
            pipeline_stats.failed_executions += 1;
        }

        Ok(())
    }
}

/// Pipeline execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    pub pipeline_id: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub last_execution: Option<SystemTime>,
}

impl PipelineStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64 * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    // Mock tool executor for testing
    struct MockToolExecutor;

    #[async_trait]
    impl ToolExecutor for MockToolExecutor {
        async fn execute(&self, context: &ToolExecutionContext) -> Result<ToolExecutionResult> {
            // Simple mock that succeeds for "echo" tool
            let success = context.tool_name == "echo";
            Ok(ToolExecutionResult {
                tool_name: context.tool_name.clone(),
                success,
                result: Some(serde_json::json!({"output": "mock result"})),
                error: if success {
                    None
                } else {
                    Some(ToolError::new(
                        "mock".to_string(),
                        "Mock failure".to_string(),
                        "test".to_string(),
                    ))
                },
                execution_time_ms: 10,
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
            })
        }

        fn generate_cache_key(&self, context: &ToolExecutionContext) -> String {
            format!(
                "mock:{}:{}",
                context.tool_name,
                serde_json::to_string(&context.parameters).unwrap_or_default()
            )
        }

        fn is_cache_result_valid(
            &self,
            _cached_result: &ToolExecutionResult,
            _context: &ToolExecutionContext,
        ) -> bool {
            true // Mock always considers cache valid
        }

        async fn validate_parameters(
            &self,
            _tool_name: &str,
            _parameters: &HashMap<String, serde_json::Value>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_tool_metadata(&self, tool_name: &str) -> Result<Option<ToolMetadata>> {
            // Return metadata for mock "echo" tool used in tests
            if tool_name == "echo" {
                Ok(Some(ToolMetadata {
                    id: "echo".to_string(),
                    name: "echo".to_string(),
                    description: "Mock echo tool for testing".to_string(),
                    category: "testing".to_string(),
                    parameters: vec![],
                    return_type: "string".to_string(),
                    source: crate::metadata::ToolSource::BuiltIn,
                    server_id: None,
                }))
            } else {
                Ok(None)
            }
        }

        async fn list_tools(&self) -> Result<Vec<ToolMetadata>> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_pipeline_registration() {
        let executor = Arc::new(MockToolExecutor {});
        let orchestrator = ToolOrchestrator::new(executor);

        let pipeline = ToolPipeline {
            id: "test_pipeline".to_string(),
            name: "Test Pipeline".to_string(),
            description: "A test pipeline".to_string(),
            steps: vec![PipelineStep {
                tool_name: "echo".to_string(),
                parameters: HashMap::new(),
                depends_on: vec![],
                timeout_seconds: Some(30),
                retry_count: 0,
                step_id: "step1".to_string(),
            }],
            max_execution_time_seconds: 60,
            created_at: SystemTime::now(),
        };

        assert!(orchestrator.register_pipeline(pipeline).await.is_ok());

        let pipelines = orchestrator.list_pipelines().await;
        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0].id, "test_pipeline");
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let executor = Arc::new(MockToolExecutor {});
        let orchestrator = ToolOrchestrator::new(executor);

        let pipeline = ToolPipeline {
            id: "test_pipeline".to_string(),
            name: "Test Pipeline".to_string(),
            description: "A test pipeline".to_string(),
            steps: vec![PipelineStep {
                tool_name: "echo".to_string(),
                parameters: HashMap::new(),
                depends_on: vec![],
                timeout_seconds: Some(30),
                retry_count: 0,
                step_id: "step1".to_string(),
            }],
            max_execution_time_seconds: 60,
            created_at: SystemTime::now(),
        };

        orchestrator.register_pipeline(pipeline).await.unwrap();

        let context = PipelineExecutionContext {
            pipeline_id: "test_pipeline".to_string(),
            user_id: Some("test_user".to_string()),
            session_id: Some("test_session".to_string()),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        };

        let result = orchestrator.execute_pipeline(&context).await.unwrap();
        assert!(result.success);
        assert_eq!(result.step_results.len(), 1);
        assert!(result.step_results.contains_key("step1"));
    }

    #[tokio::test]
    async fn test_pipeline_validation() {
        let executor = Arc::new(MockToolExecutor {});
        let orchestrator = ToolOrchestrator::new(executor);

        // Test empty pipeline
        let empty_pipeline = ToolPipeline {
            id: "empty".to_string(),
            name: "Empty Pipeline".to_string(),
            description: "Empty".to_string(),
            steps: vec![],
            max_execution_time_seconds: 60,
            created_at: SystemTime::now(),
        };

        assert!(orchestrator
            .register_pipeline(empty_pipeline)
            .await
            .is_err());

        // Test pipeline with circular dependency
        let circular_pipeline = ToolPipeline {
            id: "circular".to_string(),
            name: "Circular Pipeline".to_string(),
            description: "Circular".to_string(),
            steps: vec![
                PipelineStep {
                    tool_name: "echo".to_string(),
                    parameters: HashMap::new(),
                    depends_on: vec!["step2".to_string()],
                    timeout_seconds: Some(30),
                    retry_count: 0,
                    step_id: "step1".to_string(),
                },
                PipelineStep {
                    tool_name: "echo".to_string(),
                    parameters: HashMap::new(),
                    depends_on: vec!["step1".to_string()],
                    timeout_seconds: Some(30),
                    retry_count: 0,
                    step_id: "step2".to_string(),
                },
            ],
            max_execution_time_seconds: 60,
            created_at: SystemTime::now(),
        };

        orchestrator
            .register_pipeline(circular_pipeline)
            .await
            .unwrap();

        let context = PipelineExecutionContext {
            pipeline_id: "circular".to_string(),
            user_id: None,
            session_id: None,
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        };

        // Should fail due to circular dependency
        assert!(orchestrator.execute_pipeline(&context).await.is_err());
    }

    #[tokio::test]
    async fn test_tool_execution_caching() {
        let executor = Arc::new(MockToolExecutor {});
        let orchestrator = ToolOrchestrator::new(executor);

        let context = ToolExecutionContext {
            tool_name: "echo".to_string(),
            parameters: HashMap::new(),
            user_id: None,
            session_id: None,
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        };

        // First execution
        let result1 = orchestrator.execute_tool(&context).await.unwrap();
        assert!(result1.success);

        // Second execution should be cached
        let result2 = orchestrator.execute_tool(&context).await.unwrap();
        assert!(result2.success);

        // Results should be the same
        assert_eq!(result1.result, result2.result);
    }
}
