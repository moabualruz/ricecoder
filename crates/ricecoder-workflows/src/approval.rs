//! Approval gate implementation for workflow steps

use crate::error::{WorkflowError, WorkflowResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Approval decision for a step
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalDecision {
    /// Step was approved
    #[serde(rename = "approved")]
    Approved,
    /// Step was rejected
    #[serde(rename = "rejected")]
    Rejected,
}

/// Approval request for a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique approval request ID
    pub id: String,
    /// Step ID requiring approval
    pub step_id: String,
    /// Approval message
    pub message: String,
    /// Request creation time
    pub created_at: DateTime<Utc>,
    /// Request timeout
    pub timeout_ms: u64,
    /// Whether approval has been received
    pub approved: bool,
    /// Approval decision (if received)
    pub decision: Option<ApprovalDecision>,
    /// Approval timestamp (if received)
    pub approved_at: Option<DateTime<Utc>>,
    /// Approval comments
    pub comments: Option<String>,
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(step_id: String, message: String, timeout_ms: u64) -> Self {
        ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            step_id,
            message,
            created_at: Utc::now(),
            timeout_ms,
            approved: false,
            decision: None,
            approved_at: None,
            comments: None,
        }
    }

    /// Check if the approval request has timed out
    pub fn is_timed_out(&self) -> bool {
        let timeout_duration = Duration::milliseconds(self.timeout_ms as i64);
        Utc::now() > self.created_at + timeout_duration
    }

    /// Check if the approval request is still pending
    pub fn is_pending(&self) -> bool {
        !self.approved && !self.is_timed_out()
    }

    /// Approve the request
    pub fn approve(&mut self, comments: Option<String>) {
        self.approved = true;
        self.decision = Some(ApprovalDecision::Approved);
        self.approved_at = Some(Utc::now());
        self.comments = comments;
    }

    /// Reject the request
    pub fn reject(&mut self, comments: Option<String>) {
        self.approved = true;
        self.decision = Some(ApprovalDecision::Rejected);
        self.approved_at = Some(Utc::now());
        self.comments = comments;
    }
}

/// Manages approval gates for workflow steps
pub struct ApprovalGate {
    /// Active approval requests
    requests: HashMap<String, ApprovalRequest>,
}

impl Default for ApprovalGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalGate {
    /// Create a new approval gate manager
    pub fn new() -> Self {
        ApprovalGate {
            requests: HashMap::new(),
        }
    }

    /// Request approval for a step
    ///
    /// Creates an approval request and returns the request ID.
    /// The request will timeout after the specified duration.
    pub fn request_approval(
        &mut self,
        step_id: String,
        message: String,
        timeout_ms: u64,
    ) -> WorkflowResult<String> {
        let request = ApprovalRequest::new(step_id, message, timeout_ms);
        let request_id = request.id.clone();
        self.requests.insert(request_id.clone(), request);
        Ok(request_id)
    }

    /// Approve a pending request
    ///
    /// Marks the approval request as approved.
    /// Returns error if the request is not found or already decided.
    pub fn approve(&mut self, request_id: &str, comments: Option<String>) -> WorkflowResult<()> {
        let request = self.requests.get_mut(request_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Approval request not found: {}", request_id))
        })?;

        if request.approved {
            return Err(WorkflowError::Invalid(format!(
                "Approval request already decided: {}",
                request_id
            )));
        }

        if request.is_timed_out() {
            return Err(WorkflowError::ApprovalTimeout);
        }

