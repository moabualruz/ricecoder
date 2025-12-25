//! Agent orchestrator for managing agent lifecycle and workflows

use std::{sync::Arc, time::Duration};

use tracing::{debug, error, info, warn};

use crate::{
    coordinator::AgentCoordinator,
    error::Result,
    models::{AgentOutput, AgentTask, ProjectContext},
    registry::AgentRegistry,
    scheduler::AgentScheduler,
};

/// Configuration for retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Central orchestrator for agent lifecycle and workflows
///
/// The `AgentOrchestrator` manages the execution of agents, including:
/// - Agent lifecycle (initialization, execution, cleanup)
/// - Inter-agent communication and handoff
/// - Sequential, parallel, and conditional workflows
/// - Error handling and retry logic
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::{AgentOrchestrator, AgentRegistry, AgentTask, TaskType, TaskTarget, TaskScope};
/// use std::sync::Arc;
/// use std::path::PathBuf;
///
/// #[tokio::main]
/// async fn main() {
///     let registry = Arc::new(AgentRegistry::new());
///     let orchestrator = AgentOrchestrator::with_defaults(registry);
///
///     let task = AgentTask {
///         id: "task-1".to_string(),
///         task_type: TaskType::CodeReview,
///         target: TaskTarget {
///             files: vec![PathBuf::from("src/main.rs")],
///             scope: TaskScope::File,
///         },
///         options: Default::default(),
///     };
///
///     let results = orchestrator.execute(vec![task]).await.unwrap();
/// }
/// ```
pub struct AgentOrchestrator {
    registry: Arc<AgentRegistry>,
    scheduler: Arc<AgentScheduler>,
    coordinator: Arc<AgentCoordinator>,
    retry_config: RetryConfig,
    context: ProjectContext,
}

impl AgentOrchestrator {
    /// Create a new agent orchestrator with full dependency injection
    ///
    /// # Arguments
    ///
    /// * `registry` - The agent registry containing registered agents
    /// * `scheduler` - The scheduler for task ordering and parallelization
    /// * `coordinator` - The coordinator for result aggregation
    /// * `context` - Project context for agent execution
    /// * `retry_config` - Retry configuration for error handling
    ///
    /// # Returns
    ///
    /// A new `AgentOrchestrator` with injected dependencies
    pub fn new(
        registry: Arc<AgentRegistry>,
        scheduler: Arc<AgentScheduler>,
        coordinator: Arc<AgentCoordinator>,
        context: ProjectContext,
        retry_config: RetryConfig,
    ) -> Self {
        Self {
            registry,
            scheduler,
            coordinator,
            context,
            retry_config,
        }
    }

    /// Create a new agent orchestrator with default dependencies
    ///
    /// # Arguments
    ///
    /// * `registry` - The agent registry containing registered agents
    ///
    /// # Returns
    ///
    /// A new `AgentOrchestrator` with default scheduler, coordinator, context, and retry configuration
    pub fn with_defaults(registry: Arc<AgentRegistry>) -> Self {
        Self::new(
            registry,
            Arc::new(AgentScheduler::new()),
            Arc::new(AgentCoordinator::new()),
            ProjectContext::default(),
            RetryConfig::default(),
        )
    }

    /// Create a new agent orchestrator with custom retry configuration
    ///
    /// # Arguments
    ///
    /// * `registry` - The agent registry containing registered agents
    /// * `retry_config` - Custom retry configuration
    ///
    /// # Returns
    ///
    /// A new `AgentOrchestrator` with the specified retry configuration
    pub fn with_retry_config(registry: Arc<AgentRegistry>, retry_config: RetryConfig) -> Self {
        Self::new(
            registry,
            Arc::new(AgentScheduler::new()),
            Arc::new(AgentCoordinator::new()),
            ProjectContext::default(),
            retry_config,
        )
    }

    /// Set the retry configuration
    pub fn set_retry_config(&mut self, retry_config: RetryConfig) {
        self.retry_config = retry_config;
    }

