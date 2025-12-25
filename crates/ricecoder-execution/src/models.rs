//! Data models for execution plans and results

use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Execution plan containing all steps to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Unique identifier for the plan
    pub id: String,
    /// Human-readable name for the plan
    pub name: String,
    /// Steps to execute in order
    pub steps: Vec<ExecutionStep>,
    /// Risk score for the plan
    pub risk_score: RiskScore,
    /// Estimated duration for execution
    pub estimated_duration: Duration,
    /// Estimated complexity level
    pub estimated_complexity: ComplexityLevel,
    /// Whether approval is required before execution
    pub requires_approval: bool,
    /// Whether the plan can be edited before execution
    pub editable: bool,
}

/// A single step in an execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Unique identifier for the step
    pub id: String,
    /// Human-readable description of the step
    pub description: String,
    /// The action to perform
    pub action: StepAction,
    /// Risk score for this specific step
    pub risk_score: RiskScore,
    /// IDs of steps that must complete before this one
    pub dependencies: Vec<String>,
    /// Optional rollback action to undo this step
    pub rollback_action: Option<RollbackAction>,
    /// Current status of the step
    pub status: StepStatus,
}

/// Status of an execution step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Step is waiting to be executed
    Pending,
    /// Step is currently executing
    Running,
    /// Step completed successfully
    Completed,
    /// Step was skipped by user
    Skipped,
    /// Step failed during execution
    Failed,
    /// Step was rolled back after failure
    RolledBack,
}

/// Action to perform in a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepAction {
    /// Create a new file with content
    CreateFile {
        /// Path to the file (resolved via PathResolver)
        path: String,
        /// Content to write
        content: String,
    },
    /// Modify an existing file
    ModifyFile {
        /// Path to the file (resolved via PathResolver)
        path: String,
        /// Diff to apply
        diff: String,
    },
    /// Delete a file
    DeleteFile {
        /// Path to the file (resolved via PathResolver)
        path: String,
    },
    /// Run a shell command (legacy mode: command + args)
    RunCommand {
        /// Command to execute
        command: String,
        /// Command arguments
        args: Vec<String>,
    },
    /// Run a shell command (OpenCode-compatible shell mode)
    RunShellCommand {
        /// Full shell command string (supports pipes, redirects, &&, ;)
        command: String,
        /// Optional timeout in milliseconds (default: 120000ms)
        timeout_ms: Option<u64>,
        /// Optional working directory (default: workspace root)
        workdir: Option<String>,
        /// Required human-readable description (used as title)
        description: String,
    },
    /// Run tests
    RunTests {
        /// Optional pattern to filter tests
        pattern: Option<String>,
    },
}

/// Risk score for a plan or step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    /// Risk level (Low, Medium, High, Critical)
    pub level: RiskLevel,
    /// Numeric score (0.0 to 1.0+)
    pub score: f32,
    /// Individual risk factors contributing to the score
    pub factors: Vec<RiskFactor>,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - no approval required
    Low,
    /// Medium risk - approval may be required
    Medium,
    /// High risk - approval required
    High,
    /// Critical risk - approval required with detailed review
    Critical,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Name of the risk factor
    pub name: String,
    /// Weight/contribution to overall score
    pub weight: f32,
    /// Description of the risk factor
    pub description: String,
}

/// Complexity level of an execution plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ComplexityLevel {
    /// Simple plan with few steps
    #[default]
    Simple,
    /// Moderate complexity
    Moderate,
    /// Complex plan with many steps or dependencies
    Complex,
    /// Very complex plan
    VeryComplex,
}

/// Result of executing a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// ID of the executed plan
    pub plan_id: String,
    /// Overall execution status
    pub status: ExecutionStatus,
    /// Results for each step
    pub step_results: Vec<StepResult>,
    /// Test results if tests were run
    pub test_results: Option<TestResults>,
    /// Total duration of execution
    pub duration: Duration,
    /// Whether rollback was performed
    pub rollback_performed: bool,
}

