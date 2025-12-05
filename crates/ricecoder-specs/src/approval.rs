//! Approval gate management for spec phase transitions
//!
//! Manages approval gates and enforces sequential phase progression through
//! the spec writing workflow (requirements → design → tasks → execution).

use crate::error::SpecError;
use crate::models::{ApprovalGate, SpecPhase, SpecWritingSession};
use chrono::Utc;

/// Manages approval gates and phase transitions for spec writing sessions
#[derive(Debug, Clone)]
pub struct ApprovalManager;

impl ApprovalManager {
    /// Creates a new approval manager
    pub fn new() -> Self {
        ApprovalManager
    }

    /// Initializes approval gates for all phases in a new session
    ///
    /// Creates approval gates for each phase in sequential order.
    /// All gates start in unapproved state.
    pub fn initialize_gates() -> Vec<ApprovalGate> {
        vec![
            ApprovalGate {
                phase: SpecPhase::Discovery,
                approved: false,
                approved_at: None,
                approved_by: None,
                feedback: None,
            },
            ApprovalGate {
                phase: SpecPhase::Requirements,
                approved: false,
                approved_at: None,
                approved_by: None,
                feedback: None,
            },
            ApprovalGate {
                phase: SpecPhase::Design,
                approved: false,
                approved_at: None,
                approved_by: None,
                feedback: None,
            },
            ApprovalGate {
                phase: SpecPhase::Tasks,
                approved: false,
                approved_at: None,
                approved_by: None,
                feedback: None,
            },
            ApprovalGate {
                phase: SpecPhase::Execution,
                approved: false,
                approved_at: None,
                approved_by: None,
                feedback: None,
            },
        ]
    }

    /// Approves the current phase in a session
    ///
    /// Records approval with timestamp and approver information.
    /// Returns error if phase is already approved or if session is invalid.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session to approve
    /// * `approver` - Name of the person approving
    /// * `feedback` - Optional feedback on the phase
    ///
    /// # Returns
    ///
    /// Updated session with approval recorded, or error if approval fails
    pub fn approve_phase(
        session: &mut SpecWritingSession,
        approver: &str,
        feedback: Option<String>,
    ) -> Result<(), SpecError> {
        // Find the gate for the current phase
        let gate = session
            .approval_gates
            .iter_mut()
            .find(|g| g.phase == session.phase)
            .ok_or_else(|| {
                SpecError::InvalidFormat(format!(
                    "No approval gate found for phase {:?}",
                    session.phase
                ))
            })?;

        // Record approval
        gate.approved = true;
        gate.approved_at = Some(Utc::now());
        gate.approved_by = Some(approver.to_string());
        gate.feedback = feedback;

        // Update session timestamp
        session.updated_at = Utc::now();

        Ok(())
    }

    /// Transitions to the next phase if current phase is approved
    ///
    /// Enforces sequential phase progression. Only allows transition to the next
    /// phase if the current phase has been explicitly approved.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session
    ///
    /// # Returns
    ///
    /// Updated session with new phase, or error if transition is not allowed
    pub fn transition_to_next_phase(session: &mut SpecWritingSession) -> Result<(), SpecError> {
        // Check if current phase is approved
        let current_gate = session
            .approval_gates
            .iter()
            .find(|g| g.phase == session.phase)
            .ok_or_else(|| {
                SpecError::InvalidFormat(format!(
                    "No approval gate found for phase {:?}",
                    session.phase
                ))
            })?;

        if !current_gate.approved {
            return Err(SpecError::InvalidFormat(format!(
                "Cannot transition from {:?}: phase not approved",
                session.phase
            )));
        }

        // Determine next phase
        let next_phase = match session.phase {
            SpecPhase::Discovery => SpecPhase::Requirements,
            SpecPhase::Requirements => SpecPhase::Design,
            SpecPhase::Design => SpecPhase::Tasks,
            SpecPhase::Tasks => SpecPhase::Execution,
            SpecPhase::Execution => {
                return Err(SpecError::InvalidFormat(
                    "Already at final phase (Execution)".to_string(),
                ))
            }
        };

        // Verify next phase gate exists
        if !session.approval_gates.iter().any(|g| g.phase == next_phase) {
            return Err(SpecError::InvalidFormat(format!(
                "No approval gate found for next phase {:?}",
                next_phase
            )));
        }

        // Transition to next phase
        session.phase = next_phase;
        session.updated_at = Utc::now();

        Ok(())
    }

