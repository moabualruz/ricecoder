//! Agent trait and implementations

pub mod code_review;

use crate::error::Result;
use crate::models::{AgentInput, AgentMetrics, AgentOutput, ConfigSchema, TaskType};
use async_trait::async_trait;

pub use code_review::CodeReviewAgent;

/// Trait that all agents must implement
///
/// The `Agent` trait defines the interface for specialized agents that perform different tasks
/// within the RiceCoder framework. All agents must implement this trait to be registered and
/// executed by the orchestrator.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::{Agent, AgentInput, AgentOutput};
/// use async_trait::async_trait;
///
/// struct MyAgent;
///
/// #[async_trait]
/// impl Agent for MyAgent {
///     fn id(&self) -> &str {
///         "my-agent"
///     }
///
///     fn name(&self) -> &str {
///         "My Agent"
///     }
///
///     fn description(&self) -> &str {
///         "A custom agent for specific tasks"
///     }
///
///     fn supports(&self, task_type: TaskType) -> bool {
///         matches!(task_type, TaskType::CodeReview)
///     }
///
///     async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
///         // Implement agent logic here
///         Ok(AgentOutput::default())
///     }
/// }
/// ```
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get the agent's unique identifier
    ///
    /// The ID should be a stable, unique identifier for this agent that can be used
    /// to look up the agent in the registry.
    ///
    /// # Returns
    ///
    /// A string slice containing the agent's unique identifier
    fn id(&self) -> &str;

    /// Get the agent's human-readable name
    ///
    /// The name is used for display purposes and should be descriptive but concise.
    ///
    /// # Returns
    ///
    /// A string slice containing the agent's human-readable name
    fn name(&self) -> &str;

    /// Get the agent's description
    ///
    /// The description provides more detailed information about what the agent does
    /// and can be used for help text or documentation.
    ///
    /// # Returns
    ///
    /// A string slice containing the agent's description
    fn description(&self) -> &str;

    /// Check if the agent supports a specific task type
    ///
    /// This method is used by the registry to determine which agents can handle
    /// specific task types. An agent can support multiple task types.
    ///
    /// # Arguments
    ///
    /// * `task_type` - The task type to check support for
    ///
    /// # Returns
    ///
    /// `true` if the agent supports the given task type, `false` otherwise
    fn supports(&self, task_type: TaskType) -> bool;

    /// Execute the agent with the given input
    ///
    /// This is the main method that performs the agent's work. It should be async
    /// to support non-blocking I/O and streaming responses.
    ///
    /// # Arguments
    ///
    /// * `input` - The input containing the task, context, and configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing the agent's output or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the agent execution fails for any reason
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput>;

    /// Get the agent's configuration schema
    ///
    /// This method returns a JSON schema describing the configuration options
    /// that the agent accepts. This is used for validation and documentation.
    ///
    /// # Returns
    ///
    /// A `ConfigSchema` describing the agent's configuration options
    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::default()
    }

    /// Get the agent's performance metrics
    ///
    /// This method returns metrics about the agent's performance, including
    /// execution counts, success rates, and average execution time.
    ///
    /// # Returns
    ///
    /// An `AgentMetrics` struct containing performance metrics
    fn metrics(&self) -> AgentMetrics {
        AgentMetrics::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AgentConfig, AgentTask, ProjectContext, TaskOptions, TaskScope, TaskTarget,
    };
    use std::path::PathBuf;

    /// Mock agent for testing
    struct MockAgent {
        id: String,
        name: String,
        description: String,
        supported_types: Vec<TaskType>,
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn supports(&self, task_type: TaskType) -> bool {
            self.supported_types.contains(&task_type)
        }

        async fn execute(&self, _input: AgentInput) -> Result<AgentOutput> {
            Ok(AgentOutput::default())
        }
    }

    #[test]
    fn test_agent_trait_implementation() {
        let agent = MockAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            supported_types: vec![TaskType::CodeReview],
        };

        assert_eq!(agent.id(), "test-agent");
        assert_eq!(agent.name(), "Test Agent");
        assert_eq!(agent.description(), "A test agent");
        assert!(agent.supports(TaskType::CodeReview));
        assert!(!agent.supports(TaskType::TestGeneration));
    }

    #[test]
    fn test_agent_default_metrics() {
        let agent = MockAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            supported_types: vec![],
        };

        let metrics = agent.metrics();
        assert_eq!(metrics.execution_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.avg_duration_ms, 0.0);
    }

    #[test]
    fn test_agent_default_config_schema() {
        let agent = MockAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            supported_types: vec![],
        };

        let schema = agent.config_schema();
        assert!(schema.properties.is_empty());
    }

    #[tokio::test]
    async fn test_agent_execute() {
        let agent = MockAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            supported_types: vec![TaskType::CodeReview],
        };

        let input = AgentInput {
            task: AgentTask {
                id: "task-1".to_string(),
                task_type: TaskType::CodeReview,
                target: TaskTarget {
                    files: vec![PathBuf::from("test.rs")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
            context: ProjectContext {
                name: "test-project".to_string(),
                root: PathBuf::from("/tmp/test"),
            },
            config: AgentConfig::default(),
        };

        let result = agent.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.findings.is_empty());
        assert!(output.suggestions.is_empty());
        assert!(output.generated.is_empty());
    }

    #[test]
    fn test_multiple_supported_task_types() {
        let agent = MockAgent {
            id: "multi-agent".to_string(),
            name: "Multi Agent".to_string(),
            description: "An agent supporting multiple task types".to_string(),
            supported_types: vec![
                TaskType::CodeReview,
                TaskType::SecurityAnalysis,
                TaskType::Refactoring,
            ],
        };

        assert!(agent.supports(TaskType::CodeReview));
        assert!(agent.supports(TaskType::SecurityAnalysis));
        assert!(agent.supports(TaskType::Refactoring));
        assert!(!agent.supports(TaskType::TestGeneration));
        assert!(!agent.supports(TaskType::Documentation));
    }
}
