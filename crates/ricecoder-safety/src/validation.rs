//! Safety validation and approval gates

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    constraints::{ConstraintResult, SecurityConstraint, ValidationContext},
    error::{SafetyError, SafetyResult},
    risk::{RiskContext, RiskScore, RiskScorer},
};

/// Safety validator for operations and workflows
pub struct SafetyValidator {
    constraints: RwLock<Vec<SecurityConstraint>>,
    risk_scorer: Arc<RiskScorer>,
    approval_gates: RwLock<HashMap<String, ApprovalGate>>,
}

impl SafetyValidator {
    /// Create a new safety validator with injected dependencies
    pub fn new_with_scorer(risk_scorer: Arc<RiskScorer>) -> Self {
        Self {
            constraints: RwLock::new(Vec::new()),
            risk_scorer,
            approval_gates: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new safety validator with default dependencies
    pub fn new() -> Self {
        Self::new_with_scorer(Arc::new(RiskScorer::new()))
    }

    /// Create with defaults (alias for new)
    pub fn with_defaults() -> Self {
        Self::new()
    }

    /// Add a security constraint
    pub async fn add_constraint(&self, constraint: SecurityConstraint) -> SafetyResult<()> {
        self.constraints.write().await.push(constraint);
        Ok(())
    }

    /// Remove a security constraint
    pub async fn remove_constraint(&self, constraint_id: &str) -> SafetyResult<()> {
        let mut constraints = self.constraints.write().await;
        constraints.retain(|c| c.id != constraint_id);
        Ok(())
    }

    /// Get all constraints
    pub async fn get_constraints(&self) -> Vec<SecurityConstraint> {
        self.constraints.read().await.clone()
    }

    /// Validate an operation against all constraints
    pub async fn validate_operation(
        &self,
        context: &ValidationContext,
    ) -> SafetyResult<ValidationResult> {
        let constraints = self.constraints.read().await;
        let mut violations = Vec::new();
        let mut approval_required = Vec::new();

        for constraint in constraints.iter() {
            match constraint.validate(context).await? {
                ConstraintResult::Passed => continue,
                ConstraintResult::Failed(reason) => {
                    violations.push(ValidationViolation {
                        constraint_id: constraint.id.clone(),
                        constraint_name: constraint.name.clone(),
                        severity: constraint.severity,
                        reason,
                    });
                }
                ConstraintResult::ApprovalRequired(reason) => {
                    approval_required.push(ApprovalRequest {
                        constraint_id: constraint.id.clone(),
                        reason,
                        requested_at: chrono::Utc::now(),
                        context: None,
                    });
                }
            }
        }

        // Perform risk assessment
        let risk_score = if let Some(user_id) = &context.user_id {
            let risk_context = RiskContext {
                user_id: Some(user_id.clone()),
                operation_type: Some("operation_validation".to_string()),
                timestamp: Some(chrono::Utc::now()),
                ..Default::default()
            };

            Some(
                self.risk_scorer
                    .score_action("validate_operation", &risk_context)?,
            )
        } else {
            None
        };

        if violations.is_empty() && approval_required.is_empty() {
            Ok(ValidationResult::Passed {
                risk_score,
                message: "All safety checks passed".to_string(),
            })
        } else if !violations.is_empty() {
            Ok(ValidationResult::Failed {
                violations: violations.clone(),
                risk_score,
                message: format!("{} safety violations detected", violations.len()),
            })
        } else {
            Ok(ValidationResult::ApprovalRequired {
                requests: approval_required,
                risk_score,
                message: "Manual approval required for operation".to_string(),
            })
        }
    }

    /// Validate a workflow execution
    pub async fn validate_workflow(
        &self,
        workflow_context: &WorkflowValidationContext,
    ) -> SafetyResult<ValidationResult> {
        // Convert workflow context to operation context
        let operation_context = ValidationContext {
            user_id: workflow_context.user_id.clone(),
            session_id: workflow_context.session_id.clone(),
            estimated_execution_time_seconds: Some(workflow_context.estimated_duration_seconds),
            estimated_memory_bytes: Some(workflow_context.estimated_memory_bytes),
            additional_data: workflow_context.workflow_metadata.clone(),
            ..Default::default()
        };

        // Add workflow-specific validation
        let mut context = operation_context;

        // Check for dangerous workflow patterns
        if workflow_context.step_count > 50 {
            // Large workflows might need approval
            context
                .additional_data
                .insert("large_workflow".to_string(), serde_json::json!(true));
        }

        if workflow_context.contains_dangerous_operations {
            context
                .additional_data
                .insert("dangerous_operations".to_string(), serde_json::json!(true));
        }

        self.validate_operation(&context).await
    }

    /// Request approval for an operation
    pub async fn request_approval(&self, request: ApprovalRequest) -> SafetyResult<String> {
        let gate = ApprovalGate::new(request.constraint_id.clone());
        let request_id = gate.add_request(request).await?;

        self.approval_gates
            .write()
            .await
            .insert(request_id.clone(), gate);
        Ok(request_id)
    }

    /// Approve a pending request
    pub async fn approve_request(&self, request_id: &str, approver_id: &str) -> SafetyResult<()> {
        let mut gates = self.approval_gates.write().await;
        if let Some(gate) = gates.get_mut(request_id) {
            gate.approve(approver_id).await?;
        } else {
            return Err(SafetyError::ValidationError {
                field: "request_id".to_string(),
                message: "Approval request not found".to_string(),
            });
        }
        Ok(())
    }

    /// Reject a pending request
    pub async fn reject_request(
        &self,
        request_id: &str,
        approver_id: &str,
        reason: String,
    ) -> SafetyResult<()> {
        let mut gates = self.approval_gates.write().await;
        if let Some(gate) = gates.get_mut(request_id) {
            gate.reject(approver_id, reason).await?;
        } else {
            return Err(SafetyError::ValidationError {
                field: "request_id".to_string(),
                message: "Approval request not found".to_string(),
            });
        }
        Ok(())
    }

    /// Get pending approval requests
    pub async fn get_pending_approvals(&self) -> Vec<(String, ApprovalRequest)> {
        let gates = self.approval_gates.read().await;
        let mut pending = Vec::new();

        for (gate_id, gate) in gates.iter() {
            if let Some(request) = gate.get_pending_request().await {
                pending.push((gate_id.clone(), request));
            }
        }

        pending
    }

    /// Get risk scorer for direct access
    pub fn risk_scorer(&self) -> Arc<RiskScorer> {
        Arc::clone(&self.risk_scorer)
    }
}

/// Result of validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Validation passed
    Passed {
        risk_score: Option<crate::risk::RiskScore>,
        message: String,
    },
    /// Validation failed with violations
    Failed {
        violations: Vec<ValidationViolation>,
        risk_score: Option<crate::risk::RiskScore>,
        message: String,
    },
    /// Manual approval required
    ApprovalRequired {
        requests: Vec<ApprovalRequest>,
        risk_score: Option<crate::risk::RiskScore>,
        message: String,
    },
}

/// Validation violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    /// Constraint that was violated
    pub constraint_id: String,
    /// Human-readable constraint name
    pub constraint_name: String,
    /// Severity of the violation
    pub severity: crate::constraints::ConstraintSeverity,
    /// Reason for the violation
    pub reason: String,
}