/// Status of an execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution is pending
    Pending,
    /// Execution is in progress
    Running,
    /// Execution is waiting for approval
    WaitingApproval,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was rolled back
    RolledBack,
    /// Execution is paused
    Paused,
}

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// ID of the step
    pub step_id: String,
    /// Whether the step succeeded
    pub success: bool,
    /// Error message if step failed
    pub error: Option<String>,
    /// Duration of step execution
    pub duration: Duration,
    /// Command output (stdout/stderr) if applicable
    pub output: Option<CommandOutput>,
}

/// Output from command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: Option<i32>,
}

/// Batch execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionConfig {
    /// Whether to continue on individual failures
    pub continue_on_error: bool,
    /// Maximum number of concurrent executions (default: 1 for sequential)
    pub max_concurrent: usize,
    /// Global timeout for the entire batch (optional)
    pub batch_timeout_ms: Option<u64>,
    /// Whether to rollback on failure
    pub rollback_on_failure: bool,
}

/// Individual batch execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionResult {
    /// Step ID that was executed
    pub step_id: String,
    /// Whether this step succeeded
    pub success: bool,
    /// Command output (if applicable)
    pub output: Option<CommandOutput>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Batch execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionSummary {
    /// Total number of steps executed
    pub total_steps: usize,
    /// Number of successful executions
    pub successful: usize,
    /// Number of failed executions
    pub failed: usize,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Whether the batch was cancelled
    pub cancelled: bool,
    /// Whether rollback was performed
    pub rolled_back: bool,
}

/// Complete batch execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionOutput {
    /// Results for each step
    pub results: Vec<BatchExecutionResult>,
    /// Execution summary
    pub summary: BatchExecutionSummary,
}

/// Test results from running tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    /// Number of tests passed
    pub passed: usize,
    /// Number of tests failed
    pub failed: usize,
    /// Number of tests skipped
    pub skipped: usize,
    /// Details of test failures
    pub failures: Vec<TestFailure>,
    /// Test framework used
    pub framework: TestFramework,
}

/// Details of a test failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    /// Name of the test
    pub name: String,
    /// Failure message
    pub message: String,
    /// Optional location in code
    pub location: Option<String>,
}

/// Test framework type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestFramework {
    /// Rust (cargo test)
    Rust,
    /// TypeScript (npm test / yarn test)
    TypeScript,
    /// Python (pytest)
    Python,
    /// Other framework
    Other,
}

/// Rollback action to undo a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackAction {
    /// Type of rollback action
    pub action_type: RollbackType,
    /// Data for the rollback action
    pub data: serde_json::Value,
}

/// Type of rollback action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackType {
    /// Restore a file from backup
    RestoreFile,
    /// Delete a created file
    DeleteFile,
    /// Run a command to undo changes
    RunCommand,
}

/// Execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionMode {
    /// Execute all steps without user intervention
    #[default]
    Automatic,
    /// Require approval for each step
    StepByStep,
    /// Preview changes without applying them
    DryRun,
}

/// Execution state for pause/resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    /// ID of the execution
    pub execution_id: String,
    /// Current step index
    pub current_step_index: usize,
    /// Completed step results
    pub completed_steps: Vec<StepResult>,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Timestamp when paused
    pub paused_at: chrono::DateTime<chrono::Utc>,
}

impl ExecutionPlan {
    /// Create a new execution plan
    pub fn new(name: String, steps: Vec<ExecutionStep>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            steps,
            risk_score: RiskScore {
                level: RiskLevel::Low,
                score: 0.0,
                factors: Vec::new(),
            },
            estimated_duration: Duration::from_secs(0),
            estimated_complexity: ComplexityLevel::Simple,
            requires_approval: false,
            editable: true,
        }
    }
}

impl ExecutionStep {
    /// Create a new execution step
    pub fn new(description: String, action: StepAction) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            action,
            risk_score: RiskScore {
                level: RiskLevel::Low,
                score: 0.0,
                factors: Vec::new(),
            },
            dependencies: Vec::new(),
            rollback_action: None,
            status: StepStatus::Pending,
        }
    }
}

impl Default for RiskScore {
    fn default() -> Self {
        Self {
            level: RiskLevel::Low,
            score: 0.0,
            factors: Vec::new(),
        }
    }
}
