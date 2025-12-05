//! Approval management for execution plans
//!
//! Wraps the ApprovalGate from workflows and provides high-level approval
//! management for execution plans. Handles approval requests, decisions,
//! and enforcement of approval gates based on risk level.

use crate::error::{ExecutionError, ExecutionResult};
use crate::models::{ExecutionPlan, RiskLevel};
use ricecoder_workflows::approval::{ApprovalGate, ApprovalRequest};
use std::collections::HashMap;

/// Approval summary for a plan
#[derive(Debug, Clone)]
pub struct ApprovalSummary {
    /// Plan ID
    pub plan_id: String,
    /// Plan name
    pub plan_name: String,
    /// Number of steps
    pub step_count: usize,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Risk score
    pub risk_score: f32,
    /// Risk factors description
    pub risk_factors: String,
    /// Estimated duration in seconds
    pub estimated_duration_secs: u64,
    /// Whether approval is required
    pub requires_approval: bool,
}

/// Approval manager for execution plans
///
/// Manages approval requests and decisions for execution plans.
/// Wraps the ApprovalGate from workflows and provides plan-specific
/// approval management.
pub struct ApprovalManager {
    /// Underlying approval gate
    gate: ApprovalGate,
    /// Map of plan IDs to approval request IDs
    plan_requests: HashMap<String, String>,
    /// Map of approval request IDs to plan IDs
    request_plans: HashMap<String, String>,
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalManager {
    /// Create a new approval manager
    pub fn new() -> Self {
        ApprovalManager {
            gate: ApprovalGate::new(),
            plan_requests: HashMap::new(),
            request_plans: HashMap::new(),
        }
    }

    /// Request approval for a plan
    ///
    /// Creates an approval request for the plan and returns the request ID.
    /// The request will timeout after 30 minutes (1800000 ms).
    pub fn request_approval(&mut self, plan: &ExecutionPlan) -> ExecutionResult<String> {
        let summary = ApprovalSummary::from_plan(plan);
        let message = summary.format_message();

        let request_id = self
            .gate
            .request_approval(plan.id.clone(), message, 1_800_000) // 30 minutes
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to request approval: {}", e)))?;

        self.plan_requests.insert(plan.id.clone(), request_id.clone());
        self.request_plans.insert(request_id.clone(), plan.id.clone());

        tracing::info!(
            plan_id = %plan.id,
            request_id = %request_id,
            risk_level = ?plan.risk_score.level,
            "Approval requested for plan"
        );

        Ok(request_id)
    }

    /// Approve a plan
    ///
    /// Marks the approval request as approved.
    pub fn approve(&mut self, request_id: &str, comments: Option<String>) -> ExecutionResult<()> {
        self.gate
            .approve(request_id, comments.clone())
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to approve: {}", e)))?;

        if let Some(plan_id) = self.request_plans.get(request_id) {
            tracing::info!(
                plan_id = %plan_id,
                request_id = %request_id,
                comments = ?comments,
                "Plan approved"
            );
        }

        Ok(())
    }

    /// Reject a plan
    ///
    /// Marks the approval request as rejected.
    pub fn reject(&mut self, request_id: &str, comments: Option<String>) -> ExecutionResult<()> {
        self.gate
            .reject(request_id, comments.clone())
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to reject: {}", e)))?;

        if let Some(plan_id) = self.request_plans.get(request_id) {
            tracing::info!(
                plan_id = %plan_id,
                request_id = %request_id,
                comments = ?comments,
                "Plan rejected"
            );
        }

        Ok(())
    }

    /// Check if a plan is approved
    ///
    /// Returns true if the plan has been approved, false if rejected or pending.
    pub fn is_approved(&self, request_id: &str) -> ExecutionResult<bool> {
        self.gate
            .is_approved(request_id)
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to check approval status: {}", e)))
    }

    /// Check if a plan is rejected
    ///
    /// Returns true if the plan has been rejected, false if approved or pending.
    pub fn is_rejected(&self, request_id: &str) -> ExecutionResult<bool> {
        self.gate
            .is_rejected(request_id)
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to check rejection status: {}", e)))
    }

    /// Check if a request is still pending
    pub fn is_pending(&self, request_id: &str) -> ExecutionResult<bool> {
        self.gate
            .is_pending(request_id)
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to check pending status: {}", e)))
    }

    /// Get the approval request details
    pub fn get_request(&self, request_id: &str) -> ExecutionResult<ApprovalRequest> {
        self.gate
            .get_request_status(request_id)
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to get request status: {}", e)))
    }

    /// Get all pending approval requests
    pub fn get_pending_requests(&self) -> Vec<ApprovalRequest> {
        self.gate.get_pending_requests()
    }

    /// Get the approval request ID for a plan
    pub fn get_request_id(&self, plan_id: &str) -> Option<String> {
        self.plan_requests.get(plan_id).cloned()
    }

    /// Determine if approval is required based on risk level
    ///
    /// Returns true if approval is required for the given risk level.
    pub fn approval_required(risk_level: RiskLevel) -> bool {
        matches!(risk_level, RiskLevel::High | RiskLevel::Critical)
    }

    /// Determine if approval is strongly recommended based on risk level
    ///
    /// Returns true if approval is strongly recommended (Critical risk).
    pub fn approval_strongly_recommended(risk_level: RiskLevel) -> bool {
        matches!(risk_level, RiskLevel::Critical)
    }
}

impl ApprovalSummary {
    /// Create an approval summary from a plan
    pub fn from_plan(plan: &ExecutionPlan) -> Self {
        let risk_factors = plan
            .risk_score
            .factors
            .iter()
            .map(|f| format!("- {}: {}", f.name, f.description))
            .collect::<Vec<_>>()
            .join("\n");

        ApprovalSummary {
            plan_id: plan.id.clone(),
            plan_name: plan.name.clone(),
            step_count: plan.steps.len(),
            risk_level: plan.risk_score.level,
            risk_score: plan.risk_score.score,
            risk_factors,
            estimated_duration_secs: plan.estimated_duration.as_secs(),
            requires_approval: plan.requires_approval,
        }
    }