/// Approval request details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Constraint requiring approval
    pub constraint_id: String,
    /// Reason approval is required
    pub reason: String,
    /// When the request was made
    pub requested_at: chrono::DateTime<chrono::Utc>,
    /// Additional context
    pub context: Option<HashMap<String, serde_json::Value>>,
}

/// Approval gate for managing approval workflows
pub struct ApprovalGate {
    constraint_id: String,
    pending_request: RwLock<Option<ApprovalRequest>>,
    approval_status: RwLock<ApprovalStatus>,
}

impl ApprovalGate {
    /// Create a new approval gate
    pub fn new(constraint_id: String) -> Self {
        Self {
            constraint_id,
            pending_request: RwLock::new(None),
            approval_status: RwLock::new(ApprovalStatus::Pending),
        }
    }

    /// Add an approval request
    pub async fn add_request(&self, request: ApprovalRequest) -> SafetyResult<String> {
        *self.pending_request.write().await = Some(request);
        *self.approval_status.write().await = ApprovalStatus::Pending;

        // Generate request ID
        let request_id = format!(
            "approval_{}_{}",
            self.constraint_id,
            chrono::Utc::now().timestamp()
        );
        Ok(request_id)
    }

    /// Approve the request
    pub async fn approve(&self, approver_id: &str) -> SafetyResult<()> {
        let mut status = self.approval_status.write().await;
        *status = ApprovalStatus::Approved {
            approver_id: approver_id.to_string(),
            approved_at: chrono::Utc::now(),
        };
        *self.pending_request.write().await = None;
        Ok(())
    }

    /// Reject the request
    pub async fn reject(&self, approver_id: &str, reason: String) -> SafetyResult<()> {
        let mut status = self.approval_status.write().await;
        *status = ApprovalStatus::Rejected {
            approver_id: approver_id.to_string(),
            reason,
            rejected_at: chrono::Utc::now(),
        };
        *self.pending_request.write().await = None;
        Ok(())
    }

    /// Get the current approval status
    pub async fn get_status(&self) -> ApprovalStatus {
        self.approval_status.read().await.clone()
    }

    /// Get the pending request if any
    pub async fn get_pending_request(&self) -> Option<ApprovalRequest> {
        self.pending_request.read().await.clone()
    }
}

/// Approval status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// Approval is pending
    Pending,
    /// Approval was granted
    Approved {
        approver_id: String,
        approved_at: chrono::DateTime<chrono::Utc>,
    },
    /// Approval was denied
    Rejected {
        approver_id: String,
        reason: String,
        rejected_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Context for workflow validation
#[derive(Debug, Clone)]
pub struct WorkflowValidationContext {
    /// User executing the workflow
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Number of steps in the workflow
    pub step_count: usize,
    /// Estimated workflow duration in seconds
    pub estimated_duration_seconds: u64,
    /// Estimated memory usage in bytes
    pub estimated_memory_bytes: u64,
    /// Whether the workflow contains dangerous operations
    pub contains_dangerous_operations: bool,
    /// Workflow metadata
    pub workflow_metadata: HashMap<String, serde_json::Value>,
}

impl Default for SafetyValidator {
    fn default() -> Self {
        Self::new()
    }
}
