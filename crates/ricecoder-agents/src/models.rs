//! Data models for the agent framework
//!
//! This module contains all the data structures used for agent communication,
//! configuration, and result reporting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for an agent
///
/// This struct holds the configuration settings for an agent, including whether
/// it's enabled and any custom settings specific to that agent.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::AgentConfig;
/// use std::collections::HashMap;
///
/// let mut config = AgentConfig::default();
/// config.enabled = true;
/// config.settings.insert("timeout".to_string(), serde_json::json!(5000));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Whether the agent is enabled
    pub enabled: bool,
    /// Custom settings for the agent
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: HashMap::new(),
        }
    }
}

/// Input for an agent task
///
/// This struct contains all the information needed for an agent to execute a task,
/// including the task itself, project context, and configuration.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::{AgentInput, AgentTask, ProjectContext, AgentConfig, TaskType, TaskTarget, TaskScope};
/// use std::path::PathBuf;
///
/// let input = AgentInput {
///     task: AgentTask {
///         id: "task-1".to_string(),
///         task_type: TaskType::CodeReview,
///         target: TaskTarget {
///             files: vec![PathBuf::from("src/main.rs")],
///             scope: TaskScope::File,
///         },
///         options: Default::default(),
///     },
///     context: ProjectContext {
///         name: "my-project".to_string(),
///         root: PathBuf::from("/path/to/project"),
///     },
///     config: AgentConfig::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// The task to execute
    pub task: AgentTask,
    /// Project context
    pub context: ProjectContext,
    /// Agent configuration
    pub config: AgentConfig,
}

/// A task for an agent to execute
///
/// This struct represents a single task that an agent should perform. It includes
/// the task type, target files/scope, and any additional options.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::{AgentTask, TaskType, TaskTarget, TaskScope};
/// use std::path::PathBuf;
///
/// let task = AgentTask {
///     id: "task-1".to_string(),
///     task_type: TaskType::CodeReview,
///     target: TaskTarget {
///         files: vec![PathBuf::from("src/main.rs")],
///         scope: TaskScope::File,
///     },
///     options: Default::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Unique task identifier
    pub id: String,
    /// Type of task
    pub task_type: TaskType,
    /// Target for the task
    pub target: TaskTarget,
    /// Task options
    pub options: TaskOptions,
}

/// Type of task an agent can perform
///
/// This enum defines the different types of tasks that agents can handle.
/// Each task type represents a specific kind of analysis or transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// Code review task - analyzes code for quality, security, and best practices
    CodeReview,
    /// Test generation task - generates tests for code
    TestGeneration,
    /// Documentation task - generates or updates documentation
    Documentation,
    /// Refactoring task - suggests or performs code refactoring
    Refactoring,
    /// Security analysis task - analyzes code for security vulnerabilities
    SecurityAnalysis,
}

/// Target for a task
///
/// This struct specifies which files and what scope a task should operate on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTarget {
    /// Files to target
    pub files: Vec<PathBuf>,
    /// Scope of the task
    pub scope: TaskScope,
}

/// Scope of a task
///
/// This enum defines the scope at which a task operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskScope {
    /// Single file scope - task operates on individual files
    File,
    /// Module scope - task operates on a module or directory
    Module,
    /// Project scope - task operates on the entire project
    Project,
}

/// Options for a task
///
/// This struct holds custom options that can be passed to a task.
/// Options are stored as JSON values for flexibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskOptions {
    /// Custom options
    pub custom: HashMap<String, serde_json::Value>,
}

/// Project context for agent execution
///
/// This struct provides context about the project that an agent is working on.
/// It includes the project name and root directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Project name
    pub name: String,
    /// Project root path
    pub root: PathBuf,
}