    /// Format the approval summary as a message
    pub fn format_message(&self) -> String {
        format!(
            "Plan: {}\nSteps: {}\nRisk Level: {:?}\nRisk Score: {:.2}\nEstimated Duration: {}s\n\nRisk Factors:\n{}",
            self.plan_name,
            self.step_count,
            self.risk_level,
            self.risk_score,
            self.estimated_duration_secs,
            self.risk_factors
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ExecutionStep, RiskFactor, RiskScore, StepAction};

    fn create_test_plan() -> ExecutionPlan {
        let step = ExecutionStep::new(
            "Test step".to_string(),
            StepAction::CreateFile {
                path: "test.txt".to_string(),
                content: "test".to_string(),
            },
        );

        let mut plan = ExecutionPlan::new("Test Plan".to_string(), vec![step]);
        plan.risk_score = RiskScore {
            level: RiskLevel::High,
            score: 1.8,
            factors: vec![RiskFactor {
                name: "file_count".to_string(),
                weight: 0.5,
                description: "1 file modified".to_string(),
            }],
        };
        plan.requires_approval = true;

        plan
    }

    #[test]
    fn test_create_approval_manager() {
        let manager = ApprovalManager::new();
        assert_eq!(manager.plan_requests.len(), 0);
        assert_eq!(manager.request_plans.len(), 0);
    }

    #[test]
    fn test_request_approval() {
        let mut manager = ApprovalManager::new();
        let plan = create_test_plan();

        let request_id = manager.request_approval(&plan).unwrap();
        assert!(!request_id.is_empty());
        assert_eq!(manager.plan_requests.len(), 1);
        assert_eq!(manager.request_plans.len(), 1);
    }

    #[test]
    fn test_approve_plan() {
        let mut manager = ApprovalManager::new();
        let plan = create_test_plan();

        let request_id = manager.request_approval(&plan).unwrap();
        manager
            .approve(&request_id, Some("Looks good".to_string()))
            .unwrap();

        assert!(manager.is_approved(&request_id).unwrap());
        assert!(!manager.is_rejected(&request_id).unwrap());
        assert!(!manager.is_pending(&request_id).unwrap());
    }

    #[test]
    fn test_reject_plan() {
        let mut manager = ApprovalManager::new();
        let plan = create_test_plan();

        let request_id = manager.request_approval(&plan).unwrap();
        manager
            .reject(&request_id, Some("Needs changes".to_string()))
            .unwrap();

        assert!(!manager.is_approved(&request_id).unwrap());
        assert!(manager.is_rejected(&request_id).unwrap());
        assert!(!manager.is_pending(&request_id).unwrap());
    }

    #[test]
    fn test_get_request_id() {
        let mut manager = ApprovalManager::new();
        let plan = create_test_plan();
        let plan_id = plan.id.clone();

        let request_id = manager.request_approval(&plan).unwrap();
        assert_eq!(manager.get_request_id(&plan_id), Some(request_id));
    }

    #[test]
    fn test_approval_required() {
        assert!(!ApprovalManager::approval_required(RiskLevel::Low));
        assert!(!ApprovalManager::approval_required(RiskLevel::Medium));
        assert!(ApprovalManager::approval_required(RiskLevel::High));
        assert!(ApprovalManager::approval_required(RiskLevel::Critical));
    }

    #[test]
    fn test_approval_strongly_recommended() {
        assert!(!ApprovalManager::approval_strongly_recommended(RiskLevel::Low));
        assert!(!ApprovalManager::approval_strongly_recommended(RiskLevel::Medium));
        assert!(!ApprovalManager::approval_strongly_recommended(RiskLevel::High));
        assert!(ApprovalManager::approval_strongly_recommended(RiskLevel::Critical));
    }

    #[test]
    fn test_approval_summary_from_plan() {
        let plan = create_test_plan();
        let summary = ApprovalSummary::from_plan(&plan);

        assert_eq!(summary.plan_id, plan.id);
        assert_eq!(summary.plan_name, "Test Plan");
        assert_eq!(summary.step_count, 1);
        assert_eq!(summary.risk_level, RiskLevel::High);
        assert!(summary.requires_approval);
    }

    #[test]
    fn test_approval_summary_format_message() {
        let plan = create_test_plan();
        let summary = ApprovalSummary::from_plan(&plan);
        let message = summary.format_message();

        assert!(message.contains("Test Plan"));
        assert!(message.contains("Steps: 1"));
        assert!(message.contains("High"));
        assert!(message.contains("Risk Factors"));
    }

    #[test]
    fn test_get_pending_requests() {
        let mut manager = ApprovalManager::new();
        let plan1 = create_test_plan();
        let plan2 = create_test_plan();

        let _req1 = manager.request_approval(&plan1).unwrap();
        let _req2 = manager.request_approval(&plan2).unwrap();

        let pending = manager.get_pending_requests();
        assert_eq!(pending.len(), 2);
    }
}
