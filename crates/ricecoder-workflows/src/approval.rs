//! Approval management for workflows
//!
//! Provides approval gates and request management for workflow execution.
//! Handles human-in-the-loop approvals with timeout and status tracking.

use std::{collections::HashMap, sync::RwLock};

use chrono::{DateTime, Duration, Utc};

use crate::error::{WorkflowError, WorkflowResult};

/// Approval status
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalStatus {
    /// Approval is pending
    Pending,
    /// Approval was granted
    Approved {
        /// ID of the approver
        approver_id: String,
        /// When it was approved
        approved_at: DateTime<Utc>,
        /// Optional comments
        comments: Option<String>,
    },
    /// Approval was rejected
    Rejected {
        /// ID of the rejector
        rejector_id: String,
        /// When it was rejected
        rejected_at: DateTime<Utc>,
        /// Reason for rejection
        reason: String,
    },
    /// Approval request timed out
    TimedOut {
        /// When it timed out
        timed_out_at: DateTime<Utc>,
    },
}

/// Approval request details
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    /// Unique request ID
    pub id: String,
    /// Resource ID requiring approval
    pub resource_id: String,
    /// Approval message/description
    pub message: String,
    /// When the request was made
    pub requested_at: DateTime<Utc>,
    /// Timeout duration in milliseconds
    pub timeout_ms: u64,
    /// Current status
    pub status: ApprovalStatus,
}

impl ApprovalRequest {
    /// Check if the request has timed out
    pub fn is_timed_out(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.requested_at);
        elapsed.num_milliseconds() as u64 > self.timeout_ms
    }

    /// Get the expiration time
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.requested_at + Duration::milliseconds(self.timeout_ms as i64)
    }
}

/// Approval gate for managing approval workflows
pub struct ApprovalGate {
    /// Map of request IDs to requests
    requests: RwLock<HashMap<String, ApprovalRequest>>,
}

impl Default for ApprovalGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalGate {
    /// Create a new approval gate
    pub fn new() -> Self {
        ApprovalGate {
            requests: RwLock::new(HashMap::new()),
        }
    }

    /// Request approval for a resource
    pub fn request_approval(
        &self,
        resource_id: String,
        message: String,
        timeout_ms: u64,
    ) -> WorkflowResult<String> {
        let request_id = format!("approval_{}_{}", resource_id, Utc::now().timestamp_millis());

        let request = ApprovalRequest {
            id: request_id.clone(),
            resource_id,
            message,
            requested_at: Utc::now(),
            timeout_ms,
            status: ApprovalStatus::Pending,
        };

        let mut requests = self
            .requests
            .write()
            .map_err(|_| WorkflowError::StateError("Failed to acquire write lock".to_string()))?;

        requests.insert(request_id.clone(), request);

        Ok(request_id)
    }

    /// Approve a request
    pub fn approve(&self, request_id: &str, comments: Option<String>) -> WorkflowResult<()> {
        let mut requests = self
            .requests
            .write()
            .map_err(|_| WorkflowError::StateError("Failed to acquire write lock".to_string()))?;

        if let Some(request) = requests.get_mut(request_id) {
            if request.is_timed_out() {
                request.status = ApprovalStatus::TimedOut {
                    timed_out_at: Utc::now(),
                };
                return Err(WorkflowError::Invalid(
                    "Approval request has timed out".to_string(),
                ));
            }

            if !matches!(request.status, ApprovalStatus::Pending) {
                return Err(WorkflowError::Invalid("Request is not pending".to_string()));
            }

            request.status = ApprovalStatus::Approved {
                approver_id: "system".to_string(), // TODO: Get actual approver ID
                approved_at: Utc::now(),
                comments,
            };

            Ok(())
        } else {
            Err(WorkflowError::Invalid(format!(
                "Approval request {} not found",
                request_id
            )))
        }
    }