    /// Get the retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// Execute agents for the given tasks with retry logic
    ///
    /// This method executes the given tasks with automatic retry on failure.
    /// If execution fails, it will retry up to `max_retries` times with exponential backoff.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Vector of tasks to execute
    ///
    /// # Returns
    ///
    /// A `Result` containing the agent outputs or an error
    pub async fn execute_with_retry(&self, tasks: Vec<AgentTask>) -> Result<Vec<AgentOutput>> {
        let mut last_error = None;
        let mut backoff_ms = self.retry_config.initial_backoff_ms;

        for attempt in 0..=self.retry_config.max_retries {
            match self.execute(tasks.clone()).await {
                Ok(outputs) => {
                    if attempt > 0 {
                        info!("Orchestration succeeded on attempt {}", attempt + 1);
                    }
                    return Ok(outputs);
                }
                Err(e) => {
                    last_error = Some(e.clone());

                    if attempt < self.retry_config.max_retries {
                        warn!(
                            "Orchestration failed on attempt {}, retrying in {}ms: {}",
                            attempt + 1,
                            backoff_ms,
                            e
                        );

                        // Wait with exponential backoff
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

                        // Calculate next backoff
                        backoff_ms = std::cmp::min(
                            (backoff_ms as f64 * self.retry_config.backoff_multiplier) as u64,
                            self.retry_config.max_backoff_ms,
                        );
                    } else {
                        error!(
                            "Orchestration failed after {} attempts: {}",
                            self.retry_config.max_retries + 1,
                            e
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            crate::error::AgentError::execution_failed("Unknown error during orchestration")
        }))
    }

    /// Execute agents for the given tasks
    ///
    /// This method executes the given tasks using the orchestrator's scheduler
    /// to determine execution order and parallelism. Tasks are executed according
    /// to their dependencies, with independent tasks running in parallel.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Vector of tasks to execute
    ///
    /// # Returns
    ///
    /// A `Result` containing the agent outputs or an error
    pub async fn execute(&self, tasks: Vec<AgentTask>) -> Result<Vec<AgentOutput>> {
        info!("Starting orchestration of {} tasks", tasks.len());

        // Create execution schedule
        let schedule = self.scheduler.schedule(&tasks)?;
        debug!(
            "Created execution schedule with {} phases",
            schedule.phases.len()
        );

        let mut all_outputs = Vec::new();

        // Execute each phase
        for (phase_idx, phase) in schedule.phases.iter().enumerate() {
            debug!("Executing phase {}", phase_idx);

            // Execute all tasks in the phase in parallel
            let mut phase_futures = Vec::new();

            for task in &phase.tasks {
                let registry = self.registry.clone();
                let task = task.clone();
                let context = self.context.clone();

                let future = async move {
                    // Find agent for this task
                    let agents = registry.find_agents_by_task_type(task.task_type);

                    if agents.is_empty() {
                        error!("No agent found for task type: {:?}", task.task_type);
                        return Err(crate::error::AgentError::not_found(format!(
                            "No agent for {:?}",
                            task.task_type
                        )));
                    }

                    // Execute the first agent that supports this task
                    let agent = &agents[0];
                    debug!("Executing agent {} for task {}", agent.id(), task.id);

                    // Create agent input
                    let input = crate::models::AgentInput {
                        task,
                        context,
                        config: crate::models::AgentConfig::default(),
                    };

                    agent.execute(input).await
                };

                phase_futures.push(future);
            }

            // Wait for all futures in the phase to complete
            let phase_results = futures::future::join_all(phase_futures).await;

            for result in phase_results {
                match result {
                    Ok(output) => {
                        debug!("Agent execution succeeded");
                        all_outputs.push(output);
                    }
                    Err(e) => {
                        error!("Agent execution failed: {}", e);
                        return Err(e);
                    }
                }
            }
        }

        info!("Orchestration completed with {} outputs", all_outputs.len());
        Ok(all_outputs)
    }

    /// Execute and aggregate results from multiple agents
    ///
    /// This method executes the given tasks and then aggregates all results
    /// into a single `AgentOutput`, combining findings, suggestions, and generated content.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Vector of tasks to execute
    ///
    /// # Returns
    ///
    /// A `Result` containing the aggregated agent output or an error
    pub async fn execute_and_aggregate(&self, tasks: Vec<AgentTask>) -> Result<AgentOutput> {
        let outputs = self.execute(tasks).await?;
        self.coordinator.aggregate(outputs)
    }

    /// Execute tasks conditionally based on a predicate
    ///
    /// This method supports conditional workflows where later tasks only execute
    /// if earlier tasks meet certain conditions.
    ///
    /// # Arguments
    /// * `tasks` - Vector of tasks to execute
    /// * `condition` - Closure that determines if execution should continue
    ///   Takes the current outputs and returns true to continue, false to stop
    pub async fn execute_conditional<F>(
        &self,
        tasks: Vec<AgentTask>,
        condition: F,
    ) -> Result<Vec<AgentOutput>>
    where
        F: Fn(&[AgentOutput]) -> bool,
    {
        info!(
            "Starting conditional orchestration of {} tasks",
            tasks.len()
        );

        let schedule = self.scheduler.schedule(&tasks)?;
        debug!(
            "Created execution schedule with {} phases",
            schedule.phases.len()
        );

        let mut all_outputs = Vec::new();

        // Execute each phase
        for (phase_idx, phase) in schedule.phases.iter().enumerate() {
            debug!("Executing phase {}", phase_idx);

            // Check condition before executing phase
            if !condition(&all_outputs) {
                debug!("Condition not met, stopping execution");
                break;
            }

            // Execute all tasks in the phase in parallel
            let mut phase_futures = Vec::new();

            for task in &phase.tasks {
                let registry = self.registry.clone();
                let task = task.clone();
                let context = self.context.clone();

                let future = async move {
                    let agents = registry.find_agents_by_task_type(task.task_type);

                    if agents.is_empty() {
                        error!("No agent found for task type: {:?}", task.task_type);
                        return Err(crate::error::AgentError::not_found(format!(
                            "No agent for {:?}",
                            task.task_type
                        )));
                    }

                    let agent = &agents[0];
                    debug!("Executing agent {} for task {}", agent.id(), task.id);

                    let input = crate::models::AgentInput {
                        task,
                        context,
                        config: crate::models::AgentConfig::default(),
                    };

                    agent.execute(input).await
                };

                phase_futures.push(future);
            }

            // Wait for all futures in the phase to complete
            let phase_results = futures::future::join_all(phase_futures).await;

            for result in phase_results {
                match result {
                    Ok(output) => {
                        debug!("Agent execution succeeded");
                        all_outputs.push(output);
                    }
                    Err(e) => {
                        error!("Agent execution failed: {}", e);
                        return Err(e);
                    }
                }
            }
        }

        info!(
            "Conditional orchestration completed with {} outputs",
            all_outputs.len()
        );
        Ok(all_outputs)
    }

    /// Get the registry
    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }

    /// Get the scheduler
    pub fn scheduler(&self) -> &AgentScheduler {
        &self.scheduler
    }

    /// Get the coordinator
    pub fn coordinator(&self) -> &AgentCoordinator {
        &self.coordinator
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        agents::Agent,
        models::{TaskOptions, TaskScope, TaskTarget, TaskType},
    };

    struct TestAgent {
        id: String,
    }

    #[async_trait::async_trait]
    impl Agent for TestAgent {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            "Test Agent"
        }

        fn description(&self) -> &str {
            "A test agent"
        }

        fn supports(&self, _task_type: TaskType) -> bool {
            true
        }

        async fn execute(&self, _input: crate::models::AgentInput) -> Result<AgentOutput> {
            Ok(AgentOutput::default())
        }
    }