/// Output from an agent
///
/// This struct contains all the results produced by an agent execution,
/// including findings, suggestions, generated content, and metadata.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::{AgentOutput, Finding, Severity};
///
/// let output = AgentOutput {
///     findings: vec![Finding {
///         id: "finding-1".to_string(),
///         severity: Severity::Warning,
///         category: "quality".to_string(),
///         message: "Function is too long".to_string(),
///         location: None,
///         suggestion: Some("Consider breaking into smaller functions".to_string()),
///     }],
///     suggestions: vec![],
///     generated: vec![],
///     metadata: Default::default(),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentOutput {
    /// Findings from the agent
    pub findings: Vec<Finding>,
    /// Suggestions from the agent
    pub suggestions: Vec<Suggestion>,
    /// Generated content
    pub generated: Vec<GeneratedContent>,
    /// Metadata about the execution
    pub metadata: AgentMetadata,
}

/// A finding from an agent
///
/// This struct represents a single finding or issue discovered by an agent.
/// Findings can include code quality issues, security vulnerabilities, or other observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique finding identifier
    pub id: String,
    /// Severity level
    pub severity: Severity,
    /// Category of the finding
    pub category: String,
    /// Message describing the finding
    pub message: String,
    /// Location in code (optional)
    pub location: Option<CodeLocation>,
    /// Suggested fix (optional)
    pub suggestion: Option<String>,
}

/// Severity level of a finding
///
/// This enum defines the severity levels for findings, ordered from least to most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational - not a problem, just information
    Info,
    /// Warning - potential issue that should be addressed
    Warning,
    /// Critical issue - must be fixed
    Critical,
}

/// Location in code
///
/// This struct specifies a location in source code using file path, line, and column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    /// File path
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
}

/// A suggestion from an agent
///
/// This struct represents a suggestion for improvement, which may include
/// a diff showing the proposed changes and whether it can be auto-fixed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique suggestion identifier
    pub id: String,
    /// Description of the suggestion
    pub description: String,
    /// Diff (optional)
    pub diff: Option<FileDiff>,
    /// Whether the suggestion can be auto-fixed
    pub auto_fixable: bool,
}

/// A file diff
///
/// This struct represents a unified diff for a file, showing the proposed changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    /// File path
    pub file: PathBuf,
    /// Diff content
    pub content: String,
}

/// Generated content from an agent
///
/// This struct represents content that was generated by an agent,
/// such as new files or code snippets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedContent {
    /// File path for the generated content
    pub file: PathBuf,
    /// Content
    pub content: String,
}

/// Metadata about agent execution
///
/// This struct contains metadata about a single agent execution,
/// including execution time and resource usage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent identifier
    pub agent_id: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Tokens used (if applicable)
    pub tokens_used: usize,
}

/// Metrics for an agent
///
/// This struct contains performance metrics for an agent across multiple executions.
#[derive(Debug, Clone, Default)]
pub struct AgentMetrics {
    /// Total execution count
    pub execution_count: u64,
    /// Successful executions
    pub success_count: u64,
    /// Failed executions
    pub error_count: u64,
    /// Average duration in milliseconds
    pub avg_duration_ms: f64,
}