    /// Reject a request
    pub fn reject(&self, request_id: &str, comments: Option<String>) -> WorkflowResult<()> {
        let mut requests = self
            .requests
            .write()
            .map_err(|_| WorkflowError::StateError("Failed to acquire write lock".to_string()))?;

        if let Some(request) = requests.get_mut(request_id) {
            if request.is_timed_out() {
                request.status = ApprovalStatus::TimedOut {
                    timed_out_at: Utc::now(),
                };
                return Err(WorkflowError::Invalid(
                    "Approval request has timed out".to_string(),
                ));
            }

            if !matches!(request.status, ApprovalStatus::Pending) {
                return Err(WorkflowError::Invalid("Request is not pending".to_string()));
            }

            request.status = ApprovalStatus::Rejected {
                rejector_id: "system".to_string(), // TODO: Get actual rejector ID
                rejected_at: Utc::now(),
                reason: comments.unwrap_or_else(|| "No reason provided".to_string()),
            };

            Ok(())
        } else {
            Err(WorkflowError::Invalid(format!(
                "Approval request {} not found",
                request_id
            )))
        }
    }

    /// Check if a request is approved
    pub fn is_approved(&self, request_id: &str) -> WorkflowResult<bool> {
        let requests = self
            .requests
            .read()
            .map_err(|_| WorkflowError::StateError("Failed to acquire read lock".to_string()))?;

        if let Some(request) = requests.get(request_id) {
            if request.is_timed_out() {
                return Ok(false);
            }
            Ok(matches!(request.status, ApprovalStatus::Approved { .. }))
        } else {
            Err(WorkflowError::Invalid(format!(
                "Approval request {} not found",
                request_id
            )))
        }
    }

    /// Check if a request is rejected
    pub fn is_rejected(&self, request_id: &str) -> WorkflowResult<bool> {
        let requests = self
            .requests
            .read()
            .map_err(|_| WorkflowError::StateError("Failed to acquire read lock".to_string()))?;

        if let Some(request) = requests.get(request_id) {
            if request.is_timed_out() {
                return Ok(false);
            }
            Ok(matches!(request.status, ApprovalStatus::Rejected { .. }))
        } else {
            Err(WorkflowError::Invalid(format!(
                "Approval request {} not found",
                request_id
            )))
        }
    }

    /// Check if a request is still pending
    pub fn is_pending(&self, request_id: &str) -> WorkflowResult<bool> {
        let requests = self
            .requests
            .read()
            .map_err(|_| WorkflowError::StateError("Failed to acquire read lock".to_string()))?;

        if let Some(request) = requests.get(request_id) {
            if request.is_timed_out() {
                return Ok(false);
            }
            Ok(matches!(request.status, ApprovalStatus::Pending))
        } else {
            Err(WorkflowError::Invalid(format!(
                "Approval request {} not found",
                request_id
            )))
        }
    }

    /// Get the request status
    pub fn get_request_status(&self, request_id: &str) -> WorkflowResult<ApprovalRequest> {
        let requests = self
            .requests
            .read()
            .map_err(|_| WorkflowError::StateError("Failed to acquire read lock".to_string()))?;

        if let Some(request) = requests.get(request_id) {
            Ok(request.clone())
        } else {
            Err(WorkflowError::Invalid(format!(
                "Approval request {} not found",
                request_id
            )))
        }
    }

    /// Get all pending requests
    pub fn get_pending_requests(&self) -> Vec<ApprovalRequest> {
        if let Ok(requests) = self.requests.read() {
            requests
                .values()
                .filter(|req| matches!(req.status, ApprovalStatus::Pending) && !req.is_timed_out())
                .cloned()
                .collect()
        } else {
            eprintln!("Failed to acquire read lock, returning empty list");
            Vec::new()
        }
    }

    /// Clean up timed out requests
    pub fn cleanup_timed_out(&self) -> usize {
        if let Ok(mut requests) = self.requests.write() {
            let mut cleaned = 0;
            requests.retain(|_, req| {
                if req.is_timed_out() && matches!(req.status, ApprovalStatus::Pending) {
                    cleaned += 1;
                    false
                } else {
                    true
                }
            });
            cleaned
        } else {
            eprintln!("Failed to acquire write lock, skipping cleanup");
            0
        }
    }
}