        request.approve(comments);
        Ok(())
    }

    /// Reject a pending request
    ///
    /// Marks the approval request as rejected.
    /// Returns error if the request is not found or already decided.
    pub fn reject(&mut self, request_id: &str, comments: Option<String>) -> WorkflowResult<()> {
        let request = self.requests.get_mut(request_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Approval request not found: {}", request_id))
        })?;

        if request.approved {
            return Err(WorkflowError::Invalid(format!(
                "Approval request already decided: {}",
                request_id
            )));
        }

        if request.is_timed_out() {
            return Err(WorkflowError::ApprovalTimeout);
        }

        request.reject(comments);
        Ok(())
    }

    /// Get the status of an approval request
    pub fn get_request_status(&self, request_id: &str) -> WorkflowResult<ApprovalRequest> {
        self.requests.get(request_id).cloned().ok_or_else(|| {
            WorkflowError::NotFound(format!("Approval request not found: {}", request_id))
        })
    }

    /// Check if a step is approved
    ///
    /// Returns true if the step has been approved, false if rejected or pending.
    /// Returns error if the request is not found or timed out.
    pub fn is_approved(&self, request_id: &str) -> WorkflowResult<bool> {
        let request = self.get_request_status(request_id)?;

        if request.is_timed_out() {
            return Err(WorkflowError::ApprovalTimeout);
        }

        if !request.approved {
            return Ok(false);
        }

        Ok(request.decision == Some(ApprovalDecision::Approved))
    }

    /// Check if a step is rejected
    ///
    /// Returns true if the step has been rejected, false if approved or pending.
    /// Returns error if the request is not found or timed out.
    pub fn is_rejected(&self, request_id: &str) -> WorkflowResult<bool> {
        let request = self.get_request_status(request_id)?;

        if request.is_timed_out() {
            return Err(WorkflowError::ApprovalTimeout);
        }

        if !request.approved {
            return Ok(false);
        }

        Ok(request.decision == Some(ApprovalDecision::Rejected))
    }

    /// Check if a request is still pending
    pub fn is_pending(&self, request_id: &str) -> WorkflowResult<bool> {
        let request = self.get_request_status(request_id)?;
        Ok(request.is_pending())
    }

    /// Get all pending requests
    pub fn get_pending_requests(&self) -> Vec<ApprovalRequest> {
        self.requests
            .values()
            .filter(|r| r.is_pending())
            .cloned()
            .collect()
    }

    /// Get all requests for a specific step
    pub fn get_step_requests(&self, step_id: &str) -> Vec<ApprovalRequest> {
        self.requests
            .values()
            .filter(|r| r.step_id == step_id)
            .cloned()
            .collect()
    }

    /// Clear all requests (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.requests.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_approval_request() {
        let request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            5000,
        );

        assert_eq!(request.step_id, "step1");
        assert_eq!(request.message, "Please approve this step");
        assert_eq!(request.timeout_ms, 5000);
        assert!(!request.approved);
        assert!(request.is_pending());
    }

    #[test]
    fn test_approval_request_timeout() {
        let request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            1, // 1ms timeout
        );

        // Wait a bit to ensure timeout
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(request.is_timed_out());
        assert!(!request.is_pending());
    }

    #[test]
    fn test_approve_request() {
        let mut request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            5000,
        );

        request.approve(Some("Looks good".to_string()));

        assert!(request.approved);
        assert_eq!(request.decision, Some(ApprovalDecision::Approved));
        assert_eq!(request.comments, Some("Looks good".to_string()));
        assert!(!request.is_pending());
    }

    #[test]
    fn test_reject_request() {
        let mut request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            5000,
        );

        request.reject(Some("Needs changes".to_string()));

        assert!(request.approved);
        assert_eq!(request.decision, Some(ApprovalDecision::Rejected));
        assert_eq!(request.comments, Some("Needs changes".to_string()));
        assert!(!request.is_pending());
    }

    #[test]
    fn test_approval_gate_request_approval() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        assert!(!request_id.is_empty());
        assert_eq!(gate.get_pending_requests().len(), 1);
    }

    #[test]
    fn test_approval_gate_approve() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        gate.approve(&request_id, Some("Approved".to_string()))
            .unwrap();

        assert!(gate.is_approved(&request_id).unwrap());
        assert!(!gate.is_rejected(&request_id).unwrap());
        assert!(!gate.is_pending(&request_id).unwrap());
    }

    #[test]
    fn test_approval_gate_reject() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        gate.reject(&request_id, Some("Rejected".to_string()))
            .unwrap();

        assert!(!gate.is_approved(&request_id).unwrap());
        assert!(gate.is_rejected(&request_id).unwrap());
        assert!(!gate.is_pending(&request_id).unwrap());
    }

    #[test]
    fn test_approval_gate_get_step_requests() {
        let mut gate = ApprovalGate::new();

        let _req1 = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        let _req2 = gate
            .request_approval(
                "step1".to_string(),
                "Please approve again".to_string(),
                5000,
            )
            .unwrap();

        let _req3 = gate
            .request_approval("step2".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        let step1_requests = gate.get_step_requests("step1");
        assert_eq!(step1_requests.len(), 2);

        let step2_requests = gate.get_step_requests("step2");
        assert_eq!(step2_requests.len(), 1);
    }

    #[test]
    fn test_approval_gate_cannot_approve_twice() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        gate.approve(&request_id, None).unwrap();

        let result = gate.approve(&request_id, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_approval_gate_cannot_approve_after_timeout() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval(
                "step1".to_string(),
                "Please approve".to_string(),
                1, // 1ms timeout
            )
            .unwrap();

        // Wait for timeout
        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = gate.approve(&request_id, None);
        assert!(result.is_err());
    }
}
