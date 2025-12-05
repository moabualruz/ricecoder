//! Core data models for workflows

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow identifier
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow description
    pub description: String,
    /// Workflow parameters
    #[serde(default)]
    pub parameters: Vec<WorkflowParameter>,
    /// Workflow steps
    pub steps: Vec<WorkflowStep>,
    /// Workflow configuration
    pub config: WorkflowConfig,
}

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique step identifier
    pub id: String,
    /// Step name
    pub name: String,
    /// Step type
    pub step_type: StepType,
    /// Step configuration
    pub config: StepConfig,
    /// Dependencies on other steps
    pub dependencies: Vec<String>,
    /// Whether approval is required
    pub approval_required: bool,
    /// Error action to take on failure
    pub on_error: ErrorAction,
    /// Risk score for this step (0-100)
    #[serde(default)]
    pub risk_score: Option<u8>,
    /// Risk factors for this step
    #[serde(default)]
    pub risk_factors: RiskFactors,
}

impl Default for WorkflowStep {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            step_type: StepType::Agent(AgentStep {
                agent_id: String::new(),
                task: String::new(),
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors: RiskFactors::default(),
        }
    }
}

/// Type of workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepType {
    /// Agent execution step
    #[serde(rename = "agent")]
    Agent(AgentStep),
    /// Command execution step
    #[serde(rename = "command")]
    Command(CommandStep),
    /// Conditional branching step
    #[serde(rename = "condition")]
    Condition(ConditionStep),
    /// Parallel execution step
    #[serde(rename = "parallel")]
    Parallel(ParallelStep),
    /// Approval gate step
    #[serde(rename = "approval")]
    Approval(ApprovalStep),
}

/// Agent execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    /// Agent identifier
    pub agent_id: String,
    /// Task to execute
    pub task: String,
}

/// Command execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStep {
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Execution timeout
    pub timeout: u64,
}

/// Conditional branching step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionStep {
    /// Condition expression
    pub condition: String,
    /// Steps to execute if condition is true
    pub then_steps: Vec<String>,
    /// Steps to execute if condition is false
    pub else_steps: Vec<String>,
}

/// Parallel execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelStep {
    /// Steps to execute in parallel
    pub steps: Vec<String>,
    /// Maximum concurrent executions
    pub max_concurrency: usize,
}

/// Approval gate step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStep {
    /// Approval message
    pub message: String,
    /// Approval timeout
    pub timeout: u64,
    /// Default approval decision
    pub default: ApprovalDefault,
}

/// Default approval decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalDefault {
    /// Approve by default
    #[serde(rename = "approve")]
    Approve,
    /// Reject by default
    #[serde(rename = "reject")]
    Reject,
}

/// Step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    /// Configuration as JSON value
    #[serde(flatten)]
    pub config: serde_json::Value,
}

/// Error action on step failure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ErrorAction {
    /// Fail the workflow
    #[serde(rename = "fail")]
    Fail,
    /// Retry the step
    #[serde(rename = "retry")]
    Retry {
        /// Maximum retry attempts
        max_attempts: usize,
        /// Delay between retries in milliseconds
        delay_ms: u64,
    },
    /// Skip the step
    #[serde(rename = "skip")]
    Skip,
    /// Rollback the workflow
    #[serde(rename = "rollback")]
    Rollback,
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Workflow timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Maximum parallel steps
    pub max_parallel: Option<usize>,
}

/// Workflow execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Workflow identifier
    pub workflow_id: String,
    /// Current execution status
    pub status: WorkflowStatus,
    /// Current step being executed
    pub current_step: Option<String>,
    /// Completed steps
    pub completed_steps: Vec<String>,
    /// Results for each step
    pub step_results: HashMap<String, StepResult>,
    /// Workflow start time
    pub started_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// Workflow execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    /// Workflow is pending
    #[serde(rename = "pending")]
    Pending,
    /// Workflow is running
    #[serde(rename = "running")]
    Running,
    /// Workflow is paused
    #[serde(rename = "paused")]
    Paused,
    /// Workflow is waiting for approval
    #[serde(rename = "waiting_approval")]
    WaitingApproval,
    /// Workflow completed successfully
    #[serde(rename = "completed")]
    Completed,
    /// Workflow failed
    #[serde(rename = "failed")]
    Failed,
    /// Workflow was cancelled
    #[serde(rename = "cancelled")]
    Cancelled,
}

/// Result of a step execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepResult {
    /// Step execution status
    pub status: StepStatus,
    /// Step output
    pub output: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Step execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    /// Step is pending
    #[serde(rename = "pending")]
    Pending,
    /// Step is running
    #[serde(rename = "running")]
    Running,
    /// Step completed successfully
    #[serde(rename = "completed")]
    Completed,
    /// Step failed
    #[serde(rename = "failed")]
    Failed,
    /// Step was skipped
    #[serde(rename = "skipped")]
    Skipped,
}

/// Workflow parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (string, number, boolean, object, array)
    #[serde(rename = "type")]
    pub param_type: String,
    /// Default value if not provided
    pub default: Option<serde_json::Value>,
    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,
    /// Parameter description
    #[serde(default)]
    pub description: String,
}

/// Workflow parameters (legacy, for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter value
    pub value: serde_json::Value,
}

/// Risk factors for a workflow step
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskFactors {
    /// Impact score (0-100): potential for data loss or system damage
    #[serde(default)]
    pub impact: u8,
    /// Reversibility score (0-100): ability to undo the operation
    #[serde(default)]
    pub reversibility: u8,
    /// Complexity score (0-100): number of dependencies and interactions
    #[serde(default)]
    pub complexity: u8,
}

/// Risk assessment for a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Step identifier
    pub step_id: String,
    /// Step name
    pub step_name: String,
    /// Calculated risk score (0-100)
    pub risk_score: u8,
    /// Risk factors used in calculation
    pub risk_factors: RiskFactors,
    /// Whether approval was required
    pub approval_required: bool,
    /// Approval decision if required
    pub approval_decision: Option<ApprovalDecisionRecord>,
}

/// Record of an approval decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecisionRecord {
    /// Whether the step was approved
    pub approved: bool,
    /// Timestamp of the decision
    pub timestamp: DateTime<Utc>,
    /// Approver identifier (if available)
    pub approver: Option<String>,
    /// Approval comments
    pub comments: Option<String>,
}

/// Risk assessment report for a completed workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentReport {
    /// Workflow identifier
    pub workflow_id: String,
    /// Workflow name
    pub workflow_name: String,
    /// Overall risk score (average of all steps)
    pub overall_risk_score: u8,
    /// Risk assessments for each step
    pub step_assessments: Vec<RiskAssessment>,
    /// Safety constraint violations (if any)
    pub safety_violations: Vec<SafetyViolation>,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Record of a safety constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyViolation {
    /// Step identifier
    pub step_id: String,
    /// Type of violation
    pub violation_type: String,
    /// Description of the violation
    pub description: String,
}
