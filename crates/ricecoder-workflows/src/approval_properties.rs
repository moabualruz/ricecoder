//! Property-based tests for approval gate enforcement
//!
//! **Feature: ricecoder-workflows, Property 4: Approval Gate Enforcement**
//! **Validates: Requirements 1.4**

#[cfg(test)]
mod tests {
    use crate::approval::ApprovalGate;
    use crate::models::{
        AgentStep, ErrorAction, RiskFactors, StepConfig, StepType, Workflow, WorkflowConfig,
        WorkflowStep,
    };
    use proptest::prelude::*;

    /// Strategy for generating step IDs
    fn step_id_strategy() -> impl Strategy<Value = String> {
        r"step[0-9]{1,3}".prop_map(|s| s.to_string())
    }

    /// Strategy for generating approval messages
    fn message_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9 ]{10,50}".prop_map(|s| s.to_string())
    }

    /// Strategy for generating timeout values (in milliseconds)
    fn timeout_strategy() -> impl Strategy<Value = u64> {
        1000u64..3600000u64
    }

    /// Create a test workflow with approval-required steps
    fn create_workflow_with_approval(step_ids: Vec<String>) -> Workflow {
        let steps = step_ids
            .into_iter()
            .map(|id| WorkflowStep {
                id: id.clone(),
                name: format!("Step {}", id),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "test-task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: true,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            })
            .collect();

        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps,
            config: WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        }
    }

    proptest! {
        /// Property 4: Approval Gate Enforcement
        ///
        /// For any workflow step requiring approval, execution SHALL NOT proceed
        /// until explicit approval is received, regardless of other conditions.
        ///
        /// This property tests that:
        /// 1. A step requiring approval cannot execute without approval
        /// 2. Once approved, the step can proceed
        /// 3. Once rejected, the step cannot proceed
        /// 4. Approval decisions are immutable
        #[test]
        fn prop_approval_gate_blocks_execution_until_approved(
            step_id in step_id_strategy(),
            message in message_strategy(),
            timeout_ms in timeout_strategy(),
        ) {
            let _workflow = create_workflow_with_approval(vec![step_id.clone()]);
            let mut approval_gate = ApprovalGate::new();

            // Step 1: Request approval for the step
            let request_id = approval_gate
                .request_approval(step_id.clone(), message.clone(), timeout_ms)
                .expect("Should create approval request");

            // Step 2: Verify that approval is pending (step cannot execute)
            let is_pending = approval_gate
                .is_pending(&request_id)
                .expect("Should check pending status");
            prop_assert!(is_pending, "Approval should be pending initially");

            // Step 3: Verify that step is not approved yet
            let is_approved = approval_gate
                .is_approved(&request_id)
                .expect("Should check approval status");
            prop_assert!(!is_approved, "Step should not be approved yet");

            // Step 4: Approve the step
            approval_gate
                .approve(&request_id, None)
                .expect("Should approve request");

            // Step 5: Verify that approval is no longer pending
            let is_pending_after = approval_gate
                .is_pending(&request_id)
                .expect("Should check pending status after approval");
            prop_assert!(!is_pending_after, "Approval should not be pending after approval");

            // Step 6: Verify that step is now approved
            let is_approved_after = approval_gate
                .is_approved(&request_id)
                .expect("Should check approval status after approval");
            prop_assert!(is_approved_after, "Step should be approved after approval");

            // Step 7: Verify that step is not rejected
            let is_rejected = approval_gate
                .is_rejected(&request_id)
                .expect("Should check rejection status");
            prop_assert!(!is_rejected, "Step should not be rejected");
        }

        /// Property: Rejection blocks execution
        ///
        /// For any workflow step requiring approval, if the approval is rejected,
        /// the step cannot proceed.
        #[test]
        fn prop_approval_gate_blocks_execution_when_rejected(
            step_id in step_id_strategy(),
            message in message_strategy(),
            timeout_ms in timeout_strategy(),
        ) {
            let _workflow = create_workflow_with_approval(vec![step_id.clone()]);
            let mut approval_gate = ApprovalGate::new();

            // Request approval
            let request_id = approval_gate
                .request_approval(step_id.clone(), message.clone(), timeout_ms)
                .expect("Should create approval request");

            // Reject the approval
            approval_gate
                .reject(&request_id, None)
                .expect("Should reject request");

            // Verify that step is rejected
            let is_rejected = approval_gate
                .is_rejected(&request_id)
                .expect("Should check rejection status");
            prop_assert!(is_rejected, "Step should be rejected");

            // Verify that step is not approved
            let is_approved = approval_gate
                .is_approved(&request_id)
                .expect("Should check approval status");
            prop_assert!(!is_approved, "Step should not be approved");

            // Verify that approval is not pending
            let is_pending = approval_gate
                .is_pending(&request_id)
                .expect("Should check pending status");
            prop_assert!(!is_pending, "Approval should not be pending after rejection");
        }

        /// Property: Approval decisions are immutable
        ///
        /// For any approval request, once a decision is made (approved or rejected),
        /// the decision cannot be changed.
        #[test]
        fn prop_approval_decisions_are_immutable(
            step_id in step_id_strategy(),
            message in message_strategy(),
            timeout_ms in timeout_strategy(),
        ) {
            let mut approval_gate = ApprovalGate::new();

            // Request approval
            let request_id = approval_gate
                .request_approval(step_id.clone(), message.clone(), timeout_ms)
                .expect("Should create approval request");

            // Approve the request
            approval_gate
                .approve(&request_id, None)
                .expect("Should approve request");

            // Try to approve again - should fail
            let result = approval_gate.approve(&request_id, None);
            prop_assert!(result.is_err(), "Should not allow approving twice");

            // Try to reject after approving - should fail
            let result = approval_gate.reject(&request_id, None);
            prop_assert!(result.is_err(), "Should not allow rejecting after approving");
        }

        /// Property: Multiple approval requests can coexist
        ///
        /// For any workflow with multiple steps requiring approval, each step
        /// can have independent approval requests that don't interfere with each other.
        #[test]
        fn prop_multiple_approval_requests_independent(
            step_ids in prop::collection::vec(step_id_strategy(), 2..5),
            message in message_strategy(),
            timeout_ms in timeout_strategy(),
        ) {
            let _workflow = create_workflow_with_approval(step_ids.clone());
            let mut approval_gate = ApprovalGate::new();

            // Create approval requests for all steps
            let mut request_ids = Vec::new();
            for step_id in &step_ids {
                let request_id = approval_gate
                    .request_approval(step_id.clone(), message.clone(), timeout_ms)
                    .expect("Should create approval request");
                request_ids.push(request_id);
            }

            // Verify all requests are pending
            for request_id in &request_ids {
                let is_pending = approval_gate
                    .is_pending(request_id)
                    .expect("Should check pending status");
                prop_assert!(is_pending, "All requests should be pending initially");
            }

            // Approve the first request
            approval_gate
                .approve(&request_ids[0], None)
                .expect("Should approve first request");

            // Verify first is approved, others are still pending
            let is_approved = approval_gate
                .is_approved(&request_ids[0])
                .expect("Should check approval status");
            prop_assert!(is_approved, "First request should be approved");

            for request_id in &request_ids[1..] {
                let is_pending = approval_gate
                    .is_pending(request_id)
                    .expect("Should check pending status");
                prop_assert!(is_pending, "Other requests should still be pending");
            }

            // Reject the second request
            if request_ids.len() > 1 {
                approval_gate
                    .reject(&request_ids[1], None)
                    .expect("Should reject second request");

                let is_rejected = approval_gate
                    .is_rejected(&request_ids[1])
                    .expect("Should check rejection status");
                prop_assert!(is_rejected, "Second request should be rejected");
            }
        }

        /// Property: Approval comments are preserved
        ///
        /// For any approval request, if comments are provided during approval,
        /// they should be stored and retrievable.
        #[test]
        fn prop_approval_comments_preserved(
            step_id in step_id_strategy(),
            message in message_strategy(),
            timeout_ms in timeout_strategy(),
            comments in r"[a-zA-Z0-9 ]{5,50}",
        ) {
            let mut approval_gate = ApprovalGate::new();

            // Request approval
            let request_id = approval_gate
                .request_approval(step_id.clone(), message.clone(), timeout_ms)
                .expect("Should create approval request");

            // Approve with comments
            approval_gate
                .approve(&request_id, Some(comments.clone()))
                .expect("Should approve request");

            // Get the request and verify comments
            let request = approval_gate
                .get_request_status(&request_id)
                .expect("Should get request status");

            prop_assert_eq!(request.comments, Some(comments), "Comments should be preserved");
        }
    }
}