/// Configuration schema for an agent
///
/// This struct defines the configuration schema for an agent,
/// describing what configuration options are available.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Schema properties
    pub properties: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert!(config.enabled);
        assert!(config.settings.is_empty());
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            enabled: true,
            settings: {
                let mut map = HashMap::new();
                map.insert("key".to_string(), serde_json::json!("value"));
                map
            },
        };

        let json = serde_json::to_string(&config).expect("serialization failed");
        let deserialized: AgentConfig =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.settings, config.settings);
    }

    #[test]
    fn test_agent_output_default() {
        let output = AgentOutput::default();
        assert!(output.findings.is_empty());
        assert!(output.suggestions.is_empty());
        assert!(output.generated.is_empty());
    }

    #[test]
    fn test_agent_output_serialization() {
        let output = AgentOutput {
            findings: vec![Finding {
                id: "finding-1".to_string(),
                severity: Severity::Warning,
                category: "quality".to_string(),
                message: "Test finding".to_string(),
                location: None,
                suggestion: None,
            }],
            suggestions: vec![],
            generated: vec![],
            metadata: AgentMetadata::default(),
        };

        let json = serde_json::to_string(&output).expect("serialization failed");
        let deserialized: AgentOutput =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.findings.len(), 1);
        assert_eq!(deserialized.findings[0].id, "finding-1");
        assert_eq!(deserialized.findings[0].severity, Severity::Warning);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
        assert!(Severity::Info < Severity::Critical);
    }

    #[test]
    fn test_task_type_serialization() {
        let task_type = TaskType::CodeReview;
        let json = serde_json::to_string(&task_type).expect("serialization failed");
        let deserialized: TaskType = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized, task_type);
    }

    #[test]
    fn test_task_scope_serialization() {
        let scope = TaskScope::Module;
        let json = serde_json::to_string(&scope).expect("serialization failed");
        let deserialized: TaskScope = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized, scope);
    }

    #[test]
    fn test_finding_with_location() {
        let finding = Finding {
            id: "finding-1".to_string(),
            severity: Severity::Critical,
            category: "security".to_string(),
            message: "Security vulnerability".to_string(),
            location: Some(CodeLocation {
                file: PathBuf::from("src/main.rs"),
                line: 42,
                column: 10,
            }),
            suggestion: Some("Use safe alternative".to_string()),
        };

        assert_eq!(finding.id, "finding-1");
        assert_eq!(finding.severity, Severity::Critical);
        assert!(finding.location.is_some());
        assert!(finding.suggestion.is_some());

        let location = finding.location.unwrap();
        assert_eq!(location.line, 42);
        assert_eq!(location.column, 10);
    }

    #[test]
    fn test_suggestion_auto_fixable() {
        let suggestion = Suggestion {
            id: "suggestion-1".to_string(),
            description: "Fix naming".to_string(),
            diff: None,
            auto_fixable: true,
        };

        assert!(suggestion.auto_fixable);
    }

    #[test]
    fn test_agent_metadata_serialization() {
        let metadata = AgentMetadata {
            agent_id: "test-agent".to_string(),
            execution_time_ms: 1000,
            tokens_used: 500,
        };

        let json = serde_json::to_string(&metadata).expect("serialization failed");
        let deserialized: AgentMetadata =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.agent_id, "test-agent");
        assert_eq!(deserialized.execution_time_ms, 1000);
        assert_eq!(deserialized.tokens_used, 500);
    }

    #[test]
    fn test_agent_metrics_default() {
        let metrics = AgentMetrics::default();
        assert_eq!(metrics.execution_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.avg_duration_ms, 0.0);
    }

    #[test]
    fn test_config_schema_default() {
        let schema = ConfigSchema::default();
        assert!(schema.properties.is_empty());
    }

    #[test]
    fn test_task_target_multiple_files() {
        let target = TaskTarget {
            files: vec![
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/lib.rs"),
                PathBuf::from("tests/test.rs"),
            ],
            scope: TaskScope::Project,
        };

        assert_eq!(target.files.len(), 3);
        assert_eq!(target.scope, TaskScope::Project);
    }

    #[test]
    fn test_project_context() {
        let context = ProjectContext {
            name: "my-project".to_string(),
            root: PathBuf::from("/home/user/projects/my-project"),
        };

        assert_eq!(context.name, "my-project");
        assert_eq!(
            context.root,
            PathBuf::from("/home/user/projects/my-project")
        );
    }

    #[test]
    fn test_generated_content() {
        let content = GeneratedContent {
            file: PathBuf::from("generated/test.rs"),
            content: "// Generated code".to_string(),
        };

        assert_eq!(content.file, PathBuf::from("generated/test.rs"));
        assert_eq!(content.content, "// Generated code");
    }
}
