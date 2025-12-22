//! AI-assisted spec writing with phase guidance and approval gates
//!
//! Manages AI-assisted spec creation through sequential phases (requirements → design → tasks)
//! with conversation history, gap identification, and integration with approval gates and validation.

use chrono::Utc;

use crate::{
    approval::ApprovalManager,
    error::SpecError,
    models::{
        ConversationMessage, MessageRole, Spec, SpecMetadata, SpecPhase, SpecStatus,
        SpecWritingSession, Steering,
    },
    steering::SteeringLoader,
    validation::ValidationEngine,
};

/// Manages AI-assisted spec writing with phase guidance and approval gates
#[derive(Debug, Clone)]
pub struct AISpecWriter;

/// Gap analysis result for a spec
#[derive(Debug, Clone)]
pub struct GapAnalysis {
    /// Missing sections in the spec
    pub missing_sections: Vec<String>,
    /// Incomplete sections
    pub incomplete_sections: Vec<String>,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

impl AISpecWriter {
    /// Creates a new AI spec writer
    pub fn new() -> Self {
        AISpecWriter
    }

    /// Initializes a new spec writing session
    ///
    /// Creates a new session with initial spec and approval gates.
    /// Session starts in Discovery phase.
    ///
    /// # Arguments
    ///
    /// * `spec_id` - Unique identifier for the spec
    /// * `spec_name` - Human-readable name for the spec
    ///
    /// # Returns
    ///
    /// New spec writing session ready for phase guidance
    pub fn initialize_session(spec_id: &str, spec_name: &str) -> SpecWritingSession {
        let now = Utc::now();
        let spec = Spec {
            id: spec_id.to_string(),
            name: spec_name.to_string(),
            version: "0.1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Discovery,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        SpecWritingSession {
            id: format!("session-{}", uuid::Uuid::new_v4()),
            spec_id: spec.id.clone(),
            phase: SpecPhase::Discovery,
            conversation_history: vec![],
            approval_gates: ApprovalManager::initialize_gates(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Adds a message to the conversation history
    ///
    /// Records user or assistant messages in the session for context.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session
    /// * `role` - Role of the message sender (User, Assistant, System)
    /// * `content` - Message content
    ///
    /// # Returns
    ///
    /// Updated session with message added
    pub fn add_message(
        session: &mut SpecWritingSession,
        role: MessageRole,
        content: &str,
    ) -> Result<(), SpecError> {
        let message = ConversationMessage {
            id: format!("msg-{}", uuid::Uuid::new_v4()),
            spec_id: session.spec_id.clone(),
            role,
            content: content.to_string(),
            timestamp: Utc::now(),
        };

        session.conversation_history.push(message);
        session.updated_at = Utc::now();

        Ok(())
    }

    /// Gets phase-specific guidance for the current phase
    ///
    /// Provides guidance on what to focus on in the current phase.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session
    ///
    /// # Returns
    ///
    /// Guidance text for the current phase
    pub fn get_phase_guidance(session: &SpecWritingSession) -> String {
        match session.phase {
            SpecPhase::Discovery => {
                "Discovery Phase: Research the problem space, identify user personas, define scope, and establish feature hierarchy. Focus on understanding the problem before designing solutions.".to_string()
            }
            SpecPhase::Requirements => {
                "Requirements Phase: Define what must be built using user stories and acceptance criteria. Write EARS-compliant requirements with clear success conditions.".to_string()
            }
            SpecPhase::Design => {
                "Design Phase: Create technical design and architecture to satisfy requirements. Define data models, algorithms, and integration points.".to_string()
            }
            SpecPhase::Tasks => {
                "Tasks Phase: Break design into hierarchical implementation tasks. Define dependencies, ordering, and exit criteria for each task.".to_string()
            }
            SpecPhase::Execution => {
                "Execution Phase: implement according to spec and validate against acceptance criteria. Track task completion and validate requirements are met.".to_string()
            }
        }
    }

    /// Analyzes gaps in the current spec
    ///
    /// Identifies missing sections, incomplete sections, and suggests improvements
    /// based on the current phase.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session
    /// * `spec` - The spec being written
    ///
    /// # Returns
    ///
    /// Gap analysis with missing sections and suggestions
    pub fn analyze_gaps(session: &SpecWritingSession, spec: &Spec) -> GapAnalysis {
        let mut missing_sections = Vec::new();
        let mut incomplete_sections = Vec::new();
        let mut suggestions = Vec::new();

        match session.phase {
            SpecPhase::Discovery => {
                if spec.requirements.is_empty() {
                    suggestions.push(
                        "Consider researching similar products and existing solutions".to_string(),
                    );
                }
                suggestions.push("Define feature hierarchy (MVP → Phase 2 → Phase 3)".to_string());
                suggestions.push("Document constraints and dependencies".to_string());
            }
            SpecPhase::Requirements => {
                if spec.requirements.is_empty() {
                    missing_sections.push("Requirements".to_string());
                    suggestions.push(
                        "Add at least one requirement with user story and acceptance criteria"
                            .to_string(),
                    );
                }

                for req in &spec.requirements {
                    if req.user_story.is_empty() {
                        incomplete_sections
                            .push(format!("Requirement {}: missing user story", req.id));
                    }
                    if req.acceptance_criteria.is_empty() {
                        incomplete_sections.push(format!(
                            "Requirement {}: missing acceptance criteria",
                            req.id
                        ));
                    }
                }

                suggestions
                    .push("Ensure each requirement has clear acceptance criteria".to_string());
                suggestions.push("Prioritize requirements (MUST/SHOULD/COULD)".to_string());
            }
            SpecPhase::Design => {
                if spec.design.is_none() {
                    missing_sections.push("Design".to_string());
                    suggestions
                        .push("Create design document with overview and architecture".to_string());
                } else if let Some(design) = &spec.design {
                    if design.overview.is_empty() {
                        incomplete_sections.push("Design: missing overview".to_string());
                    }
                    if design.architecture.is_empty() {
                        incomplete_sections.push("Design: missing architecture".to_string());
                    }
                    if design.components.is_empty() {
                        incomplete_sections.push("Design: missing components".to_string());
                    }
                    if design.correctness_properties.is_empty() {
                        incomplete_sections
                            .push("Design: missing correctness properties".to_string());
                    }
                }

                suggestions.push("Define data models and type definitions".to_string());
                suggestions.push("Specify algorithms and key logic".to_string());
                suggestions.push("Document integration points and dependencies".to_string());
            }
            SpecPhase::Tasks => {
                if spec.tasks.is_empty() {
                    missing_sections.push("Tasks".to_string());
                    suggestions.push("Break design into concrete, implementable tasks".to_string());
                }

                for task in &spec.tasks {
                    if task.description.is_empty() {
                        incomplete_sections.push(format!("Task {}: missing description", task.id));
                    }
                    if task.requirements.is_empty() {
                        incomplete_sections
                            .push(format!("Task {}: missing requirement links", task.id));
                    }
                }

                suggestions.push("Define clear dependencies between tasks".to_string());
                suggestions.push("Specify exit criteria for each task".to_string());
            }
            SpecPhase::Execution => {
                suggestions.push("Implement tasks in dependency order".to_string());
                suggestions.push("Validate each task against exit criteria".to_string());
                suggestions.push("Run acceptance tests against requirements".to_string());
            }
        }

        GapAnalysis {
            missing_sections,
            incomplete_sections,
            suggestions,
        }
    }

    /// Validates spec against EARS and INCOSE rules
    ///
    /// Runs validation engine to check EARS compliance and INCOSE semantic rules.
    ///
    /// # Arguments
    ///
    /// * `spec` - The spec to validate
    ///
    /// # Returns
    ///
    /// Ok if validation passes, Err with validation errors if it fails
    pub fn validate_spec(spec: &Spec) -> Result<(), SpecError> {
        ValidationEngine::validate(spec)
    }

    /// Requests approval for the current phase
    ///
    /// Records approval with timestamp and approver information.
    /// Prevents phase transitions without explicit approval.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session
    /// * `approver` - Name of the person approving
    /// * `feedback` - Optional feedback on the phase
    ///
    /// # Returns
    ///
    /// Ok if approval is recorded, Err if approval fails
    pub fn request_approval(
        session: &mut SpecWritingSession,
        approver: &str,
        feedback: Option<String>,
    ) -> Result<(), SpecError> {
        ApprovalManager::approve_phase(session, approver, feedback)
    }

    /// Transitions to the next phase
    ///
    /// Enforces sequential phase progression. Only allows transition if current phase
    /// has been explicitly approved.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session
    ///
    /// # Returns
    ///
    /// Ok if transition succeeds, Err if transition is not allowed
    pub fn transition_to_next_phase(session: &mut SpecWritingSession) -> Result<(), SpecError> {
        ApprovalManager::transition_to_next_phase(session)
    }

    /// Checks if phase transition is allowed
    ///
    /// Returns true if current phase is approved and next phase exists.
    pub fn can_transition(session: &SpecWritingSession) -> bool {
        ApprovalManager::can_transition(session)
    }

    /// Gets the conversation history for context
    ///
    /// Returns all messages in the session for AI context.
    pub fn get_conversation_history(session: &SpecWritingSession) -> &[ConversationMessage] {
        &session.conversation_history
    }

    /// Gets the current phase
    pub fn get_current_phase(session: &SpecWritingSession) -> SpecPhase {
        session.phase
    }

    /// Checks if all phases up to a target phase are approved
    ///
    /// Useful for validating that a session has completed required phases.
    pub fn are_phases_approved_up_to(
        session: &SpecWritingSession,
        target_phase: SpecPhase,
    ) -> bool {
        ApprovalManager::are_phases_approved_up_to(session, target_phase)
    }

    /// Builds an AI prompt with steering context included
    ///
    /// Generates a prompt for AI spec writing that includes steering rules,
    /// standards, and templates. Project steering takes precedence over global steering.
    ///
    /// # Arguments
    ///
    /// * `session` - The spec writing session
    /// * `global_steering` - Global steering document (workspace-level)
    /// * `project_steering` - Project steering document (project-level)
    /// * `user_input` - The user's input or question
    ///
    /// # Returns
    ///
    /// A formatted prompt string with steering context included
    pub fn build_prompt_with_steering_context(
        session: &SpecWritingSession,
        global_steering: &Steering,
        project_steering: &Steering,
        user_input: &str,
    ) -> Result<String, SpecError> {
        // Merge steering with project taking precedence
        let merged_steering = SteeringLoader::merge(global_steering, project_steering)?;

        // Build the prompt with steering context
        let mut prompt = String::new();

        // Add phase guidance
        let phase_guidance = Self::get_phase_guidance(session);
        prompt.push_str(&format!("## Phase Guidance\n{}\n\n", phase_guidance));

        // Add steering rules if any
        if !merged_steering.rules.is_empty() {
            prompt.push_str("## Steering Rules\n");
            for rule in &merged_steering.rules {
                prompt.push_str(&format!(
                    "- **{}**: {} (Pattern: {}, Action: {})\n",
                    rule.id, rule.description, rule.pattern, rule.action
                ));
            }
            prompt.push('\n');
        }

        // Add standards if any
        if !merged_steering.standards.is_empty() {
            prompt.push_str("## Standards\n");
            for standard in &merged_steering.standards {
                prompt.push_str(&format!(
                    "- **{}**: {}\n",
                    standard.id, standard.description
                ));
            }
            prompt.push('\n');
        }

        // Add templates if any
        if !merged_steering.templates.is_empty() {
            prompt.push_str("## Available Templates\n");
            for template in &merged_steering.templates {
                prompt.push_str(&format!("- **{}**: {}\n", template.id, template.path));
            }
            prompt.push('\n');
        }

        // Add conversation history for context
        if !session.conversation_history.is_empty() {
            prompt.push_str("## Conversation History\n");
            for msg in &session.conversation_history {
                let role_str = match msg.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant",
                    MessageRole::System => "System",
                };
                prompt.push_str(&format!("**{}**: {}\n", role_str, msg.content));
            }
            prompt.push('\n');
        }

        // Add the user's current input
        prompt.push_str(&format!("## Current Request\n{}\n", user_input));

        Ok(prompt)
    }

    /// Applies steering rules to generated specs
    ///
    /// Validates that generated specs conform to steering standards.
    /// Returns a list of violations if any steering rules are not followed.
    ///
    /// # Arguments
    ///
    /// * `spec` - The generated spec to validate
    /// * `steering` - The steering document with rules and standards
    ///
    /// # Returns
    ///
    /// Ok with list of violations (empty if all rules are followed),
    /// or Err if validation fails
    pub fn validate_spec_against_steering(
        spec: &Spec,
        steering: &Steering,
    ) -> Result<Vec<String>, SpecError> {
        let mut violations = Vec::new();

        // Check that spec has required metadata
        if spec.metadata.author.is_none() {
            violations.push("Spec missing author in metadata".to_string());
        }

        // Check that spec has a version
        if spec.version.is_empty() {
            violations.push("Spec missing version".to_string());
        }

        // Check that spec name follows naming standards
        // (This is a simple check; more sophisticated checks could be added)
        if spec.name.is_empty() {
            violations.push("Spec missing name".to_string());
        }

        // Check requirements phase has requirements
        if spec.metadata.phase == SpecPhase::Requirements && spec.requirements.is_empty() {
            violations.push("Requirements phase spec has no requirements".to_string());
        }

        // Check design phase has design
        if spec.metadata.phase == SpecPhase::Design && spec.design.is_none() {
            violations.push("Design phase spec has no design".to_string());
        }

        // Check tasks phase has tasks
        if spec.metadata.phase == SpecPhase::Tasks && spec.tasks.is_empty() {
            violations.push("Tasks phase spec has no tasks".to_string());
        }

        // Validate against steering standards
        for standard in &steering.standards {
            // Check that all requirements have acceptance criteria (common standard)
            if standard.id == "require-acceptance-criteria" {
                for req in &spec.requirements {
                    if req.acceptance_criteria.is_empty() {
                        violations.push(format!(
                            "Requirement {} violates standard {}: missing acceptance criteria",
                            req.id, standard.id
                        ));
                    }
                }
            }

            // Check that all tasks have descriptions (common standard)
            if standard.id == "require-task-descriptions" {
                for task in &spec.tasks {
                    if task.description.is_empty() {
                        violations.push(format!(
                            "Task {} violates standard {}: missing description",
                            task.id, standard.id
                        ));
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Gets steering context as a formatted string for display
    ///
    /// Useful for showing users what steering rules are active.
    ///
    /// # Arguments
    ///
    /// * `steering` - The steering document
    ///
    /// # Returns
    ///
    /// Formatted string representation of steering context
    pub fn format_steering_context(steering: &Steering) -> String {
        let mut output = String::new();

        if !steering.rules.is_empty() {
            output.push_str("### Steering Rules\n");
            for rule in &steering.rules {
                output.push_str(&format!(
                    "- `{}`: {} ({})\n",
                    rule.id, rule.description, rule.action
                ));
            }
            output.push('\n');
        }

        if !steering.standards.is_empty() {
            output.push_str("### Standards\n");
            for standard in &steering.standards {
                output.push_str(&format!("- `{}`: {}\n", standard.id, standard.description));
            }
            output.push('\n');
        }

        if !steering.templates.is_empty() {
            output.push_str("### Templates\n");
            for template in &steering.templates {
                output.push_str(&format!("- `{}`: {}\n", template.id, template.path));
            }
        }

        output
    }
}

impl Default for AISpecWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AcceptanceCriterion, Design, Priority, Requirement, Task};

    fn create_test_session() -> SpecWritingSession {
        AISpecWriter::initialize_session("test-spec", "Test Spec")
    }

    fn create_test_spec() -> Spec {
        let now = Utc::now();
        Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "0.1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Discovery,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        }
    }

    #[test]
    fn test_initialize_session() {
        let session = AISpecWriter::initialize_session("test-spec", "Test Spec");

        assert_eq!(session.spec_id, "test-spec");
        assert_eq!(session.phase, SpecPhase::Discovery);
        assert!(session.conversation_history.is_empty());
        assert_eq!(session.approval_gates.len(), 5);
    }

    #[test]
    fn test_add_message_user() {
        let mut session = create_test_session();

        let result =
            AISpecWriter::add_message(&mut session, MessageRole::User, "What should we build?");
        assert!(result.is_ok());
        assert_eq!(session.conversation_history.len(), 1);
        assert_eq!(session.conversation_history[0].role, MessageRole::User);
        assert_eq!(
            session.conversation_history[0].content,
            "What should we build?"
        );
    }

    #[test]
    fn test_add_message_assistant() {
        let mut session = create_test_session();

        AISpecWriter::add_message(&mut session, MessageRole::User, "What should we build?")
            .unwrap();
        AISpecWriter::add_message(
            &mut session,
            MessageRole::Assistant,
            "Let's build a task manager",
        )
        .unwrap();

        assert_eq!(session.conversation_history.len(), 2);
        assert_eq!(session.conversation_history[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_add_multiple_messages() {
        let mut session = create_test_session();

        for i in 0..5 {
            AISpecWriter::add_message(&mut session, MessageRole::User, &format!("Message {}", i))
                .unwrap();
        }

        assert_eq!(session.conversation_history.len(), 5);
    }

    #[test]
    fn test_get_phase_guidance_discovery() {
        let session = create_test_session();
        let guidance = AISpecWriter::get_phase_guidance(&session);

        assert!(guidance.contains("Discovery"));
        assert!(guidance.contains("problem space"));
    }

    #[test]
    fn test_get_phase_guidance_requirements() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Requirements;

        let guidance = AISpecWriter::get_phase_guidance(&session);

        assert!(guidance.contains("Requirements"));
        assert!(guidance.contains("user stories"));
    }

    #[test]
    fn test_get_phase_guidance_design() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Design;

        let guidance = AISpecWriter::get_phase_guidance(&session);

        assert!(guidance.contains("Design"));
        assert!(guidance.contains("architecture"));
    }

    #[test]
    fn test_get_phase_guidance_tasks() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Tasks;

        let guidance = AISpecWriter::get_phase_guidance(&session);

        assert!(guidance.contains("Tasks"));
        assert!(guidance.contains("implementation"));
    }

    #[test]
    fn test_get_phase_guidance_execution() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Execution;

        let guidance = AISpecWriter::get_phase_guidance(&session);

        assert!(guidance.contains("Execution"));
        assert!(guidance.contains("implement"));
    }

    #[test]
    fn test_analyze_gaps_discovery_empty() {
        let session = create_test_session();
        let spec = create_test_spec();

        let gaps = AISpecWriter::analyze_gaps(&session, &spec);

        assert!(!gaps.suggestions.is_empty());
        assert!(gaps.suggestions.iter().any(|s| s.contains("research")));
    }

    #[test]
    fn test_analyze_gaps_requirements_missing() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Requirements;
        let spec = create_test_spec();

        let gaps = AISpecWriter::analyze_gaps(&session, &spec);

        assert!(gaps.missing_sections.contains(&"Requirements".to_string()));
    }

    #[test]
    fn test_analyze_gaps_requirements_incomplete() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Requirements;

        let mut spec = create_test_spec();
        spec.requirements.push(Requirement {
            id: "REQ-1".to_string(),
            user_story: "".to_string(),
            acceptance_criteria: vec![],
            priority: Priority::Must,
        });

        let gaps = AISpecWriter::analyze_gaps(&session, &spec);

        assert!(!gaps.incomplete_sections.is_empty());
    }

    #[test]
    fn test_analyze_gaps_design_missing() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Design;
        let spec = create_test_spec();

        let gaps = AISpecWriter::analyze_gaps(&session, &spec);

        assert!(gaps.missing_sections.contains(&"Design".to_string()));
    }

    #[test]
    fn test_analyze_gaps_tasks_missing() {
        let mut session = create_test_session();
        session.phase = SpecPhase::Tasks;
        let spec = create_test_spec();

        let gaps = AISpecWriter::analyze_gaps(&session, &spec);

        assert!(gaps.missing_sections.contains(&"Tasks".to_string()));
    }

    #[test]
    fn test_request_approval() {
        let mut session = create_test_session();

        let result = AISpecWriter::request_approval(
            &mut session,
            "reviewer",
            Some("Looks good".to_string()),
        );
        assert!(result.is_ok());

        let gate = session
            .approval_gates
            .iter()
            .find(|g| g.phase == SpecPhase::Discovery)
            .unwrap();
        assert!(gate.approved);
    }

    #[test]
    fn test_can_transition_before_approval() {
        let session = create_test_session();

        assert!(!AISpecWriter::can_transition(&session));
    }

    #[test]
    fn test_can_transition_after_approval() {
        let mut session = create_test_session();

        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();

        assert!(AISpecWriter::can_transition(&session));
    }

    #[test]
    fn test_transition_to_next_phase() {
        let mut session = create_test_session();

        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();
        let result = AISpecWriter::transition_to_next_phase(&mut session);

        assert!(result.is_ok());
        assert_eq!(session.phase, SpecPhase::Requirements);
    }

    #[test]
    fn test_transition_fails_without_approval() {
        let mut session = create_test_session();

        let result = AISpecWriter::transition_to_next_phase(&mut session);

        assert!(result.is_err());
        assert_eq!(session.phase, SpecPhase::Discovery);
    }

    #[test]
    fn test_get_conversation_history() {
        let mut session = create_test_session();

        AISpecWriter::add_message(&mut session, MessageRole::User, "Message 1").unwrap();
        AISpecWriter::add_message(&mut session, MessageRole::Assistant, "Message 2").unwrap();

        let history = AISpecWriter::get_conversation_history(&session);

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, MessageRole::User);
        assert_eq!(history[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_get_current_phase() {
        let mut session = create_test_session();

        assert_eq!(
            AISpecWriter::get_current_phase(&session),
            SpecPhase::Discovery
        );

        session.phase = SpecPhase::Requirements;

        assert_eq!(
            AISpecWriter::get_current_phase(&session),
            SpecPhase::Requirements
        );
    }

    #[test]
    fn test_are_phases_approved_up_to() {
        let mut session = create_test_session();

        assert!(!AISpecWriter::are_phases_approved_up_to(
            &session,
            SpecPhase::Requirements
        ));

        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();
        AISpecWriter::transition_to_next_phase(&mut session).unwrap();
        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();

        assert!(AISpecWriter::are_phases_approved_up_to(
            &session,
            SpecPhase::Requirements
        ));
    }

    #[test]
    fn test_sequential_phase_workflow() {
        let mut session = create_test_session();
        let mut spec = create_test_spec();

        // Discovery phase
        assert_eq!(session.phase, SpecPhase::Discovery);
        let guidance = AISpecWriter::get_phase_guidance(&session);
        assert!(guidance.contains("Discovery"));

        AISpecWriter::add_message(
            &mut session,
            MessageRole::User,
            "Let's build a task manager",
        )
        .unwrap();
        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();
        AISpecWriter::transition_to_next_phase(&mut session).unwrap();

        // Requirements phase
        assert_eq!(session.phase, SpecPhase::Requirements);
        spec.requirements.push(Requirement {
            id: "REQ-1".to_string(),
            user_story: "As a user, I want to create tasks".to_string(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1.1".to_string(),
                when: "user enters task".to_string(),
                then: "task is added".to_string(),
            }],
            priority: Priority::Must,
        });

        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();
        AISpecWriter::transition_to_next_phase(&mut session).unwrap();

        // Design phase
        assert_eq!(session.phase, SpecPhase::Design);
        spec.design = Some(Design {
            overview: "Task management system".to_string(),
            architecture: "Layered architecture".to_string(),
            components: vec![],
            data_models: vec![],
            correctness_properties: vec![],
        });

        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();
        AISpecWriter::transition_to_next_phase(&mut session).unwrap();

        // Tasks phase
        assert_eq!(session.phase, SpecPhase::Tasks);
        spec.tasks.push(Task {
            id: "1".to_string(),
            description: "Implement task manager".to_string(),
            subtasks: vec![],
            requirements: vec!["REQ-1".to_string()],
            status: crate::models::TaskStatus::NotStarted,
            optional: false,
        });

        AISpecWriter::request_approval(&mut session, "reviewer", None).unwrap();
        AISpecWriter::transition_to_next_phase(&mut session).unwrap();

        // Execution phase
        assert_eq!(session.phase, SpecPhase::Execution);
    }

    #[test]
    fn test_conversation_history_preserved() {
        let mut session = create_test_session();

        for i in 0..10 {
            AISpecWriter::add_message(&mut session, MessageRole::User, &format!("Message {}", i))
                .unwrap();
        }

        let history = AISpecWriter::get_conversation_history(&session);
        assert_eq!(history.len(), 10);

        for (i, msg) in history.iter().enumerate() {
            assert_eq!(msg.content, format!("Message {}", i));
            assert!(msg.timestamp <= Utc::now());
        }
    }

    #[test]
    fn test_gap_analysis_suggestions_vary_by_phase() {
        let spec = create_test_spec();

        let mut session = create_test_session();
        session.phase = SpecPhase::Discovery;
        let discovery_gaps = AISpecWriter::analyze_gaps(&session, &spec);

        session.phase = SpecPhase::Requirements;
        let requirements_gaps = AISpecWriter::analyze_gaps(&session, &spec);

        session.phase = SpecPhase::Design;
        let design_gaps = AISpecWriter::analyze_gaps(&session, &spec);

        // Each phase should have different suggestions
        assert_ne!(discovery_gaps.suggestions, requirements_gaps.suggestions);
        assert_ne!(requirements_gaps.suggestions, design_gaps.suggestions);
    }

    #[test]
    fn test_build_prompt_with_steering_context() {
        use crate::models::SteeringRule;

        let session = create_test_session();
        let global_steering = Steering {
            rules: vec![SteeringRule {
                id: "global-rule".to_string(),
                description: "Global rule".to_string(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let project_steering = Steering {
            rules: vec![SteeringRule {
                id: "project-rule".to_string(),
                description: "Project rule".to_string(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let result = AISpecWriter::build_prompt_with_steering_context(
            &session,
            &global_steering,
            &project_steering,
            "Create a task manager",
        );

        assert!(result.is_ok());
        let prompt = result.unwrap();

        // Verify prompt contains steering context
        assert!(prompt.contains("Phase Guidance"));
        assert!(prompt.contains("Steering Rules"));
        assert!(prompt.contains("global-rule"));
        assert!(prompt.contains("project-rule"));
        assert!(prompt.contains("Create a task manager"));
    }

    #[test]
    fn test_build_prompt_with_steering_context_includes_conversation() {
        let mut session = create_test_session();
        AISpecWriter::add_message(&mut session, MessageRole::User, "What should we build?")
            .unwrap();
        AISpecWriter::add_message(
            &mut session,
            MessageRole::Assistant,
            "Let's build a task manager",
        )
        .unwrap();

        let global_steering = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let project_steering = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = AISpecWriter::build_prompt_with_steering_context(
            &session,
            &global_steering,
            &project_steering,
            "Continue with requirements",
        );

        assert!(result.is_ok());
        let prompt = result.unwrap();

        // Verify conversation history is included
        assert!(prompt.contains("Conversation History"));
        assert!(prompt.contains("What should we build?"));
        assert!(prompt.contains("Let's build a task manager"));
    }

    #[test]
    fn test_build_prompt_with_steering_context_project_precedence() {
        use crate::models::SteeringRule;

        let session = create_test_session();
        let global_steering = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: "Global version".to_string(),
                pattern: "global".to_string(),
                action: "warn".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let project_steering = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: "Project version".to_string(),
                pattern: "project".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let result = AISpecWriter::build_prompt_with_steering_context(
            &session,
            &global_steering,
            &project_steering,
            "Test",
        );

        assert!(result.is_ok());
        let prompt = result.unwrap();

        // Verify project version takes precedence
        assert!(prompt.contains("Project version"));
        assert!(prompt.contains("project"));
        assert!(!prompt.contains("Global version"));
    }

    #[test]
    fn test_validate_spec_against_steering_valid() {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![Requirement {
                id: "REQ-1".to_string(),
                user_story: "As a user".to_string(),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id: "AC-1.1".to_string(),
                    when: "when".to_string(),
                    then: "then".to_string(),
                }],
                priority: Priority::Must,
            }],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let steering = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = AISpecWriter::validate_spec_against_steering(&spec, &steering);
        assert!(result.is_ok());
        let violations = result.unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_validate_spec_against_steering_missing_author() {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Discovery,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let steering = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = AISpecWriter::validate_spec_against_steering(&spec, &steering);
        assert!(result.is_ok());
        let violations = result.unwrap();
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("author")));
    }

    #[test]
    fn test_validate_spec_against_steering_requirements_phase_no_requirements() {
        let spec = Spec {
            id: "test".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let steering = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = AISpecWriter::validate_spec_against_steering(&spec, &steering);
        assert!(result.is_ok());
        let violations = result.unwrap();
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("Requirements phase")));
    }

    #[test]
    fn test_format_steering_context() {
        use crate::models::{Standard, SteeringRule, TemplateRef};

        let steering = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: "Use snake_case".to_string(),
                pattern: "^[a-z_]+$".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![Standard {
                id: "std-1".to_string(),
                description: "Test all public APIs".to_string(),
            }],
            templates: vec![TemplateRef {
                id: "tpl-1".to_string(),
                path: "templates/entity.rs".to_string(),
            }],
        };

        let output = AISpecWriter::format_steering_context(&steering);

        assert!(output.contains("Steering Rules"));
        assert!(output.contains("rule-1"));
        assert!(output.contains("Standards"));
        assert!(output.contains("std-1"));
        assert!(output.contains("Templates"));
        assert!(output.contains("tpl-1"));
    }

    #[test]
    fn test_format_steering_context_empty() {
        let steering = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let output = AISpecWriter::format_steering_context(&steering);

        // Should be empty or minimal
        assert!(output.is_empty() || output.trim().is_empty());
    }
}