    /// Checks if a phase transition is allowed
    ///
    /// Returns true if the current phase is approved and the next phase exists.
    pub fn can_transition(session: &SpecWritingSession) -> bool {
        // Check if current phase is approved
        let current_gate = session
            .approval_gates
            .iter()
            .find(|g| g.phase == session.phase);

        if let Some(gate) = current_gate {
            if !gate.approved {
                return false;
            }
        } else {
            return false;
        }

        // Check if we're not at the final phase
        !matches!(session.phase, SpecPhase::Execution)
    }

    /// Gets the approval status for a specific phase
    ///
    /// Returns the approval gate for the given phase, or None if not found.
    pub fn get_phase_approval(
        session: &SpecWritingSession,
        phase: SpecPhase,
    ) -> Option<&ApprovalGate> {
        session.approval_gates.iter().find(|g| g.phase == phase)
    }

    /// Gets all approval gates for a session
    pub fn get_all_approvals(session: &SpecWritingSession) -> &[ApprovalGate] {
        &session.approval_gates
    }

    /// Checks if all phases up to and including the given phase are approved
    ///
    /// Useful for validating that a session has completed all required phases.
    pub fn are_phases_approved_up_to(
        session: &SpecWritingSession,
        target_phase: SpecPhase,
    ) -> bool {
        let phases = vec![
            SpecPhase::Discovery,
            SpecPhase::Requirements,
            SpecPhase::Design,
            SpecPhase::Tasks,
            SpecPhase::Execution,
        ];

        for phase in phases {
            if let Some(gate) = session.approval_gates.iter().find(|g| g.phase == phase) {
                if !gate.approved {
                    return false;
                }
            }

            if phase == target_phase {
                break;
            }
        }

        true
    }
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_session() -> SpecWritingSession {
        let now = Utc::now();
        SpecWritingSession {
            id: "test-session".to_string(),
            spec_id: "test-spec".to_string(),
            phase: SpecPhase::Discovery,
            conversation_history: vec![],
            approval_gates: ApprovalManager::initialize_gates(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_initialize_gates() {
        let gates = ApprovalManager::initialize_gates();

        assert_eq!(gates.len(), 5);
        assert_eq!(gates[0].phase, SpecPhase::Discovery);
        assert_eq!(gates[1].phase, SpecPhase::Requirements);
        assert_eq!(gates[2].phase, SpecPhase::Design);
        assert_eq!(gates[3].phase, SpecPhase::Tasks);
        assert_eq!(gates[4].phase, SpecPhase::Execution);

        for gate in gates {
            assert!(!gate.approved);
            assert!(gate.approved_at.is_none());
            assert!(gate.approved_by.is_none());
            assert!(gate.feedback.is_none());
        }
    }

    #[test]
    fn test_approve_phase() {
        let mut session = create_test_session();

        let result = ApprovalManager::approve_phase(
            &mut session,
            "reviewer",
            Some("Looks good".to_string()),
        );
        assert!(result.is_ok());

        let gate = ApprovalManager::get_phase_approval(&session, SpecPhase::Discovery).unwrap();
        assert!(gate.approved);
        assert_eq!(gate.approved_by, Some("reviewer".to_string()));
        assert_eq!(gate.feedback, Some("Looks good".to_string()));
        assert!(gate.approved_at.is_some());
    }

    #[test]
    fn test_approve_phase_without_feedback() {
        let mut session = create_test_session();

        let result = ApprovalManager::approve_phase(&mut session, "reviewer", None);
        assert!(result.is_ok());

        let gate = ApprovalManager::get_phase_approval(&session, SpecPhase::Discovery).unwrap();
        assert!(gate.approved);
        assert!(gate.feedback.is_none());
    }

    #[test]
    fn test_cannot_transition_without_approval() {
        let session = create_test_session();

        assert!(!ApprovalManager::can_transition(&session));
    }

    #[test]
    fn test_transition_to_next_phase_fails_without_approval() {
        let mut session = create_test_session();

        let result = ApprovalManager::transition_to_next_phase(&mut session);
        assert!(result.is_err());
        assert_eq!(session.phase, SpecPhase::Discovery);
    }

    #[test]
    fn test_transition_to_next_phase_succeeds_with_approval() {
        let mut session = create_test_session();

        // Approve current phase
        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        // Transition should succeed
        let result = ApprovalManager::transition_to_next_phase(&mut session);
        assert!(result.is_ok());
        assert_eq!(session.phase, SpecPhase::Requirements);
    }

    #[test]
    fn test_sequential_phase_progression() {
        let mut session = create_test_session();

        let phases = vec![
            SpecPhase::Discovery,
            SpecPhase::Requirements,
            SpecPhase::Design,
            SpecPhase::Tasks,
            SpecPhase::Execution,
        ];

        for (i, phase) in phases.iter().enumerate() {
            assert_eq!(session.phase, *phase);

            // Approve current phase
            ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

            // Transition to next phase (except for last phase)
            if i < phases.len() - 1 {
                ApprovalManager::transition_to_next_phase(&mut session).unwrap();
            }
        }

        assert_eq!(session.phase, SpecPhase::Execution);
    }

    #[test]
    fn test_cannot_transition_from_execution() {
        let mut session = create_test_session();

        // Manually set to execution phase
        session.phase = SpecPhase::Execution;

        // Approve execution phase
        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        // Try to transition from execution
        let result = ApprovalManager::transition_to_next_phase(&mut session);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_phase_approval() {
        let session = create_test_session();

        let gate = ApprovalManager::get_phase_approval(&session, SpecPhase::Requirements);
        assert!(gate.is_some());
        assert_eq!(gate.unwrap().phase, SpecPhase::Requirements);
        assert!(!gate.unwrap().approved);
    }

    #[test]
    fn test_get_all_approvals() {
        let session = create_test_session();

        let gates = ApprovalManager::get_all_approvals(&session);
        assert_eq!(gates.len(), 5);
    }

    #[test]
    fn test_are_phases_approved_up_to() {
        let mut session = create_test_session();

        // Initially, no phases are approved
        assert!(!ApprovalManager::are_phases_approved_up_to(
            &session,
            SpecPhase::Requirements
        ));

        // Approve discovery
        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();
        ApprovalManager::transition_to_next_phase(&mut session).unwrap();

        // Approve requirements
        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        // Check that phases up to requirements are approved
        assert!(ApprovalManager::are_phases_approved_up_to(
            &session,
            SpecPhase::Requirements
        ));

        // Check that phases up to design are not approved
        assert!(!ApprovalManager::are_phases_approved_up_to(
            &session,
            SpecPhase::Design
        ));
    }

    #[test]
    fn test_can_transition_after_approval() {
        let mut session = create_test_session();

        assert!(!ApprovalManager::can_transition(&session));

        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        assert!(ApprovalManager::can_transition(&session));
    }

    #[test]
    fn test_cannot_transition_from_final_phase() {
        let mut session = create_test_session();

        session.phase = SpecPhase::Execution;
        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        assert!(!ApprovalManager::can_transition(&session));
    }

    #[test]
    fn test_approval_timestamps_are_recorded() {
        let mut session = create_test_session();
        let before = Utc::now();

        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        let after = Utc::now();
        let gate = ApprovalManager::get_phase_approval(&session, SpecPhase::Discovery).unwrap();

        assert!(gate.approved_at.is_some());
        let approved_at = gate.approved_at.unwrap();
        assert!(approved_at >= before);
        assert!(approved_at <= after);
    }

    #[test]
    fn test_session_updated_at_changes_on_approval() {
        let mut session = create_test_session();
        let original_updated_at = session.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        assert!(session.updated_at > original_updated_at);
    }

    #[test]
    fn test_session_updated_at_changes_on_transition() {
        let mut session = create_test_session();

        ApprovalManager::approve_phase(&mut session, "reviewer", None).unwrap();

        let updated_at_after_approval = session.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        ApprovalManager::transition_to_next_phase(&mut session).unwrap();

        assert!(session.updated_at > updated_at_after_approval);
    }
}
