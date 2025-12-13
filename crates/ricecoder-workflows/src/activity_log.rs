//! Activity logging for workflow execution

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    /// Timestamp of the activity
    pub timestamp: DateTime<Utc>,
    /// Activity type
    pub activity_type: ActivityType,
    /// Step ID (if applicable)
    pub step_id: Option<String>,
    /// Activity message
    pub message: String,
    /// Additional context
    pub context: serde_json::Value,
}

/// Type of activity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityType {
    /// Workflow started
    #[serde(rename = "workflow_started")]
    WorkflowStarted,
    /// Workflow completed
    #[serde(rename = "workflow_completed")]
    WorkflowCompleted,
    /// Workflow failed
    #[serde(rename = "workflow_failed")]
    WorkflowFailed,
    /// Workflow paused
    #[serde(rename = "workflow_paused")]
    WorkflowPaused,
    /// Workflow resumed
    #[serde(rename = "workflow_resumed")]
    WorkflowResumed,
    /// Workflow cancelled
    #[serde(rename = "workflow_cancelled")]
    WorkflowCancelled,
    /// Step started
    #[serde(rename = "step_started")]
    StepStarted,
    /// Step completed
    #[serde(rename = "step_completed")]
    StepCompleted,
    /// Step failed
    #[serde(rename = "step_failed")]
    StepFailed,
    /// Step skipped
    #[serde(rename = "step_skipped")]
    StepSkipped,
    /// Approval requested
    #[serde(rename = "approval_requested")]
    ApprovalRequested,
    /// Approval granted
    #[serde(rename = "approval_granted")]
    ApprovalGranted,
    /// Approval denied
    #[serde(rename = "approval_denied")]
    ApprovalDenied,
    /// State transition
    #[serde(rename = "state_transition")]
    StateTransition,
    /// Error occurred
    #[serde(rename = "error")]
    Error,
}

/// Activity logger for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogger {
    /// Activity log entries (limited to max_entries)
    entries: VecDeque<ActivityLogEntry>,
    /// Maximum number of entries to keep
    max_entries: usize,
}

impl ActivityLogger {
    /// Create a new activity logger
    pub fn new(max_entries: usize) -> Self {
        ActivityLogger {
            entries: VecDeque::new(),
            max_entries,
        }
    }

    /// Log an activity
    pub fn log(
        &mut self,
        activity_type: ActivityType,
        step_id: Option<String>,
        message: String,
        context: serde_json::Value,
    ) {
        let entry = ActivityLogEntry {
            timestamp: Utc::now(),
            activity_type,
            step_id,
            message,
            context,
        };

        self.entries.push_back(entry);

        // Remove oldest entry if we exceed max_entries
        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }

    /// Log workflow started
    pub fn log_workflow_started(&mut self, workflow_id: &str) {
        self.log(
            ActivityType::WorkflowStarted,
            None,
            format!("Workflow '{}' started", workflow_id),
            serde_json::json!({"workflow_id": workflow_id}),
        );
    }

    /// Log workflow completed
    pub fn log_workflow_completed(&mut self, workflow_id: &str, duration_ms: u64) {
        self.log(
            ActivityType::WorkflowCompleted,
            None,
            format!("Workflow '{}' completed in {}ms", workflow_id, duration_ms),
            serde_json::json!({"workflow_id": workflow_id, "duration_ms": duration_ms}),
        );
    }

    /// Log workflow failed
    pub fn log_workflow_failed(&mut self, workflow_id: &str, error: &str) {
        self.log(
            ActivityType::WorkflowFailed,
            None,
            format!("Workflow '{}' failed: {}", workflow_id, error),
            serde_json::json!({"workflow_id": workflow_id, "error": error}),
        );
    }

    /// Log workflow paused
    pub fn log_workflow_paused(&mut self, workflow_id: &str) {
        self.log(
            ActivityType::WorkflowPaused,
            None,
            format!("Workflow '{}' paused", workflow_id),
            serde_json::json!({"workflow_id": workflow_id}),
        );
    }

