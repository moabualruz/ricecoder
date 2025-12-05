//! Property-based tests for phase progression enforcement
//! **Feature: ricecoder-specs, Property 7: Phase Progression Enforcement**
//! **Validates: Requirements 3.1, 3.8**

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_specs::{
    approval::ApprovalManager,
    models::{SpecPhase, SpecWritingSession},
};

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_spec_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_session_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_approver_name() -> impl Strategy<Value = String> {
    "[A-Za-z ]{1,30}".prop_map(|s| s)
}

fn arb_feedback() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), "[A-Za-z0-9 ]{1,50}".prop_map(|s| Some(s)),]
}

fn arb_spec_writing_session() -> impl Strategy<Value = SpecWritingSession> {
    (arb_spec_id(), arb_session_id()).prop_map(|(spec_id, session_id)| {
        let now = Utc::now();
        SpecWritingSession {
            id: session_id,
            spec_id,
            phase: SpecPhase::Discovery,
            conversation_history: vec![],
            approval_gates: ApprovalManager::initialize_gates(),
            created_at: now,
            updated_at: now,
        }
    })
}

// ============================================================================
// Property 7: Phase Progression Enforcement
// ============================================================================

proptest! {
    /// Property: Phase transitions SHALL only occur after explicit approval
    /// of the current phase, and phases SHALL progress sequentially
    /// (requirements → design → tasks).
    ///
    /// This property verifies that:
    /// 1. Transitions without approval are rejected
    /// 2. Transitions with approval succeed
    /// 3. Phases progress in the correct order
    #[test]
    fn prop_phase_transitions_require_approval(
        mut session in arb_spec_writing_session(),
        approver in arb_approver_name(),
        feedback in arb_feedback(),
    ) {
        // Property 7.1: Cannot transition without approval
        let result = ApprovalManager::transition_to_next_phase(&mut session);
        prop_assert!(
            result.is_err(),
            "Transition without approval should fail"
        );
        prop_assert_eq!(
            session.phase,
            SpecPhase::Discovery,
            "Phase should not change on failed transition"
        );

        // Property 7.2: Can transition after approval
        let approve_result = ApprovalManager::approve_phase(&mut session, &approver, feedback.clone());
        prop_assert!(approve_result.is_ok(), "Approval should succeed");

        let transition_result = ApprovalManager::transition_to_next_phase(&mut session);
        prop_assert!(
            transition_result.is_ok(),
            "Transition after approval should succeed"
        );
        prop_assert_eq!(
            session.phase,
            SpecPhase::Requirements,
            "Phase should transition to Requirements"
        );
    }

    /// Property: Phase progression SHALL be sequential
    ///
    /// This property verifies that phases progress in the correct order:
    /// Discovery → Requirements → Design → Tasks → Execution
    #[test]
    fn prop_phase_progression_is_sequential(
        mut session in arb_spec_writing_session(),
        approver in arb_approver_name(),
    ) {
        let expected_phases = vec![
            SpecPhase::Discovery,
            SpecPhase::Requirements,
            SpecPhase::Design,
            SpecPhase::Tasks,
            SpecPhase::Execution,
        ];

        for (i, expected_phase) in expected_phases.iter().enumerate() {
            // Property 7.3: Current phase matches expected phase
            prop_assert_eq!(
                session.phase,
                *expected_phase,
                "Phase at step {} should be {:?}",
                i,
                expected_phase
            );

            // Property 7.4: Approval succeeds for current phase
            let approve_result = ApprovalManager::approve_phase(&mut session, &approver, None);
            prop_assert!(
                approve_result.is_ok(),
                "Approval should succeed for phase {:?}",
                expected_phase
            );

            // Property 7.5: Transition succeeds (except for final phase)
            if i < expected_phases.len() - 1 {
                let transition_result = ApprovalManager::transition_to_next_phase(&mut session);
                prop_assert!(
                    transition_result.is_ok(),
                    "Transition should succeed from phase {:?}",
                    expected_phase
                );
            }
        }

        // Property 7.6: Cannot transition from final phase
        prop_assert_eq!(
            session.phase,
            SpecPhase::Execution,
            "Should be at final phase"
        );
        let result = ApprovalManager::transition_to_next_phase(&mut session);
        prop_assert!(
            result.is_err(),
            "Transition from final phase should fail"
        );
    }

    /// Property: Approval gates SHALL record timestamps and approver information
    ///
    /// This property verifies that approval gates correctly record:
    /// 1. Approval timestamp
    /// 2. Approver name
    /// 3. Feedback (if provided)
    #[test]
    fn prop_approval_gates_record_metadata(
        mut session in arb_spec_writing_session(),
        approver in arb_approver_name(),
        feedback in arb_feedback(),
    ) {
        let before = Utc::now();

        let result = ApprovalManager::approve_phase(&mut session, &approver, feedback.clone());
        prop_assert!(result.is_ok(), "Approval should succeed");

        let after = Utc::now();

        // Property 7.7: Approval gate is marked as approved
        let gate = ApprovalManager::get_phase_approval(&session, SpecPhase::Discovery);
        prop_assert!(gate.is_some(), "Approval gate should exist");

        let gate = gate.unwrap();
        prop_assert!(gate.approved, "Gate should be marked as approved");

        // Property 7.8: Approval timestamp is recorded
        prop_assert!(gate.approved_at.is_some(), "Approval timestamp should be recorded");
        let approved_at = gate.approved_at.unwrap();
        prop_assert!(
            approved_at >= before && approved_at <= after,
            "Approval timestamp should be within expected range"
        );

        // Property 7.9: Approver name is recorded
        prop_assert_eq!(
            &gate.approved_by,
            &Some(approver.clone()),
            "Approver name should be recorded"
        );

        // Property 7.10: Feedback is recorded if provided
        prop_assert_eq!(
            &gate.feedback,
            &feedback,
            "Feedback should be recorded as provided"
        );
    }

    /// Property: Session updated_at timestamp SHALL change on approval and transition
    ///
    /// This property verifies that the session's updated_at timestamp
    /// is updated when approvals and transitions occur.
    #[test]
    fn prop_session_timestamp_updates(
        mut session in arb_spec_writing_session(),
        approver in arb_approver_name(),
    ) {
        let original_updated_at = session.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Property 7.11: Session timestamp updates on approval
        ApprovalManager::approve_phase(&mut session, &approver, None).unwrap();
        prop_assert!(
            session.updated_at > original_updated_at,
            "Session timestamp should update on approval"
        );

        let updated_at_after_approval = session.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Property 7.12: Session timestamp updates on transition
        ApprovalManager::transition_to_next_phase(&mut session).unwrap();
        prop_assert!(
            session.updated_at > updated_at_after_approval,
            "Session timestamp should update on transition"
        );
    }

    /// Property: can_transition SHALL return correct status
    ///
    /// This property verifies that can_transition correctly indicates
    /// whether a transition is possible.
    #[test]
    fn prop_can_transition_status_is_accurate(
        mut session in arb_spec_writing_session(),
        approver in arb_approver_name(),
    ) {
        // Property 7.13: Cannot transition before approval
        prop_assert!(
            !ApprovalManager::can_transition(&session),
            "can_transition should return false before approval"
        );

        // Property 7.14: Can transition after approval
        ApprovalManager::approve_phase(&mut session, &approver, None).unwrap();
        prop_assert!(
            ApprovalManager::can_transition(&session),
            "can_transition should return true after approval"
        );

        // Property 7.15: Cannot transition from final phase
        session.phase = SpecPhase::Execution;
        ApprovalManager::approve_phase(&mut session, &approver, None).unwrap();
        prop_assert!(
            !ApprovalManager::can_transition(&session),
            "can_transition should return false at final phase"
        );
    }

    /// Property: are_phases_approved_up_to SHALL correctly report approval status
    ///
    /// This property verifies that are_phases_approved_up_to correctly
    /// reports whether all phases up to a target phase are approved.
    #[test]
    fn prop_phases_approved_up_to_is_accurate(
        mut session in arb_spec_writing_session(),
        approver in arb_approver_name(),
    ) {
        // Property 7.16: No phases approved initially
        prop_assert!(
            !ApprovalManager::are_phases_approved_up_to(&session, SpecPhase::Discovery),
            "No phases should be approved initially"
        );

        // Property 7.17: Discovery phase approved after approval
        ApprovalManager::approve_phase(&mut session, &approver, None).unwrap();
        prop_assert!(
            ApprovalManager::are_phases_approved_up_to(&session, SpecPhase::Discovery),
            "Discovery phase should be approved"
        );

        // Property 7.18: Requirements not approved yet
        prop_assert!(
            !ApprovalManager::are_phases_approved_up_to(&session, SpecPhase::Requirements),
            "Requirements phase should not be approved yet"
        );

        // Property 7.19: Transition to Requirements
        ApprovalManager::transition_to_next_phase(&mut session).unwrap();
        ApprovalManager::approve_phase(&mut session, &approver, None).unwrap();

        // Property 7.20: Both Discovery and Requirements approved
        prop_assert!(
            ApprovalManager::are_phases_approved_up_to(&session, SpecPhase::Requirements),
            "Both Discovery and Requirements should be approved"
        );

        // Property 7.21: Design not approved yet
        prop_assert!(
            !ApprovalManager::are_phases_approved_up_to(&session, SpecPhase::Design),
            "Design phase should not be approved yet"
        );
    }

    /// Property: Multiple approvals of the same phase SHALL update the gate
    ///
    /// This property verifies that approving the same phase multiple times
    /// updates the approval gate with the latest information.
    #[test]
    fn prop_multiple_approvals_update_gate(
        mut session in arb_spec_writing_session(),
        approver1 in arb_approver_name(),
        approver2 in arb_approver_name(),
    ) {
        // First approval
        ApprovalManager::approve_phase(&mut session, &approver1, Some("First approval".to_string())).unwrap();
        let gate1 = ApprovalManager::get_phase_approval(&session, SpecPhase::Discovery).unwrap();
        let approved_at1 = gate1.approved_at;

        // Small delay
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Second approval (same phase)
        ApprovalManager::approve_phase(&mut session, &approver2, Some("Second approval".to_string())).unwrap();
        let gate2 = ApprovalManager::get_phase_approval(&session, SpecPhase::Discovery).unwrap();
        let approved_at2 = gate2.approved_at;

        // Property 7.22: Second approval updates the gate
        prop_assert_eq!(
            &gate2.approved_by,
            &Some(approver2.clone()),
            "Approver should be updated to second approver"
        );

        prop_assert_eq!(
            &gate2.feedback,
            &Some("Second approval".to_string()),
            "Feedback should be updated"
        );

        // Property 7.23: Timestamp is updated
        prop_assert!(
            approved_at2 > approved_at1,
            "Approval timestamp should be updated"
        );
    }

    /// Property: All approval gates SHALL be initialized for new sessions
    ///
    /// This property verifies that new sessions have approval gates
    /// for all phases.
    #[test]
    fn prop_all_phases_have_gates(
        session in arb_spec_writing_session(),
    ) {
        let gates = ApprovalManager::get_all_approvals(&session);

        // Property 7.24: All 5 phases have gates
        prop_assert_eq!(
            gates.len(),
            5,
            "All 5 phases should have approval gates"
        );

        // Property 7.25: Each phase has exactly one gate
        let phases = vec![
            SpecPhase::Discovery,
            SpecPhase::Requirements,
            SpecPhase::Design,
            SpecPhase::Tasks,
            SpecPhase::Execution,
        ];

        for phase in phases {
            let count = gates.iter().filter(|g| g.phase == phase).count();
            prop_assert_eq!(
                count,
                1,
                "Phase {:?} should have exactly one gate",
                phase
            );
        }

        // Property 7.26: All gates start unapproved
        for gate in gates {
            prop_assert!(
                !gate.approved,
                "Gate for phase {:?} should start unapproved",
                gate.phase
            );
        }
    }
}