    #[tokio::test]
    async fn test_execute_empty_tasks() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::with_defaults(registry);

        let results = orchestrator.execute(vec![]).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_with_agent() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
        });
        registry.register(agent);

        let orchestrator = AgentOrchestrator::with_defaults(Arc::new(registry));

        let task = AgentTask {
            id: "task1".to_string(),
            task_type: TaskType::CodeReview,
            target: TaskTarget {
                files: vec![PathBuf::from("test.rs")],
                scope: TaskScope::File,
            },
            options: TaskOptions::default(),
        };

        let results = orchestrator.execute(vec![task]).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_conditional_always_true() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
        });
        registry.register(agent);

        let orchestrator = AgentOrchestrator::with_defaults(Arc::new(registry));

        let task = AgentTask {
            id: "task1".to_string(),
            task_type: TaskType::CodeReview,
            target: TaskTarget {
                files: vec![PathBuf::from("test.rs")],
                scope: TaskScope::File,
            },
            options: TaskOptions::default(),
        };

        // Condition always returns true
        let results = orchestrator
            .execute_conditional(vec![task], |_| true)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_conditional_always_false() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
        });
        registry.register(agent);

        let orchestrator = AgentOrchestrator::with_defaults(Arc::new(registry));

        let task = AgentTask {
            id: "task1".to_string(),
            task_type: TaskType::CodeReview,
            target: TaskTarget {
                files: vec![PathBuf::from("test.rs")],
                scope: TaskScope::File,
            },
            options: TaskOptions::default(),
        };

        // Condition always returns false
        let results = orchestrator
            .execute_conditional(vec![task], |_| false)
            .await
            .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_conditional_based_on_output_count() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
        });
        registry.register(agent);

        let orchestrator = AgentOrchestrator::with_defaults(Arc::new(registry));

        let tasks = vec![
            AgentTask {
                id: "task1".to_string(),
                task_type: TaskType::CodeReview,
                target: TaskTarget {
                    files: vec![PathBuf::from("test.rs")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
            AgentTask {
                id: "task2".to_string(),
                task_type: TaskType::CodeReview,
                target: TaskTarget {
                    files: vec![PathBuf::from("test.rs")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
        ];

        // Condition: continue if we have less than 2 outputs
        // Since both tasks are in the same phase, they execute together
        // So we'll get 2 outputs in the first phase
        let results = orchestrator
            .execute_conditional(tasks, |outputs| outputs.len() < 2)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 100);
        assert_eq!(config.max_backoff_ms, 10000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            initial_backoff_ms: 200,
            max_backoff_ms: 20000,
            backoff_multiplier: 1.5,
        };

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_backoff_ms, 200);
        assert_eq!(config.max_backoff_ms, 20000);
        assert_eq!(config.backoff_multiplier, 1.5);
    }

    #[test]
    fn test_orchestrator_with_retry_config() {
        let registry = Arc::new(AgentRegistry::new());
        let retry_config = RetryConfig {
            max_retries: 5,
            initial_backoff_ms: 200,
            max_backoff_ms: 20000,
            backoff_multiplier: 1.5,
        };

        let orchestrator = AgentOrchestrator::with_retry_config(registry, retry_config);
        assert_eq!(orchestrator.retry_config().max_retries, 5);
        assert_eq!(orchestrator.retry_config().initial_backoff_ms, 200);
    }

    #[test]
    fn test_orchestrator_set_retry_config() {
        let registry = Arc::new(AgentRegistry::new());
        let mut orchestrator = AgentOrchestrator::with_defaults(registry);

        let new_config = RetryConfig {
            max_retries: 10,
            initial_backoff_ms: 500,
            max_backoff_ms: 30000,
            backoff_multiplier: 2.5,
        };

        orchestrator.set_retry_config(new_config);
        assert_eq!(orchestrator.retry_config().max_retries, 10);
        assert_eq!(orchestrator.retry_config().initial_backoff_ms, 500);
    }

    #[tokio::test]
    async fn test_execute_with_retry_success_first_attempt() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
        });
        registry.register(agent);

        let orchestrator = AgentOrchestrator::with_defaults(Arc::new(registry));

        let task = AgentTask {
            id: "task1".to_string(),
            task_type: TaskType::CodeReview,
            target: TaskTarget {
                files: vec![PathBuf::from("test.rs")],
                scope: TaskScope::File,
            },
            options: TaskOptions::default(),
        };

        let results = orchestrator.execute_with_retry(vec![task]).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_with_retry_empty_tasks() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::with_defaults(registry);

        let results = orchestrator.execute_with_retry(vec![]).await.unwrap();
        assert_eq!(results.len(), 0);
    }
}