    /// Log workflow resumed
    pub fn log_workflow_resumed(&mut self, workflow_id: &str) {
        self.log(
            ActivityType::WorkflowResumed,
            None,
            format!("Workflow '{}' resumed", workflow_id),
            serde_json::json!({"workflow_id": workflow_id}),
        );
    }

    /// Log workflow cancelled
    pub fn log_workflow_cancelled(&mut self, workflow_id: &str) {
        self.log(
            ActivityType::WorkflowCancelled,
            None,
            format!("Workflow '{}' cancelled", workflow_id),
            serde_json::json!({"workflow_id": workflow_id}),
        );
    }

    /// Log step started
    pub fn log_step_started(&mut self, step_id: &str, step_name: &str) {
        self.log(
            ActivityType::StepStarted,
            Some(step_id.to_string()),
            format!("Step '{}' started", step_name),
            serde_json::json!({"step_id": step_id, "step_name": step_name}),
        );
    }

    /// Log step completed
    pub fn log_step_completed(&mut self, step_id: &str, step_name: &str, duration_ms: u64) {
        self.log(
            ActivityType::StepCompleted,
            Some(step_id.to_string()),
            format!("Step '{}' completed in {}ms", step_name, duration_ms),
            serde_json::json!({"step_id": step_id, "step_name": step_name, "duration_ms": duration_ms}),
        );
    }

    /// Log step failed
    pub fn log_step_failed(&mut self, step_id: &str, step_name: &str, error: &str) {
        self.log(
            ActivityType::StepFailed,
            Some(step_id.to_string()),
            format!("Step '{}' failed: {}", step_name, error),
            serde_json::json!({"step_id": step_id, "step_name": step_name, "error": error}),
        );
    }

    /// Log step skipped
    pub fn log_step_skipped(&mut self, step_id: &str, step_name: &str) {
        self.log(
            ActivityType::StepSkipped,
            Some(step_id.to_string()),
            format!("Step '{}' skipped", step_name),
            serde_json::json!({"step_id": step_id, "step_name": step_name}),
        );
    }

    /// Log approval requested
    pub fn log_approval_requested(&mut self, step_id: &str, message: &str) {
        self.log(
            ActivityType::ApprovalRequested,
            Some(step_id.to_string()),
            format!("Approval requested: {}", message),
            serde_json::json!({"step_id": step_id, "message": message}),
        );
    }

    /// Log approval granted
    pub fn log_approval_granted(&mut self, step_id: &str) {
        self.log(
            ActivityType::ApprovalGranted,
            Some(step_id.to_string()),
            "Approval granted".to_string(),
            serde_json::json!({"step_id": step_id}),
        );
    }

    /// Log approval denied
    pub fn log_approval_denied(&mut self, step_id: &str) {
        self.log(
            ActivityType::ApprovalDenied,
            Some(step_id.to_string()),
            "Approval denied".to_string(),
            serde_json::json!({"step_id": step_id}),
        );
    }

    /// Log state transition
    pub fn log_state_transition(&mut self, from_state: &str, to_state: &str) {
        self.log(
            ActivityType::StateTransition,
            None,
            format!("State transition: {} -> {}", from_state, to_state),
            serde_json::json!({"from_state": from_state, "to_state": to_state}),
        );
    }

    /// Log error
    pub fn log_error(&mut self, step_id: Option<&str>, error: &str) {
        self.log(
            ActivityType::Error,
            step_id.map(|s| s.to_string()),
            format!("Error: {}", error),
            serde_json::json!({"error": error}),
        );
    }

    /// Get all activity log entries
    pub fn get_entries(&self) -> Vec<ActivityLogEntry> {
        self.entries.iter().cloned().collect()
    }

    /// Get activity log entries filtered by activity type
    pub fn get_entries_by_type(&self, activity_type: ActivityType) -> Vec<ActivityLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.activity_type == activity_type)
            .cloned()
            .collect()
    }

    /// Get activity log entries for a specific step
    pub fn get_entries_for_step(&self, step_id: &str) -> Vec<ActivityLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.step_id.as_deref() == Some(step_id))
            .cloned()
            .collect()
    }

    /// Clear all activity log entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of activity log entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if activity log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}


